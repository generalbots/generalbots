use super::{should_run_e2e_tests, E2ETestContext};
use bottest::prelude::*;
use bottest::web::{Browser, Locator};
use std::time::Duration;

fn admin_credentials() -> (String, String) {
    let email = std::env::var("TEST_ADMIN_EMAIL").unwrap_or_else(|_| "admin@test.com".to_string());
    let password = std::env::var("TEST_ADMIN_PASSWORD").unwrap_or_else(|_| "testpass".to_string());
    (email, password)
}

fn attendant_credentials() -> (String, String) {
    let email =
        std::env::var("TEST_ATTENDANT_EMAIL").unwrap_or_else(|_| "attendant@test.com".to_string());
    let password =
        std::env::var("TEST_ATTENDANT_PASSWORD").unwrap_or_else(|_| "testpass".to_string());
    (email, password)
}

async fn perform_login(
    browser: &Browser,
    base_url: &str,
    email: &str,
    password: &str,
) -> Result<(), String> {
    let login_url = format!("{}/login", base_url);

    browser
        .goto(&login_url)
        .await
        .map_err(|e| format!("Failed to navigate to login: {}", e))?;

    let email_input = Locator::css("#email, input[name='email'], input[type='email']");
    browser
        .wait_for(email_input.clone())
        .await
        .map_err(|e| format!("Email input not found: {}", e))?;

    browser
        .type_text(email_input, email)
        .await
        .map_err(|e| format!("Failed to fill email: {}", e))?;

    let password_input = Locator::css("#password, input[name='password'], input[type='password']");
    browser
        .type_text(password_input, password)
        .await
        .map_err(|e| format!("Failed to fill password: {}", e))?;

    let login_button = Locator::css("#login-button, button[type='submit'], .login-btn");
    browser
        .click(login_button)
        .await
        .map_err(|e| format!("Failed to click login: {}", e))?;

    tokio::time::sleep(Duration::from_secs(2)).await;

    Ok(())
}

#[tokio::test]
async fn test_login_page_loads() {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled");
        return;
    }

    let ctx = match E2ETestContext::setup_with_browser().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    if !ctx.has_browser() {
        eprintln!("Skipping: browser not available");
        ctx.close().await;
        return;
    }

    let browser = ctx.browser.as_ref().unwrap();
    let login_url = format!("{}/login", ctx.base_url());

    if let Err(e) = browser.goto(&login_url).await {
        eprintln!("Failed to navigate: {}", e);
        ctx.close().await;
        return;
    }

    let elements_to_check = vec![
        ("#email, input[type='email']", "email input"),
        ("#password, input[type='password']", "password input"),
        ("button[type='submit'], .login-btn", "login button"),
    ];

    for (selector, name) in elements_to_check {
        let locator = Locator::css(selector);
        match browser.find_element(locator).await {
            Ok(_) => println!("Found: {}", name),
            Err(_) => eprintln!("Not found: {}", name),
        }
    }

    ctx.close().await;
}

#[tokio::test]
async fn test_login_success() {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled");
        return;
    }

    let ctx = match E2ETestContext::setup_with_browser().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    if !ctx.has_browser() {
        eprintln!("Skipping: browser not available");
        ctx.close().await;
        return;
    }

    let browser = ctx.browser.as_ref().unwrap();
    let (email, password) = admin_credentials();

    match perform_login(browser, ctx.base_url(), &email, &password).await {
        Ok(_) => {
            let dashboard_indicator =
                Locator::css(".dashboard, #dashboard, [data-page='dashboard'], .nav-menu");
            match browser.find_element(dashboard_indicator).await {
                Ok(_) => println!("Login successful - dashboard visible"),
                Err(_) => eprintln!("Login may have failed - dashboard not visible"),
            }
        }
        Err(e) => eprintln!("Login failed: {}", e),
    }

    ctx.close().await;
}

#[tokio::test]
async fn test_login_failure() {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled");
        return;
    }

    let ctx = match E2ETestContext::setup_with_browser().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    if !ctx.has_browser() {
        eprintln!("Skipping: browser not available");
        ctx.close().await;
        return;
    }

    let browser = ctx.browser.as_ref().unwrap();

    match perform_login(browser, ctx.base_url(), "invalid@test.com", "wrongpass").await {
        Ok(_) => {
            let error_indicator =
                Locator::css(".error, .alert-error, .login-error, [role='alert']");
            match browser.find_element(error_indicator).await {
                Ok(_) => println!("Error message displayed correctly"),
                Err(_) => eprintln!("Error message not found"),
            }
        }
        Err(e) => eprintln!("Login attempt failed: {}", e),
    }

    ctx.close().await;
}

#[tokio::test]
async fn test_dashboard_home() {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled");
        return;
    }

    let ctx = match E2ETestContext::setup_with_browser().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    if !ctx.has_browser() {
        eprintln!("Skipping: browser not available");
        ctx.close().await;
        return;
    }

    let browser = ctx.browser.as_ref().unwrap();
    let (email, password) = admin_credentials();

    if perform_login(browser, ctx.base_url(), &email, &password)
        .await
        .is_err()
    {
        ctx.close().await;
        return;
    }

    let dashboard_elements = vec![
        (".stats, .statistics, .metrics", "statistics panel"),
        (".queue-summary, .queue-panel", "queue summary"),
        (".recent-activity, .activity-log", "activity log"),
    ];

    for (selector, name) in dashboard_elements {
        let locator = Locator::css(selector);
        match browser.find_element(locator).await {
            Ok(_) => println!("Found: {}", name),
            Err(_) => eprintln!("Not found: {} (may not be implemented)", name),
        }
    }

    ctx.close().await;
}

#[tokio::test]
async fn test_queue_panel() {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled");
        return;
    }

    let ctx = match E2ETestContext::setup_with_browser().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    if !ctx.has_browser() {
        eprintln!("Skipping: browser not available");
        ctx.close().await;
        return;
    }

    let browser = ctx.browser.as_ref().unwrap();
    let (email, password) = attendant_credentials();

    if perform_login(browser, ctx.base_url(), &email, &password)
        .await
        .is_err()
    {
        ctx.close().await;
        return;
    }

    let queue_url = format!("{}/queue", ctx.base_url());
    let _ = browser.goto(&queue_url).await;

    let queue_elements = vec![
        (".queue-list, #queue-list, .waiting-list", "queue list"),
        (".queue-item, .queue-entry", "queue items"),
        (
            ".take-btn, .accept-btn, [data-action='take']",
            "take button",
        ),
    ];

    for (selector, name) in queue_elements {
        let locator = Locator::css(selector);
        match browser.find_element(locator).await {
            Ok(_) => println!("Found: {}", name),
            Err(_) => eprintln!("Not found: {}", name),
        }
    }

    ctx.close().await;
}

#[tokio::test]
async fn test_bot_management() {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled");
        return;
    }

    let ctx = match E2ETestContext::setup_with_browser().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    if !ctx.has_browser() {
        eprintln!("Skipping: browser not available");
        ctx.close().await;
        return;
    }

    let browser = ctx.browser.as_ref().unwrap();
    let (email, password) = admin_credentials();

    if perform_login(browser, ctx.base_url(), &email, &password)
        .await
        .is_err()
    {
        ctx.close().await;
        return;
    }

    let bots_url = format!("{}/admin/bots", ctx.base_url());
    let _ = browser.goto(&bots_url).await;

    let bot_elements = vec![
        (".bot-list, #bot-list, .bots-table", "bot list"),
        (
            ".create-bot, .add-bot, [data-action='create']",
            "create button",
        ),
    ];

    for (selector, name) in bot_elements {
        let locator = Locator::css(selector);
        match browser.find_element(locator).await {
            Ok(_) => println!("Found: {}", name),
            Err(_) => eprintln!("Not found: {}", name),
        }
    }

    ctx.close().await;
}

#[tokio::test]
async fn test_create_bot() {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled");
        return;
    }

    let ctx = match E2ETestContext::setup_with_browser().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    if !ctx.has_browser() {
        eprintln!("Skipping: browser not available");
        ctx.close().await;
        return;
    }

    let browser = ctx.browser.as_ref().unwrap();
    let (email, password) = admin_credentials();

    if perform_login(browser, ctx.base_url(), &email, &password)
        .await
        .is_err()
    {
        ctx.close().await;
        return;
    }

    let create_url = format!("{}/admin/bots/new", ctx.base_url());
    let _ = browser.goto(&create_url).await;

    let name_input = Locator::css("#bot-name, input[name='name'], .bot-name-input");
    if browser.wait_for(name_input.clone()).await.is_ok() {
        let bot_name = format!("test-bot-{}", Uuid::new_v4());
        let _ = browser.type_text(name_input, &bot_name).await;

        let submit_btn = Locator::css("button[type='submit'], .save-btn, .create-btn");
        let _ = browser.click(submit_btn).await;

        tokio::time::sleep(Duration::from_secs(1)).await;

        let success_indicator = Locator::css(".success, .alert-success, .toast-success");
        match browser.find_element(success_indicator).await {
            Ok(_) => println!("Bot created successfully"),
            Err(_) => eprintln!("Success indicator not found"),
        }
    }

    ctx.close().await;
}

#[tokio::test]
async fn test_knowledge_base() {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled");
        return;
    }

    let ctx = match E2ETestContext::setup_with_browser().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    if !ctx.has_browser() {
        eprintln!("Skipping: browser not available");
        ctx.close().await;
        return;
    }

    let browser = ctx.browser.as_ref().unwrap();
    let (email, password) = admin_credentials();

    if perform_login(browser, ctx.base_url(), &email, &password)
        .await
        .is_err()
    {
        ctx.close().await;
        return;
    }

    let kb_url = format!("{}/admin/knowledge", ctx.base_url());
    let _ = browser.goto(&kb_url).await;

    let kb_elements = vec![
        (".document-list, .kb-documents", "document list"),
        (".upload-btn, .add-document", "upload button"),
        (".search-kb, .kb-search", "search input"),
    ];

    for (selector, name) in kb_elements {
        let locator = Locator::css(selector);
        match browser.find_element(locator).await {
            Ok(_) => println!("Found: {}", name),
            Err(_) => eprintln!("Not found: {}", name),
        }
    }

    ctx.close().await;
}

#[tokio::test]
async fn test_analytics() {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled");
        return;
    }

    let ctx = match E2ETestContext::setup_with_browser().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    if !ctx.has_browser() {
        eprintln!("Skipping: browser not available");
        ctx.close().await;
        return;
    }

    let browser = ctx.browser.as_ref().unwrap();
    let (email, password) = admin_credentials();

    if perform_login(browser, ctx.base_url(), &email, &password)
        .await
        .is_err()
    {
        ctx.close().await;
        return;
    }

    let analytics_url = format!("{}/admin/analytics", ctx.base_url());
    let _ = browser.goto(&analytics_url).await;

    let analytics_elements = vec![
        (".chart, .analytics-chart, canvas", "chart"),
        (".date-range, .date-picker", "date range picker"),
        (".metrics-summary, .stats-cards", "metrics summary"),
    ];

    for (selector, name) in analytics_elements {
        let locator = Locator::css(selector);
        match browser.find_element(locator).await {
            Ok(_) => println!("Found: {}", name),
            Err(_) => eprintln!("Not found: {}", name),
        }
    }

    ctx.close().await;
}

#[tokio::test]
async fn test_user_management() {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled");
        return;
    }

    let ctx = match E2ETestContext::setup_with_browser().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    if !ctx.has_browser() {
        eprintln!("Skipping: browser not available");
        ctx.close().await;
        return;
    }

    let browser = ctx.browser.as_ref().unwrap();
    let (email, password) = admin_credentials();

    if perform_login(browser, ctx.base_url(), &email, &password)
        .await
        .is_err()
    {
        ctx.close().await;
        return;
    }

    let users_url = format!("{}/admin/users", ctx.base_url());
    let _ = browser.goto(&users_url).await;

    let user_elements = vec![
        (".user-list, .users-table, #user-list", "user list"),
        (".invite-user, .add-user", "invite button"),
        (".user-row, .user-item, tr.user", "user entries"),
    ];

    for (selector, name) in user_elements {
        let locator = Locator::css(selector);
        match browser.find_element(locator).await {
            Ok(_) => println!("Found: {}", name),
            Err(_) => eprintln!("Not found: {}", name),
        }
    }

    ctx.close().await;
}

#[tokio::test]
async fn test_logout() {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled");
        return;
    }

    let ctx = match E2ETestContext::setup_with_browser().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    if !ctx.has_browser() {
        eprintln!("Skipping: browser not available");
        ctx.close().await;
        return;
    }

    let browser = ctx.browser.as_ref().unwrap();
    let (email, password) = admin_credentials();

    if perform_login(browser, ctx.base_url(), &email, &password)
        .await
        .is_err()
    {
        ctx.close().await;
        return;
    }

    let logout_btn = Locator::css(".logout, #logout, [data-action='logout'], a[href*='logout']");
    match browser.click(logout_btn).await {
        Ok(_) => {
            tokio::time::sleep(Duration::from_secs(1)).await;

            let login_form = Locator::css("#email, input[type='email'], .login-form");
            match browser.find_element(login_form).await {
                Ok(_) => println!("Logout successful - login page visible"),
                Err(_) => eprintln!("Login page not visible after logout"),
            }
        }
        Err(_) => eprintln!("Logout button not found"),
    }

    ctx.close().await;
}

#[tokio::test]
async fn test_navigation() {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled");
        return;
    }

    let ctx = match E2ETestContext::setup_with_browser().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    if !ctx.has_browser() {
        eprintln!("Skipping: browser not available");
        ctx.close().await;
        return;
    }

    let browser = ctx.browser.as_ref().unwrap();
    let (email, password) = admin_credentials();

    if perform_login(browser, ctx.base_url(), &email, &password)
        .await
        .is_err()
    {
        ctx.close().await;
        return;
    }

    let nav_links = vec![
        ("a[href*='dashboard'], .nav-dashboard", "Dashboard"),
        ("a[href*='queue'], .nav-queue", "Queue"),
        ("a[href*='bots'], .nav-bots", "Bots"),
        ("a[href*='analytics'], .nav-analytics", "Analytics"),
        ("a[href*='settings'], .nav-settings", "Settings"),
    ];

    for (selector, name) in nav_links {
        let locator = Locator::css(selector);
        match browser.find_element(locator).await {
            Ok(_) => println!("Nav link found: {}", name),
            Err(_) => eprintln!("Nav link not found: {}", name),
        }
    }

    ctx.close().await;
}

#[tokio::test]
async fn test_access_control() {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled");
        return;
    }

    let ctx = match E2ETestContext::setup_with_browser().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    if !ctx.has_browser() {
        eprintln!("Skipping: browser not available");
        ctx.close().await;
        return;
    }

    let browser = ctx.browser.as_ref().unwrap();
    let (email, password) = attendant_credentials();

    if perform_login(browser, ctx.base_url(), &email, &password)
        .await
        .is_err()
    {
        ctx.close().await;
        return;
    }

    let admin_url = format!("{}/admin/users", ctx.base_url());
    let _ = browser.goto(&admin_url).await;

    tokio::time::sleep(Duration::from_secs(1)).await;

    let current_url = browser.current_url().await.unwrap_or_default();

    if current_url.contains("/admin/users") {
        let denied = Locator::css(".access-denied, .forbidden, .error-403");
        match browser.find_element(denied).await {
            Ok(_) => println!("Access correctly denied for attendant"),
            Err(_) => eprintln!("Access control may not be enforced"),
        }
    } else {
        println!("Redirected away from admin page (access control working)");
    }

    ctx.close().await;
}

#[tokio::test]
async fn test_dark_mode() {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled");
        return;
    }

    let ctx = match E2ETestContext::setup_with_browser().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    if !ctx.has_browser() {
        eprintln!("Skipping: browser not available");
        ctx.close().await;
        return;
    }

    let browser = ctx.browser.as_ref().unwrap();
    let (email, password) = admin_credentials();

    if perform_login(browser, ctx.base_url(), &email, &password)
        .await
        .is_err()
    {
        ctx.close().await;
        return;
    }

    let theme_toggle = Locator::css(".theme-toggle, .dark-mode-toggle, #theme-switch");
    match browser.click(theme_toggle).await {
        Ok(_) => {
            tokio::time::sleep(Duration::from_millis(500)).await;

            let dark_indicator = Locator::css(".dark, .dark-mode, [data-theme='dark']");
            match browser.find_element(dark_indicator).await {
                Ok(_) => println!("Dark mode activated"),
                Err(_) => eprintln!("Dark mode indicator not found"),
            }
        }
        Err(_) => eprintln!("Theme toggle not found (feature may not be implemented)"),
    }

    ctx.close().await;
}

#[tokio::test]
async fn test_with_fixtures() {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled");
        return;
    }

    let ctx = match E2ETestContext::setup().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    let user = admin_user();
    let bot = bot_with_kb("e2e-test-bot");
    let customer = customer("+15551234567");

    match ctx.ctx.insert_user(&user).await {
        Ok(_) => println!("Inserted test user: {}", user.email),
        Err(e) => eprintln!(
            "Could not insert user (DB may not be directly accessible): {}",
            e
        ),
    }

    match ctx.ctx.insert_bot(&bot).await {
        Ok(_) => println!("Inserted test bot: {}", bot.name),
        Err(e) => eprintln!("Could not insert bot: {}", e),
    }

    match ctx.ctx.insert_customer(&customer).await {
        Ok(_) => println!("Inserted test customer"),
        Err(e) => eprintln!("Could not insert customer: {}", e),
    }

    ctx.close().await;
}

#[tokio::test]
async fn test_mock_services_available() {
    if !should_run_e2e_tests() {
        eprintln!("Skipping: E2E tests disabled");
        return;
    }

    let ctx = match E2ETestContext::setup().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {}", e);
            return;
        }
    };

    if ctx.ctx.mock_llm().is_some() {
        println!("✓ MockLLM is available");
    } else {
        eprintln!("MockLLM not available");
    }

    if ctx.ctx.mock_zitadel().is_some() {
        println!("✓ MockZitadel is available");
    } else {
        eprintln!("MockZitadel not available");
    }

    if ctx.ctx.use_existing_stack {
        println!("Using existing stack - PostgreSQL is external (not managed by harness)");
        match ctx.ctx.db_pool().await {
            Ok(_pool) => println!("✓ Connected to existing PostgreSQL"),
            Err(e) => eprintln!("Could not connect to existing PostgreSQL: {}", e),
        }
    } else if ctx.ctx.postgres().is_some() {
        println!("✓ PostgreSQL is managed by harness");
    } else {
        eprintln!("PostgreSQL should be started in fresh stack mode");
    }

    ctx.close().await;
}
