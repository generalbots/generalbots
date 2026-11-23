use anyhow::Result;
use botcoresecrets::SecretsManager;

pub fn get_database_url_sync() -> Result<String> {
    let manager = SecretsManager::get_clone()?;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let result = match rt {
            Ok(rt) => rt.block_on(manager.get_database_url()),
            Err(e) => Err(anyhow::anyhow!("Failed to create runtime: {}", e)),
        };
        let _ = tx.send(result);
    });
    rx.recv_timeout(std::time::Duration::from_secs(10))
        .map_err(|e| anyhow::anyhow!("Database credential lookup did not finish: {e}"))?
}

/// Read one field of a Vault secret synchronously (used by the installers,
/// which are sync command pipelines). Empty/missing values are returned as
/// `Err` so callers can fall back to an explicitly generated value.
pub fn get_secret_value_sync(path: &str, key: &str) -> Result<String> {
    let manager = SecretsManager::get_clone()?;
    let path = path.to_string();
    let key = key.to_string();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let result = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(rt) => rt.block_on(manager.get_value(&path, &key)),
            Err(error) => Err(anyhow::anyhow!("Failed to create runtime: {error}")),
        };
        let _ = tx.send(result);
    });
    let value = rx
        .recv_timeout(std::time::Duration::from_secs(10))
        .map_err(|error| anyhow::anyhow!("Vault secret lookup did not finish: {error}"))??;
    if value.is_empty() {
        Err(anyhow::anyhow!("Vault field {path}::{key} is empty"))
    } else {
        Ok(value)
    }
}

pub fn parse_database_url(url: &str) -> (String, String, String, u32, String) {
    if let Some(stripped) = url.strip_prefix("postgres://") {
        let parts: Vec<&str> = stripped.split('@').collect();
        if parts.len() == 2 {
            let user_pass: Vec<&str> = parts[0].split(':').collect();
            let host_db: Vec<&str> = parts[1].split('/').collect();
            if user_pass.len() >= 2 && host_db.len() >= 2 {
                let username = user_pass[0].to_string();
                let password = user_pass[1].to_string();
                let host_port: Vec<&str> = host_db[0].split(':').collect();
                let server = host_port[0].to_string();
                let port = host_port
                    .get(1)
                    .and_then(|p| p.parse().ok())
                    .unwrap_or(5432);
                let database = host_db[1].to_string();
                return (username, password, server, port, database);
            }
        }
    }
    (
        "".to_string(),
        "".to_string(),
        "".to_string(),
        5432,
        "".to_string(),
    )
}
