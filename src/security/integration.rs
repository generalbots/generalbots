//! TLS Integration Module
//!
//! This module provides helper functions and utilities for integrating TLS/HTTPS
//! with existing services, including automatic URL conversion and client configuration.

use anyhow::{Context, Result};
use reqwest::{Certificate, Client, ClientBuilder, Identity};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::Duration;
use tracing::{info, warn};

/// Service URL mappings for TLS conversion
#[derive(Debug, Clone)]
pub struct ServiceUrls {
    pub original: String,
    pub secure: String,
    pub port: u16,
    pub tls_port: u16,
}

/// TLS Integration Manager
#[derive(Debug)]
pub struct TlsIntegration {
    /// Service URL mappings
    services: HashMap<String, ServiceUrls>,

    /// CA certificate for validation
    ca_cert: Option<Certificate>,

    /// Client certificates for mTLS
    client_certs: HashMap<String, Identity>,

    /// Whether TLS is enabled globally
    tls_enabled: bool,

    /// Whether to enforce HTTPS for all connections
    https_only: bool,
}

impl TlsIntegration {
    /// Create a new TLS integration manager
    pub fn new(tls_enabled: bool) -> Self {
        let mut services = HashMap::new();

        // Define service mappings
        services.insert(
            "api".to_string(),
            ServiceUrls {
                original: "http://localhost:8080".to_string(),
                secure: "https://localhost:8443".to_string(),
                port: 8080,
                tls_port: 8443,
            },
        );

        services.insert(
            "llm".to_string(),
            ServiceUrls {
                original: "http://localhost:8081".to_string(),
                secure: "https://localhost:8444".to_string(),
                port: 8081,
                tls_port: 8444,
            },
        );

        services.insert(
            "embedding".to_string(),
            ServiceUrls {
                original: "http://localhost:8082".to_string(),
                secure: "https://localhost:8445".to_string(),
                port: 8082,
                tls_port: 8445,
            },
        );

        services.insert(
            "qdrant".to_string(),
            ServiceUrls {
                original: "http://localhost:6333".to_string(),
                secure: "https://localhost:6334".to_string(),
                port: 6333,
                tls_port: 6334,
            },
        );

        services.insert(
            "redis".to_string(),
            ServiceUrls {
                original: "redis://localhost:6379".to_string(),
                secure: "rediss://localhost:6380".to_string(),
                port: 6379,
                tls_port: 6380,
            },
        );

        services.insert(
            "postgres".to_string(),
            ServiceUrls {
                original: "postgres://localhost:5432".to_string(),
                secure: "postgres://localhost:5433?sslmode=require".to_string(),
                port: 5432,
                tls_port: 5433,
            },
        );

        services.insert(
            "minio".to_string(),
            ServiceUrls {
                original: "http://localhost:9000".to_string(),
                secure: "https://localhost:9001".to_string(),
                port: 9000,
                tls_port: 9001,
            },
        );

        services.insert(
            "directory".to_string(),
            ServiceUrls {
                original: "http://localhost:8080".to_string(),
                secure: "https://localhost:8446".to_string(),
                port: 8080,
                tls_port: 8446,
            },
        );

        Self {
            services,
            ca_cert: None,
            client_certs: HashMap::new(),
            tls_enabled,
            https_only: tls_enabled,
        }
    }

    /// Load CA certificate
    pub fn load_ca_cert(&mut self, ca_path: &Path) -> Result<()> {
        if ca_path.exists() {
            let ca_cert_pem = fs::read(ca_path)
                .with_context(|| format!("Failed to read CA certificate from {:?}", ca_path))?;

            let ca_cert =
                Certificate::from_pem(&ca_cert_pem).context("Failed to parse CA certificate")?;

            self.ca_cert = Some(ca_cert);
            info!("Loaded CA certificate from {:?}", ca_path);
        } else {
            warn!("CA certificate not found at {:?}", ca_path);
        }

        Ok(())
    }

    /// Load client certificate for mTLS
    pub fn load_client_cert(
        &mut self,
        service: &str,
        cert_path: &Path,
        key_path: &Path,
    ) -> Result<()> {
        if cert_path.exists() && key_path.exists() {
            let cert = fs::read(cert_path)
                .with_context(|| format!("Failed to read client cert from {:?}", cert_path))?;

            let key = fs::read(key_path)
                .with_context(|| format!("Failed to read client key from {:?}", key_path))?;

            let identity = Identity::from_pem(&[&cert[..], &key[..]].concat())
                .context("Failed to create client identity")?;

            self.client_certs.insert(service.to_string(), identity);
            info!("Loaded client certificate for service: {}", service);
        } else {
            warn!("Client certificate not found for service: {}", service);
        }

        Ok(())
    }

    /// Convert URL to HTTPS if TLS is enabled
    pub fn convert_url(&self, url: &str) -> String {
        if !self.tls_enabled {
            return url.to_string();
        }

        // Check if URL matches any known service
        for (_service, urls) in &self.services {
            if url.starts_with(&urls.original) {
                return url.replace(&urls.original, &urls.secure);
            }
        }

        // Generic conversion for unknown services
        if url.starts_with("http://") {
            url.replace("http://", "https://")
        } else if url.starts_with("redis://") {
            url.replace("redis://", "rediss://")
        } else {
            url.to_string()
        }
    }

    /// Get service URL (returns HTTPS if TLS is enabled)
    pub fn get_service_url(&self, service: &str) -> Option<String> {
        self.services.get(service).map(|urls| {
            if self.tls_enabled {
                urls.secure.clone()
            } else {
                urls.original.clone()
            }
        })
    }

    /// Create HTTPS client for a specific service
    pub fn create_client(&self, service: &str) -> Result<Client> {
        let mut builder = ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .connect_timeout(Duration::from_secs(10));

        if self.tls_enabled {
            // Use rustls for TLS
            builder = builder.use_rustls_tls();

            // Add CA certificate if available
            if let Some(ca_cert) = &self.ca_cert {
                builder = builder.add_root_certificate(ca_cert.clone());
            }

            // Add client certificate for mTLS if available
            if let Some(identity) = self.client_certs.get(service) {
                builder = builder.identity(identity.clone());
            }

            // For development, allow self-signed certificates
            if cfg!(debug_assertions) {
                builder = builder.danger_accept_invalid_certs(true);
            }

            if self.https_only {
                builder = builder.https_only(true);
            }
        }

        builder.build().context("Failed to build HTTP client")
    }

    /// Create a generic HTTPS client
    pub fn create_generic_client(&self) -> Result<Client> {
        self.create_client("generic")
    }

    /// Check if TLS is enabled
    pub fn is_tls_enabled(&self) -> bool {
        self.tls_enabled
    }

    /// Get the secure port for a service
    pub fn get_secure_port(&self, service: &str) -> Option<u16> {
        self.services.get(service).map(|urls| {
            if self.tls_enabled {
                urls.tls_port
            } else {
                urls.port
            }
        })
    }

    /// Update PostgreSQL connection string for TLS
    pub fn update_postgres_url(&self, url: &str) -> String {
        if !self.tls_enabled {
            return url.to_string();
        }

        // Parse and update PostgreSQL URL
        if url.contains("localhost:5432") || url.contains("127.0.0.1:5432") {
            let base = url
                .replace("localhost:5432", "localhost:5433")
                .replace("127.0.0.1:5432", "127.0.0.1:5433");

            // Add SSL parameters if not present
            if !base.contains("sslmode=") {
                if base.contains('?') {
                    format!("{}&sslmode=require", base)
                } else {
                    format!("{}?sslmode=require", base)
                }
            } else {
                base
            }
        } else {
            url.to_string()
        }
    }

    /// Update Redis connection string for TLS
    pub fn update_redis_url(&self, url: &str) -> String {
        if !self.tls_enabled {
            return url.to_string();
        }

        if url.starts_with("redis://") {
            url.replace("redis://", "rediss://")
                .replace(":6379", ":6380")
        } else {
            url.to_string()
        }
    }

    /// Load all certificates from a directory
    pub fn load_all_certs_from_dir(&mut self, cert_dir: &Path) -> Result<()> {
        // Load CA certificate
        let ca_path = cert_dir.join("ca.crt");
        if ca_path.exists() {
            self.load_ca_cert(&ca_path)?;
        }

        // Load service client certificates
        for service in &[
            "api",
            "llm",
            "embedding",
            "qdrant",
            "postgres",
            "redis",
            "minio",
        ] {
            let service_dir = cert_dir.join(service);
            if service_dir.exists() {
                let cert_path = service_dir.join("client.crt");
                let key_path = service_dir.join("client.key");

                if cert_path.exists() && key_path.exists() {
                    self.load_client_cert(service, &cert_path, &key_path)?;
                }
            }
        }

        Ok(())
    }
}

/// Global TLS integration instance using OnceLock for safe initialization
static TLS_INTEGRATION: OnceLock<Arc<TlsIntegration>> = OnceLock::new();

/// Initialize global TLS integration
pub fn init_tls_integration(tls_enabled: bool, cert_dir: Option<PathBuf>) -> Result<()> {
    let _ = TLS_INTEGRATION.get_or_init(|| {
        let mut integration = TlsIntegration::new(tls_enabled);

        if tls_enabled {
            if let Some(dir) = cert_dir {
                if let Err(e) = integration.load_all_certs_from_dir(&dir) {
                    warn!("Failed to load some certificates: {}", e);
                }
            }
        }

        info!("TLS integration initialized (TLS: {})", tls_enabled);
        Arc::new(integration)
    });

    Ok(())
}

/// Get the global TLS integration instance
pub fn get_tls_integration() -> Option<Arc<TlsIntegration>> {
    TLS_INTEGRATION.get().cloned()
}

/// Convert a URL to HTTPS using global TLS settings
pub fn to_secure_url(url: &str) -> String {
    if let Some(integration) = get_tls_integration() {
        integration.convert_url(url)
    } else {
        url.to_string()
    }
}

/// Create an HTTPS client for a service using global TLS settings
pub fn create_https_client(service: &str) -> Result<Client> {
    if let Some(integration) = get_tls_integration() {
        integration.create_client(service)
    } else {
        // Fallback to default client
        Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context("Failed to build default HTTP client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_conversion() {
        let integration = TlsIntegration::new(true);

        assert_eq!(
            integration.convert_url("http://localhost:8081"),
            "https://localhost:8444"
        );

        assert_eq!(
            integration.convert_url("redis://localhost:6379"),
            "rediss://localhost:6380"
        );

        assert_eq!(
            integration.convert_url("https://example.com"),
            "https://example.com"
        );
    }

    #[test]
    fn test_postgres_url_update() {
        let integration = TlsIntegration::new(true);

        assert_eq!(
            integration.update_postgres_url("postgres://user:pass@localhost:5432/db"),
            "postgres://user:pass@localhost:5433/db?sslmode=require"
        );

        assert_eq!(
            integration.update_postgres_url("postgres://localhost:5432/db?foo=bar"),
            "postgres://localhost:5433/db?foo=bar&sslmode=require"
        );
    }

    #[test]
    fn test_service_url() {
        let integration = TlsIntegration::new(true);

        assert_eq!(
            integration.get_service_url("llm"),
            Some("https://localhost:8444".to_string())
        );

        let integration_no_tls = TlsIntegration::new(false);
        assert_eq!(
            integration_no_tls.get_service_url("llm"),
            Some("http://localhost:8081".to_string())
        );
    }

    #[test]
    fn test_secure_port() {
        let integration = TlsIntegration::new(true);

        assert_eq!(integration.get_secure_port("api"), Some(8443));
        assert_eq!(integration.get_secure_port("redis"), Some(6380));
        assert_eq!(integration.get_secure_port("unknown"), None);
    }
}
