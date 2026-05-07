use std::fmt::Debug;

pub trait LLMProvider: Send + Sync + Debug {
    fn generate(&self, prompt: &str) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>>;
    fn generate_with_context(
        &self,
        prompt: &str,
        context: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>>;
}

pub trait ChannelAdapter: Send + Sync + Debug {
    fn channel_type(&self) -> &str;
    fn send_message(&self, to: &str, message: &str) -> Result<(), String>;
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DriveObjectMetadata {
    pub size: u64,
    pub content_type: Option<String>,
    pub last_modified: Option<String>,
    pub etag: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DriveObjectInfo {
    pub key: String,
    pub etag: Option<String>,
    pub size: u64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DriveListEntry {
    pub key: String,
    pub size: u64,
}

impl DriveListEntry {
    pub fn key(&self) -> Option<String> {
        Some(self.key.clone())
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DriveListResult {
    pub objects: Vec<DriveListEntry>,
    pub common_prefixes: Vec<String>,
}

impl DriveListResult {
    pub fn contents(&self) -> &[DriveListEntry] {
        &self.objects
    }

    pub fn common_prefixes(&self) -> &[String] {
        &self.common_prefixes
    }
}

pub trait DriveRepository: Send + Sync + Debug {
    fn put_object(
        &self,
        bucket: &str,
        key: &str,
        data: Vec<u8>,
        content_type: Option<&str>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>;

    fn get_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, String>> + Send>>;

    fn delete_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>;

    fn copy_object(
        &self,
        bucket: &str,
        from_key: &str,
        to_key: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>;

    fn list_objects(
        &self,
        bucket: &str,
        prefix: Option<&str>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<String>, String>> + Send>>;

    fn list_objects_with_metadata(
        &self,
        bucket: &str,
        prefix: Option<&str>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<DriveObjectInfo>, String>> + Send>>;

    fn list_all_buckets(
        &self,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<String>, String>> + Send>>;

    fn object_exists(
        &self,
        bucket: &str,
        key: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>>;

    fn get_object_metadata(
        &self,
        bucket: &str,
        key: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<DriveObjectMetadata>, String>> + Send>>;

    fn create_bucket_if_not_exists(
        &self,
        bucket: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>;

    fn delete_objects(
        &self,
        bucket: &str,
        keys: Vec<String>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>;

    fn head_bucket(
        &self,
        bucket: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>>;

    fn list_objects_v2(
        &self,
        bucket: &str,
        prefix: &str,
        delimiter: Option<&str>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<DriveListResult, String>> + Send>>;

    fn upload_file(
        &self,
        bucket: &str,
        key: &str,
        file_path: &str,
        content_type: Option<&str>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>;

    fn download_file(
        &self,
        bucket: &str,
        key: &str,
        file_path: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>;
}

pub trait ScriptRunner: Send + Sync + Debug {
    fn run_script(
        &self,
        script: &str,
        session_id: uuid::Uuid,
        bot_id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>>;

    fn get_suggestions(
        &self,
        session_id: &uuid::Uuid,
        bot_id: &str,
    ) -> Result<Vec<crate::models::Suggestion>, String>;
}

pub trait TaskOrchestrator: Send + Sync + Debug {
    fn manifest(&self) -> String;
}

pub trait SessionStore: Send + Sync + Debug {
    fn get_session(
        &self,
        session_id: &uuid::Uuid,
    ) -> Result<Option<crate::models::Session>, String>;

    fn create_session(
        &self,
        user_id: uuid::Uuid,
        bot_id: uuid::Uuid,
    ) -> Result<crate::models::Session, String>;
}

pub trait KnowledgeBase: Send + Sync + Debug {
    fn query(&self, query: &str, limit: usize) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<String>, String>> + Send>>;

    fn index_document(
        &self,
        doc_id: &str,
        content: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>;
}

pub trait SessionManagerService: Send + Sync + Debug {
    fn get_session_by_id(&mut self, session_id: uuid::Uuid) -> Result<Option<serde_json::Value>, String>;

    fn get_or_create_user_session(
        &mut self,
        user_id: uuid::Uuid,
        bot_id: uuid::Uuid,
        session_title: &str,
    ) -> Result<Option<serde_json::Value>, String>;

    fn get_or_create_anonymous_user(
        &mut self,
        user_id: Option<uuid::Uuid>,
    ) -> Result<uuid::Uuid, String>;

    fn create_session(
        &mut self,
        user_id: uuid::Uuid,
        bot_id: uuid::Uuid,
        session_title: &str,
    ) -> Result<serde_json::Value, String>;

    fn get_or_create_session_by_id(
        &mut self,
        session_id: uuid::Uuid,
        user_id: uuid::Uuid,
        bot_id: uuid::Uuid,
        session_title: &str,
    ) -> Result<serde_json::Value, String>;

    fn save_message(
        &mut self,
        session_id: uuid::Uuid,
        user_id: uuid::Uuid,
        role: i32,
        content: &str,
        message_type: i32,
    ) -> Result<(), String>;

    fn get_conversation_history(
        &mut self,
        session_id: uuid::Uuid,
        user_id: uuid::Uuid,
        limit: Option<i64>,
    ) -> Result<Vec<(String, String)>, String>;

    fn get_session_context_data(
        &self,
        session_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
    ) -> Result<String, String>;

    fn update_session_context(
        &mut self,
        session_id: &uuid::Uuid,
        user_id: &uuid::Uuid,
        context_data: String,
    ) -> Result<(), String>;

    fn get_user_sessions(
        &mut self,
        user_id: uuid::Uuid,
    ) -> Result<Vec<serde_json::Value>, String>;

    fn update_user_id(
        &mut self,
        session_id: uuid::Uuid,
        new_user_id: uuid::Uuid,
    ) -> Result<(), String>;

    fn mark_waiting(&mut self, session_id: uuid::Uuid);

    fn active_count(&self) -> usize;
}

#[cfg(feature = "database")]
pub trait BotDatabaseService: Send + Sync + Debug {
fn get_bot_pool(
&self,
bot_id: uuid::Uuid,
) -> Option<crate::db_pool::DbPool>;

fn create_table_in_bot_database(
&self,
bot_id: uuid::Uuid,
sql: &str,
) -> Result<(), String>;

    fn sync_all_bot_databases(&self) -> Result<(), String>;
}

pub trait JwtService: Send + Sync + Debug {
    fn validate_access_token(
        &self,
        token: &str,
    ) -> Result<serde_json::Value, String>;

    fn generate_access_token(
        &self,
        user_id: uuid::Uuid,
        claims: serde_json::Value,
    ) -> Result<String, String>;
}

pub trait RbacService: Send + Sync + Debug {
    fn check_permission(
        &self,
        user_id: uuid::Uuid,
        resource: &str,
        action: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>>;

    fn register_routes(
        &self,
        default_permissions: serde_json::Value,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>;
}

pub trait AuthServiceTrait: Send + Sync + Debug {
    fn get_user_by_token(
        &self,
        token: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<serde_json::Value>, String>> + Send>>;
}

pub trait TaskSchedulerService: Send + Sync + Debug {
    fn schedule_task(
        &self,
        task_id: &str,
        cron_expr: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>;
}

pub trait TaskEngineService: Send + Sync + Debug {
    fn execute_task(
        &self,
        task_id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>;
}

pub trait MetricsService: Send + Sync + Debug {
    fn record_metric(
        &self,
        name: &str,
        value: f64,
    );
}
