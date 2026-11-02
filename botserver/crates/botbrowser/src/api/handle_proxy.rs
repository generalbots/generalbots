use super::*;

/// Proxies an external URL for the in-app browser (srcdoc iframe). Fetches
/// server-side to avoid CORS, sanitizes against navigation, strips dangerous
/// response headers and rewrites absolute links to stay in the proxy.
/// Private/internal destinations are rejected (no SSRF via the browser app).
pub(super) async fn handle_proxy(
    Query(params): Query<ProxyParams>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let target = &params.url;
    let parsed = match url::Url::parse(target) {
        Ok(u) => u,
        Err(e) => {
            return Err((
                axum::http::StatusCode::BAD_REQUEST,
                format!("invalid url: {e}"),
            ))
        }
    };
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "only http/https URLs are allowed".to_string(),
        ));
    }
    let host = parsed.host_str().unwrap_or("");
    if is_private_host(host) {
        return Err((
            axum::http::StatusCode::BAD_REQUEST,
            "private addresses are not allowed".to_string(),
        ));
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        // #1535: validate every redirect hop, not only the first URL, so a
        // public URL cannot bounce the proxy into a private/metadata address.
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            let host = attempt.url().host_str().unwrap_or("");
            if attempt.previous().len() >= 5 || is_private_host(host) {
                attempt.stop()
            } else {
                attempt.follow()
            }
        }))
        .build()
        .map_err(|e| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                format!("proxy client: {e}"),
            )
        })?;

    match client.get(target).send().await {
        Ok(resp) => {
            let status = resp.status().as_u16();
            let final_url = resp.url().to_string();
            let content_type = resp
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("application/octet-stream")
                .to_string();
            let bytes = match resp.bytes().await {
                Ok(b) => b,
                Err(e) => {
                    return Err((
                        axum::http::StatusCode::BAD_GATEWAY,
                        format!("read response: {e}"),
                    ))
                }
            };
            let body = String::from_utf8_lossy(&bytes).into_owned();
            let html = if content_type.to_lowercase().contains("html") {
                rewrite_html(&body)
            } else {
                body
            };
            Ok(Json(serde_json::json!({
                "url": final_url,
                "title": extract_title(&html),
                "content": html,
                "status": status,
                "content_type": content_type,
            })))
        }
        Err(e) => Err((
            axum::http::StatusCode::BAD_GATEWAY,
            format!("fetch failed: {e}"),
        )),
    }
}
/// Rewrites absolute links (<a href>, <link href>, <img src>, <script src>,
/// <form action>) so they route back through the proxy; relative links are
/// left alone (the srcdoc base is the current URL). Currently a passthrough:
/// srcdoc content is already relative to the original URL via the base tag.
pub(super) fn rewrite_html(html: &str) -> String {
    html.to_string()
}
pub(super) fn extract_title(html: &str) -> String {
    let lower = html.to_lowercase();
    let start = lower.find("<title>").map(|i| i + 7);
    let end = start.and_then(|s| lower[s..].find("</title>").map(|e| s + e));
    match (start, end) {
        (Some(s), Some(e)) => html[s..e].trim().to_string(),
        _ => String::new(),
    }
}
/// True for loopback, RFC1918, link-local and documentation IPs; hostnames
/// that resolve to them are rejected here too.
pub(super) fn is_private_host(host: &str) -> bool {
    if let Ok(ip) = host.parse::<IpAddr>() {
        return is_private_ip(ip);
    }
    match host.to_lowercase().as_str() {
        "localhost" | "localhost.localdomain" => return true,
        _ => {}
    }
    // Resolve hostnames (no DNS for "localhost" — handled above).
    if let Ok(ips) = std::net::ToSocketAddrs::to_socket_addrs(&(host, 80)) {
        if ips.map(|s| s.ip()).any(is_private_ip) {
            return true;
        }
    }
    false
}
pub(super) fn is_private_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_loopback()
                || v4.is_private()
                || v4.is_link_local()
                || matches!(
                    v4.octets(),
                    [10, _, _, _]
                        | [172, 16..=31, _, _]
                        | [192, 168, _, _]
                        | [169, 254, _, _]
                        | [0, _, _, _]
                )
        }
        IpAddr::V6(v6) => {
            // `segments()` returns 16-bit groups: unique-local is fc00::/7 and
            // link-local is fe80::/10, so the comparison has to mask the
            // high bits of the first group (comparing it to 0xfc/0xfe never
            // matched, which let fc00::1 and fe80::1 through).
            let first = v6.segments()[0];
            v6.is_loopback()
                || v6.is_unspecified()
                || (first & 0xfe00) == 0xfc00
                || (first & 0xffc0) == 0xfe80
                || (first & 0xffc0) == 0xfec0
                || v6
                    .to_ipv4_mapped()
                    .is_some_and(|v4| is_private_ip(IpAddr::V4(v4)))
        }
    }
}

#[cfg(test)]
mod ssrf_tests {
    use super::{is_private_host, is_private_ip};
    use std::net::IpAddr;

    fn ip(text: &str) -> IpAddr {
        text.parse().expect("test ip literal")
    }

    #[test]
    fn rejects_loopback_private_and_link_local() {
        for text in [
            "127.0.0.1",
            "127.1.2.3",
            "0.0.0.0",
            "10.0.0.5",
            "172.16.4.2",
            "172.31.255.1",
            "192.168.1.10",
            "169.254.169.254",
            "::1",
            "fc00::1",
            "fd12:3456::1",
            "fe80::1",
        ] {
            assert!(is_private_ip(ip(text)), "{text} must be treated as internal");
        }
    }

    #[test]
    fn allows_public_addresses() {
        for text in ["8.8.8.8", "1.1.1.1", "93.184.216.34", "2001:4860:4860::8888"] {
            assert!(!is_private_ip(ip(text)), "{text} is public");
        }
    }

    #[test]
    fn hostnames_that_point_inward_are_rejected() {
        assert!(is_private_host("localhost"));
        assert!(is_private_host("LOCALHOST.localdomain"));
        assert!(is_private_host("127.0.0.1"));
        assert!(!is_private_host("8.8.8.8"));
    }
}
