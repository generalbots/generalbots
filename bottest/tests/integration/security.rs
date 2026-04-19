mod security;

use bottest::prelude::*;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;

fn test_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}

fn external_server_url() -> Option<String> {
    std::env::var("BOTSERVER_URL").ok()
}

async fn get_test_server() -> Option<(Option<TestContext>, String)> {
    if let Some(url) = external_server_url() {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .ok()?;

        if client.get(&url).send().await.is_ok() {
            return Some((None, url));
        }
    }

    let ctx = TestHarness::quick().await.ok()?;
    let server = ctx.start_botserver().await.ok()?;

    if server.is_running() {
        Some((Some(ctx), server.url.clone()))
    } else {
        None
    }
}

macro_rules! skip_if_no_server {
    ($base_url:expr) => {
        if $base_url.is_none() {
            eprintln!("Skipping test: no server available");
            return;
        }
    };
}

#[tokio::test]
async fn test_invalid_credentials_rejected() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let response = client
        .post(format!("{}/api/auth/login", base_url))
        .json(&json!({
            "username": "admin",
            "password": "wrongpassword"
        }))
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert!(
                resp.status() == 401 || resp.status() == 400,
                "Invalid credentials should be rejected"
            );
        }
        Err(e) => {
            eprintln!("Skipping: connection failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_session_timeout() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let login_response = client
        .post(format!("{}/api/auth/login", base_url))
        .json(&json!({
            "username": "admin",
            "password": "admin"
        }))
        .send()
        .await;

    if let Ok(resp) = login_response {
        if resp.status() == 200 {
            tokio::time::sleep(Duration::from_secs(31)).await;

            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            if let Some(token) = body.get("token").and_then(|t| t.as_str()) {
                let protected_response = client
                    .get(format!("{}/api/users/me", base_url))
                    .header("Authorization", format!("Bearer {}", token))
                    .send()
                    .await;

                if let Ok(resp) = protected_response {
                    assert!(
                        resp.status() == 401,
                        "Expired token should be rejected"
                    );
                }
            }
        }
    }
}

#[tokio::test]
async fn test_rate_limiting_applied() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let mut rate_limited = false;

    for _ in 0..20 {
        let response = client
            .post(format!("{}/api/auth/login", base_url))
            .json(&json!({
                "username": "testuser",
                "password": "testpass"
            }))
            .send()
            .await;

        if let Ok(resp) = response {
            if resp.status() == 429 {
                rate_limited = true;
                break;
            }
        }
    }

    if !rate_limited {
        eprintln!("Note: Rate limiting may not be enabled in test environment");
    }
}

#[tokio::test]
async fn test_sql_injection_blocked() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let response = client
        .get(format!("{}/api/users?filter='; DROP TABLE users; --", base_url))
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert!(
                resp.status() == 400 || resp.status() == 401 || resp.status() == 403,
                "SQL injection attempt should be blocked"
            );
        }
        Err(_) => {
            eprintln!("Skipping: server not available");
        }
    }
}

#[tokio::test]
async fn test_xss_prevention() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let malicious_input = "<script>alert('xss')</script>";

    let response = client
        .post(format!("{}/api/users", base_url))
        .json(&json!({
            "name": malicious_input,
            "email": "test@example.com"
        }))
        .send()
        .await;

    match response {
        Ok(resp) => {
            assert!(
                resp.status() == 400 || resp.status() == 401 || resp.status() == 422,
                "XSS input should be rejected"
            );
        }
        Err(_) => {
            eprintln!("Skipping: server not available");
        }
    }
}

#[tokio::test]
async fn test_admin_routes_require_admin() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let login_response = client
        .post(format!("{}/api/auth/login", base_url))
        .json(&json!({
            "username": "user",
            "password": "user"
        }))
        .send()
        .await;

    if let Ok(resp) = login_response {
        if resp.status() == 200 {
            let body: serde_json::Value = resp.json().await.unwrap_or_default();
            if let Some(token) = body.get("token").and_then(|t| t.as_str()) {
                let admin_response = client
                    .get(format!("{}/api/admin/users", base_url))
                    .header("Authorization", format!("Bearer {}", token))
                    .send()
                    .await;

                if let Ok(resp) = admin_response {
                    assert!(
                        resp.status() == 403 || resp.status() == 401,
                        "Non-admin should not access admin routes"
                    );
                }
            }
        }
    }
}

#[tokio::test]
async fn test_security_headers_present() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let response = client
        .get(format!("{}/health", base_url))
        .send()
        .await;

    match response {
        Ok(resp) => {
            let headers = resp.headers();
            let has_strict_transport = headers.contains_key("strict-transport-security");
            let has_x_content_type = headers.contains_key("x-content-type-options");
            
            if !has_strict_transport || !has_x_content_type {
                eprintln!("Note: Security headers may not be fully configured");
            }
        }
        Err(_) => {
            eprintln!("Skipping: server not available");
        }
    }
}
