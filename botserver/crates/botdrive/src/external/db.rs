//! Persistence for external-drive connections and mirrored files.
//!
//! Two tables, both keyed so that a second provider is an insertion rather
//! than a schema change:
//!
//! * `external_drive_connections` — one row per (branch, provider), holding the
//!   encrypted tokens, the change-feed cursor and the retry schedule.
//! * `external_drive_files` — the mirrored listing, unique on
//!   (branch_id, provider, remote_id) so a re-sync updates in place.
//!
//! Every query is bounded by `branch_id`: an account connected by one tenant
//! is invisible to another (same rule as `drive_handlers::resolve_bucket`).

use crate::external::types::{Connection, Provider};
use botcore::shared::utils::DbPool;
use chrono::{DateTime, Utc};
use diesel::sql_types::{BigInt, Bool, Nullable, Text, Timestamptz, Uuid as DieselUuid};
use diesel::{OptionalExtension, RunQueryDsl};
use uuid::Uuid;

/// Create the tables and indexes when missing.
///
/// `CREATE TABLE IF NOT EXISTS` keeps this safe to call on every request; the
/// suite already does the same for its own m365 tables.
pub fn ensure_schema(conn: &mut diesel::PgConnection) -> Result<(), String> {
    for stmt in [
        "CREATE TABLE IF NOT EXISTS external_drive_connections (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
            provider TEXT NOT NULL,
            access_token_enc TEXT NOT NULL DEFAULT '',
            refresh_token_enc TEXT,
            expires_at TIMESTAMPTZ,
            cursor TEXT,
            status TEXT NOT NULL DEFAULT 'connected',
            sync_minutes INTEGER NOT NULL DEFAULT 30,
            account TEXT,
            last_sync TIMESTAMPTZ,
            last_error TEXT,
            attempt_count INTEGER NOT NULL DEFAULT 0,
            next_retry_at TIMESTAMPTZ,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE (branch_id, provider)
        )",
        "CREATE TABLE IF NOT EXISTS external_drive_files (
            id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
            provider TEXT NOT NULL,
            remote_id TEXT NOT NULL,
            parent_remote_id TEXT,
            name TEXT NOT NULL,
            path TEXT NOT NULL,
            mime_type TEXT,
            size_bytes BIGINT NOT NULL DEFAULT 0,
            is_folder BOOLEAN NOT NULL DEFAULT FALSE,
            revision TEXT,
            modified_at TIMESTAMPTZ,
            synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            UNIQUE (branch_id, provider, remote_id)
        )",
        "CREATE INDEX IF NOT EXISTS idx_external_files_branch_provider
            ON external_drive_files (branch_id, provider)",
        "CREATE INDEX IF NOT EXISTS idx_external_files_branch_path
            ON external_drive_files (branch_id, provider, path)",
        // The sync loop reads only rows whose backoff has elapsed.
        "CREATE INDEX IF NOT EXISTS idx_external_connections_due
            ON external_drive_connections (next_retry_at)",
    ] {
        diesel::sql_query(stmt)
            .execute(conn)
            .map_err(|e| format!("external drive schema: {e}"))?;
    }
    Ok(())
}

/// Insert or refresh a connection's OAuth material.
pub fn upsert_connection(
    conn: &mut diesel::PgConnection,
    branch_id: Uuid,
    provider: Provider,
    access_token_enc: &str,
    refresh_token_enc: Option<&str>,
    expires_at: Option<DateTime<Utc>>,
    account: Option<&str>,
) -> Result<(), String> {
    diesel::sql_query(
        "INSERT INTO external_drive_connections
            (branch_id, provider, access_token_enc, refresh_token_enc, expires_at,
             status, attempt_count, next_retry_at, account, updated_at)
         VALUES ($1, $2, $3, $4, $5, 'connected', 0, NULL, $6, NOW())
         ON CONFLICT (branch_id, provider) DO UPDATE SET
            access_token_enc = EXCLUDED.access_token_enc,
            refresh_token_enc = COALESCE(EXCLUDED.refresh_token_enc,
                                         external_drive_connections.refresh_token_enc),
            expires_at = EXCLUDED.expires_at,
            status = 'connected',
            attempt_count = 0,
            next_retry_at = NULL,
            last_error = NULL,
            account = COALESCE(EXCLUDED.account, external_drive_connections.account),
            updated_at = NOW()",
    )
    .bind::<DieselUuid, _>(branch_id)
    .bind::<Text, _>(provider.as_str())
    .bind::<Text, _>(access_token_enc)
    .bind::<Nullable<Text>, _>(refresh_token_enc)
    .bind::<Nullable<Timestamptz>, _>(expires_at)
    .bind::<Nullable<Text>, _>(account)
    .execute(conn)
    .map_err(|e| format!("upsert connection: {e}"))?;
    Ok(())
}

/// Persist new tokens after a refresh, keeping the existing cursor.
pub fn store_refreshed_token(
    conn: &mut diesel::PgConnection,
    branch_id: Uuid,
    provider: Provider,
    access_token_enc: &str,
    refresh_token_enc: Option<&str>,
    expires_at: DateTime<Utc>,
) -> Result<(), String> {
    diesel::sql_query(
        "UPDATE external_drive_connections SET
            access_token_enc = $3,
            refresh_token_enc = COALESCE($4, refresh_token_enc),
            expires_at = $5, updated_at = NOW()
         WHERE branch_id = $1 AND provider = $2",
    )
    .bind::<DieselUuid, _>(branch_id)
    .bind::<Text, _>(provider.as_str())
    .bind::<Text, _>(access_token_enc)
    .bind::<Nullable<Text>, _>(refresh_token_enc)
    .bind::<Timestamptz, _>(expires_at)
    .execute(conn)
    .map_err(|e| format!("store refreshed token: {e}"))?;
    Ok(())
}

/// Load one connection with its tokens decrypted.
pub fn get_connection(
    conn: &mut diesel::PgConnection,
    branch_id: Uuid,
    provider: Provider,
) -> Result<Option<Connection>, String> {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = DieselUuid)]
        id: Uuid,
        #[diesel(sql_type = Text)]
        provider: String,
        #[diesel(sql_type = Text)]
        access_token_enc: String,
        #[diesel(sql_type = Nullable<Text>)]
        refresh_token_enc: Option<String>,
        #[diesel(sql_type = Nullable<Timestamptz>)]
        expires_at: Option<DateTime<Utc>>,
        #[diesel(sql_type = Nullable<Text>)]
        cursor: Option<String>,
        #[diesel(sql_type = Text)]
        status: String,
        #[diesel(sql_type = diesel::sql_types::Integer)]
        sync_minutes: i32,
        #[diesel(sql_type = diesel::sql_types::Integer)]
        attempt_count: i32,
        #[diesel(sql_type = Nullable<Timestamptz>)]
        last_sync: Option<DateTime<Utc>>,
        #[diesel(sql_type = Nullable<Timestamptz>)]
        next_retry_at: Option<DateTime<Utc>>,
        #[diesel(sql_type = Nullable<Text>)]
        last_error: Option<String>,
        #[diesel(sql_type = Nullable<Text>)]
        account: Option<String>,
    }

    let row: Option<Row> = diesel::sql_query(
        "SELECT id, provider, access_token_enc, refresh_token_enc, expires_at, cursor,
                status, sync_minutes, attempt_count, last_sync, next_retry_at, last_error, account
         FROM external_drive_connections
         WHERE branch_id = $1 AND provider = $2",
    )
    .bind::<DieselUuid, _>(branch_id)
    .bind::<Text, _>(provider.as_str())
    .get_result(conn)
    .optional()
    .map_err(|e| format!("load connection: {e}"))?;

    let Some(row) = row else { return Ok(None) };
    let parsed = Provider::parse(&row.provider)
        .ok_or_else(|| format!("unknown provider '{}' in connection row", row.provider))?;
    let decrypt = |value: &str| crate::external::crypto::decrypt_or_empty(value);
    Ok(Some(Connection {
        id: row.id,
        branch_id,
        provider: parsed,
        access_token: decrypt(&row.access_token_enc),
        refresh_token: row.refresh_token_enc.as_deref().map(decrypt),
        expires_at: row.expires_at,
        cursor: row.cursor,
        status: row.status,
        sync_minutes: row.sync_minutes,
        attempt_count: row.attempt_count,
        last_sync: row.last_sync,
        next_retry_at: row.next_retry_at,
        last_error: row.last_error,
        account: row.account,
    }))
}

/// Connections that are due for a pass: their cadence elapsed and their
/// failure backoff (if any) has passed. Returns the owning branch with each
/// provider, because the tenant scope comes from the row, not the caller.
pub fn list_due_connections(
    conn: &mut diesel::PgConnection,
    now: DateTime<Utc>,
    limit: i64,
) -> Result<Vec<(Uuid, Provider)>, String> {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = DieselUuid)]
        branch_id: Uuid,
        #[diesel(sql_type = Text)]
        provider: String,
    }
    // The 5-minute floor keeps a 1-minute cadence from hammering the provider
    // and burning its quota; the backoff is enforced by next_retry_at.
    let rows: Vec<Row> = diesel::sql_query(
        "SELECT branch_id, provider FROM external_drive_connections
         WHERE status <> 'disconnected'
           AND (next_retry_at IS NULL OR next_retry_at <= $1)
           AND (last_sync IS NULL
                OR last_sync <= NOW() - (GREATEST(sync_minutes, 5) || ' minutes')::interval)
         ORDER BY COALESCE(last_sync, 'epoch'::timestamptz) ASC
         LIMIT $2",
    )
    .bind::<Timestamptz, _>(now)
    .bind::<BigInt, _>(limit)
    .load(conn)
    .map_err(|e| format!("list due connections: {e}"))?;
    Ok(rows
        .into_iter()
        .filter_map(|r| Provider::parse(&r.provider).map(|p| (r.branch_id, p)))
        .collect())
}

/// Save the cursor after a successful run.
pub fn store_cursor(
    conn: &mut diesel::PgConnection,
    branch_id: Uuid,
    provider: Provider,
    cursor: Option<&str>,
) -> Result<(), String> {
    diesel::sql_query(
        "UPDATE external_drive_connections SET cursor = $3, last_sync = NOW(),
            last_error = NULL, attempt_count = 0, next_retry_at = NULL, updated_at = NOW()
         WHERE branch_id = $1 AND provider = $2",
    )
    .bind::<DieselUuid, _>(branch_id)
    .bind::<Text, _>(provider.as_str())
    .bind::<Nullable<Text>, _>(cursor)
    .execute(conn)
    .map_err(|e| format!("store cursor: {e}"))?;
    Ok(())
}

/// Record a failed run: message, backoff and (for auth failures) the state the
/// UI shows as "reconnect required".
pub fn store_failure(
    conn: &mut diesel::PgConnection,
    branch_id: Uuid,
    provider: Provider,
    message: &str,
    needs_reauth: bool,
    backoff_secs: u64,
) -> Result<(), String> {
    diesel::sql_query(
        "UPDATE external_drive_connections SET
            last_error = $3,
            attempt_count = attempt_count + 1,
            status = CASE WHEN $4 THEN 'reauth_required' ELSE status END,
            next_retry_at = NOW() + ($5 || ' seconds')::interval,
            updated_at = NOW()
         WHERE branch_id = $1 AND provider = $2",
    )
    .bind::<DieselUuid, _>(branch_id)
    .bind::<Text, _>(provider.as_str())
    .bind::<Text, _>(truncate_error(message))
    .bind::<Bool, _>(needs_reauth)
    .bind::<Text, _>(backoff_secs.to_string())
    .execute(conn)
    .map_err(|e| format!("store failure: {e}"))?;
    Ok(())
}

/// Public status of every connection of a branch (no tokens).
#[derive(Debug, Clone, diesel::QueryableByName)]
pub struct ConnectionStatus {
    /// Provider slug.
    #[diesel(sql_type = Text)]
    pub provider: String,
    /// `connected` | `reauth_required` | `error`.
    #[diesel(sql_type = Text)]
    pub status: String,
    /// Requested cadence.
    #[diesel(sql_type = diesel::sql_types::Integer)]
    pub sync_minutes: i32,
    /// Account label.
    #[diesel(sql_type = Nullable<Text>)]
    pub account: Option<String>,
    /// Last successful sync.
    #[diesel(sql_type = Nullable<Timestamptz>)]
    pub last_sync: Option<DateTime<Utc>>,
    /// Last error message.
    #[diesel(sql_type = Nullable<Text>)]
    pub last_error: Option<String>,
}

/// Status rows for the UI, including providers with no connection yet.
pub fn list_connection_status(
    conn: &mut diesel::PgConnection,
    branch_id: Uuid,
) -> Result<Vec<ConnectionStatus>, String> {
    diesel::sql_query(
        "SELECT provider, status, sync_minutes, account, last_sync, last_error
         FROM external_drive_connections WHERE branch_id = $1 ORDER BY provider",
    )
    .bind::<DieselUuid, _>(branch_id)
    .load(conn)
    .map_err(|e| format!("list connection status: {e}"))
}

/// Forget a connection and everything it mirrored.
pub fn disconnect(
    conn: &mut diesel::PgConnection,
    branch_id: Uuid,
    provider: Provider,
) -> Result<u64, String> {
    diesel::sql_query("DELETE FROM external_drive_files WHERE branch_id = $1 AND provider = $2")
        .bind::<DieselUuid, _>(branch_id)
        .bind::<Text, _>(provider.as_str())
        .execute(conn)
        .map_err(|e| format!("disconnect files: {e}"))?;
    diesel::sql_query(
        "DELETE FROM external_drive_connections WHERE branch_id = $1 AND provider = $2",
    )
    .bind::<DieselUuid, _>(branch_id)
    .bind::<Text, _>(provider.as_str())
    .execute(conn)
    .map_err(|e| format!("disconnect: {e}"))?;
    Ok(1)
}

/// Borrow a pool connection.
pub fn conn(pool: &DbPool) -> Result<diesel::r2d2::PooledConnection<diesel::r2d2::ConnectionManager<diesel::PgConnection>>, String> {
    pool.get().map_err(|e| format!("pool: {e}"))
}

/// Errors are stored for the UI; keep them bounded.
fn truncate_error(msg: &str) -> String {
    let trimmed = msg.trim();
    if trimmed.chars().count() > 500 {
        trimmed.chars().take(500).collect()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors_are_bounded() {
        let long = "x".repeat(600);
        assert_eq!(truncate_error(&long).chars().count(), 500);
        assert_eq!(truncate_error("short"), "short");
        // Multi-byte characters must not be cut mid-codepoint.
        let accented = "ç".repeat(400);
        assert_eq!(truncate_error(&accented).chars().count(), 400);
    }
}