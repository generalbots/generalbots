//! Suite chat page selectors and flow helpers (desktop-local / suite UI).
//!
//! Aligned with `botui/ui/suite/partials/chat.html` and chat JS. Re-exported
//! from the desktop harness for a single source of truth.

use anyhow::{bail, Context, Result};
use std::time::Duration;

use crate::web::browser::Browser;
use crate::web::Locator;

use super::Page;

/// Canonical selectors for the suite chat UI.
pub const INPUT: &str = "#messageInput, textarea[name='content'], #ai-input, .ai-input";
pub const SEND: &str = "#sendBtn, button[type='submit'], #ai-send, .ai-send";
pub const TRANSCRIPT: &str = "#messages, .messages-container";
pub const BOT_RESPONSE: &str = ".message.bot .bot-message, .message.bot, .message.assistant, .assistant-message, .bot-message, [data-role='assistant']";
pub const USER_MESSAGE: &str = ".message.user, .user-message, [data-role='user']";
pub const SUGGESTIONS: &str = ".suggestions, .suggestion-btn, [data-suggestions]";

/// Route/endpoint hints for the suite chat flow.
pub const WS_PATH: &str = "/ws";
pub const AUTH_PATH: &str = "/api/auth";
pub const BOT_PUBLIC_PATH: &str = "/api/bot/public";
pub const HTTP_SMOKE_PATH: &str = "/api/chat";

pub struct SuiteChatPage {
    pub base_url: String,
}

impl SuiteChatPage {
    #[must_use]
    pub fn new(base_url: &str) -> Self {
        Self {
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    /// Candidate URLs to reach suite chat (desktop route preferred).
    #[must_use]
    pub fn candidate_urls(&self) -> Vec<String> {
        vec![
            format!("{}/chat", self.base_url),
            format!("{}/#chat", self.base_url),
            format!("{}/suite/desktop.html#chat", self.base_url),
            format!("{}/suite/desktop.html", self.base_url),
            format!("{}/suite/partials/chat.html", self.base_url),
        ]
    }

    #[must_use]
    pub fn input_locator() -> Locator {
        Locator::css(INPUT)
    }

    #[must_use]
    pub fn send_locator() -> Locator {
        Locator::css(SEND)
    }

    #[must_use]
    pub fn bot_response_locator() -> Locator {
        Locator::css(BOT_RESPONSE)
    }

    pub async fn navigate(&self, browser: &Browser) -> Result<String> {
        let mut last_err = String::from("no URLs attempted");
        for url in self.candidate_urls() {
            match browser.goto(&url).await {
                Ok(()) => {
                    for _ in 0..10 {
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        if browser.exists(Self::input_locator()).await {
                            return Ok(url);
                        }
                    }
                    last_err = format!("input not found at {url}");
                }
                Err(e) => last_err = format!("navigate failed for {url}: {e}"),
            }
        }
        bail!("Suite chat did not load: {last_err}")
    }

    pub async fn send_message(&self, browser: &Browser, message: &str) -> Result<()> {
        browser
            .type_text(Self::input_locator(), message)
            .await
            .context("type into suite chat input")?;
        if browser.click(Self::send_locator()).await.is_err() {
            let _ = browser
                .press_key(Self::input_locator(), "Enter")
                .await;
        }
        Ok(())
    }

    pub async fn wait_for_bot_response(&self, browser: &Browser, polls: usize) -> Result<bool> {
        for _ in 0..polls {
            tokio::time::sleep(Duration::from_secs(2)).await;
            if matches!(
                browser.find_elements(Self::bot_response_locator()).await,
                Ok(ref els) if !els.is_empty()
            ) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub async fn send_and_wait(
        &self,
        browser: &Browser,
        message: &str,
        polls: usize,
    ) -> Result<bool> {
        self.send_message(browser, message).await?;
        self.wait_for_bot_response(browser, polls).await
    }
}

#[async_trait::async_trait]
impl Page for SuiteChatPage {
    fn url_pattern(&self) -> &'static str {
        "/chat"
    }

    async fn wait_for_load(&self, browser: &Browser) -> Result<()> {
        browser.wait_for(Self::input_locator()).await?;
        Ok(())
    }
}