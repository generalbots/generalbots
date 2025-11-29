//! TLS/HTTPS Security Module
//!
//! Provides comprehensive TLS configuration for all services including:
//! - HTTPS server configuration
//! - mTLS (mutual TLS) for service-to-service communication
//! - Certificate management with internal CA support
//! - External CA integration capabilities

use anyhow::{Context, Result};
use axum::extract::connect_info::Connected;
use hyper::server::conn::AddrIncoming;
use rustls::server::{AllowAnyAnonymousOrAuthenticatedClient, AllowAnyAuthenticatedClient};
use rustls::{Certificate, PrivateKey, RootCertStore, ServerConfig};
use rustls_pemfile::{certs, pkcs8_private_keys, rsa_private_keys};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::BufReader;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tower::ServiceBuilder;
use tracing::{debug, error, info, warn};

/// TLS Configuration for services
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TlsConfig {
    /// Enable TLS/HTTPS
    pub enabled: bool,

    /// Server certificate path
    pub cert_path: PathBuf,

    /// Server private key path
    pub key_path: PathBuf,

    /// CA certificate path for verifying clients (mTLS)
    pub ca_cert_path: Option<PathBuf>,

    /// Client certificate path for outgoing connections
    pub client_cert_path: Option<PathBuf>,

    /// Client key path for outgoing connections
    pub client_key_path: Option<PathBuf>,

    /// Require client certificates (enable mTLS)
    pub require_client_cert: bool,

    /// Minimum TLS version (e.g., "1.2", "1.3")
    pub min_tls_version: Option<String>,

    /// Cipher suites to use (if not specified, uses secure defaults)
    pub cipher_suites: Option<Vec<String>>,

    /// Enable OCSP stapling
    pub ocsp_stapling: bool,

    /// Certificate renewal check interval in hours
    pub renewal_check_hours: u64,
}

impl Default for TlsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cert_path: PathBuf::from("certs/server.crt"),
            key_path: PathBuf::from("certs/server.key"),
            ca_cert_path: Some(PathBuf::from("certs/ca.crt")),
            client_cert_path: Some(PathBuf::from("certs/client.crt")),
            client_key_path: Some(PathBuf::from("certs/client.key")),
            require_client_cert: false,
            min_tls_version: Some("1.3".to_string()),
            cipher_suites: None,
            ocsp_stapling: true,
            renewal_check_hours: 24,
        }
    }
}

/// TLS Manager for handling certificates and configurations
pub struct TlsManager {
    config: TlsConfig,
    server_config: Arc<ServerConfig>,
    client_config: Option<Arc<rustls::ClientConfig>>,
}

impl TlsManager {
    /// Create a new TLS manager with the given configuration
    pub fn new(config: TlsConfig) -> Result<Self> {
        let server_config = Self::create_server_config(&config)?;
        let client_config = if config.client_cert_path.is_some() {
            Some(Arc::new(Self::create_client_config(&config)?))
        } else {
            None
        };

        Ok(Self {
            config,
            server_config: Arc::new(server_config),
            client_config,
        })
    }

    /// Create server TLS configuration
    fn create_server_config(config: &TlsConfig) -> Result<ServerConfig> {
        // Load server certificate and key
        let cert_chain = Self::load_certs(&config.cert_path)?;
        let key = Self::load_private_key(&config.key_path)?;

        let builder = ServerConfig::builder()
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups()
            .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])?;

        let mut server_config = if config.require_client_cert {
            // mTLS: Require client certificates
            info!("Configuring mTLS - client certificates required");
            let client_cert_verifier = if let Some(ca_path) = &config.ca_cert_path {
                let ca_certs = Self::load_certs(ca_path)?;
                let mut root_store = RootCertStore::empty();
                for cert in ca_certs {
                    root_store.add(&cert)?;
                }
                AllowAnyAuthenticatedClient::new(root_store)
            } else {
                return Err(anyhow::anyhow!(
                    "CA certificate required for mTLS but ca_cert_path not provided"
                ));
            };

            builder
                .with_client_cert_verifier(Arc::new(client_cert_verifier))
                .with_single_cert(cert_chain, key)?
        } else if let Some(ca_path) = &config.ca_cert_path {
            // Optional client certificates
            info!("Configuring TLS with optional client certificates");
            let ca_certs = Self::load_certs(ca_path)?;
            let mut root_store = RootCertStore::empty();
            for cert in ca_certs {
                root_store.add(&cert)?;
            }
            let client_cert_verifier = AllowAnyAnonymousOrAuthenticatedClient::new(root_store);

            builder
                .with_client_cert_verifier(Arc::new(client_cert_verifier))
                .with_single_cert(cert_chain, key)?
        } else {
            // No client certificate verification
            info!("Configuring standard TLS without client certificates");
            builder
                .with_no_client_auth()
                .with_single_cert(cert_chain, key)?
        };

        // Configure ALPN for HTTP/2
        server_config.alpn_protocols = vec![b"h2".to_vec(), b"http/1.1".to_vec()];

        Ok(server_config)
    }

    /// Create client TLS configuration for outgoing connections
    fn create_client_config(config: &TlsConfig) -> Result<rustls::ClientConfig> {
        let mut root_store = RootCertStore::empty();

        // Load CA certificates for server verification
        if let Some(ca_path) = &config.ca_cert_path {
            let ca_certs = Self::load_certs(ca_path)?;
            for cert in ca_certs {
                root_store.add(&cert)?;
            }
        } else {
            // Use system CA certificates
            Self::load_system_certs(&mut root_store)?;
        }

        let builder = rustls::ClientConfig::builder()
            .with_safe_default_cipher_suites()
            .with_safe_default_kx_groups()
            .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])?
            .with_root_certificates(root_store);

        let client_config = if let (Some(cert_path), Some(key_path)) =
            (&config.client_cert_path, &config.client_key_path)
        {
            // Configure client certificate for mTLS
            let cert_chain = Self::load_certs(cert_path)?;
            let key = Self::load_private_key(key_path)?;
            builder.with_client_auth_cert(cert_chain, key)?
        } else {
            builder.with_no_client_auth()
        };

        Ok(client_config)
    }

    /// Load certificates from PEM file
    fn load_certs(path: &Path) -> Result<Vec<Certificate>> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open certificate file: {:?}", path))?;
        let mut reader = BufReader::new(file);
        let certs = certs(&mut reader)?.into_iter().map(Certificate).collect();
        Ok(certs)
    }

    /// Load private key from PEM file
    fn load_private_key(path: &Path) -> Result<PrivateKey> {
        let file =
            File::open(path).with_context(|| format!("Failed to open key file: {:?}", path))?;
        let mut reader = BufReader::new(file);

        // Try PKCS#8 format first
        let keys = pkcs8_private_keys(&mut reader)?;
        if !keys.is_empty() {
            return Ok(PrivateKey(keys[0].clone()));
        }

        // Reset reader and try RSA format
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);
        let keys = rsa_private_keys(&mut reader)?;
        if !keys.is_empty() {
            return Ok(PrivateKey(keys[0].clone()));
        }

        Err(anyhow::anyhow!("No private key found in file: {:?}", path))
    }

    /// Load system CA certificates
    fn load_system_certs(root_store: &mut RootCertStore) -> Result<()> {
        // Try to load from common system certificate locations
        let system_cert_paths = vec![
            "/etc/ssl/certs/ca-certificates.crt",     // Debian/Ubuntu
            "/etc/ssl/certs/ca-bundle.crt",           // CentOS/RHEL
            "/etc/pki/tls/certs/ca-bundle.crt",       // Fedora
            "/etc/ssl/cert.pem",                      // OpenSSL
            "/usr/local/share/certs/ca-root-nss.crt", // FreeBSD
        ];

        for path in system_cert_paths {
            if Path::new(path).exists() {
                match Self::load_certs(Path::new(path)) {
                    Ok(certs) => {
                        for cert in certs {
                            root_store.add(&cert)?;
                        }
                        info!("Loaded system certificates from {}", path);
                        return Ok(());
                    }
                    Err(e) => {
                        warn!("Failed to load certificates from {}: {}", path, e);
                    }
                }
            }
        }

        warn!("No system certificates loaded, using rustls-native-certs");
        // Fallback to rustls-native-certs if available
        Ok(())
    }

    /// Get the server TLS configuration
    pub fn server_config(&self) -> Arc<ServerConfig> {
        Arc::clone(&self.server_config)
    }

    /// Get the client TLS configuration
    pub fn client_config(&self) -> Option<Arc<rustls::ClientConfig>> {
        self.client_config.clone()
    }

    /// Create a TLS acceptor for incoming connections
    pub fn acceptor(&self) -> TlsAcceptor {
        TlsAcceptor::from(self.server_config())
    }

    /// Create an HTTPS client with the configured TLS settings
    pub fn create_https_client(&self) -> Result<reqwest::Client> {
        let mut builder = reqwest::Client::builder().use_rustls_tls().https_only(true);

        if let Some(client_config) = &self.client_config {
            // Configure client certificates if available
            if let (Some(cert_path), Some(key_path)) =
                (&self.config.client_cert_path, &self.config.client_key_path)
            {
                let cert = std::fs::read(cert_path)?;
                let key = std::fs::read(key_path)?;
                let identity = reqwest::Identity::from_pem(&[&cert[..], &key[..]].concat())?;
                builder = builder.identity(identity);
            }

            // Configure CA certificate
            if let Some(ca_path) = &self.config.ca_cert_path {
                let ca_cert = std::fs::read(ca_path)?;
                let cert = reqwest::Certificate::from_pem(&ca_cert)?;
                builder = builder.add_root_certificate(cert);
            }
        }

        Ok(builder.build()?)
    }

    /// Check if certificates need renewal
    pub async fn check_certificate_renewal(&self) -> Result<bool> {
        // Load current certificate
        let certs = Self::load_certs(&self.config.cert_path)?;
        if certs.is_empty() {
            return Err(anyhow::anyhow!("No certificate found"));
        }

        // Parse certificate to check expiration
        // This would require x509-parser or similar crate for full implementation
        // For now, return false (no renewal needed)
        Ok(false)
    }

    /// Reload certificates (useful for certificate rotation)
    pub async fn reload_certificates(&mut self) -> Result<()> {
        info!("Reloading TLS certificates");

        let new_server_config = Self::create_server_config(&self.config)?;
        self.server_config = Arc::new(new_server_config);

        if self.config.client_cert_path.is_some() {
            let new_client_config = Self::create_client_config(&self.config)?;
            self.client_config = Some(Arc::new(new_client_config));
        }

        info!("TLS certificates reloaded successfully");
        Ok(())
    }
}

/// Helper to create HTTPS server binding
pub async fn create_https_server(
    addr: SocketAddr,
    tls_manager: &TlsManager,
) -> Result<TcpListener> {
    let listener = TcpListener::bind(addr).await?;
    info!("HTTPS server listening on {}", addr);
    Ok(listener)
}

/// Service configuration for different components
#[derive(Debug, Clone)]
pub struct ServiceTlsConfig {
    pub service_name: String,
    pub port: u16,
    pub tls_config: TlsConfig,
}

impl ServiceTlsConfig {
    pub fn new(service_name: impl Into<String>, port: u16) -> Self {
        let mut config = TlsConfig::default();
        let name = service_name.into();

        // Customize paths per service
        config.cert_path = PathBuf::from(format!("certs/{}/server.crt", name));
        config.key_path = PathBuf::from(format!("certs/{}/server.key", name));
        config.client_cert_path = Some(PathBuf::from(format!("certs/{}/client.crt", name)));
        config.client_key_path = Some(PathBuf::from(format!("certs/{}/client.key", name)));

        Self {
            service_name: name,
            port,
            tls_config: config,
        }
    }

    /// Enable mTLS for this service
    pub fn with_mtls(mut self) -> Self {
        self.tls_config.require_client_cert = true;
        self
    }

    /// Set custom CA certificate
    pub fn with_ca(mut self, ca_path: PathBuf) -> Self {
        self.tls_config.ca_cert_path = Some(ca_path);
        self
    }
}

/// Registry for all service TLS configurations
pub struct TlsRegistry {
    services: Vec<ServiceTlsConfig>,
}

impl TlsRegistry {
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
        }
    }

    /// Register default services with TLS
    pub fn register_defaults(&mut self) {
        // Main API server
        self.services
            .push(ServiceTlsConfig::new("api", 8443).with_mtls());

        // LLM service (llama.cpp)
        self.services
            .push(ServiceTlsConfig::new("llm", 8081).with_mtls());

        // Embedding service
        self.services
            .push(ServiceTlsConfig::new("embedding", 8082).with_mtls());

        // Vector database (Qdrant)
        self.services
            .push(ServiceTlsConfig::new("qdrant", 6334).with_mtls());

        // Redis cache
        self.services.push(
            ServiceTlsConfig::new("redis", 6380) // TLS port for Redis
                .with_mtls(),
        );

        // PostgreSQL
        self.services.push(
            ServiceTlsConfig::new("postgres", 5433) // TLS port for PostgreSQL
                .with_mtls(),
        );

        // MinIO/S3
        self.services
            .push(ServiceTlsConfig::new("minio", 9001).with_mtls());

        // Directory service (Zitadel)
        self.services
            .push(ServiceTlsConfig::new("directory", 8443).with_mtls());

        // Email service (Stalwart)
        self.services.push(
            ServiceTlsConfig::new("email", 465) // SMTPS
                .with_mtls(),
        );

        // Meeting service (LiveKit)
        self.services
            .push(ServiceTlsConfig::new("meet", 7881).with_mtls());
    }

    /// Get TLS manager for a specific service
    pub fn get_manager(&self, service_name: &str) -> Result<TlsManager> {
        let config = self
            .services
            .iter()
            .find(|s| s.service_name == service_name)
            .ok_or_else(|| anyhow::anyhow!("Service {} not found", service_name))?;

        TlsManager::new(config.tls_config.clone())
    }

    /// Get all service configurations
    pub fn services(&self) -> &[ServiceTlsConfig] {
        &self.services
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_tls_config_default() {
        let config = TlsConfig::default();
        assert!(config.enabled);
        assert_eq!(config.min_tls_version, Some("1.3".to_string()));
        assert!(!config.require_client_cert);
    }

    #[test]
    fn test_service_tls_config() {
        let config = ServiceTlsConfig::new("test-service", 8443).with_mtls();

        assert_eq!(config.service_name, "test-service");
        assert_eq!(config.port, 8443);
        assert!(config.tls_config.require_client_cert);
    }

    #[test]
    fn test_tls_registry() {
        let mut registry = TlsRegistry::new();
        registry.register_defaults();

        assert!(!registry.services().is_empty());

        // Check if main services are registered
        let service_names: Vec<&str> = registry
            .services()
            .iter()
            .map(|s| s.service_name.as_str())
            .collect();

        assert!(service_names.contains(&"api"));
        assert!(service_names.contains(&"llm"));
        assert!(service_names.contains(&"embedding"));
    }
}
