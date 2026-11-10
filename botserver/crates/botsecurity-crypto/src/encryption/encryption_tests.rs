use super::*;

fn random_key() -> Vec<u8> {
    EncryptionKey::generate(KeyPurpose::DataEncryption).key_data
}

#[test]
fn aes_gcm_rejects_wrong_key_sizes() {
    let err = encrypt_aes_gcm(b"data", &[0u8; 16]).unwrap_err();
    assert!(err.to_string().contains("Invalid key size"));

    let err = decrypt_aes_gcm("AAAA", "AAAA", &[0u8; 8]).unwrap_err();
    assert!(err.to_string().contains("Invalid key size"));
}

#[test]
fn aes_gcm_rejects_a_tampered_ciphertext() {
    let key = random_key();
    let (nonce, ciphertext) = encrypt_aes_gcm(b"top secret", &key).expect("encrypt");

    let mut tampered = BASE64.decode(&ciphertext).expect("decode");
    tampered[0] ^= 0xff;
    let tampered = BASE64.encode(tampered);

    assert!(decrypt_aes_gcm(&nonce, &tampered, &key).is_err());
}

#[test]
fn aes_gcm_rejects_invalid_encodings() {
    let key = random_key();

    assert!(decrypt_aes_gcm("not base64!", "AAAA", &key).is_err());
    assert!(decrypt_aes_gcm(&BASE64.encode([0u8; 8]), "AAAA", &key)
        .unwrap_err()
        .to_string()
        .contains("Invalid nonce size"));
    assert!(decrypt_aes_gcm(&BASE64.encode([0u8; 12]), "not base64!", &key).is_err());
}

#[test]
fn aes_gcm_uses_a_fresh_nonce_per_encryption() {
    let key = random_key();
    let first = encrypt_aes_gcm(b"same", &key).expect("first");
    let second = encrypt_aes_gcm(b"same", &key).expect("second");

    assert_ne!(first.0, second.0);
    assert_ne!(first.1, second.1);
}

#[test]
fn field_encryption_rejects_malformed_payloads() {
    let key = random_key();

    assert!(decrypt_field("1:only-two", &key).is_err());
    assert!(decrypt_field("2:nonce:ciphertext", &key)
        .unwrap_err()
        .to_string()
        .contains("Unsupported encryption version"));
    assert!(decrypt_field("x:nonce:ciphertext", &key).is_err());
}

#[test]
fn compact_format_handles_default_key_id_and_colons_in_ciphertext() {
    let data = EncryptedData {
        algorithm: "AES-256-GCM".into(),
        nonce: "nonce".into(),
        ciphertext: "part1:part2".into(),
        key_id: None,
        version: 3,
    };

    let compact = data.to_string_compact();
    assert!(compact.contains(":default:"));

    let parsed = EncryptedData::from_string_compact(&compact).expect("parse");
    assert_eq!(parsed.version, 3);
    assert_eq!(parsed.key_id, None);
    assert_eq!(parsed.ciphertext, "part1:part2");
}

#[test]
fn compact_format_rejects_short_payloads() {
    assert!(EncryptedData::from_string_compact("1:AES:key:nonce").is_err());
    assert!(EncryptedData::from_string_compact("x:AES:key:nonce:cipher").is_err());
}

#[test]
fn derive_key_from_password_rejects_short_salts() {
    assert!(derive_key_from_password("pw", &[0u8; 8]).is_err());
}

#[test]
fn derive_key_from_password_depends_on_password_and_salt() {
    let salt_a = generate_salt();
    let salt_b = generate_salt();

    let a = derive_key_from_password("password", &salt_a).expect("a");
    let b = derive_key_from_password("password", &salt_b).expect("b");
    let c = derive_key_from_password("other", &salt_a).expect("c");

    assert_eq!(a.len(), KEY_SIZE);
    assert_ne!(a, b);
    assert_ne!(a, c);
    assert_ne!(salt_a, salt_b);
}

#[test]
fn scope_keys_are_isolated_per_domain_and_scope() {
    let master = random_key();
    let branch = Uuid::new_v4();
    let bot = Uuid::new_v4();

    let branch_key = derive_scope_key(&master, "branch", &branch);
    let same = derive_scope_key(&master, "branch", &branch);
    let other_domain = derive_scope_key(&master, "bot", &branch);
    let other_scope = derive_scope_key(&master, "branch", &bot);

    assert_eq!(branch_key, same);
    assert_ne!(branch_key, other_domain);
    assert_ne!(branch_key, other_scope);
    assert_eq!(branch_key.len(), KEY_SIZE);
}

#[test]
fn hash_for_search_is_pepper_dependent() {
    let first = hash_for_search("value", b"pepper-a");
    let second = hash_for_search("value", b"pepper-b");

    assert_eq!(first.len(), 64);
    assert_ne!(first, second);
    assert!(first.chars().all(|c| c.is_ascii_hexdigit()));
}

#[tokio::test]
async fn disabled_manager_refuses_encryption() {
    let manager = EncryptionManager::new(EncryptionConfig {
        enabled: false,
        ..EncryptionConfig::default()
    })
    .expect("manager");
    manager.initialize().await.expect("init");

    assert!(manager.encrypt(b"data").await.is_err());
    assert!(manager.encrypt_with_envelope(b"data").await.is_err());
}

#[tokio::test]
async fn manager_rejects_unknown_key_ids() {
    let manager = EncryptionManager::new(EncryptionConfig::default()).expect("manager");
    manager.initialize().await.expect("init");

    let orphan = EncryptedData {
        algorithm: "AES-256-GCM".into(),
        nonce: BASE64.encode([0u8; 12]),
        ciphertext: BASE64.encode([0u8; 16]),
        key_id: Some("missing".into()),
        version: 1,
    };

    assert!(manager.decrypt(&orphan).await.is_err());

    let without_id = EncryptedData {
        key_id: None,
        ..orphan
    };
    assert!(manager.decrypt(&without_id).await.is_err());
}

#[tokio::test]
async fn rotation_keeps_old_data_decryptable() {
    let manager = EncryptionManager::new(EncryptionConfig::default()).expect("manager");
    manager.initialize().await.expect("init");

    let encrypted = manager.encrypt(b"before rotation").await.expect("encrypt");
    let (new_dek, new_kek) = manager.rotate_keys().await.expect("rotate");

    assert_eq!(manager.primary_dek_id.read().await.clone(), Some(new_dek.clone()));
    assert_eq!(manager.primary_kek_id.read().await.clone(), Some(new_kek.clone()));
    assert_eq!(manager.decrypt(&encrypted).await.expect("decrypt"), b"before rotation");

    let old = encrypted.key_id.clone().expect("key id");
    assert_ne!(old, new_dek);
    assert_eq!(manager.get_key(&old).await.map(|k| k.is_primary), Some(false));
}

#[tokio::test]
async fn expired_keys_are_pruned_but_primary_survives() {
    let manager = EncryptionManager::new(EncryptionConfig::default()).expect("manager");
    manager.initialize().await.expect("init");

    let mut expired = EncryptionKey::generate(KeyPurpose::DataEncryption);
    expired.expires_at = Some(chrono::Utc::now() - chrono::Duration::days(1));
    let expired_id = expired.id.clone();
    manager.add_key(expired).await;

    let removed = manager.remove_expired_keys().await;
    assert_eq!(removed, 1);
    assert!(manager.get_key(&expired_id).await.is_none());

    let primary = manager.primary_dek_id.read().await.clone().expect("primary");
    assert!(manager.get_key(&primary).await.is_some());
}

#[tokio::test]
async fn envelope_encryption_requires_the_flag() {
    let manager = EncryptionManager::new(EncryptionConfig {
        envelope_encryption: false,
        ..EncryptionConfig::default()
    })
    .expect("manager");
    manager.initialize().await.expect("init");

    assert!(manager.encrypt_with_envelope(b"data").await.is_err());

    let envelope = EnvelopeEncryptedData {
        encrypted_dek: EncryptedData {
            algorithm: "AES-256-GCM".into(),
            nonce: BASE64.encode([0u8; 12]),
            ciphertext: BASE64.encode([0u8; 16]),
            key_id: None,
            version: 1,
        },
        encrypted_data: EncryptedData {
            algorithm: "AES-256-GCM".into(),
            nonce: BASE64.encode([0u8; 12]),
            ciphertext: BASE64.encode([0u8; 16]),
            key_id: None,
            version: 1,
        },
        kek_id: "missing".into(),
    };

    assert!(manager.decrypt_with_envelope(&envelope).await.is_err());
}
