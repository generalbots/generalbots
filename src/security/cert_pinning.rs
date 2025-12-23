


























use anyhow::{anyhow, Context, Result};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use reqwest::{Certificate, Client, ClientBuilder};
use ring::digest::{digest, SHA256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tracing::{debug, error, info, warn};
use x509_parser::prelude::*;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertPinningConfig {

    pub enabled: bool,


    pub pins: HashMap<String, Vec<PinnedCert>>,


    pub require_pins: bool,


    pub allow_backup_pins: bool,


    pub report_only: bool,


    pub config_path: Option<PathBuf>,


    pub cache_ttl_secs: u64,
}

impl Default for CertPinningConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            pins: HashMap::new(),
            require_pins: false,
            allow_backup_pins: true,
            report_only: false,
            config_path: None,
            cache_ttl_secs: 3600,
        }
    }
}

impl CertPinningConfig {

    pub fn new() -> Self {
        Self::default()
    }


    pub fn strict() -> Self {
        Self {
            enabled: true,
            pins: HashMap::new(),
            require_pins: true,
            allow_backup_pins: true,
            report_only: false,
            config_path: None,
            cache_ttl_secs: 3600,
        }
    }


    pub fn report_only() -> Self {
        Self {
            enabled: true,
            pins: HashMap::new(),
            require_pins: false,
            allow_backup_pins: true,
            report_only: true,
            config_path: None,
            cache_ttl_secs: 3600,
        }
    }


    pub fn add_pin(&mut self, pin: PinnedCert) {
        let hostname = pin.hostname.clone();
        self.pins.entry(hostname).or_default().push(pin);
    }


    pub fn add_pins(&mut self, hostname: &str, pins: Vec<PinnedCert>) {
        self.pins.insert(hostname.to_string(), pins);
    }


    pub fn remove_pins(&mut self, hostname: &str) {
        self.pins.remove(hostname);
    }


    pub fn get_pins(&self, hostname: &str) -> Option<&Vec<PinnedCert>> {
        self.pins.get(hostname)
    }


    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read pin config from {:?}", path))?;

        let config: Self =
            serde_json::from_str(&content).context("Failed to parse pin configuration")?;

        info!("Loaded certificate pinning config from {:?}", path);
        Ok(config)
    }


    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let content =
            serde_json::to_string_pretty(self).context("Failed to serialize pin configuration")?;

        fs::write(path, content)
            .with_context(|| format!("Failed to write pin config to {:?}", path))?;

        info!("Saved certificate pinning config to {:?}", path);
        Ok(())
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedCert {

    pub hostname: String,



    pub fingerprint: String,


    pub description: Option<String>,


    pub is_backup: bool,


    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,


    pub pin_type: PinType,
}

impl PinnedCert {

    pub fn new(hostname: &str, fingerprint: &str) -> Self {
        Self {
            hostname: hostname.to_string(),
            fingerprint: fingerprint.to_string(),
            description: None,
            is_backup: false,
            expires_at: None,
            pin_type: PinType::Leaf,
        }
    }


    pub fn backup(hostname: &str, fingerprint: &str) -> Self {
        Self {
            hostname: hostname.to_string(),
            fingerprint: fingerprint.to_string(),
            description: Some("Backup pin for certificate rotation".to_string()),
            is_backup: true,
            expires_at: None,
            pin_type: PinType::Leaf,
        }
    }


    pub fn with_type(mut self, pin_type: PinType) -> Self {
        self.pin_type = pin_type;
        self
    }


    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }


    pub fn with_expiration(mut self, expires: chrono::DateTime<chrono::Utc>) -> Self {
        self.expires_at = Some(expires);
        self
    }


    pub fn get_hash_bytes(&self) -> Result<Vec<u8>> {
        let hash_str = self
            .fingerprint
            .strip_prefix("sha256//")
            .ok_or_else(|| anyhow!("Invalid fingerprint format, expected 'sha256//BASE64'"))?;

        BASE64
            .decode(hash_str)
            .context("Failed to decode base64 fingerprint")
    }


    pub fn verify(&self, cert_der: &[u8]) -> Result<bool> {
        let expected_hash = self.get_hash_bytes()?;
        let actual_hash = compute_spki_fingerprint(cert_der)?;

        Ok(expected_hash == actual_hash)
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PinType {

    Leaf,

    Intermediate,

    Root,
}

impl Default for PinType {
    fn default() -> Self {
        Self::Leaf
    }
}


#[derive(Debug, Clone)]
pub struct PinValidationResult {

    pub valid: bool,


    pub hostname: String,


    pub matched_pin: Option<String>,


    pub actual_fingerprint: String,


    pub error: Option<String>,


    pub backup_match: bool,
}

impl PinValidationResult {

    pub fn success(hostname: &str, fingerprint: &str, backup: bool) -> Self {
        Self {
            valid: true,
            hostname: hostname.to_string(),
            matched_pin: Some(fingerprint.to_string()),
            actual_fingerprint: fingerprint.to_string(),
            error: None,
            backup_match: backup,
        }
    }


    pub fn failure(hostname: &str, actual: &str, error: &str) -> Self {
        Self {
            valid: false,
            hostname: hostname.to_string(),
            matched_pin: None,
            actual_fingerprint: actual.to_string(),
            error: Some(error.to_string()),
            backup_match: false,
        }
    }
}


#[derive(Debug)]
pub struct CertPinningManager {
    config: Arc<RwLock<CertPinningConfig>>,
    validation_cache: Arc<RwLock<HashMap<String, (PinValidationResult, std::time::Instant)>>>,
}

impl CertPinningManager {

    pub fn new(config: CertPinningConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            validation_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }


    pub fn default_manager() -> Self {
        Self::new(CertPinningConfig::default())
    }


    pub fn is_enabled(&self) -> bool {
        self.config.read().unwrap().enabled
    }


    pub fn add_pin(&self, pin: PinnedCert) -> Result<()> {
        let mut config = self
            .config
            .write()
            .map_err(|_| anyhow!("Failed to acquire write lock"))?;
        config.add_pin(pin);
        Ok(())
    }


    pub fn remove_pins(&self, hostname: &str) -> Result<()> {
        let mut config = self
            .config
            .write()
            .map_err(|_| anyhow!("Failed to acquire write lock"))?;
        config.remove_pins(hostname);


        let mut cache = self
            .validation_cache
            .write()
            .map_err(|_| anyhow!("Failed to acquire cache lock"))?;
        cache.remove(hostname);

        Ok(())
    }


    pub fn validate_certificate(
        &self,
        hostname: &str,
        cert_der: &[u8],
    ) -> Result<PinValidationResult> {
        let config = self
            .config
            .read()
            .map_err(|_| anyhow!("Failed to acquire read lock"))?;

        if !config.enabled {
            return Ok(PinValidationResult::success(hostname, "disabled", false));
        }


        if let Ok(cache) = self.validation_cache.read() {
            if let Some((result, timestamp)) = cache.get(hostname) {
                if timestamp.elapsed().as_secs() < config.cache_ttl_secs {
                    return Ok(result.clone());
                }
            }
        }


        let actual_hash = compute_spki_fingerprint(cert_der)?;
        let actual_fingerprint = format!("sha256//{}", BASE64.encode(&actual_hash));


        let pins = match config.get_pins(hostname) {
            Some(pins) => pins,
            None => {
                if config.require_pins {
                    let result = PinValidationResult::failure(
                        hostname,
                        &actual_fingerprint,
                        "No pins configured for hostname",
                    );

                    if config.report_only {
                        warn!(
                            "Certificate pinning violation (report-only): {} - {}",
                            hostname, "No pins configured"
                        );
                        return Ok(PinValidationResult::success(hostname, "report-only", false));
                    }

                    return Ok(result);
                }


                return Ok(PinValidationResult::success(
                    hostname,
                    "no-pins-required",
                    false,
                ));
            }
        };


        for pin in pins {
            match pin.verify(cert_der) {
                Ok(true) => {
                    let result =
                        PinValidationResult::success(hostname, &pin.fingerprint, pin.is_backup);

                    if pin.is_backup {
                        warn!(
                            "Certificate matched backup pin for {}: {}",
                            hostname,
                            pin.description.as_deref().unwrap_or("backup")
                        );
                    }


                    if let Ok(mut cache) = self.validation_cache.write() {
                        cache.insert(
                            hostname.to_string(),
                            (result.clone(), std::time::Instant::now()),
                        );
                    }

                    return Ok(result);
                }
                Ok(false) => continue,
                Err(e) => {
                    debug!("Pin verification error for {}: {}", hostname, e);
                    continue;
                }
            }
        }


        let result = PinValidationResult::failure(
            hostname,
            &actual_fingerprint,
            &format!(
                "Certificate fingerprint {} does not match any pinned certificate",
                actual_fingerprint
            ),
        );

        if config.report_only {
            warn!(
                "Certificate pinning violation (report-only): {} - actual fingerprint: {}",
                hostname, actual_fingerprint
            );
            return Ok(PinValidationResult::success(hostname, "report-only", false));
        }

        error!(
            "Certificate pinning failure for {}: fingerprint {} not in pin set",
            hostname, actual_fingerprint
        );

        Ok(result)
    }


    pub fn create_pinned_client(&self, hostname: &str) -> Result<Client> {
        self.create_pinned_client_with_options(hostname, None, Duration::from_secs(30))
    }


    pub fn create_pinned_client_with_options(
        &self,
        hostname: &str,
        ca_cert: Option<&Certificate>,
        timeout: Duration,
    ) -> Result<Client> {
        let config = self
            .config
            .read()
            .map_err(|_| anyhow!("Failed to acquire read lock"))?;

        let mut builder = ClientBuilder::new()
            .timeout(timeout)
            .connect_timeout(Duration::from_secs(10))
            .use_rustls_tls()
            .https_only(true)
            .tls_built_in_root_certs(true);


        if let Some(cert) = ca_cert {
            builder = builder.add_root_certificate(cert.clone());
        }




        if config.enabled && config.get_pins(hostname).is_some() {
            debug!(
                "Creating pinned client for {} with {} pins",
                hostname,
                config.get_pins(hostname).map(|p| p.len()).unwrap_or(0)
            );
        }

        builder.build().context("Failed to build HTTP client")
    }


    pub fn validate_pem_file(
        &self,
        hostname: &str,
        pem_path: &Path,
    ) -> Result<PinValidationResult> {
        let pem_data = fs::read(pem_path)
            .with_context(|| format!("Failed to read PEM file: {:?}", pem_path))?;

        let der = pem_to_der(&pem_data)?;
        self.validate_certificate(hostname, &der)
    }


    pub fn generate_pin_from_file(hostname: &str, cert_path: &Path) -> Result<PinnedCert> {
        let cert_data = fs::read(cert_path)
            .with_context(|| format!("Failed to read certificate: {:?}", cert_path))?;


        let der = if cert_data.starts_with(b"-----BEGIN") {
            pem_to_der(&cert_data)?
        } else {
            cert_data
        };

        let fingerprint = compute_spki_fingerprint(&der)?;
        let fingerprint_str = format!("sha256//{}", BASE64.encode(&fingerprint));

        Ok(PinnedCert::new(hostname, &fingerprint_str))
    }


    pub fn generate_pins_from_directory(
        hostname: &str,
        cert_dir: &Path,
    ) -> Result<Vec<PinnedCert>> {
        let mut pins = Vec::new();

        for entry in fs::read_dir(cert_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if matches!(ext, "crt" | "pem" | "cer" | "der") {
                    match Self::generate_pin_from_file(hostname, &path) {
                        Ok(pin) => {
                            info!("Generated pin from {:?}: {}", path, pin.fingerprint);
                            pins.push(pin);
                        }
                        Err(e) => {
                            warn!("Failed to generate pin from {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        Ok(pins)
    }


    pub fn export_pins(&self, path: &Path) -> Result<()> {
        let config = self
            .config
            .read()
            .map_err(|_| anyhow!("Failed to acquire read lock"))?;

        config.save_to_file(path)
    }


    pub fn import_pins(&self, path: &Path) -> Result<()> {
        let imported = CertPinningConfig::load_from_file(path)?;

        let mut config = self
            .config
            .write()
            .map_err(|_| anyhow!("Failed to acquire write lock"))?;

        for (hostname, pins) in imported.pins {
            config.pins.insert(hostname, pins);
        }


        if let Ok(mut cache) = self.validation_cache.write() {
            cache.clear();
        }

        Ok(())
    }


    pub fn get_stats(&self) -> Result<PinningStats> {
        let config = self
            .config
            .read()
            .map_err(|_| anyhow!("Failed to acquire read lock"))?;

        let mut total_pins = 0;
        let mut backup_pins = 0;
        let mut expiring_soon = 0;

        let now = chrono::Utc::now();
        let soon = now + chrono::Duration::days(30);

        for pins in config.pins.values() {
            for pin in pins {
                total_pins += 1;
                if pin.is_backup {
                    backup_pins += 1;
                }
                if let Some(expires) = pin.expires_at {
                    if expires < soon {
                        expiring_soon += 1;
                    }
                }
            }
        }

        Ok(PinningStats {
            enabled: config.enabled,
            total_hosts: config.pins.len(),
            total_pins,
            backup_pins,
            expiring_soon,
            report_only: config.report_only,
        })
    }
}


#[derive(Debug, Clone, Serialize)]
pub struct PinningStats {
    pub enabled: bool,
    pub total_hosts: usize,
    pub total_pins: usize,
    pub backup_pins: usize,
    pub expiring_soon: usize,
    pub report_only: bool,
}


pub fn compute_spki_fingerprint(cert_der: &[u8]) -> Result<Vec<u8>> {
    let (_, cert) = X509Certificate::from_der(cert_der)
        .map_err(|e| anyhow!("Failed to parse X.509 certificate: {}", e))?;


    let spki = cert.public_key().raw;


    let hash = digest(&SHA256, spki);

    Ok(hash.as_ref().to_vec())
}


pub fn compute_cert_fingerprint(cert_der: &[u8]) -> Vec<u8> {
    let hash = digest(&SHA256, cert_der);
    hash.as_ref().to_vec()
}


pub fn pem_to_der(pem_data: &[u8]) -> Result<Vec<u8>> {
    let pem_str = std::str::from_utf8(pem_data).context("Invalid UTF-8 in PEM data")?;


    let start_marker = "-----BEGIN CERTIFICATE-----";
    let end_marker = "-----END CERTIFICATE-----";

    let start = pem_str
        .find(start_marker)
        .ok_or_else(|| anyhow!("No certificate found in PEM data"))?;

    let end = pem_str
        .find(end_marker)
        .ok_or_else(|| anyhow!("Invalid PEM: missing end marker"))?;

    let base64_data = &pem_str[start + start_marker.len()..end];
    let cleaned: String = base64_data.chars().filter(|c| !c.is_whitespace()).collect();

    BASE64
        .decode(&cleaned)
        .context("Failed to decode base64 certificate data")
}


pub fn format_fingerprint(hash: &[u8]) -> String {
    hash.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(":")
}


pub fn parse_fingerprint(formatted: &str) -> Result<Vec<u8>> {

    if let Some(base64_part) = formatted.strip_prefix("sha256//") {
        return BASE64
            .decode(base64_part)
            .context("Failed to decode base64 fingerprint");
    }


    if formatted.contains(':') {
        let bytes: Result<Vec<u8>, _> = formatted
            .split(':')
            .map(|hex| u8::from_str_radix(hex, 16))
            .collect();

        return bytes.context("Failed to parse hex fingerprint");
    }


    let bytes: Result<Vec<u8>, _> = (0..formatted.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&formatted[i..i + 2], 16))
        .collect();

    bytes.context("Failed to parse fingerprint")
}
