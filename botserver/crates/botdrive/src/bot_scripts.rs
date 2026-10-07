//! `bot_scripts` — the compile queue that replaces `drive_files.etag` (#1475).
//!
//! `DriveCompiler` used the `drive_files` *inventory* as its work queue and
//! found candidates with a leading-wildcard `LIKE '%.gbdialog/%'`, which can
//! never use a btree index. Since 6.5.25 dropped `idx_drive_files_type` every
//! 5 s tick sequentially scanned the whole table and discarded the KB, media
//! and shared-file rows that were never candidates. Two further defects shared
//! the same `etag` column:
//!
//! - `git_bot_monitor::mark_for_compile` wrote a **git commit sha** while the
//!   DriveMonitor wrote an **S3 ETag**, so each writer clobbered the other's
//!   version.
//! - the compiler recorded a new version only after a *successful* compile, so
//!   a permanently broken script stayed "changed" forever and was retried on
//!   every tick.
//!
//! This module keeps the queue explicit: a `dirty` flag with a partial index
//! for an index-only scan of pending work, `source_version` separated from
//! `compiled_version` so a failure stops the spin, and leases so concurrent
//! ticks claim disjoint work.

use botcore::shared::DbPool;
use diesel::prelude::*;

/// How long a claimed script stays leased before another tick may retry it.
/// A compile is short; the lease only guards against overlapping ticks.
const DEFAULT_LEASE_SECS: i64 = 300;

/// Retry floor for a failing script. `fail_count`/`last_failed_at` feed the
/// backoff so a permanently broken `.bas` is not retried every 5 s forever.
const RETRY_FLOOR_SECS: i64 = 30;
const RETRY_CEILING_SECS: i64 = 1800;

/// Where a script's version came from. Kept apart so the two writers cannot
/// overwrite each other's meaning in a shared column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    /// Version is a git commit sha from `git_bot_monitor`.
    Git,
    /// Version is an S3 ETag from the DriveMonitor.
    Drive,
}

impl SourceKind {
    fn as_str(self) -> &'static str {
        match self {
            SourceKind::Git => "git",
            SourceKind::Drive => "drive",
        }
    }
}

/// A script awaiting (or undergoing) compilation.
#[derive(Debug, Clone)]
pub struct BotScript {
    pub branch_id: uuid::Uuid,
    pub bot_name: String,
    pub script_path: String,
    pub source_kind: String,
    pub source_version: String,
    pub compiled_version: Option<String>,
    pub fail_count: i32,
    pub last_failed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct ScriptRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: uuid::Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    bot_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    script_path: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    source_kind: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    source_version: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    compiled_version: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Int4)]
    fail_count: i32,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    last_failed_at: Option<chrono::DateTime<chrono::Utc>>,
}

pub struct BotScriptsRepository {
    pool: DbPool,
}

impl std::fmt::Debug for BotScriptsRepository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BotScriptsRepository").finish()
    }
}

impl BotScriptsRepository {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Reject paths that must never reach the compiler.
    ///
    /// `{bucket}/archive/{bot}-{stamp}/.gbdialog/...` holds the Drive copies
    /// `git_bot_monitor` archives when a bot's sources become git-owned; they
    /// are explicitly never read again (AGENTS.md). The DriveMonitor still
    /// lists them, so the queue has to refuse them — otherwise every archive
    /// pass resurrects its snapshots as work.
    pub fn is_compilable_path(script_path: &str) -> bool {
        script_path.ends_with(".bas")
            && !script_path.contains("/archive/")
            && !script_path.contains("/.gbdialog/")
    }

    /// Mark a script as needing a compile.
    ///
    /// Idempotent on `(branch_id, script_path)`. The `DO UPDATE` is gated on
    /// `compiled_version IS DISTINCT FROM EXCLUDED.source_version` — "the last
    /// version that compiled cleanly is not this version" — so re-observing an
    /// unchanged object does not requeue a script that is already compiled.
    /// Gating instead on `source_version`, or adding `OR NOT dirty`, would make
    /// every monitor tick requeue every clean script and recompile the world.
    pub fn enqueue(
        &self,
        branch_id: uuid::Uuid,
        bot_name: &str,
        script_path: &str,
        kind: SourceKind,
        source_version: &str,
    ) -> Result<bool, String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;

        let queued = diesel::sql_query(
            "INSERT INTO bot_scripts \
                 (branch_id, bot_name, script_path, source_kind, source_version, dirty) \
             VALUES ($1, $2, $3, $4, $5, TRUE) \
             ON CONFLICT (branch_id, script_path) DO UPDATE \
                SET source_kind = EXCLUDED.source_kind, \
                    source_version = EXCLUDED.source_version, \
                    bot_name = EXCLUDED.bot_name, \
                    dirty = TRUE, \
                    updated_at = NOW() \
              WHERE bot_scripts.compiled_version IS DISTINCT FROM EXCLUDED.source_version",
        )
        .bind::<diesel::sql_types::Uuid, _>(&branch_id)
        .bind::<diesel::sql_types::Text, _>(bot_name)
        .bind::<diesel::sql_types::Text, _>(script_path)
        .bind::<diesel::sql_types::Text, _>(kind.as_str())
        .bind::<diesel::sql_types::Text, _>(source_version)
        .execute(&mut conn)
        .map_err(|e| format!("enqueue {script_path}: {e}"))?;

        Ok(queued > 0)
    }

    /// Claim up to `limit` dirty scripts for `owner`, skipping any whose
    /// backoff window has not elapsed and taking a lease on the rest.
    ///
    /// The single `UPDATE ... WHERE id IN (SELECT ... FOR UPDATE SKIP LOCKED)`
    /// keeps two compiler ticks from claiming the same script.
    pub fn claim(
        &self,
        owner: &str,
        limit: i64,
        lease_secs: i64,
    ) -> Result<Vec<BotScript>, String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;
        let lease_secs = if lease_secs > 0 { lease_secs } else { DEFAULT_LEASE_SECS };
        let limit = limit.clamp(1, 500);

        // `make_interval` with explicit casts, not `($n || ' seconds')::interval`:
        // bind parameters are untyped at parse time, so `||` and `<<` cannot
        // resolve an operator ("could not determine data type of parameter $n")
        // and every tick failed with a syntax error.
        let rows = diesel::sql_query(
            "UPDATE bot_scripts SET lease_owner = $1, \
                    lease_expires_at = NOW() + make_interval(secs => $2::double precision), \
                    updated_at = NOW() \
              WHERE id IN ( \
                    SELECT id FROM bot_scripts \
                     WHERE dirty \
                       AND (lease_expires_at IS NULL OR lease_expires_at < NOW()) \
                       AND (last_failed_at IS NULL \
                            OR last_failed_at < NOW() - make_interval(secs => LEAST( \
                                  $3::bigint * (1 << LEAST(fail_count, 6)), \
                                  $4::bigint)::double precision)) \
                     ORDER BY updated_at ASC \
                     LIMIT $5 \
                     FOR UPDATE SKIP LOCKED ) \
             RETURNING branch_id, bot_name, script_path, source_kind, source_version, \
                       compiled_version, fail_count, last_failed_at",
        )
        .bind::<diesel::sql_types::Text, _>(owner)
        .bind::<diesel::sql_types::BigInt, _>(lease_secs)
        .bind::<diesel::sql_types::BigInt, _>(RETRY_FLOOR_SECS)
        .bind::<diesel::sql_types::BigInt, _>(RETRY_CEILING_SECS)
        .bind::<diesel::sql_types::BigInt, _>(limit)
        .load::<ScriptRow>(&mut conn)
        .map_err(|e| format!("claim scripts: {e}"))?;

        Ok(rows
            .into_iter()
            .map(|r| BotScript {
                branch_id: r.branch_id,
                bot_name: r.bot_name,
                script_path: r.script_path,
                source_kind: r.source_kind,
                source_version: r.source_version,
                compiled_version: r.compiled_version,
                fail_count: r.fail_count,
                last_failed_at: r.last_failed_at,
            })
            .collect())
    }

    /// Record a successful compile: the script leaves the queue and its lease
    /// is released so it is not recompiled until the source version changes.
    pub fn mark_compiled(
        &self,
        branch_id: uuid::Uuid,
        script_path: &str,
        source_version: &str,
    ) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;
        diesel::sql_query(
            "UPDATE bot_scripts \
                SET dirty = FALSE, compiled_version = $3, fail_count = 0, \
                    last_failed_at = NULL, last_error = NULL, \
                    lease_owner = NULL, lease_expires_at = NULL, updated_at = NOW() \
              WHERE branch_id = $1 AND script_path = $2",
        )
        .bind::<diesel::sql_types::Uuid, _>(&branch_id)
        .bind::<diesel::sql_types::Text, _>(script_path)
        .bind::<diesel::sql_types::Text, _>(source_version)
        .execute(&mut conn)
        .map_err(|e| format!("mark compiled {script_path}: {e}"))?;
        Ok(())
    }

    /// Record a failed compile. The script stays `dirty` so it is retried, but
    /// `fail_count`/`last_failed_at` grow the backoff window in `claim`, and the
    /// lease is released so another tick can pick it up.
    pub fn mark_failed(
        &self,
        branch_id: uuid::Uuid,
        script_path: &str,
        error: &str,
    ) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;
        diesel::sql_query(
            "UPDATE bot_scripts \
                SET fail_count = fail_count + 1, last_failed_at = NOW(), last_error = $3, \
                    lease_owner = NULL, lease_expires_at = NULL, updated_at = NOW() \
              WHERE branch_id = $1 AND script_path = $2",
        )
        .bind::<diesel::sql_types::Uuid, _>(&branch_id)
        .bind::<diesel::sql_types::Text, _>(script_path)
        .bind::<diesel::sql_types::Text, _>(truncate_error(error))
        .execute(&mut conn)
        .map_err(|e| format!("mark failed {script_path}: {e}"))?;
        Ok(())
    }

    /// Drop queue entries for a script that no longer exists (e.g. the bot's
    /// repo deleted a tool). Keeps the queue from accumulating dead paths.
    pub fn remove(&self, branch_id: uuid::Uuid, script_path: &str) -> Result<(), String> {
        let mut conn = self.pool.get().map_err(|e| e.to_string())?;
        diesel::sql_query(
            "DELETE FROM bot_scripts WHERE branch_id = $1 AND script_path = $2",
        )
        .bind::<diesel::sql_types::Uuid, _>(&branch_id)
        .bind::<diesel::sql_types::Text, _>(script_path)
        .execute(&mut conn)
        .map_err(|e| format!("remove {script_path}: {e}"))?;
        Ok(())
    }
}

/// Compiler errors can carry a whole multi-line script dump; keep `last_error`
/// bounded so one pathological failure cannot bloat every row it touches.
fn truncate_error(error: &str) -> String {
    const MAX: usize = 1000;
    if error.len() <= MAX {
        return error.to_string();
    }
    let mut end = MAX;
    while end > 0 && !error.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}... [truncated]", &error[..end])
}