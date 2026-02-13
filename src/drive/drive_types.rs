// Drive types extracted from drive/mod.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileItem {
    pub id: String,
    pub name: String,
    pub file_type: String,
    pub size: i64,
    pub mime_type: String,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub parent_id: Option<String>,
    pub url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub is_favorite: bool,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileTree {
    pub id: String,
    pub name: String,
    pub item_type: String,
    pub parent_id: Option<String>,
    pub children: Vec<FileTree>,
    pub created_at: DateTime<Utc>,
    pub modified_at: Option<DateTime<Utc>>,
    pub url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub is_expanded: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketInfo {
    pub id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub file_count: i32,
    pub total_size: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadRequest {
    pub file_name: String,
    pub file_path: String,
    pub content: Vec<u8>,
    pub mime_type: String,
    pub overwrite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFolderRequest {
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareRequest {
    pub file_ids: Vec<String>,
    pub recipient_email: Option<String>,
    pub recipient_id: Option<String>,
    pub permissions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: Option<String>,
    pub file_type: Option<String>,
    pub parent_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteRequest {
    pub file_id: String,
    pub is_favorite: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveFileRequest {
    pub file_id: String,
    pub target_parent_id: String,
    pub new_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyFileRequest {
    pub file_id: String,
    pub target_parent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadRequest {
    pub file_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFileRequest {
    pub file_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteRequest {
    pub file_id: Option<String>,
    pub content: String,
}
