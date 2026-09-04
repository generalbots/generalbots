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
use crate::drive::drive_files::drive_files as drive_files_table;
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
}

/// Helper function to download file from S3
/// Separated to avoid Send trait issues with tokio::spawn
async fn download_from_s3(file_path: &str, state: &Arc<AppState>) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let app_cfg = state.config.as_ref().ok_or_else(|| "AppState not initialized".to_string())?;
    let s3_repo = crate::drive::s3_repository::S3Repository::new(&app_cfg.drive.endpoint, &app_cfg.drive.access_key, &app_cfg.drive.secret_key, &app_cfg.drive.bucket)
        .map_err(|e| format!("Failed to create S3 operator: {}", e))?;

    // file_path format: {branch}.gbai/{bot}.gbdialog/{tool}.bas
    // In .gborg structure: bucket = {branch}.gborg, key = {branch}.gbai/{bot}.gbdialog/{tool}.bas
    let parts: Vec<&str> = file_path.split('/').collect();
    if parts.len() < 2 {
        return Err("Invalid file path for S3 download".into());
    }

    let branch_prefix = parts[0];
    let branch_name = branch_prefix.strip_suffix(".gbai").unwrap_or(branch_prefix);
    let bucket_name = format!("{}.gborg", branch_name);
    let s3_key = file_path;

    s3_repo.get_object_direct(&bucket_name, s3_key)
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

    /// Verifica drive_files e compila arquivos .bas que mudaram
    async fn check_and_compile(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        use drive_files_table::dsl::*;

        let mut conn = self.state.conn.get()?;

        let mut files: Vec<(String, String, Option<String>)> = drive_files_table::table
            .filter(file_type.eq("bas"))
            .filter(file_path.like("%.gbdialog/%"))
            .select((file_path, file_type, etag))
            .load(&mut conn)?;
        files.sort_by(|a, b| {
            let a_is_tables = a.0.contains("tables.bas");
            let b_is_tables = b.0.contains("tables.bas");
            b_is_tables.cmp(&a_is_tables)
        });

        for (query_file_path, _file_type, current_etag_opt) in files {
            let current_etag = current_etag_opt.unwrap_or_default();

            // Verificar se precisa compilar (ETag mudou ou .ast foi deletado do work dir)
            let should_compile = {
                let etags = self.last_etags.read().await;
                let etag_changed = etags.get(&query_file_path).map(|e| e != &current_etag).unwrap_or(true);
                let is_marked_missing = self.missing_files.read().await.contains(&query_file_path);
                let ast_missing = !is_marked_missing && !self.resolve_ast_path(&query_file_path).exists();
                if ast_missing {
                    debug!("Force recompile: .ast file missing for {}", query_file_path);
                }
                etag_changed || ast_missing
            };

            if should_compile {
                debug!("DriveCompiler: {} changed, compiling...", query_file_path);

                // Compilar diretamente para work dir
                if let Err(e) = self.compile_file(Uuid::nil(), &query_file_path).await {
                    error!("Failed to compile {}: {}", query_file_path, e);
                } else {
                    // Atualizar estado
                    let mut etags = self.last_etags.write().await;
                    etags.insert(query_file_path.clone(), current_etag);

                    // #1288 — the skip path also returns Ok; only claim success
                    // (and clear the missing marker) when an .ast was actually
                    // produced.
                    if self.resolve_ast_path(&query_file_path).exists() {
                        self.clear_missing(&query_file_path).await;
                        info!("DriveCompiler: {} compiled successfully", query_file_path);
                    }
                }
            }
        }

        Ok(())
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

        // Always download latest from S3 to ensure work dir is in sync
        info!("Downloading {} from S3 to work dir", fp);
        let download_result = download_from_s3(fp, &self.state).await;
        
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
                } else {
                    info!("Failed to download {} from S3 and no local copy: {}", fp, e);
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
            is_processing: Arc::clone(&self.is_processing),
            last_etags: Arc::clone(&self.last_etags),
        }
    }
}
