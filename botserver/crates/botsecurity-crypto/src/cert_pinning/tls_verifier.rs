//! TLS-time certificate pin enforcement for outbound clients (issue #2626).
//!
//! The manager used to expose `create_pinned_client` that built a plain reqwest
//! client: the pin set was never consulted during the handshake. This module
//! installs a rustls [`ServerCertVerifier`] that runs the standard webpki chain
//! check *and* the pin decision on every connection, so a wrong pin aborts the
//! handshake in strict mode.

use super::*;

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::client::WebPkiServerVerifier;
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{
    CertificateError, ClientConfig, DigitallySignedStruct, Error as RustlsError, RootCertStore,
    SignatureScheme,
};
use std::sync::OnceLock;

/// Environment variable holding the path to a pin-config JSON file.
pub const PIN_CONFIG_ENV: &str = "GB_CERT_PINNING_CONFIG";
/// Explicit opt-out: when set to `1`/`true`, pinning is disabled even if a
/// config file is present. Absence of the config file already means "system
/// trust" (the default posture for local/dev).
pub const PIN_DISABLED_ENV: &str = "GB_CERT_PINNING_DISABLED";

static GLOBAL_MANAGER: OnceLock<Arc<CertPinningManager>> = OnceLock::new();

fn env_flag_set(name: &str) -> bool {
    matches!(
        std::env::var(name).ok().as_deref(),
        Some("1") | Some("true") | Some("TRUE") | Some("yes")
    )
}

fn disabled_config() -> CertPinningConfig {
    CertPinningConfig {
        enabled: false,
        ..CertPinningConfig::default()
    }
}

/// Process-wide pinning manager, initialized once from the deployment config.
///
/// Posture: with no `GB_CERT_PINNING_CONFIG` (or the explicit opt-out set) the
/// manager is disabled and outbound traffic uses the system trust store, which
/// keeps local/dev and existing deployments working. Operators opt in by
/// pointing the variable at a pinned host map; a malformed config yields a
/// *disabled* manager with a loud error, never a silently half-enforced one.
pub fn global_pinning_manager() -> &'static Arc<CertPinningManager> {
    GLOBAL_MANAGER.get_or_init(|| {
        if env_flag_set(PIN_DISABLED_ENV) {
            info!("Certificate pinning disabled via {PIN_DISABLED_ENV}");
            return Arc::new(CertPinningManager::new(disabled_config()));
        }
        let path = match std::env::var(PIN_CONFIG_ENV) {
            Ok(path) if !path.trim().is_empty() => path,
            _ => {
                debug!("No {PIN_CONFIG_ENV}; outbound TLS uses the system trust store");
                return Arc::new(CertPinningManager::new(disabled_config()));
            }
        };
        match CertPinningConfig::load_from_file(Path::new(&path)) {
            Ok(config) => {
                info!("Certificate pinning enabled from {path}");
                Arc::new(CertPinningManager::new(config))
            }
            Err(error) => {
                error!("Failed to load certificate pin config {path}: {error}");
                Arc::new(CertPinningManager::new(disabled_config()))
            }
        }
    })
}

impl CertPinningManager {
    /// Apply the pure pin decision to a presented leaf certificate.
    ///
    /// No cache is used: every handshake is evaluated against the current
    /// config so a rotated or malicious certificate is never accepted because
    /// an earlier handshake for the host was cached as valid.
    pub fn decision_for_cert(
        &self,
        hostname: &str,
        cert_der: &[u8],
    ) -> Result<PinDecision> {
        let config = self
            .config
            .read()
            .map_err(|_| anyhow!("Failed to acquire read lock"))?;

        if !config.enabled {
            return Ok(PinDecision::Accept);
        }

        let pins = config.get_pins(hostname);
        let has_pins = pins.is_some_and(|pins| !pins.is_empty());
        let matched = pins
            .map(|pins| pins.iter().any(|pin| pin.verify(cert_der).unwrap_or(false)))
            .unwrap_or(false);

        Ok(pin_decision(
            config.enabled,
            config.report_only,
            config.require_pins,
            has_pins,
            matched,
        ))
    }

    /// Build a rustls client config that verifies chains and enforces pins.
    pub fn build_pinned_tls_config(&self) -> Result<ClientConfig> {
        let roots = load_system_roots();
        if roots.roots.is_empty() {
            warn!("No system CA roots found; pinned TLS connections will be rejected");
        }
        let provider = Arc::new(rustls::crypto::ring::default_provider());
        let verifier = WebPkiServerVerifier::builder_with_provider(Arc::new(roots), provider.clone())
            .build()
            .context("Failed to build webpki verifier")?;
        let pin_verifier = PinningServerCertVerifier::new(
            verifier,
            Arc::new(self.clone()),
        );
        let mut config = ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .context("Failed to set TLS protocol versions")?
            .dangerous()
            .with_custom_certificate_verifier(Arc::new(pin_verifier))
            .with_no_client_auth();
        // reqwest does not set ALPN on a preconfigured config. Leave the
        // request-level builder to negotiate h2/http1.1 via its own settings.
        config.alpn_protocols.clear();
        Ok(config)
    }
}

/// A verifier that chains to the system roots and then checks the host's pins.
#[derive(Debug)]
struct PinningServerCertVerifier {
    inner: Arc<WebPkiServerVerifier>,
    manager: Arc<CertPinningManager>,
}

impl PinningServerCertVerifier {
    fn new(inner: Arc<WebPkiServerVerifier>, manager: Arc<CertPinningManager>) -> Self {
        Self { inner, manager }
    }
}

impl ServerCertVerifier for PinningServerCertVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        intermediates: &[CertificateDer<'_>],
        server_name: &ServerName<'_>,
        ocsp_response: &[u8],
        now: UnixTime,
    ) -> Result<ServerCertVerified, RustlsError> {
        // Standard chain + hostname + validity verification first.
        let verified = self.inner.verify_server_cert(
            end_entity,
            intermediates,
            server_name,
            ocsp_response,
            now,
        )?;

        let hostname = match server_name {
            ServerName::DnsName(name) => name.as_ref().to_string(),
            // IP literals have no SNI; a hostname-keyed pin map cannot apply.
            ServerName::IpAddress(_) => return Ok(verified),
            _ => return Ok(verified),
        };

        match self.manager.decision_for_cert(&hostname, end_entity.as_ref()) {
            Ok(PinDecision::Accept) => Ok(verified),
            Ok(PinDecision::Reject) => {
                warn!("Certificate pinning rejected {hostname}");
                Err(RustlsError::InvalidCertificate(
                    CertificateError::ApplicationVerificationFailure,
                ))
            }
            Err(error) => {
                // Fail closed on internal errors rather than accept unverified.
                error!("Certificate pinning check failed for {hostname}: {error}");
                Err(RustlsError::InvalidCertificate(
                    CertificateError::ApplicationVerificationFailure,
                ))
            }
        }
    }

    fn verify_tls12_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        self.inner.verify_tls12_signature(message, cert, dss)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, RustlsError> {
        self.inner.verify_tls13_signature(message, cert, dss)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.inner.supported_verify_schemes()
    }
}

/// Load trust anchors from the well-known CA bundle paths. Parsing reuses the
/// crate's PEM helper so no new dependency is introduced.
fn load_system_roots() -> RootCertStore {
    let mut roots = RootCertStore::empty();
    const PATHS: &[&str] = &[
        "/etc/ssl/certs/ca-certificates.crt",
        "/etc/pki/tls/certs/ca-bundle.crt",
        "/etc/ssl/cert.pem",
    ];
    for path in PATHS {
        let Ok(data) = fs::read(path) else { continue };
        let mut ders = Vec::new();
        let mut rest: &[u8] = &data;
        while let Some(der) = next_pem_block(&mut rest) {
            ders.push(CertificateDer::from(der));
        }
        let (added, _) = roots.add_parsable_certificates(ders);
        if added > 0 {
            debug!("Loaded {added} CA roots from {path}");
            break;
        }
    }
    roots
}

/// Extract the DER of the next PEM certificate block, advancing `rest`.
fn next_pem_block<'a>(rest: &mut &'a [u8]) -> Option<Vec<u8>> {
    const BEGIN: &[u8] = b"-----BEGIN CERTIFICATE-----";
    const END: &[u8] = b"-----END CERTIFICATE-----";
    let start = find_subslice(*rest, BEGIN)?;
    let body = &(*rest)[start + BEGIN.len()..];
    let end = find_subslice(body, END)?;
    let block_end = start + BEGIN.len() + end + END.len();
    let (block, tail) = rest.split_at(block_end);
    *rest = tail;
    x509_parser::pem::pem_to_der(block)
        .ok()
        .map(|(_rest, pem)| pem.contents.to_vec())
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcgen::generate_simple_self_signed;

    fn manager_with_pin(fingerprint: &str) -> Arc<CertPinningManager> {
        let mut config = CertPinningConfig::default();
        config.add_pin(PinnedCert::new("api.example.com", fingerprint));
        Arc::new(CertPinningManager::new(config))
    }

    fn fingerprint_of(der: &[u8]) -> String {
        format!("sha256//{}", BASE64.encode(compute_spki_fingerprint(der).unwrap()))
    }

    #[test]
    fn correct_pin_is_accepted() {
        let cert = generate_simple_self_signed(vec!["api.example.com".to_string()]).unwrap();
        let der = cert.cert.der().to_vec();
        let manager = manager_with_pin(&fingerprint_of(&der));
        assert_eq!(
            manager.decision_for_cert("api.example.com", &der).unwrap(),
            PinDecision::Accept
        );
    }

    #[test]
    fn wrong_pin_is_rejected_in_strict_mode() {
        let cert = generate_simple_self_signed(vec!["api.example.com".to_string()]).unwrap();
        let wrong = generate_simple_self_signed(vec!["api.example.com".to_string()]).unwrap();
        let manager = manager_with_pin(&fingerprint_of(wrong.cert.der()));
        let mut config = manager.config.read().unwrap().clone();
        config.require_pins = true;
        let strict = CertPinningManager::new(config);
        assert_eq!(
            strict
                .decision_for_cert("api.example.com", cert.cert.der())
                .unwrap(),
            PinDecision::Reject
        );
    }

    #[test]
    fn strict_mode_without_host_pins_fails_closed() {
        let config = CertPinningConfig::strict();
        let manager = CertPinningManager::new(config);
        let cert = generate_simple_self_signed(vec!["api.example.com".to_string()]).unwrap();
        assert_eq!(
            manager
                .decision_for_cert("api.example.com", cert.cert.der())
                .unwrap(),
            PinDecision::Reject
        );
    }

    #[test]
    fn backup_pin_is_accepted() {
        let cert = generate_simple_self_signed(vec!["api.example.com".to_string()]).unwrap();
        let pin = PinnedCert::backup("api.example.com", &fingerprint_of(cert.cert.der()));
        let mut config = CertPinningConfig::default();
        config.add_pin(pin);
        let manager = CertPinningManager::new(config);
        assert_eq!(
            manager
                .decision_for_cert("api.example.com", cert.cert.der())
                .unwrap(),
            PinDecision::Accept
        );
    }
}
