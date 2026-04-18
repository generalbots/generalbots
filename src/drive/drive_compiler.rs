/// DriveCompiler - Unificado para compilar arquivos .bas do Drive (MinIO)
/// 
/// Fluxo:
/// 1. DriveMonitor (S3) baixa .bas do MinIO para /opt/gbo/data/{bot}.gbai/{bot}.gbdialog/
/// 2. DriveMonitor atualiza tabela drive_files com etag, last_modified
/// 3. DriveCompiler lê drive_files, detecta mudanças, compila para /opt/gbo/work/
/// 4. Compilados: .bas → .ast (Rhai)

use crate::basic::compiler::BasicCompiler;
use crate::core::shared::state::AppState;
use crate::core::shared::utils::get_work_path;
use crate::drive::drive_files::{drive_files as drive_files_table, DriveFileRepository};
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

/// Estado de compilação de um arquivo
#[derive(Debug, Clone)]
struct CompileState {
    etag: String,
    compiled: bool,
}

pub struct DriveCompiler {
    state: Arc<AppState>,
    work_root: PathBuf,
    is_processing: Arc<AtomicBool>,
    /// Últimos etags conhecidos: file_path -> etag
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
        info!("DriveCompiler started - monitoring drive_files table for changes");
        
        self.is_processing.store(true, Ordering::SeqCst);
        
        let compiler = self.clone();
        
        // Spawn loop que verifica drive_files a cada 30s
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
    
    /// Verifica drive_files e compila .bas files mudaram
    async fn check_and_compile(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        use drive_files_table::dsl::*;
        use diesel::dsl::eq;
        
        let mut conn = self.state.conn.get()?;
        
        // Selecionar todos os arquivos .gbdialog/*.bas não compilados ou com etag diferente
        let files: Vec<(Uuid, String, String, Option<String>)> = drive_files_table
            .filter(file_type.eq("bas"))
            .filter(file_path.like("%.gbdialog/%"))
            .select((bot_id, file_path, file_type, etag.clone()))
            .load(&mut conn)?;
        
        for (bot_id, file_path, _file_type, current_etag_opt) in files {
            let current_etag = current_etag_opt.unwrap_or_default();
            
            // Verificar se precisa compilar
            let should_compile = {
                let etags = self.last_etags.read().await;
                etags.get(&file_path).map(|e| e != &current_etag).unwrap_or(true)
            };
            
            if should_compile {
                debug!("DriveCompiler: {} changed, compiling...", file_path);
                
                // Compilar
                if let Err(e) = self.compile_file(bot_id, &file_path).await {
                    error!("Failed to compile {}: {}", file_path, e);
                } else {
                    // Atualizar estado
                    let mut etags = self.last_etags.write().await;
                    etags.insert(file_path.clone(), current_etag.clone());
                    
                    // Marcar como compilado na DB
                    diesel::update(drive_files_table
                        .filter(bot_id.eq(bot_id))
                        .filter(file_path.eq(&file_path)))
                        .set(indexed.eq(true))
                        .execute(&mut conn)?;
                    
                    info!("DriveCompiler: {} compiled successfully", file_path);
                }
            }
        }
        
        Ok(())
    }
    
    /// Compilar um arquivo .bas → .ast
    async fn compile_file(&self, bot_id: Uuid, file_path: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        // Extrair nome do bot e tool
        // file_path: salesianos.gbai/salesianos.gbdialog/tool.bas
        let parts: Vec<&str> = file_path.split('/').collect();
        if parts.len() < 3 {
            return Err("Invalid file path format".into());
        }
        
        let bot_name = parts[0].trim_end_matches(".gbai");
        let tool_name = parts.last().unwrap().trim_end_matches(".bas");
        
        // Caminho do arquivo .bas em /opt/gbo/data/
        let bas_path = format!("/opt/gbo/data/{}.gbai/{}.gbdialog/{}.bas", 
            bot_name, bot_name, tool_name);
        
        // Ler conteúdo
        let content = std::fs::read_to_string(&bas_path)
            .map_err(|e| format!("Failed to read {}: {}", bas_path, e))?;
        
        // Criar work dir
        let work_dir = self.work_root.join(format!("{}.gbai/{}.gbdialog", bot_name, bot_name));
        std::fs::create_dir_all(&work_dir)?;
        
        // Escrever .bas em work
        let work_bas_path = work_dir.join(format!("{}.bas", tool_name));
        std::fs::write(&work_bas_path, &content)?;
        
        // Compilar com BasicCompiler
        let mut compiler = BasicCompiler::new(self.state.clone(), bot_id);
        compiler.compile_file(
            work_bas_path.to_str().ok_or("Invalid path")?,
            work_dir.to_str().ok_or("Invalid path")?
        )?;
        
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
