//! Content-addressed storage for sandbox UI resources.
//!
//! The identifier is the sha256 of the bytes. The in-process cache serves the
//! preview immediately; Drive is the durable copy across restarts and nodes.
//! A changed resource is a different hash, so a pinned hash can never silently
//! point at new content.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use sha2::{Digest, Sha256};

use botcore::shared::state::AppState;

use super::csp::UiCsp;
use super::sign::signed_path;

#[derive(Debug, Clone)]
pub struct StoredResource {
    pub html: String,
    pub csp: UiCsp,
}

fn cache() -> &'static Mutex<HashMap<String, StoredResource>> {
    static CACHE: OnceLock<Mutex<HashMap<String, StoredResource>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn lock_cache() -> std::sync::MutexGuard<'static, HashMap<String, StoredResource>> {
    match cache().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

#[must_use]
pub fn resource_hash(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// Stores the HTML and returns the signed path. `ttl_secs` is clamped by the
/// signer to one hour.
pub async fn publish_resource(
    state: &AppState,
    bucket: &str,
    thread_id: &str,
    html: &str,
    csp: UiCsp,
    ttl_secs: i64,
) -> String {
    let bytes = html.as_bytes();
    let hash = resource_hash(bytes);
    lock_cache().insert(
        hash.clone(),
        StoredResource {
            html: html.to_string(),
            csp: csp.clone(),
        },
    );

    if let Some(drive) = state.drive.as_ref() {
        let key = format!("_sandbox/ui/{hash}.html");
        if let Err(error) = drive
            .put_object(bucket, &key, bytes.to_vec(), Some("text/html; charset=utf-8"))
            .await
        {
            log::warn!("sandbox resource store failed: {error}");
        }
        let meta_key = format!("_sandbox/ui/{hash}.json");
        let meta = serde_json::to_vec(&csp).unwrap_or_default();
        if let Err(error) = drive
            .put_object(bucket, &meta_key, meta, Some("application/json"))
            .await
        {
            log::warn!("sandbox metadata store failed: {error}");
        }
    }

    signed_path(thread_id, &hash, ttl_secs)
}

/// Reads a resource by hash: cache first, then Drive.
pub async fn read_resource(state: &AppState, bucket: &str, hash: &str) -> Option<StoredResource> {
    if let Some(resource) = lock_cache().get(hash).cloned() {
        return Some(resource);
    }
    let drive = state.drive.as_ref()?;
    let key = format!("_sandbox/ui/{hash}.html");
    let bytes = drive.get_object(bucket, &key).await.ok()?;
    let html = String::from_utf8(bytes).ok()?;
    let meta_key = format!("_sandbox/ui/{hash}.json");
    let csp = match drive.get_object(bucket, &meta_key).await {
        Ok(bytes) => serde_json::from_slice::<UiCsp>(&bytes).unwrap_or_default(),
        Err(_) => UiCsp::default(),
    };
    let resource = StoredResource {
        html: html.clone(),
        csp: csp.clone(),
    };
    lock_cache().insert(hash.to_string(), resource.clone());
    Some(resource)
}

/// Removes a resource from the cache (Drive retention is handled by the
/// bucket lifecycle). Used when a hash is revoked.
pub fn forget(hash: &str) {
    lock_cache().remove(hash);
}

#[cfg(test)]
mod tests {
    use super::resource_hash;

    #[test]
    fn hash_is_content_addressed() {
        assert_eq!(resource_hash(b"abc"), resource_hash(b"abc"));
        assert_ne!(resource_hash(b"abc"), resource_hash(b"abd"));
        assert_eq!(resource_hash(b"abc").len(), 64);
    }
}
