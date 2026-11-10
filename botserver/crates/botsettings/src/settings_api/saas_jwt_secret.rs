use super::*;

/// Resolves the authenticated user from the bearer session (fix #830).
///
/// The token is looked up in the persisted `login_sessions` table (the same
/// store the auth middleware rehydrates from), and the stored `user_id` is
/// mapped to a stable UUID the same way `resolve_user_role` does — never the
/// first row of the `users` table.
/// #1457 — verifies a stateless SaaS cloud JWT and returns the caller's
/// stable user id. Cloud login mints claim-light tokens (`sub`/`email`/
/// `org_id`/`branch_id`, issue #736); the `sub` claim maps to a UUID the
/// same way `SaasJwtAuthProvider` and `resolve_user_role` do.
///
/// #2112 — the signature, algorithm and expiry are checked by the shared
/// verifier, not by a local `jsonwebtoken` setup.
pub(super) fn user_id_from_saas_jwt(token: &str) -> Option<uuid::Uuid> {
    let claims = botsecurity::botsecurity_core::verified_jwt::verify_saas_jwt(token).ok()?;
    Some(uuid::Uuid::parse_str(&claims.sub).unwrap_or_else(|_| {
        uuid::Uuid::new_v5(
            &uuid::Uuid::NAMESPACE_DNS,
            format!("zitadel:{}", claims.sub).as_bytes(),
        )
    }))
}
/// Fallback user resolution for fragment handlers without a request context.
/// Returns the first user row — these are read-only UI fragments (storage,
/// 2FA status) where the session may be anonymous.
pub fn first_user(conn: &mut diesel::PgConnection) -> Option<uuid::Uuid> {
    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct IdRow {
        #[diesel(sql_type = diesel::sql_types::Uuid)]
        id: uuid::Uuid,
    }
    diesel::sql_query("SELECT id FROM users ORDER BY created_at LIMIT 1")
        .get_result::<IdRow>(conn)
        .ok()
        .map(|r| r.id)
}
pub fn resolve_user_id(
    state: &Arc<AppState>,
    headers: &axum::http::HeaderMap,
) -> Result<uuid::Uuid, (StatusCode, Html<String>)> {
    let token = headers
        .get(axum::http::header::AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .and_then(|a| a.strip_prefix("Bearer "))
        .map(|s| s.to_string());
    let Some(token) = token else {
        return Err((
            StatusCode::UNAUTHORIZED,
            Html("Missing bearer token".to_string()),
        ));
    };

    let mut conn = get_conn(state).ok_or((
        StatusCode::INTERNAL_SERVER_ERROR,
        Html("Database unavailable".to_string()),
    ))?;

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct SessionRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        user_data: String,
    }

    let row: Option<SessionRow> = diesel::sql_query(
        // user_data is jsonb: cast to text so the Text decode always succeeds
        // (a raw jsonb result would silently fail and reject valid sessions).
        "SELECT user_data::text AS user_data FROM login_sessions WHERE token = $1 LIMIT 1",
    )
    .bind::<diesel::sql_types::Text, _>(&token)
    .get_result(&mut conn)
    .ok();

    let Some(row) = row else {
        // #1457 — cloud-login tokens are stateless SaaS JWTs (HS256, signed
        // with the persisted `saas_jwt_secret`) and never have a
        // `login_sessions` row, so every settings endpoint rejected them with
        // 401 while the same token authenticated fine against /api/crm/*.
        // Validate the JWT signature directly before rejecting.
        if let Some(user_id) = user_id_from_saas_jwt(&token) {
            return Ok(user_id);
        }
        return Err((
            StatusCode::UNAUTHORIZED,
            Html("No valid session for token".to_string()),
        ));
    };

    let parsed: serde_json::Value = serde_json::from_str(&row.user_data).unwrap_or_default();
    let uid = parsed.get("user_id").and_then(|v| v.as_str()).unwrap_or("");
    if uid.is_empty() {
        return Err((
            StatusCode::UNAUTHORIZED,
            Html("Invalid session user".to_string()),
        ));
    }

    match uuid::Uuid::parse_str(uid) {
        Ok(u) => Ok(u),
        Err(_) => Ok(uuid::Uuid::new_v5(
            &uuid::Uuid::NAMESPACE_DNS,
            format!("zitadel:{uid}").as_bytes(),
        )),
    }
}
/// Whether the resolved user holds an admin group (RBAC).
pub(super) fn is_admin_user(state: &Arc<AppState>, user_id: uuid::Uuid) -> bool {
    let Ok(mut conn) = state.conn.get() else {
        return false;
    };
    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct GroupName {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }
    let names: Vec<GroupName> = diesel::sql_query(
        "SELECT g.name FROM rbac_groups g \
         JOIN rbac_user_groups ug ON ug.group_id = g.id \
         WHERE ug.user_id = $1 AND g.is_active = true",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .load(&mut conn)
    .unwrap_or_default();
    names
        .iter()
        .any(|g| g.name.to_lowercase().contains("admin"))
}
pub fn configure_settings_api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route(
            "/api/user/profile",
            get(settings_profile::user_profile_get).put(settings_profile::user_profile_put),
        )
        .route(
            "/api/user/desktops",
            get(settings_profile::user_desktops_get).put(settings_profile::user_desktops_put),
        )
        .route("/api/user/password", post(settings_profile::user_password))
        .route(
            "/api/user/api-keys",
            get(settings_credentials::api_keys_list).post(settings_credentials::api_keys_create),
        )
        .route(
            "/api/user/api-keys/:key_id",
            axum::routing::delete(settings_credentials::api_keys_delete),
        )
        .route(
            "/api/user/api-keys/:key_id/rotate",
            post(settings_credentials::api_keys_rotate),
        )
        .route(
            "/api/user/webhooks",
            get(settings_webhooks::webhooks_list).post(settings_webhooks::webhooks_create),
        )
        .route(
            "/api/user/webhooks/:webhook_id",
            axum::routing::delete(settings_webhooks::webhooks_delete),
        )
        .route(
            "/api/user/webhooks/:webhook_id/test",
            post(settings_webhooks::webhooks_test),
        )
        .route(
            "/api/user/webhooks/:webhook_id/deliveries",
            get(settings_webhooks::webhook_deliveries_list),
        )
        .route(
            "/api/user/billing/plan",
            get(settings_billing::billing_plan),
        )
        .route(
            "/api/user/billing/invoices",
            get(settings_billing::billing_invoices),
        )
        .route(
            "/api/user/billing/payment-methods",
            get(settings_billing::billing_payment_methods),
        )
        .route("/api/user/data/export", post(settings_billing::data_export))
        .route(
            "/api/user/usage/summary",
            get(crate::settings_usage::usage_summary),
        )
        .route(
            "/api/user/memory",
            get(crate::settings_memory::list).put(crate::settings_memory::upsert),
        )
        .route(
            "/api/user/memory/:key",
            axum::routing::delete(crate::settings_memory::delete),
        )
        .route(
            "/api/user/notifications/preferences",
            get(notif_prefs_get).put(notif_prefs_put),
        )
        .route("/api/oauth/:provider/connect", post(oauth_connect_handler))
        .route("/api/groups/create", post(groups_create))
        .route("/api/users/create", post(users_create))
}
pub fn get_conn(
    state: &Arc<AppState>,
) -> Option<diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>> {
    state.conn.get().ok()
}
