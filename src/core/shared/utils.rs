use crate::config::DriveConfig;
use crate::core::secrets::SecretsManager;
use anyhow::{Context, Result};
use aws_config::BehaviorVersion;
use aws_config::retry::RetryConfig;
use aws_config::timeout::TimeoutConfig;
use aws_sdk_s3::{config::Builder as S3ConfigBuilder, Client as S3Client};
use diesel::Connection;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use futures_util::StreamExt;
#[cfg(feature = "progress-bars")]
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, warn};
use reqwest::{Certificate, Client};
use rhai::{Array, Dynamic};
use serde_json::Value;
use smartstring::SmartString;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::File as TokioFile;
use tokio::io::AsyncWriteExt;
use tokio::sync::RwLock;

static SECRETS_MANAGER: std::sync::LazyLock<Arc<RwLock<Option<SecretsManager>>>> =
    std::sync::LazyLock::new(|| Arc::new(RwLock::new(None)));

pub async fn init_secrets_manager() -> Result<()> {
    let manager = SecretsManager::from_env()?;
    let mut guard = SECRETS_MANAGER.write().await;
    *guard = Some(manager);
    Ok(())
}

pub async fn get_database_url() -> Result<String> {
    let guard = SECRETS_MANAGER.read().await;
    if let Some(ref manager) = *guard {
        if manager.is_enabled() {
            return manager.get_database_url().await;
        }
    }

    Err(anyhow::anyhow!(
        "Vault not configured. Set VAULT_ADDR and VAULT_TOKEN in .env"
    ))
}

pub fn get_database_url_sync() -> Result<String> {
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        let result =
            tokio::task::block_in_place(|| handle.block_on(async { get_database_url().await }));
        if let Ok(url) = result {
            return Ok(url);
        }
    } else {
        let rt = tokio::runtime::Runtime::new()
            .map_err(|e| anyhow::anyhow!("Failed to create runtime: {}", e))?;
        if let Ok(url) = rt.block_on(async { get_database_url().await }) {
            return Ok(url);
        }
    }

    Err(anyhow::anyhow!(
        "Vault not configured. Set VAULT_ADDR and VAULT_TOKEN in .env"
    ))
}

pub async fn get_secrets_manager() -> Option<SecretsManager> {
    let guard = SECRETS_MANAGER.read().await;
    guard.clone()
}

pub async fn create_s3_operator(
    config: &DriveConfig,
) -> Result<S3Client, Box<dyn std::error::Error>> {
    let endpoint = if config.server.ends_with('/') {
        config.server.clone()
    } else {
        format!("{}/", config.server)
    };

    let (access_key, secret_key) = if config.access_key.is_empty() || config.secret_key.is_empty() {
        let guard = SECRETS_MANAGER.read().await;
        if let Some(ref manager) = *guard {
            if manager.is_enabled() {
                match manager.get_drive_credentials().await {
                    Ok((ak, sk)) => (ak, sk),
                    Err(e) => {
                        log::warn!("Failed to get drive credentials from Vault: {}", e);
                        (config.access_key.clone(), config.secret_key.clone())
                    }
                }
            } else {
                (config.access_key.clone(), config.secret_key.clone())
            }
        } else {
            (config.access_key.clone(), config.secret_key.clone())
        }
    } else {
        (config.access_key.clone(), config.secret_key.clone())
    };

    // Set CA cert for self-signed TLS (dev stack)
    if std::path::Path::new(CA_CERT_PATH).exists() {
        std::env::set_var("AWS_CA_BUNDLE", CA_CERT_PATH);
        std::env::set_var("SSL_CERT_FILE", CA_CERT_PATH);
        debug!("Set AWS_CA_BUNDLE and SSL_CERT_FILE to {} for S3 client", CA_CERT_PATH);
    }

    // Configure timeouts to prevent memory leaks on connection failures
    let timeout_config = TimeoutConfig::builder()
        .connect_timeout(Duration::from_secs(5))
        .read_timeout(Duration::from_secs(30))
        .operation_timeout(Duration::from_secs(30))
        .operation_attempt_timeout(Duration::from_secs(15))
        .build();

    // Limit retries to prevent 100% CPU on connection failures
    let retry_config = RetryConfig::standard()
        .with_max_attempts(2);

    let base_config = aws_config::defaults(BehaviorVersion::latest())
        .endpoint_url(endpoint)
        .region("auto")
        .credentials_provider(aws_sdk_s3::config::Credentials::new(
            access_key, secret_key, None, None, "static",
        ))
        .timeout_config(timeout_config)
        .retry_config(retry_config)
        .load()
        .await;
    let s3_config = S3ConfigBuilder::from(&base_config)
        .force_path_style(true)
        .build();
    Ok(S3Client::from_conf(s3_config))
}

pub fn json_value_to_dynamic(value: &Value) -> Dynamic {
    match value {
        Value::Null => Dynamic::UNIT,
        Value::Bool(b) => Dynamic::from(*b),
        Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Dynamic::from(i)
            } else if let Some(f) = n.as_f64() {
                Dynamic::from(f)
            } else {
                Dynamic::UNIT
            }
        }
        Value::String(s) => Dynamic::from(s.clone()),
        Value::Array(arr) => Dynamic::from(
            arr.iter()
                .map(json_value_to_dynamic)
                .collect::<rhai::Array>(),
        ),
        Value::Object(obj) => Dynamic::from(
            obj.iter()
                .map(|(k, v)| (SmartString::from(k), json_value_to_dynamic(v)))
                .collect::<rhai::Map>(),
        ),
    }
}

pub fn to_array(value: Dynamic) -> Array {
    if value.is_array() {
        value.cast::<Array>()
    } else if value.is_unit() || value.is::<()>() {
        Array::new()
    } else {
        Array::from([value])
    }
}

#[cfg(feature = "progress-bars")]
pub async fn download_file(url: &str, output_path: &str) -> Result<(), anyhow::Error> {
    use std::time::Duration;
    let url = url.to_string();
    let output_path = output_path.to_string();
    let download_handle = tokio::spawn(async move {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (compatible; BotServer/1.0)")
            .connect_timeout(Duration::from_secs(30))
            .read_timeout(Duration::from_secs(300))
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .build()?;
        let response = client.get(&url).send().await?;
        if response.status().is_success() {
            let total_size = response.content_length().unwrap_or(0);
            let pb = ProgressBar::new(total_size);
            #[allow(clippy::literal_string_with_formatting_args)]
            pb.set_style(ProgressStyle::default_bar()
                .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .expect("Invalid progress bar template")
                .progress_chars("#>-"));
            pb.set_message(format!("Downloading {}", url));
            let mut file = TokioFile::create(&output_path).await?;
            let mut downloaded: u64 = 0;
            let mut stream = response.bytes_stream();
            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result?;
                file.write_all(&chunk).await?;
                downloaded += chunk.len() as u64;
                pb.set_position(downloaded);
            }
            pb.finish_with_message(format!("Downloaded {}", output_path));
            Ok(())
        } else {
            Err(anyhow::anyhow!("HTTP {}: {}", response.status(), url))
        }
    });
    download_handle.await?
}

#[cfg(not(feature = "progress-bars"))]
pub async fn download_file(url: &str, output_path: &str) -> Result<(), anyhow::Error> {
    use std::time::Duration;
    let url = url.to_string();
    let output_path = output_path.to_string();
    let download_handle = tokio::spawn(async move {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (compatible; BotServer/1.0)")
            .connect_timeout(Duration::from_secs(30))
            .read_timeout(Duration::from_secs(300))
            .pool_idle_timeout(Duration::from_secs(90))
            .tcp_keepalive(Duration::from_secs(60))
            .build()?;
        let response = client.get(&url).send().await?;
        if response.status().is_success() {
            let mut file = TokioFile::create(&output_path).await?;
            let mut stream = response.bytes_stream();
            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result?;
                file.write_all(&chunk).await?;
            }
            Ok(())
        } else {
            Err(anyhow::anyhow!("HTTP {}: {}", response.status(), url))
        }
    });
    download_handle.await?
}

pub fn parse_filter(filter_str: &str) -> Result<(String, Vec<String>), Box<dyn Error>> {
    let parts: Vec<&str> = filter_str.split('=').collect();
    if parts.len() != 2 {
        return Err("Invalid filter format. Expected 'KEY=VALUE'".into());
    }
    let column = parts[0].trim();
    let value = parts[1].trim();
    if !column
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err("Invalid column name in filter".into());
    }
    Ok((format!("{} = $1", column), vec![value.to_string()]))
}

pub fn estimate_token_count(text: &str) -> usize {
    let char_count = text.chars().count();
    (char_count / 4).max(1)
}

pub fn establish_pg_connection() -> Result<PgConnection> {
    let database_url = get_database_url_sync()?;
    PgConnection::establish(&database_url)
        .with_context(|| format!("Failed to connect to database at {}", database_url))
}

pub type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn create_conn() -> Result<DbPool, anyhow::Error> {
    let database_url = get_database_url_sync()?;
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .max_size(10)
        .min_idle(Some(1))
        .connection_timeout(std::time::Duration::from_secs(5))
        .idle_timeout(Some(std::time::Duration::from_secs(300)))
        .max_lifetime(Some(std::time::Duration::from_secs(1800)))
        .build(manager)
        .map_err(|e| anyhow::anyhow!("Failed to create database pool: {}", e))
}

pub async fn create_conn_async() -> Result<DbPool, anyhow::Error> {
    let database_url = get_database_url().await?;
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .max_size(10)
        .min_idle(Some(1))
        .connection_timeout(std::time::Duration::from_secs(5))
        .idle_timeout(Some(std::time::Duration::from_secs(300)))
        .max_lifetime(Some(std::time::Duration::from_secs(1800)))
        .build(manager)
        .map_err(|e| anyhow::anyhow!("Failed to create database pool: {}", e))
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

pub fn run_migrations(pool: &DbPool) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

    let mut conn = pool.get()?;
    conn.run_pending_migrations(MIGRATIONS).map_err(
        |e| -> Box<dyn std::error::Error + Send + Sync> {
            Box::new(std::io::Error::other(format!("Migration error: {}", e)))
        },
    )?;
    Ok(())
}

pub use crate::security::sql_guard::sanitize_identifier;

pub fn sanitize_path_component(component: &str) -> String {
    component
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
        .collect::<String>()
        .trim_start_matches('.')
        .to_string()
}

pub fn sanitize_path_for_filename(path: &str) -> String {
    path.chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect()
}

pub fn get_content_type(path: &str) -> &'static str {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext.as_deref() {
        Some("html") | Some("htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("bas") => "text/plain; charset=utf-8",
        Some("png") => "image/png",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("ico") => "image/x-icon",
        Some("woff") => "font/woff",
        Some("woff2") => "font/woff2",
        Some("ttf") => "font/ttf",
        Some("eot") => "application/vnd.ms-fontobject",
        Some("otf") => "font/otf",
        Some("txt") => "text/plain; charset=utf-8",
        Some("xml") => "application/xml; charset=utf-8",
        Some("pdf") => "application/pdf",
        Some("zip") => "application/zip",
        Some("webp") => "image/webp",
        Some("mp3") => "audio/mpeg",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        _ => "application/octet-stream",
    }
}

pub fn sanitize_sql_value(value: &str) -> String {
    value.replace('\'', "''")
}

/// Default path to the local CA certificate used for internal service TLS (dev stack)
pub const CA_CERT_PATH: &str = "./botserver-stack/conf/system/certificates/ca/ca.crt";

/// Creates an HTTP client with proper TLS verification.
///
/// **Behavior:**
/// - If local CA cert exists (dev stack): uses it for verification
/// - If local CA cert doesn't exist (production): uses system CA store
///
/// # Arguments
/// * `timeout_secs` - Request timeout in seconds (default: 30)
///
/// # Returns
/// A reqwest::Client configured for TLS verification
pub fn create_tls_client(timeout_secs: Option<u64>) -> Client {
    create_tls_client_with_ca(CA_CERT_PATH, timeout_secs)
}

/// Creates an HTTP client with a custom CA certificate path.
///
/// **Behavior:**
/// - If CA cert file exists: adds it as trusted root (for self-signed/internal CA)
/// - If CA cert file doesn't exist: uses system CA store (for public CAs like Let's Encrypt)
///
/// This allows seamless transition from dev (local CA) to production (public CA).
///
/// # Arguments
/// * `ca_cert_path` - Path to the CA certificate file (ignored if file doesn't exist)
/// * `timeout_secs` - Request timeout in seconds (default: 30)
///
/// # Returns
/// A reqwest::Client configured for TLS verification
pub fn create_tls_client_with_ca(ca_cert_path: &str, timeout_secs: Option<u64>) -> Client {
    let timeout = Duration::from_secs(timeout_secs.unwrap_or(30));
    let mut builder = Client::builder().timeout(timeout);

    // Try to load local CA cert (dev stack with self-signed certs)
    // If it doesn't exist, we use system CA store (production with public certs)
    if std::path::Path::new(ca_cert_path).exists() {
        match std::fs::read(ca_cert_path) {
            Ok(ca_cert_pem) => {
                match Certificate::from_pem(&ca_cert_pem) {
                    Ok(ca_cert) => {
                        builder = builder.add_root_certificate(ca_cert);
                        debug!("Using local CA certificate from {} (dev stack mode)", ca_cert_path);
                    }
                    Err(e) => {
                        warn!("Failed to parse CA certificate from {}: {}", ca_cert_path, e);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read CA certificate from {}: {}", ca_cert_path, e);
            }
        }
    } else {
        debug!("Local CA cert not found at {}, using system CA store (production mode)", ca_cert_path);
    }

    builder.build().unwrap_or_else(|e| {
        warn!("Failed to create TLS client: {}, using default client", e);
        Client::new()
    })
}
