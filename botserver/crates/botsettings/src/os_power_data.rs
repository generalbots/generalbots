//! Battery and AC telemetry for the shell-mode OS panel (#2445).
//!
//! Reads `/sys/class/power_supply` with no extra dependencies. Returns an
//! explicit no-battery state when no battery sources are exposed.

use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Clone)]
pub(crate) struct PowerSourceInfo {
    pub(crate) name: String,
    pub(crate) kind: String,
    pub(crate) present: bool,
    pub(crate) status: Option<String>,
    pub(crate) capacity_percent: Option<u8>,
    pub(crate) online: Option<bool>,
    pub(crate) time_remaining_minutes: Option<u64>,
}

#[derive(Serialize)]
pub(crate) struct PowerStatus {
    pub(crate) present: bool,
    pub(crate) percent: Option<u8>,
    pub(crate) state: String,
    pub(crate) time_remaining_minutes: Option<u64>,
    pub(crate) ac_online: bool,
    pub(crate) sources: Vec<PowerSourceInfo>,
}

fn read_sysfs_trim(path: &Path) -> Option<String> {
    std::fs::read_to_string(path)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn read_sysfs_u64(path: &Path) -> Option<u64> {
    read_sysfs_trim(path).and_then(|v| v.parse::<u64>().ok())
}

fn power_source(base: &Path, name: &str) -> Option<PowerSourceInfo> {
    let dir = base.join(name);
    let kind = read_sysfs_trim(&dir.join("type")).unwrap_or_else(|| "unknown".to_string());
    let present = read_sysfs_trim(&dir.join("present"))
        .map(|v| v == "1")
        .unwrap_or(true);
    let status = read_sysfs_trim(&dir.join("status"));
    let capacity_percent = read_sysfs_u64(&dir.join("capacity")).map(|v| v.min(100) as u8);
    let online = read_sysfs_trim(&dir.join("online")).map(|v| v == "1");

    let mut time_remaining_minutes = None;
    if let Some(time_now) = read_sysfs_u64(&dir.join("time_to_empty_now"))
        .or_else(|| read_sysfs_u64(&dir.join("time_to_empty_avg")))
    {
        if time_now > 0 && time_now < 1_000_000 {
            time_remaining_minutes = Some((time_now + 30) / 60);
        }
    } else {
        let energy_now = read_sysfs_u64(&dir.join("energy_now"))
            .or_else(|| read_sysfs_u64(&dir.join("charge_now")));
        let power_now = read_sysfs_u64(&dir.join("power_now"))
            .or_else(|| read_sysfs_u64(&dir.join("current_now")));
        if let (Some(energy), Some(power)) = (energy_now, power_now) {
            if power > 0 {
                let minutes = ((energy as f64 / power as f64) * 60.0).round() as u64;
                if minutes < 10080 {
                    time_remaining_minutes = Some(minutes);
                }
            }
        }
    }

    Some(PowerSourceInfo {
        name: name.to_string(),
        kind,
        present,
        status,
        capacity_percent,
        online,
        time_remaining_minutes,
    })
}

/// Reads battery/AC state from `/sys/class/power_supply` (no extra deps, #2445).
pub(crate) fn read_power_status() -> PowerStatus {
    let base = Path::new("/sys/class/power_supply");
    let mut sources = Vec::new();
    if let Ok(entries) = std::fs::read_dir(base) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Some(source) = power_source(base, &name) {
                sources.push(source);
            }
        }
    }
    sources.sort_by(|a, b| a.name.cmp(&b.name));

    let ac_online = sources.iter().any(|s| {
        (s.kind.eq_ignore_ascii_case("mains")
            || s.kind.eq_ignore_ascii_case("usb")
            || s.kind.eq_ignore_ascii_case("usb_pd"))
            && s.online.unwrap_or(false)
    });

    let batteries: Vec<&PowerSourceInfo> = sources
        .iter()
        .filter(|s| s.kind.eq_ignore_ascii_case("battery") && s.present)
        .collect();
    let present = !batteries.is_empty();
    let percent = if batteries.is_empty() {
        None
    } else {
        let mut sum: u64 = 0;
        let mut count: u64 = 0;
        for b in &batteries {
            if let Some(p) = b.capacity_percent {
                sum += p as u64;
                count += 1;
            }
        }
        if count == 0 {
            None
        } else {
            Some(((sum + count / 2) / count).min(100) as u8)
        }
    };
    let state = batteries
        .iter()
        .find_map(|b| b.status.clone())
        .unwrap_or_else(|| {
            if present {
                "Unknown".to_string()
            } else {
                "NoBattery".to_string()
            }
        });
    let time_remaining_minutes = batteries.iter().find_map(|b| b.time_remaining_minutes);

    PowerStatus {
        present,
        percent,
        state,
        time_remaining_minutes,
        ac_online,
        sources,
    }
}

#[cfg(test)]
mod tests {
    use super::PowerStatus;

    fn _assert_send_sync<T: Send + Sync>() {}

    #[test]
    fn power_status_is_send_sync() {
        _assert_send_sync::<PowerStatus>();
    }
}
