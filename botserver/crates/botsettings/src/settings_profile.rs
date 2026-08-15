use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, IntoResponse},
};
use diesel::prelude::*;
use serde::Deserialize;
use std::sync::Arc;

use botcore::shared::state::AppState;

use crate::settings_api::{get_conn, hash_password, read_pref, resolve_user_id, write_pref};

fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// GET /api/user/profile — the caller's own profile (fix #830: resolved from
/// the bearer session, never the first user row).
pub async fn user_profile_get(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string())),
    };

    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err(e) => return e,
    };

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct UserRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        username: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        email: String,
    }

    let user: Option<UserRow> = diesel::sql_query("SELECT username, email FROM users WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .get_result::<UserRow>(&mut conn)
        .ok();

    let display_name = read_pref(&mut conn, user_id, "display_name")
        .as_str()
        .unwrap_or("")
        .to_string();
    let bio = read_pref(&mut conn, user_id, "bio").as_str().unwrap_or("").to_string();
    let phone = read_pref(&mut conn, user_id, "phone").as_str().unwrap_or("").to_string();
    let location = read_pref(&mut conn, user_id, "location").as_str().unwrap_or("").to_string();
    let website = read_pref(&mut conn, user_id, "website").as_str().unwrap_or("").to_string();
    let timezone = read_pref(&mut conn, user_id, "timezone").as_str().unwrap_or("UTC").to_string();

    match user {
        Some(u) => (
            StatusCode::OK,
            Html(format!(
                r#"<div class="profile-fields">
    <div class="form-group"><label>Username</label><input type="text" name="username" value="{username}" readonly></div>
    <div class="form-group"><label>Email</label><input type="email" name="email" value="{email}"></div>
    <div class="form-group"><label>Display Name</label><input type="text" name="display_name" value="{display_name}"></div>
    <div class="form-group"><label>Bio</label><textarea name="bio" rows="3">{bio}</textarea></div>
    <div class="form-group"><label>Phone</label><input type="text" name="phone" value="{phone}"></div>
    <div class="form-group"><label>Location</label><input type="text" name="location" value="{location}"></div>
    <div class="form-group"><label>Website</label><input type="text" name="website" value="{website}"></div>
    <div class="form-group"><label>Timezone</label><input type="text" name="timezone" value="{timezone}"></div>
</div>"#,
                username = html_escape(&u.username),
                email = html_escape(&u.email),
                display_name = html_escape(&display_name),
                bio = html_escape(&bio),
                phone = html_escape(&phone),
                location = html_escape(&location),
                website = html_escape(&website),
                timezone = html_escape(&timezone),
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
    pub bio: Option<String>,
    pub phone: Option<String>,
    pub location: Option<String>,
    pub website: Option<String>,
    pub timezone: Option<String>,
}

/// PUT /api/user/profile — persists the caller's editable profile fields.
pub async fn user_profile_put(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Form(form): axum::extract::Form<ProfileForm>,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string())),
    };

    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err(e) => return e,
    };

    if let Some(email) = form.email.as_deref() {
        if !email.trim().is_empty() {
            let _ = diesel::sql_query("UPDATE users SET email = $1, updated_at = NOW() WHERE id = $2")
                .bind::<diesel::sql_types::Text, _>(email)
                .bind::<diesel::sql_types::Uuid, _>(user_id)
                .execute(&mut conn);
        }
    }

    for (key, value) in [
        ("display_name", form.display_name.as_deref()),
        ("bio", form.bio.as_deref()),
        ("phone", form.phone.as_deref()),
        ("location", form.location.as_deref()),
        ("website", form.website.as_deref()),
        ("timezone", form.timezone.as_deref()),
    ] {
        if let Some(v) = value {
            if !v.trim().is_empty() {
                write_pref(&mut conn, user_id, key, &serde_json::json!(v));
            }
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

/// POST /api/user/password — verifies the current password (fix #830) before
/// replacing it with a freshly hashed one.
pub async fn user_password(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    axum::extract::Form(form): axum::extract::Form<PasswordForm>,
) -> impl IntoResponse {
    let mut conn = match get_conn(&state) {
        Some(c) => c,
        None => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Database unavailable".to_string())),
    };

    let user_id = match resolve_user_id(&state, &headers) {
        Ok(id) => id,
        Err(e) => return e,
    };

    let current = form.current_password.unwrap_or_default();
    if current.is_empty() {
        return (StatusCode::BAD_REQUEST, Html("Current password is required".to_string()));
    }

    let new_pass = form.new_password.unwrap_or_default();
    let confirm = form.confirm_password.unwrap_or_default();
    if new_pass.len() < 8 {
        return (StatusCode::BAD_REQUEST, Html("Password must be at least 8 characters".to_string()));
    }
    if new_pass != confirm {
        return (StatusCode::BAD_REQUEST, Html("Passwords do not match".to_string()));
    }

    #[derive(Debug, diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct HashRow {
        #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
        password_hash: Option<String>,
    }

    let stored: Option<HashRow> = diesel::sql_query("SELECT password_hash FROM users WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .get_result(&mut conn)
        .ok();

    let stored_hash = stored.and_then(|r| r.password_hash);
    let Some(stored_hash) = stored_hash else {
        return (
            StatusCode::BAD_REQUEST,
            Html("This account has no local password — use the identity provider to change it".to_string()),
        );
    };

    use argon2::password_hash::PasswordVerifier;
    let parsed = match argon2::PasswordHash::new(&stored_hash) {
        Ok(p) => p,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, Html("Stored hash is malformed".to_string())),
    };
    if argon2::Argon2::default()
        .verify_password(current.as_bytes(), &parsed)
        .is_err()
    {
        return (StatusCode::BAD_REQUEST, Html("Current password is incorrect".to_string()));
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
