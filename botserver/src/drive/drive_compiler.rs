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
use crate::core::shared::utils::{current_org_id, get_work_path};
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
    pub fn new(state: Arc<AppState>) -> Self {
        let work_root = PathBuf::from(get_work_path());

        Self {
            state,
            work_root,
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

            // Verificar se precisa compilar
            let should_compile = {
                let etags = self.last_etags.read().await;
                etags.get(&query_file_path).map(|e| e != &current_etag).unwrap_or(true)
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

                    info!("DriveCompiler: {} compiled successfully", query_file_path);
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

    // Determine bot name and work directory structure
    let (bot_name, work_dir) = if parts[0].ends_with(".gbai") {
        // Full path: {bot}.gbai/{bot}.gbdialog/{tool}.bas
        let bot_name = parts[0].strip_suffix(".gbai").unwrap_or(parts[0]);
        let work_dir = self.work_root.join(format!("{org_id}.gborg/{bot_name}.gbai/{bot_name}.gbdialog", org_id = current_org_id()));
        (bot_name.to_string(), work_dir)
    } else if parts.len() >= 2 && parts[0].ends_with(".gbdialog") {
        // Short path: {bot}.gbdialog/{tool}.bas
        let bot_name = parts[0].strip_suffix(".gbdialog").unwrap_or(parts[0]);
        let work_dir = self.work_root.join(format!("{org_id}.gborg/{bot_name}.gbai/{bot_name}.gbdialog", org_id = current_org_id()));
        (bot_name.to_string(), work_dir)
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

        // Check if file exists in work dir
        if !work_bas_path.exists() {
            // File doesn't exist in work dir - need to download from S3
            // This should be done by DriveMonitor, but we can try to fetch it here
            info!("File {} not found in work dir, attempting to download from S3", work_bas_path.display());
            
            // Download in separate task to avoid Send issues
            let download_result = download_from_s3(fp, &self.state).await;
            
            match download_result {
                Ok(content) => {
                    if let Err(e) = std::fs::write(&work_bas_path, content) {
                        warn!("Failed to write {} to work dir: {}", work_bas_path.display(), e);
                        return Err(format!("Failed to write file: {}", e).into());
                    }
                    info!("Downloaded {} to {}", fp, work_bas_path.display());
                }
                Err(e) => {
                    info!("Failed to download {} from S3: {}", fp, e);
                    return Err(format!("File not found in S3: {}", fp).into());
                }
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

        let branch_name = if parts[0].ends_with(".gbai") {
            parts[0].strip_suffix(".gbai").unwrap_or(parts[0]).to_string()
        } else if parts.len() >= 2 && parts[0].ends_with(".gbdialog") {
            parts[0].strip_suffix(".gbdialog").unwrap_or(parts[0]).to_string()
        } else {
            bot_name.clone()
        };
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
            is_processing: Arc::clone(&self.is_processing),
            last_etags: Arc::clone(&self.last_etags),
        }
    }
}
