use super::*;
use botsecurity_core::validation::{fetch_user_url, FetchPolicy};
use std::time::Duration;

/// Body ceiling for one proxied page. The shared fetch helper caps the body
/// while streaming it, so a hostile page cannot exhaust server memory even on
/// a fast link (#2779).
const PROXY_MAX_BYTES: usize = 2 * 1024 * 1024;

/// Proxies an external URL for the in-app browser (srcdoc iframe). Fetches
/// server-side to avoid CORS, strips dangerous response headers and rewrites
/// absolute links to stay in the proxy. Private/internal destinations are
/// rejected (no SSRF via the browser app) and every redirect hop is validated.
pub(super) async fn handle_proxy(
    Query(params): Query<ProxyParams>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, String)> {
    let target = &params.url;
    // Keep the BAD_REQUEST mapping for malformed/private URLs. The shared
    // fetch re-validates every hop (#2131) and, unlike the raw reqwest path
    // used here before, caps both body bytes and wall time (#2779).
    botsecurity_core::validation::validate_url_ssrf_strict(target)
        .map_err(|e| (axum::http::StatusCode::BAD_REQUEST, e.to_string()))?;

    let policy = FetchPolicy::default()
        .with_timeout(Duration::from_secs(15))
        .with_max_bytes(PROXY_MAX_BYTES)
        .with_user_agent("GeneralBots-BrowserProxy/1.0");
    let response = match fetch_user_url(target, &policy).await {
        Ok(response) => response,
        Err(err) => {
            log::warn!(
                "browser proxy: refused or failed page fetch ({}): {}",
                err.kind().as_str(),
                err.detail()
            );
            return Err((axum::http::StatusCode::BAD_GATEWAY, err.to_string()));
        }
    };

    let status = response.status;
    let final_url = response.final_url;
    // Read the body first: `text()` borrows the response, while taking
    // `content_type` below moves it out.
    let body = response.text();
    let content_type = response
        .content_type
        .unwrap_or_else(|| "application/octet-stream".to_string());
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
