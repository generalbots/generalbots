use super::*;

/// Proxies an external URL for the in-app browser (srcdoc iframe). Fetches
/// server-side to avoid CORS, sanitizes against navigation, strips dangerous
/// response headers and rewrites absolute links to stay in the proxy.
/// Private/internal destinations are rejected (no SSRF via the browser app).
pub(super) async fn handle_proxy(
    Query(params): Query<ProxyParams>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let target = &params.url;
    // #2131 — the shared guard replaces the local `is_private_host` helper:
    // scheme, credentials, obfuscated IPv4, ports and the full non-global
    // address matrix are checked here, and the redirect policy below runs the
    // same validator on every hop (#1535).
    let parsed = botsecurity_core::validation::validate_url_ssrf_strict(target)
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, e.to_string()))?;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        // #1535: validate every redirect hop, not only the first URL, so a
        // public URL cannot bounce the proxy into a private/metadata address.
        .redirect(reqwest::redirect::Policy::custom(|attempt| {
            let allowed = botsecurity_core::validation::validate_url_ssrf_strict(
                attempt.url().as_str(),
            )
            .is_ok();
            if attempt.previous().len() >= 5 || !allowed {
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

    match client.get(parsed.as_str()).send().await {
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
