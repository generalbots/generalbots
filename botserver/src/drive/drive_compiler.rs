/// DriveCompiler - Compilador unificado para GBDialog
///
/// Fluxo CORRETO:
/// 1. DriveMonitor (S3) lê MinIO diretamente
/// 2. Baixa .bas para /opt/gbo/work/{bot}.gbai/{bot}.gbdialog/
/// 3. Compila .bas → .ast (no mesmo work dir)
/// 4. drive_files table controla etag/status
///
/// SEM usar /opt/gbo/data/ como intermediário!
use crate::basic::compiler::{BasicCompiler, CompilerCallbacks};
use crate::core::shared::state::AppState;
use crate::core::shared::utils::get_work_path;
use crate::drive::drive_monitor::CHECK_INTERVAL_SECS;
use diesel::prelude::*;
use log::{debug, error, info, warn};
use std::collections::HashMap;
use std::error::Error;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::Duration;
use uuid::Uuid;

/// A file that failed to compile is retried after this pause. Long enough to
/// stop a permanent failure from spinning the monitor, short enough that a
/// transient S3/database hiccup still self-heals without a restart.
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

pub struct DriveCompiler {
    state: Arc<AppState>,
    work_root: PathBuf,
    /// #1279/#1288 — paths whose source object is known absent (download
    /// failed with no work copy). Prevents `ast_missing` from forcing a
    /// recompile attempt every scan for files that can never compile until
    /// their object reappears (an ETag change clears the marker).
    missing_files: Arc<RwLock<std::collections::HashSet<String>>>,
    is_processing: Arc<AtomicBool>,
    last_etags: Arc<RwLock<HashMap<String, String>>>,
    /// Paths whose compile attempt failed, with the moment of the last
    /// failure. The ETag of a file is only recorded on success, so without
    /// this a path that can never compile (e.g. a key whose first segment is
    /// not a `.gbai` branch) was retried on EVERY scan — the monitor spun at
    /// 100% CPU and every request (Drive included) queued behind it.
    /// Retry spacing itself moved to `bot_scripts` (#1475); this stays as a
    /// fast local guard so a failing path is not re-attempted in the same tick.
    failed_at: Arc<RwLock<HashMap<String, std::time::Instant>>>,
}

/// Helper function to download file from S3
/// Separated to avoid Send trait issues with tokio::spawn
async fn download_from_s3(file_path: &str, state: &Arc<AppState>) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let app_cfg = state.config.as_ref().ok_or_else(|| "AppState not initialized".to_string())?;
    let s3_repo = crate::drive::s3_repository::S3Repository::new(&app_cfg.drive.endpoint, &app_cfg.drive.access_key, &app_cfg.drive.secret_key, &app_cfg.drive.bucket)
        .map_err(|e| format!("Failed to create S3 operator: {}", e))?;

    // drive_files convention: `{bot}.gbai/{bot}.gbdialog/{tool}.bas` — the
    // first segment names the BOT, not the branch (`cristo-test` shares its
    // branch's workspace inside `cristo.gborg`). Guessing
    // `{first_segment}.gborg` therefore read the wrong bucket for every bot
    // whose name differs from its branch, so the bot's real location is
    // resolved instead; for the branch's own bot the resulting bucket and key
    // are byte-identical to the stored path.
    let parts: Vec<&str> = file_path.split('/').collect();
    if parts.len() < 2 {
        return Err("Invalid file path for S3 download".into());
    }
    let bot_segment = parts[0];
    let bot_name = bot_segment.strip_suffix(".gbai").unwrap_or(bot_segment);
    let relative = parts[1..].join("/");

    let location = match state.conn.get() {
        Ok(mut conn) => Some(botbasic_core::utils::bot_drive_location_for_name(&mut conn, bot_name)),
        Err(e) => {
            // Without the row keep the legacy derivation instead of failing
            // the compile outright.
            log::warn!("DriveCompiler: DB unavailable resolving '{bot_name}': {e}");
            None
        }
    };

    let (bucket_name, s3_key) = match location {
        Some(loc) => (loc.bucket.clone(), format!("{}{relative}", loc.bot_prefix)),
        // Legacy shape: `{bot}.gbai/…` where the path doubles as the key.
        None => (format!("{bot_name}.gborg"), file_path.to_string()),
    };

    s3_repo.get_object_direct(&bucket_name, &s3_key)
        .await
        .map_err(|e| format!("S3 get_object_direct failed for {}/{}: {}", bucket_name, s3_key, e).into())
}

impl DriveCompiler {
    /// #1279 — keys known to be missing (NoSuchBucket/NoSuchKey) with the
    /// moment their single warning was emitted. A static map is deliberate:
    /// the flood it suppresses comes from periodic scans across all monitors,
    /// not from a per-instance flow.
    fn missing_log_registry() -> &'static std::sync::Mutex<std::collections::HashMap<String, std::time::Instant>> {
        static REGISTRY: std::sync::OnceLock<std::sync::Mutex<std::collections::HashMap<String, std::time::Instant>>> =
            std::sync::OnceLock::new();
        REGISTRY.get_or_init(|| std::sync::Mutex::new(std::collections::HashMap::new()))
    }

    /// Returns true when the warning for this key is suppressed (logged
    /// within the last hour).
    fn suppress_missing_log(fp: &str) -> bool {
        let mut map = Self::missing_log_registry().lock().unwrap_or_else(|p| p.into_inner());
        let now = std::time::Instant::now();
        match map.get(fp) {
            Some(t) if now.duration_since(*t) < std::time::Duration::from_secs(3600) => true,
            _ => {
                map.insert(fp.to_string(), now);
                false
            }
        }
    }

    /// Clears the suppression when the object reappears.
    fn clear_missing_suppression(fp: &str) {
        Self::missing_log_registry()
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(fp);
    }

    /// Marks a path as absent so the scanner stops retrying it every cycle.
    async fn mark_missing(&self, fp: &str) {
        self.missing_files.write().await.insert(fp.to_string());
    }

    /// Clears the absent marker (object came back or compiled fine).
    async fn clear_missing(&self, fp: &str) {
        self.missing_files.write().await.remove(fp);
    }

    pub fn new(state: Arc<AppState>) -> Self {
        let work_root = PathBuf::from(get_work_path());

        Self {
            state,
            work_root,
            missing_files: Arc::new(RwLock::new(std::collections::HashSet::new())),
            failed_at: Arc::new(RwLock::new(HashMap::new())),
            is_processing: Arc::new(AtomicBool::new(false)),
            last_etags: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Iniciar loop de compilação baseado em drive_files
    pub async fn start_compiling(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        info!("DriveCompiler started - compiling .bas files directly to work dir");

        self.is_processing.store(true, Ordering::SeqCst);

        let compiler = self.clone();

        tokio::spawn(async move {
            let mut consecutive_db_errors: u32 = 0;

            while compiler.is_processing.load(Ordering::SeqCst) {
                tokio::time::sleep(Duration::from_secs(CHECK_INTERVAL_SECS)).await;

                match compiler.check_and_compile().await {
                    Ok(_) => {
                        consecutive_db_errors = 0;
                    }
                    Err(e) => {
                        let err_msg = e.to_string();
                        if err_msg.contains("timed out") || err_msg.contains("connection refused") {
                            consecutive_db_errors = consecutive_db_errors.saturating_add(1);
                            let backoff = CHECK_INTERVAL_SECS * (1u64 << consecutive_db_errors.min(4));
                            let backoff = backoff.min(300);
                            warn!(
                                "DriveCompiler: DB unavailable ({} consecutive), backing off {}s: {}",
                                consecutive_db_errors, backoff, err_msg
                            );
                            tokio::time::sleep(Duration::from_secs(backoff)).await;
                        } else {
                            error!("DriveCompiler error: {}", err_msg);
                        }
                    }
                }
            }
        });

        Ok(())
    }

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
    async fn check_and_compile(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
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

    /// Compilar arquivo .bas → .ast DIRETAMENTE em work/{bot}.gbai/{bot}.gbdialog/
    async fn compile_file(&self, _bot_id: Uuid, fp: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        // fp formats:
        // - {bot}.gbai/{bot}.gbdialog/{tool}.bas (full path with bucket prefix)
        // - {bot}.gbdialog/{tool}.bas (without bucket prefix)
        // - {bot}.gbkb/{doc}.txt (KB files - skip compilation)
        let parts: Vec<&str> = fp.split('/').collect();
        if parts.len() < 2 {
            return Err("Invalid file path format".into());
        }

    // Determine branch name, bot name, and work directory structure.
    // Structure: {org}.gborg/{branch}.gbai/{bot}.gbdialog/{tool}.ast
    // .gborg = organization/tenant, .gbai = branch, .gbdialog = bot
    // Branch and bot are separate: multiple bots can exist under one branch.
    let (branch_name, bot_name, work_dir) = if parts[0].ends_with(".gbai") {
        // Full path: {branch}.gbai/{bot}.gbdialog/{tool}.bas
        let branch_name = parts[0].strip_suffix(".gbai").unwrap_or(parts[0]);
        let bot_name = if parts.len() >= 2 {
            parts[1].strip_suffix(".gbdialog").unwrap_or(parts[1])
        } else {
            branch_name
        };
        let work_dir = self.work_root.join(format!("{branch_name}.gborg/{branch_name}.gbai/{bot_name}.gbdialog"));
        (branch_name.to_string(), bot_name.to_string(), work_dir)
    } else if parts.len() >= 2 && parts[0].ends_with(".gbdialog") {
        // Short path (legacy): {bot}.gbdialog/{tool}.bas
        let bot_name = parts[0].strip_suffix(".gbdialog").unwrap_or(parts[0]);
        let work_dir = self.work_root.join(format!("{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbdialog"));
        (bot_name.to_string(), bot_name.to_string(), work_dir)
    } else if parts.len() >= 2 && parts[0].ends_with(".gbkb") {
        // KB file: {bot}.gbkb/{doc}.txt - skip compilation
        debug!("Skipping KB file: {}", fp);
        return Ok(());
    } else {
        warn!("Unknown file path format: {}", fp);
        return Err("Invalid file path format".into());
    };

    // Look up the real bot_id from the database using the bot name
    let real_bot_id = Self::resolve_bot_id(&self.state, &bot_name);

    // Create work directory
    std::fs::create_dir_all(&work_dir)?;

    // Determine tool name from last part of path
    let tool_name = parts.last().unwrap_or(&"unknown").strip_suffix(".bas").unwrap_or(parts.last().unwrap_or(&"unknown"));

        // Caminho do .bas no work
        let work_bas_path = work_dir.join(format!("{}.bas", tool_name));

        // Reform #1501/#1502 — a git-owned bot's `.gbdialog` lives in its
        // repository: the git monitor materializes the committed sources into
        // this work dir and queues the file for compile. Downloading the Drive
        // object here overwrote that materialization with a stale copy — the
        // split brain that froze a media-filing bot for hours — so the
        // materialized work copy is compiled as-is.
        let git_owned = {
            let mut conn = self.state.conn.get()?;
            git_owned_bots(&mut conn).contains(&(branch_name.clone(), bot_name.clone()))
        };
        let use_work_copy = git_owned && work_bas_path.exists();

        info!("Downloading {} from S3 to work dir", fp);
        let download_result = if use_work_copy {
            info!(
                "git-owned bot '{bot_name}': compiling the source materialized from git, Drive copy bypassed"
            );
            Err(format!("git-owned bot '{bot_name}': Drive copy bypassed").into())
        } else {
            download_from_s3(fp, &self.state).await
        };
        
        match download_result {
            Ok(content) => {
                if let Err(e) = std::fs::write(&work_bas_path, content) {
                    warn!("Failed to write {} to work dir: {}", work_bas_path.display(), e);
                    return Err(format!("Failed to write file: {}", e).into());
                }
                info!("Downloaded {} to {}", fp, work_bas_path.display());
                // #1279 — the object is back; clear the suppression so a
                // future failure logs normally again.
                Self::clear_missing_suppression(fp);
            }
            Err(e) => {
                // #1279 — a missing bucket/object repeats on every drive scan
                // (NoSuchBucket): logging the full S3 error body each time
                // flooded prod logs with thousands of duplicate blocks. The
                // error text is compacted and, once a key is known-missing,
                // demoted to debug so it logs at most once per hour (the
                // entry is dropped when the object reappears via Ok).
                let error_text = e.to_string();
                let missing = error_text.contains("NoSuchBucket") || error_text.contains("NoSuchKey");
                if missing && Self::suppress_missing_log(fp) {
                    debug!("S3 object still missing (suppressed): {}", fp);
                } else if missing {
                    warn!("S3 object missing: {} (compacted; details suppressed until it reappears)", fp);
                } else if use_work_copy {
                    debug!("Git-owned source compiled from the work copy: {}", fp);
                } else {
                    info!("No Drive copy of {} ({}); using the work copy", fp, e);
                }
                if !work_bas_path.exists() {
                    // #1264 — a permanently-absent object with no work copy used
                    // to return Err every scan, so the monitor retried the same
                    // file 2×/second and burned a full core on a hopeless loop
                    // (signup capacity gate saw the resulting CPU and vetoed
                    // every free signup). Suppress like NoSuchBucket: log once,
                    // then debug until the object reappears, and return Ok so
                    // the scanner moves on without spinning.
                if missing || Self::suppress_missing_log(fp) {
                    debug!("S3 object absent, compile skipped (suppressed): {}", fp);
                } else {
                    warn!("S3 object absent, compile skipped: {} (suppressed until it reappears)", fp);
                }
                self.mark_missing(fp).await;
                return Ok(());
                }
                info!("Failed to download {} from S3, using existing work copy", fp);
            }
        }

        // Verify file exists now
        if !work_bas_path.exists() {
            info!("File {} still not found after download attempt", work_bas_path.display());
            return Ok(());
        }

        // Ler conteúdo
        let _content = std::fs::read_to_string(&work_bas_path)?;

        // Compilar com BasicCompiler (já está no work dir, então compila in-place)
        let mut callbacks = CompilerCallbacks::new();
        #[cfg(feature = "tasks")]
        {
            let schedule_fn = crate::basic::keywords::set_schedule::execute_set_schedule;
            callbacks.execute_set_schedule = Some(Box::new(move |conn, cron, script, bot_id| {
                schedule_fn(conn, cron, script, bot_id)
                    .map(|_| ())
                    .map_err(|e| e.to_string())
            }));
        }
        // ON UPDATE OF callback is always available (not behind tasks feature)
        callbacks.execute_on_update = Some(Box::new(|conn, table_name, script_name, bot_id, kind| {
            crate::basic::keywords::on_update::execute_on_update_registration(conn, table_name, script_name, bot_id, kind)
                .map(|_| ())
                .map_err(|e| e.to_string())
        }));
        // `ON EVENT "<event>"` in a tool: the tool subscribes itself to a
        // channel event at design time. Registered here (not at run time)
        // because the tool only ever runs *because* of the event.
        callbacks.execute_on_event = Some(Box::new(|conn, event, script, bot_id| {
            botcore::shared::basic_events::register_handler_on(
                conn,
                bot_id,
                event,
                script,
            )
            .map_err(|e| e.to_string())
        }));
        callbacks.execute_webhook = Some(Box::new(|conn, endpoint, script, bot_id| {
            crate::basic::keywords::webhook::execute_webhook_registration(conn, endpoint, script, bot_id)
                .map(|_| ())
                .map_err(|e| e.to_string())
        }));
        callbacks.execute_use_website = Some(Box::new(|conn, url, bot_id, refresh| {
            crate::basic::keywords::use_website::execute_use_website_preprocessing_with_refresh(conn, url, bot_id, refresh)
                .map(|_| ())
                .map_err(|e| e.to_string())
        }));
        callbacks.process_table_definitions = Some(Box::new(|runtime, bot_id, content| {
            crate::basic::keywords::table_definition::process_table_definitions(runtime, bot_id, content)
                .map(|_| ())
                .map_err(|e| e.to_string())
        }));
        callbacks.create_runtime = Some(Box::new(|state| {
            Arc::new(crate::basic::AppStateBasicRuntime(state))
        }));
        let mut compiler = BasicCompiler::with_callbacks(self.state.clone(), real_bot_id, callbacks);
        compiler.compile_file(
            work_bas_path.to_str().ok_or("Invalid path")?,
            work_dir.to_str().ok_or("Invalid path")?
        )?;

        // A tool's MCP manifest in Drive is the source of truth for its
        // argument schema. compile_file() always regenerates a manifest from
        // the .bas (with an empty schema when the script declares no
        // parameters), which clobbers the richer manifest persisted by
        // AutoTask shipped templates; tool_exec relies on
        // input_schema.properties to default arguments the LLM omitted
        // (otherwise optional params reach the script as undeclared
        // variables and abort it). Sync the Drive copy over the generated
        // one when it exists; missing manifests keep the generated fallback.
        let manifest_fp = match fp.strip_suffix(".bas") {
            Some(base) => format!("{base}.mcp.json"),
            None => fp.to_string(),
        };
        let work_manifest_path = work_dir.join(format!("{}.mcp.json", tool_name));
        // The repository's manifest is materialized next to the source, so a
        // git-owned bot keeps it (same reason as the source above).
        let manifest_sync = if use_work_copy && work_manifest_path.exists() {
            debug!("git-owned bot '{bot_name}': keeping the repository manifest for {manifest_fp}");
            Err(format!("git-owned bot '{bot_name}': Drive manifest bypassed").into())
        } else {
            download_from_s3(&manifest_fp, &self.state).await
        };
        match manifest_sync {
            Ok(bytes) => match String::from_utf8(bytes) {
                Ok(text) => {
                    if let Err(e) = std::fs::write(&work_manifest_path, text) {
                        warn!("Failed to write MCP manifest to work dir: {e}");
                    } else {
                        info!("Synced MCP manifest from Drive: {manifest_fp}");
                    }
                }
                Err(e) => warn!("MCP manifest from Drive is not UTF-8, keeping generated one: {e}"),
            },
            Err(_) => debug!("No MCP manifest in Drive for {manifest_fp}; keeping generated one"),
        }

        let work_ast_path = work_dir.join(format!("{}.ast", tool_name));
        let ast_path_str = work_ast_path.to_str().unwrap_or("").to_string();

        let branch_id = Self::resolve_branch_id(&self.state, &branch_name);

        let bot_id_str = real_bot_id.to_string();
        let branch_id_str = branch_id.to_string();
        let upsert_sql = diesel::sql_query(
            "INSERT INTO basic_tools (bot_id, tool_name, file_path, ast_path, compiled_at, is_active, branch_id) \
             VALUES ($1::uuid, $2, $3, $4, $5, true, $6::uuid) \
             ON CONFLICT (bot_id, tool_name) DO UPDATE SET \
             file_path = EXCLUDED.file_path, ast_path = EXCLUDED.ast_path, \
             compiled_at = EXCLUDED.compiled_at, is_active = true"
        )
        .bind::<diesel::sql_types::Text, _>(&bot_id_str)
        .bind::<diesel::sql_types::Text, _>(tool_name)
        .bind::<diesel::sql_types::Text, _>(fp)
        .bind::<diesel::sql_types::Text, _>(&ast_path_str)
        .bind::<diesel::sql_types::Timestamptz, _>(chrono::Utc::now())
        .bind::<diesel::sql_types::Text, _>(&branch_id_str);
        match upsert_sql.execute(&mut *self.state.conn.get()?)
        {
            Ok(_) => info!("Registered tool '{}' in database", tool_name),
            Err(e) => warn!("Failed to register tool '{}' in database: {}", tool_name, e),
        }

        info!("Compiled {} to {}.ast", fp, tool_name);
        Ok(())
    }

    /// Resolve the expected .ast path for a given file path, to check if it exists.
    /// Returns PathBuf without verifying existence — caller checks .exists().
    fn resolve_ast_path(&self, fp: &str) -> PathBuf {
        let parts: Vec<&str> = fp.split('/').collect();
        if parts.len() < 2 || parts.iter().any(|p| p.ends_with(".gbkb")) {
            return PathBuf::new();
        }

        let (branch_name, bot_name) = if parts[0].ends_with(".gbai") {
            let branch = parts[0].strip_suffix(".gbai").unwrap_or(parts[0]);
            let bot = if parts.len() >= 2 {
                parts[1].strip_suffix(".gbdialog").unwrap_or(parts[1])
            } else {
                branch
            };
            (branch.to_string(), bot.to_string())
        } else if parts.len() >= 2 && parts[0].ends_with(".gbdialog") {
            let bot = parts[0].strip_suffix(".gbdialog").unwrap_or(parts[0]);
            (bot.to_string(), bot.to_string())
        } else {
            return PathBuf::new();
        };

        let tool_name = parts.last()
            .unwrap_or(&"unknown")
            .strip_suffix(".bas")
            .unwrap_or(parts.last().unwrap_or(&"unknown"));
        let work_dir = self.work_root.join(format!("{branch_name}.gborg/{branch_name}.gbai/{bot_name}.gbdialog"));
        work_dir.join(format!("{}.ast", tool_name))
    }

    /// Resolve the branch UUID from the branch slug/name using the database.
    /// Falls back to Uuid::nil() if the branch is not found.
    fn resolve_branch_id(state: &Arc<AppState>, branch_name: &str) -> Uuid {
        use botcore::shared::models::schema::branches::dsl::*;

        let mut conn = match state.conn.get() {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to get DB connection for branch name lookup: {}", e);
                return Uuid::nil();
            }
        };

        match branches
            .filter(slug.eq(branch_name))
            .select(id)
            .first::<Uuid>(&mut *conn)
        {
            Ok(branch_id) => branch_id,
            Err(e) => {
                warn!("Branch '{}' not found in database ({}), using nil UUID", branch_name, e);
                Uuid::nil()
            }
        }
    }

    /// Resolve the real bot_id from the bot name using the database.
    /// Falls back to Uuid::nil() if the bot is not found (backward compatibility).
    fn resolve_bot_id(state: &Arc<AppState>, bot_name: &str) -> Uuid {
        use botcore::shared::models::schema::bots::dsl::*;

        let mut conn = match state.conn.get() {
            Ok(c) => c,
            Err(e) => {
                warn!("Failed to get DB connection for bot name lookup: {}", e);
                return Uuid::nil();
            }
        };

        match bots
            .filter(name.eq(bot_name))
            .select(id)
            .first::<Uuid>(&mut *conn)
        {
            Ok(bot_id) => bot_id,
            Err(e) => {
                warn!("Bot '{}' not found in database ({}), using nil UUID", bot_name, e);
                Uuid::nil()
            }
        }
    }
}

impl Clone for DriveCompiler {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            work_root: self.work_root.clone(),
            missing_files: Arc::clone(&self.missing_files),
            failed_at: Arc::clone(&self.failed_at),
            is_processing: Arc::clone(&self.is_processing),
            last_etags: Arc::clone(&self.last_etags),
        }
    }
}

/// Reform #1501 — (branch_slug, bot_name) pairs whose bot sources moved to
/// git: a vibe project of the branch carries `payload.source_imported_at`.
/// The set is re-read each compile scan (cheap single query) so newly
/// imported bots stop compiling from Drive on the very next tick.
fn git_owned_bots(
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
