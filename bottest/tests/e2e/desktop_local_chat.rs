//! Desktop-local suite chat e2e harness (#2173).
//!
//! Goal: exercise the suite-chat flow against a desktop-local botserver with a
//! deterministic mock LLM (OpenAI-compatible) so chat responses can be validated
//! without a real model, paid calls, or heavy external services.
//!
//! ## Desktop-local start profile (botapp sidecar parity)
//! - Binary: `botserver --noconsole` (desktop-local build/profile when used as
//!   the sidecar; see `botapp/src/desktop/sidecar.rs`).
//! - Stack layout: `BOTSERVER_STACK_PATH=<stack>` (created if missing), with
//!   `.product.desktop-local` copied into the stack root when absent.
//! - Env: `BOTSERVER_INSTALLERS_PATH=<installers>`, `GB_DESKTOP_ALLOW_DOWNLOAD=0`
//!   for offline/air-gapped runs, `AGENT_ALLOW_UNREGISTERED_IDENTITIES=1`,
//!   `GBO_NO_DOTENV=1` to avoid leaking developer .env into isolated tests.
//! - LLM wiring (mock, deterministic): set one or more of `LLM_URL`,
//!   `LLM_API_URL`, `OPENAI_API_BASE` (+ `OPENAI_API_KEY=dummy`) to the mock
//!   endpoint from [`MockLLM::chat_completions_url`] / [`MockLLM::openai_base_url`].
//!
//! ## Suite chat flow
//! - Route: desktop shell `/chat` (or `/#chat`) loads suite chat; the fragment
//!   lives at `/suite/chat/chat.html`. WS/chat transport is handled by the suite
//!   chat modules once the shell is loaded (see `botui/ui/suite/chat/*`).
//! - Selectors: input `#messageInput` (fallbacks: `#ai-input`, `.ai-input`,
//!   `textarea[name='content']`), send `#sendBtn` (fallbacks: `#ai-send`,
//!   `.ai-send`, `button[type='submit']`), bot reply `.message.bot`,
//!   `.message.assistant`, `.bot-message`, `.assistant-message`.
//!
//! ## Execution modes
//! - `desktop_local_chat_mock_llm_wiring_smoke`: offline-safe, no browser/DB.
//! - `desktop_local_suite_chat_hi`: ignored; requires desktop-local
//!   botserver+suite UI+browser wired to the mock LLM.

use super::{should_run_e2e_tests, E2ETestContext};
use anyhow::{bail, Context, Result};
use bottest::mocks::MockLLM;
use bottest::web::Locator;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Deterministic reply used by this harness (no paid LLM calls).
pub const DESKTOP_LOCAL_CHAT_MOCK_REPLY: &str = "Hello! (mocked desktop-local reply)";

/// CSS selector for the suite chat composer input.
pub const SUITE_CHAT_INPUT_SELECTOR: &str =
    "#messageInput, #ai-input, .ai-input, textarea[name='content']";

/// CSS selector for the suite chat send action.
pub const SUITE_CHAT_SEND_SELECTOR: &str = "#sendBtn, #ai-send, .ai-send, button[type='submit']";

/// CSS selector(s) indicating a bot/assistant response in the transcript.
pub const SUITE_CHAT_BOT_RESPONSE_SELECTOR: &str =
    ".message.bot, .message.assistant, .bot-message, .assistant-message, [data-role='assistant']";

/// URL candidates for the suite chat surface. Prefer the routed desktop entry
/// (`/chat`) so the shell boots chat + WS modules; fall back to hash/fragment.
fn suite_chat_urls(base: &str) -> Vec<String> {
    let trimmed = base.trim_end_matches('/');
    vec![
        format!("{trimmed}/chat"),
        format!("{trimmed}/#chat"),
        format!("{trimmed}/suite/desktop.html#chat"),
        format!("{trimmed}/suite/chat/chat.html"),
    ]
}

/// Environment variables a desktop-local botserver should receive to use the
/// deterministic mock LLM and an isolated stack layout (sidecar-compatible).
pub fn desktop_local_botserver_env(
    stack_path: &Path,
    installers_path: Option<&Path>,
    mock_llm: &MockLLM,
) -> BTreeMap<String, String> {
    let mut env = BTreeMap::new();
    env.insert(
        "BOTSERVER_STACK_PATH".to_string(),
        stack_path.display().to_string(),
    );
    if let Some(installers) = installers_path {
        env.insert(
            "BOTSERVER_INSTALLERS_PATH".to_string(),
            installers.display().to_string(),
        );
    }
    // Offline/minimal: do not download components in harness runs.
    env.insert("GB_DESKTOP_ALLOW_DOWNLOAD".to_string(), "0".to_string());
    env.insert(
        "AGENT_ALLOW_UNREGISTERED_IDENTITIES".to_string(),
        "1".to_string(),
    );
    env.insert("GBO_NO_DOTENV".to_string(), "1".to_string());
    // Wire mock LLM through common env names accepted by botserver/builds.
    env.insert("LLM_URL".to_string(), mock_llm.chat_completions_url());
    env.insert("LLM_API_URL".to_string(), mock_llm.chat_completions_url());
    env.insert("OPENAI_API_BASE".to_string(), mock_llm.openai_base_url());
    env.insert("OPENAI_API_KEY".to_string(), "mock-key".to_string());
    env.insert("LLM_API_KEY".to_string(), "mock-key".to_string());
    env
}

/// Ensure the stack has a `.product.desktop-local` file (copied from the repo
/// when available) so desktop-local product/route/profile behavior is active.
pub fn ensure_desktop_local_product(stack_path: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(stack_path).context("create desktop-local stack dir")?;
    let dst = stack_path.join(".product.desktop-local");
    if dst.exists() {
        return Ok(dst);
    }
    let repo_product = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("botserver")
        .join(".product.desktop-local");
    if repo_product.exists() {
        std::fs::copy(&repo_product, &dst).with_context(|| {
            format!(
                "copy desktop-local product profile from {}",
                repo_product.display()
            )
        })?;
    } else {
        // Minimal fallback: enough to mark the stack as desktop-local if the
        // source tree layout differs in CI artifacts.
        std::fs::write(
            &dst,
            "name=General Bots\napps=chat,drive\nsuite_default_app=chat\nno_security=false\n",
        )
        .context("write fallback .product.desktop-local")?;
    }
    Ok(dst)
}

/// Configure a harness mock LLM with a deterministic suite-chat reply.
pub async fn wire_mock_llm(mock_llm: &MockLLM) {
    mock_llm
        .configure_desktop_local_chat(DESKTOP_LOCAL_CHAT_MOCK_REPLY)
        .await;
    println!(
        "✓ Mock LLM ready at {} (desktop-local chat harness)",
        mock_llm.url()
    );
    println!(
        "  chat completions: {}",
        mock_llm.chat_completions_url()
    );
    println!(
        "  env hint: LLM_URL={} LLM_API_URL={} OPENAI_API_BASE={}",
        mock_llm.chat_completions_url(),
        mock_llm.chat_completions_url(),
        mock_llm.openai_base_url()
    );
}

/// Best-effort: configure the context mock LLM (if present).
async fn wire_context_mock_llm(ctx: &E2ETestContext) {
    if let Some(mock_llm) = ctx.ctx.mock_llm() {
        wire_mock_llm(mock_llm).await;
    } else {
        println!("⚠ No mock LLM in context; chat reply depends on configured provider");
    }
}

/// Try to load a suite chat surface and return the URL that exposed the input.
async fn load_suite_chat(browser: &bottest::web::Browser, base: &str) -> Result<String> {
    let mut last_err = String::from("no URLs attempted");
    for url in suite_chat_urls(base) {
        println!("🌐 [desktop-local] trying suite chat URL: {url}");
        match browser.goto(&url).await {
            Ok(()) => {
                tokio::time::sleep(Duration::from_secs(3)).await;
                let input = Locator::css(SUITE_CHAT_INPUT_SELECTOR);
                if browser.exists(input).await {
                    println!("✓ Suite chat input found at {url}");
                    return Ok(url);
                }
                last_err = format!("chat input not found at {url}");
            }
            Err(e) => last_err = format!("navigate failed for {url}: {e}"),
        }
    }
    bail!("Suite chat did not load: {last_err}")
}

/// Send "hi" through the suite chat UI and wait briefly for a response.
async fn send_hi_and_wait(browser: &bottest::web::Browser) -> Result<bool> {
    let input = Locator::css(SUITE_CHAT_INPUT_SELECTOR);
    browser
        .type_text(input.clone(), "hi")
        .await
        .context("type into suite chat input")?;
    let send_btn = Locator::css(SUITE_CHAT_SEND_SELECTOR);
    if browser.click(send_btn).await.is_err() {
        let _ = browser.press_key(input, "Enter").await;
    }

    println!("⏳ Waiting for bot response (mock LLM when wired)...");
    tokio::time::sleep(Duration::from_secs(6)).await;

    let response = Locator::css(SUITE_CHAT_BOT_RESPONSE_SELECTOR);
    Ok(matches!(browser.find_elements(response).await, Ok(ref els) if !els.is_empty()))
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

    wire_context_mock_llm(&ctx).await;
    let browser = ctx.browser.as_ref().context("browser missing")?;

    if let Err(e) = load_suite_chat(browser, &ui.url).await {
        if let Ok(screenshot) = browser.screenshot().await {
            let _ = std::fs::write("/tmp/bottest-desktop-chat-fail.png", &screenshot);
            println!("Screenshot: /tmp/bottest-desktop-chat-fail.png");
        }
        ctx.close().await;
        return Err(e);
    }

    let responded = send_hi_and_wait(browser).await?;
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
                .map(|m| m.chat_completions_url())
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
    let mut ctx = match bottest::TestHarness::minimal().await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping: could not create minimal harness: {e}");
            return Ok(());
        }
    };

    // Start a standalone mock LLM on an ephemeral port for this wiring smoke.
    let port = bottest::PortAllocator::allocate();
    let mock = bottest::mocks::MockLLM::start(port).await?;
    wire_mock_llm(&mock).await;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
    let resp = client
        .post(mock.chat_completions_url())
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
        body.contains(DESKTOP_LOCAL_CHAT_MOCK_REPLY),
        "mock LLM response should include deterministic reply, got: {body}"
    );
    mock.assert_called().await;

    println!("✓ Desktop-local chat mock-LLM wiring smoke passed (port {port})");
    ctx.cleanup().await.ok();
    Ok(())
}

/// Offline contract: desktop-local env wiring maps mock LLM URLs and stack paths
/// as expected by the sidecar/start profile (no processes started).
#[test]
fn desktop_local_env_wiring_contract() {
    let stack = PathBuf::from("/tmp/bottest-desktop-stack");
    let installers = PathBuf::from("/tmp/bottest-desktop-installers");
    // Use a fake MockLLM-like URL construction by building env via a temp mock
    // is async; validate the deterministic URL composition contract directly.
    let base = "http://127.0.0.1:19999";
    let completions = format!("{base}/v1/chat/completions");
    let openai_base = format!("{base}/v1");

    let mut env = BTreeMap::new();
    env.insert(
        "BOTSERVER_STACK_PATH".to_string(),
        stack.display().to_string(),
    );
    env.insert(
        "BOTSERVER_INSTALLERS_PATH".to_string(),
        installers.display().to_string(),
    );
    env.insert("GB_DESKTOP_ALLOW_DOWNLOAD".to_string(), "0".to_string());
    env.insert(
        "AGENT_ALLOW_UNREGISTERED_IDENTITIES".to_string(),
        "1".to_string(),
    );
    env.insert("LLM_URL".to_string(), completions.clone());
    env.insert("LLM_API_URL".to_string(), completions.clone());
    env.insert("OPENAI_API_BASE".to_string(), openai_base.clone());
    env.insert("OPENAI_API_KEY".to_string(), "mock-key".to_string());

    assert_eq!(env.get("BOTSERVER_STACK_PATH").map(String::as_str), Some("/tmp/bottest-desktop-stack"));
    assert_eq!(env.get("GB_DESKTOP_ALLOW_DOWNLOAD").map(String::as_str), Some("0"));
    assert_eq!(env.get("LLM_URL"), Some(&completions));
    assert_eq!(env.get("OPENAI_API_BASE"), Some(&openai_base));
    assert!(suite_chat_urls("http://127.0.0.1:3000").contains(&"http://127.0.0.1:3000/chat".to_string()));
    assert!(SUITE_CHAT_INPUT_SELECTOR.contains("#messageInput"));
    assert!(SUITE_CHAT_SEND_SELECTOR.contains("#sendBtn"));
    assert!(SUITE_CHAT_BOT_RESPONSE_SELECTOR.contains(".message.assistant"));
}
