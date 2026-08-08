//! #756/#770 — Caddy admin API driver.
//!
//! Thin client over Caddy's admin API (`CADDY_API_URL`, default
//! `http://127.0.0.1:2019`): upserts a reverse-proxy route mapping a custom
//! domain to a project container, or removes it. Caddy's automatic HTTPS
//! policy issues the ACME certificate for the host on first request, which
//! is what #770 relies on; when the proxy is unreachable (dev/offline) all
//! calls return structured errors instead of panicking.

use std::time::Duration;

pub struct CaddyResult {
    pub route_id: String,
}

fn caddy_api_url() -> String {
    std::env::var("CADDY_API_URL").unwrap_or_else(|_| "http://127.0.0.1:2019".to_string())
}

fn client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|e| format!("caddy http client: {e}"))
}

fn route_id(domain: &str) -> String {
    format!("gbd-{domain}")
}

fn route_payload<'a>(domain: &'a str, dial: &'a str, rid: &'a str) -> serde_json::Value {
    serde_json::json!({
        "@id": rid,
        "handle": [{
            "handler": "reverse_proxy",
            "upstreams": [{ "dial": dial }]
        }],
        "match": [{ "host": [domain] }],
        "terminal": true,
    })
}

pub async fn upsert_route(domain: &str, container: &str) -> Result<CaddyResult, String> {
    let base = caddy_api_url();
    let client = client()?;
    let rid = route_id(domain);

    let _ = client
        .delete(format!("{base}/config/id/{rid}"))
        .send()
        .await;

    let dial = format!("{container}.incus:80");
    let body = route_payload(domain, &dial, &rid);
    let resp = client
        .post(format!("{base}/config/apps/http/servers/srv0/routes"))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("caddy upsert failed: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(format!("caddy upsert returned {status}: {text}"));
    }
    Ok(CaddyResult { route_id: rid })
}

pub async fn remove_route(domain: &str) -> Result<(), String> {
    let base = caddy_api_url();
    let client = client()?;
    let resp = client
        .delete(format!("{base}/config/id/{}", route_id(domain)))
        .send()
        .await
        .map_err(|e| format!("caddy proxy unreachable: {e}"))?;
    if resp.status().is_success() || resp.status().as_u16() == 404 {
        Ok(())
    } else {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Err(format!("caddy route removal returned {status}: {text}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn route_ids_are_domain_scoped() {
        assert_eq!(route_id("app.example.com"), "gbd-app.example.com");
    }

    #[test]
    fn route_body_targets_container_dial() {
        let body = route_payload("app.example.com", "proj-prod.incus:80", "gbd-app.example.com");
        let text = serde_json::to_string(&body).unwrap_or_default();
        assert!(text.contains("proj-prod.incus:80"));
        assert!(text.contains("app.example.com"));
    }
}