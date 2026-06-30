use botcore::shared::DbPool;
use chrono::{DateTime, Utc};
use diesel::dsl::{max, sql};
use diesel::prelude::*;

diesel::table! {
    drive_files (id) {
        id -> Uuid,
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

pub use botcore::shared::schema::drive::DriveFile;

pub struct FileUpsertParams {
    pub bot_id: uuid::Uuid,
    pub file_path: String,
    pub file_type: String,
    pub etag: Option<String>,
    pub last_modified: Option<DateTime<Utc>>,
    pub indexed: bool,
    pub fail_count: i32,
    pub last_failed_at: Option<DateTime<Utc>>,
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

    pub fn get_file_state(&self, file_path: &str) -> Option<DriveFile> {
        let mut conn = match self.pool.get() {
            Ok(c) => c,
            Err(_) => return None,
        };

        drive_files::table
            .filter(drive_files::file_path.eq(file_path))
            .first(&mut conn)
            .ok()
    }

    pub fn upsert_file(
        &self,
        bot_id: uuid::Uuid,
        file_path: &str,
        file_type: &str,
        etag: Option<String>,
        last_modified: Option<DateTime<Utc>>,
    ) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        diesel::sql_query(
            "INSERT INTO drive_files (bot_id, file_path, file_type, etag, last_modified, indexed, fail_count, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, false, 0, NOW(), NOW()) \
             ON CONFLICT (bot_id, file_path) DO UPDATE SET \
             file_type = EXCLUDED.file_type, etag = EXCLUDED.etag, \
             last_modified = EXCLUDED.last_modified, updated_at = NOW()"
        )
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .bind::<diesel::sql_types::Text, _>(file_path)
        .bind::<diesel::sql_types::Text, _>(file_type)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(etag)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(last_modified)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn mark_indexed(&self, file_path: &str, etag: Option<String>) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        diesel::update(drive_files::table)
            .filter(drive_files::file_path.eq(file_path))
            .set((
                drive_files::indexed.eq(true),
                drive_files::etag.eq(etag),
                drive_files::fail_count.eq(0),
                drive_files::last_failed_at.eq(None::<DateTime<Utc>>),
                drive_files::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn mark_failed(&self, file_path: &str) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        diesel::update(drive_files::table)
            .filter(drive_files::file_path.eq(file_path))
            .set((
                drive_files::fail_count.eq(sql("fail_count + 1")),
                drive_files::last_failed_at.eq(Some(Utc::now())),
                drive_files::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn get_max_fail_count(&self) -> i32 {
        let mut conn = match self.pool.get() {
            Ok(c) => c,
            Err(_) => return 0,
        };

        drive_files::table
            .select(max(drive_files::fail_count))
            .first::<Option<i32>>(&mut conn)
            .unwrap_or(Some(0))
            .unwrap_or(0)
    }

    pub fn get_files_to_index(&self) -> Vec<DriveFile> {
        let mut conn = match self.pool.get() {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        drive_files::table
            .filter(drive_files::indexed.eq(false))
            .load(&mut conn)
            .unwrap_or_default()
    }

    pub fn delete_file(&self, file_path: &str) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        diesel::delete(drive_files::table)
            .filter(drive_files::file_path.eq(file_path))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(())
    }

    pub fn get_all_files_for_bot(&self) -> Vec<DriveFile> {
        let mut conn = match self.pool.get() {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        drive_files::table
            .load(&mut conn)
            .unwrap_or_default()
    }

    pub fn get_files_by_type(&self, file_type: &str) -> Vec<DriveFile> {
        let mut conn = match self.pool.get() {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        drive_files::table
            .filter(drive_files::file_type.eq(file_type))
            .load(&mut conn)
            .unwrap_or_default()
    }

    /// Check if a file exists for the given bot and path
    pub fn has_file(&self, file_path: &str) -> bool {
        self.get_file_state(file_path).is_some()
    }

    pub fn upsert_file_full(
        &self,
        params: FileUpsertParams,
    ) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        diesel::sql_query(
            "INSERT INTO drive_files (bot_id, file_path, file_type, etag, last_modified, indexed, fail_count, last_failed_at, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), NOW()) \
             ON CONFLICT (bot_id, file_path) DO UPDATE SET \
             etag = EXCLUDED.etag, last_modified = EXCLUDED.last_modified, \
             indexed = EXCLUDED.indexed, fail_count = EXCLUDED.fail_count, \
             last_failed_at = EXCLUDED.last_failed_at, updated_at = NOW()"
        )
        .bind::<diesel::sql_types::Uuid, _>(params.bot_id)
        .bind::<diesel::sql_types::Text, _>(&params.file_path)
        .bind::<diesel::sql_types::Text, _>(&params.file_type)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&params.etag)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(params.last_modified)
        .bind::<diesel::sql_types::Bool, _>(params.indexed)
        .bind::<diesel::sql_types::Int4, _>(params.fail_count)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(params.last_failed_at)
        .execute(&mut conn)
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    /// Mark all files matching a path pattern as indexed (for KB folder indexing)
    /// Does NOT update ETag — individual file ETags are already set by upsert_file during scan
    pub fn mark_indexed_by_pattern(&self, pattern: &str) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        diesel::update(drive_files::table)
            .filter(drive_files::file_path.like(format!("%{pattern}%")))
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
    pub fn mark_failed_by_pattern(&self, pattern: &str) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        diesel::update(drive_files::table)
            .filter(drive_files::file_path.like(format!("%{pattern}%")))
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
    pub fn get_files_by_prefix(&self, prefix: &str) -> Vec<DriveFile> {
        let mut conn = match self.pool.get() {
            Ok(c) => c,
            Err(_) => return vec![],
        };

        drive_files::table
            .filter(drive_files::file_path.like(format!("{prefix}%")))
            .load(&mut conn)
            .unwrap_or_default()
    }

    /// Delete all files for a bot whose path starts with the given prefix
    pub fn delete_by_prefix(&self, prefix: &str) -> Result<usize, String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        diesel::delete(drive_files::table)
            .filter(drive_files::file_path.like(format!("{prefix}%")))
            .execute(&mut conn)
            .map_err(|e| e.to_string())
    }

    /// Check if any files exist with the given prefix
    pub fn has_files_with_prefix(&self, prefix: &str) -> bool {
        !self.get_files_by_prefix(prefix).is_empty()
    }

    /// Reset fail_count to 0 for files that failed >= max_fail_count times
    /// and whose last_failed_at is older than the given cooldown duration.
    /// Returns the number of files reset.
    pub fn reset_failed_files(
        &self,
        max_fail_count: i32,
        cooldown: chrono::Duration,
    ) -> Result<usize, String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        let cutoff = Utc::now() - cooldown;

        diesel::update(drive_files::table)
            .filter(
                drive_files::fail_count.ge(max_fail_count)
                    .and(drive_files::last_failed_at.is_not_null())
                    .and(drive_files::last_failed_at.lt(cutoff)),
            )
            .set((
                drive_files::fail_count.eq(0),
                drive_files::indexed.eq(false),
                drive_files::last_failed_at.eq(None::<DateTime<Utc>>),
                drive_files::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .map_err(|e| e.to_string())
    }

    /// Reset fail_count to 0 for a specific file path.
    pub fn reset_file_fail_count(&self, file_path: &str) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        diesel::update(drive_files::table)
            .filter(drive_files::file_path.eq(file_path))
            .set((
                drive_files::fail_count.eq(0),
                drive_files::indexed.eq(false),
                drive_files::last_failed_at.eq(None::<DateTime<Utc>>),
                drive_files::updated_at.eq(Utc::now()),
            ))
            .execute(&mut conn)
            .map_err(|e| e.to_string())?;

        Ok(())
    }
}
