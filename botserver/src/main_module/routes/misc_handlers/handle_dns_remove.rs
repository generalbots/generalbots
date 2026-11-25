use super::*;
use crate::security::auth_api::types::AuthenticatedUser;
use axum::response::Response;
use axum::Extension;

/// POST /api/dns/remove
///
/// Deletes a record only when the caller controls the zone its hostname
/// belongs to; otherwise the request is rejected with 403.
pub async fn handle_dns_remove(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    Form(form): Form<DnsRemoveRequest>,
) -> Response {
    use diesel::prelude::*;

    let id = form.id.and_then(|s| Uuid::parse_str(&s).ok());
    let Some(id) = id else {
        return Html("<div class=\"dns-error\">Invalid record id</div>".to_string()).into_response();
    };

    let mut conn = match pool_conn(&state) {
        Ok(c) => c,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!(
                    "<div class=\"dns-error\">{}</div>",
                    html_escape(&e.1)
                )),
            )
                .into_response()
        }
    };

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct HostnameRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        hostname: String,
    }

    let record: Option<HostnameRow> =
        match diesel::sql_query("SELECT hostname FROM dns_records WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(id)
            .get_result(&mut conn)
        {
            Ok(row) => Some(row),
            Err(diesel::result::Error::NotFound) => None,
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Html(format!(
                        "<div class=\"dns-error\">{}</div>",
                        html_escape(&e.to_string())
                    )),
                )
                    .into_response()
            }
        };
    let Some(record) = record else {
        return Html("<div class=\"dns-error\">Record not found</div>".to_string()).into_response();
    };

    let owns = match caller_owns_hostname(&mut conn, &user, &record.hostname) {
        Ok(owns) => owns,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Html(format!(
                    "<div class=\"dns-error\">{}</div>",
                    html_escape(&e)
                )),
            )
                .into_response()
        }
    };
    if !owns {
        return (
            StatusCode::FORBIDDEN,
            Html("<div class=\"dns-error\">You do not control this DNS zone</div>".to_string()),
        )
            .into_response();
    }

    if let Err(e) = diesel::sql_query("DELETE FROM dns_records WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(id)
        .execute(&mut conn)
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Html(format!(
                "<div class=\"dns-error\">{}</div>",
                html_escape(&e.to_string())
            )),
        )
            .into_response();
    }

    Html("<div class=\"dns-success\">DNS record removed</div>".to_string()).into_response()
}

// ---------------------------------------------------------------------------
// User orgs: /api/user/current-org, /api/user/organizations
// ---------------------------------------------------------------------------
pub async fn handle_user_current_org(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    use diesel::prelude::*;

    let mut conn = pool_conn(&state)?;

    // Honor the persisted selection made via PUT /api/user/current-org.
    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct PrefRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        value: String,
    }
    let saved: Option<PrefRow> = diesel::sql_query(
        "SELECT preference_value::text AS value FROM user_preferences \
         WHERE preference_key = 'current_org_id' ORDER BY updated_at DESC LIMIT 1",
    )
    .get_result(&mut conn)
    .ok();
    if let Some(p) = saved {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&p.value) {
            if let Some(id_str) = parsed
                .as_str()
                .or_else(|| parsed.get("org_id").and_then(|v| v.as_str()))
            {
                if let Ok(org_id) = Uuid::parse_str(id_str) {
                    #[derive(diesel::QueryableByName)]
                    #[diesel(check_for_backend(diesel::pg::Pg))]
                    struct OrgRow {
                        #[diesel(sql_type = diesel::sql_types::Uuid)]
                        id: Uuid,
                        #[diesel(sql_type = diesel::sql_types::Text)]
                        name: String,
                    }
                    let row: Option<OrgRow> = diesel::sql_query(
                        "SELECT org_id AS id, name FROM organizations WHERE org_id = $1 LIMIT 1",
                    )
                    .bind::<diesel::sql_types::Uuid, _>(org_id)
                    .get_result(&mut conn)
                    .ok();
                    if let Some(r) = row {
                        return Ok(Json(serde_json::json!({ "id": r.id, "name": r.name })));
                    }
                }
            }
        }
    }

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct OrgRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }

    let row: Option<OrgRow> = diesel::sql_query(
        "SELECT org_id AS id, name FROM organizations ORDER BY created_at ASC LIMIT 1",
    )
    .get_result(&mut conn)
    .ok();

    match row {
        Some(r) => Ok(Json(serde_json::json!({ "id": r.id, "name": r.name }))),
        None => Ok(Json(
            serde_json::json!({ "id": Uuid::nil(), "name": "Default Organization" }),
        )),
    }
}
/// Pure membership-scope predicate (#2794): an organization may appear in the
/// caller's dropdown only when its id is in the caller's verified
/// `user_organizations` membership set. No I/O, so it is unit-tested with two
/// independent tenants.
fn org_in_caller_membership(member_org_ids: &[Uuid], org_id: Uuid) -> bool {
    member_org_ids.contains(&org_id)
}

pub async fn handle_user_organizations(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Result<Html<String>, (StatusCode, String)> {
    use diesel::prelude::*;

    // #2794: the caller identity comes only from the auth-middleware principal
    // (never from a query string or header). Membership is resolved by user_id,
    // so an anonymous/nil principal has an empty set and sees nothing.
    let caller_id = user.user_id;

    let mut conn = pool_conn(&state)?;

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct OrgIdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: Uuid,
    }

    let member_org_ids: Vec<Uuid> =
        diesel::sql_query("SELECT org_id FROM user_organizations WHERE user_id = $1")
            .bind::<diesel::sql_types::Uuid, _>(caller_id)
            .load::<OrgIdRow>(&mut conn)
            .map_err(|e| {
                log::error!("user organizations: membership lookup failed: {e}");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Unable to load organizations".to_string(),
                )
            })?
            .into_iter()
            .map(|r| r.org_id)
            .collect();

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct OrgRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }

    // Scope at the SQL boundary as well: only member organizations are ever
    // read, so a foreign org cannot be enumerated even if the in-Rust check
    // regresses. No LIMIT before the scope check for the same reason.
    let rows: Vec<OrgRow> = diesel::sql_query(
        "SELECT org_id AS id, name FROM organizations \
         WHERE org_id = ANY($1) ORDER BY created_at ASC",
    )
    .bind::<diesel::sql_types::Array<diesel::sql_types::Uuid>, _>(member_org_ids.clone())
    .load(&mut conn)
    .map_err(|e| {
        log::error!("user organizations: organization lookup failed: {e}");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Unable to load organizations".to_string(),
        )
    })?;

    let mut items = String::new();
    for r in rows
        .into_iter()
        .filter(|r| org_in_caller_membership(&member_org_ids, r.id))
        .take(50)
    {
        items.push_str(&format!(
            r##"<div class="org-dropdown-item" onclick="selectOrganization('{id}', '{name}', 'Member')">
    <span class="org-item-name">{name}</span>
</div>"##,
            id = r.id,
            name = html_escape(&r.name),
        ));
    }

    Ok(Html(items))
}
/// PUT /api/user/current-org
///
/// Persists the user's selected organization in `user_preferences`
/// (key `current_org_id`). The user is resolved from the bearer session —
/// never from client input.
pub async fn handle_user_current_org_put(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Json(payload): axum::extract::Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    use diesel::prelude::*;

    let org_id = payload
        .get("org_id")
        .and_then(|v| v.as_str())
        .and_then(|s| Uuid::parse_str(s).ok())
        .ok_or((
            StatusCode::BAD_REQUEST,
            "org_id must be a valid UUID".to_string(),
        ))?;

    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|a| a.strip_prefix("Bearer "))
        .map(|s| s.to_string());

    let user_id = match token {
        Some(t) => {
            let session = {
                let cache = botcoredirectory::auth_routes::SESSION_CACHE.read().await;
                cache.get(&t).cloned()
            };
            match session {
                Some(s) => s.user_id,
                None => {
                    return Err((
                        StatusCode::UNAUTHORIZED,
                        "No valid session for this token".to_string(),
                    ));
                }
            }
        }
        None => return Err((StatusCode::UNAUTHORIZED, "Missing bearer token".to_string())),
    };

    let mut conn = pool_conn(&state)?;

    if let Err(e) = diesel::sql_query(
        "INSERT INTO user_preferences (id, user_id, preference_key, preference_value, created_at, updated_at) \
         VALUES ($1, $2, 'current_org_id', $3::jsonb, NOW(), NOW()) \
         ON CONFLICT (user_id, preference_key) \
         DO UPDATE SET preference_value = EXCLUDED.preference_value, updated_at = NOW()",
    )
    .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
    .bind::<diesel::sql_types::Text, _>(&user_id)
    .bind::<diesel::sql_types::Text, _>(&org_id.to_string())
    .execute(&mut conn)
    {
        log::warn!("user current-org: failed to persist preference: {e}");
    }

    Ok(Json(
        serde_json::json!({ "success": true, "org_id": org_id }),
    ))
}

// ---------------------------------------------------------------------------
// Sandbox connection details: /api/v2/sandbox/connection-details
// ---------------------------------------------------------------------------
pub async fn handle_sandbox_connection_details() -> impl IntoResponse {
    Json(serde_json::json!({
        "host": "localhost",
        "ports": { "api": 8080, "ws": 8080 },
        "secure": false,
        "sandbox": true,
    }))
}
pub fn configure_misc_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/calendar/event/save", post(handle_calendar_event_save))
        .route("/api/goals/objectives/new", get(handle_goals_objective_new))
        .route("/api/ui/autotask/create", post(handle_autotask_create))
        .route("/api/services/summary", get(handle_services_summary))
        .route("/api/products/pricelists", get(handle_products_pricelists))
        .route("/api/contacts/search", get(handle_contacts_search))
        .route("/api/dns/list", get(handle_dns_list))
        .route("/api/dns/search", get(handle_dns_search))
        .route("/api/dns/register", post(handle_dns_register))
        .route("/api/dns/remove", post(handle_dns_remove))
        .route("/api/dns/:id/edit", get(handle_dns_edit_form))
        .route(
            "/api/user/current-org",
            get(handle_user_current_org).put(handle_user_current_org_put),
        )
        .route("/api/user/organizations", get(handle_user_organizations))
        .route(
            "/api/v2/sandbox/connection-details",
            get(handle_sandbox_connection_details),
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_tenants_only_see_their_own_organizations() {
        let tenant_a_org = Uuid::new_v4();
        let tenant_b_org = Uuid::new_v4();
        let member_org_ids = vec![tenant_a_org];

        assert!(org_in_caller_membership(&member_org_ids, tenant_a_org));
        // Tenant B's org is foreign: it is never listed for tenant A.
        assert!(!org_in_caller_membership(&member_org_ids, tenant_b_org));
        // The same check from tenant B's side keeps the boundary symmetric.
        assert!(org_in_caller_membership(&[tenant_b_org], tenant_b_org));
        assert!(!org_in_caller_membership(&[tenant_b_org], tenant_a_org));
    }

    #[test]
    fn empty_membership_lists_nothing() {
        // Anonymous / nil principals have no membership rows: fail closed.
        assert!(!org_in_caller_membership(&[], Uuid::new_v4()));
        assert!(!org_in_caller_membership(&[], Uuid::nil()));
    }
}
