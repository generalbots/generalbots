// Bootstrap utility functions
use log::{debug, info, warn};
use std::process::Command;

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

    let result = Command::new("pkill").args(&args).output();

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

/// Check if Valkey/Redis cache is healthy
pub fn cache_health_check() -> bool {
    // Try to PING the cache server
    match Command::new("redis-cli")
        .args(["-h", "127.0.0.1", "-p", "6379", "ping"])
        .output()
    {
        Ok(output) => {
            if output.status.success() {
                let response = String::from_utf8_lossy(&output.stdout);
                response.trim().to_uppercase() == "PONG"
            } else {
                false
            }
        }
        Err(_) => {
            // If redis-cli is not available, try TCP connection
            match Command::new("sh")
                .arg("-c")
                .arg("timeout 1 bash -c '</dev/tcp/127.0.0.1/6379' 2>/dev/null")
                .output()
            {
                Ok(output) => output.status.success(),
                Err(_) => false,
            }
        }
    }
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


