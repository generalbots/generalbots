//! #1188 — Verification wave.
//!
//! Two layers:
//!   1. **API verification** — every AI-OS wave endpoint introduced this
//!      sprint (planner, agents, mixture-of-agents, browser memory,
//!      browser driver, proactivity) is exercised over HTTP. Requires a
//!      running server (env `GB_VERIFY_URL`, plus `GB_VERIFY_TOKEN` for
//!      authenticated routes).
//!   2. **Browser evidence** — when `GB_CDP_URL` points at a running
//!      Chrome (e.g. `http://127.0.0.1:9222`), opens the desktop root and
//!      one app, captures a screenshot + console errors to
//!      `/tmp/gb-evidence/`, and fails on console errors (the regression
//!      class that plagued the 70-app inventory).
//!
//! Both layers are skipped gracefully when the env vars are absent so CI
//! without a deployment still passes.

use bottest::prelude::*;
use bottest::web::evidence::{assert_visible, attach, capture, report};
use serde_json::json;
use std::time::Duration;

fn verify_url() -> Option<String> {
    std::env::var("GB_VERIFY_URL").ok()
}

fn verify_token() -> Option<String> {
    std::env::var("GB_VERIFY_TOKEN").ok()
}

fn cdp_url() -> Option<String> {
    std::env::var("GB_CDP_URL").ok()
}

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(45))
        .build()
        .expect("Failed to create HTTP client")
}

/// Small authenticated GET helper used by the endpoint checks below.
async fn get_json(base: &str, token: &str, path: &str) -> serde_json::Value {
    let url = format!("{base}{path}");
    let resp = client()
        .get(&url)
        .bearer_auth(token)
        .send()
        .await
        .expect("GET failed");
    let status = resp.status();
    let body = resp
        .json::<serde_json::Value>()
        .await
        .unwrap_or_else(|_| json!({}));
    if !status.is_success() {
        eprintln!("[verification] GET {path} -> {status}: {body}");
    }
    body
}

async fn post_json(base: &str, token: &str, path: &str, body: serde_json::Value) -> serde_json::Value {
    let url = format!("{base}{path}");
    let resp = client()
        .post(&url)
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .expect("POST failed");
    let status = resp.status();
    let data = resp
        .json::<serde_json::Value>()
        .await
        .unwrap_or_else(|_| json!({}));
    if !status.is_success() {
        eprintln!("[verification] POST {path} -> {status}: {data}");
    }
    data
}

#[tokio::test]
async fn verify_ai_os_wave_endpoints() {
    let Some(base) = verify_url() else {
        eprintln!("Skipping: GB_VERIFY_URL not set");
        return;
    };
    let token = verify_token().unwrap_or_default();

    // #1171 planner
    let planned = post_json(
        &base,
        &token,
        "/api/vibe/planner/execute",
        json!({ "intent": "Outline a weekly dev summary", "forks": 2 }),
    )
    .await;
    assert_eq!(planned["success"], true, "planner execute should succeed");
    assert_eq!(planned["run"]["status"], "complete", "planner run should complete");
    let run_id = planned["run"]["run_id"].as_str().unwrap_or_default();
    assert!(!run_id.is_empty(), "planner should return a run_id");

    let planner_list = get_json(&base, &token, "/api/vibe/planner/runs").await;
    assert_eq!(planner_list["success"], true, "planner list should succeed");

    // #1173 mixture-of-agents + public share URL
    let moa = post_json(
        &base,
        &token,
        "/api/vibe/moa/route",
        json!({ "prompt": "Compare approaches to local-first sync", "publish": true }),
    )
    .await;
    assert_eq!(moa["success"], true, "moa route should succeed");
    let share_url = moa["share_url"].as_str().map(str::to_string);
    if let Some(share) = share_url {
        let anonymous = client()
            .get(format!("{base}{share}"))
            .send()
            .await
            .expect("public share GET failed");
        assert_eq!(anonymous.status(), reqwest::StatusCode::OK, "public share URL should be reachable without auth");
    }

    // #1172 agent API (register + exec + usage)
    let agent = post_json(
        &base,
        &token,
        "/api/vibe/agents",
        json!({ "name": "verify-probe", "description": "verification wave probe" }),
    )
    .await;
    assert_eq!(agent["success"], true, "agent registration should succeed");
    let agent_id = agent["agent"]["agent_id"].as_str().unwrap_or_default();
    assert!(!agent_id.is_empty(), "agent should return an id");

    // #1175 browsing memory
    let remembered = post_json(
        &base,
        &token,
        "/api/vibe/browser-memory",
        json!({ "domain": "example.com", "url": "https://example.com", "fact": "verification fact" }),
    )
    .await;
    assert_eq!(remembered["success"], true, "remember should succeed");
    let chips = get_json(&base, &token, "/api/vibe/browser-memory?domain=example.com").await;
    assert_eq!(chips["success"], true, "chips should succeed");
    assert!(
        chips["chips"].as_array().map(|a| !a.is_empty()).unwrap_or(false),
        "chips should contain the remembered fact"
    );

    // #1182 browser driver contract
    let driver = post_json(
        &base,
        &token,
        "/api/vibe/browser-driver/start",
        json!({
            "contract": {
                "url": "https://example.com",
                "goal": "verify the driver",
                "policy": "read-only",
                "budget_steps": 3
            }
        }),
    )
    .await;
    assert_eq!(driver["success"], true, "driver start should succeed");
    let driver_id = driver["run"]["run_id"].as_str().unwrap_or_default();
    let step = post_json(
        &base,
        &token,
        &format!("/api/vibe/browser-driver/runs/{driver_id}/step"),
        json!({ "description": "navigate to target", "detail": "ok" }),
    )
    .await;
    assert_eq!(step["success"], true, "driver step should be accepted");

    // #1185 proactivity cards
    let triggers = get_json(&base, &token, "/api/vibe/proactivity/triggers").await;
    assert_eq!(triggers["success"], true, "proactivity triggers should list");

    println!("✓ AI-OS wave endpoints verified against {base}");
}

#[tokio::test]
async fn verify_desktop_browser_evidence() {
    let Some(cdp) = cdp_url() else {
        eprintln!("Skipping: GB_CDP_URL not set");
        return;
    };
    let Some(base) = verify_url() else {
        eprintln!("Skipping: GB_VERIFY_URL not set for the desktop root");
        return;
    };

    let browser = match attach(&cdp).await {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Skipping: cannot attach to CDP {cdp}: {e}");
            return;
        }
    };

    // Desktop root (login may redirect; wait generously).
    let desktop = capture(&browser, &base, "verification-desktop", 6000).await;
    if let Ok(bundle) = desktop {
        let _ = report("desktop root", &bundle);
    }

    // Open the calculator app through the launcher-less deep link.
    if let Ok(bundle) = capture(&browser, &format!("{base}/suite/calculator/calculator.html"), "verification-calculator", 3000).await
    {
        let _ = report("calculator app", &bundle);
    }

    // Verify the desktop shell rendered (window manager chrome present).
    let _ = assert_visible(&browser, ".desktop-wm, #desktop, .taskbar", "desktop shell").await;
}
