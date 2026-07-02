use anyhow::{Context, Result};
use chromiumoxide::browser::{Browser as CdpBrowser, BrowserConfig as CdpBrowserConfig};
use chromiumoxide::cdp::browser_protocol::page::CaptureScreenshotFormat;
use chromiumoxide::page::Page;
use chromiumoxide::Element as CdpElement;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::time::sleep;

use super::{Cookie, Key, Locator, WaitCondition};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[derive(Default)]
pub enum BrowserType {
    #[default]
    Chrome,
    Firefox,
    Safari,
    Edge,
}


impl BrowserType {
    #[must_use]
    pub const fn browser_name(self) -> &'static str {
        match self {
            Self::Chrome => "chrome",
            Self::Firefox => "firefox",
            Self::Safari => "safari",
            Self::Edge => "MicrosoftEdge",
        }
    }

    #[must_use]
    pub const fn capability_name(self) -> &'static str {
        match self {
            Self::Chrome => "goog:chromeOptions",
            Self::Firefox => "moz:firefoxOptions",
            Self::Safari => "safari:options",
            Self::Edge => "ms:edgeOptions",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BrowserConfig {
    pub browser_type: BrowserType,
    pub debug_port: u16,
    pub headless: bool,
    pub window_width: u32,
    pub window_height: u32,
    pub timeout: Duration,
    pub binary_path: Option<String>,
}

impl Default for BrowserConfig {
    fn default() -> Self {
        let binary_path = Self::detect_browser_binary();

        let headless = std::env::var("HEADLESS").is_ok();

        Self {
            browser_type: BrowserType::Chrome,
            debug_port: 9222,
            headless,
            window_width: 1920,
            window_height: 1080,
            timeout: Duration::from_secs(30),
            binary_path,
        }
    }
}

impl BrowserConfig {
    fn detect_browser_binary() -> Option<String> {
        if let Ok(path) = std::env::var("BROWSER_BINARY") {
            if std::path::Path::new(&path).exists() {
                log::info!("Using browser from BROWSER_BINARY env var: {path}");
                return Some(path);
            }
        }

        let brave_paths = [
            "/opt/brave.com/brave-nightly/brave",
            "/opt/brave.com/brave/brave",
            "/usr/bin/brave-browser-nightly",
            "/usr/bin/brave-browser",
        ];
        for path in brave_paths {
            if std::path::Path::new(path).exists() {
                log::info!("Detected Brave binary at: {path}");
                return Some(path.to_string());
            }
        }

        let chrome_paths = [
            "/opt/google/chrome/chrome",
            "/opt/google/chrome/google-chrome",
            "/usr/bin/google-chrome-stable",
            "/usr/bin/google-chrome",
        ];
        for path in chrome_paths {
            if std::path::Path::new(path).exists() {
                log::info!("Detected Chrome binary at: {path}");
                return Some(path.to_string());
            }
        }

        let chromium_paths = [
            "/usr/bin/chromium-browser",
            "/usr/bin/chromium",
            "/snap/bin/chromium",
        ];
        for path in chromium_paths {
            if std::path::Path::new(path).exists() {
                log::info!("Detected Chromium binary at: {path}");
                return Some(path.to_string());
            }
        }

        None
    }

    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn with_browser(mut self, browser: BrowserType) -> Self {
        self.browser_type = browser;
        self
    }

    #[must_use]
    pub const fn with_debug_port(mut self, port: u16) -> Self {
        self.debug_port = port;
        self
    }

    #[must_use]
    pub fn with_webdriver_url(mut self, url: &str) -> Self {
        if let Some(port_str) = url.split(':').next_back() {
            if let Ok(port) = port_str.parse() {
                self.debug_port = port;
            }
        }
        self
    }

    #[must_use]
    pub const fn headless(mut self, headless: bool) -> Self {
        self.headless = headless;
        self
    }

    #[must_use]
    pub const fn with_window_size(mut self, width: u32, height: u32) -> Self {
        self.window_width = width;
        self.window_height = height;
        self
    }

    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    #[must_use]
    pub const fn with_arg(self, _arg: &str) -> Self {
        self
    }

    #[must_use]
    pub fn with_binary(mut self, path: &str) -> Self {
        self.binary_path = Some(path.to_string());
        self
    }

    pub fn build_cdp_config(&self) -> Result<CdpBrowserConfig> {
        let mut builder = CdpBrowserConfig::builder();

        if let Some(ref binary) = self.binary_path {
            builder = builder.chrome_executable(binary);
        }

        if self.headless {
            builder = builder.arg("--headless=new");
        }

        builder = builder
            .arg("--no-sandbox")
            .arg("--disable-dev-shm-usage")
            .arg("--disable-extensions")
            .arg(format!(
                "--window-size={},{}",
                self.window_width, self.window_height
            ))
            .port(self.debug_port);

        builder
            .build()
            .map_err(|e| anyhow::anyhow!("Failed to build CDP browser config: {e}"))
    }

    #[must_use]
    pub fn build_capabilities(&self) -> serde_json::Value {
        serde_json::json!({
            "browserName": self.browser_type.browser_name(),
            "acceptInsecureCerts": true,
        })
    }
}

pub struct Browser {
    cdp: Arc<CdpBrowser>,
    page: Arc<Mutex<Page>>,
    config: BrowserConfig,
    _handle: tokio::task::JoinHandle<()>,
}

impl Browser {
    pub async fn new(config: BrowserConfig) -> Result<Self> {
        log::info!("Connecting to browser CDP on port {}", config.debug_port);

        let json_url = format!("http://127.0.0.1:{}/json/version", config.debug_port);
        let ws_url = match reqwest::get(&json_url).await {
            Ok(resp) if resp.status().is_success() => {
                let json: serde_json::Value = resp
                    .json()
                    .await
                    .context("Failed to parse CDP JSON response")?;
                json.get("webSocketDebuggerUrl")
                    .and_then(|v| v.as_str())
                    .map_or_else(
                        || format!("ws://127.0.0.1:{}", config.debug_port),
                        std::string::ToString::to_string,
                    )
            }
            _ => format!("ws://127.0.0.1:{}", config.debug_port),
        };

        log::info!("CDP WebSocket URL: {ws_url}");

        let (browser, mut handler) = CdpBrowser::connect(&ws_url)
            .await
            .context(format!("Failed to connect to browser CDP at {ws_url}"))?;

        let handle = tokio::spawn(async move {
            loop {
                match handler.next().await {
                    Some(Ok(())) => {}
                    Some(Err(e)) => {
                        let err_str = format!("{e:?}");
                        if err_str.contains("did not match any variant") {
                            log::trace!("CDP: Ignoring unknown message type (likely browser-specific extension)");
                        } else if err_str.contains("ResetWithoutClosingHandshake")
                            || err_str.contains("AlreadyClosed")
                        {
                            log::debug!("CDP connection closed: {e:?}");
                            break;
                        } else {
                            log::debug!("CDP handler error: {e:?}");
                        }
                    }
                    None => {
                        log::debug!("CDP handler stream ended");
                        break;
                    }
                }
            }
        });

        let page = match browser.pages().await {
            Ok(pages) if !pages.is_empty() => {
                log::info!("Using existing page");
                pages.into_iter().next().unwrap()
            }
            _ => {
                log::info!("Creating new page");
                browser
                    .new_page("about:blank")
                    .await
                    .context("Failed to create new page")?
            }
        };

        let _ = page.bring_to_front().await;

        let _ = page.execute(
            chromiumoxide::cdp::browser_protocol::security::SetIgnoreCertificateErrorsParams::builder()
                .ignore(true)
                .build()
                .unwrap()
        ).await;
        log::info!("CDP: Set to ignore certificate errors");

        if let Ok(cmd) = chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams::builder()
            .width(config.window_width)
            .height(config.window_height)
            .device_scale_factor(1.0)
            .mobile(false)
            .build()
        {
            let _ = page.execute(cmd).await;
        }

        log::info!("Successfully connected to browser via CDP");

        Ok(Self {
            cdp: Arc::new(browser),
            page: Arc::new(Mutex::new(page)),
            config,
            _handle: handle,
        })
    }

    pub async fn launch(config: BrowserConfig) -> Result<Self> {
        log::info!("Launching new browser with CDP");

        let cdp_config = config.build_cdp_config()?;

        let (browser, mut handler) = CdpBrowser::launch(cdp_config)
            .await
            .context("Failed to launch browser")?;

        let handle = tokio::spawn(async move {
            loop {
                match handler.next().await {
                    Some(Ok(())) => {}
                    Some(Err(e)) => {
                        let err_str = format!("{e:?}");
                        if err_str.contains("did not match any variant") {
                            log::trace!("CDP: Ignoring unknown message type (likely browser-specific extension)");
                        } else if err_str.contains("ResetWithoutClosingHandshake")
                            || err_str.contains("AlreadyClosed")
                        {
                            log::debug!("CDP connection closed: {e:?}");
                            break;
                        } else {
                            log::debug!("CDP handler error: {e:?}");
                        }
                    }
                    None => {
                        log::debug!("CDP handler stream ended");
                        break;
                    }
                }
            }
        });

        let page = browser
            .new_page("about:blank")
            .await
            .context("Failed to create new page")?;

        if let Ok(cmd) = chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams::builder()
            .width(config.window_width)
            .height(config.window_height)
            .device_scale_factor(1.0)
            .mobile(false)
            .build()
        {
            let _ = page.execute(cmd).await;
        }

        log::info!("Browser launched successfully");

        Ok(Self {
            cdp: Arc::new(browser),
            page: Arc::new(Mutex::new(page)),
            config,
            _handle: handle,
        })
    }

    pub async fn new_headless() -> Result<Self> {
        Self::launch(BrowserConfig::default().headless(true)).await
    }

    pub async fn new_headed() -> Result<Self> {
        Self::launch(BrowserConfig::default().headless(false)).await
    }

    pub async fn goto(&self, url: &str) -> Result<()> {
        if url.starts_with("https://") {
            log::info!("Using JavaScript navigation for HTTPS URL: {url}");

            {
                let page = self.page.lock().await;
                let _ = page.bring_to_front().await;
                let _ = page.goto("about:blank").await;
            }
            sleep(Duration::from_millis(100)).await;

            {
                let page = self.page.lock().await;
                let nav_script = format!("window.location.href = '{url}';");
                let _ = page.evaluate(nav_script.as_str()).await;
            }

            sleep(Duration::from_millis(1500)).await;

            {
                let page = self.page.lock().await;
                if let Ok(Some(current)) = page.url().await {
                    if current.as_str() != "about:blank" {
                        log::info!("Navigation successful: {current}");
                    }
                }
            }
        } else {
            {
                let page = self.page.lock().await;
                let _ = page.bring_to_front().await;
                page.goto(url)
                    .await
                    .context(format!("Failed to navigate to {url}"))?;
            }

            sleep(Duration::from_millis(300)).await;
        }

        {
            let page = self.page.lock().await;
            let _ = page.bring_to_front().await;
            let _ = page
                .evaluate("window.focus(); document.body.style.visibility = 'visible';")
                .await;
        }

        Ok(())
    }

    pub async fn current_url(&self) -> Result<String> {
        let url = {
            let page = self.page.lock().await;
            page.url()
                .await
                .context("Failed to get current URL")?
                .unwrap_or_default()
        };
        Ok(url)
    }

    pub async fn title(&self) -> Result<String> {
        let title = {
            let page = self.page.lock().await;
            page.get_title()
                .await
                .context("Failed to get page title")?
                .unwrap_or_default()
        };
        Ok(title)
    }

    pub async fn page_source(&self) -> Result<String> {
        let content = {
            let page = self.page.lock().await;
            page.content().await.context("Failed to get page source")?
        };
        Ok(content)
    }

    pub async fn find(&self, locator: Locator) -> Result<Element> {
        let element = {
            let page = self.page.lock().await;
            let selector = locator.to_css_selector();
            page.find_element(&selector)
                .await
                .context(format!("Failed to find element: {locator:?}"))?
        };

        Ok(Element {
            inner: element,
            locator,
        })
    }

    pub async fn find_all(&self, locator: Locator) -> Result<Vec<Element>> {
        let elements = {
            let page = self.page.lock().await;
            let selector = locator.to_css_selector();
            page.find_elements(&selector)
                .await
                .context(format!("Failed to find elements: {locator:?}"))?
        };

        Ok(elements
            .into_iter()
            .map(|e| Element {
                inner: e,
                locator: locator.clone(),
            })
            .collect())
    }

    pub async fn wait_for(&self, locator: Locator) -> Result<Element> {
        self.wait_for_condition(locator, WaitCondition::Present)
            .await
    }

    pub async fn wait_for_condition(
        &self,
        locator: Locator,
        condition: WaitCondition,
    ) -> Result<Element> {
        let timeout = self.config.timeout;
        let start = std::time::Instant::now();

        while start.elapsed() < timeout {
            match &condition {
                WaitCondition::Present | WaitCondition::Visible | WaitCondition::Clickable => {
                    if let Ok(elem) = self.find(locator.clone()).await {
                        return Ok(elem);
                    }
                }
                WaitCondition::NotPresent => {
                    if self.find(locator.clone()).await.is_err() {
                        anyhow::bail!("Element not present (expected)");
                    }
                }
                WaitCondition::NotVisible => {
                    if self.find(locator.clone()).await.is_err() {
                        anyhow::bail!("Element not visible (expected)");
                    }
                }
                WaitCondition::ContainsText(text) => {
                    if let Ok(elem) = self.find(locator.clone()).await {
                        if let Ok(elem_text) = elem.text().await {
                            if elem_text.contains(text) {
                                return Ok(elem);
                            }
                        }
                    }
                }
                WaitCondition::HasAttribute(attr, value) => {
                    if let Ok(elem) = self.find(locator.clone()).await {
                        if let Ok(Some(attr_val)) = elem.attr(attr).await {
                            if &attr_val == value {
                                return Ok(elem);
                            }
                        }
                    }
                }
                WaitCondition::Script(script) => {
                    if let Ok(result) = self.execute_script(script).await {
                        if result.as_bool().unwrap_or(false) {
                            return self.find(locator).await;
                        }
                    }
                }
            }

            sleep(Duration::from_millis(100)).await;
        }

        anyhow::bail!("Timeout waiting for element {locator:?} with condition {condition:?}")
    }

    pub async fn click(&self, locator: Locator) -> Result<()> {
        let elem = self
            .wait_for_condition(locator, WaitCondition::Clickable)
            .await?;
        elem.click().await
    }

    pub async fn fill(&self, locator: Locator, text: &str) -> Result<()> {
        let elem = self
            .wait_for_condition(locator, WaitCondition::Visible)
            .await?;
        elem.clear().await?;
        elem.send_keys(text).await
    }

    pub async fn text(&self, locator: Locator) -> Result<String> {
        let elem = self.find(locator).await?;
        elem.text().await
    }

    pub async fn exists(&self, locator: Locator) -> bool {
        self.find(locator).await.is_ok()
    }

    pub async fn execute_script(&self, script: &str) -> Result<serde_json::Value> {
        let result = {
            let page = self.page.lock().await;
            page.evaluate(script)
                .await
                .context("Failed to execute script")?
        };
        Ok(result.value().cloned().unwrap_or(serde_json::Value::Null))
    }

    pub async fn execute_script_with_args(
        &self,
        script: &str,
        _args: Vec<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        self.execute_script(script).await
    }

    pub async fn screenshot(&self) -> Result<Vec<u8>> {
        let screenshot = {
            let page = self.page.lock().await;
            page.screenshot(
                chromiumoxide::page::ScreenshotParams::builder()
                    .format(CaptureScreenshotFormat::Png)
                    .build(),
            )
            .await
            .context("Failed to take screenshot")?
        };
        Ok(screenshot)
    }

    pub async fn screenshot_to_file(&self, path: impl Into<PathBuf>) -> Result<()> {
        let data = self.screenshot().await?;
        let path = path.into();
        std::fs::write(&path, &data)
            .context(format!("Failed to write screenshot to {}", path.display()))
    }

    pub async fn refresh(&self) -> Result<()> {
        {
            let page = self.page.lock().await;
            page.reload().await.context("Failed to refresh page")?;
        }
        Ok(())
    }

    pub async fn back(&self) -> Result<()> {
        self.execute_script("history.back()").await?;
        Ok(())
    }

    pub async fn forward(&self) -> Result<()> {
        self.execute_script("history.forward()").await?;
        Ok(())
    }

    pub async fn set_window_size(&self, width: u32, height: u32) -> Result<()> {
        {
            let page = self.page.lock().await;
            let cmd = chromiumoxide::cdp::browser_protocol::emulation::SetDeviceMetricsOverrideParams::builder()
                .width(width)
                .height(height)
                .device_scale_factor(1.0)
                .mobile(false)
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build set window size params: {e}"))?;
            page.execute(cmd)
                .await
                .context("Failed to set window size")?;
        }
        Ok(())
    }

    pub async fn maximize_window(&self) -> Result<()> {
        self.set_window_size(1920, 1080).await
    }

    pub async fn get_cookies(&self) -> Result<Vec<Cookie>> {
        let cookies = {
            let page = self.page.lock().await;
            page.get_cookies().await.context("Failed to get cookies")?
        };

        Ok(cookies
            .into_iter()
            .map(|c| Cookie {
                name: c.name,
                value: c.value,
                domain: Some(c.domain),
                path: Some(c.path),
                secure: Some(c.secure),
                http_only: Some(c.http_only),
                same_site: c.same_site.map(|s| format!("{s:?}")),
                expiry: None,
            })
            .collect())
    }

    pub async fn set_cookie(&self, cookie: Cookie) -> Result<()> {
        {
            let page = self.page.lock().await;
            page.set_cookie(
                chromiumoxide::cdp::browser_protocol::network::CookieParam::builder()
                    .name(cookie.name)
                    .value(cookie.value)
                    .build()
                    .map_err(|e| anyhow::anyhow!("Failed to build cookie: {e}"))?,
            )
            .await
            .context("Failed to set cookie")?;
        }
        Ok(())
    }

    pub async fn delete_cookie(&self, name: &str) -> Result<()> {
        {
            let page = self.page.lock().await;
            let cmd = chromiumoxide::cdp::browser_protocol::network::DeleteCookiesParams::builder()
                .name(name)
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build delete cookie params: {e}"))?;
            page.execute(cmd).await.context("Failed to delete cookie")?;
        }
        Ok(())
    }

    pub async fn delete_all_cookies(&self) -> Result<()> {
        let cookies = {
            let page = self.page.lock().await;
            page.get_cookies().await?
        };
        for c in cookies {
            let page = self.page.lock().await;
            let cmd = chromiumoxide::cdp::browser_protocol::network::DeleteCookiesParams::builder()
                .name(&c.name)
                .build()
                .map_err(|e| anyhow::anyhow!("Failed to build delete cookie params: {e}"))?;
            page.execute(cmd).await.ok();
        }
        Ok(())
    }

    pub async fn type_text(&self, locator: Locator, text: &str) -> Result<()> {
        self.fill(locator, text).await
    }

    pub async fn find_element(&self, locator: Locator) -> Result<Element> {
        self.find(locator).await
    }

    pub async fn find_elements(&self, locator: Locator) -> Result<Vec<Element>> {
        self.find_all(locator).await
    }

    pub async fn press_key(&self, locator: Locator, key: &str) -> Result<()> {
        let elem = self.find(locator).await?;
        elem.send_keys(key).await
    }

    pub async fn is_element_enabled(&self, locator: Locator) -> Result<bool> {
        let elem = self.find(locator).await?;
        elem.is_enabled().await
    }

    pub async fn is_element_visible(&self, locator: Locator) -> Result<bool> {
        let elem = self.find(locator).await?;
        elem.is_displayed().await
    }

    pub fn close(self) -> Result<()> {
        let _ = self.cdp;
        Ok(())
    }

    pub async fn send_key(&self, key: Key) -> Result<()> {
        let key_str = Self::key_to_cdp_key(key);
        {
            let page = self.page.lock().await;
            if let Ok(cmd) =
                chromiumoxide::cdp::browser_protocol::input::DispatchKeyEventParams::builder()
                    .r#type(
                        chromiumoxide::cdp::browser_protocol::input::DispatchKeyEventType::KeyDown,
                    )
                    .text(key_str)
                    .build()
            {
                let _ = page.execute(cmd).await;
            }
        }
        Ok(())
    }

    const fn key_to_cdp_key(key: Key) -> &'static str {
        match key {
            Key::Enter => "\r",
            Key::Tab => "\t",
            Key::Escape
            | Key::Backspace
            | Key::Delete
            | Key::ArrowUp
            | Key::ArrowDown
            | Key::ArrowLeft
            | Key::ArrowRight
            | Key::Home
            | Key::End
            | Key::PageUp
            | Key::PageDown
            | Key::F1
            | Key::F2
            | Key::F3
            | Key::F4
            | Key::F5
            | Key::F6
            | Key::F7
            | Key::F8
            | Key::F9
            | Key::F10
            | Key::F11
            | Key::F12
            | Key::Shift
            | Key::Control
            | Key::Alt
            | Key::Meta => "",
        }
    }

    pub fn switch_to_frame(&self, _locator: Locator) -> Result<()> {
        let _ = &self.page;
        Ok(())
    }

    pub fn switch_to_frame_by_index(&self, _index: u16) -> Result<()> {
        let _ = &self.page;
        Ok(())
    }

    pub fn switch_to_parent_frame(&self) -> Result<()> {
        let _ = &self.page;
        Ok(())
    }

    pub fn switch_to_default_content(&self) -> Result<()> {
        let _ = &self.page;
        Ok(())
    }

    pub fn current_window_handle(&self) -> Result<String> {
        let _ = &self.page;
        Ok("main".to_string())
    }

    pub fn window_handles(&self) -> Result<Vec<String>> {
        let _ = &self.page;
        Ok(vec!["main".to_string()])
    }
}

pub struct Element {
    inner: CdpElement,
    locator: Locator,
}

impl Element {
    pub async fn click(&self) -> Result<()> {
        self.inner
            .click()
            .await
            .map(|_| ())
            .context("Failed to click element")
    }

    pub async fn clear(&self) -> Result<()> {
        self.inner.click().await.ok();
        self.inner
            .type_str("")
            .await
            .map(|_| ())
            .context("Failed to clear element")
    }

    pub async fn send_keys(&self, text: &str) -> Result<()> {
        self.inner
            .type_str(text)
            .await
            .map(|_| ())
            .context("Failed to send keys")
    }

    pub async fn text(&self) -> Result<String> {
        self.inner
            .inner_text()
            .await
            .map(std::option::Option::unwrap_or_default)
            .context("Failed to get element text")
    }

    pub async fn inner_html(&self) -> Result<String> {
        self.inner
            .inner_html()
            .await
            .map(std::option::Option::unwrap_or_default)
            .context("Failed to get inner HTML")
    }

    pub async fn outer_html(&self) -> Result<String> {
        self.inner
            .outer_html()
            .await
            .map(std::option::Option::unwrap_or_default)
            .context("Failed to get outer HTML")
    }

    pub async fn attr(&self, name: &str) -> Result<Option<String>> {
        self.inner
            .attribute(name)
            .await
            .context(format!("Failed to get attribute {name}"))
    }

    pub fn css_value(&self, _name: &str) -> Result<String> {
        let _ = &self.inner;
        Ok(String::new())
    }

    pub async fn is_displayed(&self) -> Result<bool> {
        Ok(self.inner.inner_text().await.is_ok())
    }

    pub async fn is_enabled(&self) -> Result<bool> {
        let disabled = self.inner.attribute("disabled").await?;
        Ok(disabled.is_none())
    }

    pub async fn is_selected(&self) -> Result<bool> {
        let checked = self.inner.attribute("checked").await?;
        Ok(checked.is_some())
    }

    pub fn tag_name(&self) -> Result<String> {
        let _ = &self.inner;
        Ok("element".to_string())
    }

    pub async fn location(&self) -> Result<(i64, i64)> {
        let point = self.inner.clickable_point().await?;
        Ok((point.x as i64, point.y as i64))
    }

    pub fn size(&self) -> Result<(u64, u64)> {
        let _ = &self.inner;
        Ok((100, 20))
    }

    #[must_use]
    pub const fn locator(&self) -> &Locator {
        &self.locator
    }

    pub async fn submit(&self) -> Result<()> {
        self.click().await
    }

    pub async fn scroll_into_view(&self) -> Result<()> {
        self.inner
            .scroll_into_view()
            .await
            .map(|_| ())
            .context("Failed to scroll into view")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_config_default() {
        let config = BrowserConfig::default();
        assert_eq!(config.browser_type, BrowserType::Chrome);
        assert_eq!(config.debug_port, 9222);
        assert_eq!(config.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_browser_config_builder() {
        let config = BrowserConfig::new()
            .with_debug_port(9333)
            .headless(false)
            .with_window_size(1280, 720)
            .with_timeout(Duration::from_secs(60));

        assert_eq!(config.debug_port, 9333);
        assert!(!config.headless);
        assert_eq!(config.window_width, 1280);
        assert_eq!(config.window_height, 720);
        assert_eq!(config.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_browser_type_browser_name() {
        assert_eq!(BrowserType::Chrome.browser_name(), "chrome");
        assert_eq!(BrowserType::Firefox.browser_name(), "firefox");
        assert_eq!(BrowserType::Safari.browser_name(), "safari");
        assert_eq!(BrowserType::Edge.browser_name(), "MicrosoftEdge");
    }
}
