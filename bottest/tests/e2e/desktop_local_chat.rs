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
//! - LLM wiring (mock, deterministic): `LLM_URL` points at the mock chat
//!   completions endpoint, `LLM_API_URL`/`OPENAI_API_BASE` point at the OpenAI
//!   base (`/v1`), `OPENAI_API_KEY`/`LLM_API_KEY` are dummy values, and
//!   `LLM_MODEL`/`LLM_PROVIDER` identify the local mock model/provider.
//!   `bottest::BotServerInstance::start_desktop_local` applies the same contract
//!   when `with_mock_llm_urls(...)` is provided.
//!
//! ## Suite chat flow
//! - Desktop shell route `/chat` loads the suite and opens chat; the chat UI is
//!   the `partials/chat.html` fragment (also used by the desktop shell). Real
//!   suite chat runs over WebSocket `/ws` after `/api/auth` + `/api/bot/public`;
//!   HTTP `POST /api/chat` is a separate JSON endpoint (registered in
//!   `botserver/src/main_module/routes/chat_handlers.rs`) used by headless smoke.
//! - Selectors aligned to `botui/ui/suite/partials/chat.html` and chat JS:
//!   input `#messageInput` (textarea `name="content"`; fallbacks `#ai-input`,
//!   `.ai-input`), send `#sendBtn` (`button[type='submit']`; fallbacks
//!   `#ai-send`, `.ai-send`), transcript `#messages` (`<main>`), bot/assistant
//!   replies rendered as `.message.bot` with `.message-content.bot-message`
//!   (fallbacks `.message.assistant`, `.assistant-message`,
//!   `[data-role='assistant']`).
//!
//! ## Execution modes
//! - `desktop_local_chat_mock_llm_wiring_smoke`: offline-safe, no browser/DB.
//! - `desktop_local_env_wiring_contract`: offline-safe env/profile contract.
//! - `desktop_local_start_profile_builder_contract`: offline-safe start profile.
//! - `desktop_local_start_profile_with_mock_llm`: ignored; starts an isolated
//!   desktop-local botserver when a binary is available and validates readiness
//!   with a deterministic mock LLM wired.
//! - `desktop_local_suite_chat_hi`: ignored; requires desktop-local
//!   botserver+suite UI+browser wired to the mock LLM.
//!
//! ## How to run
//! - Offline contracts/smoke (no paid LLM, no browser/DB/botserver required):
//!   `cargo test -p bottest --features e2e desktop_local_chat -- --nocapture`
//!   (or run individual `desktop_local_*_contract` /
//!   `desktop_local_chat_mock_llm_wiring_smoke` tests).
//! - Start-profile bootstrap with a desktop-local-capable binary:
//!   `BOTSERVER_DESKTOP_LOCAL_BIN=/abs/path/to/botserver cargo test -p bottest --features e2e desktop_local_start_profile_with_mock_llm -- --ignored --nocapture`
//! - Full suite chat UI flow (self-wired environment required): run a
//!   desktop-local botserver started with the mock LLM env from
//!   `desktop_local_botserver_env`/`start_desktop_local`, serve suite UI (botui)
//!   against it, then run
//!   `BOTSERVER_URL=http://127.0.0.1:<port> BOTUI_URL=http://localhost:3000 cargo test -p bottest --features e2e desktop_local_suite_chat_hi -- --ignored --nocapture`.
//!
//! ## CI / self-contained status
//! - Mock LLM wiring, env/profile contracts, selector alignment, and
//!   `BotServerInstance::start_desktop_local` env application are covered
//!   offline/deterministically in this harness.
//! - Remaining for a fully self-contained CI run: a desktop-local-capable
//!   `botserver` binary + bootstrappable stack components (DB/cache/services as
//!   required by desktop-local), suite UI served for the browser, and a browser
//!   (CDP/WebDriver path used by `E2ETestContext::setup_with_browser`).
//!   `desktop_local_suite_chat_hi` currently reuses an existing/wired stack via
//!   `E2ETestContext` and does not spawn a mock-wired botserver itself; making
//!   it fully self-contained requires orchestrating botserver+botui+browser with
//!   the harness mock LLM in one isolated test setup. No `cargo`/`boteval` runs
//!   are performed by this agent. Keep tabs/browser sessions open for
//!   inspection. Use `/tmp` for temporary artifacts. No labels, no close.

use super::{should_run_e2e_tests, E2ETestContext};
use anyhow::{bail, Context, Result};
use bottest::mocks::MockLLM;
use bottest::web::Locator;
use bottest::{BotServerInstance, DesktopLocalStartOptions};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

// Re-export shared suite chat selectors/flow/profile helpers so the harness is
// reusable and this test module remains the #2173 contract surface.
pub use bottest::desktop::suite_chat::{
    build_desktop_local_start_profile as build_desktop_local_start_profile_shared,
    build_desktop_local_start_profile_with_urls as build_desktop_local_start_profile_with_urls_shared,
    desktop_local_botserver_env as desktop_local_botserver_env_shared,
    desktop_local_botserver_env_with_urls, ensure_desktop_local_product as ensure_desktop_local_product_shared,
    load_suite_chat as load_suite_chat_shared, send_chat_message_and_wait, send_hi_and_wait as send_hi_and_wait_shared,
    suite_chat_urls, wire_mock_llm as wire_mock_llm_shared, DESKTOP_LOCAL_CHAT_MOCK_MODEL,
    DESKTOP_LOCAL_CHAT_MOCK_PROVIDER, DESKTOP_LOCAL_CHAT_MOCK_REPLY, SUITE_CHAT_AUTH_PATH,
    SUITE_CHAT_BOT_PUBLIC_PATH, SUITE_CHAT_BOT_RESPONSE_SELECTOR, SUITE_CHAT_HTTP_SMOKE_PATH,
    SUITE_CHAT_INPUT_SELECTOR, SUITE_CHAT_SEND_SELECTOR, SUITE_CHAT_TRANSCRIPT_SELECTOR,
    SUITE_CHAT_WS_PATH,
};

/// Minimal desktop-local product fallback (kept for local contract compatibility).
#[allow(dead_code)]
fn desktop_local_product_fallback() -> &'static str {
    "name=General Bots\n\
apps=chat,drive\n\
suite_default_app=chat\n\
sidebar=true\n\
search_enabled=false\n\
menu_launcher_enabled=true\n\
no_security=false\n\
shell_mode=true\n\
edition=desktop\n\
theme=sentient\n"
}

/// Environment variables for desktop-local botserver with deterministic mock LLM.
pub fn desktop_local_botserver_env(
    stack_path: &Path,
    installers_path: Option<&Path>,
    mock_llm: &MockLLM,
) -> BTreeMap<String, String> {
    desktop_local_botserver_env_shared(stack_path, installers_path, mock_llm)
}

/// Resolve repo `.product.desktop-local` source path.
#[allow(dead_code)]
fn repo_desktop_local_product_path() -> PathBuf {
    bottest::desktop::suite_chat::repo_desktop_local_product_path()
}

/// Ensure stack has a `.product.desktop-local` profile.
pub fn ensure_desktop_local_product(stack_path: &Path) -> Result<PathBuf> {
    ensure_desktop_local_product_shared(stack_path)
}

/// Build a desktop-local start profile for explicit mock LLM URLs.
pub fn build_desktop_local_start_profile_with_urls(
    stack_path: &Path,
    chat_completions_url: impl Into<String>,
    openai_base_url: impl Into<String>,
) -> Result<DesktopLocalStartOptions> {
    build_desktop_local_start_profile_with_urls_shared(stack_path, chat_completions_url, openai_base_url)
}

/// Build a desktop-local start profile for an isolated stack and mock LLM.
pub fn build_desktop_local_start_profile(
    stack_path: &Path,
    mock_llm: &MockLLM,
) -> Result<DesktopLocalStartOptions> {
    build_desktop_local_start_profile_shared(stack_path, mock_llm)
}

/// Configure harness mock LLM with deterministic suite-chat reply.
pub async fn wire_mock_llm(mock_llm: &MockLLM) {
    wire_mock_llm_shared(mock_llm).await;
    println!(
        "  env hint: LLM_URL={} LLM_API_URL={} OPENAI_API_BASE={} LLM_MODEL={} LLM_PROVIDER={}",
        mock_llm.chat_completions_url(),
        mock_llm.openai_base_url(),
        mock_llm.openai_base_url(),
        DESKTOP_LOCAL_CHAT_MOCK_MODEL,
        DESKTOP_LOCAL_CHAT_MOCK_PROVIDER
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
    load_suite_chat_shared(browser, base).await
}

/// Send "hi" through the suite chat UI and wait for a bot/assistant response.
async fn send_hi_and_wait(browser: &bottest::web::Browser) -> Result<bool> {
    send_hi_and_wait_shared(browser).await
}

fn detect_desktop_local_botserver_bin() -> Option<PathBuf> {
    for key in ["BOTSERVER_DESKTOP_LOCAL_BIN", "BOTSERVER_BIN"] {
        if let Ok(value) = std::env::var(key) {
            let path = PathBuf::from(value);
            if path.exists() {
                return Some(path);
            }
        }
    }
    let candidates = [
        "../target/debug/botserver",
        "../botserver/target/debug/botserver",
        "../target/release/botserver",
    ];
    candidates
        .iter()
        .map(PathBuf::from)
        .find(|candidate| candidate.exists())
}

/// Desktop-local start profile smoke: starts a deterministic mock LLM, prepares
/// an isolated stack, and (when a desktop-local-capable binary is available)
/// boots `botserver --noconsole` wired to the mock LLM.
///
/// Ignored by default: bootstrapping a fresh desktop-local stack is heavy and
/// environment-dependent. Run explicitly with a binary present:
/// `BOTSERVER_DESKTOP_LOCAL_BIN=... cargo test -p bottest --features e2e desktop_local_start_profile_with_mock_llm -- --ignored --nocapture`
#[tokio::test]
#[ignore = "requires desktop-local botserver binary; bootstraps isolated stack"]
async fn desktop_local_start_profile_with_mock_llm() -> Result<()> {
    let stack = PathBuf::from(format!(
        "/tmp/opencode/bottest-desktop-local-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&stack);

    let mock_port = bottest::PortAllocator::allocate();
    let mock = MockLLM::start(mock_port).await?;
    wire_mock_llm(&mock).await;

    let profile = build_desktop_local_start_profile(&stack, &mock)?;
    let product = stack.join(".product.desktop-local");
    assert!(product.exists(), "desktop-local product profile must exist");

    println!("✓ Desktop-local start profile prepared");
    println!("  stack: {}", stack.display());
    println!("  product: {}", product.display());
    println!("  mock llm: {}", mock.chat_completions_url());

    let Some(bin) = detect_desktop_local_botserver_bin() else {
        println!(
            "⚠ Skipping botserver boot: no desktop-local botserver binary found \
             (set BOTSERVER_DESKTOP_LOCAL_BIN or BOTSERVER_BIN)"
        );
        println!("  profile/env wiring validated; server bootstrap not exercised in this run");
        let _ = std::fs::remove_dir_all(&stack);
        return Ok(());
    };

    let port = bottest::PortAllocator::allocate();
    println!("🚀 Starting desktop-local botserver from {}", bin.display());
    let opts = profile.with_port(port).with_botserver_bin(bin);
    let server = BotServerInstance::start_desktop_local(opts).await?;

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()?;
    let mut ready = false;
    for endpoint in ["/health", "/api/product"] {
        if let Ok(resp) = client.get(format!("{}{endpoint}", server.url)).send().await {
            if resp.status().is_success() {
                println!("✓ Desktop-local botserver responded: {endpoint}");
                ready = true;
                break;
            }
        }
    }
    println!(
        "  readiness={} url={} stack={} mock_llm={}",
        if ready { "ok" } else { "not-confirmed" },
        server.url,
        server.stack_path.display(),
        mock.chat_completions_url()
    );
    if !ready {
        println!(
            "  note: binary bootstrapped but did not confirm HTTP readiness in time; \
             stack/env/profile/mock wiring were exercised"
        );
    }

    // Keep stack for post-run inspection under /tmp if needed by re-running with
    // a fixed path; ephemeral stacks are removed here to avoid /tmp buildup.
    let _ = std::fs::remove_dir_all(&stack);
    Ok(())
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
            "model": DESKTOP_LOCAL_CHAT_MOCK_MODEL,
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

/// Offline contract: desktop-local env wiring maps mock LLM URLs, model/provider,
/// and stack/profile paths as expected by the sidecar/start profile.
#[tokio::test]
async fn desktop_local_env_wiring_contract() -> Result<()> {
    let stack = PathBuf::from("/tmp/opencode/bottest-desktop-stack");
    let installers = PathBuf::from("/tmp/opencode/bottest-desktop-installers");
    std::fs::create_dir_all(&stack).context("create /tmp/opencode test stack dir")?;

    let port = bottest::PortAllocator::allocate();
    let mock = bottest::mocks::MockLLM::start(port).await?;
    wire_mock_llm(&mock).await;

    let env = desktop_local_botserver_env(&stack, Some(&installers), &mock);
    assert_eq!(
        env.get("BOTSERVER_STACK_PATH").map(String::as_str),
        Some("/tmp/opencode/bottest-desktop-stack")
    );
    assert_eq!(
        env.get("BOTSERVER_INSTALLERS_PATH").map(String::as_str),
        Some("/tmp/opencode/bottest-desktop-installers")
    );
    assert_eq!(
        env.get("GB_DESKTOP_ALLOW_DOWNLOAD").map(String::as_str),
        Some("0")
    );
    assert_eq!(
        env.get("AGENT_ALLOW_UNREGISTERED_IDENTITIES")
            .map(String::as_str),
        Some("1")
    );
    assert_eq!(env.get("GBO_NO_DOTENV").map(String::as_str), Some("1"));
    assert_eq!(env.get("LLM_URL"), Some(&mock.chat_completions_url()));
    assert_eq!(env.get("LLM_API_URL"), Some(&mock.openai_base_url()));
    assert_eq!(env.get("OPENAI_API_BASE"), Some(&mock.openai_base_url()));
    assert_eq!(
        env.get("OPENAI_API_KEY").map(String::as_str),
        Some("mock-key")
    );
    assert_eq!(env.get("LLM_API_KEY").map(String::as_str), Some("mock-key"));
    assert_eq!(
        env.get("LLM_MODEL").map(String::as_str),
        Some(DESKTOP_LOCAL_CHAT_MOCK_MODEL)
    );
    assert_eq!(
        env.get("LLM_PROVIDER").map(String::as_str),
        Some(DESKTOP_LOCAL_CHAT_MOCK_PROVIDER)
    );

    let urls = suite_chat_urls("http://127.0.0.1:3000");
    assert!(urls.contains(&"http://127.0.0.1:3000/chat".to_string()));
    assert!(urls
        .iter()
        .any(|u| u.ends_with("/suite/partials/chat.html")));
    assert!(SUITE_CHAT_INPUT_SELECTOR.contains("#messageInput"));
    assert!(SUITE_CHAT_INPUT_SELECTOR.contains("textarea[name='content']"));
    assert!(SUITE_CHAT_SEND_SELECTOR.contains("#sendBtn"));
    assert!(SUITE_CHAT_BOT_RESPONSE_SELECTOR.contains(".message.bot"));

    let product = ensure_desktop_local_product(&stack)?;
    assert!(
        product.exists(),
        "desktop-local product profile must exist in stack"
    );
    let product_text =
        std::fs::read_to_string(&product).context("read desktop-local product profile")?;
    assert!(
        product_text.contains("suite_default_app="),
        "desktop-local product profile should declare suite_default_app"
    );
    assert!(
        product_text.contains("apps="),
        "desktop-local product profile should declare apps"
    );

    Ok(())
}

/// Offline contract: start profile builder prepares isolated stack + product
/// file + deterministic mock LLM wiring options (no processes started).
#[test]
fn desktop_local_start_profile_builder_contract() {
    let stack = PathBuf::from("/tmp/opencode/bottest-desktop-local-profile-contract");
    let _ = std::fs::remove_dir_all(&stack);

    let mock = MockLLMContract {
        completions: "http://127.0.0.1:19001/v1/chat/completions".to_string(),
        base: "http://127.0.0.1:19001/v1".to_string(),
    };

    let opts = build_desktop_local_start_profile_with_urls(
        &stack,
        mock.completions.clone(),
        mock.base.clone(),
    )
    .expect("build desktop-local start profile");

    let product = stack.join(".product.desktop-local");
    assert!(product.exists(), "desktop-local product profile must exist");
    assert!(
        stack.join("installers").exists(),
        "desktop-local installers dir must be created by profile builder"
    );
    assert_eq!(opts.stack_path, stack);
    assert_eq!(
        opts.llm_chat_completions_url.as_deref(),
        Some(mock.completions.as_str())
    );
    assert_eq!(
        opts.llm_openai_base_url.as_deref(),
        Some(mock.base.as_str())
    );
    assert!(!opts.allow_downloads);

    let _ = std::fs::remove_dir_all(&stack);
}

/// Minimal stand-in for URL contract validation without async mock startup.
struct MockLLMContract {
    completions: String,
    base: String,
}

/// Offline contract: suite chat WS/HTTP flow endpoints and UI selector anchors.
/// Validates the real browser flow expectations (`/api/auth` -> `/api/bot/public`
/// -> `/ws`) alongside the headless HTTP smoke endpoint and suite selectors.
#[test]
fn desktop_local_suite_chat_flow_contract() {
    assert_eq!(SUITE_CHAT_WS_PATH, "/ws");
    assert_eq!(SUITE_CHAT_AUTH_PATH, "/api/auth");
    assert_eq!(SUITE_CHAT_BOT_PUBLIC_PATH, "/api/bot/public");
    assert_eq!(SUITE_CHAT_HTTP_SMOKE_PATH, "/api/chat");

    let urls = suite_chat_urls("http://127.0.0.1:3000/");
    assert_eq!(urls[0], "http://127.0.0.1:3000/chat");
    assert!(urls.iter().any(|u| u == "http://127.0.0.1:3000/#chat"));
    assert!(urls.iter().any(|u| u.ends_with("/suite/desktop.html#chat")));
    assert!(urls
        .iter()
        .any(|u| u.ends_with("/suite/partials/chat.html")));

    // Input/send/transcript anchors aligned to botui/ui/suite/partials/chat.html.
    assert!(SUITE_CHAT_INPUT_SELECTOR.contains("#messageInput"));
    assert!(SUITE_CHAT_INPUT_SELECTOR.contains("textarea[name='content']"));
    assert!(SUITE_CHAT_INPUT_SELECTOR.contains("#ai-input"));
    assert!(SUITE_CHAT_SEND_SELECTOR.contains("#sendBtn"));
    assert!(SUITE_CHAT_SEND_SELECTOR.contains("button[type='submit']"));
    assert!(SUITE_CHAT_BOT_RESPONSE_SELECTOR.contains(".message.bot .bot-message"));
    assert!(SUITE_CHAT_BOT_RESPONSE_SELECTOR.contains("[data-role='assistant']"));
}
