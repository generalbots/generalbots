use super::slugify_bot_name::{caller_is_platform_admin, requesting_user};
use super::*;

/// Pure tenant-scope predicate for `GET /api/bots/{bot_id}/config` (#2795).
///
/// Mirrors the sibling handlers (`handle_bots_list` / `handle_bots_tree`): a
/// bot is readable when the caller is a platform admin, the bot is public, or
/// the bot's org is one of the caller's verified organizations. Kept pure so
/// cross-tenant isolation can be unit tested without a database.
fn bot_config_in_scope(
    is_public: bool,
    bot_org_id: Uuid,
    caller_org_ids: &[Uuid],
    is_platform_admin: bool,
) -> bool {
    is_platform_admin || is_public || caller_org_ids.contains(&bot_org_id)
}

/// GET /api/bots/{bot_id}/config — bot settings as JSON for the config form.
///
/// #2795 — the caller-supplied `bot_id` is never trusted for scope: the bot is
/// resolved and checked against the caller's verified tenant membership before
/// any configuration (LLM model/url/temperature) is returned. Foreign and
/// unknown ids are indistinguishable 404s.
pub async fn handle_bot_config_get(
    State(state): State<Arc<AppState>>,
    Path(bot_id): Path<Uuid>,
    req: axum::extract::Request,
) -> impl IntoResponse {
    let locale = botcore::i18n::locale_from_headers(req.headers());

    // Scope is derived from verified identity (RBAC-injected user), never from
    // a client-supplied name/id or any query/body field.
    let Some(user_id) = requesting_user(&req) else {
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
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        description: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Text)]
        llm_provider: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        enabled_tabs_json: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: Uuid,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        is_public: bool,
    }

    let bot: Option<BotRow> = diesel::sql_query(
        "SELECT name, description, llm_provider, enabled_tabs_json, org_id, is_public \
         FROM bots WHERE id = $1 LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .get_result::<BotRow>(&mut conn)
    .ok();

    let Some(bot) = bot else {
        return (
            StatusCode::NOT_FOUND,
            Json(botcore::i18n::error_body(&locale, "error-bot-not-found")),
        )
            .into_response();
    };

    // Resolve the caller's tenant scope from verified identity. Platform
    // admins and public bots skip the membership lookup; everyone else is
    // confined to orgs they actually belong to (same rule as the list/tree).
    let is_platform_admin = caller_is_platform_admin(&req, &state);
    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct OrgRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: Uuid,
    }
    let caller_org_ids: Vec<Uuid> = if is_platform_admin || bot.is_public {
        Vec::new()
    } else {
        match diesel::sql_query("SELECT org_id FROM user_organizations WHERE user_id = $1")
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .load::<OrgRow>(&mut conn)
        {
            Ok(rows) => rows.into_iter().map(|r| r.org_id).collect(),
            Err(e) => {
                // Fail closed: never grant scope when membership cannot be
                // resolved. Sanitize the DB error before it reaches the client.
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(crate::security::error_sanitizer::log_and_sanitize_json(
                        &e,
                        "bot config scope resolution",
                        None,
                    )),
                )
                    .into_response();
            }
        }
    };

    if !bot_config_in_scope(
        bot.is_public,
        bot.org_id,
        &caller_org_ids,
        is_platform_admin,
    ) {
        return (
            StatusCode::NOT_FOUND,
            Json(botcore::i18n::error_body(&locale, "error-bot-not-found")),
        )
            .into_response();
    }

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct CfgRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        config_key: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        config_value: String,
    }

    let cfgs: Vec<CfgRow> = diesel::sql_query(
        "SELECT config_key, config_value FROM bot_configuration WHERE bot_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .load::<CfgRow>(&mut conn)
    .unwrap_or_default();

    let mut config = serde_json::Map::new();
    for c in cfgs {
        config.insert(c.config_key, serde_json::Value::String(c.config_value));
    }

    let enabled_tabs: serde_json::Value = bot
        .enabled_tabs_json
        .as_deref()
        .and_then(|s| serde_json::from_str(s).ok())
        .unwrap_or_else(|| serde_json::json!(["chat"]));

    Json(serde_json::json!({
        "id": bot_id,
        "name": bot.name,
        "description": bot.description,
        "llm_provider": bot.llm_provider,
        "llm_model": config.get("llm-model").cloned().unwrap_or(serde_json::Value::Null),
        "llm_url": config.get("llm-url").cloned().unwrap_or(serde_json::Value::Null),
        "llm_temperature": config.get("llm-temperature").cloned().unwrap_or(serde_json::Value::from(0.7)),
        "enabled_tabs": enabled_tabs,
    }))
    .into_response()
}
/// PUT /api/bots/config — persist bot name/description/tabs/LLM settings.
pub async fn handle_bot_config_put(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Form(form): Form<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let locale = botcore::i18n::locale_from_headers(&headers);
    let mut conn = match pool_conn(&state) {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };

    let Some(bot_id_str) = form.get("bot_id") else {
        return (
            StatusCode::BAD_REQUEST,
            Json(botcore::i18n::error_body(&locale, "error-bot-id-required")),
        )
            .into_response();
    };
    let Ok(bot_id) = Uuid::parse_str(bot_id_str) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(botcore::i18n::error_body(&locale, "error-bot-id-invalid")),
        )
            .into_response();
    };

    let name = form.get("name").cloned().unwrap_or_default();
    let description = form.get("description").cloned();
    let llm_provider = form
        .get("llm_provider")
        .cloned()
        .unwrap_or_else(|| "openai".to_string());
    let llm_model = form.get("llm_model").cloned();
    let llm_url = form.get("llm_url").cloned();
    let llm_temperature = form.get("llm_temperature").cloned();

    let tabs: Vec<String> = form
        .iter()
        .filter(|(k, _)| k.ends_with("[]") || *k == "tabs")
        .map(|(_, v)| v.clone())
        .collect();
    let tabs_json = serde_json::to_string(&tabs).unwrap_or_else(|_| "[\"chat\"]".to_string());

    if name.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(botcore::i18n::error_body(
                &locale,
                "error-bot-name-required",
            )),
        )
            .into_response();
    }

    let result = diesel::sql_query(
        "UPDATE bots SET name = $2, description = $3, llm_provider = $4, \
                enabled_tabs_json = $5, updated_at = NOW() WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Text, _>(&name)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(description)
    .bind::<diesel::sql_types::Text, _>(&llm_provider)
    .bind::<diesel::sql_types::Text, _>(&tabs_json)
    .execute(&mut conn);

    if let Err(e) = result {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(crate::security::error_sanitizer::log_and_sanitize_json(
                &e,
                "bot update",
                None,
            )),
        )
            .into_response();
    }

    // Persist LLM settings in bot_configuration (upsert per key). The table
    // requires a NOT NULL branch_id (migration 6.5.23) and has no config_type
    // column anymore — the global fallback scope is the nil branch.
    let branch_id = uuid::Uuid::nil();
    let mut llm_settings: Vec<(&str, Option<String>)> = vec![
        ("llm-model", llm_model),
        ("llm-url", llm_url),
        ("llm-temperature", llm_temperature),
    ];
    llm_settings.retain(|(_, v)| v.is_some());
    for (key, value) in llm_settings {
        let _ = diesel::sql_query(
            "INSERT INTO bot_configuration (id, bot_id, branch_id, config_key, config_value, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW()) \
             ON CONFLICT (branch_id, bot_id, config_key) DO UPDATE SET config_value = EXCLUDED.config_value, updated_at = NOW()",
        )
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .bind::<diesel::sql_types::Text, _>(key)
        .bind::<diesel::sql_types::Text, _>(value.unwrap_or_default())
        .execute(&mut conn);
    }

    // #1427 — platform credential setters: `embedding-key` (KB embedding
    // server) and `botmodels-api-key` (BotModels anomaly API) are sensitive
    // keys, so they persist through the ConfigManager (Vault per-bot path) —
    // NEVER into bot_configuration plaintext.
    let config_manager = botcore::config::ConfigManager::new(state.conn.clone());
    let mut platform_credentials: Vec<(&str, Option<String>)> = vec![
        ("embedding-key", form.get("embedding_key").cloned()),
        ("botmodels-api-key", form.get("botmodels_api_key").cloned()),
    ];
    platform_credentials.retain(|(_, v)| !v.as_deref().map(str::trim).unwrap_or("").is_empty());
    for (key, value) in platform_credentials {
        if let Err(e) = config_manager.set_config_with_branch(
            &bot_id,
            key,
            value.as_deref().unwrap_or_default(),
            None,
        ) {
            log::error!("platform credential '{key}' persist for bot {bot_id} failed: {e}");
        }
    }

    Json(serde_json::json!({ "success": true })).into_response()
}
/// POST /api/bots — create a new bot from the modal form.
pub async fn handle_bot_create(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Form(form): Form<std::collections::HashMap<String, String>>,
) -> impl IntoResponse {
    let locale = botcore::i18n::locale_from_headers(&headers);
    let mut conn = match pool_conn(&state) {
        Ok(c) => c,
        Err(e) => return e.into_response(),
    };

    let name = form.get("name").cloned().unwrap_or_default();
    if name.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(botcore::i18n::error_body(
                &locale,
                "error-bot-name-required",
            )),
        )
            .into_response();
    }

    let description = form.get("description").cloned();
    let parent_bot_id = form.get("parent_bot_id").and_then(|v| {
        if v.is_empty() {
            None
        } else {
            Uuid::parse_str(v).ok()
        }
    });

    let bot_id = Uuid::new_v4();
    let branch_id = uuid::Uuid::nil();
    // #1289 — org scope: bots.org_id is NOT NULL; drive-created bots resolve
    // the default org. Resolve the default org the same way (any existing
    // bot's org, falling back to the bootstrap default org UUID).
    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct OrgId {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        org_id: Uuid,
    }
    let org_id: Uuid = diesel::sql_query(
        "SELECT org_id FROM bots WHERE org_id IS NOT NULL ORDER BY created_at ASC LIMIT 1",
    )
    .get_result::<OrgId>(&mut conn)
    .map(|r| r.org_id)
    .unwrap_or_else(|_| Uuid::from_u128(0xf47ac10b58cc4372a5670e02b2c3d479));
    // #1289 — is_public: the WS gateway denies connections to non-public
    // bots (`WS access denied` / close 1006). Bots created by a signed-in
    // user must be chattable immediately, exactly like the drive-discovered
    // ones on dev.
    let result = diesel::sql_query(
        "INSERT INTO bots (id, name, description, parent_bot_id, org_id, branch_id, llm_provider, llm_config, \
                context_provider, context_config, enabled_tabs_json, is_active, is_public, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $6, $5, 'openai', '{}', 'openai', '{}', '[\"chat\"]', true, true, NOW(), NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Text, _>(&name)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(description)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(parent_bot_id)
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .bind::<diesel::sql_types::Uuid, _>(org_id)
    .execute(&mut conn);

    if let Err(e) = result {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(crate::security::error_sanitizer::log_and_sanitize_json(
                &e,
                "bot create",
                None,
            )),
        )
            .into_response();
    }

    // #1289 — seed the drive objects so the bot can actually chat and the
    // launcher can link to it. Non-fatal (mirrors the signup flow): the DB
    // row exists; a drive failure is reported but does not roll back.
    let bot_slug = slugify_bot_name(&name);
    let mut drive_seeded = true;
    let mut drive_error = String::new();
    if bot_slug.is_empty() {
        drive_seeded = false;
        drive_error = "bot name produced an empty slug".to_string();
    } else if let Err(e) = seed_bot_drive_objects(&state, &bot_slug, &name).await {
        log::error!("drive seeding for new bot '{bot_slug}' failed: {e}");
        drive_seeded = false;
        drive_error = e;
    }

    Json(serde_json::json!({
        "success": true,
        "id": bot_id,
        "slug": bot_slug,
        "chat_url": format!("/{bot_slug}"),
        "drive_seeded": drive_seeded,
        "drive_error": drive_error,
    }))
    .into_response()
}
pub fn configure_bot_tree_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/bots/list", axum::routing::get(handle_bots_list))
        .route("/api/bots/tree", axum::routing::get(handle_bots_tree))
        .route(
            "/api/bots/:bot_id/config",
            axum::routing::get(handle_bot_config_get),
        )
        .route(
            "/api/bots/config",
            axum::routing::put(handle_bot_config_put),
        )
        .route("/api/bots", axum::routing::post(handle_bot_create))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn org(n: u128) -> Uuid {
        Uuid::from_u128(n)
    }

    #[test]
    fn scope_grants_platform_admin_and_public_bots() {
        let foreign = org(99);
        // A platform admin may read any bot, even a foreign private one.
        assert!(bot_config_in_scope(false, foreign, &[], true));
        // A public bot is readable by authenticated non-admins.
        assert!(bot_config_in_scope(true, foreign, &[], false));
    }

    #[test]
    fn two_tenant_isolation() {
        let tenant_a = org(1);
        let tenant_b = org(2);
        let caller_a_orgs = [tenant_a];

        // Caller A may read its own tenant's private bot.
        assert!(bot_config_in_scope(false, tenant_a, &caller_a_orgs, false));
        // Caller A may NOT read tenant B's private bot — must be a 404.
        assert!(!bot_config_in_scope(false, tenant_b, &caller_a_orgs, false));
        // A public bot of tenant B is visible to caller A.
        assert!(bot_config_in_scope(true, tenant_b, &caller_a_orgs, false));
        // A caller with no orgs cannot read any private bot.
        assert!(!bot_config_in_scope(false, tenant_a, &[], false));
    }
}
