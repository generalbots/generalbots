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
                drive_files::etag.eq(etag),
                drive_files::last_modified.eq(last_modified),
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
            .first(&mut conn)
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
}
