
mod browser_service;
mod minio;
mod postgres;
mod redis;

pub use browser_service::{BrowserService, DEFAULT_DEBUG_PORT};
pub use minio::MinioService;
pub use postgres::PostgresService;
pub use redis::RedisService;

use anyhow::Result;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;

pub const HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(30);

pub const HEALTH_CHECK_INTERVAL: Duration = Duration::from_millis(100);

pub async fn wait_for<F, Fut>(timeout: Duration, interval: Duration, mut check: F) -> Result<()>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = bool>,
{
    let start = std::time::Instant::now();
    while start.elapsed() < timeout {
        if check().await {
            return Ok(());
        }
        sleep(interval).await;
    }
    anyhow::bail!("Timeout waiting for condition")
}

pub async fn check_tcp_port(host: &str, port: u16) -> bool {
    tokio::net::TcpStream::connect((host, port)).await.is_ok()
}

pub fn ensure_dir(path: &Path) -> Result<()> {
    if !path.exists() {
        std::fs::create_dir_all(path)?;
    }
    Ok(())
}

#[async_trait::async_trait]
pub trait Service: Send + Sync {
    async fn start(&mut self) -> Result<()>;

    async fn stop(&mut self) -> Result<()>;

    async fn health_check(&self) -> Result<bool>;

    fn connection_url(&self) -> String;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServiceStatus {
    Stopped,
    Starting,
    Running,
    Stopping,
    Failed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_wait_for_success() {
        let mut counter = 0;
        let result = wait_for(Duration::from_secs(1), Duration::from_millis(10), || {
            counter += 1;
            async move { counter >= 3 }
        })
        .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_wait_for_timeout() {
        let result = wait_for(
            Duration::from_millis(50),
            Duration::from_millis(10),
            || async { false },
        )
        .await;
        assert!(result.is_err());
    }
}
