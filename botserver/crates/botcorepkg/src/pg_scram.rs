//! PostgreSQL SCRAM-SHA-256 password verifiers.
//!
//! The local database password is set as a verifier (`ALTER ROLE … PASSWORD
//! 'SCRAM-SHA-256$…'`), so the plain password never appears on a command line
//! or in the server log.

use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use hmac::{Hmac, Mac};
use rand::Rng;
use sha2::{Digest, Sha256};

type HmacSha256 = Hmac<Sha256>;

pub const ITERATIONS: u32 = 4096;

fn hmac(key: &[u8], data: &[u8]) -> [u8; 32] {
    let mut mac = match HmacSha256::new_from_slice(key) {
        Ok(mac) => mac,
        Err(_) => return [0; 32],
    };
    mac.update(data);
    mac.finalize().into_bytes().into()
}

/// PBKDF2-HMAC-SHA256 with a 32-byte output (a single block).
pub fn pbkdf2_sha256(password: &[u8], salt: &[u8], iterations: u32) -> [u8; 32] {
    let mut first = salt.to_vec();
    first.extend_from_slice(&1u32.to_be_bytes());
    let mut block = hmac(password, &first);
    let mut result = block;
    for _ in 1..iterations {
        block = hmac(password, &block);
        for (out, byte) in result.iter_mut().zip(block.iter()) {
            *out ^= byte;
        }
    }
    result
}

/// `(StoredKey, ServerKey)` for a password, salt and iteration count.
pub fn keys(password: &str, salt: &[u8], iterations: u32) -> ([u8; 32], [u8; 32]) {
    let salted = pbkdf2_sha256(password.as_bytes(), salt, iterations);
    let client_key = hmac(&salted, b"Client Key");
    let stored_key: [u8; 32] = Sha256::digest(client_key).into();
    (stored_key, hmac(&salted, b"Server Key"))
}

/// The verifier PostgreSQL stores in `pg_authid.rolpassword`.
pub fn verifier_with_salt(password: &str, salt: &[u8], iterations: u32) -> String {
    let (stored, server) = keys(password, salt, iterations);
    format!(
        "SCRAM-SHA-256${iterations}:{}${}:{}",
        STANDARD.encode(salt),
        STANDARD.encode(stored),
        STANDARD.encode(server)
    )
}

/// A verifier with a fresh random salt. `None` for passwords outside printable
/// ASCII, where the server would apply SASLprep normalisation first.
pub fn verifier(password: &str) -> Option<String> {
    if password.is_empty() || !password.bytes().all(|b| (0x21..=0x7e).contains(&b)) {
        return None;
    }
    let mut salt = [0u8; 16];
    rand::rng().fill(&mut salt);
    Some(verifier_with_salt(password, &salt, ITERATIONS))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02x}")).collect()
    }

    #[test]
    fn pbkdf2_matches_the_published_vectors() {
        assert_eq!(
            hex(&pbkdf2_sha256(b"password", b"salt", 1)),
            "120fb6cffcf8b32c43e7225256c4f837a86548c92ccc35480805987cb70be17b"
        );
        assert_eq!(
            hex(&pbkdf2_sha256(b"password", b"salt", 4096)),
            "c5e478d59288c841aa530db6845c4c8d962893a001ce4e11a4963873aa98134a"
        );
    }

    #[test]
    fn server_key_reproduces_the_rfc_7677_exchange() {
        let salt = STANDARD.decode("W22ZaJ0SNY7soEsUEjb6gQ==").unwrap();
        let (_, server_key) = keys("pencil", &salt, 4096);
        let auth_message = "n=user,r=rOprNGfwEbeRWgbNEkqO,r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0,s=W22ZaJ0SNY7soEsUEjb6gQ==,i=4096,c=biws,r=rOprNGfwEbeRWgbNEkqO%hvYDpWUa2RaTCAfuxFIlj)hNlF$k0";
        let signature = hmac(&server_key, auth_message.as_bytes());
        assert_eq!(STANDARD.encode(signature), "6rriTRBi23WpRR/wtup+mMhUZUn/dB5nLTJRsjl95G4=");
    }

    #[test]
    fn verifier_has_the_postgres_layout_and_a_fresh_salt() {
        let first = verifier("Abc123xyz").unwrap();
        let second = verifier("Abc123xyz").unwrap();
        assert!(first.starts_with("SCRAM-SHA-256$4096:"));
        assert_eq!(first.matches('$').count(), 2);
        assert_ne!(first, second);
        assert!(!first.contains("Abc123xyz"));
    }

    #[test]
    fn unusual_passwords_are_refused() {
        assert!(verifier("").is_none());
        assert!(verifier("with space").is_none());
        assert!(verifier("ação").is_none());
    }
}
