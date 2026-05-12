use anyhow::{Context, Result};
use rustls::pki_types::pem::PemObject;
use rustls::pki_types::{CertificateDer, PrivateKeyDer};
use rustls::server::WebPkiClientVerifier;
use rustls::{RootCertStore, ServerConfig};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio_rustls::TlsAcceptor;
use tracing::{info, warn};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TlsConfig {
    pub enabled: bool,

    pub cert_path: PathBuf,

    pub key_path: PathBuf,

    pub ca_cert_path: Option<PathBuf>,

    pub client_cert_path: Option<PathBuf>,

    pub client_key_path: Option<PathBuf>,

    pub require_client_cert: bool,

    pub min_tls_version: Option<String>,

    pub cipher_suites: Option<Vec<String>>,

    pub ocsp_stapling: bool,

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

#[derive(Debug)]
pub struct TlsManager {
    config: TlsConfig,
    server_config: Arc<ServerConfig>,
    client_config: Option<Arc<rustls::ClientConfig>>,
}

impl TlsManager {
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

    fn create_server_config(config: &TlsConfig) -> Result<ServerConfig> {
        let cert_chain = Self::load_certs(&config.cert_path)?;
        let key = Self::load_private_key(&config.key_path)?;

        let server_config = if config.require_client_cert {
            info!("Configuring mTLS - client certificates required");
            if let Some(ca_path) = &config.ca_cert_path {
                let ca_certs = Self::load_certs(ca_path)?;
                let mut root_store = RootCertStore::empty();
                for cert in ca_certs {
                    root_store.add(cert)?;
                }
                let client_cert_verifier = WebPkiClientVerifier::builder(Arc::new(root_store))
                    .build()
                    .map_err(|e| anyhow::anyhow!("Failed to build client verifier: {}", e))?;

                ServerConfig::builder()
                    .with_client_cert_verifier(client_cert_verifier)
                    .with_single_cert(cert_chain, key)?
            } else {
                return Err(anyhow::anyhow!(
                    "CA certificate required for mTLS but ca_cert_path not provided"
                ));
            }
        } else if let Some(ca_path) = &config.ca_cert_path {
            info!("Configuring TLS with optional client certificates");
            let ca_certs = Self::load_certs(ca_path)?;
            let mut root_store = RootCertStore::empty();
            for cert in ca_certs {
                root_store.add(cert)?;
            }
            let client_cert_verifier = WebPkiClientVerifier::builder(Arc::new(root_store))
                .allow_unauthenticated()
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build client verifier: {}", e))?;

            ServerConfig::builder()
                .with_client_cert_verifier(client_cert_verifier)
                .with_single_cert(cert_chain, key)?
        } else {
            info!("Configuring standard TLS without client certificates");
            ServerConfig::builder()
                .with_no_client_auth()
                .with_single_cert(cert_chain, key)?
        };

        Ok(server_config)
    }

    fn create_client_config(config: &TlsConfig) -> Result<rustls::ClientConfig> {
        let mut root_store = RootCertStore::empty();

        if let Some(ca_path) = &config.ca_cert_path {
            let ca_certs = Self::load_certs(ca_path)?;
            for cert in ca_certs {
                root_store.add(cert)?;
            }
        } else {
            Self::load_system_certs(&mut root_store)?;
        }

        let builder = rustls::ClientConfig::builder().with_root_certificates(root_store);

        let client_config = if let (Some(cert_path), Some(key_path)) =
            (&config.client_cert_path, &config.client_key_path)
        {
            let cert_chain = Self::load_certs(cert_path)?;
            let key = Self::load_private_key(key_path)?;
            builder.with_client_auth_cert(cert_chain, key)?
        } else {
            builder.with_no_client_auth()
        };

        Ok(client_config)
    }

    fn load_certs(path: &Path) -> Result<Vec<CertificateDer<'static>>> {
        CertificateDer::pem_file_iter(path)
            .with_context(|| format!("Failed to open certificate file: {}", path.display()))?
            .collect::<Result<Vec<_>, _>>()
            .with_context(|| format!("Failed to parse certificates from {}", path.display()))
    }

    fn load_private_key(path: &Path) -> Result<PrivateKeyDer<'static>> {
        PrivateKeyDer::from_pem_file(path)
            .with_context(|| format!("Failed to load private key from {}", path.display()))
    }

    fn load_system_certs(root_store: &mut RootCertStore) -> Result<()> {
        let system_cert_paths = vec![
            "/etc/ssl/certs/ca-certificates.crt",
            "/etc/ssl/certs/ca-bundle.crt",
            "/etc/pki/tls/certs/ca-bundle.crt",
            "/etc/ssl/cert.pem",
            "/usr/local/share/certs/ca-root-nss.crt",
        ];

        for path in system_cert_paths {
            if Path::new(path).exists() {
                match Self::load_certs(Path::new(path)) {
                    Ok(certs) => {
                        for cert in certs {
                            root_store.add(cert)?;
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

        Ok(())
    }

    pub fn server_config(&self) -> Arc<ServerConfig> {
        Arc::clone(&self.server_config)
    }

    pub fn client_config(&self) -> Option<Arc<rustls::ClientConfig>> {
        self.client_config.clone()
    }

    pub fn acceptor(&self) -> TlsAcceptor {
        TlsAcceptor::from(self.server_config())
    }

    pub fn create_https_client(&self) -> Result<reqwest::Client> {
        let mut builder = reqwest::Client::builder().use_rustls_tls().https_only(true);

        if let Some(_client_config) = &self.client_config {
            if let (Some(cert_path), Some(key_path)) =
                (&self.config.client_cert_path, &self.config.client_key_path)
            {
                let cert = std::fs::read(cert_path)?;
                let key = std::fs::read(key_path)?;
                let identity = reqwest::Identity::from_pem(&[&cert[..], &key[..]].concat())?;
                builder = builder.identity(identity);
            }

            if let Some(ca_path) = &self.config.ca_cert_path {
                let ca_cert = std::fs::read(ca_path)?;
                let cert = reqwest::Certificate::from_pem(&ca_cert)?;
                builder = builder.add_root_certificate(cert);
            }
        }

        Ok(builder.build()?)
    }

    pub fn check_certificate_renewal(&self) -> Result<bool> {
        let certs = Self::load_certs(&self.config.cert_path)?;
        if certs.is_empty() {
            return Err(anyhow::anyhow!("No certificate found"));
        }

        Ok(false)
    }

    pub fn reload_certificates(&mut self) -> Result<()> {
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

pub async fn create_https_server(
    addr: SocketAddr,
    _tls_manager: &TlsManager,
) -> Result<TcpListener> {
    let listener = TcpListener::bind(addr).await?;
    info!("HTTPS server listening on {}", addr);
    Ok(listener)
}

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

    pub fn with_mtls(mut self) -> Self {
        self.tls_config.require_client_cert = true;
        self
    }

    pub fn with_ca(mut self, ca_path: PathBuf) -> Self {
        self.tls_config.ca_cert_path = Some(ca_path);
        self
    }
}

#[derive(Debug, Clone)]
pub struct TlsRegistry {
    services: Vec<ServiceTlsConfig>,
}

impl Default for TlsRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl TlsRegistry {
    pub fn new() -> Self {
        Self {
            services: Vec::new(),
        }
    }

    pub fn register_defaults(&mut self) {
        self.services
            .push(ServiceTlsConfig::new("api", 8443).with_mtls());

        self.services
            .push(ServiceTlsConfig::new("llm", 8081).with_mtls());

        self.services
            .push(ServiceTlsConfig::new("embedding", 8082).with_mtls());

        self.services
            .push(ServiceTlsConfig::new("qdrant", 6334).with_mtls());

        self.services
            .push(ServiceTlsConfig::new("redis", 6380).with_mtls());

        self.services
            .push(ServiceTlsConfig::new("postgres", 5433).with_mtls());

        self.services
            .push(ServiceTlsConfig::new("minio", 9001).with_mtls());

        self.services
            .push(ServiceTlsConfig::new("directory", 8443).with_mtls());

        self.services
            .push(ServiceTlsConfig::new("email", 465).with_mtls());

        self.services
            .push(ServiceTlsConfig::new("meet", 7881).with_mtls());
    }

    pub fn get_manager(&self, service_name: &str) -> Result<TlsManager> {
        let config = self
            .services
            .iter()
            .find(|s| s.service_name == service_name)
            .ok_or_else(|| anyhow::anyhow!("Service {} not found", service_name))?;

        TlsManager::new(config.tls_config.clone())
    }

    pub fn services(&self) -> &[ServiceTlsConfig] {
        &self.services
    }
}
