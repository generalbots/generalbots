use std::sync::atomic::Ordering;
use std::time::Duration;

use super::types::DriveMonitor;

impl DriveMonitor {
    pub fn calculate_backoff(&self) -> Duration {
        let failures = self.consecutive_failures.load(Ordering::Relaxed);
        if failures == 0 {
            return Duration::from_secs(30);
        }
        let backoff_secs = 30u64 * (1u64 << failures.min(4));
        Duration::from_secs(backoff_secs.min(300))
    }
}
