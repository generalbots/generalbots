





use std::path::Path;
use tracing::{debug, info};


pub mod services {
    use super::*;










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


#[derive(Debug, Clone, Default)]
pub struct MtlsConfig {

    pub enabled: bool,

    pub ca_cert: Option<String>,

    pub client_cert: Option<String>,

    pub client_key: Option<String>,
}

impl MtlsConfig {

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


    pub fn is_configured(&self) -> bool {
        self.enabled
            && self.ca_cert.is_some()
            && self.client_cert.is_some()
            && self.client_key.is_some()
    }
}


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


#[derive(Debug)]
pub struct MtlsManager {
    config: MtlsConfig,
}

impl MtlsManager {

    pub fn new(config: MtlsConfig) -> Self {
        Self { config }
    }


    pub fn config(&self) -> &MtlsConfig {
        &self.config
    }


    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }


    pub fn validate(&self) -> Result<(), MtlsError> {
        if !self.config.enabled {
            return Ok(());
        }


        if let Some(ref ca) = self.config.ca_cert {
            if !ca.contains("-----BEGIN CERTIFICATE-----") {
                return Err(MtlsError::InvalidCertificate(
                    "CA certificate is not in PEM format".to_string(),
                ));
            }
        }


        if let Some(ref cert) = self.config.client_cert {
            if !cert.contains("-----BEGIN CERTIFICATE-----") {
                return Err(MtlsError::InvalidCertificate(
                    "Client certificate is not in PEM format".to_string(),
                ));
            }
        }


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
