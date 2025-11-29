//! Internal Certificate Authority (CA) Management
//!
//! This module provides functionality for managing an internal CA
//! with support for external CA integration.

use anyhow::{Context, Result};
use rcgen::{
    BasicConstraints, Certificate as RcgenCertificate, CertificateParams, DistinguishedName,
    DnType, IsCa, KeyPair, SanType,
};
use rustls::{Certificate, PrivateKey};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use time::{Duration, OffsetDateTime};
use tracing::{debug, info, warn};

/// CA Configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CaConfig {
    /// CA root certificate path
    pub ca_cert_path: PathBuf,

    /// CA private key path
    pub ca_key_path: PathBuf,

    /// Intermediate CA certificate path (optional)
    pub intermediate_cert_path: Option<PathBuf>,

    /// Intermediate CA key path (optional)
    pub intermediate_key_path: Option<PathBuf>,

    /// Certificate validity period in days
    pub validity_days: i64,

    /// Key size in bits (2048, 3072, 4096)
    pub key_size: usize,

    /// Organization name for certificates
    pub organization: String,

    /// Country code (e.g., "US", "BR")
    pub country: String,

    /// State or province
    pub state: String,

    /// Locality/City
    pub locality: String,

    /// Enable external CA integration
    pub external_ca_enabled: bool,

    /// External CA API endpoint
    pub external_ca_url: Option<String>,

    /// External CA API key
    pub external_ca_api_key: Option<String>,

    /// Certificate revocation list (CRL) path
    pub crl_path: Option<PathBuf>,

    /// OCSP responder URL
    pub ocsp_url: Option<String>,
}

impl Default for CaConfig {
    fn default() -> Self {
        Self {
            ca_cert_path: PathBuf::from("certs/ca/ca.crt"),
            ca_key_path: PathBuf::from("certs/ca/ca.key"),
            intermediate_cert_path: Some(PathBuf::from("certs/ca/intermediate.crt")),
            intermediate_key_path: Some(PathBuf::from("certs/ca/intermediate.key")),
            validity_days: 365,
            key_size: 4096,
            organization: "BotServer Internal CA".to_string(),
            country: "BR".to_string(),
            state: "SP".to_string(),
            locality: "São Paulo".to_string(),
            external_ca_enabled: false,
            external_ca_url: None,
            external_ca_api_key: None,
            crl_path: Some(PathBuf::from("certs/ca/crl.pem")),
            ocsp_url: None,
        }
    }
}

/// Certificate Authority Manager
pub struct CaManager {
    config: CaConfig,
    ca_cert: Option<RcgenCertificate>,
    intermediate_cert: Option<RcgenCertificate>,
}

impl CaManager {
    /// Create a new CA manager
    pub fn new(config: CaConfig) -> Result<Self> {
        let mut manager = Self {
            config,
            ca_cert: None,
            intermediate_cert: None,
        };

        // Load existing CA if available
        manager.load_ca()?;

        Ok(manager)
    }

    /// Initialize a new Certificate Authority
    pub fn init_ca(&mut self) -> Result<()> {
        info!("Initializing new Certificate Authority");

        // Create CA directory structure
        self.create_ca_directories()?;

        // Generate root CA
        let ca_cert = self.generate_root_ca()?;
        self.ca_cert = Some(ca_cert.clone());

        // Generate intermediate CA if configured
        if self.config.intermediate_cert_path.is_some() {
            let intermediate = self.generate_intermediate_ca(&ca_cert)?;
            self.intermediate_cert = Some(intermediate);
        }

        info!("Certificate Authority initialized successfully");
        Ok(())
    }

    /// Load existing CA certificates
    fn load_ca(&mut self) -> Result<()> {
        if self.config.ca_cert_path.exists() && self.config.ca_key_path.exists() {
            debug!("Loading existing CA from {:?}", self.config.ca_cert_path);

            let cert_pem = fs::read_to_string(&self.config.ca_cert_path)?;
            let key_pem = fs::read_to_string(&self.config.ca_key_path)?;

            let key_pair = KeyPair::from_pem(&key_pem)?;
            let params = CertificateParams::from_ca_cert_pem(&cert_pem, key_pair)?;

            self.ca_cert = Some(RcgenCertificate::from_params(params)?);

            // Load intermediate CA if exists
            if let (Some(cert_path), Some(key_path)) = (
                &self.config.intermediate_cert_path,
                &self.config.intermediate_key_path,
            ) {
                if cert_path.exists() && key_path.exists() {
                    let cert_pem = fs::read_to_string(cert_path)?;
                    let key_pem = fs::read_to_string(key_path)?;

                    let key_pair = KeyPair::from_pem(&key_pem)?;
                    let params = CertificateParams::from_ca_cert_pem(&cert_pem, key_pair)?;

                    self.intermediate_cert = Some(RcgenCertificate::from_params(params)?);
                }
            }

            info!("Loaded existing CA certificates");
        } else {
            warn!("No existing CA found, initialization required");
        }

        Ok(())
    }

    /// Generate root CA certificate
    fn generate_root_ca(&self) -> Result<RcgenCertificate> {
        let mut params = CertificateParams::default();

        // Set as CA certificate
        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

        // Set distinguished name
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CountryName, &self.config.country);
        dn.push(DnType::StateOrProvinceName, &self.config.state);
        dn.push(DnType::LocalityName, &self.config.locality);
        dn.push(DnType::OrganizationName, &self.config.organization);
        dn.push(DnType::CommonName, "BotServer Root CA");
        params.distinguished_name = dn;

        // Set validity period
        params.not_before = OffsetDateTime::now_utc();
        params.not_after = OffsetDateTime::now_utc() + Duration::days(self.config.validity_days * 2);

        // Generate key pair
        let key_pair = KeyPair::generate(&rcgen::PKCS_RSA_SHA256)?;
        params.key_pair = Some(key_pair);

        // Create certificate
        let cert = RcgenCertificate::from_params(params)?;

        // Save to disk
        fs::write(&self.config.ca_cert_path, cert.serialize_pem()?)?;
        fs::write(&self.config.ca_key_path, cert.serialize_private_key_pem())?;

        info!("Generated root CA certificate");
        Ok(cert)
    }

    /// Generate intermediate CA certificate
    fn generate_intermediate_ca(&self, root_ca: &RcgenCertificate) -> Result<RcgenCertificate> {
        let mut params = CertificateParams::default();

        // Set as intermediate CA
        params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));

        // Set distinguished name
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CountryName, &self.config.country);
        dn.push(DnType::StateOrProvinceName, &self.config.state);
        dn.push(DnType::LocalityName, &self.config.locality);
        dn.push(DnType::OrganizationName, &self.config.organization);
        dn.push(DnType::CommonName, "BotServer Intermediate CA");
        params.distinguished_name = dn;

        // Set validity period (shorter than root)
        params.not_before = OffsetDateTime::now_utc();
        params.not_after = OffsetDateTime::now_utc() + Duration::days(self.config.validity_days);

        // Generate key pair
        let key_pair = KeyPair::generate(&rcgen::PKCS_RSA_SHA256)?;
        params.key_pair = Some(key_pair);

        // Create certificate
        let cert = RcgenCertificate::from_params(params)?;

        // Sign with root CA
        let signed_cert = cert.serialize_pem_with_signer(root_ca)?;

        // Save to disk
        if let (Some(cert_path), Some(key_path)) = (
            &self.config.intermediate_cert_path,
            &self.config.intermediate_key_path,
        ) {
            fs::write(cert_path, signed_cert)?;
            fs::write(key_path, cert.serialize_private_key_pem())?;
        }

        info!("Generated intermediate CA certificate");
        Ok(cert)
    }

    /// Issue a new certificate for a service
    pub fn issue_certificate(
        &self,
        common_name: &str,
        san_names: Vec<String>,
        is_client: bool,
    ) -> Result<(String, String)> {
        let signing_ca = self.intermediate_cert.as_ref()
            .or(self.ca_cert.as_ref())
            .ok_or_else(|| anyhow::anyhow!("CA not initialized"))?;

        let mut params = CertificateParams::default();

        // Set distinguished name
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CountryName, &self.config.country);
        dn.push(DnType::StateOrProvinceName, &self.config.state);
        dn.push(DnType::LocalityName, &self.config.locality);
        dn.push(DnType::OrganizationName, &self.config.organization);
        dn.push(DnType::CommonName, common_name);
        params.distinguished_name = dn;

        // Add Subject Alternative Names
        for san in san_names {
            if san.parse::<std::net::IpAddr>().is_ok() {
                params.subject_alt_names.push(SanType::IpAddress(san.parse()?));
            } else {
                params.subject_alt_names.push(SanType::DnsName(san));
            }
        }

        // Set validity period
        params.not_before = OffsetDateTime::now_utc();
        params.not_after = OffsetDateTime::now_utc() + Duration::days(self.config.validity_days);

        // Set key usage based on certificate type
        if is_client {
            params.extended_key_usages = vec![
                rcgen::ExtendedKeyUsagePurpose::ClientAuth,
            ];
        } else {
            params.extended_key_usages = vec![
                rcgen::ExtendedKeyUsagePurpose::ServerAuth,
            ];
        }

        // Generate key pair
        let key_pair = KeyPair::generate(&rcgen::PKCS_RSA_SHA256)?;
        params.key_pair = Some(key_pair);

        // Create and sign certificate
        let cert = RcgenCertificate::from_params(params)?;
        let cert_pem = cert.serialize_pem_with_signer(signing_ca)?;
        let key_pem = cert.serialize_private_key_pem();

        Ok((cert_pem, key_pem))
    }

    /// Issue certificates for all services
    pub fn issue_service_certificates(&self) -> Result<()> {
        let services = vec![
            ("api", vec!["localhost", "botserver", "127.0.0.1"]),
            ("llm", vec!["localhost", "llm", "127.0.0.1"]),
            ("embedding", vec!["localhost", "embedding", "127.0.0.1"]),
            ("qdrant", vec!["localhost", "qdrant", "127.0.0.1"]),
            ("postgres", vec!["localhost", "postgres", "127.0.0.1"]),
            ("redis", vec!["localhost", "redis", "127.0.0.1"]),
            ("minio", vec!["localhost", "minio", "127.0.0.1"]),
            ("directory", vec!["localhost", "directory", "127.0.0.1"]),
            ("email", vec!["localhost", "email", "127.0.0.1"]),
            ("meet", vec!["localhost", "meet", "127.0.0.1"]),
        ];

        for (service, sans) in services {
            self.issue_service_certificate(service, sans)?;
        }

        Ok(())
    }

    /// Issue certificate for a specific service
    pub fn issue_service_certificate(
        &self,
        service_name: &str,
        san_names: Vec<&str>,
    ) -> Result<()> {
        let cert_dir = PathBuf::from(format!("certs/{}", service_name));
        fs::create_dir_all(&cert_dir)?;

        // Issue server certificate
        let (cert_pem, key_pem) = self.issue_certificate(
            &format!("{}.botserver.local", service_name),
            san_names.iter().map(|s| s.to_string()).collect(),
            false,
        )?;

        fs::write(cert_dir.join("server.crt"), cert_pem)?;
        fs::write(cert_dir.join("server.key"), key_pem)?;

        // Issue client certificate for mTLS
        let (client_cert_pem, client_key_pem) = self.issue_certificate(
            &format!("{}-client.botserver.local", service_name),
            vec![format!("{}-client", service_name)],
            true,
        )?;

        fs::write(cert_dir.join("client.crt"), client_cert_pem)?;
        fs::write(cert_dir.join("client.key"), client_key_pem)?;

        // Copy CA certificate for verification
        if let Ok(ca_cert) = fs::read_to_string(&self.config.ca_cert_path) {
            fs::write(cert_dir.join("ca.crt"), ca_cert)?;
        }

        info!("Issued certificates for service: {}", service_name);
        Ok(())
    }

    /// Create CA directory structure
    fn create_ca_directories(&self) -> Result<()> {
        let ca_dir = self.config.ca_cert_path.parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid CA cert path"))?;

        fs::create_dir_all(ca_dir)?;
        fs::create_dir_all("certs/api")?;
        fs::create_dir_all("certs/llm")?;
        fs::create_dir_all("certs/embedding")?;
        fs::create_dir_all("certs/qdrant")?;
        fs::create_dir_all("certs/postgres")?;
        fs::create_dir_all("certs/redis")?;
        fs::create_dir_all("certs/minio")?;
        fs::create_dir_all("certs/directory")?;
        fs::create_dir_all("certs/email")?;
        fs::create_dir_all("certs/meet")?;

        Ok(())
    }

    /// Verify a certificate against the CA
    pub fn verify_certificate(&self, cert_pem: &str) -> Result<bool> {
        // This would implement certificate verification logic
        // For now, return true as placeholder
        Ok(true)
    }

    /// Revoke a certificate
    pub fn revoke_certificate(&self, serial_number: &str, reason: &str) -> Result<()> {
        // This would implement certificate revocation
        // and update the CRL
        warn!("Certificate revocation not yet implemented");
        Ok(())
    }

    /// Generate Certificate Revocation List (CRL)
    pub fn generate_crl(&self) -> Result<()> {
        // This would generate a CRL with revoked certificates
        warn!("CRL generation not yet implemented");
        Ok(())
    }

    /// Integrate with external CA if configured
    pub async fn sync_with_external_ca(&self) -> Result<()> {
        if !self.config.external_ca_enabled {
            return Ok(());
        }

        if let (Some(url), Some(api_key)) = (&self.config.external_ca_url, &self.config.external_ca_api_key) {
            info!("Syncing with external CA at {}", url);

            // This would implement the actual external CA integration
            // For example, using ACME protocol or proprietary API

            warn!("External CA integration not yet implemented");
        }

        Ok(())
    }
}

/// Certificate request information
#[derive(Debug, Serialize, Deserialize)]
pub struct CertificateRequest {
    pub common_name: String,
    pub san_names: Vec<String>,
    pub is_client: bool,
    pub validity_days: Option<i64>,
    pub key_size: Option<usize>,
}

/// Certificate response
#[derive(Debug, Serialize, Deserialize)]
pub struct CertificateResponse {
    pub certificate: String,
    pub private_key: String,
    pub ca_certificate: String,
    pub expires_at: String,
    pub serial_number: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_ca_config_default() {
        let config = CaConfig::default();
        assert_eq!(config.validity_days, 365);
        assert_eq!(config.key_size, 4096);
        assert!(!config.external_ca_enabled);
    }

    #[test]
    fn test_ca_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = CaConfig::default();
        config.ca_cert_path = temp_dir.path().join("ca.crt");
        config.ca_key_path = temp_dir.path().join("ca.key");

        let manager = CaManager::new(config);
        assert!(manager.is_ok());
    }
}
