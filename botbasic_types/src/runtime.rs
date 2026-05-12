use botlib::db_pool::DbPool;
use crate::types::UserSession;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

pub trait BasicRuntime: Send + Sync + std::fmt::Debug {
    fn db_pool(&self) -> &DbPool;
    fn cache_client(&self) -> Option<Arc<redis::Client>>;
    fn bucket_name(&self) -> &str;
    fn hear_channels(&self) -> &std::sync::Mutex<HashMap<Uuid, std::sync::mpsc::SyncSender<String>>>;
    fn bot_database_manager(&self) -> Arc<dyn botlib::traits::BotDatabaseService>;
    fn web_adapter(&self) -> Arc<dyn botlib::traits::ChannelAdapter>;
    fn drive_repository(&self) -> Option<Arc<dyn botlib::traits::DriveRepository>>;
    fn config_value(&self, key: &str) -> Option<String>;
    fn session_manager(&self) -> Arc<tokio::sync::Mutex<dyn botlib::traits::SessionManagerService>>;
    fn update_session_user(&self, session_id: Uuid, user_id: Uuid) -> Result<(), String>;
    fn send_message(&self, response: &botlib::models::BotResponse) -> Result<(), String>;
    fn execute_script(&self, user: UserSession, script: &str) -> Result<String, String>;
    fn llm_generate(&self, prompt: &str, model: &str, api_key: &str) -> Result<String, String> {
        let _ = (prompt, model, api_key);
        Err("LLM provider not configured".to_string())
    }
    fn default_bot_id(&self) -> String {
        String::new()
    }
    fn data_dir(&self) -> String {
        botlib::work_path::get_work_path()
    }
    fn as_any(&self) -> &dyn Any;
}
