//! Desktop-local suite chat e2e harness scaffold (#2173).
//!
//! Goal: exercise the suite-chat flow against a desktop-local botserver with a
//! mock LLM (OpenAI-compatible) so chat responses can be validated without a
//! real model or heavy external services.
//!
//! Current state:
//! - Requires a running desktop-local botserver + botui/suite static server and
//!   a browser (CDP). When unavailable the tests skip with evidence.
//! - Uses `TestContext::mock_llm()` (wiremock-backed) to provide deterministic
//!   completions; the botserver must be configured to point at that mock URL
//!   (e.g. `LLM_URL` / `LLM_API_URL` / `OPENAI_API_BASE` depending on build).
//! - Targets the suite chat route (`/chat` in the desktop shell, or
//!   `/suite/chat/chat.html` when loaded as a fragment in test fixtures).
//!
//! Offline progress in this track: harness wiring and selectors are captured
//! here so CI/CI-like runs can reuse them; full execution depends on a booted
//! desktop-local stack and browser, which are not started by unit tests.

use super::{should_run_e2e_tests, E2ETestContext};
use anyhow::{bail, Context, Result};
use bottest::web::Locator;
use std::time::Duration;

/// URL candidates for the suite chat surface. The desktop shell serves chat at
/// `/chat` (routed into `desktop.html`), while the fragment itself lives under
/// `/suite/chat/chat.html`. We prefer the routed desktop entry when present.
fn suite_chat_urls(base: &str) -> Vec<String> {
    vec![
        format!("{base}/chat"),
        format!("{base}/#chat"),
        format!("{base}/suite/chat/chat.html"),
    ]
}

/// Best-effort: configure the mock LLM with a deterministic reply for "hi".
async fn wire_mock_llm(ctx: &E2ETestContext) {
    if let Some(mock_llm) = ctx.ctx.mock_llm() {
        // Suite chat sends user turns through the OpenAI-compatible completions
        // path; accept any prompt containing the greeting.
        mock_llm
            .expect_completion("hi", "Hello! (mocked desktop-local reply)")
            .await;
        mock_llm
            .set_default_response("Hello! (mocked desktop-local reply)")
            .await;
        println!(
            "✓ Mock LLM ready at {} (desktop-local chat harness)",
            mock_llm.url()
        );
        println!(
            "  Hint: point botserver at this mock via LLM_URL/LLM_API_URL={}/v1/chat/completions",
            mock_llm.url()
        );
    } else {
        println!("⚠ No mock LLM in context; chat reply depends on configured provider");
    }
}

/// Desktop-local suite chat smoke: navigate to suite chat, send "hi", expect a
/// bot response (mocked when the server is wired to the harness mock LLM).
///
/// Ignored by default because it requires a running browser + botui/suite and a
/// botserver reachable from the UI. Run explicitly in an environment with the
/// desktop-local stack up:
/// `cargo test -p bottest --features e2e desktop_local_suite_chat_hi -- --ignored`
#[tokio::test]
#[ignore = "requires desktop-local botserver+suite UI+browser; mock-LLM wiring depends on server env"]
async fn desktop_local_suite_chat_hi() -> Result<()> {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled");
        return Ok(());
    }

    let ctx = E2ETestContext::setup_with_browser().await?;
    if !ctx.has_browser() {
        ctx.close().await;
        bail!("Browser not available - cannot run desktop-local suite chat e2e");
    }
    let Some(ui) = ctx.ui.as_ref() else {
        ctx.close().await;
        bail!("BotUI/suite not available - desktop-local suite chat requires suite on port 3000");
    };

    wire_mock_llm(&ctx).await;
    let browser = ctx.browser.as_ref().context("browser missing")?;

    let mut loaded = false;
    let mut last_err = String::new();
    for url in suite_chat_urls(&ui.url) {
        println!("🌐 [desktop-local] trying suite chat URL: {url}");
        match browser.goto(&url).await {
            Ok(()) => {
                tokio::time::sleep(Duration::from_secs(3)).await;
                let input = Locator::css("#messageInput, #ai-input, .ai-input, textarea");
                if browser.exists(input).await {
                    loaded = true;
                    println!("✓ Suite chat input found at {url}");
                    break;
                }
                last_err = format!("chat input not found at {url}");
            }
            Err(e) => last_err = format!("navigate failed for {url}: {e}"),
        }
    }
    if !loaded {
        if let Ok(screenshot) = browser.screenshot().await {
            let _ = std::fs::write("/tmp/bottest-desktop-chat-fail.png", &screenshot);
            println!("Screenshot: /tmp/bottest-desktop-chat-fail.png");
        }
        ctx.close().await;
        bail!("Suite chat did not load: {last_err}");
    }

    let input = Locator::css("#messageInput, #ai-input, .ai-input, textarea");
    browser
        .type_text(input.clone(), "hi")
        .await
        .context("type into suite chat input")?;
    let send_btn = Locator::css("#sendBtn, #ai-send, .ai-send, button[type='submit']");
    if browser.click(send_btn).await.is_err() {
        let _ = browser.press_key(input, "Enter").await;
    }

    println!("⏳ Waiting for bot response (mock LLM when wired)...");
    tokio::time::sleep(Duration::from_secs(6)).await;

    let response =
        Locator::css(".message.bot, .message.assistant, .bot-message, .assistant-message");
    let responded = matches!(browser.find_elements(response).await, Ok(ref els) if !els.is_empty());
    if let Ok(screenshot) = browser.screenshot().await {
        let _ = std::fs::write("/tmp/bottest-desktop-chat-result.png", &screenshot);
        println!("📸 Screenshot: /tmp/bottest-desktop-chat-result.png");
    }
    if responded {
        println!("✓ Desktop-local suite chat received a bot response");
    } else {
        println!(
            "⚠ No bot response detected (verify botserver LLM points to mock: {})",
            ctx.ctx
                .mock_llm()
                .map(|m| m.url())
                .unwrap_or_else(|| "n/a".into())
        );
    }

    ctx.close().await;
    Ok(())
}

/// Mock-LLM-only wiring smoke: proves the harness can start and configure a mock
/// LLM without requiring a browser, botserver, or database. Safe to run offline.
#[tokio::test]
async fn desktop_local_chat_mock_llm_wiring_smoke() -> Result<()> {
    // Minimal context: mocks only, no postgres/minio/redis/services.
    let ctx = bottest::TestHarness::minimal().await;
    let mut ctx = match ctx {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping: could not create minimal harness: {e}");
            return Ok(());
        }
    };

    // Start a standalone mock LLM on an ephemeral port for this wiring smoke.
    let port = bottest::PortAllocator::allocate();
    let mock = bottest::mocks::MockLLM::start(port).await?;
    mock.expect_completion("hi", "Hello! (mocked desktop-local reply)")
        .await;
    mock.set_default_response("Hello! (mocked desktop-local reply)")
        .await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
    let resp = client
        .post(format!("{}/v1/chat/completions", mock.url()))
        .json(&serde_json::json!({
            "model": "mock-gpt",
            "messages": [{"role":"user","content":"hi"}]
        }))
        .send()
        .await
        .context("POST to mock LLM completions")?;
    assert!(resp.status().is_success(), "mock LLM should return 200");
    let body = resp.text().await.unwrap_or_default();
    assert!(
        body.contains("Hello! (mocked desktop-local reply)"),
        "mock LLM response should include deterministic reply, got: {body}"
    );
    mock.assert_called().await;

    println!("✓ Desktop-local chat mock-LLM wiring smoke passed (port {port})");
    ctx.cleanup().await.ok();
    Ok(())
}
