use crate::shared::utils::get_stack_path;
use botlib::security::command_guard::SafeCommand;
use log::{debug, info, warn};

/// Get list of processes to kill (only used in dev with local botserver-stack).
///
/// On Unix the patterns are stack-scoped paths passed to `pkill -f`, so only
/// processes started from inside the stack are matched. On Windows this list
/// is unused — `kill_stack_processes_windows()` replaces the whole flow.
pub fn get_processes_to_kill() -> Vec<(String, Vec<&'static str>)> {
    let stack = get_stack_path();
    vec![
        (format!("{}/bin/vault", stack), vec!["-9", "-f"]),
        (format!("{}/bin/tables", stack), vec!["-9", "-f"]),
        (format!("{}/bin/drive", stack), vec!["-9", "-f"]),
        (format!("{}/bin/cache", stack), vec!["-9", "-f"]),
        (format!("{}/bin/directory", stack), vec!["-9", "-f"]),
        (format!("{}/bin/llm", stack), vec!["-9", "-f"]),
        (format!("{}/bin/email", stack), vec!["-9", "-f"]),
        (format!("{}/bin/proxy", stack), vec!["-9", "-f"]),
        (format!("{}/bin/dns", stack), vec!["-9", "-f"]),
        (format!("{}/bin/meeting", stack), vec!["-9", "-f"]),
        (format!("{}/bin/vector_db", stack), vec!["-9", "-f"]),
        (format!("{}/bin/zitadel", stack), vec!["-9", "-f"]),
        (format!("{}/bin/alm", stack), vec!["-9", "-f"]),
        ("forgejo".to_string(), vec!["-9", "-f"]),
        ("caddy".to_string(), vec!["-9", "-f"]),
        ("postgres".to_string(), vec!["-9", "-f"]),
        ("minio".to_string(), vec!["-9", "-f"]),
        ("redis-server".to_string(), vec!["-9", "-f"]),
        ("zitadel".to_string(), vec!["-9", "-f"]),
        ("llama-server".to_string(), vec!["-9", "-f"]),
        ("stalwart".to_string(), vec!["-9", "-f"]),
        ("vault server".to_string(), vec!["-9", "-f"]),
        ("watcher".to_string(), vec!["-9", "-f"]),
    ]
}

/// Process patterns that belong to a component, per platform.
///
/// Used to clear a process that exists but no longer serves its port: the
/// component's own process check then reports "already running", the start is
/// skipped, and the readiness probe waits out its whole window (#2022). On Unix
/// the pattern is the stack path, so a cache installed elsewhere on the machine
/// is never touched; on Windows the stack-scoped sysinfo kill is used instead.
pub fn component_process_patterns(component: &str) -> Vec<String> {
    if component != "cache" {
        return Vec::new();
    }
    if cfg!(target_os = "windows") {
        vec![
            "redis-server.exe".to_string(),
            "valkey-server.exe".to_string(),
        ]
    } else {
        vec![format!("{}/bin/cache", get_stack_path())]
    }
}

/// Kills every process of a component whose name is known. Returns whether any
/// name was handled, so the caller can log an accurate recovery message.
pub fn kill_component_processes(component: &str) -> bool {
    let patterns = component_process_patterns(component);
    if patterns.is_empty() {
        return false;
    }

    #[cfg(windows)]
    {
        return kill_stack_component_windows(component) > 0;
    }

    #[cfg(not(windows))]
    {
        for pattern in &patterns {
            safe_pkill(&[pattern.as_str()], &["-9", "-f"]);
        }
        true
    }
}

/// Kill processes by name safely (platform-aware).
/// Logs a warning when the kill is rejected or fails so the caller knows a
/// component could not be stopped.
pub fn safe_pkill(pattern: &[&str], extra_args: &[&str]) {
    let result = kill_process(pattern, extra_args);

    match result {
        Ok(output) => {
            debug!("Kill command output: {:?}", output);
        }
        Err(e) => {
            warn!("Kill command failed for {:?}: {}", pattern, e);
        }
    }
}

#[cfg(target_os = "windows")]
fn kill_process(
    pattern: &[&str],
    _extra_args: &[&str],
) -> Result<std::process::Output, anyhow::Error> {
    let mut last_output = None;
    for image in pattern {
        let result = SafeCommand::new("taskkill")
            .and_then(|c| c.args(&["/F", "/T", "/IM", image]))
            .and_then(|c| c.execute());
        match result {
            Ok(output) => {
                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    if !stderr.contains("not found") {
                        warn!("taskkill /IM {image} exited {}: {stderr}", output.status);
                    }
                }
                last_output = Some(output);
            }
            Err(e) => {
                warn!("taskkill rejected for {image}: {e}");
                return Err(anyhow::anyhow!("{e}"));
            }
        }
    }
    last_output.ok_or_else(|| anyhow::anyhow!("no patterns to kill"))
}

#[cfg(not(target_os = "windows"))]
fn kill_process(
    pattern: &[&str],
    extra_args: &[&str],
) -> Result<std::process::Output, anyhow::Error> {
    let mut args: Vec<&str> = extra_args.to_vec();
    args.extend(pattern);
    SafeCommand::new("pkill")
        .and_then(|c| c.args(&args))
        .and_then(|c| c.execute())
        .map_err(|e| anyhow::anyhow!("{e}"))
}

// ── Windows stack-scoped process kill via sysinfo ───────────────────────

/// Resolve `<stack>/bin/` as an absolute, normalised, lower-case path for
/// case-insensitive prefix matching on Windows.
#[cfg(windows)]
fn resolve_stack_bin_dir() -> String {
    let raw = format!("{}/bin/", get_stack_path());
    let path = std::path::Path::new(&raw);
    let abs = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join(path)
    };
    abs.to_string_lossy()
        .to_ascii_lowercase()
        .replace('/', "\\")
}

/// Kill every running process whose executable lives under `stack_bin`.
/// Returns the number of processes killed.
#[cfg(windows)]
pub fn kill_stack_processes_windows() -> usize {
    let stack_bin = resolve_stack_bin_dir();
    kill_stack_scoped(&stack_bin)
}

/// Enumerate processes and kill those whose exe path starts with `stack_bin`
/// (case-insensitive, backslash-normalised).
#[cfg(windows)]
fn kill_stack_scoped(stack_bin: &str) -> usize {
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::Always),
    );

    let mut killed = 0usize;
    for (pid, proc) in sys.processes() {
        let exe = match proc.exe() {
            Some(p) => p.to_string_lossy().to_ascii_lowercase().replace('/', "\\"),
            None => continue,
        };
        if !exe.starts_with(stack_bin) {
            continue;
        }
        let name = proc.name().to_string_lossy();
        if proc.kill() {
            info!("Killed stack process {name} (PID {pid})");
            killed += 1;
        } else {
            warn!("Could not kill stack process {name} (PID {pid})");
        }
    }
    if killed == 0 {
        debug!("No stack processes found under {stack_bin}");
    }
    killed
}

/// Kill the processes of one stack component, identified by its directory
/// under `<stack>\bin\` (`cache`, `llm`, ...). A process with the same image
/// name that was started from anywhere else on the machine is never touched.
/// Returns the number of processes killed.
#[cfg(windows)]
pub fn kill_stack_component_windows(component_dir: &str) -> usize {
    let stack_bin = resolve_stack_bin_dir();
    let fragment = format!("{}{component_dir}{}", std::path::MAIN_SEPARATOR, std::path::MAIN_SEPARATOR);
    kill_stack_scoped_by_subdir(&stack_bin, &fragment)
}

/// Kill processes whose exe is under `stack_bin` AND whose path also contains
/// `subdir_fragment` (e.g. `\cache\`). Used for single-component scoped kill.
#[cfg(windows)]
fn kill_stack_scoped_by_subdir(stack_bin: &str, subdir_fragment: &str) -> usize {
    use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};

    let fragment_lower = subdir_fragment.to_ascii_lowercase().replace('/', "\\");
    let mut sys = System::new();
    sys.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_exe(UpdateKind::Always),
    );

    let mut killed = 0usize;
    for (pid, proc) in sys.processes() {
        let exe = match proc.exe() {
            Some(p) => p.to_string_lossy().to_ascii_lowercase().replace('/', "\\"),
            None => continue,
        };
        if !exe.starts_with(stack_bin) || !exe.contains(&fragment_lower) {
            continue;
        }
        let name = proc.name().to_string_lossy();
        if proc.kill() {
            info!("Killed component process {name} (PID {pid})");
            killed += 1;
        } else {
            warn!("Could not kill component process {name} (PID {pid})");
        }
    }
    killed
}

// ── Process grep / shell helpers ────────────────────────────────────────

/// Grep for process safely (platform-aware)
pub fn safe_pgrep(pattern: &str) -> String {
    match pgrep_process(pattern) {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            debug!("Process grep not available (expected on some platforms): {e}");
            String::new()
        }
    }
}

#[cfg(target_os = "windows")]
fn pgrep_process(pattern: &str) -> Result<std::process::Output, anyhow::Error> {
    SafeCommand::new("tasklist")
        .and_then(|c| c.args(&["/FI", &format!("IMAGENAME eq {pattern}")]))
        .and_then(|c| c.execute())
        .map_err(|e| anyhow::anyhow!("{e}"))
}

#[cfg(not(target_os = "windows"))]
fn pgrep_process(pattern: &str) -> Result<std::process::Output, anyhow::Error> {
    SafeCommand::new("pgrep")
        .and_then(|c| c.arg("-a"))
        .and_then(|c| c.arg(pattern))
        .and_then(|c| c.execute())
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Execute shell command safely (platform-aware)
pub fn safe_sh_command(command: &str) -> String {
    match sh_exec(command) {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            debug!("Shell command not available (expected on some platforms): {e}");
            String::new()
        }
    }
}

#[cfg(target_os = "windows")]
fn sh_exec(command: &str) -> Result<std::process::Output, anyhow::Error> {
    SafeCommand::new("cmd")
        .and_then(|c| c.arg("/C"))
        .and_then(|c| c.arg(command))
        .and_then(|c| c.execute())
        .map_err(|e| anyhow::anyhow!("{e}"))
}

#[cfg(not(target_os = "windows"))]
fn sh_exec(command: &str) -> Result<std::process::Output, anyhow::Error> {
    SafeCommand::new("sh")
        .and_then(|c| c.arg("-c"))
        .and_then(|c| c.arg(command))
        .and_then(|c| c.execute())
        .map_err(|e| anyhow::anyhow!("{e}"))
}

/// Dump all component logs
pub fn dump_all_component_logs(component: &str) {
    info!("Dumping logs for component: {component}");
}
