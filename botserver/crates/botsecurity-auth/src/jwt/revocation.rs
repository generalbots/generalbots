//! JWT logout revocation helpers (issue #2631).
//!
//! Cloud-login JWTs are HS256 tokens minted by `botcloud` with a compact
//! `sub/email/exp/...` payload. Before #2631 they carried no `jti`, so the
//! Redis blacklist (keyed by jti) could not address them and logout was a
//! no-op: a stolen token stayed valid for its full seven-day lifetime.
//!
//! Everything here is either pure (unit-testable without Redis) or a thin
//! accessor for the process-wide store installed when the JWT manager is
//! built with a shared Redis/Valkey backend. Sharing the backing store — not
//! the Rust instance — is what makes a revocation visible to every instance.

use std::collections::HashSet;
use std::sync::{Arc, RwLock as StdRwLock};

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use uuid::Uuid;

use crate::blacklist::{BlacklistEntry, BlacklistStore, InMemoryBlacklistStore};
use crate::jwt::jwt_types::{Claims, JwtManager};

/// Upper bound used only when a token's `exp` cannot be interpreted.
const DEFAULT_TTL_DAYS: i64 = 7;

static GLOBAL_BLACKLIST: std::sync::OnceLock<Arc<dyn BlacklistStore>> = std::sync::OnceLock::new();
static FALLBACK_BLACKLIST: std::sync::OnceLock<Arc<dyn BlacklistStore>> = std::sync::OnceLock::new();
/// Synchronous mirror of locally-issued revocations. Some resolvers (e.g. the
/// `/api/auth/me` cloud-JWT fallback) are plain `fn` pointers and cannot await
/// the Redis store; this cache closes that gap on the instance that handled
/// the logout. The async store remains authoritative across instances.
static LOCAL_REVOCATIONS: std::sync::OnceLock<StdRwLock<HashSet<String>>> = std::sync::OnceLock::new();

fn local_revocations() -> &'static StdRwLock<HashSet<String>> {
    LOCAL_REVOCATIONS.get_or_init(|| StdRwLock::new(HashSet::new()))
}

/// Records a revocation for synchronous verification paths. Called by logout
/// after the shared store accepted the entry.
pub fn record_local_revocation(jti: &str) {
    if let Ok(mut revoked) = local_revocations().write() {
        revoked.insert(jti.to_string());
    }
}

/// Synchronous revocation check for paths that cannot await Redis.
pub fn is_locally_revoked(jti: &str) -> bool {
    local_revocations()
        .read()
        .map(|revoked| revoked.contains(jti))
        .unwrap_or(false)
}

/// Installs the process-wide blacklist. The first installation wins so a
/// worker cannot silently swap revocation backends after boot.
pub fn init_global_blacklist(store: Arc<dyn BlacklistStore>) -> bool {
    GLOBAL_BLACKLIST.set(store).is_ok()
}

/// The blacklist shared by every JWT verification path and by logout.
///
/// Falls back to a single lazily-created in-memory store when no shared
/// backend was installed (unit tests, tooling, single-node dev without a
/// cache client). Production installs the Redis-backed store at boot.
pub fn global_blacklist() -> Arc<dyn BlacklistStore> {
    if let Some(store) = GLOBAL_BLACKLIST.get() {
        return Arc::clone(store);
    }
    Arc::clone(FALLBACK_BLACKLIST.get_or_init(|| Arc::new(InMemoryBlacklistStore::new())))
}

/// Generates a fresh `jti` (JWT ID) claim: a UUIDv4, unique per token and
/// free of user data, so it is safe in a Redis key or a log line.
pub fn generate_jti() -> String {
    Uuid::new_v4().to_string()
}

/// Verdict after a caller has verified the signature and `exp` and looked up
/// the token's `jti` in the shared blacklist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlacklistVerdict {
    /// The token has a `jti` that is not blacklisted.
    Allowed,
    /// The token's `jti` is blacklisted — reject immediately.
    Revoked,
    /// A pre-#2631 token with no `jti`. It cannot be revoked individually, so
    /// it stays valid until `exp` (callers log a warning).
    MissingJti,
}

impl BlacklistVerdict {
    /// Only an explicit blacklist hit denies access; legacy jti-less tokens
    /// remain accepted until expiry for format compatibility.
    pub fn is_rejected(self) -> bool {
        matches!(self, Self::Revoked)
    }
}

/// Pure decision logic. Signature and expiry are checked by the caller first;
/// this only interprets the blacklist lookup result.
pub fn blacklist_verdict(jti: Option<&str>, is_blacklisted: bool) -> BlacklistVerdict {
    match jti {
        None => BlacklistVerdict::MissingJti,
        Some(_) if is_blacklisted => BlacklistVerdict::Revoked,
        Some(_) => BlacklistVerdict::Allowed,
    }
}

/// Remaining lifetime of a token in whole seconds, clamped at zero.
pub fn remaining_ttl_secs(exp_epoch: i64, now_epoch: i64) -> i64 {
    exp_epoch.saturating_sub(now_epoch).max(0)
}

/// Builds a revocation entry bounded by the token's original `exp`, so the
/// Redis backend gives the key a TTL equal to the token's remaining lifetime.
/// `now` is passed in to keep this function pure and unit-testable.
pub fn revocation_entry(jti: &str, exp_epoch: i64, now: DateTime<Utc>) -> BlacklistEntry {
    let expires_at = DateTime::<Utc>::from_timestamp(exp_epoch, 0)
        .unwrap_or_else(|| now + Duration::days(DEFAULT_TTL_DAYS));
    BlacklistEntry::new(jti.to_string(), now, expires_at)
}

impl JwtManager {
    /// Signature + type + blacklist check for access tokens. Authentication
    /// providers must use this instead of the blacklist-blind
    /// `validate_access_token`.
    pub async fn validate_access_token_with_blacklist(&self, token: &str) -> Result<Claims> {
        let token_data = self.validate_token_with_blacklist(token).await?;
        if !token_data.claims.is_access_token() {
            return Err(anyhow!("Token is not an access token"));
        }
        Ok(token_data.claims)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_jti_is_a_unique_uuid_v4() {
        let a = generate_jti();
        let b = generate_jti();
        assert_ne!(a, b, "jti must be unique per token");
        let parsed = Uuid::parse_str(&a).expect("jti is a UUID");
        // RFC 4122 version nibble lives in the high bits of byte 6.
        assert_eq!(parsed.as_bytes()[6] >> 4, 4, "jti must be UUIDv4");
    }

    #[test]
    fn verdict_rejects_only_an_explicit_blacklist_hit() {
        assert_eq!(blacklist_verdict(Some("abc"), true), BlacklistVerdict::Revoked);
        assert_eq!(blacklist_verdict(Some("abc"), false), BlacklistVerdict::Allowed);
        assert_eq!(blacklist_verdict(None, false), BlacklistVerdict::MissingJti);
        assert!(blacklist_verdict(Some("abc"), true).is_rejected());
        assert!(!blacklist_verdict(Some("abc"), false).is_rejected());
        // Legacy tokens without a jti stay valid until exp (compatibility).
        assert!(!blacklist_verdict(None, false).is_rejected());
    }

    #[test]
    fn remaining_ttl_is_clamped_at_zero() {
        assert_eq!(remaining_ttl_secs(100, 40), 60);
        assert_eq!(remaining_ttl_secs(40, 100), 0);
        assert_eq!(remaining_ttl_secs(i64::MIN, i64::MAX), 0);
    }

    #[test]
    fn revocation_entry_is_bounded_by_original_exp() {
        let now = Utc::now();
        let exp = now.timestamp() + 600;
        let entry = revocation_entry("jti-1", exp, now);
        assert_eq!(entry.jti, "jti-1");
        assert_eq!(entry.revoked_at, now);
        assert_eq!(entry.expires_at.timestamp(), exp);
        assert_eq!(remaining_ttl_secs(entry.expires_at.timestamp(), now.timestamp()), 600);
    }

    #[test]
    fn local_revocation_mirror_records_and_checks_jti() {
        let jti = generate_jti();
        assert!(!is_locally_revoked(&jti));
        record_local_revocation(&jti);
        assert!(is_locally_revoked(&jti));
    }

    #[tokio::test]
    async fn global_blacklist_is_the_same_store_for_all_callers() {
        let jti = generate_jti();
        let entry = revocation_entry(&jti, Utc::now().timestamp() + 60, Utc::now());
        global_blacklist().insert(entry).await.expect("insert");
        assert!(global_blacklist().contains(&jti).await);
    }
}
