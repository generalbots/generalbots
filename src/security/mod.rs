










pub mod antivirus;
pub mod ca;
pub mod cert_pinning;
pub mod integration;
pub mod mutual_tls;
pub mod tls;

pub use antivirus::{
    AntivirusConfig, AntivirusManager, ProtectionStatus, ScanResult, ScanStatus, ScanType, Threat,
    ThreatSeverity, ThreatStatus, Vulnerability,
};
pub use ca::{CaConfig, CaManager, CertificateRequest, CertificateResponse};
pub use cert_pinning::{
    compute_spki_fingerprint, format_fingerprint, parse_fingerprint, CertPinningConfig,
    CertPinningManager, PinType, PinValidationResult, PinnedCert, PinningStats,
};
pub use integration::{
    create_https_client, get_tls_integration, init_tls_integration, to_secure_url, TlsIntegration,
};
pub use mutual_tls::{
    services::{
        configure_directory_mtls, configure_forgejo_mtls, configure_livekit_mtls,
        configure_postgres_mtls, configure_qdrant_mtls,
    },
    MtlsConfig, MtlsError, MtlsManager,
};
pub use tls::{create_https_server, ServiceTlsConfig, TlsConfig, TlsManager, TlsRegistry};

use anyhow::Result;
use std::path::PathBuf;
use tracing::{info, warn};


#[derive(Debug, Clone)]
pub struct SecurityConfig {

    pub tls_enabled: bool,


    pub mtls_enabled: bool,


    pub ca_config: CaConfig,


    pub tls_registry: TlsRegistry,


    pub auto_generate_certs: bool,


    pub renewal_threshold_days: i64,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        let mut tls_registry = TlsRegistry::new();
        tls_registry.register_defaults();

        Self {
            tls_enabled: true,
            mtls_enabled: true,
            ca_config: CaConfig::default(),
            tls_registry,
            auto_generate_certs: true,
            renewal_threshold_days: 30,
        }
    }
}


#[derive(Debug)]
pub struct SecurityManager {
    config: SecurityConfig,
    ca_manager: CaManager,
    mtls_manager: Option<MtlsManager>,
}

impl SecurityManager {

    pub fn new(config: SecurityConfig) -> Result<Self> {
        let ca_manager = CaManager::new(config.ca_config.clone())?;

        let mtls_manager = if config.mtls_enabled {

            let ca_cert = std::fs::read_to_string(&config.ca_config.ca_cert_path).ok();
            let mtls_config = MtlsConfig::new(ca_cert, None, None);
            Some(MtlsManager::new(mtls_config))
        } else {
            None
        };

        Ok(Self {
            config,
            ca_manager,
            mtls_manager,
        })
    }


    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing security infrastructure");


        if self.config.auto_generate_certs && !self.ca_exists() {
            info!("No CA found, initializing new Certificate Authority");
            self.ca_manager.init_ca()?;


            info!("Generating certificates for all services");
            self.ca_manager.issue_service_certificates()?;
        }


        if self.config.mtls_enabled {
            self.initialize_mtls().await?;
        }


        self.verify_all_certificates().await?;


        if self.config.auto_generate_certs {
            self.start_renewal_monitor().await;
        }

        info!("Security infrastructure initialized successfully");
        Ok(())
    }


    async fn initialize_mtls(&mut self) -> Result<()> {
        if let Some(ref manager) = self.mtls_manager {
            info!("Initializing mTLS for all services");

            let base_path = PathBuf::from("./botserver-stack/conf/system");


            let ca_path = base_path.join("ca/ca.crt");
            let cert_path = base_path.join("certs/api.crt");
            let key_path = base_path.join("certs/api.key");


            let _ = configure_qdrant_mtls(Some(&ca_path), Some(&cert_path), Some(&key_path));
            let _ = configure_postgres_mtls(Some(&ca_path), Some(&cert_path), Some(&key_path));
            let _ = configure_forgejo_mtls(Some(&ca_path), Some(&cert_path), Some(&key_path));
            let _ = configure_livekit_mtls(Some(&ca_path), Some(&cert_path), Some(&key_path));
            let _ = configure_directory_mtls(Some(&ca_path), Some(&cert_path), Some(&key_path));


            manager.validate()?;

            info!("mTLS initialized for all services");
        }
        Ok(())
    }


    fn ca_exists(&self) -> bool {
        self.config.ca_config.ca_cert_path.exists() && self.config.ca_config.ca_key_path.exists()
    }


    async fn verify_all_certificates(&self) -> Result<()> {
        for service in self.config.tls_registry.services() {
            let cert_path = &service.tls_config.cert_path;
            let key_path = &service.tls_config.key_path;

            if !cert_path.exists() || !key_path.exists() {
                if self.config.auto_generate_certs {
                    warn!(
                        "Certificate missing for service {}, generating...",
                        service.service_name
                    );
                    self.ca_manager.issue_service_certificate(
                        &service.service_name,
                        vec!["localhost", &service.service_name, "127.0.0.1"],
                    )?;
                } else {
                    return Err(anyhow::anyhow!(
                        "Certificate missing for service {} and auto-generation is disabled",
                        service.service_name
                    ));
                }
            }
        }

        Ok(())
    }


    async fn start_renewal_monitor(&self) {
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(24 * 60 * 60),
            );

            loop {
                interval.tick().await;


                for service in config.tls_registry.services() {
                    if let Err(e) = check_certificate_renewal(&service.tls_config).await {
                        warn!(
                            "Failed to check certificate renewal for {}: {}",
                            service.service_name, e
                        );
                    }
                }
            }
        });
    }


    pub fn get_tls_manager(&self, service_name: &str) -> Result<TlsManager> {
        self.config.tls_registry.get_manager(service_name)
    }


    pub fn ca_manager(&self) -> &CaManager {
        &self.ca_manager
    }


    pub fn is_tls_enabled(&self) -> bool {
        self.config.tls_enabled
    }


    pub fn is_mtls_enabled(&self) -> bool {
        self.config.mtls_enabled
    }


    pub fn mtls_manager(&self) -> Option<&MtlsManager> {
        self.mtls_manager.as_ref()
    }
}


async fn check_certificate_renewal(_tls_config: &TlsConfig) -> Result<()> {


    Ok(())
}


pub fn create_https_client_with_manager(tls_manager: &TlsManager) -> Result<reqwest::Client> {
    tls_manager.create_https_client()
}


pub fn convert_to_https(url: &str) -> String {
    if url.starts_with("http://") {
        url.replace("http://", "https://")
    } else if !url.starts_with("https://") {
        format!("https://{}", url)
    } else {
        url.to_string()
    }
}


pub fn get_secure_port(service: &str, default_port: u16) -> u16 {
    match service {
        "api" => 8443,
        "llm" => 8444,
        "embedding" => 8445,
        "qdrant" => 6334,
        "redis" => 6380,
        "postgres" => 5433,
        "minio" => 9001,
        "directory" => 8446,
        "email" => 465,
        "meet" => 7881,
        _ => default_port + 443,
    }
}
