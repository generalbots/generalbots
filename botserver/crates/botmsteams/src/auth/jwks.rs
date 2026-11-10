//! Bot Framework signing keys, fetched from the OpenID metadata document and
//! cached in-process.
//!
//! Microsoft publishes the keys at
//! `https://login.botframework.com/v1/.well-known/openidconfiguration` →
//! `jwks_uri`. The set rotates without notice, so an unknown `kid` triggers one
//! refresh instead of a permanent rejection, and the cache expires on its own
//! so a stale key cannot outlive its rotation. Nothing from the set is a
//! secret: the keys are public and only their `kid` ever reaches the log.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use jsonwebtoken::DecodingKey;
use serde::Deserialize;

use super::AuthError;

/// OpenID metadata for the Bot Framework channel.
const OPENID_CONFIG_URL: &str = "https://login.botframework.com/v1/.well-known/openidconfiguration";

/// How long a fetched key set is trusted before it is fetched again.
const CACHE_TTL: Duration = Duration::from_secs(24 * 60 * 60);

/// Upper bound on one metadata/key-set request.
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Deserialize)]
struct OpenIdConfiguration {
    jwks_uri: String,
}

#[derive(Deserialize)]
struct JwkSet {
    #[serde(default)]
    keys: Vec<Jwk>,
}

#[derive(Deserialize)]
struct Jwk {
    #[serde(default)]
    kid: String,
    #[serde(default)]
    kty: String,
    #[serde(default)]
    n: String,
    #[serde(default)]
    e: String,
}

struct CachedKeySet {
    keys: HashMap<String, DecodingKey>,
    fetched_at: Instant,
}

static CACHE: LazyLock<Mutex<Option<CachedKeySet>>> = LazyLock::new(|| Mutex::new(None));

/// Poisoning is not a reason to reject every activity: the guarded value is a
/// plain key map and stays usable.
fn cache() -> MutexGuard<'static, Option<CachedKeySet>> {
    CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// Resolves the key that signed a token with `kid`, refreshing the set once
/// when it is unknown or the cache has expired.
pub async fn decoding_key(kid: &str) -> Result<DecodingKey, AuthError> {
    if let Some(key) = cached_key(kid) {
        return Ok(key);
    }
    refresh().await?;
    cached_key(kid).ok_or(AuthError::InvalidToken)
}

fn cached_key(kid: &str) -> Option<DecodingKey> {
    let guard = cache();
    let cached = guard.as_ref()?;
    if cached.fetched_at.elapsed() > CACHE_TTL {
        return None;
    }
    cached.keys.get(kid).cloned()
}

fn store(keys: HashMap<String, DecodingKey>) {
    *cache() = Some(CachedKeySet {
        keys,
        fetched_at: Instant::now(),
    });
}

async fn refresh() -> Result<(), AuthError> {
    let client = reqwest::Client::builder()
        .timeout(FETCH_TIMEOUT)
        .build()
        .map_err(|e| {
            log::warn!("Teams key set: HTTP client could not be built: {e}");
            AuthError::InvalidToken
        })?;

    let metadata: OpenIdConfiguration = client
        .get(OPENID_CONFIG_URL)
        .send()
        .await
        .map_err(fetch_failed)?
        .error_for_status()
        .map_err(fetch_failed)?
        .json()
        .await
        .map_err(fetch_failed)?;

    if !metadata.jwks_uri.starts_with("https://") {
        log::warn!("Teams key set: OpenID metadata returned a non-https jwks_uri");
        return Err(AuthError::InvalidToken);
    }

    let body = client
        .get(&metadata.jwks_uri)
        .send()
        .await
        .map_err(fetch_failed)?
        .error_for_status()
        .map_err(fetch_failed)?
        .text()
        .await
        .map_err(fetch_failed)?;

    store(keys_from_jwks(&body)?);
    Ok(())
}

fn fetch_failed(error: reqwest::Error) -> AuthError {
    log::warn!("Teams key set could not be fetched: {error}");
    AuthError::InvalidToken
}

/// Parses a JWKS document into `kid` → RSA verification key. A key that cannot
/// be turned into a decoder is skipped: one malformed entry must not take the
/// whole set (and therefore every delivery) down.
pub(crate) fn keys_from_jwks(body: &str) -> Result<HashMap<String, DecodingKey>, AuthError> {
    let set: JwkSet = serde_json::from_str(body).map_err(|e| {
        log::warn!("Teams key set is not valid JSON: {e}");
        AuthError::InvalidToken
    })?;

    let mut keys = HashMap::new();
    for jwk in set.keys {
        if jwk.kid.is_empty() || !jwk.kty.eq_ignore_ascii_case("RSA") {
            continue;
        }
        if jwk.n.is_empty() || jwk.e.is_empty() {
            log::warn!("Teams key '{}' has an empty modulus or exponent", jwk.kid);
            continue;
        }
        match DecodingKey::from_rsa_components(&jwk.n, &jwk.e) {
            Ok(key) => {
                keys.insert(jwk.kid, key);
            }
            Err(e) => log::warn!("Teams key '{}' is unusable: {e}", jwk.kid),
        }
    }

    if keys.is_empty() {
        log::warn!("Teams key set carried no usable RSA key");
        return Err(AuthError::InvalidToken);
    }
    Ok(keys)
}
