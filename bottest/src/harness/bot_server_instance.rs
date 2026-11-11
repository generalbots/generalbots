//! Split from `harness.rs` — 326 lines.

use super::*;

impl Insertable for Bot {
    fn insert(&self, pool: &DbPool) -> Result<()> {
        use diesel::prelude::*;
        use diesel::sql_query;
        use diesel::sql_types::{Bool, Nullable, Text, Timestamptz, Uuid as DieselUuid};

        let mut conn = pool.get()?;
        sql_query(
            "INSERT INTO bots (id, name, description, kb_enabled, llm_enabled, llm_model, active, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
             ON CONFLICT (id) DO UPDATE SET name = $2, description = $3, kb_enabled = $4, llm_enabled = $5, llm_model = $6, active = $7, updated_at = $9",
        )
        .bind::<DieselUuid, _>(self.id)
        .bind::<Text, _>(&self.name)
        .bind::<Nullable<Text>, _>(&self.description)
        .bind::<Bool, _>(self.kb_enabled)
        .bind::<Bool, _>(self.llm_enabled)
        .bind::<Nullable<Text>, _>(&self.llm_model)
        .bind::<Bool, _>(self.active)
        .bind::<Timestamptz, _>(self.created_at)
        .bind::<Timestamptz, _>(self.updated_at)
        .execute(&mut conn)?;
        Ok(())
    }
}

impl Insertable for Session {
    fn insert(&self, pool: &DbPool) -> Result<()> {
        use diesel::prelude::*;
        use diesel::sql_query;
        use diesel::sql_types::{Nullable, Text, Timestamptz, Uuid as DieselUuid};

        let mut conn = pool.get()?;
        sql_query(
            "INSERT INTO sessions (id, bot_id, customer_id, channel, state, started_at, updated_at, ended_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT (id) DO UPDATE SET state = $5, updated_at = $7, ended_at = $8",
        )
        .bind::<DieselUuid, _>(self.id)
        .bind::<DieselUuid, _>(self.bot_id)
        .bind::<DieselUuid, _>(self.customer_id)
        .bind::<Text, _>(format!("{:?}", self.channel).to_lowercase())
        .bind::<Text, _>(format!("{:?}", self.state).to_lowercase())
        .bind::<Timestamptz, _>(self.started_at)
        .bind::<Timestamptz, _>(self.updated_at)
        .bind::<Nullable<Timestamptz>, _>(self.ended_at)
        .execute(&mut conn)?;
        Ok(())
    }
}

impl Insertable for Message {
    fn insert(&self, pool: &DbPool) -> Result<()> {
        use diesel::prelude::*;
        use diesel::sql_query;
        use diesel::sql_types::{Text, Timestamptz, Uuid as DieselUuid};

        let mut conn = pool.get()?;
        sql_query(
            "INSERT INTO messages (id, session_id, direction, content, content_type, timestamp)
             VALUES ($1, $2, $3, $4, $5, $6)
             ON CONFLICT (id) DO NOTHING",
        )
        .bind::<DieselUuid, _>(self.id)
        .bind::<DieselUuid, _>(self.session_id)
        .bind::<Text, _>(format!("{:?}", self.direction).to_lowercase())
        .bind::<Text, _>(&self.content)
        .bind::<Text, _>(format!("{:?}", self.content_type).to_lowercase())
        .bind::<Timestamptz, _>(self.timestamp)
        .execute(&mut conn)?;
        Ok(())
    }
}

impl Insertable for QueueEntry {
    fn insert(&self, pool: &DbPool) -> Result<()> {
        use diesel::prelude::*;
        use diesel::sql_query;
        use diesel::sql_types::{Nullable, Text, Timestamptz, Uuid as DieselUuid};

        let mut conn = pool.get()?;
        sql_query(
            "INSERT INTO queue_entries (id, customer_id, session_id, priority, status, entered_at, assigned_at, attendant_id)
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
             ON CONFLICT (id) DO UPDATE SET status = $5, assigned_at = $7, attendant_id = $8",
        )
        .bind::<DieselUuid, _>(self.id)
        .bind::<DieselUuid, _>(self.customer_id)
        .bind::<DieselUuid, _>(self.session_id)
        .bind::<Text, _>(format!("{:?}", self.priority).to_lowercase())
        .bind::<Text, _>(format!("{:?}", self.status).to_lowercase())
        .bind::<Timestamptz, _>(self.entered_at)
        .bind::<Nullable<Timestamptz>, _>(self.assigned_at)
        .bind::<Nullable<DieselUuid>, _>(self.attendant_id)
        .execute(&mut conn)?;
        Ok(())
    }
}

pub struct BotServerInstance {
    pub url: String,
    pub port: u16,
    pub stack_path: PathBuf,
    pub(crate) process: Option<std::process::Child>,
}

impl BotServerInstance {
    #[must_use]
    pub fn existing(url: &str) -> Self {
        let port = url
            .split(':')
            .next_back()
            .and_then(|p| p.parse().ok())
            .unwrap_or(8080);
        Self {
            url: url.to_string(),
            port,
            stack_path: PathBuf::from("./botserver-stack"),
            process: None,
        }
    }

    pub async fn start_with_main_stack() -> Result<Self> {
        let port = 8080;
        let url = "https://localhost:9000".to_string();

        let botserver_bin = std::env::var("BOTSERVER_BIN")
            .unwrap_or_else(|_| "../botserver/target/debug/botserver".to_string());

        if !PathBuf::from(&botserver_bin).exists() {
            log::warn!("Botserver binary not found at: {botserver_bin}");
            anyhow::bail!(
                "Botserver binary not found at: {botserver_bin}. Run: cd ../botserver && cargo build"
            );
        }

        let botserver_bin_path =
            std::fs::canonicalize(&botserver_bin).unwrap_or_else(|_| PathBuf::from(&botserver_bin));
        let botserver_dir = botserver_bin_path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(std::path::Path::to_path_buf)
            .unwrap_or_else(|| {
                std::fs::canonicalize("../botserver")
                    .unwrap_or_else(|_| PathBuf::from("../botserver"))
            });

        let stack_path = botserver_dir.join("botserver-stack");

        if !stack_path.exists() {
            anyhow::bail!(
                "Main botserver-stack not found at {}.\n\
                 Run botserver once to initialize: cd ../botserver && cargo run",
                stack_path.display()
            );
        }

        log::info!(
            "Starting botserver with MAIN stack at {}",
            stack_path.display()
        );
        println!("🚀 Starting BotServer with main stack...");
        println!("   Stack: {}", stack_path.display());

        let process = std::process::Command::new(&botserver_bin_path)
            .current_dir(&botserver_dir)
            .arg("--noconsole")
            .env_remove("RUST_LOG")
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .ok();

        if process.is_some() {
            let max_wait = 120;
            log::info!("Waiting for botserver to start (max {max_wait}s)...");

            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap_or_default();

            for i in 0..max_wait {
                if let Ok(resp) = client.get(format!("{url}/health")).send().await {
                    if resp.status().is_success() {
                        log::info!("Botserver ready on port {port}");
                        println!("   ✓ BotServer ready at {url}");
                        return Ok(Self {
                            url,
                            port,
                            stack_path,
                            process,
                        });
                    }
                }
                if i % 10 == 0 && i > 0 {
                    log::info!("Still waiting for botserver... ({i}s)");
                    println!("   ... waiting ({i}s)");
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            log::warn!("Botserver did not respond in time");
            println!("   ⚠ Botserver may not be ready");
        }

        Ok(Self {
            url,
            port,
            stack_path,
            process,
        })
    }
}

pub struct BotUIInstance {
    pub url: String,
    pub port: u16,
    pub(crate) process: Option<std::process::Child>,
}

impl BotUIInstance {
    #[must_use]
    pub fn existing(url: &str) -> Self {
        let port = url
            .split(':')
            .next_back()
            .and_then(|p| p.parse().ok())
            .unwrap_or(3000);
        Self {
            url: url.to_string(),
            port,
            process: None,
        }
    }
}

impl BotUIInstance {
    pub async fn start(botserver_url: &str) -> Result<Self> {
        let port = crate::ports::PortAllocator::allocate();
        let url = format!("http://127.0.0.1:{port}");

        let botui_bin = std::env::var("BOTUI_BIN")
            .unwrap_or_else(|_| "../botui/target/debug/botui".to_string());

        if !PathBuf::from(&botui_bin).exists() {
            log::warn!("BotUI binary not found at: {botui_bin}");
            return Ok(Self {
                url,
                port,
                process: None,
            });
        }

        let botui_bin_path =
            std::fs::canonicalize(&botui_bin).unwrap_or_else(|_| PathBuf::from(&botui_bin));
        let botui_dir = botui_bin_path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(std::path::Path::to_path_buf)
            .unwrap_or_else(|| {
                std::fs::canonicalize("../botui").unwrap_or_else(|_| PathBuf::from("../botui"))
            });

        log::info!("Starting botui from: {botui_bin} on port {port}");
        log::info!("  BOTUI_PORT={port}");
        log::info!("  BOTSERVER_URL={botserver_url}");
        log::info!("  Working directory: {}", botui_dir.display());

        let process = std::process::Command::new(&botui_bin_path)
            .current_dir(&botui_dir)
            .env("BOTUI_PORT", port.to_string())
            .env("BOTSERVER_URL", botserver_url)
            .env_remove("RUST_LOG")
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .ok();

        if process.is_some() {
            let max_wait = 30;
            log::info!("Waiting for botui to become ready... (max {max_wait}s)");
            for i in 0..max_wait {
                if let Ok(resp) = reqwest::get(&format!("{url}/health")).await {
                    if resp.status().is_success() {
                        log::info!("BotUI is ready on port {port}");
                        return Ok(Self { url, port, process });
                    }
                }
                if let Ok(resp) = reqwest::get(&url).await {
                    if resp.status().is_success() {
                        log::info!("BotUI is ready on port {port}");
                        return Ok(Self { url, port, process });
                    }
                }
                if i % 5 == 0 {
                    log::info!("Still waiting for botui... ({i}s)");
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            log::warn!("BotUI did not respond in time");
        }

        Ok(Self {
            url,
            port,
            process: None,
        })
    }

    #[must_use]
    pub const fn is_running(&self) -> bool {
        self.process.is_some()
    }
}

impl Drop for BotUIInstance {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.process {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Options for starting a desktop-local botserver profile from the e2e harness.
///
/// Mirrors the botapp sidecar contract as closely as practical: isolated stack,
/// `.product.desktop-local`, deterministic/mock LLM env wiring, and minimal
/// external coupling (no Vault/DB env leakage from the developer shell).
#[derive(Debug, Clone)]
pub struct DesktopLocalStartOptions {
    pub stack_path: PathBuf,
    pub port: u16,
    /// Full OpenAI-compatible chat completions endpoint (e.g. mock LLM).
    pub llm_chat_completions_url: Option<String>,
    /// OpenAI-compatible base URL (e.g. `http://127.0.0.1:PORT/v1`).
    pub llm_openai_base_url: Option<String>,
    pub installers_path: Option<PathBuf>,
    /// Source `.product.desktop-local` copied into `stack_path` when missing.
    pub product_source: Option<PathBuf>,
    pub botserver_bin: Option<PathBuf>,
    /// When `false` (recommended for harness/offline), sets
    /// `GB_DESKTOP_ALLOW_DOWNLOAD=0`.
    pub allow_downloads: bool,
    pub extra_env: Vec<(String, String)>,
}

impl Default for DesktopLocalStartOptions {
    fn default() -> Self {
        Self {
            stack_path: PathBuf::from("/tmp/bottest-desktop-stack"),
            port: 8080,
            llm_chat_completions_url: None,
            llm_openai_base_url: None,
            installers_path: None,
            product_source: None,
            botserver_bin: None,
            allow_downloads: false,
            extra_env: Vec::new(),
        }
    }
}

impl BotServerInstance {
    /// Start a desktop-local botserver in an isolated stack with optional mock
    /// LLM wiring. Intended for harness/e2e use; the binary must be a
    /// desktop-local-capable build when desktop-local product behavior is needed.
    ///
    /// Note: starting the server may bootstrap stack components (heavy). Callers
    /// that only need wiring/env/profile layout should use the pure helpers in
    /// `tests/e2e/desktop_local_chat.rs` instead.
    pub async fn start_desktop_local(opts: DesktopLocalStartOptions) -> Result<Self> {
        let DesktopLocalStartOptions {
            stack_path,
            port,
            llm_chat_completions_url,
            llm_openai_base_url,
            installers_path,
            product_source,
            botserver_bin,
            allow_downloads,
            extra_env,
        } = opts;

        std::fs::create_dir_all(&stack_path)?;
        let stack_path = stack_path.canonicalize().unwrap_or(stack_path);

        let product_dst = stack_path.join(".product.desktop-local");
        if !product_dst.exists() {
            let src = product_source.unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("..")
                    .join("botserver")
                    .join(".product.desktop-local")
            });
            if src.exists() {
                let _ = std::fs::copy(&src, &product_dst);
            }
        }

        let botserver_bin = botserver_bin
            .map(|p| p.display().to_string())
            .or_else(|| std::env::var("BOTSERVER_DESKTOP_LOCAL_BIN").ok())
            .or_else(|| std::env::var("BOTSERVER_BIN").ok())
            .unwrap_or_else(|| super::default_botserver_bin());

        if !PathBuf::from(&botserver_bin).exists() {
            anyhow::bail!(
                "Botserver binary not found at: {botserver_bin}. Build a desktop-local-capable botserver first."
            );
        }

        let botserver_bin_path =
            std::fs::canonicalize(&botserver_bin).unwrap_or_else(|_| PathBuf::from(&botserver_bin));
        let botserver_dir = botserver_bin_path
            .parent()
            .and_then(|p| p.parent())
            .and_then(|p| p.parent())
            .map(std::path::Path::to_path_buf)
            .unwrap_or_else(|| stack_path.clone());

        let installers_path = installers_path.unwrap_or_else(|| stack_path.join("installers"));
        let url = format!("http://127.0.0.1:{port}");

        let mut cmd = std::process::Command::new(&botserver_bin_path);
        cmd.current_dir(&botserver_dir)
            .arg("--noconsole")
            .arg("--stack-path")
            .arg(&stack_path)
            .arg("--port")
            .arg(port.to_string())
            .env_remove("RUST_LOG")
            .env("GBO_NO_DOTENV", "1")
            .env_remove("VAULT_ADDR")
            .env_remove("VAULT_TOKEN")
            .env_remove("VAULT_CACERT")
            .env_remove("VAULT_SKIP_VERIFY")
            .env_remove("VAULT_CLIENT_CERT")
            .env_remove("VAULT_CLIENT_KEY")
            .env_remove("DATABASE_URL")
            .env_remove("PG_URL")
            .env("BOTSERVER_STACK_PATH", &stack_path)
            .env("BOTSERVER_INSTALLERS_PATH", &installers_path)
            .env(
                "GB_DESKTOP_ALLOW_DOWNLOAD",
                if allow_downloads { "1" } else { "0" },
            )
            .env("AGENT_ALLOW_UNREGISTERED_IDENTITIES", "1")
            .stdout(std::process::Stdio::inherit())
            .stderr(std::process::Stdio::inherit());

        if let Some(ref completions) = llm_chat_completions_url {
            cmd.env("LLM_URL", completions);
            cmd.env("LLM_API_URL", completions);
        }
        if let Some(ref base) = llm_openai_base_url {
            cmd.env("OPENAI_API_BASE", base);
        }
        if llm_chat_completions_url.is_some() || llm_openai_base_url.is_some() {
            cmd.env("OPENAI_API_KEY", "mock-key");
            cmd.env("LLM_API_KEY", "mock-key");
        }
        for (k, v) in extra_env {
            cmd.env(k, v);
        }

        let process = cmd.spawn().ok();
        if process.is_some() {
            let max_wait = 300;
            log::info!("Waiting for desktop-local botserver to become ready (max {max_wait}s)...");
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .unwrap_or_default();
            for i in 0..max_wait {
                if let Ok(resp) = client.get(format!("{url}/health")).send().await {
                    if resp.status().is_success() {
                        log::info!("Desktop-local botserver ready at {url}");
                        return Ok(Self {
                            url,
                            port,
                            stack_path,
                            process,
                        });
                    }
                }
                // Desktop-local may expose /api/product before full health in some modes.
                if let Ok(resp) = client.get(format!("{url}/api/product")).send().await {
                    if resp.status().is_success() && i >= 5 {
                        log::info!("Desktop-local botserver responding at {url} (/api/product)");
                        return Ok(Self {
                            url,
                            port,
                            stack_path,
                            process,
                        });
                    }
                }
                if i % 15 == 0 && i > 0 {
                    log::info!("Still waiting for desktop-local botserver... ({i}s)");
                }
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            }
            log::warn!("Desktop-local botserver did not respond in time");
        }

        Ok(Self {
            url,
            port,
            stack_path,
            process: None,
        })
    }
}
