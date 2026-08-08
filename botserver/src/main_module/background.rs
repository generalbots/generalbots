//! background - extracted from bootstrap.rs

use botcore::shared::state::AppState;
use crate::core::bot::BotOrchestrator;
use crate::llm::local::ensure_llama_servers_running;
use log::{error, info, trace};
use std::sync::Arc;

use super::drive_monitors::{start_drive_monitors, start_drive_compiler};

/// Runs every 5 minutes and promotes trialing `billing_recurring` rows whose
/// trial period has ended to `active` at the plan price, generating the first
/// invoice (issue #778). The signup flow writes trials straight to the DB, so
/// this DB-driven job is the only path that converts them to paid.
fn start_trial_promotion_guard(app_state: Arc<AppState>) {
    #[cfg(feature = "billing")]
    {
        let pool = app_state.conn.clone();
        tokio::spawn(async move {
            info!("Billing trial promotion guard started (every 5 minutes)");
            loop {
                match pool.get() {
                    Ok(mut conn) => {
                        if let Err(e) = botbilling::lifecycle::promote_expired_trials_in_db(&mut conn) {
                            error!("Billing trial promotion failed: {}", e);
                        }
                    }
                    Err(e) => error!("Billing trial promotion pool error: {}", e),
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;
            }
        });
    }
    #[cfg(not(feature = "billing"))]
    let _ = app_state;
}


pub async fn start_background_services(
  app_state: Arc<AppState>,
  _pool: &botcore::shared::utils::DbPool,
) {
    use botcore::shared::memory_monitor::{log_process_memory, start_memory_monitor};

    // Resume workflows after server restart
    if let Err(e) =
        crate::basic::keywords::orchestration::resume_workflows_on_startup(Arc::new(crate::basic::AppStateBasicRuntime(app_state.clone()))).await
    {
        log::warn!("Failed to resume workflows on startup: {}", e);
    }

    #[cfg(feature = "tasks")]
    {
        let tasks_state = Arc::new(crate::tasks::TasksState {
            pool: app_state.conn.clone(),
            run_command: Arc::new(|_cmd: &str, _args: &[&str]| -> Result<String, String> {
                Ok("stub".to_string())
            }),
            call_llm: Arc::new(|_sys: &str, _prompt: &str| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>> {
                Box::pin(async { Ok("stub".to_string()) })
            }),
            get_config: Arc::new(|_key: &str| -> Result<String, String> {
                Ok("stub".to_string())
            }),
            cache_get: Arc::new(|_key: String| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<String>, String>> + Send>> {
                Box::pin(async { Ok(None) })
            }),
            cache_set: Arc::new(|_key: String, _value: String, _ttl: Option<u64>| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>> {
                Box::pin(async { Ok(()) })
            }),
        });
        let task_scheduler = Arc::new(crate::tasks::scheduler::TaskScheduler::new(tasks_state));
        task_scheduler.start();
    }

    #[cfg(any(feature = "research", feature = "llm"))]
    if let Err(e) = crate::core::kb::ensure_crawler_service_running(app_state.clone()).await {
        log::warn!("Failed to start website crawler service: {}", e);
    }

    // Start memory monitoring - check every 30 seconds, warn if growth > 50MB
    start_memory_monitor(30, 50);
    info!("Memory monitor started");
    log_process_memory();

    let bot_orchestrator = BotOrchestrator::new(app_state.clone());
    if let Err(e) = bot_orchestrator.mount_all_bots() {
        error!("Failed to mount bots: {}", e);
    }

    #[cfg(feature = "llm")]
    {
        let app_state_for_llm = app_state.clone();
        trace!("ensure_llama_servers_running starting...");
        if let Err(e) = ensure_llama_servers_running(app_state_for_llm).await {
            error!("Failed to start LLM servers: {}", e);
        }
        trace!("ensure_llama_servers_running completed");
    }

  // Start DriveMonitor for S3/MinIO file watching and syncing
  #[cfg(feature = "drive")]
  start_drive_monitors(app_state.clone(), _pool).await;

  // Start DriveCompiler to compile .bas files from drive_files table
  #[cfg(feature = "drive")]
  start_drive_compiler(app_state.clone()).await;

  // Start billing trial promotion (trialing -> active + first invoice)
  start_trial_promotion_guard(app_state.clone());
    // start_config_watcher(app_state.clone()).await;
}
