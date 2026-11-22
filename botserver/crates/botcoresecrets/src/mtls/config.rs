//! Resolution of the mTLS configuration from environment and Vault sources.
//! Values are paths and mode/stage selectors only — never certificate material.

use super::status::MtlsStatus;
use crate::paths::SecretPaths;
use crate::sync::secret_sync;
use anyhow::{anyhow, Result};
use std::collections::BTreeMap;
use std::env;
use std::path::Path;

/// Env var that explicitly opts a containerized install into mTLS.
pub const ENV_MODE: &str = "GB_MTLS_MODE";
/// Env var selecting the rollout stage.
pub const ENV_STAGE: &str = "GB_MTLS_STAGE";
/// Env var holding the path to the internal CA bundle (PEM).
pub const ENV_CA_CERT: &str = "GB_MTLS_CA_CERT";
/// Env var holding the path to this service's server certificate (PEM).
pub const ENV_SERVER_CERT: &str = "GB_MTLS_SERVER_CERT";
/// Env var holding the path to this service's server private key (PEM).
pub const ENV_SERVER_KEY: &str = "GB_MTLS_SERVER_KEY";
/// Env var holding the path to this service's client certificate (PEM).
pub const ENV_CLIENT_CERT: &str = "GB_MTLS_CLIENT_CERT";
/// Env var holding the path to this service's client private key (PEM).
pub const ENV_CLIENT_KEY: &str = "GB_MTLS_CLIENT_KEY";

/// Matching Vault field names under [`SecretPaths::SECURITY`].
pub(super) const FIELD_MODE: &str = "mtls_mode";
pub(super) const FIELD_STAGE: &str = "mtls_stage";
pub(super) const FIELD_CA_CERT: &str = "mtls_ca_cert";
pub(super) const FIELD_SERVER_CERT: &str = "mtls_server_cert";
pub(super) const FIELD_SERVER_KEY: &str = "mtls_server_key";
pub(super) const FIELD_CLIENT_CERT: &str = "mtls_client_cert";
pub(super) const FIELD_CLIENT_KEY: &str = "mtls_client_key";

/// Transport mode for inter-service traffic. Never inferred: plaintext is the
/// default and desktop-local installs remain there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportMode {
    /// Plaintext on loopback or a trusted private bridge (default).
    Plaintext,
    /// Mutual TLS required between services (explicit opt-in only).
    MutualTls,
}

impl Default for TransportMode {
    fn default() -> Self {
        Self::Plaintext
    }
}

impl TransportMode {
    /// Parse an operator-supplied value. Unknown/blank values keep plaintext
    /// so a typo can never silently enable a half-configured secure mode.
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "mtls" | "mutual" | "mutual_tls" | "mutual-tls" | "required" | "secure" | "true"
            | "1" => Self::MutualTls,
            _ => Self::Plaintext,
        }
    }

    /// Stable value for logs and admin reporting.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Plaintext => "plaintext",
            Self::MutualTls => "mtls",
        }
    }
}

/// Staged rollout. Ordering matters: each stage is a prerequisite of the next.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RolloutStage {
    /// CA and per-service certificates provisioned; no behaviour change.
    CertProvisioning = 0,
    /// Servers present certificates; clients verify server identity.
    ServerTls = 1,
    /// Clients present certificates; servers verify them.
    ClientCertVerification = 2,
    /// Plaintext listeners removed or firewalled after stage 2 is green.
    PlaintextDisabled = 3,
}

impl Default for RolloutStage {
    fn default() -> Self {
        Self::CertProvisioning
    }
}

impl RolloutStage {
    /// Parse a stage value. Unknown/blank values fall back to stage 0.
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "server-tls" | "server_tls" | "server" | "1" => Self::ServerTls,
            "client-cert"
            | "client-cert-verification"
            | "client_cert_verification"
            | "mutual"
            | "2" => Self::ClientCertVerification,
            "plaintext-disabled" | "plaintext_disabled" | "no-plaintext" | "3" => {
                Self::PlaintextDisabled
            }
            _ => Self::CertProvisioning,
        }
    }

    /// Stable value for logs and admin reporting.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CertProvisioning => "cert-provisioning",
            Self::ServerTls => "server-tls",
            Self::ClientCertVerification => "client-cert-verification",
            Self::PlaintextDisabled => "plaintext-disabled",
        }
    }
}

/// Resolved mTLS configuration. Holds paths only — never secret material.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MtlsConfig {
    pub mode: TransportMode,
    pub stage: RolloutStage,
    pub ca_cert_path: String,
    pub server_cert_path: String,
    pub server_key_path: String,
    pub client_cert_path: String,
    pub client_key_path: String,
}

impl Default for MtlsConfig {
    fn default() -> Self {
        Self {
            mode: TransportMode::Plaintext,
            stage: RolloutStage::CertProvisioning,
            ca_cert_path: String::new(),
            server_cert_path: String::new(),
            server_key_path: String::new(),
            client_cert_path: String::new(),
            client_key_path: String::new(),
        }
    }
}

/// Precedence: a non-blank environment value wins, then a non-blank Vault
/// field, then the supplied default. Blank means "not set", not "empty path".
fn select(
    vault: &BTreeMap<String, String>,
    environment: &BTreeMap<String, String>,
    field: &str,
    env_key: &str,
    default: &str,
) -> String {
    if let Some(value) = environment.get(env_key) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    match vault.get(field) {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                default.to_string()
            } else {
                trimmed.to_string()
            }
        }
        None => default.to_string(),
    }
}

impl MtlsConfig {
    /// Resolve from explicit sources. Pure so it is unit-testable without a
    /// running Vault or environment mutation.
    #[must_use]
    pub fn from_sources(
        vault: &BTreeMap<String, String>,
        environment: &BTreeMap<String, String>,
    ) -> Self {
        Self {
            mode: TransportMode::parse(&select(
                vault,
                environment,
                FIELD_MODE,
                ENV_MODE,
                "plaintext",
            )),
            stage: RolloutStage::parse(&select(
                vault,
                environment,
                FIELD_STAGE,
                ENV_STAGE,
                "cert-provisioning",
            )),
            ca_cert_path: select(vault, environment, FIELD_CA_CERT, ENV_CA_CERT, ""),
            server_cert_path: select(vault, environment, FIELD_SERVER_CERT, ENV_SERVER_CERT, ""),
            server_key_path: select(vault, environment, FIELD_SERVER_KEY, ENV_SERVER_KEY, ""),
            client_cert_path: select(vault, environment, FIELD_CLIENT_CERT, ENV_CLIENT_CERT, ""),
            client_key_path: select(vault, environment, FIELD_CLIENT_KEY, ENV_CLIENT_KEY, ""),
        }
    }

    /// Resolve from Vault (best effort) with environment overrides.
    pub fn load() -> Result<Self> {
        let vault = match secret_sync(SecretPaths::SECURITY) {
            Some(map) => map,
            None => BTreeMap::new(),
        };
        let mut environment = BTreeMap::new();
        for key in [
            ENV_MODE,
            ENV_STAGE,
            ENV_CA_CERT,
            ENV_SERVER_CERT,
            ENV_SERVER_KEY,
            ENV_CLIENT_CERT,
            ENV_CLIENT_KEY,
        ] {
            if let Ok(value) = env::var(key) {
                environment.insert(key.to_string(), value);
            }
        }
        Ok(Self::from_sources(&vault, &environment))
    }

    /// Names of required settings that are unset at the selected stage.
    /// Values and paths are never included, so diagnostics cannot leak layout.
    #[must_use]
    pub fn missing_requirements(&self) -> Vec<&'static str> {
        let mut missing = Vec::new();
        if self.mode != TransportMode::MutualTls {
            return missing;
        }
        if self.stage >= RolloutStage::ServerTls {
            if self.ca_cert_path.is_empty() {
                missing.push("ca_cert");
            }
            if self.server_cert_path.is_empty() {
                missing.push("server_cert");
            }
            if self.server_key_path.is_empty() {
                missing.push("server_key");
            }
        }
        if self.stage >= RolloutStage::ClientCertVerification {
            if self.client_cert_path.is_empty() {
                missing.push("client_cert");
            }
            if self.client_key_path.is_empty() {
                missing.push("client_key");
            }
        }
        missing
    }

    /// Fail-closed check used at bootstrap. Stage 0 needs no paths because it
    /// changes no behaviour; stages 1+ must point at existing PEM files.
    pub fn validate(&self) -> Result<()> {
        let missing = self.missing_requirements();
        if !missing.is_empty() {
            return Err(anyhow!("mTLS enabled but unset: {}", missing.join(", ")));
        }
        if self.mode != TransportMode::MutualTls {
            return Ok(());
        }
        for (label, path) in [
            ("ca_cert", self.ca_cert_path.as_str()),
            ("server_cert", self.server_cert_path.as_str()),
            ("server_key", self.server_key_path.as_str()),
            ("client_cert", self.client_cert_path.as_str()),
            ("client_key", self.client_key_path.as_str()),
        ] {
            if !path.is_empty() && !Path::new(path).is_file() {
                return Err(anyhow!("mTLS {label} path is not a file"));
            }
        }
        Ok(())
    }

    /// Small status snapshot for logging and admin reporting.
    #[must_use]
    pub fn status(&self) -> MtlsStatus {
        MtlsStatus {
            supported: true,
            enabled: self.mode == TransportMode::MutualTls,
            mode: self.mode,
            stage: self.stage,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(entries: &[(&str, &str)]) -> BTreeMap<String, String> {
        entries
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    fn modes(values: &[&str], expected: TransportMode) {
        for value in values {
            assert_eq!(TransportMode::parse(value), expected, "mode: {value}");
        }
    }

    fn stages(values: &[&str], expected: RolloutStage) {
        for value in values {
            assert_eq!(RolloutStage::parse(value), expected, "stage: {value}");
        }
    }

    fn select_ca(vault: &BTreeMap<String, String>, env: &BTreeMap<String, String>) -> String {
        select(vault, env, FIELD_CA_CERT, ENV_CA_CERT, "/default.pem")
    }

    #[test]
    fn mode_grammar_accepts_all_documented_aliases() {
        let mt = TransportMode::MutualTls;
        modes(
            &["mtls", "mutual", "mutual_tls", "mutual-tls", "required"],
            mt,
        );
        modes(&["secure", "true", "1"], mt);
        assert_eq!(TransportMode::MutualTls.as_str(), "mtls");
        assert_eq!(TransportMode::Plaintext.as_str(), "plaintext");
    }

    #[test]
    fn stage_grammar_accepts_all_documented_aliases() {
        let st = RolloutStage::ServerTls;
        let cc = RolloutStage::ClientCertVerification;
        let pd = RolloutStage::PlaintextDisabled;
        stages(&["server-tls", "server_tls", "server", "1"], st);
        stages(&["SERVER_TLS", " server "], st);
        stages(&["client-cert", "client-cert-verification"], cc);
        stages(
            &["CLIENT_CERT_VERIFICATION", "client_cert_verification"],
            cc,
        );
        stages(&["mutual", "2"], cc);
        stages(&["plaintext-disabled", "plaintext_disabled"], pd);
        stages(&["no-plaintext", "3"], pd);
        assert_eq!(RolloutStage::CertProvisioning.as_str(), "cert-provisioning");
    }

    #[test]
    fn typos_fail_closed_to_plaintext_and_stage_zero() {
        let pt = TransportMode::Plaintext;
        let zero = RolloutStage::CertProvisioning;
        modes(
            &[
                "",
                "   ",
                "mtl",
                "tls",
                "yes",
                "mtls!",
                "0",
                "2x",
                " plaintext ",
            ],
            pt,
        );
        stages(
            &[
                "",
                "nonsense",
                "0",
                "4",
                "server-tls-extra",
                "client",
                "server1",
            ],
            zero,
        );
    }

    #[test]
    fn select_precedence_env_over_vault_over_default() {
        let empty = map(&[]);
        let vault = map(&[(FIELD_CA_CERT, "/vault/ca.pem")]);
        let env = map(&[(ENV_CA_CERT, "/env/ca.pem")]);
        assert_eq!(select_ca(&empty, &empty), "/default.pem");
        assert_eq!(select_ca(&vault, &empty), "/vault/ca.pem");
        assert_eq!(select_ca(&vault, &env), "/env/ca.pem");
    }

    #[test]
    fn blank_values_are_treated_as_unset() {
        let empty = map(&[]);
        let blank_env = map(&[(ENV_CA_CERT, "   ")]);
        let vault = map(&[(FIELD_CA_CERT, "/vault/ca.pem")]);
        let blank_vault = map(&[(FIELD_CA_CERT, "\t")]);
        assert_eq!(select_ca(&vault, &blank_env), "/vault/ca.pem");
        assert_eq!(select_ca(&blank_vault, &empty), "/default.pem");
        assert_eq!(select_ca(&blank_vault, &blank_env), "/default.pem");
    }

    #[test]
    fn from_sources_applies_precedence_then_fails_closed() {
        let vault = map(&[(FIELD_MODE, "mtls"), (FIELD_STAGE, "server-tls")]);
        let typo_env = map(&[(ENV_MODE, "mtlss")]);
        let config = MtlsConfig::from_sources(&vault, &typo_env);
        assert_eq!(config.mode, TransportMode::Plaintext);
        assert_eq!(config.stage, RolloutStage::ServerTls);
        let blank_env = map(&[(ENV_MODE, "  ")]);
        let config = MtlsConfig::from_sources(&vault, &blank_env);
        assert_eq!(config.mode, TransportMode::MutualTls);
    }
}
