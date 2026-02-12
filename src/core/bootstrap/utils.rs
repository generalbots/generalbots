//! Utility functions for bootstrap module
use crate::security::command_guard::SafeCommand;
use log::{error, info, warn};
use std::fs;
use std::path::Path;

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(crate) struct BotExistsResult {
    #[diesel(sql_type = diesel::sql_types::Bool)]
    pub exists: bool,
}

/// Safe wrapper around pkill command
pub fn safe_pkill(args: &[&str]) {
    if let Ok(cmd) = SafeCommand::new("pkill").and_then(|c| c.args(args)) {
        let _ = cmd.execute();
    }
}

/// Safe wrapper around pgrep command
pub fn safe_pgrep(args: &[&str]) -> Option<std::process::Output> {
    SafeCommand::new("pgrep")
        .and_then(|c| c.args(args))
        .ok()
        .and_then(|cmd| cmd.execute().ok())
}

/// Safe wrapper around shell command execution
pub fn safe_sh_command(script: &str) -> Option<std::process::Output> {
    SafeCommand::new("sh")
        .and_then(|c| c.arg("-c"))
        .and_then(|c| c.trusted_shell_script_arg(script))
        .ok()
        .and_then(|cmd| cmd.execute().ok())
}

/// Safe wrapper around curl command
pub fn safe_curl(args: &[&str]) -> Option<std::process::Output> {
    match SafeCommand::new("curl") {
        Ok(cmd) => match cmd.args(args) {
            Ok(cmd_with_args) => match cmd_with_args.execute() {
                Ok(output) => Some(output),
                Err(e) => {
                    warn!("safe_curl execute failed: {}", e);
                    None
                }
            },
            Err(e) => {
                warn!("safe_curl args failed: {} - args: {:?}", e, args);
                None
            }
        },
        Err(e) => {
            warn!("safe_curl new failed: {}", e);
            None
        }
    }
}

/// Check Vault health status
pub fn vault_health_check() -> bool {
    let client_cert =
        std::path::Path::new("./botserver-stack/conf/system/certificates/botserver/client.crt");
    let client_key =
        std::path::Path::new("./botserver-stack/conf/system/certificates/botserver/client.key");

    let certs_exist = client_cert.exists() && client_key.exists();
    info!("Vault health check: certs_exist={}", certs_exist);

    let result = if certs_exist {
        info!("Using mTLS for Vault health check");
        safe_curl(&[
            "-f",
            "-sk",
            "--connect-timeout",
            "2",
            "-m",
            "5",
            "--cert",
            "./botserver-stack/conf/system/certificates/botserver/client.crt",
            "--key",
            "./botserver-stack/conf/system/certificates/botserver/client.key",
            "https://localhost:8200/v1/sys/health?standbyok=true&uninitcode=200&sealedcode=200",
        ])
    } else {
        info!("Using plain TLS for Vault health check (no client certs yet)");
        safe_curl(&[
            "-f",
            "-sk",
            "--connect-timeout",
            "2",
            "-m",
            "5",
            "https://localhost:8200/v1/sys/health?standbyok=true&uninitcode=200&sealedcode=200",
        ])
    };

    match &result {
        Some(output) => {
            let success = output.status.success();
            info!(
                "Vault health check result: success={}, status={:?}",
                success,
                output.status.code()
            );
            if !success {
                let stderr = String::from_utf8_lossy(&output.stderr);
                let stdout = String::from_utf8_lossy(&output.stdout);
                info!("Vault health check stderr: {}", stderr);
                info!("Vault health check stdout: {}", stdout);
            }
            success
        }
        None => {
            info!("Vault health check: safe_curl returned None");
            false
        }
    }
}

/// Safe wrapper around fuser command
pub fn safe_fuser(args: &[&str]) {
    if let Ok(cmd) = SafeCommand::new("fuser").and_then(|c| c.args(args)) {
        let _ = cmd.execute();
    }
}

/// Dump all component logs to error output
pub fn dump_all_component_logs(log_dir: &Path) {
    if !log_dir.exists() {
        error!("Log directory does not exist: {}", log_dir.display());
        return;
    }

    error!("========================================================================");
    error!("DUMPING ALL AVAILABLE LOGS FROM: {}", log_dir.display());
    error!("========================================================================");

    let components = vec![
        "vault", "tables", "drive", "cache", "directory", "llm",
        "vector_db", "email", "proxy", "dns", "meeting"
    ];

    for component in components {
        let component_log_dir = log_dir.join(component);
        if !component_log_dir.exists() {
            continue;
        }

        let log_files = vec!["stdout.log", "stderr.log", "postgres.log", "vault.log", "minio.log"];

        for log_file in log_files {
            let log_path = component_log_dir.join(log_file);
            if log_path.exists() {
                error!("-------------------- {} ({}) --------------------", component, log_file);
                match fs::read_to_string(&log_path) {
                    Ok(content) => {
                        let lines: Vec<&str> = content.lines().rev().take(30).collect();
                        for line in lines.iter().rev() {
                            error!("  {}", line);
                        }
                    }
                    Err(e) => {
                        error!("  Failed to read: {}", e);
                    }
                }
            }
        }
    }

    error!("========================================================================");
    error!("END OF LOG DUMP");
    error!("========================================================================");
}
