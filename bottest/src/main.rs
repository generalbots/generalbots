#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use anyhow::Result;
use std::env;
use std::path::PathBuf;
use std::process::ExitCode;
use tracing::{error, info, warn, Level};
use tracing_subscriber::FmtSubscriber;

mod bot;
mod desktop;
mod fixtures;
mod harness;
mod mocks;
mod ports;
mod services;
mod web;

pub use harness::{TestConfig, TestContext, TestHarness};
pub use ports::PortAllocator;

const CHROMEDRIVER_URL: &str = "https://storage.googleapis.com/chrome-for-testing-public";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestSuite {
    Unit,
    Integration,
    E2E,
    All,
}

impl std::str::FromStr for TestSuite {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "unit" => Ok(Self::Unit),
            "integration" | "int" => Ok(Self::Integration),
            "e2e" | "end-to-end" => Ok(Self::E2E),
            "all" => Ok(Self::All),
            _ => Err(format!("Unknown test suite: {s}")),
        }
    }
}

#[derive(Debug, Clone)]
pub struct RunnerConfig {
    pub suite: TestSuite,
    pub filter: Option<String>,
    pub parallel: bool,
    pub verbose: bool,
    pub keep_env: bool,
    pub headed: bool,
}

impl Default for RunnerConfig {
    fn default() -> Self {
        Self {
            suite: TestSuite::All,
            filter: None,
            parallel: true,
            verbose: false,
            keep_env: env::var("KEEP_ENV").is_ok(),
            headed: env::var("HEADED").is_ok(),
        }
    }
}

fn print_usage() {
    eprintln!(
        r#"
BotTest - Test Runner for General Bots

USAGE:
    bottest [OPTIONS] [SUITE]

SUITES:
    unit            Run unit tests only (fast, no external services)
    integration     Run integration tests (starts real services)
    e2e             Run end-to-end browser tests
    all             Run all test suites (default)

OPTIONS:
    -f, --filter <PATTERN>    Filter tests by name pattern
    -p, --parallel            Run tests in parallel (default)
    -s, --sequential          Run tests sequentially
    -v, --verbose             Enable verbose output
    -k, --keep-env            Keep test environment after completion
    -h, --headed              Run browser tests with visible browser
    --setup                   Download and install test dependencies
    --demo                    Run a quick browser demo (no database needed)
    --help                    Show this help message

ENVIRONMENT VARIABLES:
    KEEP_ENV=1                Keep test environment for inspection
    HEADED=1                  Run browser tests with visible browser
    DATABASE_URL              Override test database URL
    TEST_THREADS              Number of parallel test threads
    SKIP_E2E_TESTS            Skip E2E tests
    SKIP_INTEGRATION_TESTS    Skip integration tests

EXAMPLES:
    bottest unit                      Run all unit tests
    bottest integration -f queue      Run integration tests matching "queue"
    bottest e2e --headed              Run E2E tests with visible browser
    bottest all -v                    Run all tests with verbose output
    bottest --setup                   Install ChromeDriver and dependencies
    bottest --demo                    Open browser and navigate to example.com
"#
    );
}

fn parse_args() -> Result<(RunnerConfig, bool, bool)> {
    let args: Vec<String> = env::args().collect();
    let mut config = RunnerConfig::default();
    let mut setup_only = false;
    let mut demo_mode = false;
    let mut i = 1;

    while i < args.len() {
        match args[i].as_str() {
            "--help" => {
                print_usage();
                std::process::exit(0);
            }
            "--setup" => {
                setup_only = true;
            }
            "--demo" => {
                demo_mode = true;
                config.headed = true;
            }
            "-f" | "--filter" => {
                i += 1;
                if i < args.len() {
                    config.filter = Some(args[i].clone());
                } else {
                    anyhow::bail!("--filter requires a pattern argument");
                }
            }
            "-p" | "--parallel" => {
                config.parallel = true;
            }
            "-s" | "--sequential" => {
                config.parallel = false;
            }
            "-v" | "--verbose" => {
                config.verbose = true;
            }
            "-k" | "--keep-env" => {
                config.keep_env = true;
            }
            "-h" | "--headed" => {
                config.headed = true;
            }
            arg if !arg.starts_with('-') => {
                config.suite = arg.parse().map_err(|e| anyhow::anyhow!("{e}"))?;
            }
            other => {
                anyhow::bail!("Unknown argument: {other}");
            }
        }
        i += 1;
    }

    Ok((config, setup_only, demo_mode))
}

fn setup_logging(verbose: bool) {
    let level = if verbose { Level::DEBUG } else { Level::INFO };

    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .with_thread_ids(false)
        .with_file(false)
        .with_line_number(false)
        .finish();

    let _ = tracing::subscriber::set_global_default(subscriber);
}

#[derive(Debug, Clone)]
pub struct TestResults {
    pub suite: String,
    pub passed: usize,
    pub failed: usize,
    pub skipped: usize,
    pub duration_ms: u64,
    pub errors: Vec<String>,
}

impl TestResults {
    #[must_use]
    pub fn new(suite: &str) -> Self {
        Self {
            suite: suite.to_string(),
            passed: 0,
            failed: 0,
            skipped: 0,
            duration_ms: 0,
            errors: Vec::new(),
        }
    }

    #[must_use]
    pub const fn success(&self) -> bool {
        self.failed == 0 && self.errors.is_empty()
    }
}

fn get_cache_dir() -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".cache").join("bottest")
}

fn get_chromedriver_path(version: &str) -> PathBuf {
    get_cache_dir().join(format!("chromedriver-{version}"))
}

fn get_chrome_path() -> PathBuf {
    get_cache_dir().join("chrome-linux64").join("chrome")
}

fn detect_existing_browser() -> Option<String> {
    let browsers = [
        "/usr/bin/brave-browser",
        "/usr/bin/brave",
        "/usr/bin/google-chrome",
        "/usr/bin/google-chrome-stable",
        "/usr/bin/chromium",
        "/usr/bin/chromium-browser",
    ];

    for browser in browsers {
        if std::path::Path::new(browser).exists() {
            return Some(browser.to_string());
        }
    }

    let chrome_path = get_chrome_path();
    if chrome_path.exists() {
        return Some(chrome_path.to_string_lossy().to_string());
    }

    None
}

fn detect_browser_version(browser_path: &str) -> Option<String> {
    let output = std::process::Command::new(browser_path)
        .arg("--version")
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let version_str = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = version_str.split_whitespace().collect();

    for part in parts {
        if part.contains('.') && part.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            let major = part.split('.').next()?;
            return Some(major.to_string());
        }
    }

    None
}

fn detect_chromedriver_for_version(major_version: &str) -> Option<PathBuf> {
    let pattern = format!("chromedriver-{major_version}");
    let cache_dir = get_cache_dir();

    if let Ok(entries) = std::fs::read_dir(&cache_dir) {
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(&pattern) && entry.path().is_file() {
                return Some(entry.path());
            }
        }
    }

    None
}

async fn download_file(url: &str, dest: &PathBuf) -> Result<()> {
    info!("Downloading: {}", url);

    let response = reqwest::get(url).await?;

    if !response.status().is_success() {
        anyhow::bail!("Download failed with status: {}", response.status());
    }

    let bytes = response.bytes().await?;

    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }

    std::fs::write(dest, &bytes)?;
    info!("Downloaded to: {:?}", dest);

    Ok(())
}

fn extract_zip(zip_path: &PathBuf, dest_dir: &PathBuf) -> Result<()> {
    info!("Extracting: {:?} to {:?}", zip_path, dest_dir);

    let file = std::fs::File::open(zip_path)?;
    let mut archive = zip::ZipArchive::new(file)?;

    std::fs::create_dir_all(dest_dir)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest_dir.join(path),
            None => continue,
        };

        if file.name().ends_with('/') {
            std::fs::create_dir_all(&outpath)?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    std::fs::create_dir_all(p)?;
                }
            }
            let mut outfile = std::fs::File::create(&outpath)?;
            std::io::copy(&mut file, &mut outfile)?;
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if let Some(mode) = file.unix_mode() {
                std::fs::set_permissions(&outpath, std::fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}

async fn get_chromedriver_version_for_browser(major_version: &str) -> Result<String> {
    let url = format!(
        "https://googlechromelabs.github.io/chrome-for-testing/LATEST_RELEASE_{major_version}"
    );

    info!("Fetching ChromeDriver version for Chrome {}", major_version);

    let response = reqwest::get(&url).await?;
    if !response.status().is_success() {
        anyhow::bail!("Failed to get ChromeDriver version: {}", response.status());
    }

    let version = response.text().await?.trim().to_string();
    info!("Found ChromeDriver version: {}", version);
    Ok(version)
}

async fn setup_chromedriver(browser_path: &str) -> Result<PathBuf> {
    let major_version = detect_browser_version(browser_path).unwrap_or_else(|| "131".to_string());

    info!("Detected browser major version: {}", major_version);

    if let Some(existing) = detect_chromedriver_for_version(&major_version) {
        info!("Found existing ChromeDriver: {:?}", existing);
        return Ok(existing);
    }

    info!(
        "ChromeDriver for version {} not found, downloading...",
        major_version
    );

    let cache_dir = get_cache_dir();
    std::fs::create_dir_all(&cache_dir)?;

    let chrome_version = get_chromedriver_version_for_browser(&major_version).await?;

    let chromedriver_url =
        format!("{CHROMEDRIVER_URL}/{chrome_version}/linux64/chromedriver-linux64.zip");

    let zip_path = cache_dir.join("chromedriver.zip");
    download_file(&chromedriver_url, &zip_path).await?;

    extract_zip(&zip_path, &cache_dir)?;

    let extracted_driver = cache_dir.join("chromedriver-linux64").join("chromedriver");
    let final_path = get_chromedriver_path(&major_version);

    if extracted_driver.exists() {
        std::fs::rename(&extracted_driver, &final_path)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&final_path, std::fs::Permissions::from_mode(0o755))?;
        }
    }

    std::fs::remove_file(&zip_path).ok();
    std::fs::remove_dir_all(cache_dir.join("chromedriver-linux64")).ok();

    if final_path.exists() {
        info!(
            "ChromeDriver {} installed: {:?}",
            chrome_version, final_path
        );
        Ok(final_path)
    } else {
        anyhow::bail!("Failed to install ChromeDriver");
    }
}

async fn setup_chrome_for_testing() -> Result<PathBuf> {
    if let Some(browser) = detect_existing_browser() {
        info!("Found existing browser: {}", browser);
        return Ok(PathBuf::from(browser));
    }

    info!("No compatible browser found, downloading Chrome for Testing...");

    let cache_dir = get_cache_dir();
    std::fs::create_dir_all(&cache_dir)?;

    let chrome_version = get_chromedriver_version_for_browser("131")
        .await
        .unwrap_or_else(|_| "131.0.6778.204".to_string());

    let chrome_url = format!("{CHROMEDRIVER_URL}/{chrome_version}/linux64/chrome-linux64.zip");

    let zip_path = cache_dir.join("chrome.zip");
    download_file(&chrome_url, &zip_path).await?;

    extract_zip(&zip_path, &cache_dir)?;

    std::fs::remove_file(&zip_path).ok();

    let chrome_path = get_chrome_path();
    if chrome_path.exists() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&chrome_path, std::fs::Permissions::from_mode(0o755))?;
        }
        info!("Chrome installed: {:?}", chrome_path);
        Ok(chrome_path)
    } else {
        anyhow::bail!("Failed to install Chrome for Testing");
    }
}

async fn setup_test_dependencies() -> Result<(PathBuf, PathBuf)> {
    info!("Setting up test dependencies...");

    let chrome = setup_chrome_for_testing().await?;
    let chrome_str = chrome.to_string_lossy().to_string();
    let chromedriver = setup_chromedriver(&chrome_str).await?;

    info!("Dependencies ready:");
    info!("  ChromeDriver: {:?}", chromedriver);
    info!("  Browser: {:?}", chrome);

    Ok((chromedriver, chrome))
}

async fn start_chromedriver(chromedriver_path: &PathBuf, port: u16) -> Result<std::process::Child> {
    info!("Starting ChromeDriver on port {}...", port);

    let child = std::process::Command::new(chromedriver_path)
        .arg(format!("--port={port}"))
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()?;

    for _ in 0..30 {
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        if check_webdriver_available(port).await {
            info!("ChromeDriver started successfully");
            return Ok(child);
        }
    }

    anyhow::bail!("ChromeDriver failed to start");
}

async fn check_webdriver_available(port: u16) -> bool {
    let url = format!("http://localhost:{port}/status");

    let Ok(client) = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(2))
        .build()
    else {
        return false;
    };

    client.get(&url).send().await.is_ok()
}

async fn run_browser_demo() -> Result<()> {
    info!("Running browser demo...");

    let debug_port = 9222u16;

    let mut browser_service = match services::BrowserService::start(debug_port).await {
        Ok(bs) => bs,
        Err(e) => {
            anyhow::bail!("Failed to start browser: {e}");
        }
    };

    info!("Browser started on CDP port {}", debug_port);

    let config = web::BrowserConfig::default()
        .with_browser(web::BrowserType::Chrome)
        .with_debug_port(debug_port)
        .headless(false)
        .with_timeout(std::time::Duration::from_secs(30));

    let browser = match web::Browser::new(config).await {
        Ok(b) => b,
        Err(e) => {
            let _ = browser_service.stop().await;
            anyhow::bail!("Failed to connect to browser CDP: {e}");
        }
    };

    info!("Browser CDP connection established!");
    info!("Navigating to example.com...");
    browser.goto("https://example.com").await?;

    info!("Waiting 5 seconds so you can see the browser...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    info!("Navigating to Google...");
    browser.goto("https://www.google.com").await?;

    info!("Waiting 5 seconds...");
    tokio::time::sleep(std::time::Duration::from_secs(5)).await;

    info!("Closing browser...");
    let _ = browser.close();
    let _ = browser_service.stop().await;

    info!("Demo complete!");
    Ok(())
}

fn discover_test_files(test_dir: &str) -> Vec<String> {
    let path = std::path::PathBuf::from(test_dir);
    if !path.exists() {
        return Vec::new();
    }

    let mut test_files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&path) {
        for entry in entries.flatten() {
            let file_path = entry.path();
            if file_path.extension().is_some_and(|e| e == "rs") {
                if let Some(name) = file_path.file_stem() {
                    let name_str = name.to_string_lossy().to_string();
                    if name_str != "mod" {
                        test_files.push(name_str);
                    }
                }
            }
        }
    }
    test_files.sort();
    test_files
}

fn run_cargo_test(
    test_type: &str,
    filter: Option<&str>,
    parallel: bool,
    env_vars: Vec<(&str, &str)>,
    features: Option<&str>,
) -> Result<(usize, usize, usize)> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("test");
    cmd.arg("-p").arg("bottest");

    if let Some(feat) = features {
        cmd.arg("--features").arg(feat);
    }

    cmd.arg("--test").arg(test_type);

    if let Some(pattern) = filter {
        cmd.arg(pattern);
    }

    cmd.arg("--");

    if !parallel {
        cmd.arg("--test-threads=1");
    }

    cmd.arg("--nocapture");

    for (key, value) in env_vars {
        cmd.env(key, value);
    }

    let output = cmd.output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");

    let mut passed = 0usize;
    let mut failed = 0usize;
    let mut skipped = 0usize;

    for line in combined.lines() {
        if line.contains("test result:") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            for (i, part) in parts.iter().enumerate() {
                if *part == "passed;" && i > 0 {
                    passed = parts[i - 1].parse().unwrap_or(0);
                }
                if *part == "failed;" && i > 0 {
                    failed = parts[i - 1].parse().unwrap_or(0);
                }
                if *part == "ignored;" && i > 0 {
                    skipped = parts[i - 1].parse().unwrap_or(0);
                }
            }
        }
    }

    Ok((passed, failed, skipped))
}

fn run_unit_tests(config: &RunnerConfig) -> Result<TestResults> {
    info!("Running unit tests...");

    let mut results = TestResults::new("unit");
    let start = std::time::Instant::now();

    let test_files = discover_test_files("tests/unit");
    if test_files.is_empty() {
        info!("No unit test files found in tests/unit/");
        results.skipped = 1;
        return Ok(results);
    }

    info!("Discovered unit test modules: {:?}", test_files);

    let filter = config.filter.as_deref();
    let env_vars: Vec<(&str, &str)> = vec![];

    match run_cargo_test("unit", filter, config.parallel, env_vars, None) {
        Ok((passed, failed, skipped)) => {
            results.passed = passed;
            results.failed = failed;
            results.skipped = skipped;
        }
        Err(e) => {
            results
                .errors
                .push(format!("Failed to run unit tests: {e}"));
            results.failed = 1;
        }
    }

    results.duration_ms = start.elapsed().as_millis() as u64;

    info!(
        "Unit tests completed: {} passed, {} failed, {} skipped ({} ms)",
        results.passed, results.failed, results.skipped, results.duration_ms
    );

    Ok(results)
}

async fn run_integration_tests(config: &RunnerConfig) -> Result<TestResults> {
    info!("Running integration tests...");

    let mut results = TestResults::new("integration");
    let start = std::time::Instant::now();

    if env::var("SKIP_INTEGRATION_TESTS").is_ok() {
        info!("Integration tests skipped (SKIP_INTEGRATION_TESTS is set)");
        results.skipped = 1;
        return Ok(results);
    }

    let test_config = TestConfig::full();
    let ctx = match TestHarness::setup(test_config).await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to set up test harness: {}", e);
            results.failed = 1;
            results.errors.push(format!("Harness setup failed: {e}"));
            return Ok(results);
        }
    };

    info!("Test harness ready:");
    info!("  PostgreSQL: {}", ctx.database_url());
    info!("  MinIO: {}", ctx.minio_endpoint());
    info!("  Redis: {}", ctx.redis_url());
    info!("  Mock Zitadel: {}", ctx.zitadel_url());
    info!("  Mock LLM: {}", ctx.llm_url());

    let test_files = discover_test_files("tests/integration");
    if test_files.is_empty() {
        info!("No integration test files found in tests/integration/");
        results.skipped = 1;
        return Ok(results);
    }

    info!("Discovered integration test modules: {:?}", test_files);

    let filter = config.filter.as_deref();
    let db_url = ctx.database_url();
    let directory_url = ctx.zitadel_url();

    let env_vars: Vec<(&str, &str)> = vec![
        ("DATABASE_URL", &db_url),
        ("DIRECTORY_URL", &directory_url),
        ("ZITADEL_CLIENT_ID", "test-client-id"),
        ("ZITADEL_CLIENT_SECRET", "test-client-secret"),
        ("DRIVE_ACCESSKEY", "minioadmin"),
        ("DRIVE_SECRET", "minioadmin"),
    ];

    match run_cargo_test(
        "integration",
        filter,
        config.parallel,
        env_vars,
        Some("integration"),
    ) {
        Ok((passed, failed, skipped)) => {
            results.passed = passed;
            results.failed = failed;
            results.skipped = skipped;
        }
        Err(e) => {
            results
                .errors
                .push(format!("Failed to run integration tests: {e}"));
            results.failed = 1;
        }
    }

    if config.keep_env {
        info!("Keeping test environment for inspection (KEEP_ENV=1)");
        info!("  Data dir: {:?}", ctx.data_dir);
    } else {
        info!("Cleaning up test environment...");
    }

    results.duration_ms = start.elapsed().as_millis() as u64;

    info!(
        "Integration tests completed: {} passed, {} failed, {} skipped ({} ms)",
        results.passed, results.failed, results.skipped, results.duration_ms
    );

    Ok(results)
}

async fn run_e2e_tests(config: &RunnerConfig) -> Result<TestResults> {
    info!("Running E2E tests...");

    let mut results = TestResults::new("e2e");
    let start = std::time::Instant::now();

    if env::var("SKIP_E2E_TESTS").is_ok() {
        info!("E2E tests skipped (SKIP_E2E_TESTS is set)");
        results.skipped = 1;
        return Ok(results);
    }

    if config.headed {
        info!("Running with visible browser (HEADED mode)");
    } else {
        info!("Running headless");
    }

    let (chromedriver_path, chrome_path) = match setup_test_dependencies().await {
        Ok(deps) => deps,
        Err(e) => {
            warn!("Failed to setup dependencies: {}", e);
            let browser = detect_existing_browser();
            if browser.is_none() {
                info!("No WebDriver available, skipping E2E tests");
                info!("Run 'bottest --setup' to install dependencies");
                results.skipped = 1;
                return Ok(results);
            }
            let browser_path = browser.unwrap();
            let major = detect_browser_version(&browser_path).unwrap_or_else(|| "131".to_string());
            if let Some(driver) = detect_chromedriver_for_version(&major) {
                (driver, PathBuf::from(browser_path))
            } else {
                info!("No matching ChromeDriver, skipping E2E tests");
                results.skipped = 1;
                return Ok(results);
            }
        }
    };

    let webdriver_port = 4444u16;
    let mut chromedriver_process = None;

    if !check_webdriver_available(webdriver_port).await {
        match start_chromedriver(&chromedriver_path, webdriver_port).await {
            Ok(child) => {
                chromedriver_process = Some(child);
            }
            Err(e) => {
                error!("Failed to start ChromeDriver: {}", e);
                results.failed = 1;
                results
                    .errors
                    .push(format!("ChromeDriver start failed: {e}"));
                return Ok(results);
            }
        }
    }

    let test_config = TestConfig::full();
    let ctx = match TestHarness::setup(test_config).await {
        Ok(c) => c,
        Err(e) => {
            error!("Failed to set up test harness: {}", e);
            if let Some(mut child) = chromedriver_process {
                let _ = child.kill();
            }
            results.failed = 1;
            results.errors.push(format!("Harness setup failed: {e}"));
            return Ok(results);
        }
    };

    info!("Test harness ready for E2E tests");

    let server = match ctx.start_botserver().await {
        Ok(s) => s,
        Err(e) => {
            error!("Failed to start botserver: {}", e);
            if let Some(mut child) = chromedriver_process {
                let _ = child.kill();
            }
            results.failed = 1;
            results.errors.push(format!("Botserver start failed: {e}"));
            return Ok(results);
        }
    };

    if server.is_running() {
        info!("Botserver started at: {}", server.url);
    } else {
        info!("Botserver not running, E2E tests may fail");
    }

    let test_files = discover_test_files("tests/e2e");
    if test_files.is_empty() {
        info!("No E2E test files found in tests/e2e/");
        if let Some(mut child) = chromedriver_process {
            let _ = child.kill();
        }
        results.skipped = 1;
        return Ok(results);
    }

    info!("Discovered E2E test modules: {:?}", test_files);

    let filter = config.filter.as_deref();
    let headed = if config.headed { "1" } else { "" };
    let db_url = ctx.database_url();
    let directory_url = ctx.zitadel_url();
    let server_url = server.url.clone();
    let chrome_binary = chrome_path.to_string_lossy().to_string();
    let webdriver_url = format!("http://localhost:{webdriver_port}");

    let env_vars: Vec<(&str, &str)> = vec![
        ("DATABASE_URL", &db_url),
        ("DIRECTORY_URL", &directory_url),
        ("ZITADEL_CLIENT_ID", "test-client-id"),
        ("ZITADEL_CLIENT_SECRET", "test-client-secret"),
        ("DRIVE_ACCESSKEY", "minioadmin"),
        ("DRIVE_SECRET", "minioadmin"),
        ("BOTSERVER_URL", &server_url),
        ("HEADED", headed),
        ("CHROME_BINARY", &chrome_binary),
        ("WEBDRIVER_URL", &webdriver_url),
    ];

    match run_cargo_test("e2e", filter, false, env_vars, Some("e2e")) {
        Ok((passed, failed, skipped)) => {
            results.passed = passed;
            results.failed = failed;
            results.skipped = skipped;
        }
        Err(e) => {
            results.errors.push(format!("Failed to run E2E tests: {e}"));
            results.failed = 1;
        }
    }

    if let Some(mut child) = chromedriver_process {
        info!("Stopping ChromeDriver...");
        let _ = child.kill();
        let _ = child.wait();
    }

    if config.keep_env {
        info!("Keeping test environment for inspection (KEEP_ENV=1)");
        info!("  Server URL: {}", server.url);
        info!("  Data dir: {:?}", ctx.data_dir);
    } else {
        info!("Cleaning up test environment...");
    }

    results.duration_ms = start.elapsed().as_millis() as u64;

    info!(
        "E2E tests completed: {} passed, {} failed, {} skipped ({} ms)",
        results.passed, results.failed, results.skipped, results.duration_ms
    );

    Ok(results)
}

fn print_summary(results: &[TestResults]) {
    println!("\n{}", "=".repeat(60));
    println!("TEST SUMMARY");
    println!("{}", "=".repeat(60));

    let mut total_passed = 0;
    let mut total_failed = 0;
    let mut total_skipped = 0;
    let mut total_duration = 0;

    for result in results {
        println!(
            "\n{} tests: {} passed, {} failed, {} skipped ({} ms)",
            result.suite, result.passed, result.failed, result.skipped, result.duration_ms
        );

        for error in &result.errors {
            println!("  ERROR: {error}");
        }

        total_passed += result.passed;
        total_failed += result.failed;
        total_skipped += result.skipped;
        total_duration += result.duration_ms;
    }

    println!("\n{}", "-".repeat(60));
    println!(
        "TOTAL: {total_passed} passed, {total_failed} failed, {total_skipped} skipped ({total_duration} ms)"
    );
    println!("{}", "=".repeat(60));

    if total_failed > 0 {
        println!("\n❌ TESTS FAILED");
    } else {
        println!("\n✅ ALL TESTS PASSED");
    }
}

#[tokio::main]
async fn main() -> ExitCode {
    let (config, setup_only, demo_mode) = match parse_args() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: {e}");
            print_usage();
            return ExitCode::from(1);
        }
    };

    setup_logging(config.verbose);

    info!(
        "BotTest - General Bots Test Suite v{}",
        env!("CARGO_PKG_VERSION")
    );

    if setup_only {
        info!("Setting up test dependencies...");
        match setup_test_dependencies().await {
            Ok((chromedriver, chrome)) => {
                println!("\n✅ Dependencies installed successfully!");
                println!("  ChromeDriver: {}", chromedriver.display());
                println!("  Browser: {}", chrome.display());
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("\n❌ Setup failed: {e}");
                return ExitCode::from(1);
            }
        }
    }

    if demo_mode {
        info!("Running browser demo...");
        match run_browser_demo().await {
            Ok(()) => {
                println!("\n✅ Browser demo completed successfully!");
                return ExitCode::SUCCESS;
            }
            Err(e) => {
                eprintln!("\n❌ Browser demo failed: {e}");
                return ExitCode::from(1);
            }
        }
    }

    info!("Running {:?} tests", config.suite);

    let start = std::time::Instant::now();
    let mut all_results = Vec::new();

    let result = match config.suite {
        TestSuite::Unit => run_unit_tests(&config),
        TestSuite::Integration => run_integration_tests(&config).await,
        TestSuite::E2E => run_e2e_tests(&config).await,
        TestSuite::All => {
            let unit = run_unit_tests(&config);
            let integration = run_integration_tests(&config).await;
            let e2e = run_e2e_tests(&config).await;

            match (unit, integration, e2e) {
                (Ok(u), Ok(i), Ok(e)) => {
                    all_results.push(u);
                    all_results.push(i);
                    all_results.push(e);
                    Ok(TestResults::new("all"))
                }
                (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => Err(e),
            }
        }
    };

    match result {
        Ok(results) => {
            if all_results.is_empty() {
                all_results.push(results);
            }
        }
        Err(e) => {
            error!("Test execution failed: {}", e);
            return ExitCode::from(1);
        }
    }

    let total_duration = start.elapsed();
    for result in &mut all_results {
        if result.duration_ms == 0 {
            result.duration_ms = total_duration.as_millis() as u64;
        }
    }

    print_summary(&all_results);

    let all_passed = all_results.iter().all(TestResults::success);
    if all_passed {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}
