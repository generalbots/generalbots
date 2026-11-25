//! Files of the move on disk: the markers, the former storage and the former
//! sealed secret files set aside, and putting them back. Secret files are
//! only renamed or written through [`crate::protected_file`].

use super::api::InitData;
use crate::protected_file::{read_secret, write_secret};
use crate::secrets_engine::{DATA_DIR, MIGRATED_MARKER, MIGRATING_MARKER};
use serde::{Deserialize, Serialize};
use std::io;
use std::path::{Path, PathBuf};

/// Sealed init data of the secrets server (root token, unseal keys).
pub const INIT_FILE: &str = "conf/vault/init.json";
/// Sealed unseal keys, `VAULT_UNSEAL_KEY_<n>=<key>` lines.
pub const KEYS_FILE: &str = "vault-unseal-keys";
/// Sealed scoped token of the server; void once the storage changes.
pub const TOKEN_FILE: &str = "conf/vault/server-token";
/// Config of the secrets server; OpenBao needs other storage settings.
pub const CONFIG_FILE: &str = "conf/vault/config.hcl";
const SET_ASIDE: [&str; 4] = [INIT_FILE, KEYS_FILE, TOKEN_FILE, CONFIG_FILE];

/// What the migrating marker records: no secrets, only names and processes.
#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarkerData {
    pub stamp: String,
    pub pids: Vec<u32>,
}

pub struct Layout {
    stack: PathBuf,
}

impl Layout {
    #[must_use]
    pub fn new(stack: &Path) -> Self {
        Self { stack: stack.to_path_buf() }
    }

    fn at(&self, rel: &str) -> PathBuf {
        self.stack.join(rel)
    }

    /// The former storage, kept under this name after the move.
    #[must_use]
    pub fn backup_dir(&self, stamp: &str) -> PathBuf {
        self.at(&format!("{DATA_DIR}-hashicorp-{stamp}"))
    }

    /// Folder the former sealed files are moved into.
    #[must_use]
    pub fn aside_dir(&self, stamp: &str) -> PathBuf {
        self.at(&format!("conf/vault/hashicorp-{stamp}"))
    }

    pub fn marker(&self) -> io::Result<MarkerData> {
        let text = std::fs::read_to_string(self.at(MIGRATING_MARKER))?;
        serde_json::from_str(&text).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))
    }

    fn write_marker(&self, data: &MarkerData) -> io::Result<()> {
        let path = self.at(MIGRATING_MARKER);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("tmp");
        let json = serde_json::to_string(data).map_err(io::Error::other)?;
        std::fs::write(&tmp, json).map_err(|e| step("writing the migrating marker", &tmp, e))?;
        rename_retry(&tmp, &path, "writing the migrating marker")
    }

    /// Starts the move: from here on a restart rolls it back first.
    pub fn begin(&self, stamp: &str) -> io::Result<()> {
        self.write_marker(&MarkerData { stamp: stamp.to_string(), pids: Vec::new() })
    }

    /// Records a server process started by the move, so a later recovery can
    /// stop it if this process dies.
    pub fn record_pid(&self, pid: u32) -> io::Result<()> {
        let mut data = self.marker()?;
        data.pids.push(pid);
        self.write_marker(&data)
    }

    /// The unseal keys of the current server, from the sealed file.
    pub fn unseal_keys(&self) -> io::Result<Vec<String>> {
        Ok(parse_unseal_keys(&read_secret(&self.at(KEYS_FILE))?))
    }

    /// The root token, from the sealed init data.
    pub fn root_token(&self) -> io::Result<String> {
        let text = read_secret(&self.at(INIT_FILE))?;
        let json: serde_json::Value =
            serde_json::from_str(&text).map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "init data is not JSON"))?;
        json.get("root_token")
            .and_then(serde_json::Value::as_str)
            .filter(|t| !t.is_empty())
            .map(str::to_string)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "init data has no root token"))
    }

    /// Renames the former storage to its backup name and leaves an empty
    /// storage folder.
    pub fn set_aside_storage(&self, stamp: &str) -> io::Result<()> {
        let data = self.at(DATA_DIR);
        rename_retry(&data, &self.backup_dir(stamp), "setting the former storage aside")?;
        std::fs::create_dir_all(&data).map_err(|e| step("making the new storage folder", &data, e))
    }

    /// Moves the former sealed files (and the config), as they are, into
    /// the aside folder.
    pub fn set_aside_files(&self, stamp: &str) -> io::Result<()> {
        let aside = self.aside_dir(stamp);
        std::fs::create_dir_all(&aside).map_err(|e| step("making the aside folder", &aside, e))?;
        for rel in SET_ASIDE {
            let from = self.at(rel);
            if from.is_file() {
                rename_retry(&from, &aside.join(file_name(rel)), "setting a former sealed file aside")?;
            }
        }
        Ok(())
    }

    /// Seals the new init data and unseal keys at the usual paths, in the
    /// format the regular start reads.
    pub fn write_init(&self, init: &InitData, threshold: u8) -> io::Result<()> {
        let mut json = serde_json::json!({
            "unseal_keys_b64": init.keys_b64,
            "unseal_keys_hex": init.keys_hex,
            "unseal_shares": init.keys_b64.len(),
            "unseal_threshold": threshold,
            "recovery_keys_b64": [],
            "recovery_keys_hex": [],
            "recovery_keys_shares": 0,
            "recovery_keys_threshold": 0,
            "root_token": init.root_token,
        });
        let mut text = serde_json::to_string_pretty(&json).map_err(io::Error::other)?;
        super::tree::wipe_value(&mut json);
        let written = write_secret(&self.at(INIT_FILE), &text).map_err(|e| step("sealing the new init data", &self.at(INIT_FILE), e));
        super::tree::wipe_string(&mut text);
        written?;
        let mut keys: String = init
            .keys_b64
            .iter()
            .enumerate()
            .map(|(i, key)| format!("VAULT_UNSEAL_KEY_{}={key}\n", i + 1))
            .collect();
        let written = write_secret(&self.at(KEYS_FILE), &keys).map_err(|e| step("sealing the new unseal keys", &self.at(KEYS_FILE), e));
        super::tree::wipe_string(&mut keys);
        written
    }

    /// Writes the config OpenBao starts with (no secrets in it).
    pub fn write_config(&self, text: &str) -> io::Result<()> {
        let path = self.at(CONFIG_FILE);
        std::fs::write(&path, text).map_err(|e| step("writing the OpenBao config", &path, e))
    }

    /// Ends the move: OpenBao serves this stack from now on.
    pub fn finish(&self) -> io::Result<()> {
        std::fs::write(self.at(MIGRATED_MARKER), b"1\n").map_err(|e| step("writing the migrated marker", &self.at(MIGRATED_MARKER), e))?;
        remove_if_present(&self.at(MIGRATING_MARKER))
    }

    /// Puts the former storage and sealed files back and drops the marker.
    /// Safe to repeat: each step only acts on what is still set aside.
    pub fn roll_back(&self) -> io::Result<()> {
        let marker = self.marker()?;
        let backup = self.backup_dir(&marker.stamp);
        if backup.is_dir() {
            let data = self.at(DATA_DIR);
            if data.exists() {
                remove_dir_retry(&data)?;
            }
            rename_retry(&backup, &data, "putting the former storage back")?;
        }
        let aside = self.aside_dir(&marker.stamp);
        if aside.is_dir() {
            for rel in SET_ASIDE {
                let kept = aside.join(file_name(rel));
                if kept.is_file() {
                    let target = self.at(rel);
                    remove_if_present(&target)?;
                    rename_retry(&kept, &target, "putting a former sealed file back")?;
                } else if rel == TOKEN_FILE {
                    // The former storage had no server token; drop one the
                    // new server may have been given.
                    remove_if_present(&self.at(rel))?;
                }
            }
            std::fs::remove_dir(&aside).map_err(|e| step("removing the aside folder", &aside, e))?;
        }
        remove_if_present(&self.at(MIGRATING_MARKER))
    }
}

/// How long a rename waits out a file another process still holds open
/// (antivirus or indexer scans, a server that has just exited on Windows).
const RETRY_FOR: std::time::Duration = std::time::Duration::from_secs(20);
const RETRY_EVERY: std::time::Duration = std::time::Duration::from_millis(250);

/// The error names the step and the path; paths hold no secret.
fn step(what: &str, path: &Path, e: io::Error) -> io::Error {
    io::Error::new(e.kind(), format!("{what} ({}): {e}", path.display()))
}

/// Windows refuses a rename (access denied, sharing violation) while any
/// file under the path is open without delete sharing.
fn transient(e: &io::Error) -> bool {
    e.kind() == io::ErrorKind::PermissionDenied || matches!(e.raw_os_error(), Some(5 | 32 | 33))
}

fn with_retry(what: &str, path: &Path, mut op: impl FnMut() -> io::Result<()>) -> io::Result<()> {
    let start = std::time::Instant::now();
    loop {
        match op() {
            Err(e) if transient(&e) && start.elapsed() < RETRY_FOR => std::thread::sleep(RETRY_EVERY),
            Err(e) => return Err(step(what, path, e)),
            Ok(()) => {
                if start.elapsed() > RETRY_EVERY {
                    log::info!("{what}: done after {} ms of retries", start.elapsed().as_millis());
                }
                return Ok(());
            }
        }
    }
}

fn rename_retry(from: &Path, to: &Path, what: &str) -> io::Result<()> {
    with_retry(what, from, || std::fs::rename(from, to))
}

fn remove_dir_retry(dir: &Path) -> io::Result<()> {
    with_retry("removing the partial new storage", dir, || std::fs::remove_dir_all(dir))
}

fn file_name(rel: &str) -> &str {
    rel.rsplit('/').next().unwrap_or(rel)
}

fn remove_if_present(path: &Path) -> io::Result<()> {
    match std::fs::remove_file(path) {
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            let mut perms = std::fs::metadata(path)?.permissions();
            clear_readonly(&mut perms);
            std::fs::set_permissions(path, perms)?;
            std::fs::remove_file(path)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
        other => other,
    }
}

#[cfg(unix)]
fn clear_readonly(perms: &mut std::fs::Permissions) {
    use std::os::unix::fs::PermissionsExt;
    perms.set_mode(0o600);
}

#[cfg(not(unix))]
fn clear_readonly(perms: &mut std::fs::Permissions) {
    perms.set_readonly(false);
}

/// Unseal keys in the text of the sealed keys file, in key order.
#[must_use]
pub fn parse_unseal_keys(text: &str) -> Vec<String> {
    text.lines()
        .filter_map(|line| line.trim().strip_prefix("VAULT_UNSEAL_KEY_").and_then(|rest| rest.split_once('=')))
        .map(|(_, key)| key.trim().to_string())
        .filter(|key| !key.is_empty())
        .collect()
}
