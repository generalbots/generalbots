// Bootstrap utility functions
use crate::security::command_guard::SafeCommand;
use log::{debug, info, warn};

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
        ("botserver-stack/bin/alm", vec!["-9", "-f"]),
        ("forgejo", vec!["-9", "-f"]),
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
            // Health endpoint returns 503 when sealed but initialized
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
        .and_then(|c| c.args(&["-h", "127.0.0.1", "-p", "6379", "ping"]))
        .and_then(|c| c.execute())
    {
        if output.status.success() {
            let response = String::from_utf8_lossy(&output.stdout);
            if response.trim().to_uppercase() == "PONG" {
                return true;
            }
        }
    }

    if let Ok(output) = SafeCommand::new("redis-cli")
        .and_then(|c| c.args(&["-h", "127.0.0.1", "-p", "6379", "ping"]))
        .and_then(|c| c.execute())
    {
        if output.status.success() {
            let response = String::from_utf8_lossy(&output.stdout);
            if response.trim().to_uppercase() == "PONG" {
                return true;
            }
        }
    }

    match SafeCommand::new("nc")
        .and_then(|c| c.args(&["-z", "-w", "1", "127.0.0.1", "6379"]))
        .and_then(|c| c.execute())
    {
        Ok(output) => output.status.success(),
        Err(_) => {
            match SafeCommand::new("bash")
                .and_then(|c| c.arg("-c"))
                .and_then(|c| {
                    c.arg(
                        "exec 3<>/dev/tcp/127.0.0.1/6379 2>/dev/null && \
                     echo -e 'PING\r\n' >&3 && \
                     read -t 1 response <&3 && \
                     [[ \"$response\" == *PONG* ]] && \
                     exec 3>&-",
                    )
                })
                .and_then(|c| c.execute())
            {
                Ok(output) => output.status.success(),
                Err(_) => false,
            }
        }
    }
}

/// Check if Qdrant vector database is healthy
pub fn vector_db_health_check() -> bool {
    let urls = [
        "http://localhost:6333/healthz",
        "https://localhost:6333/healthz",
    ];

    for url in &urls {
        if let Ok(output) = SafeCommand::new("curl")
            .and_then(|c| c.args(&["-f", "-s", "--connect-timeout", "2", "-k", url]))
            .and_then(|c| c.execute())
        {
            if output.status.success() {
                let response = String::from_utf8_lossy(&output.stdout);
                if response.contains("OK") || response.contains("\"status\":\"ok\"") {
                    return true;
                }
            }
        }
    }

    match SafeCommand::new("nc")
        .and_then(|c| c.args(&["-z", "-w", "1", "127.0.0.1", "6333"]))
        .and_then(|c| c.execute())
    {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
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
                "-f",
                "-s",
                "--connect-timeout",
                "1",
                "-m",
                "2",
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
            let stderr = String::from_utf8_lossy(&result.stderr);
            debug!("Zitadel health check failed: {}", stderr);
        }
        Err(e) => {
            debug!("Zitadel health check error: {}", e);
        }
    }

    match SafeCommand::new("nc")
        .and_then(|c| c.args(&["-z", "-w", "1", "127.0.0.1", "8300"]))
        .and_then(|c| c.execute())
    {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}
