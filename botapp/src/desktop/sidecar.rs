use log::{info, warn};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Sidecar {
    child: Arc<Mutex<Option<Child>>>,
}

impl Sidecar {
    pub fn new() -> Self {
        Self {
            child: Arc::new(Mutex::new(None)),
        }
    }

    pub(crate) fn botserver_path() -> PathBuf {
        if let Ok(p) = std::env::var("BOTSERVER_BIN") {
            return PathBuf::from(p);
        }

        let exe = std::env::current_exe().unwrap_or_default();
        let dir = exe.parent().unwrap_or_else(|| std::path::Path::new("."));

        #[cfg(target_os = "windows")]
        let name = "botserver.exe";
        #[cfg(not(target_os = "windows"))]
        let name = "botserver";

        dir.join(name)
    }

    pub(crate) fn stack_path() -> PathBuf {
        if let Ok(p) = std::env::var("BOTSERVER_STACK_PATH") {
            return PathBuf::from(p);
        }
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("generalbots")
            .join("stack")
    }

    pub fn start(&self, resource_dir: Option<PathBuf>) -> anyhow::Result<()> {
        let path = Self::botserver_path();
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "botserver not found at {}",
                path.display()
            ));
        }

        let stack = Self::stack_path();
        std::fs::create_dir_all(&stack)?;

        if let Some(ref res) = resource_dir {
            let product_src = res.join(".product.desktop-local");
            let product_dst = stack.join(".product.desktop-local");
            if product_src.exists() && !product_dst.exists() {
                if let Err(e) = std::fs::copy(&product_src, &product_dst) {
                    warn!("Could not copy .product.desktop-local to stack: {e}");
                }
            }
        }

        // An explicit path wins (development runs point at the checkout's
        // bundled installers); packaged builds use the resource directory.
        let installers = std::env::var("BOTSERVER_INSTALLERS_PATH")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(std::path::PathBuf::from)
            .or_else(|| resource_dir.as_ref().map(|r| r.join("installers")))
            .unwrap_or_else(|| stack.join("installers"));

        info!(
            "Starting botserver sidecar: {} (stack: {}, installers: {})",
            path.display(),
            stack.display(),
            installers.display()
        );

        // #1620 — the sidecar output is the only place the raw model reply and
        // the tool-call parser outcome are visible on a packaged desktop run.
        // Capture both streams into the stack so a failed generation can be
        // diagnosed without a debugger or a terminal.
        let log_dir = stack.join("logs");
        std::fs::create_dir_all(&log_dir)?;
        let log_path = log_dir.join("botserver-sidecar.log");
        let log_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_path)
            .ok();

        let mut command = Command::new(&path);
        command
            .env("BOTSERVER_STACK_PATH", &stack)
            .env("BOTSERVER_INSTALLERS_PATH", &installers);
        match log_file.as_ref().and_then(|file| file.try_clone().ok()) {
            Some(stdout) => {
                command.stdout(Stdio::from(stdout));
            }
            None => {
                command.stdout(Stdio::null());
            }
        }
        match log_file.as_ref().and_then(|file| file.try_clone().ok()) {
            Some(stderr) => {
                command.stderr(Stdio::from(stderr));
            }
            None => {
                command.stderr(Stdio::null());
            }
        }

        let child = command.spawn()?;

        info!(
            "botserver sidecar spawned (pid {}), output: {}",
            child.id(),
            log_path.display()
        );
        match self.child.lock() {
            Ok(mut slot) => *slot = Some(child),
            Err(poisoned) => {
                warn!("sidecar handle mutex was poisoned; recovering it");
                *poisoned.into_inner() = Some(child);
            }
        }
        Ok(())
    }

    pub async fn wait_for_api(url: &str, timeout_secs: u64) -> bool {
        // The sidecar URL is a loopback address and uses plain HTTP; never
        // disable certificate verification for it (#security disabled-cert).
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()
        {
            Ok(c) => c,
            Err(_) => return false,
        };

        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(timeout_secs);

        loop {
            if start.elapsed() > timeout {
                warn!(
                    "Timed out after {}s waiting for API at {}",
                    timeout_secs, url
                );
                return false;
            }

            match client.get(url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    info!(
                        "API ready at {} ({:.1}s)",
                        url,
                        start.elapsed().as_secs_f64()
                    );
                    return true;
                }
                _ => {}
            }

            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    }

    pub fn stop(&self) {
        let mut slot = match self.child.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("sidecar handle mutex was poisoned while stopping; recovering it");
                poisoned.into_inner()
            }
        };
        if let Some(mut child) = slot.take() {
            info!("Stopping botserver sidecar (pid {})...", child.id());
            let _ = child.kill();
            let _ = child.wait();
            info!("botserver sidecar stopped");
        }
    }

    pub fn is_running(&self) -> bool {
        let mut guard = match self.child.lock() {
            Ok(guard) => guard,
            Err(poisoned) => {
                warn!("sidecar handle mutex was poisoned while checking state; recovering it");
                poisoned.into_inner()
            }
        };
        if let Some(ref mut child) = *guard {
            match child.try_wait() {
                Ok(None) => true,
                _ => {
                    *guard = None;
                    false
                }
            }
        } else {
            false
        }
    }
}

impl Drop for Sidecar {
    fn drop(&mut self) {
        self.stop();
    }
}
