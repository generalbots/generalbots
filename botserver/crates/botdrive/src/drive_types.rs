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
    pub bucket: Option<String>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadResponse {
    pub content: String,
    pub file_name: String,
    pub mime_type: Option<String>,
}
// drive change test

// ====== Per-user file scoping (Issue #589) ======

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum FileScope {
    #[default]
    User,
    Bot,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ListFilesParams {
    pub path: Option<String>,
    pub bucket: Option<String>,
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub sort: Option<String>,
    pub order: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PaginatedResponse<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub page: u32,
    pub page_size: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileListItem {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<String>,
    pub is_kb: bool,
    pub is_public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BucketListItem {
    pub name: String,
    pub is_gbai: bool,
    pub is_gborg: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListBucketsParams {
    pub bot: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteFileBody {
    pub bucket: Option<String>,
    pub path: String,
    pub content: String,
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteFileBody {
    pub bucket: Option<String>,
    pub path: String,
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateFolderBody {
    pub bucket: Option<String>,
    pub path: String,
    pub name: String,
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadFileBody {
    pub bucket: Option<String>,
    pub path: String,
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CopyFileBody {
    pub source_bucket: Option<String>,
    pub source_path: String,
    pub dest_bucket: Option<String>,
    pub dest_path: String,
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveFileBody {
    pub source_bucket: Option<String>,
    pub source_path: String,
    pub dest_bucket: Option<String>,
    pub dest_path: String,
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQueryParams {
    pub query: Option<String>,
    pub bucket: Option<String>,
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecentQueryParams {
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
    pub bucket: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenFileBody {
    pub bucket: Option<String>,
    pub path: String,
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub success: bool,
}

#[derive(Debug, Serialize)]
pub struct DownloadFileResponse {
    pub content: String,
    pub file_name: String,
}

#[derive(Debug, Serialize)]
pub struct OpenFileResponse {
    pub app: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct QuotaResponse {
    pub used_bytes: u64,
    pub total_bytes: u64,
    pub available_bytes: u64,
    pub percentage_used: f64,
}

#[derive(Debug, Serialize)]
pub struct TrashItem {
    pub id: String,
    pub user_id: String,
    pub bucket: String,
    pub path: String,
    pub original_path: String,
    pub is_dir: bool,
    pub size: u64,
    pub deleted_at: String,
    pub expires_at: String,
}

#[derive(Debug, Deserialize)]
pub struct TrashQueryParams {
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RestoreTrashBody {
    pub id: String,
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct EmptyTrashBody {
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct StarToggleBody {
    pub bucket: Option<String>,
    pub path: String,
    pub starred: bool,
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct StarItem {
    pub id: String,
    pub bucket: String,
    pub path: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateShareBody {
    pub bucket: Option<String>,
    pub path: String,
    pub recipient_id: String,
    pub permissions: Option<String>,
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ShareItem {
    pub id: String,
    pub owner_id: String,
    pub recipient_id: String,
    pub bucket: String,
    pub path: String,
    pub permissions: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateBotRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateBotResponse {
    pub name: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct DeleteBotRequest {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub struct DeleteBotResponse {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct UploadChunkBody {
    pub bucket: Option<String>,
    pub path: String,
    pub content: String,
    pub offset: Option<u64>,
    pub total_size: Option<u64>,
    pub complete: Option<bool>,
    pub upload_id: Option<String>,
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
}

// ====== Public share links (token-based, revocable) ======

#[derive(Debug, Deserialize)]
pub struct CreatePublicLinkBody {
    pub bucket: Option<String>,
    pub path: String,
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
    /// Optional RFC3339 expiry (e.g. "2026-12-31T23:59:59Z").
    pub expires_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreatePublicLinkResponse {
    pub token: String,
    pub url: String,
    pub created_at: String,
    pub expires_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct RevokePublicLinkBody {
    pub token: Option<String>,
    pub bucket: Option<String>,
    pub path: Option<String>,
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PublicLinkItem {
    pub token: String,
    pub url: String,
    pub bucket: String,
    pub path: String,
    pub created_at: String,
    pub expires_at: Option<String>,
    pub revoked: bool,
}

#[derive(Debug, Deserialize)]
pub struct ListPublicLinksParams {
    pub path: Option<String>,
    pub bucket: Option<String>,
    pub scope: Option<FileScope>,
    pub user_id: Option<String>,
}
