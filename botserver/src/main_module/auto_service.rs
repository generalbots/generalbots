#[cfg(feature = "automation")]
use log::{error, trace};
#[cfg(feature = "automation")]
use botcore::shared::memory_monitor::{MemoryStats, register_thread};

pub fn start_automation_service(app_state: std::sync::Arc<botcore::shared::state::AppState>) {
    #[cfg(feature = "automation")]
    {
        tokio::spawn(async move {
            register_thread("automation-service", "automation");
            let automation = crate::core::automation::AutomationService::new(app_state);
            trace!(
                "[TASK] AutomationService starting, RSS={}",
                MemoryStats::format_bytes(MemoryStats::current().rss_bytes)
            );
            loop {
                botcore::shared::memory_monitor::record_thread_activity("automation-service");
                if let Err(e) = automation.check_scheduled_tasks().await {
                    error!("Error checking scheduled tasks: {}", e);
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(60)).await;
            }
        });
    }
    #[cfg(not(feature = "automation"))]
    let _ = app_state;
}
