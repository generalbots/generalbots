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

/// One log file the panel can surface (#1874, #2102). `path` is always the
/// resolved-or-expected location so the panel can show where the file will
/// appear; `exists=false` means it has not been created yet (or the host logs
/// to journald instead of a file).
#[derive(Serialize)]
pub(crate) struct LogFileInfo {
    pub(crate) service: String,
    pub(crate) path: String,
    pub(crate) exists: bool,
    pub(crate) size_bytes: u64,
    /// `file` when the path exists, `journal` when the service logs to the
    /// systemd journal only, `missing` when nothing can be read yet.
    pub(crate) source: &'static str,
}

/// #2102 — the desktop sidecar redirects both streams here
/// (`botapp/src/desktop/sidecar.rs`), so on a packaged install this is the
/// only log file that ever exists for `botserver`.
const SIDECAR_LOG: &str = "logs/botserver-sidecar.log";

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

/// Candidate log files for a service, in priority order. Pure so it can be
/// unit-tested against a temporary stack directory (#2102).
pub(crate) fn log_file_candidates_in(stack: &Path, service: &str) -> Vec<PathBuf> {
    if service != "botserver" {
        return vec![
            stack.join("logs").join(service).join("stdout.log"),
            stack.join("logs").join(format!("{service}.log")),
        ];
    }
    vec![
        // #2102 — desktop sidecar output first: it is the only file a
        // packaged install writes.
        stack.join(SIDECAR_LOG),
        stack.join("logs/stdout.log"),
        stack.join("logs/botserver.log"),
        stack.join("logs/botserver/stdout.log"),
        // Future rotation target, if the sidecar ever starts rotating.
        stack.join("logs/botserver/current.log"),
        stack.join("botserver.log"),
    ]
}

/// Where `botserver` is expected to write on this deployment: the sidecar file
/// in shell mode (desktop), the redirected stdout file otherwise.
pub(crate) fn expected_log_path(service: &str) -> PathBuf {
    let stack = PathBuf::from(botcore::shared::utils::get_stack_path());
    if service != "botserver" {
        return log_file_candidates_in(&stack, service)
            .into_iter()
            .next()
            .unwrap_or_else(|| stack.join("logs").join(format!("{service}.log")));
    }
    if botcore::product::is_shell_mode() {
        return stack.join(SIDECAR_LOG);
    }
    stack.join("logs/stdout.log")
}

pub(crate) fn log_file_in(stack: &Path, service: &str) -> Option<PathBuf> {
    log_file_candidates_in(stack, service)
        .into_iter()
        .find(|path| path.is_file())
}

pub(crate) fn log_file_for(service: &str) -> Option<PathBuf> {
    let stack = PathBuf::from(botcore::shared::utils::get_stack_path());
    if let Some(path) = log_file_in(&stack, service) {
        return Some(path);
    }
    // Development fallback: `cargo run` from the repo root leaves the log next
    // to the working directory.
    if service == "botserver" {
        if let Ok(cwd) = std::env::current_dir() {
            let fallback = cwd.join("botserver.log");
            if fallback.is_file() {
                return Some(fallback);
            }
        }
    }
    None
}

/// Every known service log with its on-disk path and size (#1874, #2102).
/// Missing files still report the expected path so the panel can say where the
/// log will appear instead of showing an empty row.
pub(crate) fn log_file_infos() -> Vec<LogFileInfo> {
    let mut services: Vec<&str> = vec!["botserver"];
    for (name, _unit, _process) in SERVICES {
        if !services.contains(name) {
            services.push(name);
        }
    }
    let journal_ready = journal_available();
    services
        .into_iter()
        .map(|service| match log_file_for(service) {
            Some(path) => LogFileInfo {
                service: service.to_string(),
                path: path.to_string_lossy().to_string(),
                exists: true,
                size_bytes: std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0),
                source: "file",
            },
            None => {
                let expected = expected_log_path(service);
                let journal = journal_ready && journal_unit(service).is_some();
                LogFileInfo {
                    service: service.to_string(),
                    path: expected.to_string_lossy().to_string(),
                    exists: false,
                    size_bytes: 0,
                    source: if journal { "journal" } else { "missing" },
                }
            }
        })
        .collect()
}

/// systemd unit name for an allow-listed service (`botserver` is always
/// allowed even though it also appears in [`SERVICES`]).
pub(crate) fn journal_unit(service: &str) -> Option<&'static str> {
    SERVICES
        .iter()
        .find(|(name, _unit, _process)| *name == service)
        .map(|(_name, unit, _process)| *unit)
}

/// Whether `journalctl` can be executed on this host at all.
pub(crate) fn journal_available() -> bool {
    cfg!(target_os = "linux") && run("which", &["journalctl"]).is_some()
}

/// #2102 — production runs botserver under systemd and never writes a file, so
/// the panel falls back to the journal. The script is built only from the
/// allow-listed unit name (validated charset) and a clamped line count; the
/// caller's input never reaches the shell.
pub(crate) fn journal_tail(service: &str, lines: usize) -> Option<String> {
    let unit = journal_unit(service)?;
    if !unit
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '@'))
    {
        return None;
    }
    let script = format!("journalctl -u {unit} -n {lines} --no-pager");
    let output = SafeCommand::new("sh")
        .and_then(|command| command.arg("-c"))
        .and_then(|command| command.trusted_shell_script_arg(&script))
        .and_then(|command| command.execute())
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if text.is_empty() {
        return None;
    }
    Some(text)
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

#[cfg(test)]
mod tests {
    use super::*;

    /// #2102 — a temporary stack root that is removed again by the caller.
    fn temp_stack(label: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("gb-os-data-{label}-{}", std::process::id()));
        assert!(std::fs::remove_dir_all(&dir).is_ok() || !dir.exists());
        assert!(std::fs::create_dir_all(dir.join("logs")).is_ok());
        dir
    }

    #[test]
    fn sidecar_log_is_the_first_candidate() {
        let stack = temp_stack("candidates");
        let candidates = log_file_candidates_in(&stack, "botserver");
        assert_eq!(candidates.first(), Some(&stack.join(SIDECAR_LOG)));
        assert!(candidates.contains(&stack.join("logs/stdout.log")));
        assert!(std::fs::remove_dir_all(&stack).is_ok());
    }

    /// The desktop sidecar file must be found even when no other log exists.
    #[test]
    fn resolves_the_desktop_sidecar_log() {
        let stack = temp_stack("sidecar");
        let path = stack.join(SIDECAR_LOG);
        assert!(std::fs::write(&path, "hello\n").is_ok());
        assert_eq!(log_file_in(&stack, "botserver"), Some(path.clone()));
        assert_eq!(tail(&path, 10), "hello");
        assert!(std::fs::remove_dir_all(&stack).is_ok());
    }

    /// A stack with no log at all resolves to nothing, and the expected path
    /// still points at a concrete location the panel can display.
    #[test]
    fn missing_log_reports_the_expected_path() {
        let stack = temp_stack("missing");
        assert_eq!(log_file_in(&stack, "botserver"), None);
        let expected = log_file_candidates_in(&stack, "botserver")
            .into_iter()
            .next()
            .unwrap_or_default();
        assert!(expected.ends_with("botserver-sidecar.log"));
        assert!(!expected.is_file());
        assert!(std::fs::remove_dir_all(&stack).is_ok());
    }

    /// The journal script only ever carries an allow-listed unit name.
    #[test]
    fn journal_unit_is_allow_listed() {
        assert_eq!(journal_unit("botserver"), Some("botserver"));
        assert_eq!(journal_unit("postgresql"), Some("postgresql"));
        assert_eq!(journal_unit("../../etc/passwd"), None);
    }
}
