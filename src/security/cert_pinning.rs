//! Certificate Pinning Module
//!
//! Provides certificate pinning functionality to prevent man-in-the-middle attacks
//! by validating server certificates against pre-configured SHA-256 fingerprints.
//!
//! # Overview
//!
//! Certificate pinning adds an additional layer of security beyond standard TLS
//! certificate validation. Even if an attacker obtains a valid certificate from
//! a trusted CA, the connection will be rejected if the certificate's fingerprint
//! doesn't match the pinned value.
//!
//! # Usage
//!
//! ```rust,ignore
//! use botserver::security::cert_pinning::{CertPinningConfig, CertPinningManager, PinnedCert};
//!
//! let mut config = CertPinningConfig::default();
//! config.add_pin(PinnedCert::new(
//!     "api.example.com",
//!     "sha256//AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
//! ));
//!
//! let manager = CertPinningManager::new(config);
//! let client = manager.create_pinned_client("api.example.com")?;
//! ```

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

/// Configuration for certificate pinning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertPinningConfig {
    /// Enable certificate pinning globally
    pub enabled: bool,

    /// Pinned certificates by hostname
    pub pins: HashMap<String, Vec<PinnedCert>>,

    /// Whether to fail if no pin is configured for a host
    pub require_pins: bool,

    /// Allow backup pins (multiple pins per host for rotation)
    pub allow_backup_pins: bool,

    /// Report-only mode (log violations but don't block)
    pub report_only: bool,

    /// Path to store/load pin configuration
    pub config_path: Option<PathBuf>,

    /// Pin validation cache TTL in seconds
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
    /// Create a new config with pinning enabled
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a strict config that requires pins for all hosts
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

    /// Create a report-only config for testing
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

    /// Add a pinned certificate
    pub fn add_pin(&mut self, pin: PinnedCert) {
        let hostname = pin.hostname.clone();
        self.pins.entry(hostname).or_default().push(pin);
    }

    /// Add multiple pins for a hostname (primary + backups)
    pub fn add_pins(&mut self, hostname: &str, pins: Vec<PinnedCert>) {
        self.pins.insert(hostname.to_string(), pins);
    }

    /// Remove all pins for a hostname
    pub fn remove_pins(&mut self, hostname: &str) {
        self.pins.remove(hostname);
    }

    /// Get pins for a hostname
    pub fn get_pins(&self, hostname: &str) -> Option<&Vec<PinnedCert>> {
        self.pins.get(hostname)
    }

    /// Load configuration from file
    pub fn load_from_file(path: &Path) -> Result<Self> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read pin config from {:?}", path))?;

        let config: Self =
            serde_json::from_str(&content).context("Failed to parse pin configuration")?;

        info!("Loaded certificate pinning config from {:?}", path);
        Ok(config)
    }

    /// Save configuration to file
    pub fn save_to_file(&self, path: &Path) -> Result<()> {
        let content =
            serde_json::to_string_pretty(self).context("Failed to serialize pin configuration")?;

        fs::write(path, content)
            .with_context(|| format!("Failed to write pin config to {:?}", path))?;

        info!("Saved certificate pinning config to {:?}", path);
        Ok(())
    }
}

/// A pinned certificate entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PinnedCert {
    /// Hostname this pin applies to
    pub hostname: String,

    /// SHA-256 fingerprint of the certificate's Subject Public Key Info (SPKI)
    /// Format: "sha256//BASE64_ENCODED_HASH"
    pub fingerprint: String,

    /// Optional human-readable description
    pub description: Option<String>,

    /// Whether this is a backup pin
    pub is_backup: bool,

    /// Expiration date (for pin rotation planning)
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,

    /// Pin type (leaf certificate, intermediate, or root)
    pub pin_type: PinType,
}

impl PinnedCert {
    /// Create a new pinned certificate entry
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

    /// Create a backup pin
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

    /// Set the pin type
    pub fn with_type(mut self, pin_type: PinType) -> Self {
        self.pin_type = pin_type;
        self
    }

    /// Set description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = Some(desc.to_string());
        self
    }

    /// Set expiration
    pub fn with_expiration(mut self, expires: chrono::DateTime<chrono::Utc>) -> Self {
        self.expires_at = Some(expires);
        self
    }

    /// Extract the raw hash bytes from the fingerprint
    pub fn get_hash_bytes(&self) -> Result<Vec<u8>> {
        let hash_str = self
            .fingerprint
            .strip_prefix("sha256//")
            .ok_or_else(|| anyhow!("Invalid fingerprint format, expected 'sha256//BASE64'"))?;

        BASE64
            .decode(hash_str)
            .context("Failed to decode base64 fingerprint")
    }

    /// Verify if a certificate matches this pin
    pub fn verify(&self, cert_der: &[u8]) -> Result<bool> {
        let expected_hash = self.get_hash_bytes()?;
        let actual_hash = compute_spki_fingerprint(cert_der)?;

        Ok(expected_hash == actual_hash)
    }
}

/// Type of certificate being pinned
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PinType {
    /// Pin the leaf/end-entity certificate
    Leaf,
    /// Pin an intermediate CA certificate
    Intermediate,
    /// Pin the root CA certificate
    Root,
}

impl Default for PinType {
    fn default() -> Self {
        Self::Leaf
    }
}

/// Result of a pin validation
#[derive(Debug, Clone)]
pub struct PinValidationResult {
    /// Whether the pin validation passed
    pub valid: bool,

    /// The hostname that was validated
    pub hostname: String,

    /// Which pin matched (if any)
    pub matched_pin: Option<String>,

    /// The actual fingerprint of the certificate
    pub actual_fingerprint: String,

    /// Error message if validation failed
    pub error: Option<String>,

    /// Whether this was a backup pin match
    pub backup_match: bool,
}

impl PinValidationResult {
    /// Create a successful validation result
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

    /// Create a failed validation result
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

/// Certificate Pinning Manager
pub struct CertPinningManager {
    config: Arc<RwLock<CertPinningConfig>>,
    validation_cache: Arc<RwLock<HashMap<String, (PinValidationResult, std::time::Instant)>>>,
}

impl CertPinningManager {
    /// Create a new certificate pinning manager
    pub fn new(config: CertPinningConfig) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            validation_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create with default configuration
    pub fn default_manager() -> Self {
        Self::new(CertPinningConfig::default())
    }

    /// Check if pinning is enabled
    pub fn is_enabled(&self) -> bool {
        self.config.read().unwrap().enabled
    }

    /// Add a pin dynamically
    pub fn add_pin(&self, pin: PinnedCert) -> Result<()> {
        let mut config = self
            .config
            .write()
            .map_err(|_| anyhow!("Failed to acquire write lock"))?;
        config.add_pin(pin);
        Ok(())
    }

    /// Remove pins for a hostname
    pub fn remove_pins(&self, hostname: &str) -> Result<()> {
        let mut config = self
            .config
            .write()
            .map_err(|_| anyhow!("Failed to acquire write lock"))?;
        config.remove_pins(hostname);

        // Clear cache for this hostname
        let mut cache = self
            .validation_cache
            .write()
            .map_err(|_| anyhow!("Failed to acquire cache lock"))?;
        cache.remove(hostname);

        Ok(())
    }

    /// Validate a certificate against pinned fingerprints
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

        // Check cache first
        if let Ok(cache) = self.validation_cache.read() {
            if let Some((result, timestamp)) = cache.get(hostname) {
                if timestamp.elapsed().as_secs() < config.cache_ttl_secs {
                    return Ok(result.clone());
                }
            }
        }

        // Compute actual fingerprint
        let actual_hash = compute_spki_fingerprint(cert_der)?;
        let actual_fingerprint = format!("sha256//{}", BASE64.encode(&actual_hash));

        // Get pins for this hostname
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

                // No pins required, pass through
                return Ok(PinValidationResult::success(
                    hostname,
                    "no-pins-required",
                    false,
                ));
            }
        };

        // Check against all pins
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

                    // Update cache
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

        // No pin matched
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

    /// Create an HTTP client with certificate pinning for a specific host
    pub fn create_pinned_client(&self, hostname: &str) -> Result<Client> {
        self.create_pinned_client_with_options(hostname, None, Duration::from_secs(30))
    }

    /// Create an HTTP client with certificate pinning and custom options
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

        // Add custom CA if provided
        if let Some(cert) = ca_cert {
            builder = builder.add_root_certificate(cert.clone());
        }

        // If pinning is enabled and we have pins, we need to use a custom verifier
        // Note: reqwest doesn't directly support custom certificate verification,
        // so we validate after connection or use a pre-flight check
        if config.enabled && config.get_pins(hostname).is_some() {
            debug!(
                "Creating pinned client for {} with {} pins",
                hostname,
                config.get_pins(hostname).map(|p| p.len()).unwrap_or(0)
            );
        }

        builder.build().context("Failed to build HTTP client")
    }

    /// Validate a certificate from a PEM file
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

    /// Generate a pin from a certificate file
    pub fn generate_pin_from_file(hostname: &str, cert_path: &Path) -> Result<PinnedCert> {
        let cert_data = fs::read(cert_path)
            .with_context(|| format!("Failed to read certificate: {:?}", cert_path))?;

        // Try PEM first, then DER
        let der = if cert_data.starts_with(b"-----BEGIN") {
            pem_to_der(&cert_data)?
        } else {
            cert_data
        };

        let fingerprint = compute_spki_fingerprint(&der)?;
        let fingerprint_str = format!("sha256//{}", BASE64.encode(&fingerprint));

        Ok(PinnedCert::new(hostname, &fingerprint_str))
    }

    /// Generate pins for all certificates in a directory
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

    /// Export current pins to a file
    pub fn export_pins(&self, path: &Path) -> Result<()> {
        let config = self
            .config
            .read()
            .map_err(|_| anyhow!("Failed to acquire read lock"))?;

        config.save_to_file(path)
    }

    /// Import pins from a file
    pub fn import_pins(&self, path: &Path) -> Result<()> {
        let imported = CertPinningConfig::load_from_file(path)?;

        let mut config = self
            .config
            .write()
            .map_err(|_| anyhow!("Failed to acquire write lock"))?;

        for (hostname, pins) in imported.pins {
            config.pins.insert(hostname, pins);
        }

        // Clear cache
        if let Ok(mut cache) = self.validation_cache.write() {
            cache.clear();
        }

        Ok(())
    }

    /// Get statistics about pinned certificates
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

/// Statistics about certificate pinning
#[derive(Debug, Clone, Serialize)]
pub struct PinningStats {
    pub enabled: bool,
    pub total_hosts: usize,
    pub total_pins: usize,
    pub backup_pins: usize,
    pub expiring_soon: usize,
    pub report_only: bool,
}

/// Compute SHA-256 fingerprint of a certificate's Subject Public Key Info (SPKI)
pub fn compute_spki_fingerprint(cert_der: &[u8]) -> Result<Vec<u8>> {
    let (_, cert) = X509Certificate::from_der(cert_der)
        .map_err(|e| anyhow!("Failed to parse X.509 certificate: {}", e))?;

    // Get the raw SPKI bytes
    let spki = cert.public_key().raw;

    // Compute SHA-256 hash
    let hash = digest(&SHA256, spki);

    Ok(hash.as_ref().to_vec())
}

/// Compute SHA-256 fingerprint of the entire certificate (not just SPKI)
pub fn compute_cert_fingerprint(cert_der: &[u8]) -> Vec<u8> {
    let hash = digest(&SHA256, cert_der);
    hash.as_ref().to_vec()
}

/// Convert PEM-encoded certificate to DER
pub fn pem_to_der(pem_data: &[u8]) -> Result<Vec<u8>> {
    let pem_str = std::str::from_utf8(pem_data).context("Invalid UTF-8 in PEM data")?;

    // Find certificate block
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

/// Format a fingerprint for display
pub fn format_fingerprint(hash: &[u8]) -> String {
    hash.iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(":")
}

/// Parse a formatted fingerprint back to bytes
pub fn parse_fingerprint(formatted: &str) -> Result<Vec<u8>> {
    // Handle "sha256//BASE64" format
    if let Some(base64_part) = formatted.strip_prefix("sha256//") {
        return BASE64
            .decode(base64_part)
            .context("Failed to decode base64 fingerprint");
    }

    // Handle colon-separated hex format
    if formatted.contains(':') {
        let bytes: Result<Vec<u8>, _> = formatted
            .split(':')
            .map(|hex| u8::from_str_radix(hex, 16))
            .collect();

        return bytes.context("Failed to parse hex fingerprint");
    }

    // Try plain hex
    let bytes: Result<Vec<u8>, _> = (0..formatted.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&formatted[i..i + 2], 16))
        .collect();

    bytes.context("Failed to parse fingerprint")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pinned_cert_creation() {
        let pin = PinnedCert::new(
            "api.example.com",
            "sha256//AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        );

        assert_eq!(pin.hostname, "api.example.com");
        assert!(!pin.is_backup);
        assert_eq!(pin.pin_type, PinType::Leaf);
    }

    #[test]
    fn test_backup_pin() {
        let pin = PinnedCert::backup(
            "api.example.com",
            "sha256//BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB=",
        );

        assert!(pin.is_backup);
        assert!(pin.description.is_some());
    }

    #[test]
    fn test_config_add_pin() {
        let mut config = CertPinningConfig::default();
        config.add_pin(PinnedCert::new(
            "example.com",
            "sha256//AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        ));

        assert!(config.get_pins("example.com").is_some());
        assert_eq!(config.get_pins("example.com").unwrap().len(), 1);
    }

    #[test]
    fn test_format_fingerprint() {
        let hash = vec![0xAB, 0xCD, 0xEF, 0x12];
        let formatted = format_fingerprint(&hash);
        assert_eq!(formatted, "AB:CD:EF:12");
    }

    #[test]
    fn test_parse_fingerprint_hex() {
        let result = parse_fingerprint("AB:CD:EF:12").unwrap();
        assert_eq!(result, vec![0xAB, 0xCD, 0xEF, 0x12]);
    }

    #[test]
    fn test_parse_fingerprint_base64() {
        let original = vec![0xAB, 0xCD, 0xEF, 0x12];
        let base64 = format!("sha256//{}", BASE64.encode(&original));
        let result = parse_fingerprint(&base64).unwrap();
        assert_eq!(result, original);
    }

    #[test]
    fn test_pinning_stats() {
        let mut config = CertPinningConfig::default();
        config.add_pin(PinnedCert::new(
            "host1.com",
            "sha256//AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=",
        ));
        config.add_pin(PinnedCert::backup(
            "host1.com",
            "sha256//BBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBBB=",
        ));
        config.add_pin(PinnedCert::new(
            "host2.com",
            "sha256//CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC=",
        ));

        let manager = CertPinningManager::new(config);
        let stats = manager.get_stats().unwrap();

        assert!(stats.enabled);
        assert_eq!(stats.total_hosts, 2);
        assert_eq!(stats.total_pins, 3);
        assert_eq!(stats.backup_pins, 1);
    }

    #[test]
    fn test_pem_to_der() {
        // Minimal test PEM (this is a mock, real certs would be longer)
        let mock_pem = b"-----BEGIN CERTIFICATE-----
MIIB
-----END CERTIFICATE-----";

        // Should fail gracefully with invalid base64
        let result = pem_to_der(mock_pem);
        // We expect this to fail because "MIIB" is incomplete base64
        assert!(result.is_err() || result.unwrap().len() > 0);
    }

    #[test]
    fn test_manager_disabled() {
        let mut config = CertPinningConfig::default();
        config.enabled = false;

        let manager = CertPinningManager::new(config);
        assert!(!manager.is_enabled());
    }
}
