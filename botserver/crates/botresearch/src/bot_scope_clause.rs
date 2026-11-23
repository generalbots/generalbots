use super::*;
use botlib::kb_scope::KbTenantScope;

/// A research bot paired with the tenant branch that owns it, resolved from
/// the caller's verified claims rather than from a process-global default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ResearchScope {
    pub bot_id: uuid::Uuid,
    pub branch_id: uuid::Uuid,
}

/// Pure authorization of a resolved bot against the caller's verified branch.
/// A bot may only be used when the caller branch is known and matches the
/// bot's owner; an unverifiable or mismatched caller is always denied so no
/// knowledge-base row can cross tenants.
pub fn authorize_scope(
    scope: ResearchScope,
    caller_branch: Option<uuid::Uuid>,
) -> Option<ResearchScope> {
    match caller_branch {
        Some(branch) if branch == scope.branch_id => Some(scope),
        _ => None,
    }
}

/// Resolves the caller's research scope exclusively from verified JWT claims.
///
/// The bot is selected by the signed `branch_id` claim, never by the
/// process-global `is_default_for_branch` lookup: every branch has a default
/// bot, so a global lookup let one tenant read another tenant's knowledge
/// base. Returns `None` when the token is missing/invalid or when that branch
/// has no default bot; callers MUST deny the request on `None`.
pub fn resolve_research_scope(
    pool: &DbPool,
    headers: &axum::http::HeaderMap,
) -> Option<ResearchScope> {
    let caller_branch = botsecurity_core::tenant::branch_from_claims(headers)?;
    let mut conn = pool.get().ok()?;

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct ScopeRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: uuid::Uuid,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        branch_id: uuid::Uuid,
    }

    let row = diesel::sql_query(
        "SELECT id, branch_id FROM bots \
         WHERE branch_id = $1 AND is_default_for_branch = TRUE \
         ORDER BY created_at ASC LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(caller_branch)
    .get_result::<ScopeRow>(&mut conn)
    .ok()?;

    authorize_scope(
        ResearchScope {
            bot_id: row.id,
            branch_id: row.branch_id,
        },
        Some(caller_branch),
    )
}

/// Builds the SQL fragment that constrains a KB query to the caller's tenant.
/// `scope` is `None` only when the caller's tenant could not be verified, in
/// which case the fragment denies every row. An empty fragment would expose
/// the whole knowledge base and is never returned. The bot id is a validated
/// `Uuid`, so its display form cannot inject SQL.
pub fn bot_scope_clause(scope: &Option<ResearchScope>) -> String {
    match scope {
        Some(s) => format!(" AND bot_id = '{}'", s.bot_id),
        None => " AND 1 = 0".to_string(),
    }
}

pub async fn handle_source_counts<S: ResearchState>(
    State(state): State<Arc<S>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let conn = state.db_pool().clone();
    let scope = bot_scope_clause(&state.resolve_scope(&headers));

    let (all, web) = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return (0i64, 0i64);
            }
        };

        let total: i64 = diesel::sql_query(format!(
            "SELECT COUNT(*) AS n FROM kb_documents WHERE 1 = 1{}",
            scope
        ))
        .get_result::<SourceCountRow>(&mut db_conn)
        .map(|r| r.n)
        .unwrap_or(0);

        (total, 0i64)
    })
    .await
    .unwrap_or((0, 0));

    let docs = all.saturating_sub(web);
    Json(serde_json::json!({
        "all": all,
        "web": web,
        "docs": docs,
        "kb": all,
    }))
}
pub async fn handle_list_collections<S: ResearchState>(
    State(state): State<Arc<S>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let conn = state.db_pool().clone();
    let scope = bot_scope_clause(&state.resolve_scope(&headers));

    let collections = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        let result: Result<Vec<CollectionRow>, _> = diesel::sql_query(format!(
            "SELECT id, name, folder_path, document_count::bigint AS document_count FROM kb_collections WHERE 1 = 1{} ORDER BY name ASC",
            scope
        ))
        .load(&mut db_conn);

        match result {
            Ok(colls) => colls,
            Err(e) => {
                log::error!("Failed to load research collections: {}", e);
                Vec::new()
            }
        }
    })
    .await
    .unwrap_or_default();

    let mut html = String::new();

    for c in &collections {
        html.push_str("<div class=\"collection-item\" data-id=\"");
        html.push_str(&html_escape(&c.id.to_string()));
        html.push_str("\">");
        html.push_str("<div class=\"collection-icon\"></div>");
        html.push_str("<div class=\"collection-info\">");
        html.push_str("<span class=\"collection-name\">");
        html.push_str(&html_escape(&c.name));
        html.push_str("</span>");
        html.push_str("<span class=\"collection-desc\">");
        html.push_str(&html_escape(&c.folder_path));
        html.push_str(" · ");
        html.push_str(&c.document_count.to_string());
        html.push_str(" documents</span>");
        html.push_str("</div>");
        html.push_str("<button class=\"btn-icon-sm\" hx-get=\"/api/ui/research/collections/");
        html.push_str(&html_escape(&c.id.to_string()));
        html.push_str("\" hx-target=\"#main-results\">");
        html.push_str("<svg width=\"16\" height=\"16\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\"><polyline points=\"9 18 15 12 9 6\"></polyline></svg>");
        html.push_str("</button>");
        html.push_str("</div>");
    }

    if collections.is_empty() {
        html.push_str("<div class=\"empty-state\">");
        html.push_str("<p>No collections yet</p>");
        html.push_str("<p class=\"hint\">Create a collection to organize your knowledge base</p>");
        html.push_str("</div>");
    }

    Html(html)
}
pub async fn handle_create_collection<S: ResearchState>(
    State(state): State<Arc<S>>,
    headers: axum::http::HeaderMap,
    Json(payload): Json<NewCollectionRequest>,
) -> impl IntoResponse {
    let Some(scope) = state.resolve_scope(&headers) else {
        return Html(String::new());
    };
    let bot_id = scope.bot_id;
    // The friendly name is caller-supplied; the bot/branch pair is not. Derive
    // the physical collection name from the verified scope so two tenants can
    // never collide on the same `kb_<name>` collection (issue #2818).
    let Some(kb_scope) = KbTenantScope::new(bot_id, scope.branch_id) else {
        log::warn!("Refusing KB collection creation without a verified tenant scope");
        return Html(String::new());
    };
    let conn = state.db_pool().clone();
    let id = uuid::Uuid::new_v4();
    let name = payload.name.clone();
    let description = payload.description.unwrap_or_default();
    let qdrant_collection = kb_scope.collection_name(&name);

    let id_clone = id;
    let name_clone = name.clone();
    let folder_clone = description.clone();
    let folder_for_closure = folder_clone.clone();

    let _ = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return;
            }
        };

        let _ = diesel::sql_query(
            "INSERT INTO kb_collections (id, bot_id, name, folder_path, qdrant_collection, document_count) VALUES ($1, $2, $3, $4, $5, 0)",
        )
        .bind::<diesel::sql_types::Uuid, _>(&id_clone)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(Some(bot_id))
        .bind::<diesel::sql_types::Text, _>(&name)
        .bind::<diesel::sql_types::Text, _>(&folder_for_closure)
        .bind::<diesel::sql_types::Text, _>(&qdrant_collection)
        .execute(&mut db_conn);
    })
    .await;

    let mut html = String::new();
    html.push_str("<div class=\"collection-item new-item\" data-id=\"");
    html.push_str(&html_escape(&id_clone.to_string()));
    html.push_str("\">");
    html.push_str("<div class=\"collection-icon\"></div>");
    html.push_str("<div class=\"collection-info\">");
    html.push_str("<span class=\"collection-name\">");
    html.push_str(&html_escape(&name_clone));
    html.push_str("</span>");
    html.push_str("<span class=\"collection-desc\">");
    html.push_str(&html_escape(&folder_clone));
    html.push_str("</span>");
    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}
pub async fn handle_get_collection<S: ResearchState>(
    State(state): State<Arc<S>>,
    headers: axum::http::HeaderMap,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let conn = state.db_pool().clone();
    let scope = bot_scope_clause(&state.resolve_scope(&headers));

    let documents = tokio::task::spawn_blocking(move || {
        let mut db_conn = match conn.get() {
            Ok(c) => c,
            Err(e) => {
                log::error!("DB connection error: {}", e);
                return Vec::new();
            }
        };

        let collection_id = match uuid::Uuid::parse_str(&id) {
            Ok(u) => u.to_string(),
            Err(_) => return Vec::new(),
        };

        diesel::sql_query(format!(
            "SELECT id, file_path, collection_name, metadata FROM kb_documents WHERE collection_name = $1{} ORDER BY file_path ASC LIMIT 50",
            scope
        ))
        .bind::<diesel::sql_types::Text, _>(&collection_id)
        .load::<KbDocumentRow>(&mut db_conn)
        .unwrap_or_default()
    })
    .await
    .unwrap_or_default();

    let mut html = String::new();
    html.push_str("<div class=\"collection-results\">");
    html.push_str("<div class=\"results-header\">");
    html.push_str("<h3>Collection Contents</h3>");
    html.push_str("<span class=\"result-count\">");
    html.push_str(&documents.len().to_string());
    html.push_str(" documents</span>");
    html.push_str("</div>");
    html.push_str("<div class=\"results-list\">");

    if documents.is_empty() {
        html.push_str("<div class=\"empty-state\">");
        html.push_str("<p>No documents in this collection</p>");
        html.push_str("<p class=\"hint\">Add documents to build your knowledge base</p>");
        html.push_str("</div>");
    } else {
        for doc in &documents {
            html.push_str(&format_search_result(doc));
        }
    }

    html.push_str("</div>");
    html.push_str("</div>");

    Html(html)
}

#[cfg(test)]
mod tests {
    use super::*;

    const BOT_A: &str = "11111111-1111-4111-8111-111111111111";
    const BOT_B: &str = "22222222-2222-4222-8222-222222222222";
    const BRANCH_A: &str = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
    const BRANCH_B: &str = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";

    fn scope(bot: &str, branch: &str) -> ResearchScope {
        ResearchScope {
            bot_id: uuid::Uuid::parse_str(bot).unwrap(),
            branch_id: uuid::Uuid::parse_str(branch).unwrap(),
        }
    }

    #[test]
    fn verified_matching_branch_is_authorized() {
        let owner = scope(BOT_A, BRANCH_A);
        let caller = uuid::Uuid::parse_str(BRANCH_A).unwrap();
        assert_eq!(authorize_scope(owner, Some(caller)), Some(owner));
    }

    #[test]
    fn mismatched_branch_is_denied() {
        let owner = scope(BOT_A, BRANCH_A);
        let other = uuid::Uuid::parse_str(BRANCH_B).unwrap();
        assert_eq!(authorize_scope(owner, Some(other)), None);
    }

    #[test]
    fn unverified_caller_is_denied() {
        let owner = scope(BOT_A, BRANCH_A);
        assert_eq!(authorize_scope(owner, None), None);
    }

    #[test]
    fn unverified_caller_never_yields_an_unscoped_filter() {
        let filter = bot_scope_clause(&None);
        assert!(!filter.is_empty(), "an empty filter exposes every tenant");
        assert!(
            filter.contains("1 = 0"),
            "denied filter must match no rows: {filter}"
        );
        assert!(!filter.contains("bot_id"), "no bot id to scope by: {filter}");
    }

    #[test]
    fn verified_scope_filters_by_its_resolved_bot() {
        let owner = scope(BOT_B, BRANCH_B);
        let filter = bot_scope_clause(&Some(owner));
        assert_eq!(filter, format!(" AND bot_id = '{BOT_B}'"));
    }

    #[test]
    fn cross_tenant_bot_cannot_be_used_to_build_a_filter() {
        let owner = scope(BOT_A, BRANCH_A);
        let other_branch = uuid::Uuid::parse_str(BRANCH_B).unwrap();
        let denied = authorize_scope(owner, Some(other_branch));
        assert_eq!(bot_scope_clause(&denied), " AND 1 = 0");
    }

    #[test]
    fn identical_friendly_names_never_share_a_collection_or_filter() {
        let tenant_a = scope(BOT_A, BRANCH_A);
        let tenant_b = scope(BOT_B, BRANCH_B);
        let scope_a = KbTenantScope::new(tenant_a.bot_id, tenant_a.branch_id).unwrap();
        let scope_b = KbTenantScope::new(tenant_b.bot_id, tenant_b.branch_id).unwrap();

        let collection_a = scope_a.collection_name("hr");
        let collection_b = scope_b.collection_name("hr");
        assert_ne!(collection_a, collection_b, "tenant collections must not collide");
        assert!(collection_a.contains(BOT_A) && collection_a.contains(BRANCH_A));
        assert!(collection_b.contains(BOT_B) && collection_b.contains(BRANCH_B));
        assert_ne!(scope_a.filter(), scope_b.filter());
        assert_eq!(scope_a.filter()["must"][0]["match"]["value"], BOT_A);
        assert_eq!(scope_b.filter()["must"][0]["match"]["value"], BOT_B);

        let unverified = authorize_scope(tenant_a, Some(uuid::Uuid::parse_str(BRANCH_B).unwrap()));
        assert!(unverified.is_none());
    }
}
