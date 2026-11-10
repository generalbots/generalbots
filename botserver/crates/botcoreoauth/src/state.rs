//! Signed, single-use OAuth `state` tokens (#2130).
//!
//! The `state` parameter round-trips through the user agent, so nothing in it
//! may be trusted unless we can prove this server minted it. The token is
//!
//! ```text
//! v1.<base64url(payload-json)>.<base64url(HMAC-SHA256(v1.payload-json))>
//! ```
//!
//! keyed with a server secret (`OAUTH_STATE_SECRET`, else the Vault entry
//! `gbo/oauth-state` / `gbo/jwt`, else a per-process ephemeral key). The MAC is
//! verified with a constant-time compare **before** any field is read, so a
//! forged or tampered token is indistinguishable from garbage.
//!
//! The payload carries a random `nonce` that is also set in a short-lived
//! `HttpOnly` cookie (`crate::cookies::nonce_cookie`) and registered in
//! `crate::nonce_store`. The callback therefore proves three things: the state
//! was issued by us (MAC), it was issued to *this* browser (cookie), and it has
//! not been used before (single-use registry).

use std::fmt;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use hmac::{Hmac, Mac};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::OAuthProvider;

/// How long a state token stays valid, in seconds.
pub const STATE_TTL_SECS: i64 = 600;
/// Tolerated clock skew when a token looks like it comes from the future.
const FUTURE_SKEW_SECS: i64 = 60;
/// Token version prefix; a future format change gets `v2`.
const TOKEN_VERSION: &str = "v1";
/// Environment variable holding the state signing secret.
const SECRET_ENV: &str = "OAUTH_STATE_SECRET";
/// Vault paths consulted (in order) when the env var is absent.
const SECRET_VAULT_PATHS: [&str; 2] = [
    botcoresecrets::paths::SecretPaths::OAUTH_STATE,
    botcoresecrets::paths::SecretPaths::JWT,
];
/// Field name read from the Vault secret.
const SECRET_VAULT_FIELD: &str = "secret";
/// Bound on the Vault lookup so a slow/unsealed Vault cannot stall a login.
const SECRET_LOOKUP_TIMEOUT: Duration = Duration::from_secs(5);

/// Why a state token was refused. Every variant aborts the flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateError {
    /// Not a `v1.payload.mac` triple, or the payload is not our JSON.
    Malformed,
    /// The MAC does not match: the token was forged or tampered with.
    BadSignature,
    /// The token is older than [`STATE_TTL_SECS`] (or minted in the future).
    Expired,
}

impl fmt::Display for StateError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed => write!(f, "malformed OAuth state"),
            Self::BadSignature => write!(f, "OAuth state signature mismatch"),
            Self::Expired => write!(f, "OAuth state expired"),
        }
    }
}

impl std::error::Error for StateError {}

/// The pending OAuth flow, as carried by the signed `state` parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthState {
    pub token: String,
    pub provider: OAuthProvider,
    pub redirect_after: Option<String>,
    pub created_at: i64,
    /// Random, single-use value bound to the browser cookie.
    pub nonce: String,
    /// Owning user id when the flow is an account *link* (Settings) rather
    /// than a fresh login. Only trusted because the whole payload is signed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_id: Option<String>,
    /// PKCE code verifier (S256). Generated server-side, carried inside the
    /// signed state token, and presented during the code exchange. Never
    /// logged.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pkce_verifier: Option<String>,
}

impl OAuthState {
    pub fn new(provider: OAuthProvider, redirect_after: Option<String>) -> Self {
        Self {
            token: uuid::Uuid::new_v4().to_string(),
            provider,
            redirect_after,
            created_at: now_secs(),
            nonce: generate_nonce(),
            user_id: None,
            pkce_verifier: None,
        }
    }

    /// Marks the flow as an account link for `user_id` (Settings flow).
    pub fn for_user(mut self, user_id: &str) -> Self {
        self.user_id = Some(user_id.to_string());
        self
    }

    /// Attaches a freshly generated PKCE S256 code verifier.
    pub fn with_pkce(mut self) -> Self {
        self.pkce_verifier = Some(crate::generate_pkce_verifier());
        self
    }

    pub fn is_expired(&self) -> bool {
        self.is_expired_at(now_secs())
    }

    /// Expiry check against an explicit clock, so tests are deterministic.
    pub fn is_expired_at(&self, now: i64) -> bool {
        now.saturating_sub(self.created_at) > STATE_TTL_SECS
            || self.created_at.saturating_sub(now) > FUTURE_SKEW_SECS
    }
}

/// Signs and verifies [`OAuthState`] tokens.
#[derive(Clone)]
pub struct StateSigner {
    key: Arc<Vec<u8>>,
}

impl fmt::Debug for StateSigner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Never print the key material, only that one is present.
        f.debug_struct("StateSigner")
            .field("key_len", &self.key.len())
            .finish()
    }
}

impl StateSigner {
    /// Builds a signer from raw key material. Empty keys are refused so a
    /// misconfiguration cannot degrade into an unkeyed MAC.
    pub fn from_secret(secret: &[u8]) -> Result<Self, StateError> {
        if secret.is_empty() {
            return Err(StateError::Malformed);
        }
        Ok(Self {
            key: Arc::new(secret.to_vec()),
        })
    }

    fn mac(&self, signing_input: &[u8]) -> Vec<u8> {
        // `new_from_slice` on HMAC accepts any key length, so the only error
        // case is unreachable; fall back to an empty tag that never verifies
        // instead of panicking.
        match Hmac::<Sha256>::new_from_slice(&self.key) {
            Ok(mut mac) => {
                mac.update(signing_input);
                mac.finalize().into_bytes().to_vec()
            }
            Err(_) => Vec::new(),
        }
    }

    /// Serialises and signs a state token.
    pub fn sign(&self, state: &OAuthState) -> Result<String, StateError> {
        let payload = serde_json::to_vec(state).map_err(|_| StateError::Malformed)?;
        let payload_b64 = URL_SAFE_NO_PAD.encode(&payload);
        let signing_input = format!("{TOKEN_VERSION}.{payload_b64}");
        let mac = self.mac(signing_input.as_bytes());
        if mac.is_empty() {
            return Err(StateError::Malformed);
        }
        Ok(format!(
            "{signing_input}.{}",
            URL_SAFE_NO_PAD.encode(mac)
        ))
    }

    /// Verifies the MAC (constant time) and the lifetime, then returns the
    /// payload. No field is read before the MAC matches.
    pub fn verify(&self, token: &str) -> Result<OAuthState, StateError> {
        self.verify_at(token, now_secs())
    }

    /// [`Self::verify`] against an explicit clock.
    pub fn verify_at(&self, token: &str, now: i64) -> Result<OAuthState, StateError> {
        let mut parts = token.split('.');
        let (Some(version), Some(payload_b64), Some(mac_b64), None) =
            (parts.next(), parts.next(), parts.next(), parts.next())
        else {
            return Err(StateError::Malformed);
        };
        if version != TOKEN_VERSION || payload_b64.is_empty() || mac_b64.is_empty() {
            return Err(StateError::Malformed);
        }

        let provided_mac = URL_SAFE_NO_PAD
            .decode(mac_b64)
            .map_err(|_| StateError::Malformed)?;
        let signing_input = format!("{version}.{payload_b64}");
        let expected_mac = self.mac(signing_input.as_bytes());
        if !botsecurity_core::constant_time::eq(&expected_mac, &provided_mac) {
            return Err(StateError::BadSignature);
        }

        let payload = URL_SAFE_NO_PAD
            .decode(payload_b64)
            .map_err(|_| StateError::Malformed)?;
        let state: OAuthState =
            serde_json::from_slice(&payload).map_err(|_| StateError::Malformed)?;
        if state.nonce.is_empty() {
            return Err(StateError::Malformed);
        }
        if state.is_expired_at(now) {
            return Err(StateError::Expired);
        }
        Ok(state)
    }
}

/// Process-wide signer, resolved once from the environment/Vault.
///
/// The key must be stable across restarts for a flow that is already at the
/// provider to survive a deploy; `OAUTH_STATE_SECRET` (or the Vault entry) is
/// therefore the expected configuration. Without either, a random key is
/// generated for the lifetime of the process and pending flows simply expire —
/// never an unsigned fallback.
pub fn server_signer() -> Arc<StateSigner> {
    static SIGNER: OnceLock<Arc<StateSigner>> = OnceLock::new();
    SIGNER
        .get_or_init(|| Arc::new(resolve_server_signer()))
        .clone()
}

fn resolve_server_signer() -> StateSigner {
    if let Some(secret) = env_secret() {
        match StateSigner::from_secret(secret.as_bytes()) {
            Ok(signer) => return signer,
            Err(_) => log::error!("OAuth state secret in {SECRET_ENV} is unusable — ignoring it"),
        }
    }
    if let Some(secret) = vault_secret() {
        match StateSigner::from_secret(secret.as_bytes()) {
            Ok(signer) => return signer,
            Err(_) => log::error!("OAuth state secret from Vault is unusable — ignoring it"),
        }
    }

    let mut random = [0u8; 32];
    rand::rng().fill_bytes(&mut random);
    log::warn!(
        "No OAuth state secret configured ({SECRET_ENV} or Vault {SECRET_VAULT_PATHS:?}): \
         signing OAuth state with an ephemeral key — pending flows are dropped on restart"
    );
    match StateSigner::from_secret(&random) {
        Ok(signer) => signer,
        Err(_) => {
            // Unreachable: `random` is 32 bytes. Degrade to a fixed, non-empty
            // key rather than panic; tokens still verify within the process.
            log::error!("OAuth state signer could not be built from random bytes");
            StateSigner {
                key: Arc::new(vec![0u8; 32]),
            }
        }
    }
}

fn env_secret() -> Option<String> {
    std::env::var(SECRET_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Reads the signing secret from Vault. Blocking and bounded: it runs once per
/// process (the result is cached by [`server_signer`]).
fn vault_secret() -> Option<String> {
    let manager = botcoresecrets::SecretsManager::get_clone().ok()?;
    if !manager.is_enabled() {
        return None;
    }
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let result = match rt {
            Ok(rt) => rt.block_on(async move {
                for path in SECRET_VAULT_PATHS {
                    if let Ok(secrets) = manager.get_secret(path).await {
                        if let Some(secret) = secrets
                            .get(SECRET_VAULT_FIELD)
                            .map(|value| value.trim().to_string())
                            .filter(|value| !value.is_empty())
                        {
                            return Some(secret);
                        }
                    }
                }
                None
            }),
            Err(_) => None,
        };
        let _ = tx.send(result);
    });
    rx.recv_timeout(SECRET_LOOKUP_TIMEOUT).ok().flatten()
}

/// Current Unix time in seconds.
pub fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

/// 32 random bytes, base64url-encoded: the browser-binding nonce.
pub fn generate_nonce() -> String {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    URL_SAFE_NO_PAD.encode(bytes)
}
