use super::*;

#[test]
fn default_config_requires_tls_1_3_and_a_ca_bundle() {
    let config = TlsConfig::default();

    assert!(config.enabled);
    assert_eq!(config.min_tls_version.as_deref(), Some("1.3"));
    assert!(config.ca_cert_path.is_some());
    assert!(!config.require_client_cert);
    assert_eq!(config.renewal_check_hours, 24);
}

#[test]
fn service_config_derives_certificate_paths_from_the_service_name() {
    let service = ServiceTlsConfig::new("api", 8443);

    assert_eq!(service.service_name, "api");
    assert_eq!(service.port, 8443);
    assert_eq!(service.tls_config.cert_path, PathBuf::from("certs/api/server.crt"));
    assert_eq!(service.tls_config.key_path, PathBuf::from("certs/api/server.key"));
    assert_eq!(
        service.tls_config.client_cert_path,
        Some(PathBuf::from("certs/api/client.crt"))
    );
    assert!(!service.tls_config.require_client_cert);
}

#[test]
fn service_config_builders_toggle_mtls_and_ca() {
    let service = ServiceTlsConfig::new("llm", 8081)
        .with_mtls()
        .with_ca(PathBuf::from("/etc/gb/ca.crt"));

    assert!(service.tls_config.require_client_cert);
    assert_eq!(
        service.tls_config.ca_cert_path,
        Some(PathBuf::from("/etc/gb/ca.crt"))
    );
}

#[test]
fn registry_defaults_register_every_internal_service() {
    let mut registry = TlsRegistry::new();
    assert!(registry.services().is_empty());

    registry.register_defaults();

    let names: Vec<&str> = registry
        .services()
        .iter()
        .map(|s| s.service_name.as_str())
        .collect();

    assert_eq!(names.len(), 10);
    for expected in [
        "api",
        "llm",
        "embedding",
        "qdrant",
        "redis",
        "postgres",
        "minio",
        "directory",
        "email",
        "meet",
    ] {
        assert!(names.contains(&expected), "missing service {expected}");
    }
    assert!(registry
        .services()
        .iter()
        .all(|s| s.tls_config.require_client_cert));
}

#[test]
fn registry_lookup_reports_unknown_services() {
    let mut registry = TlsRegistry::default();
    registry.register_defaults();

    let err = registry.get_manager("nope").unwrap_err();
    assert!(err.to_string().contains("Service nope not found"));
}

#[test]
fn manager_reports_a_missing_certificate_file() {
    let config = TlsConfig {
        cert_path: PathBuf::from("/nonexistent/server.crt"),
        key_path: PathBuf::from("/nonexistent/server.key"),
        ..TlsConfig::default()
    };

    let err = TlsManager::new(config).unwrap_err();
    assert!(err.to_string().contains("Failed to open certificate file"));
}

#[test]
fn manager_requires_a_ca_when_client_certs_are_mandatory() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (cert_path, key_path) = write_keypair(dir.path());

    let config = TlsConfig {
        cert_path,
        key_path,
        ca_cert_path: None,
        client_cert_path: None,
        client_key_path: None,
        require_client_cert: true,
        ..TlsConfig::default()
    };

    let err = TlsManager::new(config).unwrap_err();
    assert!(err.to_string().contains("CA certificate required for mTLS"));
}

#[test]
fn manager_builds_server_and_client_configs_from_a_real_keypair() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (cert_path, key_path) = write_keypair(dir.path());

    let config = TlsConfig {
        cert_path: cert_path.clone(),
        key_path: key_path.clone(),
        ca_cert_path: Some(cert_path.clone()),
        client_cert_path: Some(cert_path),
        client_key_path: Some(key_path),
        ..TlsConfig::default()
    };

    let mut manager = TlsManager::new(config).expect("manager");
    assert!(manager.client_config().is_some());
    assert!(manager.create_https_client().is_ok());
    assert!(!manager.check_certificate_renewal().expect("renewal"));
    assert!(manager.reload_certificates().is_ok());
    let _ = manager.acceptor();
}

#[test]
fn manager_without_client_certificates_has_no_client_config() {
    let dir = tempfile::tempdir().expect("tempdir");
    let (cert_path, key_path) = write_keypair(dir.path());

    let config = TlsConfig {
        cert_path,
        key_path,
        ca_cert_path: None,
        client_cert_path: None,
        client_key_path: None,
        ..TlsConfig::default()
    };

    let manager = TlsManager::new(config).expect("manager");
    assert!(manager.client_config().is_none());
    assert!(manager.create_https_client().is_ok());
}

fn write_keypair(dir: &Path) -> (PathBuf, PathBuf) {
    let _ = rustls::crypto::ring::default_provider().install_default();

    let cert = rcgen::generate_simple_self_signed(vec!["localhost".to_string()])
        .expect("self-signed certificate");

    let cert_path = dir.join("server.crt");
    std::fs::write(&cert_path, cert.cert.pem()).expect("write cert");
    let key_path = dir.join("server.key");
    std::fs::write(&key_path, cert.signing_key.serialize_pem()).expect("write key");

    (cert_path, key_path)
}
