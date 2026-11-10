use super::*;

fn self_signed_der() -> Vec<u8> {
    let cert = rcgen::generate_simple_self_signed(vec!["example.com".to_string()])
        .expect("self-signed certificate");
    cert.cert.der().to_vec()
}

fn pin_for(hostname: &str, der: &[u8]) -> PinnedCert {
    let hash = compute_spki_fingerprint(der).expect("fingerprint");
    PinnedCert::new(hostname, &format!("sha256//{}", BASE64.encode(hash)))
}

#[test]
fn pinned_cert_verifies_matching_certificate() {
    let der = self_signed_der();
    let pin = pin_for("example.com", &der);

    assert!(pin.verify(&der).expect("verify"));
    assert_eq!(pin.get_hash_bytes().expect("hash").len(), 32);
}

#[test]
fn pinned_cert_rejects_a_different_certificate() {
    let der = self_signed_der();
    let other = self_signed_der();
    let pin = pin_for("example.com", &der);

    assert!(!pin.verify(&other).expect("verify"));
}

#[test]
fn pinned_cert_rejects_plain_hex_fingerprints() {
    let der = self_signed_der();
    let hash = compute_spki_fingerprint(&der).expect("fingerprint");
    let pin = PinnedCert::new("example.com", &format_fingerprint(&hash));

    assert!(pin.get_hash_bytes().is_err());
}

#[test]
fn backup_pin_is_marked_as_backup() {
    let pin = PinnedCert::backup("example.com", "sha256//AAAA");

    assert!(pin.is_backup);
    assert!(pin.description.is_some());
    assert_eq!(pin.pin_type, PinType::Leaf);
}

#[test]
fn config_add_and_remove_pins_are_keyed_by_hostname() {
    let mut config = CertPinningConfig::default();
    config.add_pin(PinnedCert::new("a.example", "sha256//AAAA"));
    config.add_pin(PinnedCert::new("a.example", "sha256//BBBB"));
    config.add_pin(PinnedCert::new("b.example", "sha256//CCCC"));

    assert_eq!(config.get_pins("a.example").map(Vec::len), Some(2));
    assert_eq!(config.get_pins("b.example").map(Vec::len), Some(1));

    config.remove_pins("a.example");
    assert!(config.get_pins("a.example").is_none());
    assert!(config.get_pins("b.example").is_some());
}

#[test]
fn config_presets_differ_in_their_policy() {
    assert!(!CertPinningConfig::default().require_pins);
    assert!(CertPinningConfig::strict().require_pins);
    assert!(CertPinningConfig::report_only().report_only);
}

#[test]
fn config_round_trips_through_a_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("pins.json");

    let mut config = CertPinningConfig::strict();
    config.add_pin(PinnedCert::new("example.com", "sha256//AAAA"));
    config.save_to_file(&path).expect("save");

    let loaded = CertPinningConfig::load_from_file(&path).expect("load");
    assert_eq!(loaded.get_pins("example.com").map(Vec::len), Some(1));
    assert!(loaded.require_pins);
}

#[test]
fn load_from_file_reports_missing_and_invalid_files() {
    let dir = tempfile::tempdir().expect("tempdir");

    assert!(CertPinningConfig::load_from_file(&dir.path().join("absent.json")).is_err());

    let invalid = dir.path().join("invalid.json");
    fs::write(&invalid, b"not json").expect("write");
    assert!(CertPinningConfig::load_from_file(&invalid).is_err());
}

#[test]
fn manager_is_disabled_returns_success_without_a_pin() {
    let manager = CertPinningManager::new(CertPinningConfig {
        enabled: false,
        ..CertPinningConfig::default()
    });

    let result = manager
        .validate_certificate("example.com", &self_signed_der())
        .expect("validate");

    assert!(result.valid);
    assert_eq!(result.matched_pin.as_deref(), Some("disabled"));
}

#[test]
fn manager_without_pins_and_without_requirement_succeeds() {
    let manager = CertPinningManager::default_manager();

    let result = manager
        .validate_certificate("example.com", &self_signed_der())
        .expect("validate");

    assert!(result.valid);
    assert_eq!(result.matched_pin.as_deref(), Some("no-pins-required"));
}

#[test]
fn manager_strict_mode_fails_without_pins() {
    let manager = CertPinningManager::new(CertPinningConfig::strict());

    let result = manager
        .validate_certificate("example.com", &self_signed_der())
        .expect("validate");

    assert!(!result.valid);
    assert_eq!(result.error.as_deref(), Some("No pins configured for hostname"));
    assert!(result.actual_fingerprint.starts_with("sha256//"));
}

#[test]
fn manager_report_only_mode_never_fails() {
    let manager = CertPinningManager::new(CertPinningConfig::report_only());

    let result = manager
        .validate_certificate("example.com", &self_signed_der())
        .expect("validate");

    assert!(result.valid);
}

#[test]
fn manager_report_only_mode_downgrades_a_pin_mismatch() {
    let der = self_signed_der();
    let manager = CertPinningManager::new(CertPinningConfig::report_only());
    manager
        .add_pin(pin_for("example.com", &self_signed_der()))
        .expect("add pin");

    let result = manager
        .validate_certificate("example.com", &der)
        .expect("validate");

    assert!(result.valid);
    assert_eq!(result.matched_pin.as_deref(), Some("report-only"));
}

#[test]
fn manager_matches_a_configured_pin() {
    let der = self_signed_der();
    let manager = CertPinningManager::default_manager();
    manager.add_pin(pin_for("example.com", &der)).expect("add pin");

    let result = manager
        .validate_certificate("example.com", &der)
        .expect("validate");

    assert!(result.valid);
    assert!(!result.backup_match);
}

#[test]
fn manager_fails_when_no_pin_matches() {
    let der = self_signed_der();
    let manager = CertPinningManager::default_manager();
    manager
        .add_pin(pin_for("example.com", &self_signed_der()))
        .expect("add pin");

    let result = manager
        .validate_certificate("example.com", &der)
        .expect("validate");

    assert!(!result.valid);
    assert!(result.error.as_deref().unwrap_or_default().contains("does not match"));
}

#[test]
fn manager_flags_backup_pin_matches() {
    let der = self_signed_der();
    let hash = compute_spki_fingerprint(&der).expect("fingerprint");
    let manager = CertPinningManager::default_manager();
    manager
        .add_pin(PinnedCert::backup(
            "example.com",
            &format!("sha256//{}", BASE64.encode(hash)),
        ))
        .expect("add pin");

    let result = manager
        .validate_certificate("example.com", &der)
        .expect("validate");

    assert!(result.valid);
    assert!(result.backup_match);
}

#[test]
fn manager_caches_successful_validations() {
    let der = self_signed_der();
    let manager = CertPinningManager::default_manager();
    manager.add_pin(pin_for("example.com", &der)).expect("add pin");

    let first = manager
        .validate_certificate("example.com", &der)
        .expect("first");

    // A later pin change must not invalidate an entry inside the TTL window.
    manager
        .add_pin(pin_for("example.com", &self_signed_der()))
        .expect("add pin");

    let cached = manager
        .validate_certificate("example.com", &der)
        .expect("cached");
    assert_eq!(first.matched_pin, cached.matched_pin);

    let fresh = manager
        .validate_certificate("other.example", &der)
        .expect("fresh");
    assert_eq!(fresh.matched_pin.as_deref(), Some("no-pins-required"));
}

#[test]
fn manager_imports_pins_from_a_file() {
    let dir = tempfile::tempdir().expect("tempdir");
    let source = dir.path().join("pins.json");

    let mut config = CertPinningConfig::default();
    config.add_pin(PinnedCert::new("example.com", "sha256//AAAA"));
    config.save_to_file(&source).expect("save");

    let manager = CertPinningManager::new(CertPinningConfig::strict());
    manager.import_pins(&source).expect("import");

    let result = manager
        .validate_certificate("example.com", &self_signed_der())
        .expect("validate");

    assert!(!result.valid);
    assert!(result.error.as_deref().unwrap_or_default().contains("does not match"));
}

#[test]
fn generate_pin_from_file_accepts_pem_and_der() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cert = rcgen::generate_simple_self_signed(vec!["example.com".to_string()])
        .expect("self-signed certificate");

    let pem_path = dir.path().join("cert.pem");
    fs::write(&pem_path, cert.cert.pem()).expect("write pem");
    let der_path = dir.path().join("cert.der");
    fs::write(&der_path, cert.cert.der()).expect("write der");

    let from_pem = CertPinningManager::generate_pin_from_file("example.com", &pem_path)
        .expect("pem pin");
    let from_der = CertPinningManager::generate_pin_from_file("example.com", &der_path)
        .expect("der pin");

    assert_eq!(from_pem.fingerprint, from_der.fingerprint);
    assert!(from_pem.verify(cert.cert.der()).expect("verify"));
}

#[test]
fn generate_pins_from_directory_skips_unsupported_files() {
    let dir = tempfile::tempdir().expect("tempdir");
    let cert = rcgen::generate_simple_self_signed(vec!["example.com".to_string()])
        .expect("self-signed certificate");

    fs::write(dir.path().join("cert.pem"), cert.cert.pem()).expect("write pem");
    fs::write(dir.path().join("notes.txt"), b"ignore me").expect("write txt");

    let pins = CertPinningManager::generate_pins_from_directory("example.com", dir.path())
        .expect("pins");

    assert_eq!(pins.len(), 1);
    assert_eq!(pins[0].hostname, "example.com");
}

#[test]
fn validate_pem_file_reports_a_missing_file() {
    let manager = CertPinningManager::default_manager();
    let err = manager
        .validate_pem_file("example.com", Path::new("/nonexistent/cert.pem"))
        .unwrap_err();

    assert!(err.to_string().contains("Failed to read PEM file"));
}
