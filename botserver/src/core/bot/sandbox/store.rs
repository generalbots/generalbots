//! Content-addressed storage for sandbox UI resources.
//!
//! The identifier is the sha256 of the bytes. The in-process cache serves the
//! preview immediately; Drive is the durable copy across restarts and nodes.
//! A changed resource is a different hash, so a pinned hash can never silently
//! point at new content.
//!
//! #2541 — the in-process cache is an LRU/TTL bounded map
//! (`GB_SANDBOX_CACHE_CAP` = 256, `GB_SANDBOX_CACHE_TTL_MS` = 1 h). Drive is
//! always the durable source, so evicting a preview only costs a re-read. The
//! revocation denylist is deliberately unbounded: dropping a revocation could
//! resurrect a resource whose durable objects were deleted.

use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::core::bot::bounded::{self, BoundedMap};
use botbasic_ai::keywords::code_sandbox::preview_store;

use sha2::{Digest, Sha256};

use botcore::shared::state::AppState;

use super::csp::UiCsp;
use super::sign::signed_path;

#[derive(Debug, Clone)]
pub struct StoredResource {
    pub html: String,
    pub csp: UiCsp,
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

fn boot_id() -> u64 {
    static BOOT: OnceLock<u64> = OnceLock::new();
    *BOOT.get_or_init(now_ms)
}

/// #2541 — the reason code for a cached preview, if any. Never a silent miss:
/// `expired` and `interrupted_by_restart` are reported to callers.
#[must_use]
pub fn preview_status(hash: &str) -> Option<String> {
    preview_store::shared()
        .status(hash, now_ms())
        .map(|status| status.reason_code().to_string())
}

/// #2541 — bounded preview cache. The durable copy in Drive is reloaded on a
/// miss, so LRU/TTL eviction can never lose user data.
fn cache() -> &'static Mutex<BoundedMap<String, StoredResource>> {
    static CACHE: OnceLock<Mutex<BoundedMap<String, StoredResource>>> = OnceLock::new();
    CACHE.get_or_init(|| {
        Mutex::new(BoundedMap::new(
            bounded::cap_from_env("GB_SANDBOX_CACHE_CAP", 256),
            bounded::ttl_from_env("GB_SANDBOX_CACHE_TTL_MS", 3_600_000),
        ))
    })
}

fn lock_cache() -> std::sync::MutexGuard<'static, BoundedMap<String, StoredResource>> {
    match cache().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// #2798 — hashes whose signed URLs have been revoked. Consulted before any
/// cache or Drive lookup so a still-valid token cannot resurrect a resource
/// whose durable objects were deleted.
fn denylist() -> &'static Mutex<HashSet<String>> {
    static DENYLIST: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
    DENYLIST.get_or_init(|| Mutex::new(HashSet::new()))
}

fn lock_denylist() -> std::sync::MutexGuard<'static, HashSet<String>> {
    match denylist().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// #2798 — pure revocation predicate. Free of global state so the
/// publish → revoke → stale-URL decision is unit-testable.
#[must_use]
pub fn is_revoked(denylist: &HashSet<String>, hash: &str) -> bool {
    denylist.contains(hash)
}

/// #2798 — pure counterpart of [`is_revoked`]: add `hash` to the denylist.
pub fn mark_revoked(denylist: &mut HashSet<String>, hash: &str) {
    denylist.insert(hash.to_string());
}

/// #2798 — live query used by both the read and serve paths.
#[must_use]
pub fn is_hash_revoked(hash: &str) -> bool {
    let denylist = lock_denylist();
    is_revoked(&denylist, hash)
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
    title: Option<&str>,
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
    // #2541 — TTL/capacity-bounded preview cache with boot reconciliation.
    preview_store::shared().insert(&hash, html, 3_600_000, now_ms());

    if let Some(drive) = state.drive.as_ref() {
        let key = format!("_sandbox/ui/{hash}.html");
        if let Err(error) = drive
            .put_object(
                bucket,
                &key,
                bytes.to_vec(),
                Some("text/html; charset=utf-8"),
            )
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
        // #1676 — catalog sidecar so the built-in MCP server can list the app
        // by title without parsing the HTML.
        let catalog_key = format!("_sandbox/ui/{hash}.meta.json");
        let catalog = serde_json::json!({
            "title": title.unwrap_or("Generated app"),
            "thread_id": thread_id,
            "created_at": chrono::Utc::now().to_rfc3339(),
        })
        .to_string();
        if let Err(error) = drive
            .put_object(
                bucket,
                &catalog_key,
                catalog.into_bytes(),
                Some("application/json"),
            )
            .await
        {
            log::warn!("sandbox catalog store failed: {error}");
        }
    }

    signed_path(thread_id, &hash, ttl_secs)
}

/// Reads a resource by hash: revocation gate, then cache, then Drive.
pub async fn read_resource(state: &AppState, bucket: &str, hash: &str) -> Option<StoredResource> {
    // #2798 — checked before the cache AND the Drive fallback: a revoked hash
    // must never be re-inserted by a request that still carries a valid token.
    if is_hash_revoked(hash) {
        return None;
    }
    {
        let mut cache = lock_cache();
        if let Some(resource) = cache.get(&hash.to_string()).cloned() {
            return Some(resource);
        }
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
    lock_cache().remove(&hash.to_string());
    preview_store::shared().forget(hash);
}

/// #2798 — revoke a published resource: deny the hash in-process, delete its
/// durable Drive objects and drop the cached copies. A signed URL issued
/// before revocation keeps its valid signature but can no longer be served.
pub async fn revoke_resource(state: &AppState, bucket: &str, hash: &str) {
    // Deny first so a concurrent read cannot slip through between the Drive
    // deletion and `forget`. Release the lock before awaiting the Drive I/O.
    {
        let mut denylist = lock_denylist();
        mark_revoked(&mut denylist, hash);
    }
    if let Some(drive) = state.drive.as_ref() {
        for suffix in ["html", "json", "meta.json"] {
            let key = format!("_sandbox/ui/{hash}.{suffix}");
            if let Err(error) = drive.delete_object(bucket, &key).await {
                log::warn!("sandbox revoke could not delete {key}: {error}");
            }
        }
    }
    forget(hash);
}

/// #2541 — called at boot after the durable stores are available.
pub fn reconcile_previews() {
    preview_store::shared().reconcile_on_boot(boot_id(), now_ms());
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use crate::core::bot::sandbox::sign::{sign_resource, verify_resource};
    use super::{is_revoked, mark_revoked, resource_hash};

    #[test]
    fn hash_is_content_addressed() {
        assert_eq!(resource_hash(b"abc"), resource_hash(b"abc"));
        assert_ne!(resource_hash(b"abc"), resource_hash(b"abd"));
        assert_eq!(resource_hash(b"abc").len(), 64);
    }

    /// #2798 — the pure revoke/check predicate only denies the revoked hash;
    /// unrelated resources stay readable and re-revoking is idempotent.
    #[test]
    fn revoke_predicate_only_denies_the_revoked_hash() {
        let mut denylist = HashSet::new();
        let a = resource_hash(b"resource-a");
        let b = resource_hash(b"resource-b");
        assert!(!is_revoked(&denylist, &a));
        assert!(!is_revoked(&denylist, &b));
        mark_revoked(&mut denylist, &a);
        assert!(is_revoked(&denylist, &a));
        assert!(!is_revoked(&denylist, &b));
        mark_revoked(&mut denylist, &a);
        assert_eq!(denylist.len(), 1);
    }

    /// #2798 — publish → revoke → stale signed URL. The token keeps verifying
    /// (correct signature, not expired) but the denylist now makes the read
    /// path return `None`, which the route maps to 404.
    #[test]
    fn published_then_revoked_url_is_denied() {
        let hash = resource_hash(b"<html>published app</html>");
        let token = sign_resource("thread-1", &hash, 60);
        let mut denylist = HashSet::new();

        // Freshly published: the signed URL verifies and the hash is readable.
        assert_eq!(
            verify_resource(&hash, &token).ok().as_deref(),
            Some("thread-1")
        );
        assert!(!is_revoked(&denylist, &hash));

        // Revoked: the *same* signed URL still verifies, yet the read gate
        // (`read_resource` → `is_hash_revoked`) now denies it → HTTP 404.
        mark_revoked(&mut denylist, &hash);
        assert_eq!(
            verify_resource(&hash, &token).ok().as_deref(),
            Some("thread-1")
        );
        assert!(is_revoked(&denylist, &hash));
    }
}
