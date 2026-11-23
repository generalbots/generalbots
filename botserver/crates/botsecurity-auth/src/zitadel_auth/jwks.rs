//! Zitadel bearer-token verification against the directory's JWKS (#2851).
//!
//! The old `decode_jwt_user` base64-decoded the payload and trusted whatever
//! `sub`, `email` and `roles` it contained. Any caller could therefore mint a
//! token for an admin and — through `rbac_role_resolver` /
//! `resolve_user_role_with_email` — escalate to `Role::Admin`.
//!
//! Every token is now verified as an RS256 JWT whose signature chains to a key
//! published by the directory at `{issuer}/oauth/v2/keys`:
//!
//! - `alg: none` and HMAC algorithms are rejected before any claim is read;
//! - the `kid` header must select a published RSA signing key (refreshed once
//!   on a miss so a key rollover does not reject every token forever);
//! - `jsonwebtoken::decode` checks the signature, `exp` and `iss`, with a small
//!   clock-skew allowance; a token signed by any other key is unusable.
//!
//! Keys are public, cached in-process with a short TTL and never logged; only
//! the `kid` ever appears in a warning.

use std::collections::HashMap;
use std::sync::{LazyLock, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde::Deserialize;
use serde_json::Value;

use super::{AuthError, ZitadelAuthConfig};

/// How long a fetched key set is trusted before it is fetched again.
const CACHE_TTL: Duration = Duration::from_secs(300);
/// Upper bound on one JWKS request.
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);
/// Allowed clock skew between the directory and this host, in seconds.
const CLOCK_SKEW_SECONDS: u64 = 30;

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
    alg: Option<String>,
    #[serde(rename = "use", default)]
    use_: Option<String>,
    #[serde(default)]
    n: String,
    #[serde(default)]
    e: String,
}

struct CachedKeySet {
    keys: HashMap<String, DecodingKey>,
    fetched_at: Instant,
}

static CACHE: LazyLock<Mutex<HashMap<String, CachedKeySet>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

/// A poisoned mutex here only holds a plain key map; recovering it is safe and
/// preferable to rejecting every authenticated request.
fn cache() -> MutexGuard<'static, HashMap<String, CachedKeySet>> {
    CACHE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// The directory's canonical JWKS endpoint, derived from the trusted issuer
/// rather than from the token (which is untrusted until it is verified).
pub(super) fn jwks_uri(config: &ZitadelAuthConfig) -> Result<String, AuthError> {
    let base = config.issuer_url.trim_end_matches('/');
    let parsed = url::Url::parse(base)
        .map_err(|_| AuthError::InternalError("Invalid directory issuer".to_string()))?;
    let host = parsed
        .host_str()
        .ok_or_else(|| AuthError::InternalError("Invalid directory issuer".to_string()))?;
    let loopback = host == "localhost" || host.starts_with("127.") || host == "[::1]";
    if parsed.scheme() != "https" && !(parsed.scheme() == "http" && loopback) {
        return Err(AuthError::InternalError(
            "Insecure directory issuer".to_string(),
        ));
    }
    Ok(format!("{base}/oauth/v2/keys"))
}

/// Verifies `token` against the published key set and returns its claims.
/// This is the only route from a bearer token to a `ZitadelUser`.
pub(super) async fn verify_token(
    http_client: &reqwest::Client,
    config: &ZitadelAuthConfig,
    token: &str,
) -> Result<Value, AuthError> {
    let header = decode_header(token).map_err(|_| AuthError::InvalidToken)?;
    // Rejects `alg:none` and HS256/RS256 confusion: the public key is public,
    // so an HMAC token "signed" with it would be trivially forgeable.
    if header.alg != Algorithm::RS256 {
        return Err(AuthError::InvalidToken);
    }
    let kid = header.kid.ok_or(AuthError::InvalidToken)?;
    let key = decoding_key(http_client, config, &kid).await?;
    verify_with_key(token, &key, config)
}

async fn decoding_key(
    http_client: &reqwest::Client,
    config: &ZitadelAuthConfig,
    kid: &str,
) -> Result<DecodingKey, AuthError> {
    let uri = jwks_uri(config)?;
    if let Some(key) = cached_key(&uri, kid) {
        return Ok(key);
    }
    refresh(http_client, &uri).await?;
    cached_key(&uri, kid).ok_or(AuthError::InvalidToken)
}

fn cached_key(uri: &str, kid: &str) -> Option<DecodingKey> {
    let guard = cache();
    let set = guard.get(uri)?;
    if set.fetched_at.elapsed() > CACHE_TTL {
        return None;
    }
    set.keys.get(kid).cloned()
}

async fn refresh(http_client: &reqwest::Client, uri: &str) -> Result<(), AuthError> {
    let response = http_client
        .get(uri)
        .timeout(FETCH_TIMEOUT)
        .send()
        .await
        .map_err(|e| {
            log::warn!("Zitadel JWKS fetch failed: {e}");
            AuthError::InvalidToken
        })?;
    if !response.status().is_success() {
        log::warn!("Zitadel JWKS fetch returned {}", response.status());
        return Err(AuthError::InvalidToken);
    }
    let body = response.text().await.map_err(|e| {
        log::warn!("Zitadel JWKS body read failed: {e}");
        AuthError::InvalidToken
    })?;
    let keys = keys_from_jwks(&body)?;
    cache().insert(
        uri.to_string(),
        CachedKeySet {
            keys,
            fetched_at: Instant::now(),
        },
    );
    Ok(())
}

/// Parses a JWKS document into `kid` → RSA verification key. One malformed or
/// non-signing entry is skipped rather than taking the whole set down.
pub(super) fn keys_from_jwks(body: &str) -> Result<HashMap<String, DecodingKey>, AuthError> {
    let set: JwkSet = serde_json::from_str(body).map_err(|e| {
        log::warn!("Zitadel JWKS is not valid JSON: {e}");
        AuthError::InvalidToken
    })?;

    let mut keys = HashMap::new();
    for jwk in set.keys {
        if jwk.kid.is_empty() || !jwk.kty.eq_ignore_ascii_case("RSA") {
            continue;
        }
        if let Some(use_) = jwk.use_.as_deref() {
            if !use_.eq_ignore_ascii_case("sig") {
                continue;
            }
        }
        if let Some(alg) = jwk.alg.as_deref() {
            if !alg.eq_ignore_ascii_case("RS256") {
                continue;
            }
        }
        if jwk.n.is_empty() || jwk.e.is_empty() {
            log::warn!("Zitadel JWKS key '{}' has no modulus/exponent", jwk.kid);
            continue;
        }
        match DecodingKey::from_rsa_components(&jwk.n, &jwk.e) {
            Ok(key) => {
                keys.insert(jwk.kid, key);
            }
            Err(e) => log::warn!("Zitadel JWKS key is unusable: {e}"),
        }
    }

    if keys.is_empty() {
        log::warn!("Zitadel JWKS carried no usable RSA signing key");
        return Err(AuthError::InvalidToken);
    }
    Ok(keys)
}

/// Signature + claim validation with an already resolved key. Keeping this
/// separate lets the tests exercise verification without a network mock.
pub(super) fn verify_with_key(
    token: &str,
    key: &DecodingKey,
    config: &ZitadelAuthConfig,
) -> Result<Value, AuthError> {
    let mut validation = Validation::new(Algorithm::RS256);
    validation.leeway = CLOCK_SKEW_SECONDS;
    validation.validate_exp = true;
    validation.set_required_spec_claims(&["exp", "iss", "sub"]);
    validation.set_issuer(&[config.issuer_url.trim_end_matches('/')]);

    match decode::<Value>(token, key, &validation) {
        Ok(data) => Ok(data.claims),
        Err(e) => {
            if e.kind() == &jsonwebtoken::errors::ErrorKind::ExpiredSignature {
                Err(AuthError::ExpiredToken)
            } else {
                log::warn!("Zitadel token verification failed: {e}");
                Err(AuthError::InvalidToken)
            }
        }
    }
}

#[cfg(test)]
pub(super) fn seed_key(uri: &str, kid: &str, key: DecodingKey) {
    cache().insert(
        uri.to_string(),
        CachedKeySet {
            keys: HashMap::from([(kid.to_string(), key)]),
            fetched_at: Instant::now(),
        },
    );
}
