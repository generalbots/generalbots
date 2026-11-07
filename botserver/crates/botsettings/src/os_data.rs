//! Host probes for the shell-mode Operating System panel (#1852).
//!
//! Kept separate from the routes so each file stays small. Every command goes
//! through `SafeCommand`; the readers only touch documented `/proc` and
//! `/etc` files otherwise.

use botsecurity::command_guard::SafeCommand;
use serde::Serialize;
use std::path::{Path, PathBuf};

/// Allow-listed services. The second column is the systemd unit, the third is
/// the process name used by `pgrep -x` for the status probe.
pub(crate) const SERVICES: &[(&str, &str, &str)] = &[
    ("botserver", "botserver", "botserver"),
    ("postgresql", "postgresql", "postgres"),
    ("valkey", "valkey", "valkey-server"),
    ("minio", "minio", "minio"),
    ("qdrant", "qdrant", "qdrant"),
    ("vault", "vault", "vault"),
    ("zitadel", "zitadel", "zitadel"),
    ("llm", "llama-server", "llama-server"),
    ("caddy", "caddy", "caddy"),
    ("stalwart", "stalwart", "stalwart"),
];

#[derive(Serialize)]
pub(crate) struct DiskInfo {
    pub(crate) filesystem: String,
    pub(crate) mount: String,
    pub(crate) total_bytes: u64,
    pub(crate) used_bytes: u64,
    pub(crate) percent: u8,
}

#[derive(Serialize)]
pub(crate) struct ServiceInfo {
    pub(crate) name: String,
    pub(crate) unit: String,
    pub(crate) running: bool,
}

fn run(program: &str, args: &[&str]) -> Option<String> {
    let mut command = SafeCommand::new(program).ok()?;
    for arg in args {
        command = command.arg(*arg).ok()?;
    }
    let output = command.execute().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).to_string())
}

pub(crate) fn read_hostname() -> String {
    for path in ["/etc/hostname", "/proc/sys/kernel/hostname"] {
        if let Some(value) = std::fs::read_to_string(path)
            .ok()
            .and_then(|text| text.lines().next().map(str::trim).map(str::to_string))
            .filter(|value| !value.is_empty())
        {
            return value;
        }
    }
    std::env::var("COMPUTERNAME")
        .or_else(|_| std::env::var("HOSTNAME"))
        .unwrap_or_else(|_| "unknown".to_string())
}

pub(crate) fn read_os() -> String {
    std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|text| {
            text.lines().find_map(|line| {
                line.strip_prefix("PRETTY_NAME=").map(|value| {
                    value
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .to_string()
                })
            })
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

pub(crate) fn read_kernel() -> String {
    std::fs::read_to_string("/proc/sys/kernel/osrelease")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

pub(crate) fn read_uptime() -> u64 {
    std::fs::read_to_string("/proc/uptime")
        .ok()
        .and_then(|text| {
            text.split_whitespace()
                .next()
                .and_then(|value| value.parse::<f64>().ok())
        })
        .map(|value| value as u64)
        .unwrap_or(0)
}

pub(crate) fn read_timezone() -> String {
    if let Some(value) = std::fs::read_to_string("/etc/timezone")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    {
        return value;
    }
    std::fs::read_link("/etc/localtime")
        .ok()
        .and_then(|path| {
            path.to_string_lossy()
                .split("zoneinfo/")
                .nth(1)
                .map(str::to_string)
        })
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

pub(crate) fn read_disks() -> Vec<DiskInfo> {
    let Some(output) = run("df", &["-P", "-k"]) else {
        return Vec::new();
    };
    output
        .lines()
        .skip(1)
        .filter_map(|line| {
            let mut fields = line.split_whitespace();
            let filesystem = fields.next()?.to_string();
            let total_kb: u64 = fields.next()?.parse().ok()?;
            let used_kb: u64 = fields.next()?.parse().ok()?;
            let _available = fields.next()?;
            let percent: u8 = fields.next()?.trim_end_matches('%').parse().ok()?;
            let mount = fields.collect::<Vec<_>>().join(" ");
            Some(DiskInfo {
                filesystem,
                mount,
                total_bytes: total_kb.saturating_mul(1024),
                used_bytes: used_kb.saturating_mul(1024),
                percent,
            })
        })
        .collect()
}

fn service_running(process: &str) -> bool {
    run("pgrep", &["-x", process]).is_some()
}

pub(crate) fn read_services() -> Vec<ServiceInfo> {
    SERVICES
        .iter()
        .map(|(name, unit, process)| ServiceInfo {
            name: (*name).to_string(),
            unit: (*unit).to_string(),
            running: service_running(process),
        })
        .collect()
}

pub(crate) fn log_file_for(service: &str) -> Option<PathBuf> {
    let stack = PathBuf::from(botcore::shared::utils::get_stack_path());
    let mut candidates = Vec::new();
    if service == "botserver" {
        candidates.push(stack.join("logs/stdout.log"));
        candidates.push(stack.join("logs/botserver.log"));
        candidates.push(stack.join("logs/botserver/stdout.log"));
        candidates.push(stack.join("botserver.log"));
        if let Ok(cwd) = std::env::current_dir() {
            candidates.push(cwd.join("botserver.log"));
        }
    } else {
        candidates.push(stack.join("logs").join(service).join("stdout.log"));
        candidates.push(stack.join("logs").join(format!("{service}.log")));
    }
    candidates.into_iter().find(|path| path.is_file())
}

pub(crate) fn tail(path: &Path, lines: usize) -> String {
    let Ok(bytes) = std::fs::read(path) else {
        return String::new();
    };
    let text = String::from_utf8_lossy(&bytes);
    let all: Vec<&str> = text.lines().collect();
    let start = all.len().saturating_sub(lines);
    all[start..].join("\n")
}
