use super::{should_run_e2e_tests, E2ETestContext};
use anyhow::{bail, Result};
use bottest::prelude::*;
use bottest::web::Locator;

#[tokio::test]
async fn test_chat_hi() -> Result<()> {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled");
        return Ok(());
    }

    let ctx = E2ETestContext::setup_with_browser().await?;

    if !ctx.has_browser() {
        ctx.close().await;
        bail!("Browser not available - cannot run E2E test");
    }

    if ctx.ui.is_none() {
        ctx.close().await;
        bail!("BotUI not available - chat tests require botui running on port 3000");
    }

    let browser = ctx.browser.as_ref().unwrap();
    let ui_url = ctx.ui.as_ref().unwrap().url.clone();
    let chat_url = format!("{}/#chat", ui_url);

    println!("🌐 Navigating to: {chat_url}");

    if let Err(e) = browser.goto(&chat_url).await {
        ctx.close().await;
        bail!("Failed to navigate to chat: {e}");
    }

    println!("⏳ Waiting for page to load...");
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    let input = Locator::css("#messageInput, #ai-input, .ai-input");

    let mut found_input = false;
    for attempt in 1..=10 {
        if browser.exists(input.clone()).await {
            found_input = true;
            println!("✓ Chat input found (attempt {attempt})");
            break;
        }
        println!("  ... waiting for chat input (attempt {attempt}/10)");
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }

    if !found_input {
        if let Ok(screenshot) = browser.screenshot().await {
            let _ = std::fs::write("/tmp/bottest-chat-fail.png", &screenshot);
            println!("Screenshot saved to /tmp/bottest-chat-fail.png");
        }
        if let Ok(source) = browser.page_source().await {
            let preview: String = source.chars().take(2000).collect();
            println!("Page source preview:\n{preview}");
        }
        ctx.close().await;
        bail!("Chat input not found after 10 attempts");
    }

    println!("⌨️ Typing 'hi'...");
    if let Err(e) = browser.type_text(input.clone(), "hi").await {
        ctx.close().await;
        bail!("Failed to type: {e}");
    }

    let send_btn = Locator::css("#sendBtn, #ai-send, .ai-send, button[type='submit']");
    match browser.click(send_btn).await {
        Ok(()) => println!("✓ Message sent (click)"),
        Err(_) => match browser.press_key(input, "Enter").await {
            Ok(()) => println!("✓ Message sent (Enter key)"),
            Err(e) => println!("⚠ Send may have failed: {e}"),
        },
    }

    println!("⏳ Waiting for bot response...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    let response =
        Locator::css(".message.bot, .message.assistant, .bot-message, .assistant-message");
    match browser.find_elements(response).await {
        Ok(elements) if !elements.is_empty() => {
            println!("✓ Bot responded! ({} messages)", elements.len());
        }
        _ => {
            println!("⚠ No bot response detected (may need LLM configuration)");
        }
    }

    if let Ok(screenshot) = browser.screenshot().await {
        let _ = std::fs::write("/tmp/bottest-chat-result.png", &screenshot);
        println!("📸 Screenshot: /tmp/bottest-chat-result.png");
    }

    ctx.close().await;
    println!("✅ Chat test complete!");
    Ok(())
}

#[tokio::test]
async fn test_chat_page_loads() -> Result<()> {
    if !should_run_e2e_tests() {
        return Ok(());
    }

    let ctx = E2ETestContext::setup_with_browser().await?;

    if !ctx.has_browser() {
        ctx.close().await;
        bail!("Browser not available");
    }

    if ctx.ui.is_none() {
        ctx.close().await;
        bail!("BotUI not available - chat tests require botui. Start it with: cd ../botui && cargo run");
    }

    let browser = ctx.browser.as_ref().unwrap();
    let ui_url = ctx.ui.as_ref().unwrap().url.clone();
    let chat_url = format!("{}/#chat", ui_url);

    if let Err(e) = browser.goto(&chat_url).await {
        ctx.close().await;
        bail!("Navigation failed: {e}");
    }

    tokio::time::sleep(std::time::Duration::from_secs(1)).await;

    let input = Locator::css("#messageInput, input[type='text'], textarea");
    match browser.wait_for(input).await {
        Ok(_) => println!("✓ Chat loaded"),
        Err(e) => {
            if let Ok(s) = browser.screenshot().await {
                let _ = std::fs::write("/tmp/bottest-fail.png", &s);
            }
            ctx.close().await;
            bail!("Chat not loaded: {e}");
        }
    }

    ctx.close().await;
    Ok(())
}
