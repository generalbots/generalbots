//! Package integrity for marketplace installs (issue #2761).
//!
//! A `.gbskill` bundle is a JSON object `{ "manifest": {...}, "scripts": {...} }`.
//! Before any bundle is written into a bot bucket we verify:
//!   * the blob SHA-256 matches the checksum recorded at publish time;
//!   * the stored manifest carries a valid server-keyed HMAC signature;
//!   * every script in the bundle is an allowlisted, relative `.bas` name from
//!     the signed manifest, so nothing can escape the install root or be
//!     executed as an undeclared file.
//!
//! The signing key is server-held (`MARKETPLACE_PACKAGE_SIGNING_KEY`, injected
//! from Vault). Without it both publishing and installing fail closed: an
//! unverifiable package is never written to a bot's Drive.

use std::sync::OnceLock;

use botsecurity::botsecurity_core::constant_time;
use hmac::{Hmac, Mac};
use sha2::{Digest, Sha256};

const SIGNING_KEY_ENV: &str = "MARKETPLACE_PACKAGE_SIGNING_KEY";
const CHECKSUM_FIELD: &str = "content_sha256";
const SIGNATURE_FIELD: &str = "signature";
const ALLOWED_EXTENSION: &str = ".bas";
const MAX_SCRIPT_NAME: usize = 128;

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerifyError {
    SigningKeyUnavailable,
    InvalidContent,
    ChecksumMismatch,
    SignatureMismatch,
    UnsafeScriptName,
    UndeclaredScript,
    MissingEntry,
}

impl std::fmt::Display for VerifyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::SigningKeyUnavailable => "marketplace signing key is unavailable",
            Self::InvalidContent => "package content is not a valid skill bundle",
            Self::ChecksumMismatch => "package checksum does not match the signed manifest",
            Self::SignatureMismatch => "package manifest signature is invalid",
            Self::UnsafeScriptName => "package contains an unsafe script name",
            Self::UndeclaredScript => "package contains an undeclared script",
            Self::MissingEntry => "package manifest has no declared entry script",
        };
        f.write_str(message)
    }
}

impl std::error::Error for VerifyError {}

fn resolve_signing_key() -> Option<Vec<u8>> {
    match std::env::var(SIGNING_KEY_ENV) {
        Ok(value) if !value.trim().is_empty() => Some(value.into_bytes()),
        Ok(_) => {
            tracing::error!("{SIGNING_KEY_ENV} is empty; marketplace packages cannot be signed");
            None
        }
        Err(_) => {
            tracing::error!("{SIGNING_KEY_ENV} is not set; marketplace packages cannot be signed");
            None
        }
    }
}

/// Process-wide signing key, resolved once from the environment (Vault).
pub fn signing_key() -> Option<&'static [u8]> {
    static KEY: OnceLock<Option<Vec<u8>>> = OnceLock::new();
    KEY.get_or_init(resolve_signing_key).as_deref()
}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

/// Pure predicate: a script name that can safely be materialised below the
/// install root. No separators, no `..`, no leading dot, no control characters
/// and only a single `.bas` file name are accepted; anything else (absolute
/// paths, traversal, symlink-like aliases, non-BASIC executables) is refused.
pub fn safe_script_name(name: &str) -> bool {
    if name.is_empty() || name.len() > MAX_SCRIPT_NAME {
        return false;
    }
    if name.starts_with('.') || name.contains("..") {
        return false;
    }
    if name.contains('/') || name.contains('\\') || name.contains('\0') {
        return false;
    }
    if !name.ends_with(ALLOWED_EXTENSION) {
        return false;
    }
    let stem = &name[..name.len() - ALLOWED_EXTENSION.len()];
    !stem.is_empty() && stem.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

fn str_array(value: Option<&serde_json::Value>) -> Vec<&str> {
    value
        .and_then(|v| v.as_array())
        .map(|items| items.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_else(Vec::new)
}

/// Deterministic bytes covered by the MAC. Sorted lists make the signature
/// independent of JSON array order; binding slug, version, checksum, entry,
/// scripts and permissions prevents relocating a signature to another package.
fn canonical_message(slug: &str, version: &str, checksum: &str, manifest: &serde_json::Value) -> String {
    let mut scripts = str_array(manifest.get("scripts"));
    let mut permissions = str_array(manifest.get("permissions"));
    let mut prompts = str_array(manifest.get("prompts"));
    scripts.sort_unstable();
    permissions.sort_unstable();
    prompts.sort_unstable();
    let entry = manifest.get("entry").and_then(|v| v.as_str()).unwrap_or("");
    format!(
        "gbskill-v1\nslug:{slug}\nversion:{version}\nsha256:{checksum}\nentry:{entry}\n\
         scripts:{}\npermissions:{}\nprompts:{}\n",
        scripts.join(","),
        permissions.join(","),
        prompts.join(",")
    )
}

fn compute_signature(key: &[u8], message: &str) -> Option<String> {
    let mut mac = HmacSha256::new_from_slice(key).ok()?;
    mac.update(message.as_bytes());
    Some(hex::encode(mac.finalize().into_bytes()))
}

/// Adds `content_sha256` and `signature` to the manifest stored for a version.
/// Called at publish/seed time; fails closed when no server key is configured.
pub fn sign_manifest(
    slug: &str,
    version: &str,
    content: &[u8],
    manifest: &serde_json::Value,
) -> Result<serde_json::Value, VerifyError> {
    let key = signing_key().ok_or(VerifyError::SigningKeyUnavailable)?;
    sign_manifest_with_key(key, slug, version, content, manifest)
}

fn sign_manifest_with_key(
    key: &[u8],
    slug: &str,
    version: &str,
    content: &[u8],
    manifest: &serde_json::Value,
) -> Result<serde_json::Value, VerifyError> {
    let checksum = sha256_hex(content);
    let signature = compute_signature(key, &canonical_message(slug, version, &checksum, manifest))
        .ok_or(VerifyError::SigningKeyUnavailable)?;
    let mut signed = manifest.clone();
    let fields = signed.as_object_mut().ok_or(VerifyError::InvalidContent)?;
    fields.insert(CHECKSUM_FIELD.to_string(), serde_json::Value::String(checksum));
    fields.insert(SIGNATURE_FIELD.to_string(), serde_json::Value::String(signature));
    Ok(signed)
}

/// Verifies a downloaded blob against the signed manifest stored in the
/// database. Pure apart from reading the process key; the keyed variants below
/// are used by the inline tests.
pub fn verify_bundle(
    slug: &str,
    version: &str,
    content: &[u8],
    manifest: &serde_json::Value,
) -> Result<(), VerifyError> {
    let key = signing_key().ok_or(VerifyError::SigningKeyUnavailable)?;
    verify_bundle_with_key(key, slug, version, content, manifest)
}

fn verify_bundle_with_key(
    key: &[u8],
    slug: &str,
    version: &str,
    content: &[u8],
    manifest: &serde_json::Value,
) -> Result<(), VerifyError> {
    let checksum = manifest.get(CHECKSUM_FIELD).and_then(|v| v.as_str()).ok_or(VerifyError::ChecksumMismatch)?;
    if !constant_time::eq_str(checksum, &sha256_hex(content)) {
        return Err(VerifyError::ChecksumMismatch);
    }
    let signature = manifest.get(SIGNATURE_FIELD).and_then(|v| v.as_str()).ok_or(VerifyError::SignatureMismatch)?;
    let expected = compute_signature(key, &canonical_message(slug, version, checksum, manifest))
        .ok_or(VerifyError::SigningKeyUnavailable)?;
    if !constant_time::eq_str(signature, &expected) {
        return Err(VerifyError::SignatureMismatch);
    }
    validate_bundle_scripts(content, manifest)
}

/// Structural check of the bundle: script names confined to the install root,
/// every declared script present and allowlisted, and the entry must be one of
/// the declared scripts. This is the "no arbitrary script" allowlist.
pub fn validate_bundle_scripts(content: &[u8], manifest: &serde_json::Value) -> Result<(), VerifyError> {
    let bundle: serde_json::Value = serde_json::from_slice(content).map_err(|_| VerifyError::InvalidContent)?;
    let scripts = bundle.get("scripts").and_then(|v| v.as_object()).ok_or(VerifyError::InvalidContent)?;
    let declared = str_array(manifest.get("scripts"));
    let entry = manifest.get("entry").and_then(|v| v.as_str()).ok_or(VerifyError::MissingEntry)?;
    if !declared.iter().any(|name| *name == entry) {
        return Err(VerifyError::MissingEntry);
    }
    for (name, body) in scripts {
        if !safe_script_name(name) {
            return Err(VerifyError::UnsafeScriptName);
        }
        if !declared.iter().any(|allowed| *allowed == name.as_str()) {
            return Err(VerifyError::UndeclaredScript);
        }
        if !body.is_string() {
            return Err(VerifyError::InvalidContent);
        }
    }
    for name in declared {
        if !safe_script_name(name) {
            return Err(VerifyError::UnsafeScriptName);
        }
        if !scripts.contains_key(name) {
            return Err(VerifyError::InvalidContent);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const KEY: &[u8] = b"marketplace-package-test-key-0123456789";
    const OTHER_KEY: &[u8] = b"a-key-the-server-never-used-9876543210";

    fn manifest(scripts: &[&str]) -> serde_json::Value {
        serde_json::json!({
            "entry": scripts.first().copied().unwrap_or(""),
            "scripts": scripts,
            "permissions": ["table.read", "file.write"],
            "prompts": [],
        })
    }

    fn bundle(scripts: &[(&str, &str)]) -> Vec<u8> {
        let map: serde_json::Map<String, serde_json::Value> = scripts
            .iter()
            .map(|(name, body)| (name.to_string(), serde_json::Value::String(body.to_string())))
            .collect();
        serde_json::json!({ "manifest": {}, "scripts": map }).to_string().into_bytes()
    }

    #[test]
    fn safe_script_names_are_confined_to_the_install_root() {
        for good in ["parse_expense.bas", "Webhook2.bas", "a-b_c.bas"] {
            assert!(safe_script_name(good), "{good} should be allowed");
        }
        for bad in [
            "",
            "../evil.bas",
            "sub/evil.bas",
            "C:\\evil.bas",
            ".hidden.bas",
            "foo..bas",
            "no_extension",
            "payload.txt",
            "link -> /etc/passwd.bas",
        ] {
            assert!(!safe_script_name(bad), "{bad} must be refused");
        }
        assert!(!safe_script_name(&"x".repeat(129)));
    }

    #[test]
    fn signature_round_trip_detects_checksum_and_manifest_tampering() {
        let declared = ["parse.bas", "report.bas"];
        let raw = manifest(&declared);
        let content = bundle(&[("parse.bas", "TALK \"hi\""), ("report.bas", "TALK \"bye\"")]);
        let signed = sign_manifest_with_key(KEY, "demo", "1.0.0", &content, &raw).unwrap();
        assert!(verify_bundle_with_key(KEY, "demo", "1.0.0", &content, &signed).is_ok());

        let mut flipped = content.clone();
        if let Some(last) = flipped.last_mut() {
            *last = last.wrapping_add(1);
        }
        assert_eq!(
            verify_bundle_with_key(KEY, "demo", "1.0.0", &flipped, &signed),
            Err(VerifyError::ChecksumMismatch)
        );

        let mut relabeled = signed.clone();
        relabeled["entry"] = serde_json::json!("other.bas");
        assert_eq!(
            verify_bundle_with_key(KEY, "demo", "1.0.0", &content, &relabeled),
            Err(VerifyError::SignatureMismatch)
        );
        assert_eq!(
            verify_bundle_with_key(OTHER_KEY, "demo", "1.0.0", &content, &signed),
            Err(VerifyError::SignatureMismatch)
        );
    }

    #[test]
    fn unsigned_and_malformed_packages_are_rejected() {
        let raw = manifest(&["parse.bas"]);
        let content = bundle(&[("parse.bas", "TALK \"hi\"")]);
        assert_eq!(
            verify_bundle_with_key(KEY, "demo", "1.0.0", &content, &raw),
            Err(VerifyError::ChecksumMismatch)
        );
        assert_eq!(
            validate_bundle_scripts(b"not json", &raw),
            Err(VerifyError::InvalidContent)
        );
    }

    #[test]
    fn traversal_and_undeclared_scripts_are_rejected_even_when_signed() {
        let traversal = manifest(&["../evil.bas"]);
        let traversal_bundle = bundle(&[("../evil.bas", "TALK \"evil\"")]);
        let signed = sign_manifest_with_key(KEY, "demo", "1.0.0", &traversal_bundle, &traversal).unwrap();
        assert_eq!(
            verify_bundle_with_key(KEY, "demo", "1.0.0", &traversal_bundle, &signed),
            Err(VerifyError::UnsafeScriptName)
        );

        let declared = manifest(&["parse.bas"]);
        let extra = bundle(&[("parse.bas", "TALK \"hi\""), ("stealth.bas", "TALK \"stealth\"")]);
        let signed = sign_manifest_with_key(KEY, "demo", "1.0.0", &extra, &declared).unwrap();
        assert_eq!(
            verify_bundle_with_key(KEY, "demo", "1.0.0", &extra, &signed),
            Err(VerifyError::UndeclaredScript)
        );

        let no_entry = serde_json::json!({ "scripts": ["parse.bas"] });
        assert_eq!(
            validate_bundle_scripts(&bundle(&[("parse.bas", "TALK \"hi\"")]), &no_entry),
            Err(VerifyError::MissingEntry)
        );
    }
}
