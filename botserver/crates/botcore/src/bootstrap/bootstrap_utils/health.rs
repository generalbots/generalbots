use crate::shared::utils::get_stack_path;
use botlib::security::command_guard::SafeCommand;
use log::debug;

/// Check if vault is healthy
pub fn vault_health_check() -> bool {
    let vault_addr = std::env::var("VAULT_ADDR").unwrap_or_default();
    if vault_addr.is_empty() {
        return false;
    }
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

/// Check if Qdrant vector database is healthy
pub fn vector_db_health_check() -> bool {
    let qdrant_url = if let Ok(sm) = botcoresecrets::SecretsManager::get() {
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

/// Check if Zitadel directory is healthy
pub fn zitadel_health_check() -> bool {
    if let Ok(output) = SafeCommand::new("ss")
        .and_then(|c| c.args(&["-tln"]))
        .and_then(|c| c.execute())
    {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if !stdout.contains(":8300") {
                return false;
            }
        }
    }

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
/// llama.cpp answers /health once the model is loaded (10-20 s on CPU).
pub fn llm_health_check() -> bool {
    SafeCommand::new("curl")
        .and_then(|c| c.args(&["-sf", "--connect-timeout", "2", "-m", "3", "http://127.0.0.1:8081/health"]))
        .and_then(|c| c.execute())
        .map(|output| output.status.success())
        .unwrap_or(false)
}

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
    let urls = ["http://localhost:4747"];

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
    if let Ok(output) = alm_ci_pgrep() {
        return output.status.success();
    }
    false
}

#[cfg(target_os = "windows")]
fn alm_ci_pgrep() -> Result<std::process::Output, anyhow::Error> {
    SafeCommand::new("tasklist")
        .and_then(|c| c.args(&["/FI", "IMAGENAME eq forgejo-runner.exe"]))
        .and_then(|c| c.execute())
        .map_err(|e| anyhow::anyhow!("{}", e))
}

#[cfg(not(target_os = "windows"))]
fn alm_ci_pgrep() -> Result<std::process::Output, anyhow::Error> {
    SafeCommand::new("pgrep")
        .and_then(|c| c.args(&["-x", "forgejo-runner"]))
        .and_then(|c| c.execute())
        .map_err(|e| anyhow::anyhow!("{}", e))
}
