//! Mutual TLS (mTLS) Module
//!
//! This module provides mutual TLS authentication for service-to-service communication.
//! It enables secure connections between BotServer and its dependent services like
//! PostgreSQL, Qdrant, LiveKit, Forgejo, and Directory services.

use std::path::Path;
use tracing::{debug, info};

/// Services module containing mTLS configuration functions for each service
pub mod services {
    use super::*;

    /// Configure mTLS for PostgreSQL connections
    ///
    /// # Arguments
    /// * `ca_cert_path` - Path to the CA certificate
    /// * `client_cert_path` - Path to the client certificate
    /// * `client_key_path` - Path to the client private key
    ///
    /// # Returns
    /// Result containing the SSL mode string or an error
    pub fn configure_postgres_mtls(
        ca_cert_path: Option<&Path>,
        client_cert_path: Option<&Path>,
        client_key_path: Option<&Path>,
    ) -> Result<String, MtlsError> {
        match (ca_cert_path, client_cert_path, client_key_path) {
            (Some(ca), Some(cert), Some(key)) => {
                if !ca.exists() {
                    return Err(MtlsError::CertificateNotFound(
                        ca.to_string_lossy().to_string(),
                    ));
                }
                if !cert.exists() {
                    return Err(MtlsError::CertificateNotFound(
                        cert.to_string_lossy().to_string(),
                    ));
                }
                if !key.exists() {
                    return Err(MtlsError::KeyNotFound(key.to_string_lossy().to_string()));
                }

                info!("PostgreSQL mTLS configured with client certificates");
                Ok("verify-full".to_string())
            }
            (Some(ca), None, None) => {
                if !ca.exists() {
                    return Err(MtlsError::CertificateNotFound(
                        ca.to_string_lossy().to_string(),
                    ));
                }
                info!("PostgreSQL TLS configured with CA verification only");
                Ok("verify-ca".to_string())
            }
            _ => {
                debug!("PostgreSQL mTLS not configured, using default connection");
                Ok("prefer".to_string())
            }
        }
    }

    /// Configure mTLS for Qdrant vector database connections
    ///
    /// # Arguments
    /// * `ca_cert_path` - Path to the CA certificate
    /// * `client_cert_path` - Path to the client certificate
    /// * `client_key_path` - Path to the client private key
    ///
    /// # Returns
    /// Result containing the mTLS configuration or an error
    pub fn configure_qdrant_mtls(
        ca_cert_path: Option<&Path>,
        client_cert_path: Option<&Path>,
        client_key_path: Option<&Path>,
    ) -> Result<MtlsConfig, MtlsError> {
        match (ca_cert_path, client_cert_path, client_key_path) {
            (Some(ca), Some(cert), Some(key)) => {
                let ca_pem = std::fs::read_to_string(ca)
                    .map_err(|e| MtlsError::IoError(format!("Failed to read CA cert: {}", e)))?;
                let cert_pem = std::fs::read_to_string(cert).map_err(|e| {
                    MtlsError::IoError(format!("Failed to read client cert: {}", e))
                })?;
                let key_pem = std::fs::read_to_string(key)
                    .map_err(|e| MtlsError::IoError(format!("Failed to read client key: {}", e)))?;

                info!("Qdrant mTLS configured successfully");
                Ok(MtlsConfig {
                    enabled: true,
                    ca_cert: Some(ca_pem),
                    client_cert: Some(cert_pem),
                    client_key: Some(key_pem),
                })
            }
            _ => {
                debug!("Qdrant mTLS not configured");
                Ok(MtlsConfig::default())
            }
        }
    }

    /// Configure mTLS for LiveKit media server connections
    ///
    /// # Arguments
    /// * `ca_cert_path` - Path to the CA certificate
    /// * `client_cert_path` - Path to the client certificate
    /// * `client_key_path` - Path to the client private key
    ///
    /// # Returns
    /// Result containing the mTLS configuration or an error
    pub fn configure_livekit_mtls(
        ca_cert_path: Option<&Path>,
        client_cert_path: Option<&Path>,
        client_key_path: Option<&Path>,
    ) -> Result<MtlsConfig, MtlsError> {
        match (ca_cert_path, client_cert_path, client_key_path) {
            (Some(ca), Some(cert), Some(key)) => {
                let ca_pem = std::fs::read_to_string(ca)
                    .map_err(|e| MtlsError::IoError(format!("Failed to read CA cert: {}", e)))?;
                let cert_pem = std::fs::read_to_string(cert).map_err(|e| {
                    MtlsError::IoError(format!("Failed to read client cert: {}", e))
                })?;
                let key_pem = std::fs::read_to_string(key)
                    .map_err(|e| MtlsError::IoError(format!("Failed to read client key: {}", e)))?;

                info!("LiveKit mTLS configured successfully");
                Ok(MtlsConfig {
                    enabled: true,
                    ca_cert: Some(ca_pem),
                    client_cert: Some(cert_pem),
                    client_key: Some(key_pem),
                })
            }
            _ => {
                debug!("LiveKit mTLS not configured");
                Ok(MtlsConfig::default())
            }
        }
    }

    /// Configure mTLS for Forgejo git server connections
    ///
    /// # Arguments
    /// * `ca_cert_path` - Path to the CA certificate
    /// * `client_cert_path` - Path to the client certificate
    /// * `client_key_path` - Path to the client private key
    ///
    /// # Returns
    /// Result containing the mTLS configuration or an error
    pub fn configure_forgejo_mtls(
        ca_cert_path: Option<&Path>,
        client_cert_path: Option<&Path>,
        client_key_path: Option<&Path>,
    ) -> Result<MtlsConfig, MtlsError> {
        match (ca_cert_path, client_cert_path, client_key_path) {
            (Some(ca), Some(cert), Some(key)) => {
                let ca_pem = std::fs::read_to_string(ca)
                    .map_err(|e| MtlsError::IoError(format!("Failed to read CA cert: {}", e)))?;
                let cert_pem = std::fs::read_to_string(cert).map_err(|e| {
                    MtlsError::IoError(format!("Failed to read client cert: {}", e))
                })?;
                let key_pem = std::fs::read_to_string(key)
                    .map_err(|e| MtlsError::IoError(format!("Failed to read client key: {}", e)))?;

                info!("Forgejo mTLS configured successfully");
                Ok(MtlsConfig {
                    enabled: true,
                    ca_cert: Some(ca_pem),
                    client_cert: Some(cert_pem),
                    client_key: Some(key_pem),
                })
            }
            _ => {
                debug!("Forgejo mTLS not configured");
                Ok(MtlsConfig::default())
            }
        }
    }

    /// Configure mTLS for Directory service (LDAP/AD) connections
    ///
    /// # Arguments
    /// * `ca_cert_path` - Path to the CA certificate
    /// * `client_cert_path` - Path to the client certificate
    /// * `client_key_path` - Path to the client private key
    ///
    /// # Returns
    /// Result containing the mTLS configuration or an error
    pub fn configure_directory_mtls(
        ca_cert_path: Option<&Path>,
        client_cert_path: Option<&Path>,
        client_key_path: Option<&Path>,
    ) -> Result<MtlsConfig, MtlsError> {
        match (ca_cert_path, client_cert_path, client_key_path) {
            (Some(ca), Some(cert), Some(key)) => {
                let ca_pem = std::fs::read_to_string(ca)
                    .map_err(|e| MtlsError::IoError(format!("Failed to read CA cert: {}", e)))?;
                let cert_pem = std::fs::read_to_string(cert).map_err(|e| {
                    MtlsError::IoError(format!("Failed to read client cert: {}", e))
                })?;
                let key_pem = std::fs::read_to_string(key)
                    .map_err(|e| MtlsError::IoError(format!("Failed to read client key: {}", e)))?;

                info!("Directory service mTLS configured successfully");
                Ok(MtlsConfig {
                    enabled: true,
                    ca_cert: Some(ca_pem),
                    client_cert: Some(cert_pem),
                    client_key: Some(key_pem),
                })
            }
            _ => {
                debug!("Directory service mTLS not configured");
                Ok(MtlsConfig::default())
            }
        }
    }
}

/// mTLS configuration structure
#[derive(Debug, Clone, Default)]
pub struct MtlsConfig {
    /// Whether mTLS is enabled
    pub enabled: bool,
    /// CA certificate PEM content
    pub ca_cert: Option<String>,
    /// Client certificate PEM content
    pub client_cert: Option<String>,
    /// Client private key PEM content
    pub client_key: Option<String>,
}

impl MtlsConfig {
    /// Create a new mTLS configuration
    pub fn new(
        ca_cert: Option<String>,
        client_cert: Option<String>,
        client_key: Option<String>,
    ) -> Self {
        let enabled = ca_cert.is_some() && client_cert.is_some() && client_key.is_some();
        Self {
            enabled,
            ca_cert,
            client_cert,
            client_key,
        }
    }

    /// Check if mTLS is properly configured
    pub fn is_configured(&self) -> bool {
        self.enabled
            && self.ca_cert.is_some()
            && self.client_cert.is_some()
            && self.client_key.is_some()
    }
}

/// mTLS error types
#[derive(Debug, thiserror::Error)]
pub enum MtlsError {
    #[error("Certificate not found: {0}")]
    CertificateNotFound(String),

    #[error("Private key not found: {0}")]
    KeyNotFound(String),

    #[error("Invalid certificate format: {0}")]
    InvalidCertificate(String),

    #[error("Invalid key format: {0}")]
    InvalidKey(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("TLS configuration error: {0}")]
    TlsConfigError(String),
}

/// mTLS Manager for handling mutual TLS connections
#[derive(Debug)]
pub struct MtlsManager {
    config: MtlsConfig,
}

impl MtlsManager {
    /// Create a new mTLS manager
    pub fn new(config: MtlsConfig) -> Self {
        Self { config }
    }

    /// Get the current configuration
    pub fn config(&self) -> &MtlsConfig {
        &self.config
    }

    /// Check if mTLS is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Validate the mTLS configuration
    pub fn validate(&self) -> Result<(), MtlsError> {
        if !self.config.enabled {
            return Ok(());
        }

        // Validate CA certificate
        if let Some(ref ca) = self.config.ca_cert {
            if !ca.contains("-----BEGIN CERTIFICATE-----") {
                return Err(MtlsError::InvalidCertificate(
                    "CA certificate is not in PEM format".to_string(),
                ));
            }
        }

        // Validate client certificate
        if let Some(ref cert) = self.config.client_cert {
            if !cert.contains("-----BEGIN CERTIFICATE-----") {
                return Err(MtlsError::InvalidCertificate(
                    "Client certificate is not in PEM format".to_string(),
                ));
            }
        }

        // Validate client key
        if let Some(ref key) = self.config.client_key {
            if !key.contains("-----BEGIN") || !key.contains("PRIVATE KEY-----") {
                return Err(MtlsError::InvalidKey(
                    "Client key is not in PEM format".to_string(),
                ));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mtls_config_default() {
        let config = MtlsConfig::default();
        assert!(!config.enabled);
        assert!(config.ca_cert.is_none());
        assert!(config.client_cert.is_none());
        assert!(config.client_key.is_none());
    }

    #[test]
    fn test_mtls_config_new() {
        let config = MtlsConfig::new(
            Some("ca_cert".to_string()),
            Some("client_cert".to_string()),
            Some("client_key".to_string()),
        );
        assert!(config.enabled);
        assert!(config.is_configured());
    }

    #[test]
    fn test_mtls_config_partial() {
        let config = MtlsConfig::new(Some("ca_cert".to_string()), None, None);
        assert!(!config.enabled);
        assert!(!config.is_configured());
    }

    #[test]
    fn test_mtls_manager_validation() {
        let config = MtlsConfig {
            enabled: true,
            ca_cert: Some(
                "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----".to_string(),
            ),
            client_cert: Some(
                "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----".to_string(),
            ),
            client_key: Some(
                "-----BEGIN PRIVATE KEY-----\ntest\n-----END PRIVATE KEY-----".to_string(),
            ),
        };
        let manager = MtlsManager::new(config);
        assert!(manager.validate().is_ok());
    }

    #[test]
    fn test_mtls_manager_invalid_cert() {
        let config = MtlsConfig {
            enabled: true,
            ca_cert: Some("invalid".to_string()),
            client_cert: Some(
                "-----BEGIN CERTIFICATE-----\ntest\n-----END CERTIFICATE-----".to_string(),
            ),
            client_key: Some(
                "-----BEGIN PRIVATE KEY-----\ntest\n-----END PRIVATE KEY-----".to_string(),
            ),
        };
        let manager = MtlsManager::new(config);
        assert!(manager.validate().is_err());
    }
}
