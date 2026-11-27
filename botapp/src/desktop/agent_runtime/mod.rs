//! Agent runtime: detection, provisioning and removal of the product's
//! own WSL2 distro (`GeneralBotsLite`) with Incus and the workspace image.
//!
//! Compiled under `desktop-local`.  On non-Windows or non-gbgov builds
//! every command answers `unsupported`.

pub(crate) mod commands;
mod detect;
pub(crate) mod host;
mod manifest;
pub(crate) mod provision;
mod removal;
pub(crate) mod state;

pub(crate) use commands::AgentRuntimeManager;
pub(crate) use removal::{remove as remove_runtime, RemovalResult};

pub(crate) const NSIS_UNINSTALL_HOOK: &str = removal::NSIS_HOOK;

// ── FakeHost for unit tests ─────────────────────────────────────────────

#[cfg(test)]
pub(crate) mod tests {
    use super::host::{DownloadError, RuntimeHost};
    use std::collections::HashMap;
    use std::io;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    pub(crate) struct FakeHost {
        env: HashMap<String, String>,
        arch: String,
        os_build: u32,
        wsl_policy_blocked: bool,
        wsl_installed: bool,
        virtualization: bool,
        wsl_reboot_pending: bool,
        distros: HashMap<String, String>,
        files: Mutex<HashMap<String, Vec<u8>>>,
        disk_space: u64,
        command_results: Mutex<HashMap<String, (i32, String, String)>>,
        elevated_calls: Mutex<Vec<String>>,
        elevated_fail: bool,
        download_data: Mutex<HashMap<String, Vec<u8>>>,
        download_fail: bool,
        sha256_override: Mutex<Option<String>>,
    }

    impl FakeHost {
        pub(crate) fn new() -> Self {
            Self {
                env: HashMap::new(),
                arch: "x86_64".into(),
                os_build: 22000,
                wsl_policy_blocked: false,
                wsl_installed: false,
                virtualization: true,
                wsl_reboot_pending: false,
                distros: HashMap::new(),
                files: Mutex::new(HashMap::new()),
                disk_space: 100 * 1024 * 1024 * 1024,
                command_results: Mutex::new(HashMap::new()),
                elevated_calls: Mutex::new(Vec::new()),
                elevated_fail: false,
                download_data: Mutex::new(HashMap::new()),
                download_fail: false,
                sha256_override: Mutex::new(None),
            }
        }

        pub(crate) fn with_env(mut self, name: &str, value: &str) -> Self {
            self.env.insert(name.into(), value.into());
            self
        }
        pub(crate) fn with_arch(mut self, arch: &str) -> Self {
            self.arch = arch.into(); self
        }
        pub(crate) fn with_os_build(mut self, build: u32) -> Self {
            self.os_build = build; self
        }
        pub(crate) fn with_wsl_policy_blocked(mut self, blocked: bool) -> Self {
            self.wsl_policy_blocked = blocked; self
        }
        pub(crate) fn with_wsl_installed(mut self, installed: bool) -> Self {
            self.wsl_installed = installed; self
        }
        pub(crate) fn with_virtualization(mut self, available: bool) -> Self {
            self.virtualization = available; self
        }
        pub(crate) fn with_wsl_reboot_pending(mut self, pending: bool) -> Self {
            self.wsl_reboot_pending = pending; self
        }
        pub(crate) fn with_distro(mut self, name: &str, base_path: &str) -> Self {
            self.distros.insert(name.into(), base_path.into()); self
        }
        pub(crate) fn with_file(self, path: &str, data: Vec<u8>) -> Self {
            self.files.lock().expect("lock").insert(path.into(), data); self
        }
        pub(crate) fn with_disk_space(mut self, bytes: u64) -> Self {
            self.disk_space = bytes; self
        }
        pub(crate) fn with_command_result(
            self, key: &str, code: i32, stdout: &str, stderr: &str,
        ) -> Self {
            self.command_results.lock().expect("lock")
                .insert(key.into(), (code, stdout.into(), stderr.into()));
            self
        }
        pub(crate) fn with_elevated_fail(mut self) -> Self {
            self.elevated_fail = true; self
        }
        pub(crate) fn with_download(self, url: &str, data: Vec<u8>) -> Self {
            self.download_data.lock().expect("lock").insert(url.into(), data); self
        }
        pub(crate) fn with_download_fail(mut self) -> Self {
            self.download_fail = true; self
        }
        pub(crate) fn with_sha256_override(self, digest: &str) -> Self {
            *self.sha256_override.lock().expect("lock") = Some(digest.into()); self
        }
        pub(crate) fn elevated_calls(&self) -> Vec<String> {
            self.elevated_calls.lock().expect("lock").clone()
        }
    }

    impl RuntimeHost for FakeHost {
        fn env(&self, name: &str) -> Option<String> { self.env.get(name).cloned() }
        fn arch(&self) -> &str { &self.arch }
        fn os_build(&self) -> u32 { self.os_build }
        fn wsl_policy_blocked(&self) -> bool { self.wsl_policy_blocked }
        fn wsl_installed(&self) -> bool { self.wsl_installed }
        fn virtualization_available(&self) -> bool { self.virtualization }
        fn wsl_reboot_pending(&self) -> bool { self.wsl_reboot_pending }

        fn distro_registered(&self, name: &str) -> bool {
            self.distros.contains_key(name)
        }
        fn distro_base_path(&self, name: &str) -> Option<PathBuf> {
            self.distros.get(name).map(PathBuf::from)
        }
        fn file_exists(&self, path: &Path) -> bool {
            let key = path.to_string_lossy().replace('\\', "/");
            self.files.lock().expect("lock").contains_key(&key)
        }
        fn read_file(&self, path: &Path) -> io::Result<Vec<u8>> {
            let key = path.to_string_lossy().replace('\\', "/");
            self.files.lock().expect("lock").get(&key).cloned()
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "not found"))
        }
        fn write_file(&self, path: &Path, data: &[u8]) -> io::Result<()> {
            let key = path.to_string_lossy().replace('\\', "/");
            self.files.lock().expect("lock").insert(key, data.to_vec());
            Ok(())
        }
        fn remove_file(&self, path: &Path) -> io::Result<()> {
            let key = path.to_string_lossy().replace('\\', "/");
            self.files.lock().expect("lock").remove(&key);
            Ok(())
        }
        fn create_dir_all(&self, _path: &Path) -> io::Result<()> { Ok(()) }
        fn remove_dir_all(&self, _path: &Path) -> io::Result<()> { Ok(()) }
        fn file_len(&self, path: &Path) -> io::Result<u64> {
            let key = path.to_string_lossy().replace('\\', "/");
            self.files.lock().expect("lock").get(&key)
                .map(|d| d.len() as u64)
                .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "not found"))
        }
        fn available_disk_space(&self, _path: &Path) -> io::Result<u64> {
            Ok(self.disk_space)
        }
        fn sha256_file(&self, path: &Path) -> io::Result<String> {
            if let Some(ref digest) = *self.sha256_override.lock().expect("lock") {
                return Ok(digest.clone());
            }
            use sha2::{Digest, Sha256};
            let data = self.read_file(path)?;
            let mut hasher = Sha256::new();
            for chunk in data.chunks(64 * 1024) { hasher.update(chunk); }
            Ok(hex::encode(hasher.finalize()))
        }
        fn run_command(
            &self, program: &str, _args: &[&str],
        ) -> io::Result<(i32, String, String)> {
            let results = self.command_results.lock().expect("lock");
            for (key, result) in results.iter() {
                if program.contains(key) { return Ok(result.clone()); }
            }
            Ok((0, String::new(), String::new()))
        }
        fn run_wsl_install_elevated(&self) -> io::Result<()> {
            self.elevated_calls.lock().expect("lock")
                .push("wsl --install --no-distribution".into());
            if self.elevated_fail {
                Err(io::Error::new(io::ErrorKind::PermissionDenied, "UAC denied"))
            } else {
                Ok(())
            }
        }
        fn download(
            &self, url: &str, dest: &Path, _expected_size: u64,
            on_progress: &dyn Fn(u64, u64), _cancel: &dyn Fn() -> bool,
        ) -> Result<u64, DownloadError> {
            if self.download_fail {
                return Err(DownloadError::Network("fake network error".into()));
            }
            let data = self.download_data.lock().expect("lock")
                .get(url).cloned().unwrap_or_default();
            let len = data.len() as u64;
            self.write_file(dest, &data).map_err(DownloadError::Io)?;
            on_progress(len, len);
            Ok(len)
        }
        fn fetch_bytes(&self, url: &str) -> Result<Vec<u8>, DownloadError> {
            if self.download_fail {
                return Err(DownloadError::Network("fake network error".into()));
            }
            self.download_data.lock().expect("lock")
                .get(url).cloned()
                .ok_or_else(|| DownloadError::Network(format!("no fake data for {url}")))
        }
    }
}
