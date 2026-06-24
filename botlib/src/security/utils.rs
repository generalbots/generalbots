pub fn get_stack_path() -> String {
    let stack_dir = std::path::Path::new("./botserver-stack");
    let has_env = std::path::Path::new("./.env").exists()
        || std::path::Path::new("/opt/gbo/bin/.env").exists();
    let production_base = std::path::Path::new("/opt/gbo/bin/botserver");
    if stack_dir.exists() {
        "./botserver-stack".to_string()
    } else if has_env || production_base.exists() {
        "/opt/gbo".to_string()
    } else {
        "./botserver-stack".to_string()
    }
}

pub fn ca_cert_path() -> String {
    format!("{}/conf/system/certificates/ca/ca.crt", get_stack_path())
}

#[cfg(feature = "http-client")]
pub fn create_tls_client(timeout_secs: Option<u64>) -> reqwest::Client {
    create_tls_client_with_ca(&ca_cert_path(), timeout_secs)
}

#[cfg(feature = "http-client")]
pub fn create_tls_client_with_ca(ca_cert_path: &str, timeout_secs: Option<u64>) -> reqwest::Client {
    use std::time::Duration;
    use log::{debug, warn};

    let timeout = Duration::from_secs(timeout_secs.unwrap_or(30));
    let mut builder = reqwest::Client::builder().timeout(timeout);

    if std::path::Path::new(ca_cert_path).exists() {
        match std::fs::read(ca_cert_path) {
            Ok(ca_cert_pem) => match reqwest::Certificate::from_pem(&ca_cert_pem) {
                Ok(ca_cert) => {
                    builder = builder.add_root_certificate(ca_cert);
                    debug!("Using local CA certificate from {} (dev stack mode)", ca_cert_path);
                }
                Err(e) => {
                    warn!("Failed to parse CA certificate from {}: {}", ca_cert_path, e);
                }
            },
            Err(e) => {
                warn!("Failed to read CA certificate from {}: {}", ca_cert_path, e);
            }
        }
    } else {
        debug!("Local CA cert not found at {}, using system CA store (production mode)", ca_cert_path);
    }

    builder.build().unwrap_or_else(|e| {
        warn!("Failed to create TLS client: {}, using default client", e);
        reqwest::Client::new()
    })
}
