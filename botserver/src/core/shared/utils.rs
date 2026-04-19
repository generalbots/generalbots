use crate::core::secrets::SecretsManager;
use anyhow::{Context, Result};
#[cfg(feature = "drive")]
use crate::core::config::DriveConfig;
#[cfg(feature = "drive")]
use aws_config::retry::RetryConfig;
#[cfg(feature = "drive")]
use aws_config::timeout::TimeoutConfig;
#[cfg(feature = "drive")]
use aws_config::BehaviorVersion;
#[cfg(feature = "drive")]
use aws_sdk_s3::{config::Builder as S3ConfigBuilder, Client as S3Client};
use diesel::Connection;
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};

#[cfg(feature = "progress-bars")]
use indicatif::{ProgressBar, ProgressStyle};
use log::{debug, warn};
use reqwest::{Certificate, Client};
use rhai::{Array, Dynamic};
use serde_json::Value;
use smartstring::SmartString;
use std::error::Error;
use std::sync::Arc;
use std::sync::RwLock;
use std::time::Duration;
use tokio::fs::File as TokioFile;
use tokio::io::AsyncWriteExt;

static SECRETS_MANAGER: std::sync::LazyLock<Arc<RwLock<Option<SecretsManager>>>> =
    std::sync::LazyLock::new(|| Arc::new(RwLock::new(None)));

pub async fn init_secrets_manager() -> Result<()> {
    let manager = SecretsManager::get()?.clone();
    let mut guard = SECRETS_MANAGER.write().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    *guard = Some(manager);
    Ok(())
}

pub async fn get_database_url() -> Result<String> {
    let manager = {
        let guard = SECRETS_MANAGER.read().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
        (*guard).as_ref().map(|manager| manager.clone())
    };

    if let Some(manager) = manager {
        return manager.get_database_url().await;
    }

    Err(anyhow::anyhow!(
        "Secrets manager not initialized"
    ))
}

pub fn get_database_url_sync() -> Result<String> {
    let guard = SECRETS_MANAGER.read().map_err(|e| anyhow::anyhow!("Lock poisoned: {}", e))?;
    if let Some(ref manager) = *guard {
        let manager_clone = manager.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            let result = match rt {
                Ok(rt) => rt.block_on(manager_clone.get_database_url()),
                Err(e) => Err(anyhow::anyhow!("Failed to create runtime: {}", e)),
            };
            let _ = tx.send(result);
        });
        return rx.recv().map_err(|e| anyhow::anyhow!("Channel error: {}", e))?;
    }

    Err(anyhow::anyhow!(
        "Secrets manager not initialized"
    ))
}

pub async fn get_secrets_manager() -> Option<SecretsManager> {
    let guard = SECRETS_MANAGER.read().ok()?;
    guard.clone()
}
pub fn get_secrets_manager_sync() -> Option<SecretsManager> {
    let guard = SECRETS_MANAGER.read().ok()?;
    guard.clone()
}

pub fn get_work_path() -> String {
    let sm = get_secrets_manager_sync();
    if let Some(sm) = sm {
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            let result = match rt {
                Ok(rt) => rt.block_on(sm.get_value("gbo/app", "work_path"))
                    .unwrap_or_else(|_| get_work_path_default()),
                Err(_) => get_work_path_default(),
            };
            let _ = tx.send(result);
        });
        rx.recv().unwrap_or_else(|_| get_work_path_default())
    } else {
        get_work_path_default()
    }
}

/// Returns the work directory path.
/// In production (system container with .env but no botserver-stack): /opt/gbo/work
/// In development (with botserver-stack directory): ./botserver-stack/data/system/work
fn get_work_path_default() -> String {
    let has_env = std::path::Path::new("./.env").exists() 
        || std::path::Path::new("/opt/gbo/bin/.env").exists();
    let production_work = std::path::Path::new("/opt/gbo/work");
    if has_env || production_work.exists() {
        "/opt/gbo/work".to_string()
    } else {
        "./botserver-stack/data/system/work".to_string()
    }
}

/// Returns the stack base path.
/// In production (system container with .env): /opt/gbo
/// In development: ./botserver-stack
pub fn get_stack_path() -> String {
    let has_env = std::path::Path::new("./.env").exists() 
        || std::path::Path::new("/opt/gbo/bin/.env").exists();
    let production_base = std::path::Path::new("/opt/gbo/bin/botserver");
    if has_env || production_base.exists() {
        "/opt/gbo".to_string()
    } else {
        "./botserver-stack".to_string()
    }
}

#[cfg(feature = "drive")]
pub async fn create_s3_operator(
    config: &DriveConfig,
) -> Result<S3Client, Box<dyn std::error::Error>> {
    let endpoint = {
        let base = if config.server.starts_with("http://") || config.server.starts_with("https://") {
            config.server.clone()
        } else {
            format!("http://{}", config.server)
        };
        let with_port = if base.contains("://") {
            let without_scheme = base.split("://").nth(1).unwrap_or("");
            let has_port = without_scheme.contains(':');
            if has_port || without_scheme.is_empty() {
                base
            } else {
                format!("{}:9100", base.trim_end_matches('/'))
            }
        } else {
            format!("http://{}:9100", base)
        };
        if with_port.ends_with('/') {
            with_port
        } else {
            format!("{}/", with_port)
        }
    };
    log::info!("Creating S3 operator with endpoint: {}, access_key: {}", endpoint, config.access_key);

    let (access_key, secret_key) = if config.access_key.is_empty() || config.secret_key.is_empty() {
        let (manager, is_vault_enabled) = {
            let guard = SECRETS_MANAGER.read().map_err(|e| format!("Lock poisoned: {}", e))?;
            if let Some(ref manager) = *guard {
                (Some(manager.clone()), manager.is_enabled())
            } else {
                (None, false)
            }
        };

        match (manager, is_vault_enabled) {
            (Some(manager), true) => {
                match manager.get_drive_credentials().await {
                    Ok((ak, sk)) => (ak, sk),
                    Err(e) => {
                        log::warn!("Failed to get drive credentials from Vault: {}", e);
                        (config.access_key.clone(), config.secret_key.clone())
                    }
                }
            }
            _ => (config.access_key.clone(), config.secret_key.clone())
        }
    } else {
        (config.access_key.clone(), config.secret_key.clone())
    };

    // Set CA cert for self-signed TLS (dev stack)
    let ca_cert = ca_cert_path();
    if std::path::Path::new(&ca_cert).exists() {
        std::env::set_var("AWS_CA_BUNDLE", &ca_cert);
        std::env::set_var("SSL_CERT_FILE", &ca_cert);
        debug!(
            "Set AWS_CA_BUNDLE and SSL_CERT_FILE to {} for S3 client",
            ca_cert
        );
    }

    // Configure timeouts to prevent memory leaks on connection failures
    let timeout_config = TimeoutConfig::builder()
        .connect_timeout(Duration::from_secs(5))
        .read_timeout(Duration::from_secs(30))
        .operation_timeout(Duration::from_secs(30))
        .operation_attempt_timeout(Duration::from_secs(15))
        .build();

    // Limit retries to prevent 100% CPU on connection failures
    let retry_config = RetryConfig::standard().with_max_attempts(2);

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
            pb.set_style(ProgressStyle::default_bar()
                .template("{msg}\n{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
                .unwrap_or(ProgressStyle::default_bar())
                .progress_chars("#>-"));
            pb.set_message(format!("Downloading {}", url));
            let mut file = TokioFile::create(&output_path).await?;
            let bytes = response.bytes().await?;
            file.write_all(&bytes).await?;
            pb.set_position(bytes.len() as u64);
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
            let bytes = response.bytes().await?;
            file.write_all(&bytes).await?;
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
    let mut conn = pool.get()?;
    run_migrations_on_conn(&mut conn)
}

pub fn run_migrations_on_conn(
    conn: &mut diesel::PgConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    // Flat migrations with version-ordinal-feature naming
    const FLAT_MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
    conn.run_pending_migrations(FLAT_MIGRATIONS).map_err(|e| {
        Box::new(std::io::Error::other(format!(
            "Migration error: {}",
            e
        ))) as Box<dyn std::error::Error + Send + Sync>
    })?;

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
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '_'
            }
        })
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

/// Returns the path to the local CA certificate used for internal service TLS (dev stack).
/// In production this path may not exist, which is fine — the system CA store is used instead.
pub fn ca_cert_path() -> String {
    format!("{}/conf/system/certificates/ca/ca.crt", get_stack_path())
}

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
    create_tls_client_with_ca(&ca_cert_path(), timeout_secs)
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
            Ok(ca_cert_pem) => match Certificate::from_pem(&ca_cert_pem) {
                Ok(ca_cert) => {
                    builder = builder.add_root_certificate(ca_cert);
                    debug!(
                        "Using local CA certificate from {} (dev stack mode)",
                        ca_cert_path
                    );
                }
                Err(e) => {
                    warn!(
                        "Failed to parse CA certificate from {}: {}",
                        ca_cert_path, e
                    );
                }
            },
            Err(e) => {
                warn!("Failed to read CA certificate from {}: {}", ca_cert_path, e);
            }
        }
    } else {
        debug!(
            "Local CA cert not found at {}, using system CA store (production mode)",
            ca_cert_path
        );
    }

    builder.build().unwrap_or_else(|e| {
        warn!("Failed to create TLS client: {}, using default client", e);
        Client::new()
    })
}

pub fn format_timestamp_plain(ms: i64) -> String {
    let secs = ms / 1000;
    let mins = secs / 60;
    let hours = mins / 60;
    format!("{:02}:{:02}:{:02}", hours, mins % 60, secs % 60)
}

pub fn format_timestamp_vtt(ms: i64) -> String {
    let secs = ms / 1000;
    let mins = secs / 60;
    let hours = mins / 60;
    let millis = ms % 1000;
    format!(
        "{:02}:{:02}:{:02}.{:03}",
        hours,
        mins % 60,
        secs % 60,
        millis
    )
}

pub fn format_timestamp_srt(ms: i64) -> String {
    let secs = ms / 1000;
    let mins = secs / 60;
    let hours = mins / 60;
    let millis = ms % 1000;
    format!(
        "{:02}:{:02}:{:02},{:03}",
        hours,
        mins % 60,
        secs % 60,
        millis
    )
}

pub fn parse_hex_color(hex: &str) -> Option<(u8, u8, u8)> {
    let hex = hex.trim_start_matches('#');
    if hex.len() < 6 {
        return None;
    }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
    Some((r, g, b))
}

/// Estimates token count based on model type and truncates text to fit within token limit
pub fn truncate_text_for_model(text: &str, model: &str, max_tokens: usize) -> String {
    let chars_per_token = estimate_chars_per_token(model);
    let max_chars = max_tokens * chars_per_token;

    if text.chars().count() <= max_chars {
        return text.to_string();
    }

    // Get first max_chars characters safely (UTF-8 aware)
    let truncated: String = text.chars().take(max_chars).collect();

    // Try to truncate at word boundary
    if let Some(last_space_idx) = truncated.rfind(' ') {
        truncated[..last_space_idx].to_string()
    } else {
        truncated
    }
}

/// Estimates characters per token based on model type
fn estimate_chars_per_token(model: &str) -> usize {
    if model.contains("llama") || model.contains("mistral") {
        3 // Llama/Mistral models: ~3 chars per token
    } else {
        4 // GPT/Claude/BERT models and default: ~4 chars per token
    }
}

/// Convert date string from user locale format to ISO format (YYYY-MM-DD) for PostgreSQL.
///
/// The LLM automatically formats dates according to the user's language/idiom based on:
/// 1. The conversation context (user's language)
/// 2. The PARAM LIKE example (e.g., "15/12/2026" for DD/MM/YYYY)
///
/// This function handles the most common formats:
/// - ISO: YYYY-MM-DD (already in ISO, returned as-is)
/// - Brazilian/Portuguese: DD/MM/YYYY or DD/MM/YY
/// - US/English: MM/DD/YYYY or MM/DD/YY
///
/// If the value doesn't match any date pattern, returns it unchanged.
///
/// NOTE: This function does NOT try to guess ambiguous formats.
/// The LLM is responsible for formatting dates correctly based on user language.
/// The PARAM declaration's LIKE example tells the LLM the expected format.
///
/// # Arguments
/// * `value` - The date string to convert (as provided by the LLM)
///
/// # Returns
/// ISO formatted date string (YYYY-MM-DD) or original value if not a recognized date
pub fn convert_date_to_iso_format(value: &str) -> String {
    let value = value.trim();

    // Already in ISO format (YYYY-MM-DD) - return as-is
    if value.len() == 10 && value.chars().nth(4) == Some('-') && value.chars().nth(7) == Some('-') {
        let parts: Vec<&str> = value.split('-').collect();
        if parts.len() == 3
            && parts[0].len() == 4
            && parts[1].len() == 2
            && parts[2].len() == 2
            && parts[0].chars().all(|c| c.is_ascii_digit())
            && parts[1].chars().all(|c| c.is_ascii_digit())
            && parts[2].chars().all(|c| c.is_ascii_digit())
        {
            if let (Ok(year), Ok(month), Ok(day)) =
                (parts[0].parse::<u32>(), parts[1].parse::<u32>(), parts[2].parse::<u32>())
            {
                if (1..=12).contains(&month) && (1..=31).contains(&day) && (1900..=2100).contains(&year) {
                    return value.to_string();
                }
            }
        }
    }

    // Handle slash-separated formats: DD/MM/YYYY or MM/DD/YYYY
    // We need to detect which format based on the PARAM declaration's LIKE example
    // For now, default to DD/MM/YYYY (Brazilian format) as this is the most common for this bot
    // TODO: Pass language/idiom from session to determine correct format
    if value.len() >= 8 && value.len() <= 10 {
        let parts: Vec<&str> = value.split('/').collect();
        if parts.len() == 3 {
            let all_numeric = parts[0].chars().all(|c| c.is_ascii_digit())
                && parts[1].chars().all(|c| c.is_ascii_digit())
                && parts[2].chars().all(|c| c.is_ascii_digit());

            if all_numeric {
                // Parse the three parts
                let a = parts[0].parse::<u32>().ok();
                let b = parts[1].parse::<u32>().ok();
                let c = if parts[2].len() == 2 {
                    // Convert 2-digit year to 4-digit
                    parts[2].parse::<u32>().ok().map(|y| {
                        if y < 50 {
                            2000 + y
                        } else {
                            1900 + y
                        }
                    })
                } else {
                    parts[2].parse::<u32>().ok()
                };

                if let (Some(first), Some(second), Some(third)) = (a, b, c) {
                    // Default: DD/MM/YYYY format (Brazilian/Portuguese)
                    // The LLM should format dates according to the user's language
                    // and the PARAM LIKE example (e.g., "15/12/2026" for DD/MM/YYYY)
                    let (year, month, day) = (third, second, first);

                    // Validate the determined date
                    if (1..=31).contains(&day) && (1..=12).contains(&month) && (1900..=2100).contains(&year) {
                        return format!("{:04}-{:02}-{:02}", year, month, day);
                    }
                }
            }
        }
    }

    // Not a recognized date pattern, return unchanged
    value.to_string()
}
