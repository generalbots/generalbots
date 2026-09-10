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
    let (caddy, _) = botcoresecrets::app_runtime();
    if caddy.is_empty() {
        std::env::var("CADDY_API_URL").unwrap_or_else(|_| "http://127.0.0.1:2019".to_string())
    } else {
        caddy
    }
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

/// Forward-auth target: the vibe domain-auth endpoint on the botserver API.
/// The base URL comes from the app runtime secret (`vibe_api_url`), which in
/// production must point at a botserver address reachable from the proxy
/// container (e.g. `http://bot.incus:5858`).
fn auth_uri(domain: &str) -> String {
    let (_, vibe) = botcoresecrets::app_runtime();
    format!("{vibe}/api/vibe/domain-auth?domain={domain}")
}

fn route_payload(domain: &str, dial: &str, rid: &str, access: &str) -> serde_json::Value {
    let proxy = serde_json::json!({
        "handler": "reverse_proxy",
        "upstreams": [{ "dial": dial }]
    });
    if access == "public" {
        serde_json::json!({
            "@id": rid,
            "handle": [proxy],
            "match": [{ "host": [domain] }],
            "terminal": true,
        })
    } else {
        // Access-controlled app: run the JWT check first (forward_auth), and
        // only when it passes (2xx) proxy to the app container. Non-2xx
        // responses (401/302/403) go straight back to the client.
        serde_json::json!({
            "@id": rid,
            "handle": [{
                "handler": "subroute",
                "routes": [
                    {
                        "handle": [{
                            "handler": "forward_auth",
                            "uri": auth_uri(domain),
                            "headers": {}
                        }],
                        "terminal": true
                    },
                    {
                        "handle": [proxy],
                        "terminal": true
                    }
                ]
            }],
            "match": [{ "host": [domain] }],
            "terminal": true,
        })
    }
}

pub async fn upsert_route(domain: &str, container: &str, access: &str) -> Result<CaddyResult, String> {
    let dial = format!("{container}.incus:80");
    upsert_route_to(domain, &dial, access).await
}

/// #1261 — upsert a Caddy reverse-proxy route for `domain` → `dial`, where
/// `dial` is an explicit `host:port` (e.g. `10.0.0.42:3000`). The proxy
/// container cannot resolve `{container}.incus` names, so the vibe publish
/// path resolves the container's real IP at deploy time and dials it
/// directly, keeping published apps reachable at `{app}.{platform-domain}`.
pub async fn upsert_route_to(domain: &str, dial: &str, access: &str) -> Result<CaddyResult, String> {
    let base = caddy_api_url();
    let client = client()?;
    let rid = route_id(domain);

    // Remove ALL previous routes matching this host before inserting. Caddy
    // serves the FIRST matching route, so a stale duplicate can 502 the app
    // even when a correct route also exists (#1265). Duplicate @ids also make
    // the id index unresolvable (DELETE /config/id/{rid} returns 500
    // `invalid traversal path`), so removal goes through the routes LIST
    // INDEX: snapshot once, collect every matching index, then delete from
    // the HIGHEST index down. Deleting a higher index never shifts lower
    // ones, so each captured index stays valid for the whole pass — no
    // arithmetic on the index itself (a subtracted `removed` counter here
    // once deleted foreign hosts' routes; see #1305).
    let routes_path = "config/apps/http/servers/srv0/routes";
    delete_routes_matching_host(&base, &client, routes_path, domain).await?;

    let body = route_payload(domain, dial, &rid, access);
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
    // #1302 — persist the route into the proxy's Caddyfile so a manual
    // `caddy reload` (which rebuilds from the file and drops admin-API
    // routes) does not 502 the published app. Enabled by setting
    // `CADDYFILE_INCUS_CONTAINER` (e.g. `proxy`): the config is pulled
    // from that container, the site block appended/updated, and pushed
    // back. Purely in-memory installs keep the admin-API behavior.
    if access == "public" {
        if let Ok(container) = std::env::var("CADDYFILE_INCUS_CONTAINER") {
            if !container.trim().is_empty() {
                persist_route_block(&container, domain, dial);
            }
        }
    }
    Ok(CaddyResult { route_id: rid })
}

/// Appends/updates the site block for `domain` ({domain} → {dial}) in the
/// Caddyfile of the given incus container (e.g. `proxy`), via file
/// pull/edit/push — the same argv-clean mechanism the VM workflow uses.
/// Idempotent: an existing block whose host line matches is replaced.
/// Failures are logged, never fatal — the admin-API route stays live and a
/// broken edit is never pushed (the edit happens on a temp copy).
fn persist_route_block(container: &str, domain: &str, dial: &str) {
    use std::io::Write;
    // Hostname guard: the domain is interpolated into the Caddyfile, so only
    // plain hostname characters are accepted — no braces/newlines/spaces that
    // could inject Caddyfile directives.
    let valid_domain = !domain.is_empty()
        && domain
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-');
    if !valid_domain {
        log::warn!("Caddyfile persistence skipped: invalid domain '{domain}'");
        return;
    }
    let block = format!(
        "\n{domain} {{ \n\timport tls_config\n\treverse_proxy {dial}\n}}\n"
    );
    let result = (|| -> Result<(), String> {
        if !container
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-')
        {
            return Err("invalid proxy container name".to_string());
        }
        let run = |args: &[&str]| -> Result<String, String> {
            let mut cmd =
                botlib::security::SafeCommand::new("incus").map_err(|e| e.to_string())?;
            for a in args {
                cmd = cmd.trusted_arg(a).map_err(|e| e.to_string())?;
            }
            let out = cmd.execute().map_err(|e| e.to_string())?;
            Ok(String::from_utf8_lossy(&out.stdout).to_string())
        };
        let conf = "/opt/gbo/conf/config";
        let tmp = std::env::temp_dir().join(format!("gbo-caddy-{uuid}.cfg", uuid = uuid::Uuid::new_v4()));
        run(&["file", "pull", &format!("{container}{conf}"), tmp.to_str().ok_or("tmp path")?])?;
        let raw = std::fs::read_to_string(&tmp).map_err(|e| format!("read: {e}"))?;
        let lines: Vec<&str> = raw.split('\n').collect();
        let start = lines.iter().position(|l| l.trim() == format!("{domain} {{"));
        let out = match start {
            Some(i) => {
                let mut end = i;
                for (j, l) in lines.iter().enumerate().skip(i + 1) {
                    if l.trim() == "}" {
                        end = j;
                        break;
                    }
                }
                let mut v = lines[..i].to_vec();
                v.extend(block.lines());
                v.extend(&lines[end + 1..]);
                v.join("\n")
            }
            None => format!("{raw}\n{block}"),
        };
        let edited = tmp.with_file_name(format!("gbo-caddy-edited-{}", uuid::Uuid::new_v4()));
        let mut f = std::fs::File::create(&edited).map_err(|e| format!("open: {e}"))?;
        f.write_all(out.as_bytes()).map_err(|e| format!("write: {e}"))?;
        drop(f);
        // Never push an unvalidated edit over the live config: stage the
        // candidate inside the proxy, run `caddy validate` against it, and
        // only on success back up the live file and swap. A rejected edit
        // leaves the production Caddyfile untouched.
        let candidate = "/tmp/gbo-caddy-candidate.cfg";
        let backup = "/tmp/gbo-caddy-config.prev";
        run(&["file", "push", edited.to_str().ok_or("edited path")?, &format!("{container}{candidate}")])?;
        let validate = run(&[
            "exec", container, "--", "caddy", "validate",
            "--adapter", "caddyfile", "--config", candidate,
        ]);
        if validate.is_err() {
            let _ = run(&["exec", container, "--", "rm", "-f", candidate]);
            return Err("candidate Caddyfile rejected by caddy validate".to_string());
        }
        run(&["exec", container, "--", "cp", conf, backup])?;
        run(&["file", "push", edited.to_str().ok_or("edited path")?, &format!("{container}{conf}")])?;
        let _ = run(&["exec", container, "--", "rm", "-f", candidate]);
        let _ = std::fs::remove_file(&tmp);
        let _ = std::fs::remove_file(&edited);
        Ok(())
    })();
    if let Err(e) = result {
        log::warn!("Caddyfile persistence for {domain} failed: {e}");
    }
}

pub async fn remove_route(domain: &str) -> Result<(), String> {
    let base = caddy_api_url();
    let client = client()?;
    // Single authoritative sweep by match-host list index (see upsert for
    // why the id path is not used). Best-effort: a 404 (nothing to remove)
    // is a normal outcome; the snapshot-and-delete pass covers duplicates.
    let routes_path = "config/apps/http/servers/srv0/routes";
    delete_routes_matching_host(&base, &client, routes_path, domain).await
}

/// True when a Caddy route's `match` targets `domain` in any of its host
/// matchers. Only exact host equality qualifies — never a prefix or suffix
/// match, so `app.example.com` cannot touch `example.com` or a shared
/// wildcard route.
fn route_matches_host(route: &serde_json::Value, domain: &str) -> bool {
    route
        .get("match")
        .and_then(|m| m.as_array())
        .map(|arr| {
            arr.iter().any(|m| {
                m.get("host")
                    .and_then(|h| h.as_array())
                    .map(|hosts| hosts.iter().any(|h| h.as_str() == Some(domain)))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}

/// Snapshot the routes array once, then delete every route matching
/// `domain` from the highest index down. Safety properties:
///
/// 1. Each index is captured from ONE consistent snapshot and used as-is —
///    reverse-order deletion keeps lower indices stable, so only routes
///    verified to match `domain` are ever removed.
/// 2. A bulk-delete guard aborts when more than `MAX_DELETES` routes match:
///    that signals a corrupted config, and blindly wiping it could take
///    down unrelated production sites.
///
/// Fails only when the proxy is unreachable; a missing list (404) and a
/// failed individual delete are tolerated (logged), matching the previous
/// best-effort behavior.
const MAX_HOST_ROUTE_DELETES: usize = 8;

async fn delete_routes_matching_host(
    base: &str,
    client: &reqwest::Client,
    routes_path: &str,
    domain: &str,
) -> Result<(), String> {
    let resp = client
        .get(format!("{base}/{routes_path}"))
        .send()
        .await
        .map_err(|e| format!("caddy proxy unreachable: {e}"))?;
    if !resp.status().is_success() {
        log::debug!("Caddy: routes list unavailable ({}) for {domain}", resp.status());
        return Ok(());
    }
    let routes: Vec<serde_json::Value> = match resp.json().await {
        Ok(r) => r,
        Err(e) => {
            log::warn!("Caddy: routes list not JSON for {domain}: {e}");
            return Ok(());
        }
    };
    let mut indices: Vec<usize> = routes
        .iter()
        .enumerate()
        .filter(|(_, route)| route_matches_host(route, domain))
        .map(|(i, _)| i)
        .collect();
    if indices.len() > MAX_HOST_ROUTE_DELETES {
        return Err(format!(
            "caddy safety abort: {} routes match {domain} (max {MAX_HOST_ROUTE_DELETES}) — refusing bulk delete",
            indices.len()
        ));
    }
    // Highest index first; each raw captured index stays valid.
    indices.sort_unstable();
    for index in indices.into_iter().rev() {
        log::info!("Caddy: removing stale route index {index} for {domain}");
        let url = format!("{base}/{routes_path}/{index}");
        if let Ok(resp) = client.delete(url).send().await {
            if !resp.status().is_success() {
                log::warn!(
                    "Caddy: index delete for {domain} at {index} returned {}",
                    resp.status()
                );
            }
        }
    }
    Ok(())
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
        let body = route_payload("app.example.com", "proj-prod.incus:80", "gbd-app.example.com", "public");
        let text = serde_json::to_string(&body).unwrap_or_default();
        assert!(text.contains("proj-prod.incus:80"));
        assert!(text.contains("app.example.com"));
        assert!(!text.contains("forward_auth"));
    }

    #[test]
    fn host_matcher_is_exact_only() {
        let route = serde_json::json!({
            "match": [{ "host": ["app.example.com"] }],
            "handle": []
        });
        assert!(route_matches_host(&route, "app.example.com"));
        assert!(!route_matches_host(&route, "example.com"));
        assert!(!route_matches_host(&route, "other.example.com"));
        let wildcard = serde_json::json!({ "match": [{ "host": ["*.example.com"] }] });
        assert!(!route_matches_host(&wildcard, "app.example.com"));
        let bare = serde_json::json!({ "handle": [] });
        assert!(!route_matches_host(&bare, "app.example.com"));
    }

    #[test]
    fn access_controlled_route_installs_forward_auth() {
        let body = route_payload("app.example.com", "proj-prod.incus:80", "gbd-app.example.com", "rbac");
        let text = serde_json::to_string(&body).unwrap_or_default();
        assert!(text.contains("forward_auth"));
        assert!(text.contains("/api/vibe/domain-auth?domain=app.example.com"));
        assert!(text.contains("proj-prod.incus:80"));
    }
}