//! Bot Framework bearer-token verification.
//!
//! Microsoft signs every inbound activity with an RS256 JWT issued by
//! `https://api.botframework.com`, with the bot's Microsoft App ID as `aud`.
//! The token is only a proof when the signature is checked against the Bot
//! Framework key set **and** the claims are constrained, so all of the
//! following are enforced:
//!
//! - `alg` is RS256 (never `none`, never an HMAC algorithm: the public key is
//!   public, so HS256 with it would be forgeable);
//! - the `kid` header selects a published key (`jwks`), which is refreshed
//!   once when the `kid` is unknown so a Microsoft key rollover does not
//!   reject deliveries forever;
//! - `iss`, `aud`, `exp` and `nbf` are validated, with the clock skew the Bot
//!   Framework documentation allows.

use jsonwebtoken::{decode, decode_header, Algorithm, DecodingKey, Validation};
use serde_json::Value;

use super::{jwks, AuthError};

/// Issuer of every Bot Framework token.
pub const BOT_FRAMEWORK_ISSUER: &str = "https://api.botframework.com";

/// Allowed clock skew between Microsoft and this host, in seconds.
const CLOCK_SKEW_SECONDS: u64 = 300;

/// Verifies `token` for `app_id`, resolving the signing key from the Bot
/// Framework key set.
pub async fn verify(token: &str, app_id: &str) -> Result<(), AuthError> {
    let header = decode_header(token).map_err(|_| AuthError::InvalidToken)?;
    if header.alg != Algorithm::RS256 {
        return Err(AuthError::InvalidToken);
    }
    let kid = header.kid.ok_or(AuthError::InvalidToken)?;
    let key = jwks::decoding_key(&kid).await?;
    verify_with_key(token, app_id, &key)
}

/// Signature and claim validation with an already resolved key.
pub fn verify_with_key(token: &str, app_id: &str, key: &DecodingKey) -> Result<(), AuthError> {
    if app_id.trim().is_empty() {
        return Err(AuthError::Unconfigured);
    }

    let mut validation = Validation::new(Algorithm::RS256);
    validation.leeway = CLOCK_SKEW_SECONDS;
    validation.validate_nbf = true;
    validation.set_required_spec_claims(&["exp", "aud", "iss"]);
    validation.set_audience(&[app_id.trim()]);
    validation.set_issuer(&[BOT_FRAMEWORK_ISSUER]);

    match decode::<Value>(token, key, &validation) {
        Ok(_) => Ok(()),
        Err(e) => {
            log::warn!("Teams activity token rejected: {e}");
            Err(AuthError::InvalidToken)
        }
    }
}
