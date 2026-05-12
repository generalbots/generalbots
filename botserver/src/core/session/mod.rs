pub use botcoresession::*;

pub async fn start_session() {}
pub async fn get_sessions() -> Vec<serde_json::Value> { vec![] }
pub async fn get_session_history() -> Vec<serde_json::Value> { vec![] }
pub async fn create_session() -> String { String::new() }

#[derive(Debug)]
pub struct LocalSessionManager(pub botcoresession::SessionManager);

impl botlib::traits::SessionManagerService for LocalSessionManager {
    fn get_session_by_id(&mut self, _session_id: uuid::Uuid) -> Result<Option<botlib::models::UserSession>, String> { Ok(None) }
    fn get_or_create_user_session(&mut self, _user_id: uuid::Uuid, _bot_id: uuid::Uuid, _session_title: &str) -> Result<Option<botlib::models::UserSession>, String> { Ok(None) }
    fn get_or_create_anonymous_user(&mut self, _user_id: Option<uuid::Uuid>) -> Result<uuid::Uuid, String> { Ok(uuid::Uuid::nil()) }
    fn create_session(&mut self, _user_id: uuid::Uuid, _bot_id: uuid::Uuid, _session_title: &str) -> Result<botlib::models::UserSession, String> { Err("not implemented".to_string()) }
    fn get_or_create_session_by_id(&mut self, _session_id: uuid::Uuid, _user_id: uuid::Uuid, _bot_id: uuid::Uuid, _session_title: &str) -> Result<botlib::models::UserSession, String> { Err("not implemented".to_string()) }
    fn save_message(&mut self, _session_id: uuid::Uuid, _user_id: uuid::Uuid, _role: i32, _content: &str, _message_type: i32) -> Result<(), String> { Ok(()) }
    fn get_conversation_history(&mut self, _session_id: uuid::Uuid, _user_id: uuid::Uuid, _limit: Option<i64>) -> Result<Vec<(String, String)>, String> { Ok(Vec::new()) }
    fn get_session_context_data(&self, _session_id: &uuid::Uuid, _user_id: &uuid::Uuid) -> Result<String, String> { Ok(String::new()) }
    fn update_session_context(&mut self, _session_id: &uuid::Uuid, _user_id: &uuid::Uuid, _context_data: String) -> Result<(), String> { Ok(()) }
    fn get_user_sessions(&mut self, _user_id: uuid::Uuid) -> Result<Vec<botlib::models::UserSession>, String> { Ok(Vec::new()) }
    fn update_user_id(&mut self, _session_id: uuid::Uuid, _new_user_id: uuid::Uuid) -> Result<(), String> { Ok(()) }
    fn mark_waiting(&mut self, _session_id: uuid::Uuid) {}
    fn active_count(&self) -> usize { 0 }
}
