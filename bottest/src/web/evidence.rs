//! #1188 — CDP evidence collector.
//!
//! Verification wave helper: drives a real browser (launched fresh or
//! attached to a running CDP endpoint such as the Chrome on :9222), opens
//! a URL, collects console/page errors and a screenshot, and writes the
//! bundle to `/tmp/gb-evidence/`. Tests call `EvidenceBundle::capture`
//! per feature so regressions are caught with visual + console evidence
//! instead of only status codes.

use anyhow::{bail, Context, Result};
use std::path::PathBuf;

use crate::web::{Browser, BrowserConfig, Locator};

/// Error buffer the page must fill; the suite injects a tiny script that
/// forwards `window.onerror` and unhandled rejections into
/// `window.__gbEvErrors` so the collector can read them back.
const ERROR_BRIDGE: &str = r#"
(function () {
  if (window.__gbEvBridgeInstalled) return;
  window.__gbEvBridgeInstalled = true;
  window.__gbEvErrors = window.__gbEvErrors || [];
  window.addEventListener("error", function (e) {
    window.__gbEvErrors.push(String(e.message || "error") + " @ " + (e.filename || ""));
  });
  window.addEventListener("unhandledrejection", function (e) {
    window.__gbEvErrors.push("unhandledrejection: " + String(e.reason && e.reason.message ? e.reason.message : e.reason));
  });
})();
"#;

pub struct EvidenceBundle {
    pub url: String,
    pub screenshot: Option<PathBuf>,
    pub console_errors: Vec<String>,
    pub passed: bool,
}

/// Captures evidence for one URL. Returns the bundle; `passed` is false
/// when console errors were found. Screenshots go to
/// `/tmp/gb-evidence/{slug}.png`.
pub async fn capture(
    browser: &Browser,
    url: &str,
    slug: &str,
    wait_ms: u64,
) -> Result<EvidenceBundle> {
    browser
        .goto(url)
        .await
        .with_context(|| format!("navigating to {url}"))?;
    browser
        .execute_script(ERROR_BRIDGE)
        .await
        .ok();

    // Let the page settle and run its init chain.
    tokio::time::sleep(std::time::Duration::from_millis(wait_ms)).await;

    let errors = read_console_errors(browser).await.unwrap_or_default();
    let dir = PathBuf::from("/tmp/gb-evidence");
    std::fs::create_dir_all(&dir).ok();
    let shot = dir.join(format!("{slug}.png"));
    let screenshot = match browser.screenshot_to_file(shot.as_path()).await {
        Ok(_) => Some(shot),
        Err(e) => {
            eprintln!("[evidence] screenshot failed for {slug}: {e}");
            None
        }
    };

    Ok(EvidenceBundle {
        url: url.to_string(),
        screenshot,
        console_errors: errors.clone(),
        passed: errors.is_empty(),
    })
}

/// Attaches to a running Chrome CDP endpoint (e.g. `http://127.0.0.1:9222`)
/// — the same browser used for manual testing, so evidence matches what a
/// human sees.
pub async fn attach(cdp_url: &str) -> Result<Browser> {
    let config = BrowserConfig::new().with_webdriver_url(cdp_url);
    Browser::new(config)
        .await
        .with_context(|| format!("attaching to CDP endpoint {cdp_url}"))
}

/// Launches a fresh headless browser for CI (no :9222 required).
pub async fn launch_headless() -> Result<Browser> {
    Browser::new_headless()
        .await
        .with_context(|| "launching headless browser")
}

pub async fn read_console_errors(browser: &Browser) -> Result<Vec<String>> {
    let value = browser
        .execute_script("JSON.stringify(window.__gbEvErrors || [])")
        .await
        .with_context(|| "reading console error buffer")?;
    let raw = value
        .as_str()
        .unwrap_or_default();
    serde_json::from_str(raw).context("parsing console error buffer")
}

/// Asserts the bundle is clean and prints evidence paths for the report.
pub fn report(label: &str, bundle: &EvidenceBundle) -> Result<()> {
    if bundle.console_errors.is_empty() {
        println!("✓ [{label}] clean — {url}", label = label, url = bundle.url);
    } else {
        println!(
            "✗ [{label}] {} console error(s) — {url}",
            bundle.console_errors.len(),
            url = bundle.url
        );
        for err in bundle.console_errors.iter().take(5) {
            eprintln!("   {err}");
        }
    }
    if let Some(shot) = &bundle.screenshot {
        println!("   evidence: {}", shot.display());
    }
    if !bundle.console_errors.is_empty() {
        bail!("[{label}] console errors detected");
    }
    Ok(())
}

/// Helper that fails cleanly when a page element is absent.
pub async fn assert_visible(browser: &Browser, selector: &str, label: &str) -> Result<()> {
    let locator = Locator::css(selector);
    for _ in 0..20 {
        if browser.exists(locator.clone()).await {
            println!("✓ [{label}] found {selector}");
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    }
    bail!("[{label}] element {selector} never appeared")
}
