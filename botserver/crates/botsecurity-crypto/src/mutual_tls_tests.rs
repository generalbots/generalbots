use super::services::*;
use super::*;

fn write(dir: &Path, name: &str, content: &str) -> std::path::PathBuf {
    let path = dir.join(name);
    std::fs::write(&path, content).expect("write fixture");
    path
}

const CERT_PEM: &str = "-----BEGIN CERTIFICATE-----\nMIIB\n-----END CERTIFICATE-----\n";
const KEY_PEM: &str = "-----BEGIN PRIVATE KEY-----\nMIIB\n-----END PRIVATE KEY-----\n";

#[test]
fn new_requires_all_three_parts() {
    assert!(!MtlsConfig::new(None, None, None).enabled);
    assert!(!MtlsConfig::new(Some("ca".into()), None, None).enabled);
    assert!(!MtlsConfig::new(Some("ca".into()), Some("cert".into()), None).enabled);

    let full = MtlsConfig::new(Some("ca".into()), Some("cert".into()), Some("key".into()));
    assert!(full.enabled);
    assert!(full.is_configured());
}

#[test]
fn default_config_is_not_configured() {
    assert!(!MtlsConfig::default().is_configured());
    assert!(!MtlsManager::new(MtlsConfig::default()).is_enabled());
}

#[test]
fn manager_accepts_well_formed_pem_material() {
    let config = MtlsConfig::new(
        Some(CERT_PEM.into()),
        Some(CERT_PEM.into()),
        Some(KEY_PEM.into()),
    );

    assert!(MtlsManager::new(config).validate().is_ok());
}

#[test]
fn manager_rejects_a_non_pem_ca() {
    let config = MtlsConfig::new(
        Some("not a cert".into()),
        Some(CERT_PEM.into()),
        Some(KEY_PEM.into()),
    );

    let err = MtlsManager::new(config).validate().unwrap_err();
    assert!(matches!(err, MtlsError::InvalidCertificate(_)));
    assert!(err.to_string().contains("CA certificate"));
}

#[test]
fn manager_rejects_a_non_pem_client_certificate() {
    let config = MtlsConfig::new(
        Some(CERT_PEM.into()),
        Some("not a cert".into()),
        Some(KEY_PEM.into()),
    );

    let err = MtlsManager::new(config).validate().unwrap_err();
    assert!(err.to_string().contains("Client certificate"));
}

#[test]
fn manager_rejects_a_non_pem_key() {
    let config = MtlsConfig::new(
        Some(CERT_PEM.into()),
        Some(CERT_PEM.into()),
        Some("-----BEGIN SOMETHING-----".into()),
    );

    let err = MtlsManager::new(config).validate().unwrap_err();
    assert!(matches!(err, MtlsError::InvalidKey(_)));
}

#[test]
fn disabled_manager_skips_validation() {
    let config = MtlsConfig::new(Some("junk".into()), Some("junk".into()), Some("junk".into()));
    let manager = MtlsManager::new(MtlsConfig {
        enabled: false,
        ..config
    });

    assert!(manager.validate().is_ok());
}

#[test]
fn postgres_uses_verify_full_when_all_paths_exist() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ca = write(dir.path(), "ca.pem", CERT_PEM);
    let cert = write(dir.path(), "client.pem", CERT_PEM);
    let key = write(dir.path(), "client.key", KEY_PEM);

    let mode = configure_postgres_mtls(Some(&ca), Some(&cert), Some(&key)).expect("mode");
    assert_eq!(mode, "verify-full");
}

#[test]
fn postgres_uses_verify_ca_with_only_a_ca() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ca = write(dir.path(), "ca.pem", CERT_PEM);

    assert_eq!(
        configure_postgres_mtls(Some(&ca), None, None).expect("mode"),
        "verify-ca"
    );
}

#[test]
fn postgres_falls_back_to_prefer_without_certificates() {
    assert_eq!(
        configure_postgres_mtls(None, None, None).expect("mode"),
        "prefer"
    );
}

#[test]
fn postgres_reports_missing_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ca = write(dir.path(), "ca.pem", CERT_PEM);
    let missing = dir.path().join("absent.pem");

    let err = configure_postgres_mtls(Some(&missing), None, None).unwrap_err();
    assert!(matches!(err, MtlsError::CertificateNotFound(_)));

    let err = configure_postgres_mtls(Some(&ca), Some(&missing), Some(&missing)).unwrap_err();
    assert!(matches!(err, MtlsError::CertificateNotFound(_)));

    let err = configure_postgres_mtls(Some(&ca), Some(&ca), Some(&missing)).unwrap_err();
    assert!(matches!(err, MtlsError::KeyNotFound(_)));
}

#[test]
fn service_configs_load_pem_contents_when_all_paths_are_present() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ca = write(dir.path(), "ca.pem", CERT_PEM);
    let cert = write(dir.path(), "client.pem", CERT_PEM);
    let key = write(dir.path(), "client.key", KEY_PEM);

    let configs = [
        configure_qdrant_mtls(Some(&ca), Some(&cert), Some(&key)),
        configure_livekit_mtls(Some(&ca), Some(&cert), Some(&key)),
        configure_forgejo_mtls(Some(&ca), Some(&cert), Some(&key)),
        configure_directory_mtls(Some(&ca), Some(&cert), Some(&key)),
    ];

    for config in configs {
        let config = config.expect("config");
        assert!(config.enabled);
        assert_eq!(config.ca_cert.as_deref(), Some(CERT_PEM));
        assert_eq!(config.client_key.as_deref(), Some(KEY_PEM));
        assert!(MtlsManager::new(config).validate().is_ok());
    }
}

#[test]
fn service_configs_are_disabled_without_all_paths() {
    let dir = tempfile::tempdir().expect("tempdir");
    let ca = write(dir.path(), "ca.pem", CERT_PEM);

    let config = configure_qdrant_mtls(Some(&ca), None, None).expect("config");
    assert!(!config.enabled);
    assert!(config.ca_cert.is_none());
}

#[test]
fn service_configs_surface_io_errors() {
    let dir = tempfile::tempdir().expect("tempdir");
    let missing = dir.path().join("absent.pem");

    let err = configure_livekit_mtls(Some(&missing), Some(&missing), Some(&missing)).unwrap_err();
    assert!(matches!(err, MtlsError::IoError(_)));
    assert!(err.to_string().contains("Failed to read CA cert"));
}
