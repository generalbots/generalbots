use super::*;
use botcoresecrets::mtls::runtime::{secure_database_url, secure_http_url, ClientTlsPaths};
use botcoresecrets::mtls::status::init as mtls_init;

/// Resolve the database URL with the mTLS transport applied. A configuration
/// failure is fatal here: falling back to plaintext would defeat the opt-in.
fn database_url_with_mtls() -> Result<String, anyhow::Error> {
    let url = get_database_url_sync()?;
    match mtls_init() {
        Ok(config) => Ok(secure_database_url(&url, config)),
        Err(error) => Err(anyhow::anyhow!(
            "mTLS configuration unavailable for database transport: {error}"
        )),
    }
}

/// Build an HTTP client that verifies the peer against the configured CA and
/// presents this service's client certificate. Errors name the missing item
/// without printing paths or certificate/key material.
fn client_with_tls_paths(
    paths: &ClientTlsPaths,
    timeout: Duration,
) -> Result<Client, anyhow::Error> {
    let mut builder = Client::builder()
        .timeout(timeout)
        .connect_timeout(Duration::from_secs(5));
    if paths.has_ca() {
        let ca_pem = std::fs::read(&paths.ca_cert)
            .map_err(|e| anyhow::anyhow!("could not read mTLS CA path: {e}"))?;
        let ca = Certificate::from_pem(&ca_pem)
            .map_err(|e| anyhow::anyhow!("could not parse mTLS CA: {e}"))?;
        builder = builder.add_root_certificate(ca);
    }
    if paths.has_identity() {
        let mut identity_pem = std::fs::read(&paths.client_cert)
            .map_err(|e| anyhow::anyhow!("could not read mTLS client cert path: {e}"))?;
        let key_pem = std::fs::read(&paths.client_key)
            .map_err(|e| anyhow::anyhow!("could not read mTLS client key path: {e}"))?;
        identity_pem.extend_from_slice(&key_pem);
        let identity = reqwest::Identity::from_pem(&identity_pem)
            .map_err(|e| anyhow::anyhow!("could not parse mTLS client identity: {e}"))?;
        builder = builder.identity(identity);
    }
    builder
        .build()
        .map_err(|e| anyhow::anyhow!("could not build mTLS client: {e}"))
}

pub fn create_conn() -> Result<DbPool, anyhow::Error> {
    let database_url = database_url_with_mtls()?;
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .max_size(50)
        .min_idle(Some(0))
        .connection_timeout(std::time::Duration::from_secs(5))
        .idle_timeout(Some(std::time::Duration::from_secs(300)))
        .max_lifetime(Some(std::time::Duration::from_secs(1800)))
        .build(manager)
        .map_err(|e| anyhow::anyhow!("Failed to create database pool: {}", e))
}
pub async fn create_conn_async() -> Result<DbPool, anyhow::Error> {
    let database_url = database_url_with_mtls()?;
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    Pool::builder()
        .max_size(50)
        .min_idle(Some(0))
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
pub fn run_migrations(
    _conn: &mut diesel::PgConnection,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Migrations handled by botserver, not botcore
    Ok(())
}
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
    let config = match mtls_init() {
        Ok(config) => config,
        Err(error) => {
            warn!("[mtls] configuration unavailable for HTTP client: {error}");
            return create_tls_client_with_ca(&ca_cert_path(), timeout_secs);
        }
    };
    let paths = match ClientTlsPaths::from_config(config) {
        Some(paths) => paths,
        None => return create_tls_client_with_ca(&ca_cert_path(), timeout_secs),
    };
    let timeout = Duration::from_secs(timeout_secs.unwrap_or(30));
    match client_with_tls_paths(&paths, timeout) {
        Ok(client) => client,
        Err(error) => {
            warn!("[mtls] client certificate setup failed: {error}; using CA verification only");
            create_tls_client_with_ca(&paths.ca_cert, timeout_secs)
        }
    }
}
/// Rewrite an internal HTTP service URL to HTTPS when mTLS is active.
/// Plaintext installs keep the original URL.
#[must_use]
pub fn internal_service_url(base: &str) -> String {
    match mtls_init() {
        Ok(config) => secure_http_url(base, config),
        Err(error) => {
            warn!("[mtls] configuration unavailable for service URL: {error}");
            base.to_string()
        }
    }
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
    let mut builder = Client::builder()
        .timeout(timeout)
        .connect_timeout(Duration::from_secs(5));

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
pub(super) fn estimate_chars_per_token(model: &str) -> usize {
    if model.contains("llama") || model.contains("mistral") {
        3 // Llama/Mistral models: ~3 chars per token
    } else {
        4 // GPT/Claude/BERT models and default: ~4 chars per token
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_tls_material_fails_closed_without_leaking_paths() {
        let paths = ClientTlsPaths {
            ca_cert: "/no/such/ca.pem".to_string(),
            client_cert: "/no/such/client.pem".to_string(),
            client_key: "/no/such/client.key".to_string(),
        };
        let result = client_with_tls_paths(&paths, Duration::from_secs(1));
        assert!(result.is_err(), "missing CA must not build a client");
        if let Err(error) = result {
            let message = error.to_string();
            assert!(message.contains("could not read mTLS CA path"));
            assert!(!message.contains("no/such"), "path leaked: {message}");
        }
    }
}
