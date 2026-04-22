/// DriveCompiler - Compilador unificado para GBDialog
///
/// Fluxo CORRETO:
/// 1. DriveMonitor (S3) lê MinIO diretamente
/// 2. Baixa .bas para /opt/gbo/work/{bot}.gbai/{bot}.gbdialog/
/// 3. Compila .bas → .ast (no mesmo work dir)
/// 4. drive_files table controla etag/status
///
/// SEM usar /opt/gbo/data/ como intermediário!

use crate::basic::compiler::BasicCompiler;
use crate::core::config::DriveConfig;
use crate::core::shared::state::AppState;
use crate::core::shared::utils::get_work_path;
use crate::drive::drive_files::drive_files as drive_files_table;
use crate::drive::drive_monitor::monitor::CHECK_INTERVAL_SECS;
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
async fn download_from_s3(file_path: &str) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
    let config = DriveConfig::default();
    let s3_repo = crate::core::shared::utils::create_s3_operator(&config)
        .await
        .map_err(|e| format!("Failed to create S3 operator: {}", e))?;

    // file_path format: {bot}.gbai/{bot}.gbdialog/{tool}.bas
    // S3 bucket = first part ({bot}.gbai), key = rest
    let parts: Vec<&str> = file_path.split('/').collect();
    if parts.len() < 2 {
        return Err("Invalid file path for S3 download".into());
    }

    let bucket_name = parts[0];
    let s3_key = parts[1..].join("/");

    s3_repo.get_object_direct(bucket_name, &s3_key)
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

        // Loop que verifica drive_files a cada 1s
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(CHECK_INTERVAL_SECS));

            while compiler.is_processing.load(Ordering::SeqCst) {
                interval.tick().await;

                if let Err(e) = compiler.check_and_compile().await {
                    error!("DriveCompiler error: {}", e);
                }
            }
        });

        Ok(())
    }

    /// Verifica drive_files e compila arquivos .bas que mudaram
    async fn check_and_compile(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        use drive_files_table::dsl::*;

        let mut conn = self.state.conn.get()?;

        // Selecionar todos os arquivos .gbdialog/*.bas
        let files: Vec<(Uuid, String, String, Option<String>)> = drive_files_table::table
            .filter(file_type.eq("bas"))
            .filter(file_path.like("%.gbdialog/%"))
            .select((bot_id, file_path, file_type, etag))
            .load(&mut conn)?;

        for (query_bot_id, query_file_path, _file_type, current_etag_opt) in files {
            let current_etag = current_etag_opt.unwrap_or_default();

            // Verificar se precisa compilar
            let should_compile = {
                let etags = self.last_etags.read().await;
                etags.get(&query_file_path).map(|e| e != &current_etag).unwrap_or(true)
            };

            if should_compile {
                debug!("DriveCompiler: {} changed, compiling...", query_file_path);

                // Compilar diretamente para work dir
                if let Err(e) = self.compile_file(query_bot_id, &query_file_path).await {
                    error!("Failed to compile {}: {}", query_file_path, e);
                } else {
                    // Atualizar estado
                    let mut etags = self.last_etags.write().await;
                    etags.insert(query_file_path.clone(), current_etag.clone());

                    info!("DriveCompiler: {} compiled successfully", query_file_path);
                }
            }
        }

        Ok(())
    }

    /// Compilar arquivo .bas → .ast DIRETAMENTE em work/{bot}.gbai/{bot}.gbdialog/
    async fn compile_file(&self, bot_id: Uuid, file_path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        // file_path formats:
        // - {bot}.gbai/{bot}.gbdialog/{tool}.bas (full path with bucket prefix)
        // - {bot}.gbdialog/{tool}.bas (without bucket prefix)
        // - {bot}.gbkb/{doc}.txt (KB files - skip compilation)
        let parts: Vec<&str> = file_path.split('/').collect();
        if parts.len() < 2 {
            return Err("Invalid file path format".into());
        }

    // Determine bot name and work directory structure
    let (_bot_name, work_dir) = if parts[0].ends_with(".gbai") {
        // Full path: {bot}.gbai/{bot}.gbdialog/{tool}.bas
        let bot_name = parts[0].strip_suffix(".gbai").unwrap_or(parts[0]);
        let work_dir = self.work_root.join(format!("{}.gbai/{}.gbdialog", bot_name, bot_name));
        (bot_name, work_dir)
    } else if parts.len() >= 2 && parts[0].ends_with(".gbdialog") {
        // Short path: {bot}.gbdialog/{tool}.bas
        let bot_name = parts[0].strip_suffix(".gbdialog").unwrap_or(parts[0]);
        let work_dir = self.work_root.join(format!("{}.gbai/{}.gbdialog", bot_name, bot_name));
        (bot_name, work_dir)
    } else if parts.len() >= 2 && parts[0].ends_with(".gbkb") {
        // KB file: {bot}.gbkb/{doc}.txt - skip compilation
        debug!("Skipping KB file: {}", file_path);
        return Ok(());
    } else {
        warn!("Unknown file path format: {}", file_path);
        return Err("Invalid file path format".into());
    };

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
            warn!("File {} not found in work dir, attempting to download from S3", work_bas_path.display());
            
            // Download in separate task to avoid Send issues
            let download_result = download_from_s3(file_path).await;
            
            match download_result {
                Ok(content) => {
                    if let Err(e) = std::fs::write(&work_bas_path, content) {
                        warn!("Failed to write {} to work dir: {}", work_bas_path.display(), e);
                        return Err(format!("Failed to write file: {}", e).into());
                    }
                    info!("Downloaded {} to {}", file_path, work_bas_path.display());
                }
                Err(e) => {
                    warn!("Failed to download {} from S3: {}", file_path, e);
                    return Err(format!("File not found in S3: {}", file_path).into());
                }
            }
        }

        // Verify file exists now
        if !work_bas_path.exists() {
            warn!("File {} still not found after download attempt", work_bas_path.display());
            return Ok(());
        }

        // Ler conteúdo
        let _content = std::fs::read_to_string(&work_bas_path)?;

        // Compilar com BasicCompiler (já está no work dir, então compila in-place)
        let mut compiler = BasicCompiler::new(self.state.clone(), bot_id);
        compiler.compile_file(
            work_bas_path.to_str().ok_or("Invalid path")?,
            work_dir.to_str().ok_or("Invalid path")?
        )?;

        info!("Compiled {} to {}.ast", file_path, tool_name);
        Ok(())
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
