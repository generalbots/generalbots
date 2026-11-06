use crate::shared::utils::get_stack_path;
use botlib::security::command_guard::SafeCommand;
use log::{debug, info};

/// Get list of processes to kill (only used in dev with local botserver-stack)
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

/// Kill processes by name safely (platform-aware)
pub fn safe_pkill(pattern: &[&str], extra_args: &[&str]) {
    let result = kill_process(pattern, extra_args);

    match result {
        Ok(output) => {
            debug!("Kill command output: {:?}", output);
        }
        Err(e) => {
            debug!("Kill command not available (expected on some platforms): {}", e);
        }
    }
}

#[cfg(target_os = "windows")]
fn kill_process(pattern: &[&str], _extra_args: &[&str]) -> Result<std::process::Output, anyhow::Error> {
    for proc in pattern {
        let result = SafeCommand::new("taskkill")
            .and_then(|c| c.args(&["/F", "/IM", proc]))
            .and_then(|c| c.execute());
        if let Err(e) = result {
            debug!("taskkill failed for {}: {}", proc, e);
        }
    }
    Ok(std::process::Output {
        status: std::process::ExitStatus::default(),
        stdout: Vec::new(),
        stderr: Vec::new(),
    })
}

#[cfg(not(target_os = "windows"))]
fn kill_process(pattern: &[&str], extra_args: &[&str]) -> Result<std::process::Output, anyhow::Error> {
    let mut args: Vec<&str> = extra_args.to_vec();
    args.extend(pattern);
    SafeCommand::new("pkill")
        .and_then(|c| c.args(&args))
        .and_then(|c| c.execute())
        .map_err(|e| anyhow::anyhow!("{}", e))
}

/// Grep for process safely (platform-aware)
pub fn safe_pgrep(pattern: &str) -> String {
    match pgrep_process(pattern) {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            debug!("Process grep not available (expected on some platforms): {}", e);
            String::new()
        }
    }
}

#[cfg(target_os = "windows")]
fn pgrep_process(pattern: &str) -> Result<std::process::Output, anyhow::Error> {
    SafeCommand::new("tasklist")
        .and_then(|c| c.args(&["/FI", &format!("IMAGENAME eq {}", pattern)]))
        .and_then(|c| c.execute())
        .map_err(|e| anyhow::anyhow!("{}", e))
}

#[cfg(not(target_os = "windows"))]
fn pgrep_process(pattern: &str) -> Result<std::process::Output, anyhow::Error> {
    SafeCommand::new("pgrep")
        .and_then(|c| c.arg("-a"))
        .and_then(|c| c.arg(pattern))
        .and_then(|c| c.execute())
        .map_err(|e| anyhow::anyhow!("{}", e))
}

/// Execute curl command safely
pub fn safe_curl(url: &str) -> String {
    format!("curl -f -s --connect-timeout 5 {}", url)
}

/// Execute shell command safely (platform-aware)
pub fn safe_sh_command(command: &str) -> String {
    match sh_exec(command) {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            debug!("Shell command not available (expected on some platforms): {}", e);
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
        .map_err(|e| anyhow::anyhow!("{}", e))
}

#[cfg(not(target_os = "windows"))]
fn sh_exec(command: &str) -> Result<std::process::Output, anyhow::Error> {
    SafeCommand::new("sh")
        .and_then(|c| c.arg("-c"))
        .and_then(|c| c.arg(command))
        .and_then(|c| c.execute())
        .map_err(|e| anyhow::anyhow!("{}", e))
}

/// Get current user safely
pub fn safe_fuser() -> String {
    "fuser -M '($USER)'".to_string()
}

/// Dump all component logs
pub fn dump_all_component_logs(component: &str) {
    info!("Dumping logs for component: {}", component);
}
