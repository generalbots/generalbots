//! #2697 — origin validation for the desktop main WebView.
//!
//! `GB_DESKTOP_SUITE_URL` and the dev-server fallback are attacker-influenced
//! strings that end up in `WebviewUrl::External` / `Webview::navigate`. The
//! main window holds privileged IPC, so it must never become a browser for an
//! arbitrary origin. This module is the single place that decides which URLs
//! may be loaded there, and installs a Tauri navigation hook as
//! defence-in-depth against in-page redirects (the mobile thin client does the
//! same in `crate::mobile::mod`).

use tauri::Url;

/// Whether the Tauri `devUrl` may be followed. A debug build that embeds the
/// frontend (`custom-protocol`) must serve its own assets: opening port 3000
/// there would point the privileged WebView at whatever unrelated server the
/// developer happens to be running.
pub(super) fn use_dev_server() -> bool {
    cfg!(debug_assertions) && !cfg!(feature = "custom-protocol")
}

/// Accept only the loopback sidecar origin (port from `sidecar_port()`), plus
/// the dev-server origin in debug builds. Reject other schemes, hosts, ports
/// and embedded credentials before they ever reach WebKit.
pub(super) fn is_allowed_suite_url(url: &Url) -> bool {
    if !matches!(url.scheme(), "http" | "https") {
        return false;
    }
    // `user@host` does not change the network origin but disguises it in the
    // address bar and can smuggle credentials into privileged requests.
    if !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    // Keep the host allowlist tight: no public names, no private/LAN ranges,
    // no IPv6 literal, no metadata endpoint. The sidecar binds 127.0.0.1.
    if !matches!(url.host_str(), Some("localhost") | Some("127.0.0.1")) {
        return false;
    }
    let port = match url.port_or_known_default() {
        Some(port) => port,
        None => return false,
    };
    if port == crate::desktop::sidecar::probe::sidecar_port() {
        return true;
    }
    // The dev server is plain http only; https on its port is not a suite.
    use_dev_server() && url.scheme() == "http" && port == 3000
}

/// Parses a configured suite URL and applies the full origin policy. A hostile
/// value is refused rather than silently falling through to another target.
pub(super) fn parse_suite_url(raw: &str) -> Option<Url> {
    let url = Url::parse(raw.trim()).ok()?;
    is_allowed_suite_url(&url).then_some(url)
}

/// The app's own document origin, loaded before the handoff to the sidecar.
/// `WebviewUrl::App` resolves to `tauri://localhost` everywhere except
/// Windows/Android, where the custom protocol is `https://tauri.localhost`.
/// Both must be permitted or the navigation hook would blank the window on
/// startup. `.localhost` is reserved by RFC 6761 and cannot resolve off-host.
fn is_app_document(url: &Url) -> bool {
    match url.scheme() {
        "tauri" => url.host_str() == Some("localhost"),
        "https" => url.host_str() == Some("tauri.localhost"),
        _ => false,
    }
}

/// Predicate installed on the "main" WebView (config-created and recreated).
pub(super) fn is_allowed_main_navigation(url: &Url) -> bool {
    is_app_document(url) || is_allowed_suite_url(url)
}

/// Registers the navigation hook before Tauri creates any WebView. It is
/// scoped to the privileged "main" window: the wizard and splash have their
/// own local origins and are not subject to the suite origin policy.
pub fn navigation_guard<R: tauri::Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("gb-origin-guard")
        .on_navigation(|webview, url| {
            if webview.label() != "main" {
                return true;
            }
            is_allowed_main_navigation(url)
        })
        .build()
}

#[cfg(test)]
mod tests {
    use super::{is_allowed_main_navigation, is_allowed_suite_url, parse_suite_url};

    /// A hostile corpus: external origins, non-http schemes, wrong ports,
    /// credentials, private/metadata hosts and look-alike userinfo.
    const HOSTILE: &[&str] = &[
        "https://evil.example/suite/desktop.html",
        "http://evil.example:8080/suite/desktop.html",
        "file:///etc/passwd",
        "javascript:alert(1)",
        "data:text/html,<script>alert(1)</script>",
        "ftp://127.0.0.1:8080/suite/desktop.html",
        "http://localhost:9999/suite/desktop.html",
        "https://localhost:3000/suite/desktop.html",
        "http://user@127.0.0.1:8080/suite/desktop.html",
        "http://user:pass@localhost:8080/suite/desktop.html",
        "http://127.0.0.1@evil.example/suite/desktop.html",
        "http://169.254.169.254/latest/meta-data/",
        "http://192.0.2.10:8080/suite/desktop.html",
        "http://198.51.100.7:8080/suite/desktop.html",
        "http://[::1]:8080/suite/desktop.html",
        "not a url",
        "",
    ];

    #[test]
    fn hostile_suite_urls_are_refused() {
        for hostile in HOSTILE {
            assert!(
                parse_suite_url(hostile).is_none(),
                "accepted hostile URL: {hostile}"
            );
        }
    }

    #[test]
    fn loopback_sidecar_urls_are_accepted() {
        let port = crate::desktop::sidecar::probe::sidecar_port();
        for host in ["127.0.0.1", "localhost"] {
            let raw = format!("http://{host}:{port}/suite/desktop.html?token=abc");
            let parsed = parse_suite_url(&raw).expect("valid loopback URL");
            assert!(is_allowed_suite_url(&parsed));
            assert!(parse_suite_url(&format!("  {raw}  ")).is_some());
        }
    }

    #[test]
    fn main_navigation_allows_the_app_document_but_no_external_origin() {
        for doc in [
            "tauri://localhost/suite/desktop.html",
            "https://tauri.localhost/suite/desktop.html",
        ] {
            let url = tauri::Url::parse(doc).expect("valid app URL");
            assert!(
                is_allowed_main_navigation(&url),
                "blocked the app document: {doc}"
            );
            assert!(!is_allowed_suite_url(&url));
        }
        let external = tauri::Url::parse("https://evil.example/").expect("valid URL");
        assert!(!is_allowed_main_navigation(&external));
    }
}
