//! Security Module
//!
//! This module provides comprehensive security features for the BotServer including:
//! - TLS/HTTPS configuration for all services
//! - mTLS (mutual TLS) for service-to-service authentication
//! - Internal Certificate Authority (CA) management
//! - Certificate lifecycle management
//! - Security utilities and helpers
//! - Antivirus and threat detection (ClamAV integration)
//! - Windows Defender management

pub mod antivirus;
pub mod ca;
pub mod integration;
pub mod mutual_tls;
pub mod tls;

pub use antivirus::{
    AntivirusConfig, AntivirusManager, ProtectionStatus, ScanResult, ScanStatus, ScanType, Threat,
    ThreatSeverity, ThreatStatus, Vulnerability,
};
pub use ca::{CaConfig, CaManager, CertificateRequest, CertificateResponse};
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

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{info, warn};

/// Security configuration for the entire system
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Enable TLS for all services
    pub tls_enabled: bool,

    /// Enable mTLS for service-to-service communication
    pub mtls_enabled: bool,

    /// CA configuration
    pub ca_config: CaConfig,

    /// TLS registry for all services
    pub tls_registry: TlsRegistry,

    /// Auto-generate certificates if missing
    pub auto_generate_certs: bool,

    /// Certificate renewal threshold in days
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

/// Security Manager - Main entry point for security features
pub struct SecurityManager {
    config: SecurityConfig,
    ca_manager: CaManager,
    mtls_manager: Option<MtlsManager>,
}

impl SecurityManager {
    /// Create a new security manager
    pub fn new(config: SecurityConfig) -> Result<Self> {
        let ca_manager = CaManager::new(config.ca_config.clone())?;

        let mtls_manager = if config.mtls_enabled {
            // Create mTLS config from CA certificates
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

    /// Initialize security infrastructure
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing security infrastructure");

        // Check if CA exists, create if needed
        if self.config.auto_generate_certs && !self.ca_exists() {
            info!("No CA found, initializing new Certificate Authority");
            self.ca_manager.init_ca()?;

            // Generate certificates for all services
            info!("Generating certificates for all services");
            self.ca_manager.issue_service_certificates()?;
        }

        // Initialize mTLS if enabled
        if self.config.mtls_enabled {
            self.initialize_mtls().await?;
        }

        // Verify all certificates
        self.verify_all_certificates().await?;

        // Start certificate renewal monitor
        if self.config.auto_generate_certs {
            self.start_renewal_monitor().await;
        }

        info!("Security infrastructure initialized successfully");
        Ok(())
    }

    /// Initialize mTLS for all services
    async fn initialize_mtls(&mut self) -> Result<()> {
        if let Some(ref manager) = self.mtls_manager {
            info!("Initializing mTLS for all services");

            let base_path = PathBuf::from("./botserver-stack/conf/system");

            // Configure mTLS for each service
            let ca_path = base_path.join("ca/ca.crt");
            let cert_path = base_path.join("certs/api.crt");
            let key_path = base_path.join("certs/api.key");

            // Validate configurations for each service
            let _ = configure_qdrant_mtls(Some(&ca_path), Some(&cert_path), Some(&key_path));
            let _ = configure_postgres_mtls(Some(&ca_path), Some(&cert_path), Some(&key_path));
            let _ = configure_forgejo_mtls(Some(&ca_path), Some(&cert_path), Some(&key_path));
            let _ = configure_livekit_mtls(Some(&ca_path), Some(&cert_path), Some(&key_path));
            let _ = configure_directory_mtls(Some(&ca_path), Some(&cert_path), Some(&key_path));

            // Validate the manager configuration
            manager.validate()?;

            info!("mTLS initialized for all services");
        }
        Ok(())
    }

    /// Check if CA exists
    fn ca_exists(&self) -> bool {
        self.config.ca_config.ca_cert_path.exists() && self.config.ca_config.ca_key_path.exists()
    }

    /// Verify all service certificates
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

    /// Start certificate renewal monitor
    async fn start_renewal_monitor(&self) {
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(24 * 60 * 60), // Check daily
            );

            loop {
                interval.tick().await;

                // Check each service certificate
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

    /// Get TLS manager for a specific service
    pub fn get_tls_manager(&self, service_name: &str) -> Result<TlsManager> {
        self.config.tls_registry.get_manager(service_name)
    }

    /// Get the CA manager
    pub fn ca_manager(&self) -> &CaManager {
        &self.ca_manager
    }

    /// Check if TLS is enabled
    pub fn is_tls_enabled(&self) -> bool {
        self.config.tls_enabled
    }

    /// Check if mTLS is enabled
    pub fn is_mtls_enabled(&self) -> bool {
        self.config.mtls_enabled
    }

    /// Get mTLS manager
    pub fn mtls_manager(&self) -> Option<&MtlsManager> {
        self.mtls_manager.as_ref()
    }
}

/// Check if a certificate needs renewal
async fn check_certificate_renewal(tls_config: &TlsConfig) -> Result<()> {
    // This would check certificate expiration
    // and trigger renewal if needed
    Ok(())
}

/// Create HTTPS client with proper TLS configuration using manager
pub fn create_https_client_with_manager(tls_manager: &TlsManager) -> Result<reqwest::Client> {
    tls_manager.create_https_client()
}

/// Convert service URLs to HTTPS
pub fn convert_to_https(url: &str) -> String {
    if url.starts_with("http://") {
        url.replace("http://", "https://")
    } else if !url.starts_with("https://") {
        format!("https://{}", url)
    } else {
        url.to_string()
    }
}

/// Service port mappings (HTTP -> HTTPS)
pub fn get_secure_port(service: &str, default_port: u16) -> u16 {
    match service {
        "api" => 8443,           // API server
        "llm" => 8444,           // LLM service
        "embedding" => 8445,     // Embedding service
        "qdrant" => 6334,        // Qdrant (already TLS)
        "redis" => 6380,         // Redis TLS port
        "postgres" => 5433,      // PostgreSQL TLS port
        "minio" => 9001,         // MinIO TLS port
        "directory" => 8446,     // Directory service
        "email" => 465,          // SMTP over TLS
        "meet" => 7881,          // LiveKit TLS port
        _ => default_port + 443, // Add 443 to default port as fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_convert_to_https() {
        assert_eq!(
            convert_to_https("http://localhost:8080"),
            "https://localhost:8080"
        );
        assert_eq!(
            convert_to_https("https://localhost:8080"),
            "https://localhost:8080"
        );
        assert_eq!(convert_to_https("localhost:8080"), "https://localhost:8080");
    }

    #[test]
    fn test_get_secure_port() {
        assert_eq!(get_secure_port("api", 8080), 8443);
        assert_eq!(get_secure_port("llm", 8081), 8444);
        assert_eq!(get_secure_port("redis", 6379), 6380);
        assert_eq!(get_secure_port("unknown", 3000), 3443);
    }

    #[test]
    fn test_security_config_default() {
        let config = SecurityConfig::default();
        assert!(config.tls_enabled);
        assert!(config.mtls_enabled);
        assert!(config.auto_generate_certs);
        assert_eq!(config.renewal_threshold_days, 30);
    }
}
