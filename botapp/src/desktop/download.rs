//! Native Save dialog for suite downloads (#1834).
//!
//! The suite runs inside a WebView, where the browser-only
//! "blob + `<a download>`" pattern produces no visible result (WebKitGTK has
//! no download handler wired on the window). Chat attachments and Drive both
//! call this command instead: it asks for a destination with the native dialog
//! and writes the bytes there.
//!
//! #2690 — the URL is supplied by the webview, so this command is the SSRF
//! boundary. Only the app's own local sidecar/dev origin, its release/update
//! hosts and first-party bot hosts may be fetched. Remote hostnames are also
//! resolved and checked before connecting, so a trusted name cannot point at
//! a private address. Every refusal fails closed.

use std::net::IpAddr;
use std::path::PathBuf;

use super::drive::path_scope;
use super::{fs_scope, i18n};
use tauri_plugin_dialog::DialogExt;

/// Upper bound for one download. Office documents and conversation archives
/// are far below this; the cap only stops a runaway endpoint from exhausting
/// memory before the dialog is confirmed.
const MAX_DOWNLOAD_BYTES: usize = 512 * 1024 * 1024;

/// Hosts that publish the app's own releases/updates (also used by `update.rs`).
const RELEASE_HOSTS: &[&str] = &["github.com", "objects.githubusercontent.com"];

/// First-party product hosts from AGENTS.md (same set as the media allowlist).
const PRODUCT_HOSTS: &[&str] = &[
    "pragmatismo.com.br",
    "cloud.pragmatismo.com.br",
    "chat.pragmatismo.com.br",
    "login.pragmatismo.com.br",
    "login.generalbots.org",
    "generalbots.org",
];

/// Bot workspaces live at `https://{bot}.generalbots.org/…`. One label only,
/// so `a.b.generalbots.org` and a sibling-suffix host never match.
const BOT_SUBDOMAIN_SUFFIX: &str = ".generalbots.org";

/// Operator-controlled extra hosts: comma-separated exact domain names.
const DOWNLOAD_HOSTS_ENV: &str = "GB_DOWNLOAD_HOSTS";

/// The only loopback host names accepted: the sidecar and the dev server.
const LOCAL_HOSTS: &[&str] = &["localhost", "127.0.0.1"];

/// Normalise a URL host: IPv6 hosts arrive bracketed from `url::Url`.
fn bare_host(host: &str) -> &str {
    host.strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(host)
}

/// A syntactically valid DNS name, never an IP literal or a wildcard.
fn domain_host(host: &str) -> bool {
    !host.is_empty()
        && !host.starts_with('.')
        && !host.ends_with('.')
        && !host.contains("..")
        && host
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'.')
        && host.parse::<IpAddr>().is_err()
}

fn configured_hosts(raw: Option<&str>) -> Vec<String> {
    raw.unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|host| domain_host(*host))
        .map(|host| host.to_ascii_lowercase())
        .collect()
}

/// Exact, case-insensitive matching against the built-in and operator lists.
fn trusted_remote_host(host: &str, extra: &[String]) -> bool {
    let lower = host.to_ascii_lowercase();
    if RELEASE_HOSTS
        .iter()
        .chain(PRODUCT_HOSTS.iter())
        .any(|allowed| lower == *allowed)
        || extra.iter().any(|allowed| lower == *allowed)
    {
        return true;
    }
    match lower.strip_suffix(BOT_SUBDOMAIN_SUFFIX) {
        Some(label) => {
            !label.is_empty()
                && !label.contains('.')
                && label
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
        }
        None => false,
    }
}

/// Pure URL decision so hostile URLs are testable without a server or network.
/// `extra` is the parsed operator list; `sidecar_port` keeps this deterministic.
fn url_allowed(url: &reqwest::Url, extra: &[String], sidecar_port: u16) -> bool {
    if !matches!(url.scheme(), "http" | "https") {
        return false;
    }
    if !url.username().is_empty() || url.password().is_some() {
        return false;
    }
    let Some(host) = url.host_str() else {
        return false;
    };
    let host = bare_host(host);
    let port = url.port_or_known_default();
    if LOCAL_HOSTS.contains(&host) {
        // The app's own origin is the sole loopback exception, on fixed ports.
        return port
            .is_some_and(|port| port == sidecar_port || matches!(port, 3000 | 4000 | 5000 | 8080));
    }
    // Literal IPs are never trusted: link-local (169.254.169.254), private and
    // IPv4-mapped IPv6 targets are all SSRF payloads.
    if host.parse::<IpAddr>().is_ok() {
        return false;
    }
    url.scheme() == "https" && port == Some(443) && trusted_remote_host(host, extra)
}

fn is_allowed_download_url(url: &reqwest::Url) -> bool {
    let raw = std::env::var(DOWNLOAD_HOSTS_ENV).ok();
    let extra = configured_hosts(raw.as_deref());
    url_allowed(url, &extra, super::sidecar_port())
}

/// Non-globally-routable IPv4 ranges not covered by the stable
/// `is_private`/`is_link_local`/etc. predicates (`is_global` is unstable on
/// this toolchain): CGNAT (100.64/10), benchmarking (198.18/15), and the
/// IETF protocol assignments (192.0.0/24).
fn is_reserved_v4(v4: &std::net::Ipv4Addr) -> bool {
    let o = v4.octets();
    (o[0] == 100 && (o[1] & 0xc0) == 64)   // 100.64.0.0/10 (CGNAT)
        || (o[0] == 198 && o[1] == 18)     // 198.18.0.0/15 (benchmarking)
        || (o[0] == 192 && o[1] == 0 && o[2] == 0) // 192.0.0.0/24
}

/// RFC 4193 unique-local addresses (fc00::/7).
fn is_unique_local_v6(v6: &std::net::Ipv6Addr) -> bool {
    (v6.segments()[0] & 0xfe00) == 0xfc00
}

/// Reject every address that is not globally routable. IPv4-mapped IPv6 is
/// reduced to its IPv4 form first so `::ffff:127.0.0.1` cannot slip through.
fn ip_is_public(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            !v4.is_loopback()
                && !v4.is_private()
                && !v4.is_link_local()
                && !v4.is_unspecified()
                && !v4.is_multicast()
                && !is_reserved_v4(&v4)
        }
        IpAddr::V6(v6) => {
            // `::1` is IPv4-compatible and `to_ipv4()` maps it to 0.0.0.1;
            // reject loopback before attempting the conversion.
            if v6.is_loopback() {
                return false;
            }
            if let Some(v4) = v6.to_ipv4() {
                return ip_is_public(IpAddr::V4(v4));
            }
            !v6.is_loopback()
                && !v6.is_unspecified()
                && !v6.is_multicast()
                // Unique-local (fc00::/7) and link-local (fe80::/10) are
                // covered below; other reserved ranges are non-routable but
                // `is_global` is unstable, so reject the well-known ones.
                && !v6.is_unicast_link_local()
                && !is_unique_local_v6(&v6)
        }
    }
}

/// Resolve a trusted hostname and refuse it when any answer is non-public.
/// This closes the DNS-rebinding gap left by the static hostname allowlist.
async fn host_resolves_public(host: &str, port: u16) -> Result<bool, String> {
    let mut addresses = tokio::net::lookup_host((host, port))
        .await
        .map_err(|e| i18n::t_args("download-failed", &[("error", &e.to_string())]))?;
    let mut saw_public = false;
    while let Some(address) = addresses.next() {
        if !ip_is_public(address.ip()) {
            log::warn!("download IPC: refusing {host} — resolves to a non-public address");
            return Ok(false);
        }
        saw_public = true;
    }
    Ok(saw_public)
}

/// Keep only the base name the Save dialog should suggest. The user still
/// chooses the final path; path separators and control characters never reach
/// the native dialog.
fn safe_file_name(raw: &str) -> String {
    let base = raw.rsplit(['/', '\\']).next().unwrap_or(raw).trim();
    let clean: String = base
        .chars()
        .filter(|c| {
            c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | ' ' | '(' | ')' | '[' | ']')
        })
        .collect();
    if clean.trim().is_empty() {
        "download".to_string()
    } else {
        clean
    }
}

/// Save one download to a user-chosen path.
///
/// Supply either `url` (the suite fetches it, keeping the bearer token out of
/// the native layer) or `bytes` (a Blob the caller already fetched from a POST
/// endpoint such as Drive's export). Returns the saved path, or `None` when the
/// user cancelled the dialog.
///
/// # Errors
/// Returns a message when the URL is not permitted, the download fails, or the
/// file cannot be written.
#[tauri::command]
pub async fn save_download(
    app: tauri::AppHandle,
    file_name: String,
    url: Option<String>,
    bytes: Option<Vec<u8>>,
) -> Result<Option<String>, String> {
    // #2690: validate before any dialog or socket. A hostile URL is rejected
    // even if the user never reaches the confirmation step.
    let parsed = match url {
        Some(raw) => {
            let parsed = reqwest::Url::parse(raw.trim())
                .map_err(|e| i18n::t_args("download-failed", &[("error", &e.to_string())]))?;
            if !is_allowed_download_url(&parsed) {
                return Err(i18n::t("download-url-not-allowed"));
            }
            Some(parsed)
        }
        None => None,
    };

    let name = safe_file_name(&file_name);
    // The native picker is the explicit consent gate (#2690): no request is
    // made until the user selects a destination, and cancelling sends nothing.
    let Some(target) = app
        .dialog()
        .file()
        .set_title(i18n::t("dialog-save-file"))
        .set_file_name(name.clone())
        .blocking_save_file()
    else {
        return Ok(None);
    };
    let path: PathBuf = target
        .into_path()
        .map_err(|e| i18n::t_args("download-invalid-destination", &[("error", &e.to_string())]))?;
    // #2497 — the native picker is the human confirmation, but its result is
    // still caller-adjacent data: canonicalise and scope-check before writing
    // so a symlink or a forged dialog result cannot drop bytes outside the
    // folders the shell owns.
    let scoped = path_scope::resolve_in_scopes(&path.to_string_lossy(), &fs_scope::scopes())?;

    let payload = match (parsed, bytes) {
        (Some(parsed), None) => {
            let host = match parsed.host_str() {
                Some(host) => host,
                None => return Err(i18n::t("download-url-not-allowed")),
            };
            let host = bare_host(host);
            let port = match parsed.port_or_known_default() {
                Some(port) => port,
                None => return Err(i18n::t("download-url-not-allowed")),
            };
            if !LOCAL_HOSTS.contains(&host) && !host_resolves_public(host, port).await? {
                return Err(i18n::t("download-url-not-allowed"));
            }
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                // A redirect could move the request from a trusted host to an
                // internal one after the checks — never follow one (#2690).
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|e| i18n::t_args("download-failed", &[("error", &e.to_string())]))?;
            let mut response = client
                .get(parsed)
                .send()
                .await
                .map_err(|e| i18n::t_args("download-failed", &[("error", &e.to_string())]))?;
            let status = response.status();
            if !status.is_success() {
                return Err(i18n::t_args(
                    "download-failed",
                    &[("error", &format!("HTTP {status}"))],
                ));
            }
            if response
                .content_length()
                .is_some_and(|length| length > MAX_DOWNLOAD_BYTES as u64)
            {
                return Err(i18n::t("download-too-large"));
            }
            // `content_length()` is optional on chunked responses; enforce the
            // cap while streaming so an unbounded body cannot allocate memory.
            let mut payload = Vec::new();
            while let Some(chunk) = response
                .chunk()
                .await
                .map_err(|e| i18n::t_args("download-failed", &[("error", &e.to_string())]))?
            {
                if payload.len().saturating_add(chunk.len()) > MAX_DOWNLOAD_BYTES {
                    return Err(i18n::t("download-too-large"));
                }
                payload.extend_from_slice(&chunk);
            }
            payload
        }
        (None, Some(bytes)) => {
            if bytes.len() > MAX_DOWNLOAD_BYTES {
                return Err(i18n::t("download-too-large"));
            }
            bytes
        }
        _ => return Err(i18n::t("download-provide-source")),
    };

    tokio::fs::write(&scoped, payload).await.map_err(|e| {
        log::warn!("download IPC: write failed for {}: {e}", scoped.display());
        // Only the sanitised base name crosses the IPC boundary; the full
        // target stays in the local log above.
        i18n::t_args(
            "download-write-failed",
            &[("path", &name), ("error", &e.to_string())],
        )
    })?;
    log::info!("download IPC: saved {}", scoped.display());
    Ok(Some(scoped.to_string_lossy().to_string()))
}

#[cfg(test)]
mod tests {
    use super::{configured_hosts, ip_is_public, url_allowed};
    use std::net::IpAddr;

    fn allowed(raw: &str) -> bool {
        match reqwest::Url::parse(raw) {
            Ok(url) => url_allowed(&url, &[], 8080),
            Err(_) => false,
        }
    }

    #[test]
    fn own_origins_are_allowed() {
        for good in [
            "http://127.0.0.1:8080/api/files/x",
            "http://localhost:8080/api/files/x",
            "http://localhost:3000/suite/",
            "http://127.0.0.1:4000/cloud/",
            "https://github.com/generalbots/generalbots/releases/download/v1/a.tar.gz",
            "https://objects.githubusercontent.com/asset?token=x",
            "https://generalbots.org/",
            "https://beiner.generalbots.org/drive",
        ] {
            assert!(allowed(good), "own origin refused: {good}");
        }
    }

    #[test]
    fn hostile_loopback_and_internal_targets_are_refused() {
        for hostile in [
            "http://localhost:22/",
            "http://127.0.0.1:9000/",
            "http://[::1]:8080/",
            "http://[::ffff:127.0.0.1]:8080/",
            "http://169.254.169.254/latest/meta-data/",
            "http://169.254.169.254:8080/latest/meta-data/",
            "http://10.0.0.1:8080/internal",
            "http://192.168.1.1:8080/internal",
            "http://172.16.0.1:8080/internal",
            "http://0.0.0.0:8080/internal",
            "http://100.64.0.1:8080/internal",
        ] {
            assert!(!allowed(hostile), "SSRF URL accepted: {hostile}");
        }
    }

    #[test]
    fn arbitrary_and_sibling_hosts_are_refused() {
        for hostile in [
            "https://example.com/file",
            "http://github.com/file",
            "http://github.com:8080/file",
            "https://evil.pragmatismo.com.br/",
            "https://pragmatismo.com.br.evil.example/",
            "https://a.b.generalbots.org/drive",
            "https://evilgeneralbots.org/drive",
            "https://github.com.evil.example/",
            "http://user@127.0.0.1:8080/file",
            "file:///etc/passwd",
            "//evil.example/file",
        ] {
            assert!(!allowed(hostile), "untrusted URL accepted: {hostile}");
        }
    }

    #[test]
    fn operator_hosts_are_exact_domain_names() {
        let extra = configured_hosts(Some(
            " Chat.Example.com , 10.0.0.1, *.evil.example, good.example ",
        ));
        assert_eq!(
            extra,
            vec!["chat.example.com".to_string(), "good.example".to_string()]
        );
        assert!(configured_hosts(None).is_empty());
        let url = reqwest::Url::parse("https://chat.example.com/file").unwrap();
        assert!(url_allowed(&url, &extra, 8080));
        let bad = reqwest::Url::parse("https://evil.example/file").unwrap();
        assert!(!url_allowed(&bad, &extra, 8080));
    }

    #[test]
    fn only_public_addresses_pass_ssrf_check() {
        for private in [
            "127.0.0.1",
            "::1",
            "169.254.169.254",
            "10.0.0.1",
            "192.168.1.1",
            "172.16.0.1",
            "0.0.0.0",
            "224.0.0.1",
            "::ffff:127.0.0.1",
            "100.64.0.1",
        ] {
            let ip = private.parse::<IpAddr>().unwrap();
            assert!(!ip_is_public(ip), "non-public IP accepted: {private}");
        }
        for public in ["8.8.8.8", "140.82.112.3", "2606:4700::1111"] {
            let ip = public.parse::<IpAddr>().unwrap();
            assert!(ip_is_public(ip), "public IP refused: {public}");
        }
    }
}
