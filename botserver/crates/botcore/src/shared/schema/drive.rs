use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

diesel::table! {
    drive_files (id) {
        id -> Uuid,
        bot_id -> Uuid,
        file_path -> Text,
        file_type -> Varchar,
        etag -> Nullable<Text>,
        last_modified -> Nullable<Timestamptz>,
        file_size -> Nullable<Int8>,
        indexed -> Bool,
        fail_count -> Int4,
        last_failed_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

// Query-only struct (no defaults needed)
#[derive(Queryable, Debug, Clone)]
pub struct DriveFile {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub file_path: String,
    pub file_type: String,
    pub etag: Option<String>,
    pub last_modified: Option<DateTime<Utc>>,
    pub file_size: Option<i64>,
    pub indexed: bool,
    pub fail_count: i32,
    pub last_failed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Insert struct - uses diesel defaults
#[derive(Insertable, Debug)]
#[diesel(table_name = drive_files)]
pub struct NewDriveFile {
    pub bot_id: Uuid,
    pub file_path: String,
    pub file_type: String,
    pub etag: Option<String>,
    pub last_modified: Option<DateTime<Utc>>,
    pub file_size: Option<i64>,
    pub indexed: Option<bool>,
    pub fail_count: Option<i32>,
}

// Update struct
#[derive(AsChangeset, Debug)]
#[diesel(table_name = drive_files)]
pub struct DriveFileUpdate {
    pub etag: Option<String>,
    pub last_modified: Option<DateTime<Utc>>,
    pub file_size: Option<i64>,
    pub indexed: Option<bool>,
    pub fail_count: Option<i32>,
    pub last_failed_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}
