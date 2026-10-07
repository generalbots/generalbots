//! Compile-queue draining for `DriveCompiler` (#1475).
//!
//! Split out of `drive_compiler.rs` to keep that file inside the 450-line
//! limit. Everything here is about *what to compile*: the `bot_scripts` dirty
//! set, the lease that stops two ticks taking the same script, the retry
//! backoff, and the probe for bots whose sources moved to git. Turning a script
//! into a `.ast` stays in the parent module (`compile_file`).

use super::*;

/// How many queued scripts one tick claims. Bounded so a large backlog is
/// drained over several ticks instead of monopolising the loop.
const COMPILE_BATCH_SIZE: i64 = 64;

/// How long a claimed script stays leased. The lease only guards against
/// overlapping ticks; the retry backoff for a *failing* script lives in
/// `bot_scripts` (`fail_count`/`last_failed_at`), not here.
const COMPILE_LEASE: std::time::Duration = std::time::Duration::from_secs(300);

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct MissingArtifactRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: uuid::Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    script_path: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    bot_name: String,
}

impl DriveCompiler {
    /// Drain the `bot_scripts` compile queue (#1475).
    ///
    /// This used to read the `drive_files` *inventory* with a leading-wildcard
    /// `LIKE '%.gbdialog/%'`, which can never use a btree index. Since 6.5.25
    /// dropped `idx_drive_files_type`, every 5 s tick sequentially scanned the
    /// whole table and discarded the ~9.5k KB/media/shared rows that were never
    /// candidates. It also recompiled git-owned tools forever: the git monitor
    /// and the DriveMonitor wrote different meanings into the same `etag`
    /// column (a commit sha vs an S3 ETag) and the version was only recorded on
    /// success, so a broken script was retried on every tick.
    ///
    /// `bot_scripts` fixes all three: a partial index on `dirty` makes this an
    /// index-only scan of pending work, `source_version` is separated from
    /// `compiled_version`, and `claim` leases the batch so overlapping ticks
    /// cannot take the same script.
    pub(crate) async fn check_and_compile(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let scripts = botdrive::BotScriptsRepository::new(self.state.conn.clone());
        let owner = self.lease_owner();
        let claimed = scripts
            .claim(&owner, COMPILE_BATCH_SIZE, COMPILE_LEASE.as_secs() as i64)
            .map_err(|e| -> Box<dyn Error + Send + Sync> { e.into() })?;
        if claimed.is_empty() {
            return Ok(());
        }
        debug!(
            "DriveCompiler: claimed {} script(s) from the bot_scripts queue",
            claimed.len()
        );

        // Reform #1501 — git-owned detection is only used to WARN when a
        // git-owned bot's source changes in Drive; compilation still proceeds
        // so Drive remains the operational fallback when a bot's repo lags.
        let git_owned = {
            let mut conn = self.state.conn.get()?;
            git_owned_bots(&mut conn)
        };

        // `tables.bas` defines the schema every other script compiles against,
        // so compile it first within a batch.
        let mut claimed = claimed;
        claimed.sort_by(|a, b| {
            b.script_path.contains("tables.bas")
                .cmp(&a.script_path.contains("tables.bas"))
        });

        for script in claimed {
            let query_file_path = script.script_path.clone();
            let source_version = script.source_version.clone();

            // Reform #1501 — the leading drive_files path segment is the
            // object-key form "{branch}.gbai", while git_owned_bots returns the
            // bare branch slug, so normalize before comparing.
            let branch_segment = query_file_path.split('/').next().unwrap_or("");
            let branch_slug = branch_segment.strip_suffix(".gbai").unwrap_or(branch_segment);
            let bot_segment = query_file_path
                .split('/')
                .nth(1)
                .unwrap_or("")
                .strip_suffix(".gbdialog")
                .unwrap_or("");
            if !bot_segment.is_empty()
                && git_owned.contains(&(branch_slug.to_string(), bot_segment.to_string()))
                && script.source_kind == "drive"
            {
                debug!(
                    "DriveCompiler: {} changed in Drive for git-owned bot '{}' — the repository is the source of truth (push through git to make the change durable)",
                    query_file_path, bot_segment
                );
            }

            debug!(
                "DriveCompiler: compiling {} ({} @ {}, {} prior failure(s))",
                query_file_path, script.source_kind, source_version, script.fail_count
            );

            match self.compile_file(Uuid::nil(), &query_file_path).await {
                Err(e) => {
                    let msg = e.to_string();
                    if let Err(db_err) =
                        scripts.mark_failed(script.branch_id, &query_file_path, &msg)
                    {
                        error!("DriveCompiler: mark_failed {}: {db_err}", query_file_path);
                    }
                    self.failed_at
                        .write()
                        .await
                        .insert(query_file_path.clone(), std::time::Instant::now());
                    if msg.contains("Invalid file path") {
                        warn!(
                            "DriveCompiler: {} cannot be compiled ({msg}) — backing off before retry",
                            query_file_path
                        );
                    } else {
                        error!("Failed to compile {}: {msg}", query_file_path);
                    }
                }
                Ok(()) => {
                    self.failed_at.write().await.remove(&query_file_path);
                    // #1288 — only claim success when an .ast was actually
                    // produced; `compile_file` returns Ok on the skip path too.
                    // A skip means the source object is not reachable (stale
                    // bucket, deleted tool), which is not a compile failure:
                    // dropping the entry lets the DriveMonitor re-add it if the
                    // object ever returns, instead of retrying forever behind
                    // the backoff.
                    if self.resolve_ast_path(&query_file_path).exists() {
                        if let Err(db_err) = scripts.mark_compiled(
                            script.branch_id,
                            &query_file_path,
                            &source_version,
                        ) {
                            error!(
                                "DriveCompiler: mark_compiled {}: {db_err}",
                                query_file_path
                            );
                        }
                        let mut etags = self.last_etags.write().await;
                        etags.insert(query_file_path.clone(), source_version);
                        self.clear_missing(&query_file_path).await;
                        info!("DriveCompiler: {} compiled successfully", query_file_path);
                    } else if let Err(db_err) = scripts.remove(script.branch_id, &query_file_path)
                    {
                        error!(
                            "DriveCompiler: drop unresolvable {}: {db_err}",
                            query_file_path
                        );
                    } else {
                        debug!(
                            "DriveCompiler: {} produced no .ast (source unreachable) — dropped from the queue",
                            query_file_path
                        );
                    }
                }
            }
        }

        // Self-heal: a script whose .ast vanished (work dir wiped, bot
        // re-provisioned) is no longer dirty, so nothing would requeue it.
        // Rather than scan every file, requeue only what the compiler already
        // knows about and whose artifact is gone.
        self.requeue_missing_artifacts(&scripts).await;

        Ok(())
    }

    /// Requeue known scripts whose compiled artifact disappeared.
    async fn requeue_missing_artifacts(&self, scripts: &botdrive::BotScriptsRepository) {
        let paths: Vec<(uuid::Uuid, String, String)> = {
            let Ok(mut conn) = self.state.conn.get() else {
                return;
            };
            let rows = diesel::sql_query(
                "SELECT branch_id, script_path, bot_name FROM bot_scripts \
                 WHERE NOT dirty AND script_path LIKE '%.bas' LIMIT 500",
            )
            .load::<MissingArtifactRow>(&mut conn);
            match rows {
                Ok(r) => r
                    .into_iter()
                    .map(|r| (r.branch_id, r.script_path, r.bot_name))
                    .collect(),
                Err(e) => {
                    debug!("DriveCompiler: artifact requeue scan failed: {e}");
                    return;
                }
            }
        };
        for (branch_id, path, bot_name) in paths {
            if !botdrive::BotScriptsRepository::is_compilable_path(&path) {
                continue;
            }
            if self.resolve_ast_path(&path).exists() {
                continue;
            }
            match scripts.enqueue(
                branch_id,
                &bot_name,
                &path,
                botdrive::SourceKind::Drive,
                "artifact-missing",
            ) {
                Ok(true) => warn!(
                    "DriveCompiler: {} compiled artifact missing — requeued",
                    path
                ),
                Ok(false) => {}
                Err(e) => debug!("DriveCompiler: requeue {}: {e}", path),
            }
        }
    }

    /// Stable per-process identifier used as the queue lease owner, so a
    /// restarted process does not appear to still hold outstanding leases.
    fn lease_owner(&self) -> String {
        static OWNER: std::sync::OnceLock<String> = std::sync::OnceLock::new();
        OWNER
            .get_or_init(|| format!("drive-compiler-{}", Uuid::new_v4()))
            .clone()
    }

}
/// Reform #1501 — (branch_slug, bot_name) pairs whose bot sources moved to
/// git: a vibe project of the branch carries `payload.source_imported_at`.
/// The set is re-read each compile scan (cheap single query) so newly
/// imported bots stop compiling from Drive on the very next tick.
pub(crate) fn git_owned_bots(
    conn: &mut diesel::PgConnection,
) -> std::collections::HashSet<(String, String)> {
    #[derive(diesel::QueryableByName)]
    struct Row {
        #[diesel(sql_type = diesel::sql_types::Text)]
        branch_slug: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
    }
    // Reform #1501/#1502 — "git-owned" means the git-pull monitor feeds this
    // bot (vibe project in git mode), which is the same condition
    // `git_bot_monitor::loop_ops::list_monitored_bots` uses. Keying on
    // `payload->>'source_imported_at'` missed projects provisioned later, so
    // the two paths disagreed about who owns a bot's sources.
    diesel::sql_query(
        "SELECT br.slug AS branch_slug, vp.name \n         FROM vibe_projects vp \n         JOIN branches br ON br.id = vp.branch_id \n         WHERE vp.project_type = 'bot' AND vp.source_control = 'git'",
    )
    .load::<Row>(conn)
    .unwrap_or_default()
    .into_iter()
    .map(|r| (r.branch_slug, r.name))
    .collect()
}
