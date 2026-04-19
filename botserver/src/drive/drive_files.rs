use crate::core::shared::DbPool;
use chrono::{DateTime, Utc};
use diesel::dsl::{max, sql};
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

pub mod dsl {
    pub use super::drive_files::*;
}

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

pub struct DriveFileRepository {
    pool: DbPool,
}

impl std::fmt::Debug for DriveFileRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DriveFileRepository").finish()
    }
}

impl DriveFileRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    pub fn get_file_state(&self, bot_id: Uuid, file_path: &str) -> Option<DriveFile> {
        let mut conn = match self.pool.get() {
            Ok(c) => c,
            Err(_) => return None,
        };

        drive_files::table
            .filter(
                drive_files::bot_id
                    .eq(bot_id)
                    .and(drive_files::file_path.eq(file_path)),
            )
            .first(&mut conn)
            .ok()
    }

    pub fn upsert_file(
        &self,
        bot_id: Uuid,
        file_path: &str,
        file_type: &str,
        etag: Option<String>,
        last_modified: Option<DateTime<Utc>>,
    ) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        let now = Utc::now();
        let etag_clone = etag.clone();
        let last_modified_clone = last_modified.clone();

        diesel::insert_into(drive_files::table)
            .values((
                drive_files::bot_id.eq(bot_id),
                drive_files::file_path.eq(file_path),
                drive_files::file_type.eq(file_type),
                drive_files::etag.eq(etag),
                drive_files::last_modified.eq(last_modified),
                drive_files::indexed.eq(false),
                drive_files::fail_count.eq(0),
                drive_files::created_at.eq(now),
                drive_files::updated_at.eq(now),
            ))
            .on_conflict((drive_files::bot_id, drive_files::file_path))
            .do_update()
            .set((
                drive_files::etag.eq(etag_clone),
                drive_files::last_modified.eq(last_modified_clone),
                drive_files::updated_at.eq(now),
            ))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn mark_indexed(&self, bot_id: Uuid, file_path: &str) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        diesel::update(drive_files::table)
            .filter(
                drive_files::bot_id
                    .eq(bot_id)
                    .and(drive_files::file_path.eq(file_path)),
            )
            .set((
                drive_files::indexed.eq(true),
                drive_files::fail_count.eq(0),
                drive_files::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn mark_failed(&self, bot_id: Uuid, file_path: &str) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        diesel::update(drive_files::table)
            .filter(
                drive_files::bot_id
                    .eq(bot_id)
                    .and(drive_files::file_path.eq(file_path)),
            )
            .set((
                drive_files::fail_count.eq(sql("fail_count + 1")),
                drive_files::last_failed_at.eq(Some(Utc::now())),
                drive_files::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn get_max_fail_count(&self, bot_id: Uuid) -> i32 {
        let mut conn = match self.pool.get() {
            Ok(c) => c,
            Err(_) => return 0,
        };

        drive_files::table
            .filter(drive_files::bot_id.eq(bot_id))
            .select(max(drive_files::fail_count))
            .first::<Option<i32>>(&mut conn)
            .unwrap_or(Some(0))
            .unwrap_or(0)
    }

    pub fn get_files_to_index(&self, bot_id: Uuid) -> Vec<DriveFile> {
        let mut conn = match self.pool.get() {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        drive_files::table
            .filter(
                drive_files::bot_id
                    .eq(bot_id)
                    .and(drive_files::indexed.eq(false)),
            )
            .load(&mut conn)
            .unwrap_or_default()
    }

    pub fn delete_file(&self, bot_id: Uuid, file_path: &str) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        diesel::delete(drive_files::table)
            .filter(
                drive_files::bot_id
                    .eq(bot_id)
                    .and(drive_files::file_path.eq(file_path)),
            )
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn get_all_files_for_bot(&self, bot_id: Uuid) -> Vec<DriveFile> {
        let mut conn = match self.pool.get() {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        drive_files::table
            .filter(drive_files::bot_id.eq(bot_id))
            .load(&mut conn)
            .unwrap_or_default()
    }

    pub fn get_files_by_type(&self, bot_id: Uuid, file_type: &str) -> Vec<DriveFile> {
        let mut conn = match self.pool.get() {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        drive_files::table
            .filter(
                drive_files::bot_id
                    .eq(bot_id)
                    .and(drive_files::file_type.eq(file_type)),
            )
            .load(&mut conn)
            .unwrap_or_default()
    }

    /// Check if a file exists for the given bot and path
    pub fn has_file(&self, bot_id: Uuid, file_path: &str) -> bool {
        self.get_file_state(bot_id, file_path).is_some()
    }

    /// Upsert a file with full state (including indexed and fail_count)
    pub fn upsert_file_full(
        &self,
        bot_id: Uuid,
        file_path: &str,
        file_type: &str,
        etag: Option<String>,
        last_modified: Option<DateTime<Utc>>,
        indexed: bool,
        fail_count: i32,
        last_failed_at: Option<DateTime<Utc>>,
    ) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        let now = Utc::now();

        diesel::insert_into(drive_files::table)
            .values((
                drive_files::bot_id.eq(bot_id),
                drive_files::file_path.eq(file_path),
                drive_files::file_type.eq(file_type),
                drive_files::etag.eq(&etag),
                drive_files::last_modified.eq(last_modified),
                drive_files::indexed.eq(indexed),
                drive_files::fail_count.eq(fail_count),
                drive_files::last_failed_at.eq(last_failed_at),
                drive_files::created_at.eq(now),
                drive_files::updated_at.eq(now),
            ))
            .on_conflict((drive_files::bot_id, drive_files::file_path))
            .do_update()
            .set((
                drive_files::etag.eq(&etag),
                drive_files::last_modified.eq(last_modified),
                drive_files::indexed.eq(indexed),
                drive_files::fail_count.eq(fail_count),
                drive_files::last_failed_at.eq(last_failed_at),
                drive_files::updated_at.eq(now),
            ))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Mark all files matching a path pattern as indexed (for KB folder indexing)
    pub fn mark_indexed_by_pattern(&self, bot_id: Uuid, pattern: &str) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        diesel::update(drive_files::table)
            .filter(
                drive_files::bot_id
                    .eq(bot_id)
                    .and(drive_files::file_path.like(format!("%{pattern}%"))),
            )
            .set((
                drive_files::indexed.eq(true),
                drive_files::fail_count.eq(0),
                drive_files::last_failed_at.eq(None::<DateTime<Utc>>),
                drive_files::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Mark all files matching a path pattern as failed (increment fail_count)
    pub fn mark_failed_by_pattern(&self, bot_id: Uuid, pattern: &str) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        diesel::update(drive_files::table)
            .filter(
                drive_files::bot_id
                    .eq(bot_id)
                    .and(drive_files::file_path.like(format!("%{pattern}%"))),
            )
            .set((
                drive_files::fail_count.eq(sql("fail_count + 1")),
                drive_files::last_failed_at.eq(Some(Utc::now())),
                drive_files::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Get all files for a bot whose path starts with the given prefix
    pub fn get_files_by_prefix(&self, bot_id: Uuid, prefix: &str) -> Vec<DriveFile> {
        let mut conn = match self.pool.get() {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        drive_files::table
            .filter(
                drive_files::bot_id
                    .eq(bot_id)
                    .and(drive_files::file_path.like(format!("{prefix}%"))),
            )
            .load(&mut conn)
            .unwrap_or_default()
    }

    /// Delete all files for a bot whose path starts with the given prefix
    pub fn delete_by_prefix(&self, bot_id: Uuid, prefix: &str) -> Result<usize, String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        diesel::delete(drive_files::table)
            .filter(
                drive_files::bot_id
                    .eq(bot_id)
                    .and(drive_files::file_path.like(format!("{prefix}%"))),
            )
            .execute(&mut conn)
            .map_err(|e| e.to_string())
    }

    /// Check if any files exist with the given prefix
    pub fn has_files_with_prefix(&self, bot_id: Uuid, prefix: &str) -> bool {
        !self.get_files_by_prefix(bot_id, prefix).is_empty()
    }
}
