mod api;
mod basic_runtime;
mod database;
mod security;
mod performance;
mod compliance;
mod accessibility;
mod internationalization;

use bottest::prelude::*;

pub async fn setup_database_test() -> TestContext {
    TestHarness::database_only()
        .await
        .expect("Failed to setup database test context")
}

pub async fn setup_quick_test() -> TestContext {
    TestHarness::quick()
        .await
        .expect("Failed to setup quick test context")
}

pub async fn setup_full_test() -> TestContext {
    TestHarness::full()
        .await
        .expect("Failed to setup full test context")
}

pub fn should_run_integration_tests() -> bool {
    if std::env::var("SKIP_INTEGRATION_TESTS").is_ok() {
        return false;
    }
    true
}

#[tokio::test]
async fn test_harness_database_only() {
    if !should_run_integration_tests() {
        eprintln!("Skipping: integration tests disabled");
        return;
    }

    let ctx = TestHarness::database_only().await;
    match ctx {
        Ok(ctx) => {
            assert!(ctx.ports.postgres >= 15000);
            assert!(ctx.data_dir.exists());
            assert!(ctx.data_dir.to_str().unwrap().contains("bottest-"));
        }
        Err(e) => {
            eprintln!("Skipping: failed to setup test harness: {}", e);
        }
    }
}

#[tokio::test]
async fn test_harness_quick() {
    if !should_run_integration_tests() {
        eprintln!("Skipping: integration tests disabled");
        return;
    }

    let ctx = TestHarness::quick().await;
    match ctx {
        Ok(ctx) => {
            assert!(ctx.mock_llm().is_some());
            assert!(ctx.mock_zitadel().is_some());
        }
        Err(e) => {
            eprintln!("Skipping: failed to setup test harness: {}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_harness_minimal() {
    let ctx = TestHarness::minimal().await.unwrap();
    assert!(ctx.postgres().is_none());
    assert!(ctx.minio().is_none());
    assert!(ctx.redis().is_none());
    assert!(ctx.mock_llm().is_none());
    assert!(ctx.mock_zitadel().is_none());
}

#[tokio::test]
#[ignore]
async fn test_context_cleanup() {
    let mut ctx = TestHarness::minimal().await.unwrap();
    let data_dir = ctx.data_dir.clone();
    assert!(data_dir.exists());

    ctx.cleanup().await.unwrap();

    assert!(!data_dir.exists());
}
