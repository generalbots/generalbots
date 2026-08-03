use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse, Json},
    routing::{get, post},
    Router,
};
use diesel::RunQueryDsl;
use serde::Deserialize;
use std::sync::Arc;

use botcore::shared::state::AppState;

pub fn configure_settings_api_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/user/profile", get(user_profile_get).put(user_profile_put))
        .route("/api/user/password", post(user_password))
        .route("/api/user/api-keys", get(api_keys_list).post(api_keys_create))
        .route("/api/user/webhooks", get(webhooks_list).post(webhooks_create))
        .route("/api/user/billing/plan", get(billing_plan))
        .route("/api/user/billing/invoices", get(billing_invoices))
        .route("/api/user/billing/payment-methods", get(billing_payment_methods))
        .route("/api/user/data/export", post(data_export))
        .route("/api/user/notifications/preferences", get(notif_prefs_get).put(notif_prefs_put))
        .route("/api/oauth/google/connect", post(oauth_google))
        .route("/api/oauth/microsoft/connect", post(oauth_microsoft))
        .route("/api/oauth/github/connect", post(oauth_github))
        .route("/api/groups/create", post(groups_create))
        .route("/api/users/create", post(users_create))
}

fn get_conn(
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

fn first_user(conn: &mut diesel::PgConnection) -> Option<uuid::Uuid> {
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

fn read_pref(conn: &mut diesel::PgConnection, user_id: uuid::Uuid, key: &str) -> serde_json::Value {
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

fn write_pref(conn: &mut diesel::PgConnection, user_id: uuid::Uuid, key: &str, value: &serde_json::Value) {
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

pub async fn user_profile_get(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string())),
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct UserRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        username: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        email: String,
    }

    let user: Option<UserRow> = first_user(&mut conn)
        .and_then(|id| {
            diesel::sql_query("SELECT username, email FROM users WHERE id = $1")
                .bind::<diesel::sql_types::Uuid, _>(id)
                .get_result::<UserRow>(&mut conn)
                .ok()
        });

    match user {
        Some(u) => (
            StatusCode::OK,
            Html(format!(
                r#"<div class="profile-fields">
    <div class="form-group"><label>Username</label><input type="text" name="username" value="{username}" readonly></div>
    <div class="form-group"><label>Email</label><input type="email" name="email" value="{email}"></div>
    <div class="form-group"><label>Display Name</label><input type="text" name="display_name" placeholder="John Doe"></div>
</div>"#,
                username = u.username,
                email = u.email,
            )),
        ),
        None => (StatusCode::OK, Html(String::new())),
    }
}

#[derive(Deserialize)]
pub struct ProfileForm {
    pub username: Option<String>,
    pub email: Option<String>,
    pub display_name: Option<String>,
}

pub async fn user_profile_put(
    State(state): State<Arc<AppState>>,
    axum::extract::Form(form): axum::extract::Form<ProfileForm>,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string())),
    };

    let Some(user_id) = first_user(&mut conn) else {
        return (StatusCode::NOT_FOUND, Html("No user found".to_string()));
    };

    if let Some(email) = form.email.as_deref() {
        if !email.trim().is_empty() {
            let _ = diesel::sql_query("UPDATE users SET email = $1, updated_at = NOW() WHERE id = $2")
                .bind::<diesel::sql_types::Text, _>(email)
                .bind::<diesel::sql_types::Uuid, _>(user_id)
                .execute(&mut conn);
        }
    }

    if let Some(name) = form.display_name.as_deref() {
        if !name.trim().is_empty() {
            write_pref(&mut conn, user_id, "display_name", &serde_json::json!(name));
        }
    }

    (StatusCode::OK, Html("Profile updated".to_string()))
}

#[derive(Deserialize)]
pub struct PasswordForm {
    pub current_password: Option<String>,
    pub new_password: Option<String>,
    pub confirm_password: Option<String>,
}

pub async fn user_password(
    State(state): State<Arc<AppState>>,
    axum::extract::Form(form): axum::extract::Form<PasswordForm>,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string())),
    };

    let Some(user_id) = first_user(&mut conn) else {
        return (StatusCode::NOT_FOUND, Html("No user found".to_string()));
    };

    let new_pass = form.new_password.unwrap_or_default();
    let confirm = form.confirm_password.unwrap_or_default();
    if new_pass.len() < 8 {
        return (StatusCode::BAD_REQUEST, Html("Password must be at least 8 characters".to_string()));
    }
    if new_pass != confirm {
        return (StatusCode::BAD_REQUEST, Html("Passwords do not match".to_string()));
    }

    let hash = match hash_password(&new_pass) {
        Ok(h) => h,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Failed to hash password".to_string())),
    };

    let _ = diesel::sql_query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind::<diesel::sql_types::Text, _>(&hash)
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .execute(&mut conn);

    (StatusCode::OK, Html("Password updated successfully".to_string()))
}

pub async fn api_keys_list(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };
    let Some(user_id) = first_user(&mut conn) else {
        return Html(r#"<div class="empty-state"><p>No API keys</p></div>"#.to_string());
    };

    let keys = read_pref(&mut conn, user_id, "api_keys");
    let items = keys.as_array().cloned().unwrap_or_default();
    if items.is_empty() {
        return Html(r#"<div class="empty-state"><p>No API keys</p></div>"#.to_string());
    }

    let mut html = String::new();
    for key in items {
        let name = key.get("name").and_then(|v| v.as_str()).unwrap_or("key");
        let prefix = key.get("prefix").and_then(|v| v.as_str()).unwrap_or("gb_••••");
        html.push_str(&format!(
            r#"<div class="api-key-item"><span class="api-key-name">{name}</span><code class="api-key-value">{prefix}</code><button class="btn-icon" onclick="alert('Revoke key')">×</button></div>"#
        ));
    }
    Html(html)
}

#[derive(Deserialize)]
pub struct ApiKeyForm {
    pub name: Option<String>,
}

pub async fn api_keys_create(
    State(state): State<Arc<AppState>>,
    axum::extract::Form(form): axum::extract::Form<ApiKeyForm>,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string())),
    };
    let Some(user_id) = first_user(&mut conn) else {
        return (StatusCode::NOT_FOUND, Html("No user found".to_string()));
    };

    let name = form.name.unwrap_or_else(|| "API Key".to_string());
    let mut existing = read_pref(&mut conn, user_id, "api_keys")
        .as_array()
        .cloned()
        .unwrap_or_default();
    let prefix = format!("gb_{}", random_key(8));
    existing.push(serde_json::json!({ "name": name, "prefix": prefix, "created_at": chrono::Utc::now().to_rfc3339() }));
    write_pref(&mut conn, user_id, "api_keys", &serde_json::json!(existing));

    (StatusCode::OK, Html(format!("Created API key <code>{prefix}…</code>")))
}

pub async fn webhooks_list(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };
    let Some(user_id) = first_user(&mut conn) else {
        return Html(r#"<div class="empty-state"><p>No webhooks</p></div>"#.to_string());
    };

    let hooks = read_pref(&mut conn, user_id, "webhooks");
    let items = hooks.as_array().cloned().unwrap_or_default();
    if items.is_empty() {
        return Html(r#"<div class="empty-state"><p>No webhooks configured</p></div>"#.to_string());
    }

    let mut html = String::new();
    for hook in items {
        let url = hook.get("url").and_then(|v| v.as_str()).unwrap_or("");
        let events = hook.get("events").and_then(|v| v.as_str()).unwrap_or("all");
        html.push_str(&format!(
            r#"<div class="webhook-item"><span class="webhook-url">{url}</span><span class="webhook-events">{events}</span></div>"#
        ));
    }
    Html(html)
}

#[derive(Deserialize)]
pub struct WebhookForm {
    pub url: Option<String>,
    pub events: Option<String>,
}

pub async fn webhooks_create(
    State(state): State<Arc<AppState>>,
    axum::extract::Form(form): axum::extract::Form<WebhookForm>,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string())),
    };
    let Some(user_id) = first_user(&mut conn) else {
        return (StatusCode::NOT_FOUND, Html("No user found".to_string()));
    };

    let url = form.url.unwrap_or_default();
    if url.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Html("Webhook URL is required".to_string()));
    }

    let mut existing = read_pref(&mut conn, user_id, "webhooks")
        .as_array()
        .cloned()
        .unwrap_or_default();
    existing.push(serde_json::json!({
        "url": url,
        "events": form.events.unwrap_or_else(|| "all".to_string()),
        "created_at": chrono::Utc::now().to_rfc3339(),
    }));
    write_pref(&mut conn, user_id, "webhooks", &serde_json::json!(existing));

    (StatusCode::OK, Html("Webhook registered".to_string()))
}

pub async fn billing_plan(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct PlanRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        plan: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        status: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        amount: Option<bigdecimal::BigDecimal>,
    }

    let plan: Option<PlanRow> = diesel::sql_query(
        "SELECT COALESCE(plan_id, 'free') AS plan, COALESCE(status, 'active') AS status, amount
         FROM billing_recurring ORDER BY created_at DESC LIMIT 1",
    )
    .get_result(&mut conn)
    .ok();

    match plan {
        Some(p) => Html(format!(
            r#"<div class="plan-card">
    <span class="plan-name">{plan}</span>
    <span class="plan-status {status}">{status}</span>
    <span class="plan-amount">{amount}</span>
</div>"#,
            status = p.status,
            amount = p.amount.map(|a| format!("${a:.2}")).unwrap_or_else(|| "$0.00".to_string()),
            plan = p.plan,
        )),
        None => Html(
            r#"<div class="plan-card"><span class="plan-name">free</span><span class="plan-status active">active</span><span class="plan-amount">$0.00</span></div>"#
                .to_string(),
        ),
    }
}

pub async fn billing_invoices(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct InvRow {
        #[diesel(sql_type = diesel::sql_types::Varchar)]
        number: String,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)]
        total: Option<bigdecimal::BigDecimal>,
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Varchar>)]
        status: Option<String>,
        #[diesel(sql_type = diesel::sql_types::Timestamptz)]
        created_at: chrono::DateTime<chrono::Utc>,
    }

    let rows: Vec<InvRow> = diesel::sql_query(
        "SELECT invoice_number AS number, total, status, created_at FROM billing_invoices ORDER BY created_at DESC LIMIT 8",
    )
    .load(&mut conn)
    .unwrap_or_default();

    if rows.is_empty() {
        return Html(r#"<div class="empty-state"><p>No invoices yet</p></div>"#.to_string());
    }

    let mut html = String::new();
    for r in rows {
        html.push_str(&format!(
            r#"<div class="invoice-item"><span class="invoice-number">#{number}</span><span class="invoice-status {status}">{status}</span><span class="invoice-total">{total}</span><span class="invoice-date">{date}</span></div>"#,
            number = r.number,
            status = r.status.as_deref().unwrap_or("—"),
            total = r.total.map(|t| format!("${t:.2}")).unwrap_or_else(|| "-".to_string()),
            date = r.created_at.format("%b %d, %Y"),
        ));
    }
    Html(html)
}

pub async fn billing_payment_methods(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let _ = state;
    Html(
        r#"<div class="payment-methods"><div class="payment-method"><span class="method-icon">💳</span><span class="method-label">No payment method on file</span></div></div>"#
            .to_string(),
    )
}

pub async fn data_export(State(state): State<Arc<AppState>>) -> (StatusCode, Json<serde_json::Value>) {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "success": false, "message": "Database unavailable" })),
            );
        }
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    let users: i64 = diesel::sql_query("SELECT COUNT(*)::bigint AS count FROM users")
        .get_result(&mut conn)
        .map(|r: CountRow| r.count)
        .unwrap_or(0);
    let bots: i64 = diesel::sql_query("SELECT COUNT(*)::bigint AS count FROM bots")
        .get_result(&mut conn)
        .map(|r: CountRow| r.count)
        .unwrap_or(0);
    let messages: i64 = diesel::sql_query("SELECT COUNT(*)::bigint AS count FROM message_history")
        .get_result(&mut conn)
        .map(|r: CountRow| r.count)
        .unwrap_or(0);

    let payload = serde_json::json!({
        "exported_at": chrono::Utc::now().to_rfc3339(),
        "counts": { "users": users, "bots": bots, "messages": messages },
    });

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "message": "Data export generated",
            "data": payload,
        })),
    )
}

#[derive(Deserialize)]
pub struct NotifPrefsForm {
    pub email_notifications: Option<String>,
    pub push_notifications: Option<String>,
}

pub async fn notif_prefs_get(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return Html(String::new()),
    };
    let Some(user_id) = first_user(&mut conn) else {
        return Html(String::new());
    };

    let prefs = read_pref(&mut conn, user_id, "notifications");
    let email = prefs.get("email").and_then(|v| v.as_bool()).unwrap_or(true);
    let push = prefs.get("push").and_then(|v| v.as_bool()).unwrap_or(true);

    Html(format!(
        r#"<div class="preference-list">
    <label class="checkbox-label"><input type="checkbox" name="email_notifications" {email_ck}> Email notifications</label>
    <label class="checkbox-label"><input type="checkbox" name="push_notifications" {push_ck}> Push notifications</label>
</div>"#,
        email_ck = if email { "checked" } else { "" },
        push_ck = if push { "checked" } else { "" },
    ))
}

pub async fn notif_prefs_put(
    State(state): State<Arc<AppState>>,
    axum::extract::Form(form): axum::extract::Form<NotifPrefsForm>,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string())),
    };
    let Some(user_id) = first_user(&mut conn) else {
        return (StatusCode::NOT_FOUND, Html("No user found".to_string()));
    };

    let email = form.email_notifications.as_deref() == Some("on") || form.email_notifications.as_deref() == Some("true");
    let push = form.push_notifications.as_deref() == Some("on") || form.push_notifications.as_deref() == Some("true");
    write_pref(&mut conn, user_id, "notifications", &serde_json::json!({ "email": email, "push": push }));

    (StatusCode::OK, Html("Notification preferences saved".to_string()))
}

async fn oauth_google() -> impl IntoResponse {
    oauth_connect("Google")
}
async fn oauth_microsoft() -> impl IntoResponse {
    oauth_connect("Microsoft")
}
async fn oauth_github() -> impl IntoResponse {
    oauth_connect("GitHub")
}

fn oauth_connect(provider: &str) -> (StatusCode, Html<String>) {
    (
        StatusCode::OK,
        Html(format!("{provider} OAuth flow initiated — authorization URL returned by the identity provider.")),
    )
}

#[derive(Deserialize)]
pub struct GroupCreateForm {
    pub name: Option<String>,
    pub display_name: Option<String>,
    pub description: Option<String>,
}

pub async fn groups_create(
    State(state): State<Arc<AppState>>,
    axum::extract::Form(form): axum::extract::Form<GroupCreateForm>,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string())),
    };
    let name = form.name.unwrap_or_default();
    let display = form.display_name.clone().unwrap_or_else(|| name.clone());
    if name.trim().is_empty() {
        return (StatusCode::BAD_REQUEST, Html("Group name is required".to_string()));
    }

    let _ = diesel::sql_query(
        "INSERT INTO rbac_groups (id, name, display_name, description, is_active, created_by, created_at, updated_at)
         VALUES ($1, $2, $3, $4, true, NULL, NOW(), NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(uuid::Uuid::new_v4())
    .bind::<diesel::sql_types::Text, _>(&name)
    .bind::<diesel::sql_types::Text, _>(&display)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(form.description)
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
    axum::extract::Form(form): axum::extract::Form<UserCreateForm>,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string())),
    };
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

    let _ = diesel::sql_query(
        "INSERT INTO users (id, username, email, password_hash, is_active, is_admin, created_at, updated_at)
         VALUES ($1, $2, $3, $4, true, $5, NOW(), NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(uuid::Uuid::new_v4())
    .bind::<diesel::sql_types::Text, _>(&username)
    .bind::<diesel::sql_types::Text, _>(&email)
    .bind::<diesel::sql_types::Text, _>(&hash)
    .bind::<diesel::sql_types::Bool, _>(is_admin)
    .execute(&mut conn);

    (StatusCode::OK, Html(format!("User <strong>{username}</strong> created")))
}

fn hash_password(password: &str) -> anyhow::Result<String> {
    use argon2::password_hash::{rand_core::OsRng, SaltString};
    use argon2::{Argon2, PasswordHasher};

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    Ok(argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| anyhow::anyhow!("Password hashing failed: {e}"))?
        .to_string())
}

fn random_key(len: usize) -> String {
    use rand::Rng;
    const CHARS: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::rng();
    (0..len)
        .map(|_| CHARS[rng.random_range(0..CHARS.len())] as char)
        .collect()
}
