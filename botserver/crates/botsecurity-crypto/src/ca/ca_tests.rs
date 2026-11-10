use super::*;
use tempfile::TempDir;
use x509_parser::prelude::FromDer;

fn der_of(cert_pem: &str) -> Vec<u8> {
    use base64::{engine::general_purpose::STANDARD, Engine as _};

    let start = cert_pem.find("-----BEGIN CERTIFICATE-----").expect("begin");
    let end = cert_pem.find("-----END CERTIFICATE-----").expect("end");
    let body: String = cert_pem[start + "-----BEGIN CERTIFICATE-----".len()..end]
        .chars()
        .filter(|c| !c.is_whitespace())
        .collect();

    STANDARD.decode(body).expect("base64")
}

fn temp_config(dir: &TempDir) -> CaConfig {
    CaConfig {
        ca_cert_path: dir.path().join("ca.crt"),
        ca_key_path: dir.path().join("ca.key"),
        intermediate_cert_path: Some(dir.path().join("intermediate.crt")),
        intermediate_key_path: Some(dir.path().join("intermediate.key")),
        crl_path: Some(dir.path().join("crl.pem")),
        ..CaConfig::default()
    }
}

fn initialized_ca(dir: &TempDir) -> CaManager {
    let mut manager = CaManager::new(temp_config(dir)).expect("manager");
    manager.generate_root_ca().expect("root ca");
    manager
}

fn serial_of(cert_pem: &str) -> String {
    let der = der_of(cert_pem);
    let (_, cert) =
        x509_parser::certificate::X509Certificate::from_der(&der).expect("parse certificate");
    cert.raw_serial_as_string().to_uppercase()
}

#[test]
fn manager_without_ca_cannot_issue_certificates() {
    let dir = TempDir::new().expect("tempdir");
    let manager = CaManager::new(temp_config(&dir)).expect("manager");

    let err = manager
        .issue_certificate("example.com", vec![], false)
        .unwrap_err();
    assert!(err.to_string().contains("CA not initialized"));
}

#[test]
fn generate_root_ca_writes_pem_files() {
    let dir = TempDir::new().expect("tempdir");
    let config = temp_config(&dir);
    let manager = initialized_ca(&dir);

    assert!(config.ca_cert_path.exists());
    assert!(config.ca_key_path.exists());

    let cert = fs::read_to_string(&config.ca_cert_path).expect("cert");
    assert!(cert.starts_with("-----BEGIN CERTIFICATE-----"));
    assert!(manager.ca_key.is_some());
    assert!(manager.ca_params.is_some());
}

#[test]
fn issued_certificate_carries_common_name_and_subject_alt_names() {
    let dir = TempDir::new().expect("tempdir");
    let manager = initialized_ca(&dir);

    let (cert_pem, key_pem) = manager
        .issue_certificate(
            "api.example.com",
            vec!["api.example.com".to_string(), "10.1.2.3".to_string()],
            false,
        )
        .expect("issue");

    assert!(key_pem.contains("PRIVATE KEY"));

    let der = der_of(&cert_pem);
    let (_, cert) = x509_parser::certificate::X509Certificate::from_der(&der).expect("parse");

    let cn = cert
        .subject()
        .iter_common_name()
        .next()
        .and_then(|cn| cn.as_str().ok())
        .expect("common name");
    assert_eq!(cn, "api.example.com");

    let sans = cert
        .subject_alternative_name()
        .expect("san extension")
        .expect("san present")
        .value
        .general_names
        .clone();

    use x509_parser::extensions::GeneralName;
    assert!(sans
        .iter()
        .any(|name| matches!(name, GeneralName::DNSName(dns) if *dns == "api.example.com")));
    assert!(sans
        .iter()
        .any(|name| matches!(name, GeneralName::IPAddress(ip) if *ip == [10, 1, 2, 3])));
}

#[test]
fn client_certificates_use_client_auth_usage() {
    let dir = TempDir::new().expect("tempdir");
    let manager = initialized_ca(&dir);

    let (cert_pem, _) = manager
        .issue_certificate("client.example.com", vec![], true)
        .expect("issue");

    let der = der_of(&cert_pem);
    let (_, cert) = x509_parser::certificate::X509Certificate::from_der(&der).expect("parse");
    let eku = cert
        .extended_key_usage()
        .expect("eku")
        .expect("eku present")
        .value;

    assert!(eku.client_auth);
    assert!(!eku.server_auth);
}

#[test]
fn load_ca_reloads_an_existing_keypair() {
    let dir = TempDir::new().expect("tempdir");
    initialized_ca(&dir);

    let reloaded = CaManager::new(temp_config(&dir)).expect("reload");
    assert!(reloaded.ca_key.is_some());
    assert!(reloaded.ca_params.is_some());

    let (cert_pem, _) = reloaded
        .issue_certificate("reloaded.example.com", vec![], false)
        .expect("issue after reload");
    assert!(cert_pem.contains("BEGIN CERTIFICATE"));
}

#[test]
fn verify_rejects_missing_ca_or_invalid_pem() {
    let dir = TempDir::new().expect("tempdir");
    let manager = CaManager::new(temp_config(&dir)).expect("manager");

    assert!(!manager.verify_certificate("-----BEGIN CERTIFICATE-----").expect("verify"));

    fs::write(&temp_config(&dir).ca_cert_path, "ca").expect("write ca");
    assert!(!manager.verify_certificate("").expect("empty"));
    assert!(!manager
        .verify_certificate("not a certificate")
        .expect("garbage"));
}

#[test]
fn verify_accepts_a_live_certificate_and_rejects_a_revoked_serial() {
    let dir = TempDir::new().expect("tempdir");
    let manager = initialized_ca(&dir);
    let (cert_pem, _) = manager
        .issue_certificate("live.example.com", vec![], false)
        .expect("issue");

    assert!(manager.verify_certificate(&cert_pem).expect("verify"));

    let serial = serial_of(&cert_pem);
    manager
        .revoke_certificate(&serial, "key compromise")
        .expect("revoke");

    assert!(!manager.verify_certificate(&cert_pem).expect("verify"));

    let crl_path = temp_config(&dir).ca_cert_path.with_extension("crl");
    let crl = fs::read_to_string(crl_path).expect("crl");
    assert!(crl.contains("BEGIN X509 CRL"));
    assert!(crl.contains("key compromise"));
}

#[test]
fn revoke_matches_serials_case_insensitively() {
    let dir = TempDir::new().expect("tempdir");
    let manager = initialized_ca(&dir);
    let (cert_pem, _) = manager
        .issue_certificate("case.example.com", vec![], false)
        .expect("issue");

    manager
        .revoke_certificate(&serial_of(&cert_pem).to_lowercase(), "rotation")
        .expect("revoke");

    assert!(!manager.verify_certificate(&cert_pem).expect("verify"));
}

#[test]
fn revocation_entries_accumulate_in_the_crl() {
    let dir = TempDir::new().expect("tempdir");
    let manager = initialized_ca(&dir);

    manager.revoke_certificate("AAAA", "first").expect("revoke");
    manager.revoke_certificate("BBBB", "second").expect("revoke");

    let revoked = fs::read_to_string(temp_config(&dir).ca_cert_path.with_extension("revoked"))
        .expect("revoked list");
    assert_eq!(revoked.lines().count(), 2);
    assert!(revoked.contains("AAAA|first|"));
    assert!(revoked.contains("BBBB|second|"));
}

#[test]
fn external_ca_sync_is_a_noop_when_disabled() {
    let dir = TempDir::new().expect("tempdir");
    let manager = CaManager::new(temp_config(&dir)).expect("manager");

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("runtime");
    runtime
        .block_on(manager.sync_with_external_ca())
        .expect("sync");
}
