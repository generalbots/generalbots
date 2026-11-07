use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use chromiumoxide::browser::{Browser as CdpBrowser, BrowserConfig as CdpBrowserConfig};
use chromiumoxide::page::Page;
use chrono::{DateTime, Utc};
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;

pub mod agent;
pub mod api;

pub use agent::*;
pub use api::*;

pub type SessionMap = Arc<Mutex<std::collections::HashMap<String, BrowserSession>>>;

/// Metadata that ties a browser session to the container/VM and to the
/// conversation that opened it (#1843). Sessions without a VM still work as
/// the legacy isolated headless browser.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BrowserTabMeta {
    #[serde(default)]
    pub vm: Option<String>,
    #[serde(default)]
    pub conversation_id: Option<String>,
    #[serde(default)]
    pub vm_controlled: bool,
    #[serde(default)]
    pub title: Option<String>,
}

/// Wire shape of a tab in the shared VM tab strip. `vm_controlled` is the
/// green badge: the tab was opened by an agent inside the user's VM, not by
/// the person typing in the browser app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrowserTab {
    pub id: String,
    pub url: String,
    pub title: String,
    pub vm: String,
    pub conversation_id: Option<String>,
    pub vm_controlled: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct BrowserSession {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub browser: Arc<Mutex<CdpBrowser>>,
    pub page: Arc<Mutex<Page>>,
    pub current_url: Arc<Mutex<String>>,
    pub title: Arc<Mutex<String>>,
    pub vm: String,
    pub conversation_id: Arc<Mutex<Option<String>>>,
    pub vm_controlled: bool,
}

/// Returns the CDP endpoint to attach to, or None when the session should
/// launch its own chromium process. Mirrors the env contract used by the
/// Vibe browser tools (`BROWSER_CDP_URL`) so prod can expose a shared
/// chromium without shipping a binary into the container.
///
/// #1843: `BROWSER_CDP_URL_TEMPLATE` may contain `{vm}` and lets every tab of
/// a VM attach to that VM's single Chromium (X session inside the container)
/// instead of launching a process per tab.
fn cdp_connect_url(vm: &str) -> Option<String> {
    if let Ok(template) = std::env::var("BROWSER_CDP_URL_TEMPLATE") {
        let template = template.trim();
        if !template.is_empty() {
            return Some(template.replace("{vm}", vm));
        }
    }
    std::env::var("BROWSER_CDP_URL")
        .ok()
        .filter(|u| !u.trim().is_empty())
}

impl BrowserSession {
    pub async fn new(headless: bool) -> Result<Self> {
        Self::new_with_meta(headless, BrowserTabMeta::default()).await
    }

    pub async fn new_with_meta(headless: bool, meta: BrowserTabMeta) -> Result<Self> {
        let id = uuid::Uuid::new_v4().to_string();
        let vm = {
            let vm = meta.vm.clone().unwrap_or_default();
            if vm.trim().is_empty() {
                "default".to_string()
            } else {
                vm.trim().to_string()
            }
        };

        let (browser, mut handler) = if let Some(cdp_url) = cdp_connect_url(&vm) {
            CdpBrowser::connect(&cdp_url)
                .await
                .context("Failed to connect to browser CDP endpoint")?
        } else {
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

            CdpBrowser::launch(config)
                .await
                .context("Failed to launch browser")?
        };

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
        let title = page.get_title().await.ok().flatten().unwrap_or_default();

        Ok(Self {
            id,
            created_at: Utc::now(),
            browser: Arc::new(Mutex::new(browser)),
            page: Arc::new(Mutex::new(page)),
            current_url: Arc::new(Mutex::new(url)),
            title: Arc::new(Mutex::new(meta.title.unwrap_or(title))),
            vm,
            conversation_id: Arc::new(Mutex::new(meta.conversation_id)),
            vm_controlled: meta.vm_controlled,
        })
    }

    /// Snapshot for the shared tab strip. The URL is tracked when navigation
    /// goes through this crate; `about:blank` stays empty on purpose.
    pub async fn tab(&self) -> BrowserTab {
        let url = self.current_url.lock().await.clone();
        let title = self.title.lock().await.clone();
        BrowserTab {
            id: self.id.clone(),
            url,
            title: if title.trim().is_empty() {
                "New Tab".to_string()
            } else {
                title
            },
            vm: self.vm.clone(),
            conversation_id: self.conversation_id.lock().await.clone(),
            vm_controlled: self.vm_controlled,
            created_at: self.created_at,
        }
    }

    /// Explicit attach: another conversation may claim this tab, so a shared
    /// bank/login tab is never silently reused. The caller decides when that
    /// is acceptable.
    pub async fn attach_conversation(&self, conversation_id: Option<String>) {
        *self.conversation_id.lock().await = conversation_id;
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
        *self.title.lock().await = title.clone();

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
        *self.title.lock().await = title.clone();

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
        *self.title.lock().await = title.clone();

        Ok(PageState {
            url: current_url,
            title,
            text_snippet: text_content.chars().take(3000).collect(),
            text_length: text_content.len(),
        })
    }

    /// Coordinate click for the VNC-like surface. The frame the user sees is
    /// the page screenshot, so selector-based clicks cannot be derived from it.
    pub async fn click_at(&self, x: f64, y: f64) -> Result<()> {
        let page = self.page.lock().await;
        page.click(chromiumoxide::layout::Point { x, y })
            .await
            .context("Failed to click at coordinates")?;
        Ok(())
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
        let data = page
            .screenshot(params)
            .await
            .context("Failed to screenshot")?;
        Ok(data)
    }

    pub async fn execute(&self, script: &str) -> Result<Value> {
        let page = self.page.lock().await;
        let result = page
            .evaluate(script)
            .await
            .context("Failed to execute script")?;
        Ok(result.value().cloned().unwrap_or(Value::Null))
    }

    pub async fn extract_page_state(&self) -> Result<PageState> {
        let page = self.page.lock().await;
        let current_url = page.url().await.ok().flatten().unwrap_or_default();
        let title = page.get_title().await.ok().flatten().unwrap_or_default();
        let text_content = extract_page_text(&page).await;

        *self.current_url.lock().await = current_url.clone();
        *self.title.lock().await = title.clone();

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
