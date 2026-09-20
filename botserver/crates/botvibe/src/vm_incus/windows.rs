//! `vm_incus::windows` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

/// Working directory for driver commands: `/tmp` on Linux, the OS temp dir
/// on Windows (where `/tmp` does not exist).
pub(crate) fn driver_cwd() -> std::path::PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::temp_dir()
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::path::PathBuf::from("/tmp")
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn wsl_distro() -> String {
    std::env::var("GBO_WSL_DISTRO")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "Debian".to_string())
}

#[cfg(target_os = "windows")]
pub(crate) fn wsl_exec_args(command: &[String]) -> Vec<String> {
    let mut args = vec![
        "-d".to_string(),
        wsl_distro(),
        "-u".to_string(),
        "root".to_string(),
        "--".to_string(),
    ];
    args.extend(command.iter().cloned());
    args
}

#[cfg(target_os = "windows")]
pub(crate) fn ensure_wsl_keepalive() -> Result<(), String> {
    static KEEPALIVE: OnceLock<std::sync::Mutex<Option<std::process::Child>>> = OnceLock::new();
    let mut child = KEEPALIVE
        .get_or_init(|| std::sync::Mutex::new(None))
        .lock()
        .map_err(|_| "WSL keepalive lock poisoned".to_string())?;
    if let Some(existing) = child.as_mut() {
        match existing.try_wait() {
            Ok(None) => return Ok(()),
            Ok(Some(_)) => *child = None,
            Err(e) => return Err(format!("inspect WSL keepalive: {e}")),
        }
    }
    let command = ["/bin/sleep".to_string(), "infinity".to_string()];
    let spawned = spawn_persistent("wsl", &wsl_exec_args(&command), &driver_cwd())
        .map_err(|e| format!("start Debian WSL keepalive: {e}"))?;
    *child = Some(spawned);
    Ok(())
}

#[cfg(target_os = "windows")]
pub(crate) fn windows_path_to_wsl(path: &std::path::Path) -> Result<String, String> {
    let text = path.to_string_lossy().replace('\\', "/");
    let bytes = text.as_bytes();
    if bytes.len() < 3 || bytes[1] != b':' || bytes[2] != b'/' {
        return Err(format!("cannot map Windows path into WSL: {text}"));
    }
    let drive = (bytes[0] as char).to_ascii_lowercase();
    Ok(format!("/mnt/{drive}/{}", &text[3..]))
}

impl VmLifecycle {
    /// Run an `incus` invocation on the current platform.
    pub(crate) fn incus_run(&self, args: &[String], timeout: u64) -> Result<RunOutput, GuardError> {
        #[cfg(target_os = "windows")]
        {
            let mut command = vec!["incus".to_string()];
            command.extend(args.iter().cloned());
            checked_run("wsl", &wsl_exec_args(&command), &driver_cwd(), timeout)
        }
        #[cfg(not(target_os = "windows"))]
        {
            checked_run("incus", args, &driver_cwd(), timeout)
        }
    }

    /// Provision WSL2 + the selected distro + Incus automatically (Windows only).
    #[cfg(target_os = "windows")]
    pub(crate) fn ensure_incus_wsl(&self) -> Result<(), String> {
        let cwd = driver_cwd();
        let distro = wsl_distro();

        if checked_run("wsl", &["--status".to_string()], &cwd, 30).is_err() {
            checked_run(
                "wsl",
                &["--set-default-version".to_string(), "2".to_string()],
                &cwd,
                30,
            )
            .ok();
        }

        if checked_run("wsl", &wsl_exec_args(&["true".to_string()]), &cwd, 30).is_err() {
            checked_run(
                "wsl",
                &[
                    "--install".to_string(),
                    "-d".to_string(),
                    distro.clone(),
                    "--no-launch".to_string(),
                ],
                &cwd,
                600,
            )
            .map_err(|e| {
                format!("wsl --install -d {distro} failed: {e} (a reboot may be required)")
            })?;
        }

        if checked_run(
            "wsl",
            &wsl_exec_args(&["incus".to_string(), "version".to_string()]),
            &cwd,
            30,
        )
        .is_ok()
        {
            return ensure_wsl_keepalive();
        }

        checked_run(
            "wsl",
            &wsl_exec_args(&["apt-get".to_string(), "update".to_string()]),
            &cwd,
            300,
        )
        .map_err(|e| format!("apt-get update inside {distro} failed: {e}"))?;
        checked_run(
            "wsl",
            &wsl_exec_args(&[
                "apt-get".to_string(),
                "install".to_string(),
                "-y".to_string(),
                "incus".to_string(),
            ]),
            &cwd,
            600,
        )
        .map_err(|e| format!("apt-get install incus inside {distro} failed: {e}"))?;
        checked_run(
            "wsl",
            &wsl_exec_args(&[
                "systemctl".to_string(),
                "enable".to_string(),
                "--now".to_string(),
                "incus".to_string(),
            ]),
            &cwd,
            120,
        )
        .map_err(|e| format!("starting Incus inside {distro} failed: {e}"))?;

        checked_run(
            "wsl",
            &wsl_exec_args(&[
                "incus".to_string(),
                "admin".to_string(),
                "init".to_string(),
                "--minimal".to_string(),
            ]),
            &cwd,
            300,
        )
        .map_err(|e| format!("incus admin init --minimal failed in {distro}: {e}"))?;
        ensure_wsl_keepalive()
    }

    pub(crate) fn linux_available(&self) -> bool {
        if std::env::var("VIBE_INCUS_FORCE_UNAVAILABLE").as_deref() == Ok("1") {
            return false;
        }
        #[cfg(target_os = "windows")]
        {
            static WSL_INCUS_READY: OnceLock<bool> = OnceLock::new();
            *WSL_INCUS_READY.get_or_init(|| {
                let _ = self.ensure_incus_wsl();
                checked_run(
                    "wsl",
                    &wsl_exec_args(&["incus".to_string(), "version".to_string()]),
                    &driver_cwd(),
                    30,
                )
                .is_ok()
            })
        }
        #[cfg(not(target_os = "windows"))]
        {
            checked_run("incus", &["version".to_string()], &driver_cwd(), 5).is_ok()
        }
    }

    /// Resolve the container's primary IPv4 address from `incus list` JSON
    /// (`state.network.eth0.addresses[].address` where family=inet). The host
    /// cannot resolve `{container}.incus` DNS names, so the health probe must
    /// dial the real IP. On Windows/WSL2 the address is only reachable from
    /// inside the WSL2 VM unless a proxy device is configured.
    pub(crate) fn linux_ip(&self, name: &str) -> Result<Option<String>, String> {
        self.skip_if_unavailable()?;
        let list = self.linux_list()?;
        Ok(list
            .as_array()
            .and_then(|arr| arr.iter().find(|i| i["name"].as_str() == Some(name)))
            .and_then(|i| i["state"]["network"]["eth0"]["addresses"].as_array())
            .and_then(|addrs| {
                addrs
                    .iter()
                    .find(|a| a["family"].as_str() == Some("inet"))
                    .and_then(|a| a["address"].as_str().map(str::to_string))
            }))
    }
}
