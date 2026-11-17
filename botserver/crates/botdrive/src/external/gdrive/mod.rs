//! Google Drive client (Drive API v3).
//!
//! Same two-phase shape as OneDrive, different vocabulary: the first sync
//! walks `files.list`, then `changes.getStartPageToken` captures a cursor and
//! every later pass is a `changes.list` replay. The page token is the only
//! state kept between runs.
//!
//! `files.list` is used only for the initial traversal. Polling it on a timer
//! is what gets a Google account rate-limited (`userRateLimitExceeded`) and
//! eventually blocked, so the delta path is the one that runs afterwards.

mod api;
mod auth;
mod parse;

pub use api::{download, fetch_change_page, fetch_list_page, fetch_page, start_page_token};
pub use auth::{
    authorize_url, authorize_url_with_pkce, exchange_code, exchange_code_with_pkce, refresh_token,
    refresh_token_optional_secret, TokenSet,
};
pub use parse::{classify_status, parse_page};
pub(super) use parse::{percent_encode, DRIVE_API};

#[cfg(test)]
#[path = "../gdrive_tests.rs"]
mod tests;
