use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use chromiumoxide::browser::{Browser as CdpBrowser, BrowserConfig as CdpBrowserConfig};
use chromiumoxide::page::Page;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;

pub mod agent;
pub mod api;

pub use agent::*;
pub use api::*;

pub type SessionMap = Arc<Mutex<std::collections::HashMap<String, BrowserSession>>>;

#[derive(Debug, Clone)]
pub struct BrowserSession {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub browser: Arc<Mutex<CdpBrowser>>,
    pub page: Arc<Mutex<Page>>,
    pub current_url: Arc<Mutex<String>>,
}

impl BrowserSession {
    pub async fn new(headless: bool) -> Result<Self> {
        let id = uuid::Uuid::new_v4().to_string();
        // Unique per-session Chrome profile: avoids the singleton lock that
        // kills concurrent launches (all sessions share the default profile).
        let profile_dir = std::env::temp_dir().join(format!("gb-browser-{id}"));
        let mut builder = CdpBrowserConfig::builder()
            .no_sandbox()
            .user_data_dir(&profile_dir)
            .viewport(chromiumoxide::handler::viewport::Viewport {
                width: 1280,
                height: 720,
                device_scale_factor: None,
                emulating_mobile: false,
                is_landscape: false,
                has_touch: false,
            })
            .window_size(1280, 720)
            .launch_timeout(Duration::from_secs(30))
            .request_timeout(Duration::from_secs(30));

        if headless {
            builder = builder.headless_mode(chromiumoxide::browser::HeadlessMode::New);
        }

        let config = builder.build().map_err(|e| anyhow::anyhow!("{e}"))?;

        let (browser, mut handler) = CdpBrowser::launch(config)
            .await
            .context("Failed to launch browser")?;

        tokio::spawn(async move {
            while let Some(event) = handler.next().await {
                if let Err(e) = event {
                    log::error!("Browser event handler error: {e}");
                }
            }
        });

        let page = browser
            .new_page("about:blank")
            .await
            .context("Failed to create new page")?;

        let url = page.url().await.ok().flatten().unwrap_or_default();

        Ok(Self {
            id,
            created_at: Utc::now(),
            browser: Arc::new(Mutex::new(browser)),
            page: Arc::new(Mutex::new(page)),
            current_url: Arc::new(Mutex::new(url)),
        })
    }

    pub async fn navigate(&self, url: &str) -> Result<PageState> {
        let page = self.page.lock().await;
        page.goto(url).await.context("Failed to navigate")?;
        page.wait_for_navigation()
            .await
            .context("Failed to wait for navigation")?;

        let current_url = page.url().await.ok().flatten().unwrap_or_default();
        let title = page.get_title().await.ok().flatten().unwrap_or_default();
        let text_content = extract_page_text(&page).await;

        *self.current_url.lock().await = current_url.clone();

        Ok(PageState {
            url: current_url,
            title,
            text_snippet: text_content.chars().take(3000).collect(),
            text_length: text_content.len(),
        })
    }

    pub async fn navigate_with_result(&self, url: &str) -> Result<NavigationResult> {
        let page = self.page.lock().await;
        page.goto(url).await.context("Failed to navigate")?;
        page.wait_for_navigation()
            .await
            .context("Failed to wait for navigation")?;

        let current_url = page.url().await.ok().flatten().unwrap_or_default();
        let title = page.get_title().await.ok().flatten().unwrap_or_default();
        let text_content = extract_page_text(&page).await;
        let links = extract_links(&page).await;
        let headings = extract_headings(&page).await;

        *self.current_url.lock().await = current_url.clone();

        Ok(NavigationResult {
            url: current_url,
            title,
            text: text_content.chars().take(5000).collect(),
            links,
            headings,
            screenshot_b64: None,
        })
    }

    pub async fn click(&self, selector: &str) -> Result<PageState> {
        let page = self.page.lock().await;
        if let Ok(element) = page.find_element(selector).await {
            element.click().await.context("Failed to click element")?;
            tokio::time::sleep(Duration::from_millis(500)).await;
        }

        let current_url = page.url().await.ok().flatten().unwrap_or_default();
        let title = page.get_title().await.ok().flatten().unwrap_or_default();
        let text_content = extract_page_text(&page).await;

        *self.current_url.lock().await = current_url.clone();

        Ok(PageState {
            url: current_url,
            title,
            text_snippet: text_content.chars().take(3000).collect(),
            text_length: text_content.len(),
        })
    }

    pub async fn fill(&self, selector: &str, text: &str) -> Result<()> {
        let page = self.page.lock().await;
        if let Ok(element) = page.find_element(selector).await {
            element.click().await.ok();
            let clear_js = format!(
                r#"document.querySelector('{}').value = '';"#,
                selector.replace('\'', "\\'")
            );
            let _ = page.evaluate(clear_js.as_str()).await;
            element
                .type_str(text)
                .await
                .context("Failed to type text")?;
        }
        Ok(())
    }

    pub async fn screenshot(&self) -> Result<Vec<u8>> {
        let page = self.page.lock().await;
        let params = chromiumoxide::page::ScreenshotParams::builder()
            .format(chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat::Png)
            .from_surface(true)
            .build();
        let data = page.screenshot(params).await.context("Failed to screenshot")?;
        Ok(data)
    }

    pub async fn execute(&self, script: &str) -> Result<Value> {
        let page = self.page.lock().await;
        let result = page.evaluate(script).await.context("Failed to execute script")?;
        Ok(result.value().cloned().unwrap_or(Value::Null))
    }

    pub async fn extract_page_state(&self) -> Result<PageState> {
        let page = self.page.lock().await;
        let current_url = page.url().await.ok().flatten().unwrap_or_default();
        let title = page.get_title().await.ok().flatten().unwrap_or_default();
        let text_content = extract_page_text(&page).await;

        Ok(PageState {
            url: current_url,
            title,
            text_snippet: text_content.chars().take(3000).collect(),
            text_length: text_content.len(),
        })
    }

    pub async fn extract_text(&self) -> Result<String> {
        let page = self.page.lock().await;
        Ok(extract_page_text(&page).await)
    }

    pub async fn extract_links(&self) -> Result<Vec<LinkInfo>> {
        let page = self.page.lock().await;
        Ok(extract_links(&page).await)
    }

    pub async fn close(&self) -> Result<()> {
        let page = self.page.lock().await;
        let _ = page.clone().close().await;
        Ok(())
    }
}

async fn extract_page_text(page: &Page) -> String {
    page.evaluate("document.body.innerText")
        .await
        .ok()
        .and_then(|r| r.into_value::<String>().ok())
        .unwrap_or_default()
}

async fn extract_links(page: &Page) -> Vec<LinkInfo> {
    let script = r#"
        JSON.stringify(
            Array.from(document.querySelectorAll('a[href]')).map(a => ({
                text: a.innerText.trim(),
                href: a.href,
                title: a.title || ''
            })).filter(l => l.href.startsWith('http'))
        );
    "#;

    page.evaluate(script)
        .await
        .ok()
        .and_then(|r| r.into_value::<Vec<LinkInfo>>().ok())
        .unwrap_or_default()
}

async fn extract_headings(page: &Page) -> Vec<String> {
    let script = r#"
        JSON.stringify(
            Array.from(document.querySelectorAll('h1,h2,h3')).map(h => h.tagName + ': ' + h.innerText.trim())
        );
    "#;

    page.evaluate(script)
        .await
        .ok()
        .and_then(|r| r.into_value::<Vec<String>>().ok())
        .unwrap_or_default()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageState {
    pub url: String,
    pub title: String,
    pub text_snippet: String,
    pub text_length: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavigationResult {
    pub url: String,
    pub title: String,
    pub text: String,
    pub links: Vec<LinkInfo>,
    pub headings: Vec<String>,
    pub screenshot_b64: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkInfo {
    pub text: String,
    pub href: String,
    pub title: String,
}
