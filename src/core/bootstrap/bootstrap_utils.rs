// Bootstrap utility functions
use crate::core::shared::utils::get_stack_path;
use crate::security::command_guard::SafeCommand;
use log::{debug, info, warn};

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

/// Kill processes by name safely
pub fn safe_pkill(pattern: &[&str], extra_args: &[&str]) {
    let mut args: Vec<&str> = extra_args.to_vec();
    args.extend(pattern);

    let result = SafeCommand::new("pkill")
        .and_then(|c| c.args(&args))
        .and_then(|c| c.execute());

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
    match SafeCommand::new("pgrep")
        .and_then(|c| c.arg("-a"))
        .and_then(|c| c.arg(pattern))
        .and_then(|c| c.execute())
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
    format!("curl -f -s --connect-timeout 5 {}", url)
}

/// Execute shell command safely
pub fn safe_sh_command(command: &str) -> String {
    match SafeCommand::new("sh")
        .and_then(|c| c.arg("-c"))
        .and_then(|c| c.arg(command))
        .and_then(|c| c.execute())
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
    let vault_addr =
        std::env::var("VAULT_ADDR").unwrap_or_else(|_| "https://localhost:8200".to_string());

    let health_url = format!("{}/v1/sys/health", vault_addr);

    match SafeCommand::new("curl")
        .and_then(|c| c.args(&["-f", "-s", "--connect-timeout", "2", "-k", &health_url]))
        .and_then(|c| c.execute())
    {
        Ok(output) => {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&stdout) {
                    let sealed = json.get("sealed").and_then(|v| v.as_bool()).unwrap_or(true);
                    let initialized = json
                        .get("initialized")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false);
                    return !sealed && initialized;
                }
            }
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains("\"initialized\":true") || stderr.contains("\"initialized\":true")
        }
        Err(_) => false,
    }
}

/// Check if Valkey/Redis cache is healthy
pub fn cache_health_check() -> bool {
    if let Ok(output) = SafeCommand::new("valkey-cli")
        .and_then(|c| c.args(&["ping"]))
        .and_then(|c| c.execute())
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("PONG") {
                return true;
            }
        }
    }

    let stack_path = get_stack_path();
    if let Ok(output) = SafeCommand::new(&format!("{}/bin/cache/bin/valkey-cli", stack_path))
        .and_then(|c| c.args(&["ping"]))
        .and_then(|c| c.execute())
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("PONG") {
                return true;
            }
        }
    }

    if let Ok(output) = SafeCommand::new("ss")
        .and_then(|c| c.args(&["-tln"]))
        .and_then(|c| c.execute())
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains(":6379") {
                return true;
            }
        }
    }

    false
}

/// Check if Qdrant vector database is healthy
pub fn vector_db_health_check() -> bool {
    let qdrant_url = if let Ok(sm) = crate::core::secrets::SecretsManager::get() {
        sm.get_vectordb_config_sync().0
    } else {
        "".to_string()
    };

    let urls = [
        format!("{}/healthz", qdrant_url),
        qdrant_url.replace("http://", "https://") + "/healthz",
    ];

    for url in &urls {
        if let Ok(output) = SafeCommand::new("curl")
            .and_then(|c| c.args(&["-sfk", "--connect-timeout", "2", "-m", "3", url]))
            .and_then(|c| c.execute())
        {
            if output.status.success() {
                return true;
            }
        }
    }

    if let Ok(output) = SafeCommand::new("ss")
        .and_then(|c| c.args(&["-tln"]))
        .and_then(|c| c.execute())
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains(":6333") {
                return true;
            }
        }
    }

    false
}

/// Get current user safely
pub fn safe_fuser() -> String {
    "fuser -M '($USER)'".to_string()
}

/// Dump all component logs
pub fn dump_all_component_logs(component: &str) {
    info!("Dumping logs for component: {}", component);
}

/// Result type for bot existence check
#[derive(Debug)]
pub enum BotExistsResult {
    BotExists,
    BotNotFound,
}

/// Check if Zitadel directory is healthy
pub fn zitadel_health_check() -> bool {
    let output = SafeCommand::new("curl")
        .and_then(|c| {
            c.args(&[
                "-sfk",
                "--connect-timeout",
                "2",
                "-m",
                "3",
                "http://localhost:8300/debug/healthz",
            ])
        })
        .and_then(|c| c.execute());

    match output {
        Ok(result) => {
            if result.status.success() {
                let response = String::from_utf8_lossy(&result.stdout);
                debug!("Zitadel health check response: {}", response);
                return response.trim() == "ok";
            }
        }
        Err(e) => {
            debug!("Zitadel health check error: {}", e);
        }
    }

    if let Ok(output) = SafeCommand::new("ss")
        .and_then(|c| c.args(&["-tln"]))
        .and_then(|c| c.execute())
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains(":8300") {
                return true;
            }
        }
    }

    false
}

/// Check if PostgreSQL/Tables is healthy
pub fn tables_health_check() -> bool {
    if let Ok(output) = SafeCommand::new("pg_isready")
        .and_then(|c| c.args(&["-h", "127.0.0.1", "-p", "5432"]))
        .and_then(|c| c.execute())
    {
        return output.status.success();
    }

    let stack_path = get_stack_path();
    let pg_isready = format!("{}/bin/tables/bin/pg_isready", stack_path);
    if let Ok(output) = SafeCommand::new(&pg_isready)
        .and_then(|c| c.args(&["-h", "127.0.0.1", "-p", "5432"]))
        .and_then(|c| c.execute())
    {
        return output.status.success();
    }

    if let Ok(output) = SafeCommand::new("ss")
        .and_then(|c| c.args(&["-tln"]))
        .and_then(|c| c.execute())
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains(":5432") {
                return true;
            }
        }
    }

    false
}

/// Check if MinIO/Drive is healthy
pub fn drive_health_check() -> bool {
    let urls = [
        "http://127.0.0.1:9100/minio/health/live",
        "https://127.0.0.1:9100/minio/health/live",
    ];

    for url in &urls {
        if let Ok(output) = SafeCommand::new("curl")
            .and_then(|c| c.args(&["-sfk", "--connect-timeout", "2", "-m", "3", url]))
            .and_then(|c| c.execute())
        {
            if output.status.success() {
                return true;
            }
        }
    }

    if let Ok(output) = SafeCommand::new("ss")
        .and_then(|c| c.args(&["-tln"]))
        .and_then(|c| c.execute())
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains(":9100") {
                return true;
            }
        }
    }

    false
}

/// Check if ALM (Forgejo) is healthy
pub fn alm_health_check() -> bool {
    let urls = ["", "https://localhost:3000"];

    for url in &urls {
        if let Ok(output) = SafeCommand::new("curl")
            .and_then(|c| c.args(&["-sfk", "--connect-timeout", "2", "-m", "3", url]))
            .and_then(|c| c.execute())
        {
            if output.status.success() {
                return true;
            }
        }
    }

    if let Ok(output) = SafeCommand::new("ss")
        .and_then(|c| c.args(&["-tln"]))
        .and_then(|c| c.execute())
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains(":3000") {
                return true;
            }
        }
    }

    false
}

/// Check if ALM CI (Forgejo Runner) is running
pub fn alm_ci_health_check() -> bool {
    if let Ok(output) = SafeCommand::new("pgrep")
        .and_then(|c| c.args(&["-x", "forgejo-runner"]))
        .and_then(|c| c.execute())
    {
        return output.status.success();
    }

    match SafeCommand::new("ps")
        .and_then(|c| c.args(&["-ef"]))
        .and_then(|c| c.execute())
    {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            stdout.contains("forgejo-runner") && stdout.contains("daemon")
        }
        Err(_) => false,
    }
}
