//! Verified tenant scope for the products API (issue #2128).
//!
//! A product write may only touch the caller's own tenant branch. The scope is
//! derived exclusively from the **verified** credentials carried by the request
//! (`botsecurity_core::scope`: signed JWT claim or session-cache email), never
//! from a request header/body field and never from the crate-level default bot.
//!
//! The previous handlers did
//! `branch_from_jwt_pool(..).unwrap_or_else(|| get_bot_context(..))`, which
//! turned an unauthenticated `POST /api/products/items` into a silent insert in
//! whatever branch the server considered default — another tenant's catalogue.
//! Writes now fail closed with `401` instead.

use super::*;

/// Branch the request is allowed to read/write, or `None` when it carries no
/// verified identity with a resolvable branch.
///
/// Deliberately takes no default-bot resolver: a scope helper that can fall
/// back is exactly how a write lands in the wrong tenant, so the parameter does
/// not exist here.
///
/// The server-minted `branch_id` claim is authoritative and needs no database
/// round-trip, so it is read first; only tokens without that claim (opaque
/// suite sessions) are resolved through the session cache / CRM lookup.
pub(crate) fn verified_branch(headers: &HeaderMap, pool: &DbPool) -> Option<Uuid> {
    crate::scope::branch_from_claim(headers)
        .or_else(|| crate::scope::branch_from_jwt_pool(headers, pool))
}

/// `401` for a write whose caller has no verified tenant branch.
pub(crate) fn unauthorized(locale: &botlib::i18n::Locale) -> ApiError {
    api_error(locale, StatusCode::UNAUTHORIZED, "error-auth-token-missing")
}
