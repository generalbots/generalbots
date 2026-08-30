pub fn get_stack_path() -> String {
    if let Ok(path) = std::env::var("BOTSERVER_STACK_PATH") {
        if !path.trim().is_empty() {
            return path;
        }
    }
    if let Ok(path) = std::env::var("GBO_STACK_PATH") {
        if !path.trim().is_empty() {
            return path;
        }
    }
    // Production deployment: the stack data lives at `/opt/gbo` and the
    // binary at `/opt/gbo/bin/botserver`. This must be checked before the
    // dev-relative `./botserver-stack` probe below: in prod the process CWD is
    // normally `/opt/gbo/bin`, where a stray `botserver-stack` directory can
    // exist and would otherwise shadow the real stack root.
    if std::path::Path::new("/opt/gbo/bin/botserver").exists()
        || std::path::Path::new("/opt/gbo/bin/.env").exists()
    {
        return "/opt/gbo".to_string();
    }
    // Development checkout: the stack data dir sits next to the repo root.
    // Absent an explicit marker, assume the relative layout for local runs.
    "./botserver-stack".to_string()
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

#[cfg(test)]
mod path_tests {
    use super::get_stack_path;
    use std::sync::Mutex;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn stack_path_honors_explicit_env_override() {
        let _guard = ENV_LOCK.lock().unwrap();
        let previous = std::env::var_os("BOTSERVER_STACK_PATH");
        std::env::set_var("BOTSERVER_STACK_PATH", "/tmp/some-stack");
        assert_eq!(get_stack_path(), "/tmp/some-stack");
        match previous {
            Some(v) => std::env::set_var("BOTSERVER_STACK_PATH", v),
            None => std::env::remove_var("BOTSERVER_STACK_PATH"),
        }
    }
}
