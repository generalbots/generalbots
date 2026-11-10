use super::*;

pub(super) fn slugify_bot_name(name: &str) -> String {
    name.to_lowercase()
        .replace([' ', '_'], "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect()
}
/// #1289 — a bot can only chat if its drive objects exist: drive_monitor
/// discovers bots from `{branch}.gbai/{bot}.gbdialog/*` keys, the WS session
/// runs `{bot}.gbdialog/start.bas`, and `.gbot/` carries per-bot config.
/// Seed the minimal working set into Drive right after the DB row insert
/// (best-effort: a Drive outage must not block bot creation, matching the
/// signup flow's non-fatal bucket creation).
pub(super) async fn seed_bot_drive_objects(
    state: &Arc<AppState>,
    bot_slug: &str,
    bot_name: &str,
) -> Result<(), String> {
    let s3 = state
        .drive
        .as_ref()
        .ok_or_else(|| "drive storage unavailable".to_string())?;
    let bucket = DEFAULT_GBORG_BUCKET;
    let prefix = DEFAULT_WORKSPACE_PREFIX;

    // 1. start.bas — the session entry point (double quotes: BASIC syntax).
    let start_bas = format!(
        "REM {bot_name} - session entry point (auto-generated at bot creation)\n\
         ADD_SUGGESTION \"What can you do?\"\n\
         ADD_SUGGESTION \"Tell me about yourself\"\n\
         TALK \"Hello! I'm {bot_name}. How can I help you today?\"\n"
    );
    s3.put_object(
        bucket,
        &format!("{prefix}/{bot_slug}.gbdialog/start.bas"),
        start_bas.into_bytes(),
        Some("text/plain"),
    )
    .await
    .map_err(|e| format!("seed start.bas: {e}"))?;

    // 2. config.bas — an empty tool table (kept for parity with templates).
    s3.put_object(
        bucket,
        &format!("{prefix}/{bot_slug}.gbdialog/config.bas"),
        b"REM placeholder tool configuration\n".to_vec(),
        Some("text/plain"),
    )
    .await
    .map_err(|e| format!("seed config.bas: {e}"))?;

    // 3. {bot}.gbot/config.csv — per-bot configuration header.
    let config_csv = "name,value\n".to_string();
    s3.put_object(
        bucket,
        &format!("{prefix}/{bot_slug}.gbot/config.csv"),
        config_csv.into_bytes(),
        Some("text/csv"),
    )
    .await
    .map_err(|e| format!("seed gbot config: {e}"))?;

    // 4. {bot}.gbkb/docs/.keep — knowledge-base directory marker.
    s3.put_object(
        bucket,
        &format!("{prefix}/{bot_slug}.gbkb/docs/.keep"),
        Vec::new(),
        Some("text/plain"),
    )
    .await
    .map_err(|e| format!("seed gbkb marker: {e}"))?;

    log::info!(
        "Seeded drive objects for bot '{bot_slug}' in {bucket}/{prefix} (gbdialog, gbot, gbkb)"
    );
    Ok(())
}
pub(super) fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
pub(super) fn pool_conn(
    state: &Arc<AppState>,
) -> Result<
    diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
    (StatusCode, String),
> {
    state
        .conn
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB error: {e}")))
}
/// Resolves the authenticated caller from the request extensions. The auth
/// middleware inserts `AuthenticatedUser` for every authenticated request.
fn requesting_user(req: &axum::extract::Request) -> Option<Uuid> {
    req.extensions()
        .get::<crate::security::auth_api::types::AuthenticatedUser>()
        .filter(|user| user.is_authenticated())
        .map(|user| user.user_id)
}

/// Platform admins (explicit superadmin, or a member of the reserved root org
/// `default`) may see every bot. Everyone else is confined to the bots of the
/// organizations they belong to (plus public bots).
fn caller_is_platform_admin(req: &axum::extract::Request, state: &Arc<AppState>) -> bool {
    let Some(user) = req
        .extensions()
        .get::<crate::security::auth_api::types::AuthenticatedUser>()
    else {
        return false;
    };
    if user.is_super_admin() {
        return true;
    }
    let Some(email) = user.email.as_deref().filter(|email| !email.is_empty()) else {
        return false;
    };
    let Ok(mut conn) = pool_conn(state) else {
        return false;
    };

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    diesel::sql_query(
        "SELECT COUNT(*) AS count FROM user_organizations uo \
         JOIN organizations o ON o.org_id = uo.org_id \
         JOIN users u ON u.id = uo.user_id \
         WHERE lower(u.email) = lower($1) AND o.slug = 'default'",
    )
    .bind::<diesel::sql_types::Text, _>(email)
    .get_result::<CountRow>(&mut conn)
    .map(|row| row.count > 0)
    .unwrap_or(false)
}

/// GET /api/bots/list — bots visible to the caller, for the "New Bot" parent
/// dropdown. Tenant isolation (#1782): a non-platform-admin only sees bots of
/// the organizations it belongs to and public bots — never every tenant's.
pub async fn handle_bots_list(
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
) -> axum::response::Response {
    let Some(user_id) = requesting_user(&req) else {
        let locale = botcore::i18n::locale_from_headers(req.headers());
        return (
            StatusCode::UNAUTHORIZED,
            Json(botcore::i18n::error_body(
                &locale,
                "error-auth-token-missing",
            )),
        )
            .into_response();
    };
    let mut conn = match pool_conn(&state) {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct BotRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }

    let rows: Vec<BotRow> = if caller_is_platform_admin(&req, &state) {
        diesel::sql_query("SELECT id, name FROM bots ORDER BY name ASC LIMIT 100")
            .load::<BotRow>(&mut conn)
            .unwrap_or_default()
    } else {
        diesel::sql_query(
            "SELECT b.id, b.name FROM bots b \
             WHERE b.is_public = true \
                OR b.org_id IN (SELECT uo.org_id FROM user_organizations uo WHERE uo.user_id = $1) \
             ORDER BY b.name ASC LIMIT 100",
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .load::<BotRow>(&mut conn)
        .unwrap_or_default()
    };

    let bots: Vec<serde_json::Value> = rows
        .into_iter()
        .map(|b| serde_json::json!({ "id": b.id, "name": b.name }))
        .collect();

    Json(serde_json::json!(bots)).into_response()
}
/// GET /api/bots/tree — HTML bot hierarchy (roots + sub-bots via parent_bot_id).
/// Scoped to the caller's organizations for non-platform-admins (#1782).
pub async fn handle_bots_tree(
    State(state): State<Arc<AppState>>,
    req: axum::extract::Request,
) -> Html<String> {
    let Some(user_id) = requesting_user(&req) else {
        return Html(String::new());
    };
    let mut conn = match pool_conn(&state) {
        Ok(c) => c,
        Err(_) => return Html(String::new()),
    };

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct BotRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        description: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
        parent_bot_id: Option<Uuid>,
    }

    let rows: Vec<BotRow> = if caller_is_platform_admin(&req, &state) {
        diesel::sql_query(
            "SELECT id, name, description, parent_bot_id FROM bots ORDER BY name ASC LIMIT 200",
        )
        .load::<BotRow>(&mut conn)
        .unwrap_or_default()
    } else {
        diesel::sql_query(
            "SELECT b.id, b.name, b.description, b.parent_bot_id FROM bots b \
             WHERE b.is_public = true \
                OR b.org_id IN (SELECT uo.org_id FROM user_organizations uo WHERE uo.user_id = $1) \
             ORDER BY b.name ASC LIMIT 200",
        )
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .load::<BotRow>(&mut conn)
        .unwrap_or_default()
    };

    if rows.is_empty() {
        return Html(
            r##"<div class="empty-state"><p>No bots found. Create your first bot to get started.</p></div>"##
                .to_string(),
        );
    }

    let roots: Vec<&BotRow> = rows.iter().filter(|b| b.parent_bot_id.is_none()).collect();

    pub(super) fn render_bot(bot: &BotRow, rows: &[BotRow], depth: usize) -> String {
        let children: Vec<&BotRow> = rows
            .iter()
            .filter(|b| b.parent_bot_id == Some(bot.id))
            .collect();

        let desc = bot
            .description
            .as_deref()
            .map(|d| format!("<span class=\"bot-desc\">{}</span>", html_escape(d)))
            .unwrap_or_default();

        let toggle = if children.is_empty() {
            String::new()
        } else {
            format!(
                r##"<span class="bot-toggle" data-bot-id="{id}" onclick="toggleBotChildren('{id}')">▸</span>"##,
                id = bot.id
            )
        };

        let children_html: String = children
            .iter()
            .map(|c| render_bot(c, rows, depth + 1))
            .collect();

        let children_block = if children.is_empty() {
            String::new()
        } else {
            format!(
                r##"<div class="bot-children" id="bot-children-{id}">{children}</div>"##,
                id = bot.id,
                children = children_html
            )
        };

        format!(
            r##"<div class="bot-tree-item" data-bot-id="{id}" onclick="selectBot('{id}', '{name}', {is_root})">
    {toggle}
    <span class="bot-tree-icon">🤖</span>
    <span class="bot-tree-label">{name}</span>
    {desc}
    {children_block}
</div>"##,
            id = bot.id,
            name = html_escape(&bot.name),
            is_root = if depth == 0 { "true" } else { "false" },
        )
    }

    let mut tree = String::new();
    for root in roots {
        tree.push_str(&render_bot(root, &rows, 0));
    }

    Html(format!(r##"<div class="bot-tree-view">{tree}</div>"##))
}
