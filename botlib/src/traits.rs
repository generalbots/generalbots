use std::fmt::Debug;

pub type BoxError = Box<dyn std::error::Error + Send + Sync>;
pub type BoxFutureResult = std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, BoxError>> + Send>>;
pub type BoxFutureString = std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>>;
pub type BoxFutureUnit = std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>;
pub type BoxFutureBool = std::pin::Pin<Box<dyn std::future::Future<Output = Result<bool, String>> + Send>>;
pub type BoxFutureVecU8 = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<u8>, String>> + Send>>;
pub type BoxFutureVecString = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<String>, String>> + Send>>;
pub type BoxFutureVecDriveObject = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<DriveObjectInfo>, String>> + Send>>;
pub type BoxFutureOptionDriveMeta = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<DriveObjectMetadata>, String>> + Send>>;
pub type BoxFutureDriveList = std::pin::Pin<Box<dyn std::future::Future<Output = Result<DriveListResult, String>> + Send>>;
pub type BoxFutureOptionValue = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<serde_json::Value>, String>> + Send>>;
pub type BoxFutureValue = std::pin::Pin<Box<dyn std::future::Future<Output = Result<serde_json::Value, String>> + Send>>;
pub type BoxFutureVecValue = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<serde_json::Value>, String>> + Send>>;

pub trait LLMProvider: Send + Sync + Debug {
fn generate(&self, prompt: &str, config: &serde_json::Value, model: &str, key: &str) -> BoxFutureResult;
fn generate_simple(&self, prompt: &str) -> BoxFutureString;
fn generate_with_context(
&self,
prompt: &str,
context: &str,
) -> BoxFutureString;
}

pub trait ChannelAdapter: Send + Sync + Debug {
    fn channel_type(&self) -> &str;
    fn send_message(&self, to: &str, message: &str) -> Result<(), String>;

    fn send_message_to_session(&self, session_id: &str, message: &str) -> Result<(), String> {
        let _ = (session_id, message);
        Err("send_message_to_session: not implemented".to_string())
    }
    
    fn add_connection(&self, session_id: &str, sender: std::sync::mpsc::Sender<String>) -> Result<(), String> {
        let _ = (session_id, sender);
        Err("add_connection: not implemented".to_string())
    }
    
    fn remove_connection(&self, session_id: &str) -> Result<(), String> {
        let _ = session_id;
        Err("remove_connection: not implemented".to_string())
    }
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
    ) -> BoxFutureUnit;

    fn get_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> BoxFutureVecU8;

    fn delete_object(
        &self,
        bucket: &str,
        key: &str,
    ) -> BoxFutureUnit;

    fn copy_object(
        &self,
        bucket: &str,
        from_key: &str,
        to_key: &str,
    ) -> BoxFutureUnit;

    fn list_objects(
        &self,
        bucket: &str,
        prefix: Option<&str>,
    ) -> BoxFutureVecString;

    fn list_objects_with_metadata(
        &self,
        bucket: &str,
        prefix: Option<&str>,
    ) -> BoxFutureVecDriveObject;

    fn list_all_buckets(
        &self,
    ) -> BoxFutureVecString;

    fn object_exists(
        &self,
        bucket: &str,
        key: &str,
    ) -> BoxFutureBool;

    fn get_object_metadata(
        &self,
        bucket: &str,
        key: &str,
    ) -> BoxFutureOptionDriveMeta;

    fn create_bucket_if_not_exists(
        &self,
        bucket: &str,
    ) -> BoxFutureUnit;

    fn delete_objects(
        &self,
        bucket: &str,
        keys: Vec<String>,
    ) -> BoxFutureUnit;

    fn head_bucket(
        &self,
        bucket: &str,
    ) -> BoxFutureBool;

    fn list_objects_v2(
        &self,
        bucket: &str,
        prefix: &str,
        delimiter: Option<&str>,
    ) -> BoxFutureDriveList;

    fn upload_file(
        &self,
        bucket: &str,
        key: &str,
        file_path: &str,
        content_type: Option<&str>,
    ) -> BoxFutureUnit;

    fn download_file(
        &self,
        bucket: &str,
        key: &str,
        file_path: &str,
    ) -> BoxFutureUnit;

    fn get_object_direct(
        &self,
        bucket: &str,
        key: &str,
    ) -> BoxFutureVecU8 {
        self.get_object(bucket, key)
    }
    
    fn list_buckets(
        &self,
    ) -> BoxFutureVecString {
        self.list_all_buckets()
    }
}

pub trait ScriptRunner: Send + Sync + Debug {
    fn run_script(
        &self,
        script: &str,
        session_id: uuid::Uuid,
        bot_id: &str,
    ) -> BoxFutureString;

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
    fn query(&self, query: &str, limit: usize) -> BoxFutureVecString;

    fn index_document(
        &self,
        doc_id: &str,
        content: &str,
    ) -> BoxFutureUnit;
}

pub trait SessionManagerService: Send + Debug {
    fn get_session_by_id(&mut self, session_id: uuid::Uuid) -> Result<Option<crate::models::UserSession>, String>;

    fn get_or_create_user_session(
        &mut self,
        user_id: uuid::Uuid,
        bot_id: uuid::Uuid,
        session_title: &str,
    ) -> Result<Option<crate::models::UserSession>, String>;

    fn get_or_create_anonymous_user(
        &mut self,
        user_id: Option<uuid::Uuid>,
    ) -> Result<uuid::Uuid, String>;

    fn create_session(
        &mut self,
        user_id: uuid::Uuid,
        bot_id: uuid::Uuid,
        session_title: &str,
    ) -> Result<crate::models::UserSession, String>;

    fn get_or_create_session_by_id(
        &mut self,
        session_id: uuid::Uuid,
        user_id: uuid::Uuid,
        bot_id: uuid::Uuid,
        session_title: &str,
    ) -> Result<crate::models::UserSession, String>;

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
    ) -> Result<Vec<crate::models::UserSession>, String>;

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
    ) -> BoxFutureBool;

    fn register_routes(
        &self,
        default_permissions: serde_json::Value,
    ) -> BoxFutureUnit;
}

pub trait AuthServiceTrait: Send + Sync + Debug {
    fn api_url(&self) -> String;

    fn client_id(&self) -> String;

    fn client_secret(&self) -> String;

    fn get_access_token(
        &self,
    ) -> BoxFutureString;

    fn get_user_by_token(
        &self,
        token: &str,
    ) -> BoxFutureOptionValue;

    fn list_users(
        &self,
        limit: i64,
        offset: i64,
    ) -> BoxFutureValue {
        let _ = (limit, offset);
        Box::pin(async { Err("list_users: not implemented".to_string()) })
    }

    fn create_user(
        &self,
        email: &str,
        first_name: &str,
        last_name: &str,
        username: Option<&str>,
    ) -> BoxFutureString {
        let _ = (email, first_name, last_name, username);
        Box::pin(async { Err("create_user: not implemented".to_string()) })
    }

    fn add_org_member(
        &self,
        org_id: &str,
        user_id: &str,
        roles: Vec<String>,
    ) -> BoxFutureUnit {
        let _ = (org_id, user_id, roles);
        Box::pin(async { Err("add_org_member: not implemented".to_string()) })
    }

    fn set_user_password(
        &self,
        user_id: &str,
        password: &str,
    ) -> BoxFutureUnit {
        let _ = (user_id, password);
        Box::pin(async { Err("set_user_password: not implemented".to_string()) })
    }

    fn list_organizations(
        &self,
    ) -> BoxFutureValue {
        Box::pin(async { Err("list_organizations: not implemented".to_string()) })
    }

    fn http_get(
        &self,
        url: String,
    ) -> BoxFutureValue {
        let _ = url;
        Box::pin(async { Err("http_get: not implemented".to_string()) })
    }

    fn http_post(
        &self,
        url: String,
        body: serde_json::Value,
    ) -> BoxFutureValue {
        let _ = (url, body);
        Box::pin(async { Err("http_post: not implemented".to_string()) })
    }

    fn get_user(
        &self,
        user_id: &str,
    ) -> BoxFutureValue {
        let _ = user_id;
        Box::pin(async { Err("get_user: not implemented".to_string()) })
    }

    fn search_users(
        &self,
        query: &str,
    ) -> BoxFutureVecValue {
        let _ = query;
        Box::pin(async { Err("search_users: not implemented".to_string()) })
    }

    fn get_user_memberships(
        &self,
        user_id: &str,
        offset: i64,
        limit: i64,
    ) -> BoxFutureValue {
        let _ = (user_id, offset, limit);
        Box::pin(async { Err("get_user_memberships: not implemented".to_string()) })
    }

    fn remove_org_member(
        &self,
        org_id: &str,
        user_id: &str,
    ) -> BoxFutureUnit {
        let _ = (org_id, user_id);
        Box::pin(async { Err("remove_org_member: not implemented".to_string()) })
    }

    fn get_org_members(
        &self,
        org_id: &str,
    ) -> BoxFutureVecValue {
        let _ = org_id;
        Box::pin(async { Err("get_org_members: not implemented".to_string()) })
    }

    fn http_patch(
        &self,
        url: String,
        body: serde_json::Value,
    ) -> BoxFutureValue {
        let _ = (url, body);
        Box::pin(async { Err("http_patch: not implemented".to_string()) })
    }

    fn http_delete(
        &self,
        url: String,
    ) -> BoxFutureValue {
        let _ = url;
        Box::pin(async { Err("http_delete: not implemented".to_string()) })
    }

    fn http_put(
        &self,
        url: String,
        body: serde_json::Value,
    ) -> BoxFutureValue {
        let _ = (url, body);
        Box::pin(async { Err("http_put: not implemented".to_string()) })
    }
}

pub trait TaskSchedulerService: Send + Sync + Debug {
    fn schedule_task(
        &self,
        task_id: &str,
        cron_expr: &str,
    ) -> BoxFutureUnit;
}

pub trait TaskEngineService: Send + Sync + Debug {
    fn execute_task(
        &self,
        task_id: &str,
    ) -> BoxFutureUnit;
}

pub trait MetricsService: Send + Sync + Debug {
    fn record_metric(
        &self,
        name: &str,
        value: f64,
    );
}
