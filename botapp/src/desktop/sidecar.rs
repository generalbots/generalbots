use log::{info, warn};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};

mod bootlog;
mod paths;
mod probe;
mod product;

use bootlog::{file_inode, log_was_recreated, write_boot_banner};
use product::sync_product_file;
use probe::{
    api_is_answering, clear_pid, decide_start, pid_is_alive, pid_looks_like_botserver,
    port_is_open, record_pid, recorded_pid, terminate_pid, StartDecision, API_PRODUCT_URL,
    SIDECAR_PORT,
};

/// Owns the child process handle. Kept behind an `Arc` so that dropping a clone
/// of [`Sidecar`] (for example a temporary taken from Tauri's managed state)
/// does not kill a sidecar another handle is still using.
struct SidecarInner {
    child: Mutex<Option<Child>>,
}

impl Drop for SidecarInner {
    fn drop(&mut self) {
        let child = match self.child.lock() {
            Ok(mut slot) => slot.take(),
            Err(poisoned) => poisoned.into_inner().take(),
        };
        if let Some(mut child) = child {
            info!("Stopping botserver sidecar (pid {})...", child.id());
            let _ = child.kill();
            let _ = child.wait();
            // The pid file described this child; the assistant is gone now.
            clear_pid();
            info!("botserver sidecar stopped");
        }
    }
}

#[derive(Clone)]
pub struct Sidecar {
    inner: Arc<SidecarInner>,
}

impl Sidecar {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(SidecarInner {
                child: Mutex::new(None),
            }),
        }
    }

    pub(crate) fn botserver_path() -> PathBuf {
        let explicit = std::env::var("BOTSERVER_BIN").ok();
        let beside = std::env::current_exe()
            .ok()
            .and_then(|exe| exe.parent().map(|dir| dir.join(paths::BINARY_NAME)))
            .filter(|path| path.exists());
        paths::resolve_binary(explicit.as_deref(), beside, &Self::stack_path())
    }

    pub(crate) fn stack_path() -> PathBuf {
        let explicit = std::env::var("BOTSERVER_STACK_PATH").ok();
        paths::resolve_stack(explicit.as_deref(), dirs::data_local_dir())
    }

    #[cfg(target_os = "windows")]
    pub(crate) fn prepare_windows_runtime(installers: &std::path::Path) -> anyhow::Result<std::ffi::OsString> {
        let stack = Self::stack_path();
        // Diesel's PostgreSQL client is dynamically linked on Windows.
        // Keep libpq and its runtime dependencies beside botserver.exe so
        // Windows can resolve them without relying on the build machine's
        // PostgreSQL installation or inherited PATH.
        let runtime_dlls = [
            "libpq.dll",
            "libcrypto-3-x64.dll",
            "libssl-3-x64.dll",
            "libiconv-2.dll",
            "libintl-9.dll",
            "libwinpthread-1.dll",
            "liblz4.dll",
            "libzstd.dll",
            "zlib1.dll",
        ];
        for name in runtime_dlls {
            let destination = stack.join(name);
            let source = installers.join(name);
            if source.is_file() && source != destination {
                std::fs::copy(&source, &destination).map_err(|e| {
                    anyhow::anyhow!("could not stage PostgreSQL runtime file {name}: {e}")
                })?;
            } else if !destination.is_file() {
                return Err(anyhow::anyhow!(
                    "PostgreSQL runtime file {name} is missing from the desktop bundle"
                ));
            }
        }
        let mut paths = vec![stack];
        paths.extend(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default()));
        Ok(std::env::join_paths(paths)?)
    }

    pub fn start(&self, resource_dir: Option<PathBuf>) -> anyhow::Result<()> {
        // #2023 — idempotent start. Two spawners race on first boot (the app's
        // startup sidecar and the wizard's `start_assistant`), and a sidecar
        // from a previous app run can still own port 8080. Attach in every one
        // of those cases instead of booting a duplicate assistant.
        let child_running = self.is_running();
        let api_answering = !child_running && api_is_answering();
        let port_open = !child_running && !api_answering && port_is_open();
        match decide_start(child_running, api_answering, port_open) {
            StartDecision::ReuseChild => {
                info!("botserver sidecar is already running; reusing the live process");
                return Ok(());
            }
            StartDecision::AttachApi => {
                info!("an assistant is already serving {API_PRODUCT_URL}; attaching to it");
                return Ok(());
            }
            StartDecision::AttachPort => {
                info!(
                    "port {SIDECAR_PORT} is already held by an assistant that is still booting; attaching to it instead of starting a second one"
                );
                return Ok(());
            }
            StartDecision::Spawn => {}
        }

        let path = Self::botserver_path();
        if !path.exists() {
            return Err(anyhow::anyhow!("botserver not found at {}", path.display()));
        }

        let stack = Self::stack_path();
        std::fs::create_dir_all(&stack)?;

        // Seed or refresh the product file in the stack directory. On an
        // upgrade the bundled file may carry new apps or keys; compare content
        // and replace the stale copy, keeping the previous version so the user
        // can inspect what changed.
        if let Some(ref res) = resource_dir {
            let product_src = res.join(".product.desktop-local");
            let product_dst = stack.join(".product.desktop-local");
            sync_product_file(&product_src, &product_dst);
        }

        // Development runs are built from the checkout, where Tauri's resource
        // directory does not contain the product file. Seed the stack from the
        // repository so the desktop-local profile (apps, no auth bypass,
        // taskbar search) is what the sidecar actually loads.
        let stack_product = stack.join(".product.desktop-local");
        if !stack_product.exists() {
            let dev_product = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("..")
                .join("botserver")
                .join(".product.desktop-local");
            sync_product_file(&dev_product, &stack_product);
        }

        // Packaged builds ship the suite next to this executable; the sidecar
        // serves it so the page and `/api` share the `http://127.0.0.1:8080`
        // origin. An explicit BOTUI_PATH still wins.
        let packaged_ui = resource_dir
            .as_ref()
            .map(|dir| dir.join("ui").join("suite"))
            .filter(|dir| dir.exists())
            .or_else(|| {
                let stack_ui = stack.join("ui").join("suite");
                stack_ui.exists().then_some(stack_ui)
            });

        // An explicit path wins (development runs point at the checkout's
        // bundled installers); a packaged MSI/NSIS install uses the resource
        // directory, and the self-extracting executable uses the payload that
        // was extracted into the stack. Only select a directory that exists:
        // the self-extractor has no resource installers/, and preferring a
        // missing path there hid the bundled stack/installers (#1793).
        let installers = std::env::var("BOTSERVER_INSTALLERS_PATH")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .map(std::path::PathBuf::from)
            .or_else(|| {
                resource_dir
                    .as_ref()
                    .map(|r| r.join("installers"))
                    .filter(|dir| dir.exists())
            })
            .or_else(|| {
                let dir = stack.join("installers");
                dir.exists().then_some(dir)
            })
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

        // #1895 — this file is the only diagnostic channel on a packaged
        // desktop install, and it must be provable that the running sidecar
        // writes to it. Stamp every boot, remember the inode the sidecar is
        // appending to, and warn if the path is re-created behind its back
        // (a rotation that truncates leaves the process appending to an
        // orphaned inode: the log never updates again).
        if let Some(ref file) = log_file {
            write_boot_banner(file, &log_path);
        }
        let opened_inode = file_inode(&log_path);

        let mut command = Command::new(&path);
        #[cfg(target_os = "windows")]
        command.env("PATH", Self::prepare_windows_runtime(&installers)?);
        // The default build carries the `console` feature, so without this flag
        // the sidecar starts the ratatui TUI: its logs go to the (invisible)
        // TUI instead of the sidecar log, and a hidden/redirected process can
        // wedge on terminal setup. The desktop sidecar must run headless.
        command
            .arg("--noconsole")
            .current_dir(&stack)
            .env_remove("DATABASE_URL")
            .env_remove("PG_URL")
            .env("BOTSERVER_STACK_PATH", &stack)
            .env("BOTSERVER_INSTALLERS_PATH", &installers)
            // The desktop bundle carries only the small vendored components
            // (drive on Linux/Windows, plus a few Windows services). Everything
            // else is fetched once unless the user asked for an air-gapped run.
            // Without this the Windows first run failed at the first component
            // that was not bundled instead of downloading it.
            .env(
                "GB_DESKTOP_ALLOW_DOWNLOAD",
                if std::env::var("GB_DESKTOP_OFFLINE").as_deref() == Ok("1") {
                    "0"
                } else {
                    "1"
                },
            )
            // Single-user desktop: there is no sign-in screen, so the local
            // identity must be allowed to provision the agent workspace.
            .env("AGENT_ALLOW_UNREGISTERED_IDENTITIES", "1");
        if let Some(ref ui) = packaged_ui {
            command.env("BOTUI_PATH", ui);
        }
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

        // #1895 — a path whose inode changed after spawn means every later
        // line from this sidecar is lost in an unlinked file. Say so loudly;
        // the launcher keeps a stable inode by opening in append mode, so a
        // mismatch is a real bug rather than a normal rotation.
        if log_was_recreated(opened_inode, file_inode(&log_path)) {
            warn!(
                "sidecar log {} was re-created after the sidecar opened it — output is going to an orphaned file; never truncate this path while the sidecar runs",
                log_path.display()
            );
        }
        let pid = child.id();
        match self.inner.child.lock() {
            Ok(mut slot) => *slot = Some(child),
            Err(poisoned) => {
                warn!("sidecar handle mutex was poisoned; recovering it");
                *poisoned.into_inner() = Some(child);
            }
        }
        record_pid(pid);
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

    /// Stops the assistant, whether this app spawned it or not.
    ///
    /// #2023 — the wizard's one intentional restart (first boot: the auth
    /// service needs `directory_config.json`, which only exists after the first
    /// boot) must be able to replace an assistant this handle does not own —
    /// one left by a previous app run, or one attached to through the
    /// idempotent `start`.
    pub fn stop(&self) {
        let mut stopped = false;
        {
            let mut slot = match self.inner.child.lock() {
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
                stopped = true;
            }
        }

        let recorded = if stopped { None } else { recorded_pid() };
        if let Some(pid) = recorded {
            // Only signal the recorded pid while something actually holds the
            // sidecar port: a recycled pid must never be killed.
            if port_is_open() && pid_is_alive(pid) && pid_looks_like_botserver(pid) {
                info!("Stopping the attached botserver sidecar (pid {pid})...");
                stopped = terminate_pid(pid);
            } else {
                info!(
                    "recorded sidecar pid {pid} is not serving port {SIDECAR_PORT}; nothing to stop"
                );
            }
        }

        clear_pid();
        if !stopped {
            return;
        }
        // Wait (bounded) for the socket to be released so an immediate `start`
        // can bind instead of attaching to the process being torn down.
        for _ in 0..15 {
            if !port_is_open() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(200));
        }
        info!("botserver sidecar stopped");
    }

    pub fn is_running(&self) -> bool {
        let mut guard = match self.inner.child.lock() {
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
