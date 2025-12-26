use anyhow::Result;
use rcgen::{
    BasicConstraints, CertificateParams, DistinguishedName, DnType, IsCa, Issuer, KeyPair, SanType,
};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::fs;
use std::path::PathBuf;
use time::{Duration, OffsetDateTime};
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CaConfig {
    pub ca_cert_path: PathBuf,

    pub ca_key_path: PathBuf,

    pub intermediate_cert_path: Option<PathBuf>,

    pub intermediate_key_path: Option<PathBuf>,

    pub validity_days: i64,

    pub key_size: usize,

    pub organization: String,

    pub country: String,

    pub state: String,

    pub locality: String,

    pub external_ca_enabled: bool,

    pub external_ca_url: Option<String>,

    pub external_ca_api_key: Option<String>,

    pub crl_path: Option<PathBuf>,

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

pub struct CaManager {
    config: CaConfig,
    ca_params: Option<CertificateParams>,
    ca_key: Option<KeyPair>,
    intermediate_params: Option<CertificateParams>,
    intermediate_key: Option<KeyPair>,
}

impl std::fmt::Debug for CaManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CaManager")
            .field("config", &self.config)
            .field("ca_params", &self.ca_params.is_some())
            .field("ca_key", &self.ca_key.is_some())
            .field("intermediate_params", &self.intermediate_params.is_some())
            .field("intermediate_key", &self.intermediate_key.is_some())
            .finish()
    }
}

impl CaManager {
    pub fn new(config: CaConfig) -> Result<Self> {
        let mut manager = Self {
            config,
            ca_params: None,
            ca_key: None,
            intermediate_params: None,
            intermediate_key: None,
        };

        manager.load_ca()?;

        Ok(manager)
    }

    pub fn init_ca(&mut self) -> Result<()> {
        info!("Initializing new Certificate Authority");

        self.create_ca_directories()?;

        self.generate_root_ca()?;

        if self.config.intermediate_cert_path.is_some() {
            self.generate_intermediate_ca()?;
        }

        info!("Certificate Authority initialized successfully");
        Ok(())
    }

    fn load_ca(&mut self) -> Result<()> {
        if self.config.ca_cert_path.exists() && self.config.ca_key_path.exists() {
            debug!("Loading existing CA from {:?}", self.config.ca_cert_path);

            let key_pem = fs::read_to_string(&self.config.ca_key_path)?;
            let key_pair = KeyPair::from_pem(&key_pem)?;

            let mut params = CertificateParams::default();
            params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

            let mut dn = DistinguishedName::new();
            dn.push(DnType::CountryName, &self.config.country);
            dn.push(DnType::OrganizationName, &self.config.organization);
            dn.push(DnType::CommonName, "BotServer Root CA");
            params.distinguished_name = dn;

            self.ca_params = Some(params);
            self.ca_key = Some(key_pair);

            if let (Some(cert_path), Some(key_path)) = (
                &self.config.intermediate_cert_path,
                &self.config.intermediate_key_path,
            ) {
                if cert_path.exists() && key_path.exists() {
                    let key_pem = fs::read_to_string(key_path)?;
                    let key_pair = KeyPair::from_pem(&key_pem)?;

                    let mut params = CertificateParams::default();
                    params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));

                    let mut dn = DistinguishedName::new();
                    dn.push(DnType::CountryName, &self.config.country);
                    dn.push(DnType::OrganizationName, &self.config.organization);
                    dn.push(DnType::CommonName, "BotServer Intermediate CA");
                    params.distinguished_name = dn;

                    self.intermediate_params = Some(params);
                    self.intermediate_key = Some(key_pair);
                }
            }

            info!("Loaded existing CA certificates");
        } else {
            warn!("No existing CA found, initialization required");
        }

        Ok(())
    }

    fn generate_root_ca(&mut self) -> Result<()> {
        let mut params = CertificateParams::default();

        params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);

        let mut dn = DistinguishedName::new();
        dn.push(DnType::CountryName, &self.config.country);
        dn.push(DnType::StateOrProvinceName, &self.config.state);
        dn.push(DnType::LocalityName, &self.config.locality);
        dn.push(DnType::OrganizationName, &self.config.organization);
        dn.push(DnType::CommonName, "BotServer Root CA");
        params.distinguished_name = dn;

        params.not_before = OffsetDateTime::now_utc();
        params.not_after =
            OffsetDateTime::now_utc() + Duration::days(self.config.validity_days * 2);

        let key_pair = KeyPair::generate()?;

        let cert = params.self_signed(&key_pair)?;

        fs::write(&self.config.ca_cert_path, cert.pem())?;
        fs::write(&self.config.ca_key_path, key_pair.serialize_pem())?;

        self.ca_params = Some(params);
        self.ca_key = Some(key_pair);

        info!("Generated root CA certificate");
        Ok(())
    }

    fn generate_intermediate_ca(&mut self) -> Result<()> {
        let ca_params = self
            .ca_params
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Root CA params not available"))?;
        let ca_key = self
            .ca_key
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Root CA key not available"))?;

        let mut params = CertificateParams::default();

        params.is_ca = IsCa::Ca(BasicConstraints::Constrained(0));

        let mut dn = DistinguishedName::new();
        dn.push(DnType::CountryName, &self.config.country);
        dn.push(DnType::StateOrProvinceName, &self.config.state);
        dn.push(DnType::LocalityName, &self.config.locality);
        dn.push(DnType::OrganizationName, &self.config.organization);
        dn.push(DnType::CommonName, "BotServer Intermediate CA");
        params.distinguished_name = dn;

        params.not_before = OffsetDateTime::now_utc();
        params.not_after = OffsetDateTime::now_utc() + Duration::days(self.config.validity_days);

        let key_pair = KeyPair::generate()?;

        let issuer = Issuer::from_params(ca_params, ca_key);

        let cert = params.signed_by(&key_pair, &issuer)?;

        if let (Some(cert_path), Some(key_path)) = (
            &self.config.intermediate_cert_path,
            &self.config.intermediate_key_path,
        ) {
            fs::write(cert_path, cert.pem())?;
            fs::write(key_path, key_pair.serialize_pem())?;
        }

        self.intermediate_params = Some(params);
        self.intermediate_key = Some(key_pair);

        info!("Generated intermediate CA certificate");
        Ok(())
    }

    pub fn issue_certificate(
        &self,
        common_name: &str,
        san_names: Vec<String>,
        is_client: bool,
    ) -> Result<(String, String)> {
        let (signing_params, signing_key) =
            match (&self.intermediate_params, &self.intermediate_key) {
                (Some(params), Some(key)) => (params, key),
                _ => match (&self.ca_params, &self.ca_key) {
                    (Some(params), Some(key)) => (params, key),
                    _ => return Err(anyhow::anyhow!("CA not initialized")),
                },
            };

        let mut params = CertificateParams::default();

        let mut dn = DistinguishedName::new();
        dn.push(DnType::CountryName, &self.config.country);
        dn.push(DnType::StateOrProvinceName, &self.config.state);
        dn.push(DnType::LocalityName, &self.config.locality);
        dn.push(DnType::OrganizationName, &self.config.organization);
        dn.push(DnType::CommonName, common_name);
        params.distinguished_name = dn;

        for san in san_names {
            if san.parse::<std::net::IpAddr>().is_ok() {
                params
                    .subject_alt_names
                    .push(SanType::IpAddress(san.parse()?));
            } else {
                params
                    .subject_alt_names
                    .push(SanType::DnsName(san.try_into()?));
            }
        }

        params.not_before = OffsetDateTime::now_utc();
        params.not_after = OffsetDateTime::now_utc() + Duration::days(self.config.validity_days);

        if is_client {
            params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ClientAuth];
        } else {
            params.extended_key_usages = vec![rcgen::ExtendedKeyUsagePurpose::ServerAuth];
        }

        let key_pair = KeyPair::generate()?;

        let issuer = Issuer::from_params(signing_params, signing_key);

        let cert = params.signed_by(&key_pair, &issuer)?;
        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();

        Ok((cert_pem, key_pem))
    }

    pub fn issue_service_certificates(&self) -> Result<()> {
        let services = vec![
            ("api", vec!["localhost", "api", "127.0.0.1"]),
            ("llm", vec!["localhost", "llm", "127.0.0.1"]),
            ("embedding", vec!["localhost", "embedding", "127.0.0.1"]),
            ("vectordb", vec!["localhost", "vectordb", "127.0.0.1"]),
            ("tables", vec!["localhost", "tables", "127.0.0.1"]),
            ("cache", vec!["localhost", "cache", "127.0.0.1"]),
            ("drive", vec!["localhost", "drive", "127.0.0.1"]),
            ("directory", vec!["localhost", "directory", "127.0.0.1"]),
            ("email", vec!["localhost", "email", "127.0.0.1"]),
            ("meet", vec!["localhost", "meet", "127.0.0.1"]),
        ];

        for (service, sans) in services {
            self.issue_service_certificate(service, sans)?;
        }

        Ok(())
    }

    pub fn issue_service_certificate(
        &self,
        service_name: &str,
        san_names: Vec<&str>,
    ) -> Result<()> {
        let cert_dir = PathBuf::from(format!("certs/{}", service_name));
        fs::create_dir_all(&cert_dir)?;

        let (cert_pem, key_pem) = self.issue_certificate(
            &format!("{}.botserver.local", service_name),
            san_names.iter().map(|s| (*s).to_string()).collect(),
            false,
        )?;

        fs::write(cert_dir.join("server.crt"), cert_pem)?;
        fs::write(cert_dir.join("server.key"), key_pem)?;

        let (client_cert_pem, client_key_pem) = self.issue_certificate(
            &format!("{}-client.botserver.local", service_name),
            vec![format!("{}-client", service_name)],
            true,
        )?;

        fs::write(cert_dir.join("client.crt"), client_cert_pem)?;
        fs::write(cert_dir.join("client.key"), client_key_pem)?;

        if let Ok(ca_cert) = fs::read_to_string(&self.config.ca_cert_path) {
            fs::write(cert_dir.join("ca.crt"), ca_cert)?;
        }

        info!("Issued certificates for service: {}", service_name);
        Ok(())
    }

    fn create_ca_directories(&self) -> Result<()> {
        let ca_dir = self
            .config
            .ca_cert_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Invalid CA cert path"))?;

        fs::create_dir_all(ca_dir)?;
        fs::create_dir_all("certs/api")?;
        fs::create_dir_all("certs/llm")?;
        fs::create_dir_all("certs/embedding")?;
        fs::create_dir_all("certs/vectordb")?;
        fs::create_dir_all("certs/tables")?;
        fs::create_dir_all("certs/cache")?;
        fs::create_dir_all("certs/drive")?;
        fs::create_dir_all("certs/directory")?;
        fs::create_dir_all("certs/email")?;
        fs::create_dir_all("certs/meet")?;

        Ok(())
    }

    pub fn verify_certificate(&self, cert_pem: &str) -> Result<bool> {
        if !self.config.ca_cert_path.exists() {
            debug!("CA certificate not found");
            return Ok(false);
        }

        if cert_pem.is_empty() || !cert_pem.contains("BEGIN CERTIFICATE") {
            debug!("Invalid certificate PEM format");
            return Ok(false);
        }

        let revoked_path = self.config.ca_cert_path.with_extension("revoked");
        if revoked_path.exists() {
            let revoked_content = fs::read_to_string(&revoked_path)?;
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            cert_pem.hash(&mut hasher);
            let cert_hash = format!("{:016x}", hasher.finish());
            if revoked_content
                .lines()
                .any(|line| line.contains(&cert_hash))
            {
                debug!("Certificate is revoked");
                return Ok(false);
            }
        }

        info!("Certificate verified successfully");
        Ok(true)
    }

    pub fn revoke_certificate(&self, serial_number: &str, reason: &str) -> Result<()> {
        let revoked_path = self.config.ca_cert_path.with_extension("revoked");

        let entry = format!(
            "{}|{}|{}\n",
            serial_number,
            reason,
            OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339)?
        );

        let mut content = if revoked_path.exists() {
            fs::read_to_string(&revoked_path)?
        } else {
            String::new()
        };

        content.push_str(&entry);
        fs::write(&revoked_path, content)?;

        info!("Certificate {} revoked. Reason: {}", serial_number, reason);

        self.generate_crl()?;

        Ok(())
    }

    pub fn generate_crl(&self) -> Result<()> {
        let revoked_path = self.config.ca_cert_path.with_extension("revoked");
        let crl_path = self.config.ca_cert_path.with_extension("crl");

        let mut crl_content = String::from("-----BEGIN X509 CRL-----\n");
        let _ = writeln!(
            crl_content,
            "# CRL Generated: {}",
            OffsetDateTime::now_utc().format(&time::format_description::well_known::Rfc3339)?
        );
        let _ = writeln!(crl_content, "# Issuer: {}", self.config.organization);

        if revoked_path.exists() {
            let revoked = fs::read_to_string(&revoked_path)?;
            for line in revoked.lines() {
                if !line.is_empty() {
                    let _ = writeln!(crl_content, "# Revoked: {}", line);
                }
            }
        }

        crl_content.push_str("-----END X509 CRL-----\n");
        fs::write(&crl_path, crl_content)?;

        info!("CRL generated at {:?}", crl_path);

        Ok(())
    }

    pub async fn sync_with_external_ca(&self) -> Result<()> {
        if !self.config.external_ca_enabled {
            return Ok(());
        }

        let (Some(url), Some(api_key)) = (
            &self.config.external_ca_url,
            &self.config.external_ca_api_key,
        ) else {
            return Ok(());
        };

        info!("Syncing with external CA at {}", url);

        let client = reqwest::Client::new();

        let response = client
            .get(format!("{}/status", url))
            .header("Authorization", format!("Bearer {}", api_key))
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await?;

        if response.status().is_success() {
            info!("External CA sync successful");
        } else {
            warn!("External CA returned status: {}", response.status());
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CertificateRequest {
    pub common_name: String,
    pub san_names: Vec<String>,
    pub is_client: bool,
    pub validity_days: Option<i64>,
    pub key_size: Option<usize>,
}

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
        assert_eq!(config.country, "BR");
        assert_eq!(config.organization, "BotServer Internal CA");
    }

    #[test]
    fn test_ca_config_paths() {
        let config = CaConfig::default();
        assert_eq!(config.ca_cert_path, PathBuf::from("certs/ca/ca.crt"));
        assert_eq!(config.ca_key_path, PathBuf::from("certs/ca/ca.key"));
        assert!(config.intermediate_cert_path.is_some());
        assert!(config.intermediate_key_path.is_some());
    }

    #[test]
    fn test_ca_manager_creation() {
        let temp_dir = TempDir::new().unwrap();
        let mut config = CaConfig::default();
        config.ca_cert_path = temp_dir.path().join("ca.crt");
        config.ca_key_path = temp_dir.path().join("ca.key");
        config.intermediate_cert_path = Some(temp_dir.path().join("intermediate.crt"));
        config.intermediate_key_path = Some(temp_dir.path().join("intermediate.key"));
        config.crl_path = Some(temp_dir.path().join("crl.pem"));

        let manager = CaManager::new(config);
        assert!(manager.is_ok());
    }

    #[test]
    fn test_certificate_request_structure() {
        let request = CertificateRequest {
            common_name: "test.example.com".to_string(),
            san_names: vec!["alt.example.com".to_string()],
            is_client: false,
            validity_days: Some(365),
            key_size: Some(2048),
        };

        assert_eq!(request.common_name, "test.example.com");
        assert_eq!(request.san_names.len(), 1);
        assert!(!request.is_client);
    }

    #[test]
    fn test_certificate_response_structure() {
        let response = CertificateResponse {
            certificate: "-----BEGIN CERTIFICATE-----".to_string(),
            private_key: "-----BEGIN PRIVATE KEY-----".to_string(),
            ca_certificate: "-----BEGIN CERTIFICATE-----".to_string(),
            expires_at: "2026-01-01T00:00:00Z".to_string(),
            serial_number: "1234567890".to_string(),
        };

        assert!(response.certificate.starts_with("-----BEGIN"));
        assert!(response.private_key.starts_with("-----BEGIN"));
        assert!(!response.serial_number.is_empty());
    }

    #[test]
    fn test_ca_config_external_ca() {
        let mut config = CaConfig::default();
        config.external_ca_enabled = true;
        config.external_ca_url = Some("https://ca.example.com".to_string());
        config.external_ca_api_key = Some("secret-key".to_string());

        assert!(config.external_ca_enabled);
        assert_eq!(
            config.external_ca_url,
            Some("https://ca.example.com".to_string())
        );
    }
}
