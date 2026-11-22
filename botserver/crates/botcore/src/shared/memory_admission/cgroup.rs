//! One-shot cgroup memory ceiling detection with a graceful fallback.
//!
//! Never panics and never starts a watcher: a missing, unlimited (`max`) or
//! malformed control file simply yields the conservative fallback, which keeps
//! the admission thresholds usable on a plain laptop or inside a desktop
//! installer where `/sys/fs/cgroup` is not mounted.

use serde::Serialize;
use std::path::Path;

/// Used when no finite cgroup limit can be read (unlimited, v1 sentinel,
/// unmounted or malformed control file). Deliberately small enough that a
/// single Office import cannot exhaust a typical 4-8 GB desktop.
pub const FALLBACK_LIMIT_BYTES: u64 = 1_073_741_824;
/// Floor for the derived per-request budget so tiny ceilings still admit the
/// smallest useful document instead of refusing everything.
pub const MIN_PER_REQUEST_BUDGET_BYTES: u64 = 32 * 1024 * 1024;

/// v1 reports "no limit" as a value close to `i64::MAX` rather than `max`.
const UNLIMITED_V1_THRESHOLD: u64 = 9_007_199_254_740_991;

const V2_ROOT: &str = "/sys/fs/cgroup";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CgroupSource {
    /// cgroup v2 `memory.max` / `memory.current`.
    V2,
    /// cgroup v1 `memory.limit_in_bytes` / `memory.usage_in_bytes`.
    V1,
    /// No finite limit was available; the fallback is in effect.
    Fallback,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct CgroupMemory {
    pub limit_bytes: u64,
    pub current_bytes: Option<u64>,
    pub source: CgroupSource,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LimitReading {
    Limited(u64),
    Unlimited,
    Missing,
}

impl CgroupMemory {
    /// Reads the host cgroup control files once. Safe to call on every boot;
    /// no loop, no background task, no I/O after this call.
    pub fn detect() -> Self {
        Self::detect_from(Path::new(V2_ROOT))
    }

    fn detect_from(root: &Path) -> Self {
        let (limit_bytes, source) = match read_limit(&root.join("memory.max")) {
            LimitReading::Limited(bytes) => (bytes, CgroupSource::V2),
            LimitReading::Unlimited => (FALLBACK_LIMIT_BYTES, CgroupSource::Fallback),
            LimitReading::Missing => match read_limit(&root.join("memory/memory.limit_in_bytes")) {
                LimitReading::Limited(bytes) if bytes < UNLIMITED_V1_THRESHOLD => {
                    (bytes, CgroupSource::V1)
                }
                LimitReading::Limited(_) | LimitReading::Unlimited => {
                    (FALLBACK_LIMIT_BYTES, CgroupSource::Fallback)
                }
                LimitReading::Missing => (FALLBACK_LIMIT_BYTES, CgroupSource::Fallback),
            },
        };

        let current_bytes = match read_number(&root.join("memory.current")) {
            Some(bytes) => Some(bytes),
            None => read_number(&root.join("memory/memory.usage_in_bytes")),
        };

        Self {
            limit_bytes,
            current_bytes,
            source,
        }
    }

    /// Headroom reported by the cgroup at detection time, saturating at zero
    /// when usage already exceeds the limit.
    pub fn available_bytes(&self) -> u64 {
        let used = match self.current_bytes {
            Some(bytes) => bytes,
            None => 0,
        };
        self.limit_bytes.saturating_sub(used)
    }

    /// Per-request peak-heap budget: three eighths of the ceiling, with a
    /// 32 MiB floor that never exceeds the ceiling itself. Leaving five eighths
    /// for the runtime, other requests and page cache keeps one import from
    /// taking the whole sidecar down.
    pub fn per_request_budget(&self) -> u64 {
        let scaled = self.limit_bytes.saturating_mul(3) / 8;
        let floor = MIN_PER_REQUEST_BUDGET_BYTES.min(self.limit_bytes);
        scaled.max(floor)
    }

    /// True when a finite cgroup limit was actually found.
    pub fn is_limited(&self) -> bool {
        self.source != CgroupSource::Fallback
    }
}

fn read_limit(path: &Path) -> LimitReading {
    let raw = match std::fs::read_to_string(path) {
        Ok(value) => value,
        Err(_) => return LimitReading::Missing,
    };
    parse_limit_text(&raw)
}

fn parse_limit_text(raw: &str) -> LimitReading {
    let text = raw.trim();
    if text.eq_ignore_ascii_case("max") {
        LimitReading::Unlimited
    } else {
        match text.parse::<u64>() {
            Ok(bytes) if bytes > 0 => LimitReading::Limited(bytes),
            _ => LimitReading::Missing,
        }
    }
}

fn read_number(path: &Path) -> Option<u64> {
    let raw = match std::fs::read_to_string(path) {
        Ok(value) => value,
        Err(_) => return None,
    };
    match raw.trim().parse::<u64>() {
        Ok(bytes) => Some(bytes),
        Err(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_v2_sentinel_and_numbers() {
        assert_eq!(parse_limit_text("max\n"), LimitReading::Unlimited);
        assert_eq!(parse_limit_text(" MAX "), LimitReading::Unlimited);
        assert_eq!(
            parse_limit_text("4294967296\n"),
            LimitReading::Limited(4_294_967_296)
        );
    }

    #[test]
    fn malformed_and_zero_limits_are_missing() {
        assert_eq!(parse_limit_text("garbage"), LimitReading::Missing);
        assert_eq!(parse_limit_text("0"), LimitReading::Missing);
        assert_eq!(parse_limit_text(""), LimitReading::Missing);
    }

    #[test]
    fn missing_control_files_fall_back_gracefully() {
        let missing = std::env::temp_dir().join("gb-cgroup-tests-no-such-root");
        let detected = CgroupMemory::detect_from(&missing);
        assert_eq!(detected.limit_bytes, FALLBACK_LIMIT_BYTES);
        assert_eq!(detected.source, CgroupSource::Fallback);
        assert!(!detected.is_limited());
        assert_eq!(detected.current_bytes, None);
    }

    #[test]
    fn budget_is_derived_from_the_ceiling() {
        let four_gib = CgroupMemory {
            limit_bytes: 4 * 1024 * 1024 * 1024,
            current_bytes: None,
            source: CgroupSource::V2,
        };
        assert_eq!(four_gib.per_request_budget(), 1536 * 1024 * 1024);
        let one_gib = CgroupMemory {
            limit_bytes: FALLBACK_LIMIT_BYTES,
            current_bytes: None,
            source: CgroupSource::Fallback,
        };
        assert_eq!(one_gib.per_request_budget(), 384 * 1024 * 1024);
    }

    #[test]
    fn tiny_ceiling_uses_floor_without_exceeding_it() {
        let tiny = CgroupMemory {
            limit_bytes: 16 * 1024 * 1024,
            current_bytes: None,
            source: CgroupSource::V1,
        };
        assert_eq!(tiny.per_request_budget(), 16 * 1024 * 1024);
    }

    #[test]
    fn available_bytes_never_underflows() {
        let over = CgroupMemory {
            limit_bytes: 1024,
            current_bytes: Some(4096),
            source: CgroupSource::V2,
        };
        assert_eq!(over.available_bytes(), 0);
    }
}
