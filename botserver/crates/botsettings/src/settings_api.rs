use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
    routing::{get, post},
    Router,
};
use diesel::RunQueryDsl;
use serde::Deserialize;
use std::sync::Arc;

use botcore::shared::state::AppState;

use crate::settings_billing;
use crate::settings_credentials;
use crate::settings_profile;

/// Resolves the authenticated user from the bearer session (fix #830).
///
/// The token is looked up in the persisted `login_sessions` table (the same
/// store the auth middleware rehydrates from), and the stored `user_id` is
/// mapped to a stable UUID the same way `resolve_user_role` does — never the
/// first row of the `users` table.
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
        return Err((StatusCode::UNAUTHORIZED, Html("Missing bearer token".to_string())));
    };

    let mut conn = get_conn(state)
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string())))?;

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct SessionRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        user_data: String,
    }

    let row: Option<SessionRow> = diesel::sql_query(
        "SELECT user_data FROM login_sessions WHERE token = $1 LIMIT 1",
    )
    .bind::<diesel::sql_types::Text, _>(&token)
    .get_result(&mut conn)
    .ok();

    let Some(row) = row else {
        return Err((StatusCode::UNAUTHORIZED, Html("No valid session for token".to_string())));
    };

    let parsed: serde_json::Value = serde_json::from_str(&row.user_data).unwrap_or_default();
    let uid = parsed.get("user_id").and_then(|v| v.as_str()).unwrap_or("");
    if uid.is_empty() {
        return Err((StatusCode::UNAUTHORIZED, Html("Invalid session user".to_string())));
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
fn is_admin_user(state: &Arc<AppState>, user_id: uuid::Uuid) -> bool {
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
    names.iter().any(|g| g.name.to_lowercase().contains("admin"))
}

pub fn configure_settings_api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/user/profile", get(settings_profile::user_profile_get).put(settings_profile::user_profile_put))
        .route("/api/user/password", post(settings_profile::user_password))
        .route("/api/user/api-keys", get(settings_credentials::api_keys_list).post(settings_credentials::api_keys_create))
        .route("/api/user/api-keys/:key_id", axum::routing::delete(settings_credentials::api_keys_delete))
        .route("/api/user/webhooks", get(settings_credentials::webhooks_list).post(settings_credentials::webhooks_create))
        .route("/api/user/webhooks/:webhook_id", axum::routing::delete(settings_credentials::webhooks_delete))
        .route("/api/user/billing/plan", get(settings_billing::billing_plan))
        .route("/api/user/billing/invoices", get(settings_billing::billing_invoices))
        .route("/api/user/billing/payment-methods", get(settings_billing::billing_payment_methods))
        .route("/api/user/data/export", post(settings_billing::data_export))
        .route("/api/user/notifications/preferences", get(notif_prefs_get).put(notif_prefs_put))
        .route("/api/oauth/:provider/connect", post(oauth_connect_handler))
        .route("/api/groups/create", post(groups_create))
        .route("/api/users/create", post(users_create))
}

pub fn get_conn(
    state: &Arc<AppState>,
) -> Option<diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>> {
    state.conn.get().ok()
}

#[derive(Debug, diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct TextRow {
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    value: Option<String>,
}

pub fn read_pref(conn: &mut diesel::PgConnection, user_id: uuid::Uuid, key: &str) -> serde_json::Value {
    diesel::sql_query(
        "SELECT preference_value::text AS value FROM user_preferences WHERE user_id = $1 AND preference_key = $2",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Text, _>(key)
    .get_result::<TextRow>(conn)
    .ok()
    .and_then(|r| r.value)
    .and_then(|v| serde_json::from_str(&v).ok())
    .unwrap_or(serde_json::Value::Null)
}

pub fn write_pref(conn: &mut diesel::PgConnection, user_id: uuid::Uuid, key: &str, value: &serde_json::Value) {
    let json = value.to_string();
    let _ = diesel::sql_query(
        "INSERT INTO user_preferences (id, user_id, preference_key, preference_value, created_at, updated_at)
         VALUES ($1, $2, $3, $4::jsonb, NOW(), NOW())
         ON CONFLICT (user_id, preference_key)
         DO UPDATE SET preference_value = EXCLUDED.preference_value, updated_at = NOW()",
    )
    .bind::<diesel::sql_types::Uuid, _>(uuid::Uuid::new_v4())
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Text, _>(key)
    .bind::<diesel::sql_types::Text, _>(&json)
    .execute(conn);
}


#[derive(Deserialize)]
pub struct NotifPrefsForm {
    pub email_dm: Option<String>,
    pub email_mentions: Option<String>,
    pub email_digest: Option<String>,
    pub email_marketing: Option<String>,
    pub push_enabled: Option<String>,
    pub push_sound: Option<String>,
    pub desktop_notifications: Option<String>,
    pub badge_count: Option<String>,
}

fn on_or_true(v: &Option<String>) -> bool {
    v.as_deref().map(|s| s == "on" || s == "true").unwrap_or(false)
}

pub async fn notif_prefs_get(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html(String::new())),
    };
    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let prefs = read_pref(&mut conn, user_id, "notifications");
    let email_dm = prefs.get("email_dm").and_then(|v| v.as_bool()).unwrap_or(true);
    let email_mentions = prefs.get("email_mentions").and_then(|v| v.as_bool()).unwrap_or(true);
    let email_digest = prefs.get("email_digest").and_then(|v| v.as_bool()).unwrap_or(false);
    let email_marketing = prefs.get("email_marketing").and_then(|v| v.as_bool()).unwrap_or(false);
    let push_enabled = prefs.get("push_enabled").and_then(|v| v.as_bool()).unwrap_or(false);
    let push_sound = prefs.get("push_sound").and_then(|v| v.as_bool()).unwrap_or(true);
    let desktop_notifications = prefs.get("desktop_notifications").and_then(|v| v.as_bool()).unwrap_or(true);
    let badge_count = prefs.get("badge_count").and_then(|v| v.as_bool()).unwrap_or(true);

    (StatusCode::OK, Html(format!(
        r#"<div class="preference-list">
    <label class="checkbox-label"><input type="checkbox" name="email_dm" {email_dm_ck}> Direct messages</label>
    <label class="checkbox-label"><input type="checkbox" name="email_mentions" {email_mentions_ck}> Mentions</label>
    <label class="checkbox-label"><input type="checkbox" name="email_digest" {email_digest_ck}> Weekly digest</label>
    <label class="checkbox-label"><input type="checkbox" name="email_marketing" {email_marketing_ck}> Marketing</label>
    <label class="checkbox-label"><input type="checkbox" name="push_enabled" {push_enabled_ck}> Push notifications</label>
    <label class="checkbox-label"><input type="checkbox" name="push_sound" {push_sound_ck}> Sound</label>
    <label class="checkbox-label"><input type="checkbox" name="desktop_notifications" {desktop_ck}> Desktop notifications</label>
    <label class="checkbox-label"><input type="checkbox" name="badge_count" {badge_ck}> Badge count</label>
</div>"#,
        email_dm_ck = if email_dm { "checked" } else { "" },
        email_mentions_ck = if email_mentions { "checked" } else { "" },
        email_digest_ck = if email_digest { "checked" } else { "" },
        email_marketing_ck = if email_marketing { "checked" } else { "" },
        push_enabled_ck = if push_enabled { "checked" } else { "" },
        push_sound_ck = if push_sound { "checked" } else { "" },
        desktop_ck = if desktop_notifications { "checked" } else { "" },
        badge_ck = if badge_count { "checked" } else { "" },
    )))
}

pub async fn notif_prefs_put(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Form(form): axum::extract::Form<NotifPrefsForm>,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string())),
    };
    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let prefs = serde_json::json!({
        "email_dm": on_or_true(&form.email_dm),
        "email_mentions": on_or_true(&form.email_mentions),
        "email_digest": on_or_true(&form.email_digest),
        "email_marketing": on_or_true(&form.email_marketing),
        "push_enabled": on_or_true(&form.push_enabled),
        "push_sound": on_or_true(&form.push_sound),
        "desktop_notifications": on_or_true(&form.desktop_notifications),
        "badge_count": on_or_true(&form.badge_count),
    });
    write_pref(&mut conn, user_id, "notifications", &prefs);

    (StatusCode::OK, Html("Notification preferences saved".to_string()))
}

/// POST /api/oauth/{provider}/connect — starts the account-linking flow.
/// Delegates to [`crate::settings_oauth::oauth_connect`], which builds a
/// user-bound OAuth state (PKCE) and returns the provider redirect URL.
async fn oauth_connect_handler(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Path(provider_name): axum::extract::Path<String>,
) -> axum::response::Response {
    crate::settings_oauth::oauth_connect(state, headers, provider_name).await
}

#[derive(Deserialize)]
pub struct GroupCreateForm {
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
}

pub async fn groups_create(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Form(form): axum::extract::Form<GroupCreateForm>,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string())),
    };
    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err(e) => return e,
    };
    if !is_admin_user(&state, user_id) {
        return (StatusCode::FORBIDDEN, Html("Admin role required".to_string()));
    }

    let name = form.name.unwrap_or_default();
    let display = form.display_name.clone().unwrap_or_else(|| name.clone());
    if name.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Html("Group name is required".to_string()));
    }

    let _ = diesel::sql_query(
        "INSERT INTO rbac_groups (id, name, display_name, description, is_active, created_by, created_at, updated_at)
         VALUES ($1, $2, $3, $4, true, $5, NOW(), NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(uuid::Uuid::new_v4())
    .bind::<diesel::sql_types::Text, _>(&name)
    .bind::<diesel::sql_types::Text, _>(&display)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(form.description)
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .execute(&mut conn);

    (StatusCode::OK, Html(format!("Group <strong>{display}</strong> created")))
}

#[derive(Deserialize)]
pub struct UserCreateForm {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub role: Option<String>,
}

pub async fn users_create(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Form(form): axum::extract::Form<UserCreateForm>,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string())),
    };
    let caller_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err(e) => return e,
    };
    if !is_admin_user(&state, caller_id) {
        return (StatusCode::FORBIDDEN, Html("Admin role required".to_string()));
    }

    let username = form.username.unwrap_or_default();
    let email = form.email.unwrap_or_default();
    let password = form.password.unwrap_or_default();
    if username.trim().is_empty() || email.trim().is_empty() || password.len() < 8 {
        return (StatusCode::BAD_REQUEST, Html("Username, email and a password of at least 8 characters are required".to_string()));
    }

    let hash = match hash_password(&password) {
        Ok(h) => h,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Failed to hash password".to_string())),
    };
    let is_admin = form.role.as_deref() == Some("admin");

    let inserted = diesel::sql_query(
        "INSERT INTO users (id, username, email, password_hash, is_active, is_admin, created_at, updated_at)
         VALUES ($1, $2, $3, $4, true, $5, NOW(), NOW()) ON CONFLICT (email) DO NOTHING",
    )
    .bind::<diesel::sql_types::Uuid, _>(uuid::Uuid::new_v4())
    .bind::<diesel::sql_types::Text, _>(&username)
    .bind::<diesel::sql_types::Text, _>(&email)
    .bind::<diesel::sql_types::Text, _>(&hash)
    .bind::<diesel::sql_types::Bool, _>(is_admin)
    .execute(&mut conn);

    match inserted {
        Ok(1) => (StatusCode::OK, Html(format!("User <strong>{username}</strong> created"))),
        Ok(_) => (StatusCode::CONFLICT, Html("A user with this email already exists".to_string())),
        Err(e) => {
            log::error!("users_create insert failed: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, Html("Failed to create user".to_string()))
        }
    }
}

pub fn hash_password(password: &str) -> anyhow::Result<String> {
    use argon2::password_hash::{rand_core::OsRng, SaltString};
    use argon2::{Argon2, PasswordHasher};

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    Ok(argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Password hashing failed: {e}"))?
        .to_string())
}

pub fn random_key(len: usize) -> String {
    use rand::Rng;
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::rng();
    (0..len)
        .map(|_| CHARS[rng.random_range(0..CHARS.len())] as char)
        .collect()
}
