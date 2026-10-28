use log::{error, info, warn};
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

    fn botserver_path() -> PathBuf {
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

    fn stack_path() -> PathBuf {
        if let Ok(p) = std::env::var("BOTSERVER_STACK_PATH") {
            return PathBuf::from(p);
        }
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("generalbots")
            .join("stack")
    }

    pub fn start(&self) -> anyhow::Result<()> {
        let path = Self::botserver_path();
        if !path.exists() {
            return Err(anyhow::anyhow!(
                "botserver not found at {}",
                path.display()
            ));
        }

        let stack = Self::stack_path();
        std::fs::create_dir_all(&stack)?;

        info!(
            "Starting botserver sidecar: {} (stack: {})",
            path.display(),
            stack.display()
        );

        let child = Command::new(&path)
            .env("BOTSERVER_STACK_PATH", &stack)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        info!("botserver sidecar spawned (pid {})", child.id());
        *self.child.lock().unwrap() = Some(child);
        Ok(())
    }

    pub async fn wait_for_api(url: &str, timeout_secs: u64) -> bool {
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .danger_accept_invalid_certs(true)
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
        if let Some(mut child) = self.child.lock().unwrap().take() {
            info!("Stopping botserver sidecar (pid {})...", child.id());
            let _ = child.kill();
            let _ = child.wait();
            info!("botserver sidecar stopped");
        }
    }

    pub fn is_running(&self) -> bool {
        let mut guard = self.child.lock().unwrap();
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
