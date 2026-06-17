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
    fn generate_stream(
        &self,
        prompt: &str,
        config: &serde_json::Value,
        tx: tokio::sync::mpsc::Sender<String>,
        model: &str,
        key: &str,
        tools: Option<&Vec<serde_json::Value>>,
    ) -> BoxFutureUnit;
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

    fn start_voice_session(&self, session_id: &str, user_id: &str) -> BoxFutureString {
        let _ = (session_id, user_id);
        Box::pin(async { Err("start_voice_session: not implemented".to_string()) })
    }

    fn stop_voice_session(&self, session_id: &str) -> BoxFutureUnit {
        let _ = session_id;
        Box::pin(async { Err("stop_voice_session: not implemented".to_string()) })
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
