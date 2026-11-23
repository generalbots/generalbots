use super::*;
use ring::signature::{Ed25519KeyPair, KeyPair};

fn keypair() -> Ed25519KeyPair {
    let rng = ring::rand::SystemRandom::new();
    let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).expect("test key generation");
    Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).expect("test key parsing")
}

fn sign_digest(key: &Ed25519KeyPair, content: &[u8]) -> String {
    let digest = hex_decode(&SignatureService::hash_document(content).0).expect("test digest");
    BASE64.encode(key.sign(&digest).as_ref())
}

fn document(
    signer_id: Uuid,
    content: &[u8],
    signature_value: &str,
    claimed: ValidationStatus,
) -> SignedDocument {
    let (sha256, sha512) = SignatureService::hash_document(content);
    SignedDocument {
        id: Uuid::new_v4(),
        signature_request_id: Uuid::new_v4(),
        document_id: Uuid::new_v4(),
        signer_id,
        document_hash_sha256: sha256,
        document_hash_sha512: sha512,
        signed_at: Utc::now(),
        algorithm: SignatureAlgorithm::Ed25519,
        signature_value: signature_value.into(),
        certificate_chain: Vec::new(),
        timestamp_token: None,
        ocsp_response: None,
        validation_status: claimed,
    }
}

fn service_with(key: &Ed25519KeyPair, signer_id: Uuid) -> SignatureService {
    let mut service = SignatureService::new();
    service
        .register_key(signer_id, key.public_key().as_ref())
        .expect("test key registration");
    service
}

#[test]
fn valid_signature_verifies_even_when_status_is_claimed() {
    let signer = Uuid::new_v4();
    let key = keypair();
    let service = service_with(&key, signer);
    let signature = sign_digest(&key, b"contract v1");
    // The caller claims Valid; the verdict must come from the crypto check.
    let doc = document(signer, b"contract v1", &signature, ValidationStatus::Valid);
    assert_eq!(service.validate(&doc), ValidationStatus::Valid);
}

#[test]
fn forged_signature_is_rejected_despite_claimed_valid_status() {
    let signer = Uuid::new_v4();
    let key = keypair();
    let service = service_with(&key, signer);
    let forged = BASE64.encode(vec![0u8; ED25519_SIGNATURE_LEN]);
    let doc = document(signer, b"contract v1", &forged, ValidationStatus::Valid);
    assert_eq!(service.validate(&doc), ValidationStatus::Invalid);
}

#[test]
fn signature_from_another_key_is_rejected() {
    let signer = Uuid::new_v4();
    let trusted = keypair();
    let attacker = keypair();
    let service = service_with(&trusted, signer);
    let signature = sign_digest(&attacker, b"contract v1");
    let doc = document(signer, b"contract v1", &signature, ValidationStatus::Valid);
    assert_eq!(service.validate(&doc), ValidationStatus::Invalid);
}

#[test]
fn tampered_document_is_rejected() {
    let signer = Uuid::new_v4();
    let key = keypair();
    let service = service_with(&key, signer);
    let signature = sign_digest(&key, b"contract v1");
    // Signature is over v1; the document hash now describes v2.
    let doc = document(signer, b"contract v2", &signature, ValidationStatus::Valid);
    assert_eq!(service.validate(&doc), ValidationStatus::Invalid);
}

#[test]
fn inconsistent_hash_pair_is_rejected() {
    let signer = Uuid::new_v4();
    let key = keypair();
    let service = service_with(&key, signer);
    let signature = sign_digest(&key, b"contract v1");
    let mut doc = document(signer, b"contract v1", &signature, ValidationStatus::Valid);
    doc.document_hash_sha512 = "00".repeat(64);
    assert_eq!(service.validate(&doc), ValidationStatus::Invalid);
}

#[test]
fn missing_authoritative_key_fails_closed() {
    let signer = Uuid::new_v4();
    let key = keypair();
    let service = SignatureService::new(); // no key registered
    let signature = sign_digest(&key, b"contract v1");
    let doc = document(signer, b"contract v1", &signature, ValidationStatus::Valid);
    assert_eq!(service.validate(&doc), ValidationStatus::Unknown);
    assert_ne!(service.validate(&doc), ValidationStatus::Valid);
}

#[test]
fn fabricated_placeholder_document_is_never_valid() {
    let service = SignatureService::new();
    let doc = SignedDocument {
        id: Uuid::new_v4(),
        signature_request_id: Uuid::nil(),
        document_id: Uuid::nil(),
        signer_id: Uuid::nil(),
        document_hash_sha256: String::new(),
        document_hash_sha512: String::new(),
        signed_at: Utc::now(),
        algorithm: SignatureAlgorithm::Ed25519,
        signature_value: "placeholder".into(),
        certificate_chain: vec!["cert1".into()],
        timestamp_token: None,
        ocsp_response: None,
        validation_status: ValidationStatus::Valid,
    };
    // With no authoritative key provisioned the verdict is `Unknown`; with a
    // key it is `Invalid`. Either way it must never be `Valid`.
    assert_ne!(service.validate(&doc), ValidationStatus::Valid);
}

#[test]
fn completion_does_not_manufacture_a_valid_verdict() {
    let signer = Uuid::new_v4();
    let key = keypair();
    let mut service = SignatureService::new();
    let request = service.create_request(
        Uuid::nil(),
        Uuid::new_v4(),
        signer,
        b"contract v1",
        None,
        None,
    );
    let forged = BASE64.encode(vec![0u8; ED25519_SIGNATURE_LEN]);
    let signed = service
        .complete_signature(request.clone(), forged.clone(), Vec::new())
        .expect("completion");
    assert_ne!(signed.validation_status, ValidationStatus::Valid);

    service
        .register_key(signer, key.public_key().as_ref())
        .expect("test key registration");
    let signature = sign_digest(&key, b"contract v1");
    let signed = service
        .complete_signature(request, signature, Vec::new())
        .expect("completion");
    assert_eq!(signed.validation_status, ValidationStatus::Valid);
}
