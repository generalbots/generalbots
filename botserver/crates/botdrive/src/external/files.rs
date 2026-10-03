//! The mirrored listing: what the Drive app's External tab reads.
//!
//! Split out of [`super::db`] because the two halves move at different rates —
//! connection state changes when a user reconnects, while the file mirror is
//! written on every sync pass and read on every listing.
//!
//! Rows are keyed on (branch_id, provider, remote_id): re-syncing an unchanged
//! item is a no-op because the upsert skips identical revisions, which is what
//! keeps a delta feed cheap on the database side too.

use crate::external::types::{ExternalFile, Provider};
use chrono::{DateTime, Utc};
use diesel::sql_types::{BigInt, Bool, Nullable, Text, Timestamptz, Uuid as DieselUuid};
use diesel::{OptionalExtension, RunQueryDsl};
use uuid::Uuid;

/// Upsert one mirrored file. A re-sync of an unchanged revision is a no-op in
/// practice because the WHERE clause skips it.
pub fn upsert_file(
    conn: &mut diesel::PgConnection,
    branch_id: Uuid,
    provider: Provider,
    file: &ExternalFile,
) -> Result<(), String> {
    diesel::sql_query(
        "INSERT INTO external_drive_files
            (branch_id, provider, remote_id, parent_remote_id, name, path, mime_type,
             size_bytes, is_folder, revision, modified_at, synced_at)
         VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,NOW())
         ON CONFLICT (branch_id, provider, remote_id) DO UPDATE SET
            parent_remote_id = EXCLUDED.parent_remote_id,
            name = EXCLUDED.name,
            path = EXCLUDED.path,
            mime_type = EXCLUDED.mime_type,
            size_bytes = EXCLUDED.size_bytes,
            is_folder = EXCLUDED.is_folder,
            revision = EXCLUDED.revision,
            modified_at = EXCLUDED.modified_at,
            synced_at = NOW()
         WHERE external_drive_files.revision IS DISTINCT FROM EXCLUDED.revision
            OR external_drive_files.name IS DISTINCT FROM EXCLUDED.name",
    )
    .bind::<DieselUuid, _>(branch_id)
    .bind::<Text, _>(provider.as_str())
    .bind::<Text, _>(&file.remote_id)
    .bind::<Nullable<Text>, _>(file.parent_remote_id.as_deref())
    .bind::<Text, _>(&file.name)
    .bind::<Text, _>(&file.path)
    .bind::<Nullable<Text>, _>(file.mime_type.as_deref())
    .bind::<BigInt, _>(file.size_bytes)
    .bind::<Bool, _>(file.is_folder)
    .bind::<Nullable<Text>, _>(file.revision.as_deref())
    .bind::<Nullable<Timestamptz>, _>(file.modified_at)
    .execute(conn)
    .map_err(|e| format!("upsert external file: {e}"))?;
    Ok(())
}

/// Remove mirrored files the provider reported as deleted.
pub fn remove_files(
    conn: &mut diesel::PgConnection,
    branch_id: Uuid,
    provider: Provider,
    remote_ids: &[String],
) -> Result<u64, String> {
    if remote_ids.is_empty() {
        return Ok(0);
    }
    let ids: Vec<String> = remote_ids
        .iter()
        .filter(|id| !id.is_empty())
        .cloned()
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    if ids.is_empty() {
        return Ok(0);
    }
    let deleted = diesel::sql_query(
        "DELETE FROM external_drive_files
         WHERE branch_id = $1 AND provider = $2 AND remote_id = ANY($3)",
    )
    .bind::<DieselUuid, _>(branch_id)
    .bind::<Text, _>(provider.as_str())
    .bind::<diesel::sql_types::Array<Text>, _>(&ids)
    .execute(conn)
    .map_err(|e| format!("remove external files: {e}"))?;
    Ok(deleted as u64)
}

/// A file as returned to the UI.
#[derive(Debug, Clone, diesel::QueryableByName)]
pub struct FileRow {
    /// Row id.
    #[diesel(sql_type = DieselUuid)]
    pub id: Uuid,
    /// Provider slug.
    #[diesel(sql_type = Text)]
    pub provider: String,
    /// Provider id.
    #[diesel(sql_type = Text)]
    pub remote_id: String,
    /// File name.
    #[diesel(sql_type = Text)]
    pub name: String,
    /// Path relative to the provider root.
    #[diesel(sql_type = Text)]
    pub path: String,
    /// MIME type.
    #[diesel(sql_type = Nullable<Text>)]
    pub mime_type: Option<String>,
    /// Size in bytes.
    #[diesel(sql_type = BigInt)]
    pub size_bytes: i64,
    /// True for folders.
    #[diesel(sql_type = Bool)]
    pub is_folder: bool,
    /// Last modification.
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub modified_at: Option<DateTime<Utc>>,
}

/// List mirrored files, optionally scoped to one provider and one folder.
pub fn list_files(
    conn: &mut diesel::PgConnection,
    branch_id: Uuid,
    provider: Option<Provider>,
    prefix: Option<&str>,
    limit: i64,
) -> Result<Vec<FileRow>, String> {
    let prefix = prefix.unwrap_or("").trim_matches('/');
    let rows: Vec<FileRow> = diesel::sql_query(
        "SELECT id, provider, remote_id, name, path, mime_type, size_bytes, is_folder, modified_at
         FROM external_drive_files
         WHERE branch_id = $1
           AND ($2::text IS NULL OR provider = $2)
           AND ($3::text = '' OR path LIKE $3 || '/%' OR path = $3)
         ORDER BY is_folder DESC, path ASC
         LIMIT $4",
    )
    .bind::<DieselUuid, _>(branch_id)
    .bind::<Nullable<Text>, _>(provider.map(|p| p.as_str()))
    .bind::<Text, _>(prefix)
    .bind::<BigInt, _>(limit.clamp(1, 5000))
    .load(conn)
    .map_err(|e| format!("list external files: {e}"))?;
    Ok(rows)
}

/// Look up one mirrored file inside the caller's branch.
pub fn get_file(
    conn: &mut diesel::PgConnection,
    branch_id: Uuid,
    provider: Provider,
    remote_id: &str,
) -> Result<Option<FileRow>, String> {
    diesel::sql_query(
        "SELECT id, provider, remote_id, name, path, mime_type, size_bytes, is_folder, modified_at
         FROM external_drive_files
         WHERE branch_id = $1 AND provider = $2 AND remote_id = $3",
    )
    .bind::<DieselUuid, _>(branch_id)
    .bind::<Text, _>(provider.as_str())
    .bind::<Text, _>(remote_id)
    .get_result(conn)
    .optional()
    .map_err(|e| format!("get external file: {e}"))
}

/// Look up one mirrored file by its own row id, inside the caller's branch.
pub fn get_file_by_id(
    conn: &mut diesel::PgConnection,
    branch_id: Uuid,
    provider: Provider,
    row_id: Uuid,
) -> Result<Option<FileRow>, String> {
    diesel::sql_query(
        "SELECT id, provider, remote_id, name, path, mime_type, size_bytes, is_folder, modified_at
         FROM external_drive_files
         WHERE branch_id = $1 AND provider = $2 AND id = $3",
    )
    .bind::<DieselUuid, _>(branch_id)
    .bind::<Text, _>(provider.as_str())
    .bind::<DieselUuid, _>(row_id)
    .get_result(conn)
    .optional()
    .map_err(|e| format!("get external file by id: {e}"))
}

/// How many files are mirrored for a provider — the number the UI shows next to
/// the connection card.
pub fn count_files(
    conn: &mut diesel::PgConnection,
    branch_id: Uuid,
    provider: Provider,
) -> Result<i64, String> {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = BigInt)]
        total: i64,
    }
    let row: Row = diesel::sql_query(
        "SELECT COUNT(*) AS total FROM external_drive_files
         WHERE branch_id = $1 AND provider = $2",
    )
    .bind::<DieselUuid, _>(branch_id)
    .bind::<Text, _>(provider.as_str())
    .get_result(conn)
    .map_err(|e| format!("count external files: {e}"))?;
    Ok(row.total)
}
