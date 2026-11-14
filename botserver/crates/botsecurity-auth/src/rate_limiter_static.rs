//! Quota for the suite's own static files.
//!
//! When the server also serves the suite UI (the embedded UI of the desktop
//! edition, or an on-disk `ui/` folder), opening the desktop shell requests a
//! few hundred scripts and stylesheets in a single burst. Counted against the
//! API quota (100 requests per second, burst 150, shared by the whole process)
//! that burst is rejected: the browser receives `429` with a JSON body in
//! place of `chat-websocket.js` or `chat-send.js`, and chat reports a failed
//! connection. Static files therefore draw from their own, larger quota.
//! `/api/*` and `/ws` never match, so the API and authentication limits are
//! unchanged.

use axum::http::Method;
use governor::Quota;
use std::num::NonZeroU32;

const STATIC_RPS: u32 = 1000;
const STATIC_BURST: u32 = 2000;

const STATIC_EXTENSIONS: &[&str] = &[
    "js", "mjs", "css", "map", "html", "htm", "svg", "png", "jpg", "jpeg", "gif", "webp", "ico",
    "woff", "woff2", "ttf", "otf", "ftl", "webmanifest", "wasm", "mp3", "ogg", "wav",
];

pub(crate) fn static_asset_quota() -> Quota {
    let one = NonZeroU32::MIN;
    Quota::per_second(NonZeroU32::new(STATIC_RPS).unwrap_or(one))
        .allow_burst(NonZeroU32::new(STATIC_BURST).unwrap_or(one))
}

/// True for a `GET`/`HEAD` of a file with a static extension outside `/api/`
/// and `/ws`.
pub(crate) fn is_static_asset_request(method: &Method, path: &str) -> bool {
    if method != Method::GET && method != Method::HEAD {
        return false;
    }
    if path.starts_with("/api/") || path == "/ws" || path.starts_with("/ws/") {
        return false;
    }
    let file_name = path.rsplit('/').next().unwrap_or("");
    match file_name.rsplit_once('.') {
        Some((stem, extension)) if !stem.is_empty() => {
            let extension = extension.to_ascii_lowercase();
            STATIC_EXTENSIONS.contains(&extension.as_str())
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn suite_scripts_and_styles_are_static() {
        assert!(is_static_asset_request(&Method::GET, "/suite/chat/chat-websocket.js"));
        assert!(is_static_asset_request(&Method::GET, "/suite/chat/chat.css"));
        assert!(is_static_asset_request(&Method::HEAD, "/suite/partials/chat.html"));
        assert!(is_static_asset_request(&Method::GET, "/suite/assets/LOGO.SVG"));
    }

    #[test]
    fn api_and_websocket_paths_keep_the_api_quota() {
        assert!(!is_static_asset_request(&Method::GET, "/api/files/report.html"));
        assert!(!is_static_asset_request(&Method::GET, "/api/auth/login"));
        assert!(!is_static_asset_request(&Method::GET, "/ws"));
        assert!(!is_static_asset_request(&Method::GET, "/ws/default"));
    }

    #[test]
    fn writes_and_extensionless_paths_keep_the_api_quota() {
        assert!(!is_static_asset_request(&Method::POST, "/suite/chat/chat.js"));
        assert!(!is_static_asset_request(&Method::GET, "/health"));
        assert!(!is_static_asset_request(&Method::GET, "/suite/.js"));
        assert!(!is_static_asset_request(&Method::GET, "/suite/data.json"));
    }

    #[test]
    fn the_static_quota_admits_one_cold_page_load() {
        let limiter = governor::RateLimiter::direct(static_asset_quota());
        assert!((0..400).all(|_| limiter.check().is_ok()));
    }
}
