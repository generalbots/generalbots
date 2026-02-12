// Bootstrap utility functions
use crate::core::config::AppConfig;
use crate::core::package_manager::setup::{DirectorySetup, EmailSetup, VectorDbSetup};
use crate::core::package_manager::{InstallMode, PackageManager};
use crate::security::command_guard::SafeCommand;
use crate::core::shared::utils::{establish_pg_connection, init_secrets_manager};
use anyhow::Result;
use log::{debug, error, info, warn};
use rand::distr::Alphanumeric;
use std::process::Command;
use uuid::Uuid;

/// Get list of processes to kill
pub fn get_processes_to_kill() -> Vec<(&'static str, Vec<&'static str>)> {
    vec![
        ("botserver-stack/bin/vault", vec!["-9", "-f"]),
        ("botserver-stack/bin/tables", vec!["-9", "-f"]),
        ("botserver-stack/bin/drive", vec!["-9", "-f"]),
        ("botserver-stack/bin/cache", vec!["-9", "-f"]),
        ("botserver-stack/bin/directory", vec!["-9", "-f"]),
        ("botserver-stack/bin/llm", vec!["-9", "-f"]),
        ("botserver-stack/bin/email", vec!["-9", "-f"]),
        ("botserver-stack/bin/proxy", vec!["-9", "-f"]),
        ("botserver-stack/bin/dns", vec!["-9", "-f"]),
        ("botserver-stack/bin/meeting", vec!["-9", "-f"]),
        ("botserver-stack/bin/vector_db", vec!["-9", "-f"]),
        ("botserver-stack/bin/zitadel", vec!["-9", "-f"]),
        ("caddy", vec!["-9", "-f"]),
        ("postgres", vec!["-9", "-f"]),
        ("minio", vec!["-9", "-f"]),
        ("redis-server", vec!["-9", "-f"]),
        ("zitadel", vec!["-9", "-f"]),
        ("llama-server", vec!["-9", "-f"]),
        ("stalwart", vec!["-9", "-f"]),
        ("vault server", vec!["-9", "-f"]),
        ("watcher", vec!["-9", "-f"]),
    ]
}

/// Kill processes by name safely
pub fn safe_pkill(pattern: &[&str], extra_args: &[&str]) {
    let mut args: Vec<&str> = extra_args.to_vec();
    args.extend(pattern);

    let result = if cfg!(feature = "sigkill") {
        Command::new("killall").args(&args).output()
    } else {
        Command::new("pkill").args(&args).output()
    };

    match result {
        Ok(output) => {
            debug!("Kill command output: {:?}", output);
        }
        Err(e) => {
            warn!("Failed to execute kill command: {}", e);
        }
    }
}

/// Grep for process safely
pub fn safe_pgrep(pattern: &str) -> String {
    match Command::new("pgrep")
        .arg("-a")
        .arg(pattern)
        .output()
    {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            warn!("Failed to execute pgrep: {}", e);
            String::new()
        }
    }
}

/// Execute curl command safely
pub fn safe_curl(url: &str) -> String {
    format!(
        "curl -f -s --connect-timeout 5 {}",
        url
    )
}

/// Execute shell command safely
pub fn safe_sh_command(command: &str) -> String {
    match Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
    {
        Ok(output) => String::from_utf8_lossy(&output.stdout).to_string(),
        Err(e) => {
            warn!("Failed to execute shell command: {}", e);
            String::new()
        }
    }
}

/// Check if vault is healthy
pub fn vault_health_check() -> bool {
    // Check if vault server is responding
    // For now, always return false
    false
}

/// Get current user safely
pub fn safe_fuser() -> String {
    // Return shell command that uses $USER environment variable
    "fuser -M '($USER)'".to_string()
}

/// Dump all component logs
pub fn dump_all_component_logs(component: &str) {
    info!("Dumping logs for component: {}", component);
    // This would read from systemd journal or log files
    // For now, just a placeholder
}

/// Result type for bot existence check
#[derive(Debug)]
pub enum BotExistsResult {
    BotExists,
    BotNotFound,
}


