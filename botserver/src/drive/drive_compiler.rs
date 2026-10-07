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

mod compile;
mod queue;

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
