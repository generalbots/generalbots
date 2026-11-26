use axum::{extract::State, http::HeaderMap, http::StatusCode, Json};
use botcore::shared::state::AppState;
use botsettings::settings_api::{read_pref, try_write_pref};
use diesel::Connection;
use serde::Deserialize;
use serde_json::{json, Value};
use std::sync::Arc;

#[derive(Deserialize)]
pub struct Profile {
    display_name: String,
    avatar_color: String,
}

pub async fn get(State(state): State<Arc<AppState>>, headers: HeaderMap) -> Result<Json<Value>, StatusCode> {
    let user = super::custom_instructions::authed_user(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let mut db = state.conn.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(Json(json!({
        "display_name": read_pref(&mut db, user, "display_name"),
        "avatar_color": read_pref(&mut db, user, "avatar_color")
    })))
}

pub async fn put(State(state): State<Arc<AppState>>, headers: HeaderMap, Json(profile): Json<Profile>) -> Result<Json<Value>, StatusCode> {
    let user = super::custom_instructions::authed_user(&headers).ok_or(StatusCode::UNAUTHORIZED)?;
    let name = profile.display_name.trim();
    let color = &profile.avatar_color;
    if name.is_empty() || name.chars().count() > 100 || name.chars().any(char::is_control)
        || !color.strip_prefix('#').is_some_and(|s| s.len() == 6 && s.bytes().all(|b| b.is_ascii_hexdigit())) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let mut db = state.conn.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    db.transaction::<_, diesel::result::Error, _>(|conn| {
        try_write_pref(conn, user, "display_name", &json!(name))?;
        try_write_pref(conn, user, "avatar_color", &json!(color))?;
        Ok(())
    }).map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    Ok(Json(json!({"display_name":name, "avatar_color":color})))
}
