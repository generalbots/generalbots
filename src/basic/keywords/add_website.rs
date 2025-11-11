use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{error, trace};
use rhai::{Dynamic, Engine};
use std::sync::Arc;
pub fn add_website_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
 let state_clone = Arc::clone(&state);
 let user_clone = user.clone();
 engine
 .register_custom_syntax(&["ADD_WEBSITE", "$expr$"], false, move |context, inputs| {
 let url = context.eval_expression_tree(&inputs[0])?;
 let url_str = url.to_string().trim_matches('"').to_string();
 trace!("ADD_WEBSITE command executed: {} for user: {}", url_str, user_clone.user_id);
 let is_valid = url_str.starts_with("http://") || url_str.starts_with("https://");
 if !is_valid {
 return Err(Box::new(rhai::EvalAltResult::ErrorRuntime("Invalid URL format. Must start with http:// or https://".into(), rhai::Position::NONE)));
 }
 let state_for_task = Arc::clone(&state_clone);
 let user_for_task = user_clone.clone();
 let url_for_task = url_str.clone();
 let (tx, rx) = std::sync::mpsc::channel();
 std::thread::spawn(move || {
 let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(2).enable_all().build();
 let send_err = if let Ok(rt) = rt {
 let result = rt.block_on(async move {
 crawl_and_index_website(&state_for_task, &user_for_task, &url_for_task).await
 });
 tx.send(result).err()
 } else {
 tx.send(Err("Failed to build tokio runtime".to_string())).err()
 };
 if send_err.is_some() {
 error!("Failed to send result from thread");
 }
 });
 match rx.recv_timeout(std::time::Duration::from_secs(120)) {
 Ok(Ok(message)) => {
 Ok(Dynamic::from(message))
 }
 Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(e.into(), rhai::Position::NONE))),
 Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
 Err(Box::new(rhai::EvalAltResult::ErrorRuntime("ADD_WEBSITE timed out".into(), rhai::Position::NONE)))
 }
 Err(e) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(format!("ADD_WEBSITE failed: {}", e).into(), rhai::Position::NONE))),
 }
 })
 .unwrap();
}
async fn crawl_and_index_website(_state: &AppState, _user: &UserSession, _url: &str) -> Result<String, String> {
 Err("Web automation functionality has been removed from this build".to_string())
}
