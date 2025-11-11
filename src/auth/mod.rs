use actix_web::{HttpRequest, HttpResponse, Result, web};
use log::error;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;
use crate::shared::state::AppState;
pub struct AuthService {}
impl AuthService {
 pub fn new() -> Self {
 Self {}
 }
}
#[actix_web::get("/api/auth")]
async fn auth_handler(
 _req: HttpRequest,
 data: web::Data<AppState>,
 web::Query(params): web::Query<HashMap<String, String>>,
) -> Result<HttpResponse> {
 let bot_name = params.get("bot_name").cloned().unwrap_or_default();
 let _token = params.get("token").cloned();
 let user_id = {
 let mut sm = data.session_manager.lock().await;
 sm.get_or_create_anonymous_user(None).map_err(|e| {
 error!("Failed to create anonymous user: {}", e);
 actix_web::error::ErrorInternalServerError("Failed to create user")
 })?
 };
 let (bot_id, bot_name) = tokio::task::spawn_blocking({
 let bot_name = bot_name.clone();
 let conn = data.conn.clone();
 move || {
 let mut db_conn = conn.get().map_err(|e| format!("Failed to get database connection: {}", e))?;
 use crate::shared::models::schema::bots::dsl::*;
 use diesel::prelude::*;
 match bots
 .filter(name.eq(&bot_name))
 .filter(is_active.eq(true))
 .select((id, name))
 .first::<(Uuid, String)>(&mut db_conn)
 .optional()
 {
 Ok(Some((id_val, name_val))) => Ok((id_val, name_val)),
 Ok(None) => {
 match bots
 .filter(is_active.eq(true))
 .select((id, name))
 .first::<(Uuid, String)>(&mut db_conn)
 .optional()
 {
 Ok(Some((id_val, name_val))) => Ok((id_val, name_val)),
 Ok(None) => Err("No active bots found".to_string()),
 Err(e) => Err(format!("DB error: {}", e)),
 }
 }
 Err(e) => Err(format!("DB error: {}", e)),
 }
 }
 })
 .await
 .map_err(|e| {
 error!("Spawn blocking failed: {}", e);
 actix_web::error::ErrorInternalServerError("DB thread error")
 })?
 .map_err(|e| {
 error!("{}", e);
 actix_web::error::ErrorInternalServerError(e)
 })?;
 let session = {
 let mut sm = data.session_manager.lock().await;
 sm.get_or_create_user_session(user_id, bot_id, "Auth Session")
 .map_err(|e| {
 error!("Failed to create session: {}", e);
 actix_web::error::ErrorInternalServerError(e.to_string())
 })?
 .ok_or_else(|| {
 error!("Failed to create session");
 actix_web::error::ErrorInternalServerError("Failed to create session")
 })?
 };
 let auth_script_path = format!("./work/{}.gbai/{}.gbdialog/auth.ast", bot_name, bot_name);
 if tokio::fs::metadata(&auth_script_path).await.is_ok() {
 let auth_script = match tokio::fs::read_to_string(&auth_script_path).await {
 Ok(content) => content,
 Err(e) => {
 error!("Failed to read auth script: {}", e);
 return Ok(HttpResponse::Ok().json(serde_json::json!({
 "user_id": session.user_id,
 "session_id": session.id,
 "status": "authenticated"
 })));
 }
 };
 let script_service = crate::basic::ScriptService::new(Arc::clone(&data), session.clone());
 match tokio::time::timeout(
 std::time::Duration::from_secs(5),
 async {
 script_service
 .compile(&auth_script)
 .and_then(|ast| script_service.run(&ast))
 }
 ).await {
 Ok(Ok(result)) => {
 if result.to_string() == "false" {
 error!("Auth script returned false");
 return Ok(HttpResponse::Unauthorized()
 .json(serde_json::json!({"error": "Authentication failed"})));
 }
 }
 Ok(Err(e)) => {
 error!("Auth script execution error: {}", e);
 }
 Err(_) => {
 error!("Auth script timeout");
 }
 }
 }
 Ok(HttpResponse::Ok().json(serde_json::json!({
 "user_id": session.user_id,
 "session_id": session.id,
 "status": "authenticated"
 })))
}
#[cfg(test)]
pub mod auth_test;
