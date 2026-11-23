//! Signature and path-containment primitives for plugin manifests (#2833).
//! Split out of `plugin.rs` so the registry types stay reviewable.

use crate::plugin::PluginError;
use hmac::{Hmac, Mac};
use std::path::{Component, Path};

type HmacSha256 = Hmac<sha2::Sha256>;

/// Produces the signature for canonical manifest JSON. Used by the plugin
/// signing tool and inline tests, never by runtime verification.
pub fn sign_manifest_json(canonical_json: &str, key: &[u8]) -> Result<String, PluginError> {
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|_| PluginError::AuthError("invalid signing key".to_string()))?;
    mac.update(canonical_json.as_bytes());
    Ok(hex::encode(mac.finalize().into_bytes()))
}

/// Detached HMAC-SHA256 verification in constant time.
pub fn verify_manifest_signature(
    canonical_json: &str,
    signature_hex: &str,
    key: &[u8],
) -> Result<(), PluginError> {
    let expected = hex::decode(signature_hex)
        .map_err(|_| PluginError::AuthError("plugin signature is not hex".to_string()))?;
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|_| PluginError::AuthError("invalid signing key".to_string()))?;
    mac.update(canonical_json.as_bytes());
    mac.verify_slice(&expected)
        .map_err(|_| PluginError::AuthError("plugin signature does not match".to_string()))
}

/// Convenience for callers holding the key as hex (env/Vault).
pub fn verify_manifest_signature_with_hex_key(
    canonical_json: &str,
    signature_hex: &str,
    key_hex: &str,
) -> Result<(), PluginError> {
    let key = decode_signing_key(key_hex)?;
    verify_manifest_signature(canonical_json, signature_hex, &key)
}

/// Convenience for signing with a hex key (tooling/tests).
pub fn sign_manifest_json_with_hex_key(
    canonical_json: &str,
    key_hex: &str,
) -> Result<String, PluginError> {
    let key = decode_signing_key(key_hex)?;
    sign_manifest_json(canonical_json, &key)
}

/// Decodes the hex signing key, failing closed on an unset or malformed value.
pub fn decode_signing_key(key_hex: &str) -> Result<Vec<u8>, PluginError> {
    if key_hex.trim().is_empty() {
        return Err(PluginError::AuthError("plugin signing key is not configured".to_string()));
    }
    hex::decode(key_hex)
        .map_err(|_| PluginError::AuthError("plugin signing key is not valid hex".to_string()))
}

/// True when the relative `candidate` resolves inside `root` without any `..`
/// segments, absolute paths, drive prefixes or Windows separators (#2833).
pub fn is_path_within(root: &Path, candidate: &str) -> bool {
    let candidate = candidate.trim().replace('\\', "/");
    if candidate.is_empty() || candidate.starts_with('/') || candidate.contains(':') {
        return false;
    }
    let mut depth = 0usize;
    for component in Path::new(&candidate).components() {
        match component {
            Component::Normal(_) => depth += 1,
            Component::CurDir => {}
            Component::ParentDir => {
                if depth == 0 {
                    return false;
                }
                depth -= 1;
            }
            _ => return false,
        }
    }
    root.is_absolute() && depth > 0
}
