use log::{debug, info, trace, warn};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, RwLock};
use std::time::{Duration, Instant};
#[cfg(feature = "monitoring")]
use sysinfo::{Pid, ProcessesToUpdate, System};

static THREAD_REGISTRY: LazyLock<RwLock<HashMap<String, ThreadInfo>>> =
LazyLock::new(|| RwLock::new(HashMap::new()));

static COMPONENT_TRACKER: LazyLock<ComponentMemoryTracker> =
LazyLock::new(|| ComponentMemoryTracker::new(60));

#[derive(Debug, Clone)]
pub struct ThreadInfo {
pub name: String,
pub started_at: Instant,
pub last_activity: Instant,
pub activity_count: u64,
pub component: String,
}

pub fn register_thread(name: &str, component: &str) {
let info = ThreadInfo {
name: name.to_string(),
started_at: Instant::now(),
last_activity: Instant::now(),
activity_count: 0,
component: component.to_string(),
};
if let Ok(mut registry) = THREAD_REGISTRY.write() {
registry.insert(name.to_string(), info);
}
trace!("[THREAD] Registered: {} (component: {})", name, component);
}

pub fn record_thread_activity(name: &str) {
if let Ok(mut registry) = THREAD_REGISTRY.write() {
if let Some(info) = registry.get_mut(name) {
info.last_activity = Instant::now();
info.activity_count += 1;
}
}
}

pub fn unregister_thread(name: &str) {
if let Ok(mut registry) = THREAD_REGISTRY.write() {
registry.remove(name);
}
info!("[THREAD] Unregistered: {}", name);
}

pub fn log_thread_stats() {
if let Ok(registry) = THREAD_REGISTRY.read() {
info!("[THREADS] Active thread count: {}", registry.len());
for (name, info) in registry.iter() {
let uptime = info.started_at.elapsed().as_secs();
let idle = info.last_activity.elapsed().as_secs();
info!(
"[THREAD] {} | component={} | uptime={}s | idle={}s | activities={}",
name, info.component, uptime, idle, info.activity_count
);
}
}
}

#[derive(Debug, Clone)]
pub struct MemoryStats {
pub rss_bytes: u64,
pub virtual_bytes: u64,
pub timestamp: Instant,
}

impl MemoryStats {
pub fn current() -> Self {
let (rss, virt) = get_process_memory().unwrap_or((0, 0));
Self {
rss_bytes: rss,
virtual_bytes: virt,
timestamp: Instant::now(),
}
}


pub fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

pub fn log(&self) {
    info!(
        "[MEMORY] RSS={}, Virtual={}",
        Self::format_bytes(self.rss_bytes),
        Self::format_bytes(self.virtual_bytes),
    );
}

}

/// Get jemalloc memory statistics when the feature is enabled
#[cfg(feature = "jemalloc")]
pub fn get_jemalloc_stats() -> Option<JemallocStats> {
use tikv_jemalloc_ctl::{epoch, stats};


// Advance the epoch to refresh statistics
if epoch::advance().is_err() {
    return None;
}

let allocated = stats::allocated::read().ok()? as u64;
let active = stats::active::read().ok()? as u64;
let resident = stats::resident::read().ok()? as u64;
let mapped = stats::mapped::read().ok()? as u64;
let retained = stats::retained::read().ok()? as u64;

Some(JemallocStats {
    allocated,
    active,
    resident,
    mapped,
    retained,
})

}

#[cfg(not(feature = "jemalloc"))]
pub fn get_jemalloc_stats() -> Option<JemallocStats> {
None
}

/// Jemalloc memory statistics
#[derive(Debug, Clone)]
pub struct JemallocStats {
/// Total bytes allocated by the application
pub allocated: u64,
/// Total bytes in active pages allocated by the application
pub active: u64,
/// Total bytes in physically resident pages
pub resident: u64,
/// Total bytes in active extents mapped by the allocator
pub mapped: u64,
/// Total bytes retained (not returned to OS)
pub retained: u64,
}

impl JemallocStats {
pub fn log(&self) {
info!(
"[JEMALLOC] allocated={} active={} resident={} mapped={} retained={}",
MemoryStats::format_bytes(self.allocated),
MemoryStats::format_bytes(self.active),
MemoryStats::format_bytes(self.resident),
MemoryStats::format_bytes(self.mapped),
MemoryStats::format_bytes(self.retained),
);
}


/// Calculate fragmentation ratio (1.0 = no fragmentation)
pub fn fragmentation_ratio(&self) -> f64 {
    if self.allocated > 0 {
        self.active as f64 / self.allocated as f64
    } else {
        1.0
    }
}

}

/// Log jemalloc stats if available
pub fn log_jemalloc_stats() {
if let Some(stats) = get_jemalloc_stats() {
stats.log();
let frag = stats.fragmentation_ratio();
if frag > 1.5 {
warn!("[JEMALLOC] High fragmentation detected: {:.2}x", frag);
}
}
}

#[derive(Debug)]
pub struct MemoryCheckpoint {
pub name: String,
pub stats: MemoryStats,
}

impl MemoryCheckpoint {
pub fn new(name: &str) -> Self {
let stats = MemoryStats::current();
info!(
"[CHECKPOINT] {} started at RSS={}",
name,
MemoryStats::format_bytes(stats.rss_bytes)
);
Self {
name: name.to_string(),
stats,
}
}


pub fn compare_and_log(&self) {
    let current = MemoryStats::current();
    let diff = current.rss_bytes as i64 - self.stats.rss_bytes as i64;

    if diff > 0 {
        warn!(
            "[CHECKPOINT] {} INCREASED by {}",
            self.name,
            MemoryStats::format_bytes(diff as u64),
        );
    } else if diff < 0 {
        info!(
            "[CHECKPOINT] {} decreased by {}",
            self.name,
            MemoryStats::format_bytes((-diff) as u64),
        );
    } else {
        debug!("[CHECKPOINT] {} unchanged", self.name);
    }
}

}

pub struct ComponentMemoryTracker {
components: Mutex<HashMap<String, Vec<MemoryStats>>>,
max_history: usize,
}

impl ComponentMemoryTracker {
pub fn new(max_history: usize) -> Self {
Self {
components: Mutex::new(HashMap::new()),
max_history,
}
}


pub fn record(&self, component: &str) {
    let stats = MemoryStats::current();
    if let Ok(mut components) = self.components.lock() {
        let history = components.entry(component.to_string()).or_default();
        history.push(stats);

        if history.len() > self.max_history {
            history.remove(0);
        }
    }
}

pub fn get_growth_rate(&self, component: &str) -> Option<f64> {
    if let Ok(components) = self.components.lock() {
        if let Some(history) = components.get(component) {
            if history.len() >= 2 {
                let first = &history[0];
                let last = &history[history.len() - 1];
                let duration = last.timestamp.duration_since(first.timestamp).as_secs_f64();
                if duration > 0.0 {
                    let byte_diff = last.rss_bytes as f64 - first.rss_bytes as f64;
                    return Some(byte_diff / duration);
                }
            }
        }
    }
    None
}

pub fn log_all(&self) {
    if let Ok(components) = self.components.lock() {
        for (name, history) in components.iter() {
            if let Some(last) = history.last() {
                let growth = self.get_growth_rate(name);
                let growth_str = growth
                    .map(|g| {
                        let sign = if g >= 0.0 { "+" } else { "-" };
                        format!("{}{}/s", sign, MemoryStats::format_bytes(g.abs() as u64))
                    })
                    .unwrap_or_else(|| "N/A".to_string());
                info!(
                    "[COMPONENT] {} | RSS={} | Growth={}",
                    name,
                    MemoryStats::format_bytes(last.rss_bytes),
                    growth_str
                );
            }
        }
    }
}

}

pub fn record_component(component: &str) {
COMPONENT_TRACKER.record(component);
}

pub fn log_component_stats() {
COMPONENT_TRACKER.log_all();
}

pub struct LeakDetector {
baseline: Mutex<u64>,
growth_threshold_bytes: u64,
consecutive_growth_count: Mutex<usize>,
max_consecutive_growth: usize,
}

impl LeakDetector {
pub fn new(growth_threshold_mb: u64, max_consecutive_growth: usize) -> Self {
Self {
baseline: Mutex::new(0),
growth_threshold_bytes: growth_threshold_mb * 1024 * 1024,
consecutive_growth_count: Mutex::new(0),
max_consecutive_growth,
}
}


pub fn reset_baseline(&self) {
    let current = MemoryStats::current();
    if let Ok(mut baseline) = self.baseline.lock() {
        *baseline = current.rss_bytes;
    }
    if let Ok(mut count) = self.consecutive_growth_count.lock() {
        *count = 0;
    }
}

pub fn check(&self) -> Option<String> {
    let current = MemoryStats::current();

    let baseline_val = match self.baseline.lock() {
        Ok(b) => *b,
        Err(_) => return None,
    };

    if baseline_val == 0 {
        if let Ok(mut baseline) = self.baseline.lock() {
            *baseline = current.rss_bytes;
        }
        return None;
    }

    let growth = current.rss_bytes.saturating_sub(baseline_val);

    if growth > self.growth_threshold_bytes {
        let count = match self.consecutive_growth_count.lock() {
            Ok(mut c) => {
                *c += 1;
                *c
            }
            Err(_) => return None,
        };

        if count >= self.max_consecutive_growth {
            return Some(format!(
                "POTENTIAL MEMORY LEAK: grew by {} over {} checks. RSS={}, Baseline={}",
                MemoryStats::format_bytes(growth),
                count,
                MemoryStats::format_bytes(current.rss_bytes),
                MemoryStats::format_bytes(baseline_val),
            ));
        }
    } else {
        if let Ok(mut count) = self.consecutive_growth_count.lock() {
            *count = 0;
        }
        if let Ok(mut baseline) = self.baseline.lock() {
            *baseline = current.rss_bytes;
        }
    }

    None
}

}

pub fn start_memory_monitor(interval_secs: u64, warn_threshold_mb: u64) {
let detector = LeakDetector::new(warn_threshold_mb, 5);


tokio::spawn(async move {
    register_thread("memory-monitor", "monitoring");

    info!(
        "[MONITOR] Started (interval={}s, threshold={}MB)",
        interval_secs, warn_threshold_mb
    );

    let mut prev_rss: u64 = 0;
    let mut tick_count: u64 = 0;

    // First 2 minutes: check every 10 seconds for aggressive tracking
    // After that: use normal interval
    let startup_interval = Duration::from_secs(10);
    let normal_interval = Duration::from_secs(interval_secs);
    let startup_ticks = 12; // 2 minutes of 10-second intervals

    let mut interval = tokio::time::interval(startup_interval);

    loop {
        interval.tick().await;
        tick_count += 1;
        record_thread_activity("memory-monitor");

        let stats = MemoryStats::current();
        let rss_diff = if prev_rss > 0 {
            stats.rss_bytes as i64 - prev_rss as i64
        } else {
            0
        };

        let diff_str = if rss_diff > 0 {
            format!("+{}", MemoryStats::format_bytes(rss_diff as u64))
        } else if rss_diff < 0 {
            format!("-{}", MemoryStats::format_bytes((-rss_diff) as u64))
        } else {
            "±0".to_string()
        };

        trace!(
            "[MONITOR] tick={} RSS={} ({}) Virtual={}",
            tick_count,
            MemoryStats::format_bytes(stats.rss_bytes),
            diff_str,
            MemoryStats::format_bytes(stats.virtual_bytes),
        );

        // Log jemalloc stats every 5 ticks if available
        if tick_count.is_multiple_of(5) {
            log_jemalloc_stats();
        }

        prev_rss = stats.rss_bytes;
        record_component("global");

        if let Some(warning) = detector.check() {
            warn!("[MONITOR] {}", warning);
            stats.log();
            log_component_stats();
            log_thread_stats();
        }

        // Switch to normal interval after startup period
        if tick_count == startup_ticks {
            trace!("[MONITOR] Switching to normal interval ({}s)", interval_secs);
            interval = tokio::time::interval(normal_interval);
        }
    }
});

}

#[cfg(feature = "monitoring")]
#[cfg(feature = "monitoring")]
pub fn get_process_memory() -> Option<(u64, u64)> {
let pid = Pid::from_u32(std::process::id());
let mut sys = System::new();
sys.refresh_processes(ProcessesToUpdate::Some(&[pid]), true);


sys.process(pid).map(|p| (p.memory(), p.virtual_memory()))

}

#[cfg(not(feature = "monitoring"))]
pub fn get_process_memory() -> Option<(u64, u64)> {
None
}

pub fn log_process_memory() {
if let Some((rss, virt)) = get_process_memory() {
trace!(
"[PROCESS] RSS={}, Virtual={}",
MemoryStats::format_bytes(rss),
MemoryStats::format_bytes(virt)
);
}
}

#[cfg(test)]
mod tests {
use super::*;


#[test]
fn test_memory_stats() {
    let stats = MemoryStats::current();
    assert!(stats.rss_bytes > 0 || stats.virtual_bytes >= 0);
}

#[test]
fn test_format_bytes() {
    assert_eq!(MemoryStats::format_bytes(500), "500 B");
    assert_eq!(MemoryStats::format_bytes(1024), "1.00 KB");
    assert_eq!(MemoryStats::format_bytes(1024 * 1024), "1.00 MB");
    assert_eq!(MemoryStats::format_bytes(1024 * 1024 * 1024), "1.00 GB");
}

#[test]
fn test_checkpoint() {
    let checkpoint = MemoryCheckpoint::new("test");
    checkpoint.compare_and_log();
}

#[test]
fn test_thread_registry() {
    register_thread("test-thread", "test-component");
    record_thread_activity("test-thread");
    log_thread_stats();
    unregister_thread("test-thread");
}

}
