//! cache - extracted from bootstrap.rs

use log::{info, warn};
use std::sync::Arc;


pub async fn init_redis() -> Option<Arc<redis::Client>> {
    use crate::core::secrets::{SecretPaths, SecretsManager};

    // Try environment variables first
    let env_url = std::env::var("CACHE_URL")
        .or_else(|_| std::env::var("REDIS_URL"))
        .or_else(|_| std::env::var("VALKEY_URL"))
        .ok();

    // Check for Redis database number for namespace isolation
    let redis_db = std::env::var("REDIS_DB").ok()
        .and_then(|s| s.parse::<u8>().ok())
        .unwrap_or(0);

    // Build candidate URLs: try env first, then Vault with password if available
    let mut urls: Vec<String> = Vec::new();
    if let Some(url) = env_url.clone() {
        let url = if redis_db > 0 {
            let base = url.trim_end_matches('/');
            format!("{base}/{redis_db}")
        } else {
            url
        };
        urls.push(url);
    }
    if let Ok(secrets) = SecretsManager::get() {
        if let Ok(data) = secrets.get_secret(SecretPaths::CACHE).await {
            let host = data.get("host").cloned().unwrap_or_else(|| "".into());
            let port = data.get("port").and_then(|p| p.parse().ok()).unwrap_or(6379);
            if let Some(url_val) = data.get("url").filter(|u| !u.is_empty() && u.contains("://")) {
                urls.push(url_val.clone());
                info!("Cache: using Vault URL with credentials");
            } else {
                if env_url.is_none() {
                    urls.push(format!("redis://{}:{}", host, port));
                }
                if let Some(pass) = data.get("password").filter(|p| !p.is_empty()) {
                    urls.push(format!("redis://:{}@{}:{}", pass, host, port));
                    info!("Cache: built URL with Vault password");
                }
            }
        } else if env_url.is_none() {
            urls.push(String::new());
        }
    } else if env_url.is_none() {
        urls.push("redis://localhost:6379".to_string());
    }

    for u in &urls {
        let masked = u.split('@').next_back().unwrap_or(u);
        info!("Cache URL candidate: {}", masked);
    }
    info!("Attempting to connect to cache, trying {} URL(s)", urls.len());

    let max_attempts = 12;
    let mut attempt = 0;
    let mut url_index = 0;

    loop {
        attempt += 1;
        let cache_url = urls[url_index % urls.len()].clone();

        let result = tokio::task::spawn_blocking(move || {
            match redis::Client::open(cache_url.as_str()) {
                Ok(client) => {
                    let timeout = std::time::Duration::from_secs(3);
                    match client.get_connection_with_timeout(timeout) {
                        Ok(mut conn) => {
                            match redis::cmd("PING").query::<String>(&mut conn) {
                                Ok(response) if response == "PONG" => {
                                    log::info!("Cache initialized - Valkey connected via {}", cache_url.split('@').next_back().unwrap_or(&cache_url));
                                    Ok(Some(Arc::new(client)))
                                }
                                Ok(response) => {
                                    log::info!("Cache initialized - Valkey connected via {} (PING: {})", cache_url.split('@').next_back().unwrap_or(&cache_url), response);
                                    Ok(Some(Arc::new(client)))
                                }
                                Err(e) => {
                                    Err(format!("Cache PING failed: {}", e))
                                }
                            }
                        }
                        Err(e) => {
                            Err(format!("Failed to establish cache connection: {}", e))
                        }
                    }
                }
                Err(e) => {
                    Err(format!("Failed to create cache client: {}", e))
                }
            }
        })
        .await;

        match result {
            Ok(Ok(Some(client))) => return Some(client),
            Ok(Ok(None)) => return None,
            Ok(Err(e)) => {
                if url_index + 1 < urls.len() {
                    url_index += 1;
                    info!("Trying next cache URL...");
                    continue;
                }
                if attempt < max_attempts {
                    info!("Cache connection attempt {}/{} failed: {}. Retrying in 5s...", attempt, max_attempts, e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                } else {
                    warn!("Cache connection failed after {} attempts: {}. Cache functions will be disabled.", max_attempts, e);
                    return None;
                }
            }
            Err(e) => {
                if attempt < max_attempts {
                    info!("Cache connection attempt {}/{} failed with task error: {}. Retrying in 5s...", attempt, max_attempts, e);
                    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                    continue;
                } else {
                    warn!("Cache connection failed after {} attempts with task error: {}. Cache functions will be disabled.", max_attempts, e);
                    return None;
                }
            }
        }
    }
}
