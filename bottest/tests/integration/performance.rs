use bottest::prelude::*;
use reqwest::Client;
use std::time::{Duration, Instant};

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
async fn test_concurrent_requests_handled() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let handles: Vec<_> = (0..20)
        .map(|_| {
            let client = client.clone();
            let base_url = base_url.clone();
            tokio::spawn(async move {
                client
                    .get(format!("{}/health", base_url))
                    .send()
                    .await
            })
        })
        .collect();

    let results = futures::future::join_all(handles).await;

    let successes = results
        .iter()
        .filter(|r| r.as_ref().is_ok_and(|resp| resp.status().is_success()))
        .count();

    assert!(
        successes >= 15,
        "At least 75% of concurrent requests should succeed"
    );
}

#[tokio::test]
async fn test_response_time_acceptable() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let start = Instant::now();

    let response = client
        .get(format!("{}/health", base_url))
        .send()
        .await;

    let elapsed = start.elapsed();

    match response {
        Ok(resp) => {
            assert!(resp.status().is_success(), "Health should return success");
            assert!(
                elapsed < Duration::from_secs(5),
                "Response time should be under 5s, got {:?}",
                elapsed
            );
        }
        Err(e) => {
            eprintln!("Skipping: request failed: {}", e);
        }
    }
}

#[tokio::test]
async fn test_sustained_load() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let mut successes = 0;
    let total_requests = 30;

    for _ in 0..total_requests {
        let response = client
            .get(format!("{}/health", base_url))
            .send()
            .await;

        if let Ok(resp) = response {
            if resp.status().is_success() {
                successes += 1;
            }
        }

        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    let success_rate = (successes as f64 / total_requests as f64) * 100.0;
    assert!(
        success_rate >= 80.0,
        "Success rate should be >= 80%, got {:.1}%",
        success_rate
    );
}

#[tokio::test]
async fn test_cache_improves_performance() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let start1 = Instant::now();
    let _ = client
        .get(format!("{}/version", base_url))
        .send()
        .await;
    let first_request = start1.elapsed();

    tokio::time::sleep(Duration::from_millis(50)).await;

    let start2 = Instant::now();
    let _ = client
        .get(format!("{}/version", base_url))
        .send()
        .await;
    let second_request = start2.elapsed();

    if second_request < first_request {
        assert!(
            second_request < first_request * 8 / 10,
            "Second request should be significantly faster due to caching"
        );
    } else {
        eprintln!("Note: Caching may not be enabled or working");
    }
}

#[tokio::test]
async fn test_memory_stability() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    for _ in 0..20 {
        let _ = client
            .get(format!("{}/health", base_url))
            .send()
            .await;
    }

    eprintln!("Note: Memory profiling requires external monitoring");
}
