//! Microsoft Graph client for OneDrive.
//!
//! Sync uses the `delta` endpoint, not `children`: the first call walks the
//! tree (`@odata.nextLink` until exhausted) and hands back an
//! `@odata.deltaLink`; every later call replays just the changes since that
//! link. That is what makes this affordable — a `files.list` poll is O(corpus)
//! and burns the org's Graph quota.
//!
//! Parsing is separated from the HTTP calls so it can be unit-tested without a
//! network.

mod auth;
mod delta;

pub use auth::{
    authorize_url, authorize_url_with_pkce, exchange_code, exchange_code_with_pkce, refresh_token,
    refresh_token_optional_secret, TokenSet,
};
pub(crate) use delta::parse_ts;
pub use delta::{classify_status, download, fetch_page, parse_delta};
pub(crate) use delta::{percent_encode, GRAPH_BASE};

#[cfg(test)]
mod tests;
