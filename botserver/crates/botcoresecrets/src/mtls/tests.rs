//! Unit tests for the mTLS configuration resolver (no Vault or env mutation).

use super::config::{
    MtlsConfig, RolloutStage, TransportMode, ENV_CA_CERT, ENV_MODE, FIELD_CA_CERT,
    FIELD_CLIENT_CERT, FIELD_CLIENT_KEY, FIELD_MODE, FIELD_SERVER_CERT, FIELD_SERVER_KEY,
    FIELD_STAGE,
};
use std::collections::BTreeMap;

fn sources(
    mode: &str,
    stage: &str,
    ca: &str,
    server_cert: &str,
    server_key: &str,
    client_cert: &str,
    client_key: &str,
) -> (BTreeMap<String, String>, BTreeMap<String, String>) {
    let mut vault = BTreeMap::new();
    vault.insert(FIELD_MODE.to_string(), mode.to_string());
    vault.insert(FIELD_STAGE.to_string(), stage.to_string());
    vault.insert(FIELD_CA_CERT.to_string(), ca.to_string());
    vault.insert(FIELD_SERVER_CERT.to_string(), server_cert.to_string());
    vault.insert(FIELD_SERVER_KEY.to_string(), server_key.to_string());
    vault.insert(FIELD_CLIENT_CERT.to_string(), client_cert.to_string());
    vault.insert(FIELD_CLIENT_KEY.to_string(), client_key.to_string());
    (vault, BTreeMap::new())
}

#[test]
fn test_transport_mode_parsing_fails_closed() {
    assert_eq!(TransportMode::parse("mtls"), TransportMode::MutualTls);
    assert_eq!(TransportMode::parse(" REQUIRED "), TransportMode::MutualTls);
    assert_eq!(TransportMode::parse("true"), TransportMode::MutualTls);
    assert_eq!(TransportMode::parse(""), TransportMode::Plaintext);
    assert_eq!(TransportMode::parse("typo"), TransportMode::Plaintext);
}

#[test]
fn test_stage_ordering_and_parsing() {
    assert!(RolloutStage::CertProvisioning < RolloutStage::ServerTls);
    assert!(RolloutStage::ServerTls < RolloutStage::ClientCertVerification);
    assert!(RolloutStage::ClientCertVerification < RolloutStage::PlaintextDisabled);
    assert_eq!(RolloutStage::parse("server-tls"), RolloutStage::ServerTls);
    assert_eq!(
        RolloutStage::parse("client-cert-verification"),
        RolloutStage::ClientCertVerification
    );
    assert_eq!(RolloutStage::parse("nonsense"), RolloutStage::CertProvisioning);
}

#[test]
fn test_environment_overrides_vault_and_blank_falls_through() {
    let (vault, mut environment) = sources("mtls", "server-tls", "", "", "", "", "");
    environment.insert(ENV_MODE.to_string(), "plaintext".to_string());
    environment.insert(ENV_CA_CERT.to_string(), "   ".to_string());
    let config = MtlsConfig::from_sources(&vault, &environment);
    assert_eq!(config.mode, TransportMode::Plaintext);
    assert_eq!(config.stage, RolloutStage::ServerTls);
    assert!(config.ca_cert_path.is_empty());
}

#[test]
fn test_missing_requirements_follow_stage() {
    let (vault, environment) = sources("mtls", "server-tls", "", "", "", "", "");
    let config = MtlsConfig::from_sources(&vault, &environment);
    assert_eq!(
        config.missing_requirements(),
        vec!["ca_cert", "server_cert", "server_key"]
    );

    let (vault, environment) = sources(
        "mtls",
        "client-cert-verification",
        "/ca.pem",
        "/server.pem",
        "/server.key",
        "",
        "",
    );
    let config = MtlsConfig::from_sources(&vault, &environment);
    assert_eq!(config.missing_requirements(), vec!["client_cert", "client_key"]);

    let (vault, environment) = sources("plaintext", "plaintext-disabled", "", "", "", "", "");
    let config = MtlsConfig::from_sources(&vault, &environment);
    assert!(config.missing_requirements().is_empty());
}

#[test]
fn test_validate_accepts_plaintext_and_rejects_missing_files() {
    assert!(MtlsConfig::default().validate().is_ok());

    let manifest = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
    let (vault, environment) = sources(
        "mtls",
        "server-tls",
        manifest,
        manifest,
        manifest,
        "",
        "",
    );
    let config = MtlsConfig::from_sources(&vault, &environment);
    assert!(config.validate().is_ok());

    let (vault, environment) = sources(
        "mtls",
        "server-tls",
        manifest,
        manifest,
        "/definitely/not/a/generalbots/key.pem",
        "",
        "",
    );
    let broken = MtlsConfig::from_sources(&vault, &environment);
    assert!(broken.validate().is_err());
}

#[test]
fn test_status_booleans_reflect_opt_in() {
    let disabled = MtlsConfig::default().status();
    assert!(disabled.supported);
    assert!(!disabled.enabled);

    let (vault, environment) = sources("mtls", "cert-provisioning", "", "", "", "", "");
    let enabled = MtlsConfig::from_sources(&vault, &environment).status();
    assert!(enabled.supported);
    assert!(enabled.enabled);
    assert_eq!(enabled.mode, TransportMode::MutualTls);
}
