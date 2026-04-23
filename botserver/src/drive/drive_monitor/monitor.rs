use std::sync::atomic::Ordering;
use std::time::Duration;

use super::types::DriveMonitor;

/// Intervalo de verificação do DriveMonitor e DriveCompiler (em segundos)
pub const CHECK_INTERVAL_SECS: u64 = 1;

impl DriveMonitor {
    pub fn calculate_backoff(&self) -> Duration {
        let failures = self.consecutive_failures.load(Ordering::Relaxed);
        if failures == 0 {
            return Duration::from_secs(CHECK_INTERVAL_SECS);
        }
        let backoff_secs = CHECK_INTERVAL_SECS * (1u64 << failures.min(4));
        Duration::from_secs(backoff_secs.min(300))
    }
}
