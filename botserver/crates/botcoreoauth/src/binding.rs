//! Browser binding and single-use enforcement for a verified OAuth state
//! (#2130).
//!
//! A valid MAC only proves this server issued the state. It does not prove the
//! callback arrives in the same browser that started the flow (login CSRF) nor
//! that the state is fresh (replay). Both checks live here so the login
//! callback and the Settings account-link callback enforce exactly the same
//! rules.

use axum::http::HeaderMap;

use crate::cookies;
use crate::nonce_store;
use crate::state::now_secs;

/// Why a callback was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingError {
    /// The request carries no `gb_oauth_nonce` cookie at all.
    MissingCookie,
    /// The cookie value is not the nonce of this state.
    CookieMismatch,
    /// The nonce was already consumed (replay) or was never issued.
    Replay,
}

impl std::fmt::Display for BindingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingCookie => write!(f, "no browser-binding nonce cookie"),
            Self::CookieMismatch => write!(f, "nonce cookie does not match the state"),
            Self::Replay => write!(f, "state nonce already used or unknown"),
        }
    }
}

impl std::error::Error for BindingError {}

/// Verifies that `nonce` was issued to the browser behind `headers` and marks
/// it used.
///
/// Order matters: the cookie is compared (constant time) before the nonce is
/// consumed, so a request without the cookie cannot burn a pending flow.
pub fn consume_bound_nonce(headers: &HeaderMap, nonce: &str) -> Result<(), BindingError> {
    consume_bound_user_nonce(headers, nonce).map(|_| ())
}

/// [`consume_bound_nonce`] for account linking: also returns the user id the
/// flow was started for, so the caller can compare it with the signed state
/// instead of trusting the state payload alone.
pub fn consume_bound_user_nonce(
    headers: &HeaderMap,
    nonce: &str,
) -> Result<Option<String>, BindingError> {
    let provided =
        cookies::cookie_value(headers, cookies::NONCE_COOKIE).ok_or(BindingError::MissingCookie)?;
    if !botsecurity_core::constant_time::eq_str(&provided, nonce) {
        return Err(BindingError::CookieMismatch);
    }
    match nonce_store::consume(nonce, now_secs()) {
        Some(pending) => Ok(pending.user_id().map(|id| id.to_string())),
        None => Err(BindingError::Replay),
    }
}
