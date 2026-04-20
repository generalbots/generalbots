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
use crate::core::shared::state::AppState;
use crate::core::shared::utils::get_work_path;
use crate::drive::drive_files::drive_files as drive_files_table;
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

        // Loop que verifica drive_files a cada 30s
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));

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
        // file_path: {bot}.gbai/{bot}.gbdialog/{tool}.bas
        let parts: Vec<&str> = file_path.split('/').collect();
        if parts.len() < 3 {
            return Err("Invalid file path format".into());
        }

        let bot_name = parts[0].trim_end_matches(".gbai");
        let tool_name = parts.last().ok_or("Invalid file path")?.trim_end_matches(".bas");

        // Work dir: /opt/gbo/work/{bot}.gbai/{bot}.gbdialog/
        let work_dir = self.work_root.join(format!("{}.gbai/{}.gbdialog", bot_name, bot_name));
        std::fs::create_dir_all(&work_dir)?;

        // Caminho do .bas no work
        let work_bas_path = work_dir.join(format!("{}.bas", tool_name));

        // Baixar do MinIO direto para work dir
        // (isso pressupõe que o DriveMonitor já sincronizou, ou buscamos do S3 aqui)
        // Por enquanto, assumimos que o arquivo já está em work dir de sincronização anterior
        // Se não existir, precisa buscar do S3

        if !work_bas_path.exists() {
            // Buscar do S3 - isso deveria ser feito pelo DriveMonitor
                // Por enquanto, apenas logamos
                warn!("File {} not found in work dir, skipping", work_bas_path.display());
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
