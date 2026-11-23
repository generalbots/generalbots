//! Inline regression tests for the plugin security gates (#2833).

use super::*;
use std::path::Path;

fn sample_manifest(signature: Option<String>) -> PluginManifest {
    PluginManifest {
        id: Uuid::parse_str("11111111-1111-1111-1111-111111111111").unwrap(),
        name: "sample".to_string(),
        version: "1.0.0".to_string(),
        base_url: "https://api.example.com".to_string(),
        description: None,
        permissions: vec![PLUGIN_CALL_PERMISSION.to_string()],
        authentication: AuthType::None,
        functions: Vec::new(),
        bot_id: Some("bot-1".to_string()),
        auth_vault_path: None,
        source_path: Some("plugins/sample.json".to_string()),
        signature,
    }
}

fn policy(key_hex: Option<String>) -> PluginSecurityPolicy {
    PluginSecurityPolicy {
        allowed_hosts: vec!["api.example.com".to_string()],
        signing_key_hex: key_hex,
        work_root: PathBuf::from("/opt/gbo/work"),
    }
}

#[test]
fn signature_is_required_and_detects_tampering() {
    let key = b"test-signing-key";
    let mut manifest = sample_manifest(None);
    let canonical = manifest.canonical_json().unwrap();
    assert!(manifest.verify_signature(key).is_err(), "unsigned must be rejected");

    let signature = sign_manifest_json(&canonical, key).unwrap();
    manifest.signature = Some(signature);
    assert!(manifest.verify_signature(key).is_ok());

    let mut tampered = manifest.clone();
    tampered.base_url = "https://evil.example.net".to_string();
    assert!(tampered.verify_signature(key).is_err(), "tampering must be rejected");

    let mut tampered_permission = manifest.clone();
    tampered_permission.permissions.push("admin".to_string());
    assert!(tampered_permission.verify_signature(key).is_err());

    assert!(manifest.verify_signature(b"other-key").is_err());
}

#[test]
fn detached_verifier_rejects_empty_and_malformed_signatures() {
    let canonical = sample_manifest(None).canonical_json().unwrap();
    assert!(verify_manifest_signature(&canonical, "", b"key").is_err());
    assert!(verify_manifest_signature(&canonical, "not-hex", b"key").is_err());
    assert!(verify_manifest_signature_with_hex_key(&canonical, "", "").is_err());
}

#[test]
fn path_containment_rejects_traversal_and_absolute_paths() {
    let root = Path::new("/opt/gbo/work");
    assert!(is_path_within(root, "plugins/foo.json"));
    assert!(is_path_within(root, "./plugins/foo.json"));
    assert!(!is_path_within(root, "../secret.json"));
    assert!(!is_path_within(root, "plugins/../../secret.json"));
    assert!(!is_path_within(root, "/etc/passwd"));
    assert!(!is_path_within(root, "C:\\secret"));
    assert!(!is_path_within(root, ""));
    assert!(!is_path_within(Path::new("relative/root"), "plugins/foo.json"));
}

#[test]
fn base_url_must_be_https_and_allowlisted() {
    let hosts = policy(None).allowed_hosts;
    assert!(sample_manifest(None).is_host_allowed(&hosts).is_ok());
    let mut bad = sample_manifest(None);
    bad.base_url = "http://api.example.com/v1".to_string();
    assert!(bad.is_host_allowed(&hosts).is_err());
    bad.base_url = "https://evil.example.net/v1".to_string();
    assert!(bad.is_host_allowed(&hosts).is_err());
    bad.base_url = "https://user:pass@api.example.com/v1".to_string();
    assert!(bad.is_host_allowed(&hosts).is_err());
}

#[tokio::test]
async fn register_requires_signature_allowlist_and_contained_path() {
    let key_hex = hex::encode(b"test-signing-key");
    let registry = PluginRegistry::with_policy(policy(Some(key_hex.clone())));
    let mut manifest = sample_manifest(None);

    assert!(registry.register(manifest.clone()).await.is_err(), "unsigned rejected");
    let canonical = manifest.canonical_json().unwrap();
    manifest.signature =
        Some(sign_manifest_json_with_hex_key(&canonical, &key_hex).unwrap());
    assert!(registry.register(manifest.clone()).await.is_ok());

    let mut escaping = manifest.clone();
    escaping.source_path = Some("../outside.json".to_string());
    assert!(registry.register(escaping).await.is_err(), "path escape rejected");

    let mut unlisted = manifest.clone();
    unlisted.base_url = "https://evil.example.net".to_string();
    assert!(registry.register(unlisted).await.is_err(), "unlisted host rejected");
}

#[tokio::test]
async fn authorize_enforces_permission_and_bot_scope() {
    let key_hex = hex::encode(b"test-signing-key");
    let registry = PluginRegistry::with_policy(policy(Some(key_hex.clone())));
    let mut manifest = sample_manifest(None);
    manifest.signature = Some(
        sign_manifest_json_with_hex_key(&manifest.canonical_json().unwrap(), &key_hex).unwrap(),
    );
    registry.register(manifest.clone()).await.unwrap();

    assert!(registry.authorize("sample", "bot-1").await.is_ok());
    assert!(registry.authorize("sample", "other-bot").await.is_err());
    assert!(registry.authorize("missing", "bot-1").await.is_err());

    let mut no_permission = manifest.clone();
    no_permission.permissions.clear();
    no_permission.signature = Some(
        sign_manifest_json_with_hex_key(&no_permission.canonical_json().unwrap(), &key_hex)
            .unwrap(),
    );
    registry.unregister("sample").await;
    registry.register(no_permission).await.unwrap();
    assert!(registry.authorize("sample", "bot-1").await.is_err(), "permission enforced");
}
