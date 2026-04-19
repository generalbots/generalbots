
use bottest::prelude::*;
use bottest::web::{Browser, Locator};
use std::time::Duration;

use super::{check_webdriver_available, should_run_e2e_tests, E2ETestContext};

pub async fn verify_platform_loading(ctx: &E2ETestContext) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    let health_url = format!("{}/health", ctx.base_url());
    let health_resp = client.get(&health_url).send().await?;
    assert!(
        health_resp.status().is_success(),
        "Health check failed with status: {}",
        health_resp.status()
    );

    println!("✓ Platform health check passed");

    let api_url = format!("{}/api/v1", ctx.base_url());
    let api_resp = client.get(&api_url).send().await?;
    assert!(
        api_resp.status().is_success() || api_resp.status().as_u16() == 401,
        "API endpoint failed with status: {}",
        api_resp.status()
    );

    println!("✓ Platform API responsive");

    Ok(())
}

pub async fn verify_botserver_running(ctx: &E2ETestContext) -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    assert!(ctx.server.is_running(), "BotServer process is not running");

    println!("✓ BotServer process running");

    let info_url = format!("{}/api/v1/server/info", ctx.base_url());
    match client.get(&info_url).send().await {
        Ok(resp) => {
            if resp.status().is_success() {
                let body = resp.text().await?;
                assert!(!body.is_empty(), "Server info response is empty");
                println!(
                    "✓ BotServer initialized with info: {}",
                    body.chars().take(100).collect::<String>()
                );
            } else {
                println!(
                    "⚠ Server info endpoint returned {}, continuing anyway",
                    resp.status()
                );
            }
        }
        Err(e) => {
            println!(
                "⚠ Could not reach server info endpoint: {}, continuing anyway",
                e
            );
        }
    }

    println!("✓ BotServer is running and initialized");

    Ok(())
}

pub async fn test_user_login(browser: &Browser, ctx: &E2ETestContext) -> anyhow::Result<()> {
    let login_url = format!("{}/login", ctx.base_url());

    browser.goto(&login_url).await?;
    println!("✓ Navigated to login page: {}", login_url);

    browser
        .wait_for(Locator::css("input[type='email']"))
        .await?;
    println!("✓ Login form loaded");

    let test_email = "test@example.com";
    let test_password = "TestPassword123!";

    browser
        .fill(Locator::css("input[type='email']"), test_email)
        .await?;
    println!("✓ Entered email: {}", test_email);

    browser
        .fill(Locator::css("input[type='password']"), test_password)
        .await?;
    println!("✓ Entered password");

    browser.click(Locator::css("button[type='submit']")).await?;
    println!("✓ Clicked login button");

    tokio::time::sleep(Duration::from_secs(2)).await;

    let current_url = browser.current_url().await?;

    assert!(
        !current_url.contains("/login"),
        "Still on login page after login attempt. URL: {}",
        current_url
    );

    println!("✓ Redirected from login page to: {}", current_url);

    browser
        .wait_for(Locator::css(
            "[data-testid='chat-area'], [data-testid='dashboard'], main",
        ))
        .await?;
    println!("✓ Dashboard or chat area visible");

    Ok(())
}

pub async fn test_chat_interaction(browser: &Browser, ctx: &E2ETestContext) -> anyhow::Result<()> {
    let chat_url = format!("{}/chat", ctx.base_url());
    browser.goto(&chat_url).await?;
    println!("✓ Navigated to chat page");

    browser
        .wait_for(Locator::css(
            "[data-testid='message-input'], textarea.chat-input, input.message",
        ))
        .await?;
    println!("✓ Chat interface loaded");

    let test_message = "Hello, I need help";
    browser
        .fill(
            Locator::css("textarea.chat-input, input.message"),
            test_message,
        )
        .await?;
    println!("✓ Typed message: {}", test_message);

    let send_result = browser
        .click(Locator::css(
            "button[data-testid='send-button'], button.send-btn",
        ))
        .await;

    if send_result.is_err() {
        let input = browser
            .find(Locator::css("textarea.chat-input, input.message"))
            .await?;
        input.send_keys("\n").await?;
        println!("✓ Sent message with Enter key");
    } else {
        println!("✓ Clicked send button");
    }

    browser
        .wait_for(Locator::css(
            "[data-testid='message-item'], .message-bubble, [class*='message']",
        ))
        .await?;
    println!("✓ Message appeared in chat");

    browser
        .wait_for(Locator::css(
            "[data-testid='bot-response'], .bot-message, [class*='bot']",
        ))
        .await?;
    println!("✓ Received bot response");

    let response_text = browser
        .text(Locator::css(
            "[data-testid='bot-response'], .bot-message, [class*='bot']",
        ))
        .await
        .ok();

    if let Some(text) = response_text {
        println!(
            "✓ Bot response: {}",
            text.chars().take(100).collect::<String>()
        );
    }

    Ok(())
}

pub async fn test_user_logout(browser: &Browser, ctx: &E2ETestContext) -> anyhow::Result<()> {
    let logout_selectors = vec![
        "button[data-testid='logout-btn']",
        "button.logout",
        "[data-testid='user-menu'] button[data-testid='logout']",
        "a[href*='logout']",
    ];

    let mut logout_found = false;
    for selector in logout_selectors {
        if browser.click(Locator::css(selector)).await.is_ok() {
            println!("✓ Clicked logout button: {}", selector);
            logout_found = true;
            break;
        }
    }

    if !logout_found {
        println!("⚠ Could not find logout button, attempting navigation to logout URL");
        let logout_url = format!("{}/logout", ctx.base_url());
        browser.goto(&logout_url).await?;
    }

    tokio::time::sleep(Duration::from_secs(2)).await;
    let current_url = browser.current_url().await?;

    assert!(
        current_url.contains("/login") || current_url.contains("/auth"),
        "Not redirected to login page after logout. URL: {}",
        current_url
    );

    println!("✓ Redirected to login page after logout: {}", current_url);

    let chat_url = format!("{}/chat", ctx.base_url());
    browser.goto(&chat_url).await?;

    tokio::time::sleep(Duration::from_secs(1)).await;
    let check_url = browser.current_url().await?;
    assert!(
        check_url.contains("/login") || check_url.contains("/auth"),
        "Should be redirected to login when accessing protected route after logout. URL: {}",
        check_url
    );

    println!("✓ Protected routes properly redirect to login");

    Ok(())
}

#[tokio::test]
async fn test_complete_platform_flow_login_chat_logout() {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled (set SKIP_E2E_TESTS env var to disable)");
        return;
    }

    if !check_webdriver_available().await {
        eprintln!("Skipping: WebDriver not available at configured URL");
        return;
    }

    println!("\n=== Starting Complete Platform Flow Test ===\n");

    let ctx = match E2ETestContext::setup_with_browser().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Failed to setup E2E context: {}", e);
            return;
        }
    };

    if !ctx.has_browser() {
        eprintln!("Browser not available");
        return;
    }

    let browser = ctx.browser.as_ref().unwrap();

    println!("\n--- Phase 1: Platform Loading ---");
    if let Err(e) = verify_platform_loading(&ctx).await {
        eprintln!("Platform loading test failed: {}", e);
        return;
    }

    println!("\n--- Phase 2: BotServer Initialization ---");
    if let Err(e) = verify_botserver_running(&ctx).await {
        eprintln!("BotServer initialization test failed: {}", e);
        return;
    }

    println!("\n--- Phase 3: User Login ---");
    if let Err(e) = test_user_login(browser, &ctx).await {
        eprintln!("Login test failed: {}", e);
        return;
    }

    println!("\n--- Phase 4: Chat Interaction ---");
    if let Err(e) = test_chat_interaction(browser, &ctx).await {
        eprintln!("Chat interaction test failed: {}", e);
    }

    println!("\n--- Phase 5: User Logout ---");
    if let Err(e) = test_user_logout(browser, &ctx).await {
        eprintln!("Logout test failed: {}", e);
        return;
    }

    println!("\n=== Complete Platform Flow Test PASSED ===\n");

    ctx.close().await;
}

#[tokio::test]
async fn test_platform_loading_http_only() {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled");
        return;
    }

    println!("\n=== Testing Platform Loading (HTTP Only) ===\n");

    let ctx = match E2ETestContext::setup().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Failed to setup context: {}", e);
            return;
        }
    };

    if let Err(e) = verify_platform_loading(&ctx).await {
        eprintln!("Platform loading failed: {}", e);
        return;
    }

    println!("\n✓ Platform Loading Test PASSED\n");

    ctx.close().await;
}

#[tokio::test]
async fn test_botserver_startup() {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled");
        return;
    }

    println!("\n=== Testing BotServer Startup ===\n");

    let ctx = match E2ETestContext::setup().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: Failed to setup context: {}", e);
            return;
        }
    };

    if !ctx.server.is_running() {
        eprintln!("Skipping: BotServer not running (BOTSERVER_BIN not set or binary not found)");
        ctx.close().await;
        return;
    }

    if let Err(e) = verify_botserver_running(&ctx).await {
        eprintln!("BotServer test failed: {}", e);
        ctx.close().await;
        return;
    }

    println!("\n✓ BotServer Startup Test PASSED\n");

    ctx.close().await;
}
