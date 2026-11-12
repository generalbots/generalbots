//! Reusable suite chat selectors, routes, and flow helpers for desktop-local and
//! e2e harnesses.
//!
//! Selectors and routes are aligned with `botui/ui/suite/partials/chat.html`,
//! `botui/ui/suite/js/chat-init.js`, and desktop-local startup/wiring docs.
//! The browser flow is WebSocket-driven (`/api/auth` + `/api/bot/public` + `/ws`);
//! `POST /api/chat` is a headless JSON smoke path.

use crate::harness::DesktopLocalStartOptions;
use crate::mocks::MockLLM;
use crate::web::browser::Browser;
use crate::web::Locator;
use anyhow::{bail, Context, Result};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// Deterministic reply for offline suite-chat smoke (no paid LLM calls).
pub const DESKTOP_LOCAL_CHAT_MOCK_REPLY: &str = "Hello! (mocked desktop-local reply)";

/// Default model/provider identifiers for the deterministic mock wiring.
pub const DESKTOP_LOCAL_CHAT_MOCK_MODEL: &str = "mock-gpt";
pub const DESKTOP_LOCAL_CHAT_MOCK_PROVIDER: &str = "openai";

/// CSS selector for the suite chat composer input.
pub const SUITE_CHAT_INPUT_SELECTOR: &str =
    "#messageInput, textarea[name='content'], #ai-input, .ai-input";

/// CSS selector for the suite chat send action.
pub const SUITE_CHAT_SEND_SELECTOR: &str =
    "#sendBtn, button[type='submit'], #ai-send, .ai-send";

/// CSS selector(s) indicating a bot/assistant response in the transcript.
pub const SUITE_CHAT_BOT_RESPONSE_SELECTOR: &str = ".message.bot .bot-message, .message.bot, .message.assistant, .assistant-message, .bot-message, [data-role='assistant']";

/// Main transcript container in suite chat partial.
pub const SUITE_CHAT_TRANSCRIPT_SELECTOR: &str = "#messages, .messages-container";

/// Endpoints involved in suite chat (WS flow) and headless smoke.
pub const SUITE_CHAT_WS_PATH: &str = "/ws";
pub const SUITE_CHAT_AUTH_PATH: &str = "/api/auth";
pub const SUITE_CHAT_BOT_PUBLIC_PATH: &str = "/api/bot/public";
pub const SUITE_CHAT_HTTP_SMOKE_PATH: &str = "/api/chat";

/// URL candidates for the suite chat surface. Prefer routed desktop entry points
/// so the shell boots chat + WS modules; include the partial as structural fallback.
#[must_use]
pub fn suite_chat_urls(base: &str) -> Vec<String> {
    let trimmed = base.trim_end_matches('/');
    vec![
        format!("{trimmed}/chat"),
        format!("{trimmed}/#chat"),
        format!("{trimmed}/suite/desktop.html#chat"),
        format!("{trimmed}/suite/desktop.html"),
        format!("{trimmed}/suite/partials/chat.html"),
    ]
}

/// Minimal fallback `.product.desktop-local` content for offline harness stacks.
/// `no_security=true` is preferred for self-contained smoke runs to avoid auth
/// blocking; callers can overwrite the file for stricter scenarios.
fn desktop_local_product_fallback(no_security: bool) -> String {
    let ns = if no_security { "true" } else { "false" };
    format!(
        "name=General Bots\n\
apps=chat,drive\n\
suite_default_app=chat\n\
sidebar=true\n\
search_enabled=false\n\
menu_launcher_enabled=true\n\
no_security={ns}\n\
shell_mode=true\n\
edition=desktop\n\
theme=sentient\n"
    )
}

/// Environment variables for a desktop-local botserver using a deterministic
/// mock OpenAI-compatible LLM and an isolated offline stack layout.
#[must_use]
pub fn desktop_local_botserver_env(
    stack_path: &Path,
    installers_path: Option<&Path>,
    mock_llm: &MockLLM,
) -> BTreeMap<String, String> {
    desktop_local_botserver_env_with_urls(
        stack_path,
        installers_path,
        &mock_llm.chat_completions_url(),
        &mock_llm.openai_base_url(),
    )
}

/// Like [`desktop_local_botserver_env`] but takes explicit mock URLs.
#[must_use]
pub fn desktop_local_botserver_env_with_urls(
    stack_path: &Path,
    installers_path: Option<&Path>,
    chat_completions_url: &str,
    openai_base_url: &str,
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
    env.insert("GB_DESKTOP_ALLOW_DOWNLOAD".to_string(), "0".to_string());
    env.insert(
        "AGENT_ALLOW_UNREGISTERED_IDENTITIES".to_string(),
        "1".to_string(),
    );
    env.insert("GBO_NO_DOTENV".to_string(), "1".to_string());
    env.insert("LLM_URL".to_string(), chat_completions_url.to_string());
    env.insert("LLM_API_URL".to_string(), openai_base_url.to_string());
    env.insert("OPENAI_API_BASE".to_string(), openai_base_url.to_string());
    env.insert("OPENAI_API_KEY".to_string(), "mock-key".to_string());
    env.insert("LLM_API_KEY".to_string(), "mock-key".to_string());
    env.insert(
        "LLM_MODEL".to_string(),
        DESKTOP_LOCAL_CHAT_MOCK_MODEL.to_string(),
    );
    env.insert(
        "LLM_PROVIDER".to_string(),
        DESKTOP_LOCAL_CHAT_MOCK_PROVIDER.to_string(),
    );
    env
}

/// Resolve the `.product.desktop-local` source shipped in this repository.
#[must_use]
pub fn repo_desktop_local_product_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("botserver")
        .join(".product.desktop-local")
}

/// Ensure the stack has a `.product.desktop-local` file (copied from repo when
/// available). Existing profiles are preserved. If `no_security` is requested and
/// a fallback must be written, the fallback uses that value.
pub fn ensure_desktop_local_product(stack_path: &Path) -> Result<PathBuf> {
    ensure_desktop_local_product_with_security(stack_path, false)
}

/// Ensure desktop-local product profile exists, optionally preferring no-security
/// for offline self-contained harness runs when writing a fallback.
pub fn ensure_desktop_local_product_with_security(
    stack_path: &Path,
    no_security_fallback: bool,
) -> Result<PathBuf> {
    std::fs::create_dir_all(stack_path).context("create desktop-local stack dir")?;
    let dst = stack_path.join(".product.desktop-local");
    if dst.exists() {
        return Ok(dst);
    }
    let repo_product = repo_desktop_local_product_path();
    if repo_product.exists() {
        std::fs::copy(&repo_product, &dst).with_context(|| {
            format!(
                "copy desktop-local product profile from {}",
                repo_product.display()
            )
        })?;
    } else {
        std::fs::write(&dst, desktop_local_product_fallback(no_security_fallback))
            .context("write fallback .product.desktop-local")?;
    }
    Ok(dst)
}

/// Build a desktop-local start profile for an isolated stack and explicit mock
/// LLM URLs. Prepares stack/product/installers layout; returns options for
/// [`crate::BotServerInstance::start_desktop_local`] (no processes started).
pub fn build_desktop_local_start_profile_with_urls(
    stack_path: &Path,
    chat_completions_url: impl Into<String>,
    openai_base_url: impl Into<String>,
) -> Result<DesktopLocalStartOptions> {
    ensure_desktop_local_product(stack_path)?;
    let installers_path = stack_path.join("installers");
    std::fs::create_dir_all(&installers_path).context("create desktop-local installers dir")?;
    Ok(DesktopLocalStartOptions::for_stack(stack_path.to_path_buf())
        .with_mock_llm_urls(chat_completions_url, openai_base_url)
        .with_installers_path(installers_path)
        .with_product_source(repo_desktop_local_product_path()))
}

/// Build a desktop-local start profile for an isolated stack and mock LLM.
pub fn build_desktop_local_start_profile(
    stack_path: &Path,
    mock_llm: &MockLLM,
) -> Result<DesktopLocalStartOptions> {
    build_desktop_local_start_profile_with_urls(
        stack_path,
        mock_llm.chat_completions_url(),
        mock_llm.openai_base_url(),
    )
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
    println!("  openai base: {}", mock_llm.openai_base_url());
    println!("  chat completions: {}", mock_llm.chat_completions_url());
}

/// Try to load a suite chat surface and return the URL that exposed the input.
pub async fn load_suite_chat(browser: &Browser, base: &str) -> Result<String> {
    let mut last_err = String::from("no URLs attempted");
    for url in suite_chat_urls(base) {
        println!("🌐 [suite-chat] trying URL: {url}");
        match browser.goto(&url).await {
            Ok(()) => {
                for attempt in 1..=10 {
                    tokio::time::sleep(Duration::from_secs(2)).await;
                    let input = Locator::css(SUITE_CHAT_INPUT_SELECTOR);
                    if browser.exists(input).await {
                        println!("✓ Suite chat input found at {url} (poll {attempt}/10)");
                        return Ok(url);
                    }
                }
                last_err = format!("chat input/composer not found at {url}");
            }
            Err(e) => last_err = format!("navigate failed for {url}: {e}"),
        }
    }
    bail!("Suite chat did not load (expected #messageInput/textarea[name='content']): {last_err}")
}

/// Send a message through the suite chat UI and wait for a bot/assistant response.
pub async fn send_chat_message_and_wait(
    browser: &Browser,
    message: &str,
    timeout_polls: usize,
) -> Result<bool> {
    let input = Locator::css(SUITE_CHAT_INPUT_SELECTOR);
    browser
        .type_text(input.clone(), message)
        .await
        .context("type into suite chat input")?;
    let send_btn = Locator::css(SUITE_CHAT_SEND_SELECTOR);
    if browser.click(send_btn).await.is_err() {
        // Textarea Enter submits in chat-init.js unless Shift is held.
        let _ = browser.press_key(input, "Enter").await;
    }

    println!("⏳ Waiting for bot response...");
    for attempt in 1..=timeout_polls {
        tokio::time::sleep(Duration::from_secs(2)).await;
        let response = Locator::css(SUITE_CHAT_BOT_RESPONSE_SELECTOR);
        if matches!(browser.find_elements(response).await, Ok(ref els) if !els.is_empty()) {
            println!("✓ Bot response element detected (poll {attempt}/{timeout_polls})");
            return Ok(true);
        }
    }
    Ok(false)
}

/// Convenience: send "hi" and wait for a reply.
pub async fn send_hi_and_wait(browser: &Browser) -> Result<bool> {
    send_chat_message_and_wait(browser, "hi", 12).await
}