use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use crate::limits::{LimitExceeded, LimitType, SystemLimits};

#[derive(Debug)]
struct RateLimitEntry {
    count: AtomicU64,
    window_start: RwLock<Instant>,
}

impl RateLimitEntry {
    fn new() -> Self {
        Self {
            count: AtomicU64::new(0),
            window_start: RwLock::new(Instant::now()),
        }
    }
}

#[derive(Debug)]
pub struct RateLimiter {
    limits: SystemLimits,
    per_user_minute: RwLock<HashMap<String, Arc<RateLimitEntry>>>,
    per_user_hour: RwLock<HashMap<String, Arc<RateLimitEntry>>>,
    global_minute: Arc<RateLimitEntry>,
    global_hour: Arc<RateLimitEntry>,
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new(SystemLimits::default())
    }
}

impl RateLimiter {
    pub fn new(limits: SystemLimits) -> Self {
        Self {
            limits,
            per_user_minute: RwLock::new(HashMap::new()),
            per_user_hour: RwLock::new(HashMap::new()),
            global_minute: Arc::new(RateLimitEntry::new()),
            global_hour: Arc::new(RateLimitEntry::new()),
        }
    }

    pub async fn check_rate_limit(&self, user_id: &str) -> Result<(), LimitExceeded> {
        self.check_global_limits().await?;
        self.check_user_limits(user_id).await
    }

    async fn check_global_limits(&self) -> Result<(), LimitExceeded> {
        let now = Instant::now();
        {
            let window_start = self.global_minute.window_start.read().await;
            if now.duration_since(*window_start) > Duration::from_secs(60) {
                drop(window_start);
                let mut window_start = self.global_minute.window_start.write().await;
                *window_start = now;
                self.global_minute.count.store(0, Ordering::SeqCst);
            }
        }
        let count = self.global_minute.count.fetch_add(1, Ordering::SeqCst) + 1;
        let max = u64::from(self.limits.max_api_calls_per_minute) * 100;
        if count > max {
            self.global_minute.count.fetch_sub(1, Ordering::SeqCst);
            return Err(LimitExceeded {
                limit_type: LimitType::ApiCallsMinute,
                current: count,
                maximum: max,
                retry_after_secs: Some(60),
            });
        }
        {
            let window_start = self.global_hour.window_start.read().await;
            if now.duration_since(*window_start) > Duration::from_secs(3600) {
                drop(window_start);
                let mut window_start = self.global_hour.window_start.write().await;
                *window_start = now;
                self.global_hour.count.store(0, Ordering::SeqCst);
            }
        }
        let hour_count = self.global_hour.count.fetch_add(1, Ordering::SeqCst) + 1;
        let hour_max = u64::from(self.limits.max_api_calls_per_hour) * 100;
        if hour_count > hour_max {
            self.global_hour.count.fetch_sub(1, Ordering::SeqCst);
            return Err(LimitExceeded {
                limit_type: LimitType::ApiCallsHour,
                current: hour_count,
                maximum: hour_max,
                retry_after_secs: Some(3600),
            });
        }
        Ok(())
    }

    async fn check_user_limits(&self, user_id: &str) -> Result<(), LimitExceeded> {
        self.check_user_minute_limit(user_id).await?;
        self.check_user_hour_limit(user_id).await
    }

    async fn check_user_minute_limit(&self, user_id: &str) -> Result<(), LimitExceeded> {
        let entry = {
            let map = self.per_user_minute.read().await;
            map.get(user_id).cloned()
        };
        let entry = match entry {
            Some(e) => e,
            None => {
                let new_entry = Arc::new(RateLimitEntry::new());
                let mut map = self.per_user_minute.write().await;
                map.insert(user_id.to_string(), Arc::clone(&new_entry));
                new_entry
            }
        };
        let now = Instant::now();
        {
            let window_start = entry.window_start.read().await;
            if now.duration_since(*window_start) > Duration::from_secs(60) {
                drop(window_start);
                let mut window_start = entry.window_start.write().await;
                *window_start = now;
                entry.count.store(0, Ordering::SeqCst);
            }
        }
        let count = entry.count.fetch_add(1, Ordering::SeqCst) + 1;
        let max = u64::from(self.limits.max_api_calls_per_minute);
        if count > max {
            entry.count.fetch_sub(1, Ordering::SeqCst);
            return Err(LimitExceeded {
                limit_type: LimitType::ApiCallsMinute,
                current: count,
                maximum: max,
                retry_after_secs: Some(60),
            });
        }
        Ok(())
    }

    async fn check_user_hour_limit(&self, user_id: &str) -> Result<(), LimitExceeded> {
        let entry = {
            let map = self.per_user_hour.read().await;
            map.get(user_id).cloned()
        };
        let entry = match entry {
            Some(e) => e,
            None => {
                let new_entry = Arc::new(RateLimitEntry::new());
                let mut map = self.per_user_hour.write().await;
                map.insert(user_id.to_string(), Arc::clone(&new_entry));
                new_entry
            }
        };
        let now = Instant::now();
        {
            let window_start = entry.window_start.read().await;
            if now.duration_since(*window_start) > Duration::from_secs(3600) {
                drop(window_start);
                let mut window_start = entry.window_start.write().await;
                *window_start = now;
                entry.count.store(0, Ordering::SeqCst);
            }
        }
        let count = entry.count.fetch_add(1, Ordering::SeqCst) + 1;
        let max = u64::from(self.limits.max_api_calls_per_hour);
        if count > max {
            entry.count.fetch_sub(1, Ordering::SeqCst);
            return Err(LimitExceeded {
                limit_type: LimitType::ApiCallsHour,
                current: count,
                maximum: max,
                retry_after_secs: Some(3600),
            });
        }
        Ok(())
    }

    pub async fn cleanup_stale_entries(&self) {
        let now = Instant::now();
        let stale_threshold = Duration::from_secs(7200);
        {
            let mut map = self.per_user_minute.write().await;
            let mut to_remove = Vec::new();
            for (user_id, entry) in map.iter() {
                let window_start = entry.window_start.read().await;
                if now.duration_since(*window_start) > stale_threshold {
                    to_remove.push(user_id.clone());
                }
            }
            for user_id in to_remove {
                map.remove(&user_id);
            }
        }
        {
            let mut map = self.per_user_hour.write().await;
            let mut to_remove = Vec::new();
            for (user_id, entry) in map.iter() {
                let window_start = entry.window_start.read().await;
                if now.duration_since(*window_start) > stale_threshold {
                    to_remove.push(user_id.clone());
                }
            }
            for user_id in to_remove {
                map.remove(&user_id);
            }
        }
    }
}
