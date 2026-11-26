//! Moves the secrets of a stack written by the former HashiCorp Vault binary
//! into OpenBao, which cannot read that storage.
//!
//! The former server is started on its own storage and unsealed; every KV
//! secret of every KV mount and every ACL policy is read over the loopback
//! HTTP API into memory. The former storage and its sealed init data, keys
//! and server token and its config are set aside (never deleted), OpenBao is
//! initialised on a fresh raft storage (5 shares, threshold 3) with its init data sealed by the
//! same writer, and everything is written back and compared. Tokens are not
//! moved: the server's scoped token is made again by the regular start.
//!
//! Any failure before the `openbao-migrated` marker puts the former storage
//! and files back, so the former binary keeps serving. A process that dies
//! mid-move leaves the `openbao-migrating` marker; the next run rolls back
//! first and then tries again. Secret values never reach argv, logs or any
//! file other than the sealed init data.

mod api;
mod layout;
mod server;
mod tree;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod rehearsal;

pub use layout::{Layout, CONFIG_FILE, INIT_FILE, KEYS_FILE, TOKEN_FILE};
pub use tree::KvVersion;

use crate::secrets_engine::{Engine, StackFacts};
use anyhow::{anyhow, Result};
use log::{info, warn};
use std::path::PathBuf;
use std::process::Child;
use std::time::Duration;

const SHARES: u8 = 5;
const THRESHOLD: u8 = 3;

/// A point of the move, for rehearsing failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Exported,
    StorageSetAside,
    OpenBaoInitialised,
}

/// A failure injected by a rehearsal; production runs use none.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fault {
    /// Kill the OpenBao process after this many secrets were written.
    KillOpenBaoAfterWrites(usize),
    /// Stop at this stage as if the process had died: no rollback, servers
    /// left running.
    CrashAt(Stage),
}

pub struct Migration {
    pub stack: PathBuf,
    /// Address the stack's `config.hcl` listens on (`http://127.0.0.1:8200`).
    pub addr: String,
    pub ready_timeout: Duration,
    pub fault: Option<Fault>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Report {
    /// Secrets per mount read from the former storage.
    pub before: Vec<(String, usize)>,
    /// Secrets per mount read back from OpenBao.
    pub after: Vec<(String, usize)>,
    pub policies: usize,
    /// The OpenBao process, left running and unsealed.
    pub openbao_pid: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Nothing to move (fresh OpenBao stack, or already moved).
    NotNeeded,
    /// Not tried this time; nothing was changed.
    Skipped(String),
    Migrated(Report),
    /// The move failed and the former storage was put back.
    RolledBack(String),
    /// Stopped by [`Fault::CrashAt`].
    Crashed,
    /// Putting the former storage back failed: needs a person.
    RecoveryFailed(String),
}

impl Migration {
    #[must_use]
    pub fn new(stack: PathBuf, addr: String) -> Self {
        Self { stack, addr, ready_timeout: Duration::from_secs(60), fault: None }
    }

    /// Rolls back an interrupted move, then moves the stack if it is pending.
    pub async fn run(&self) -> Outcome {
        let layout = Layout::new(&self.stack);
        let api = match api::Api::new(&self.addr) {
            Ok(api) => api,
            Err(e) => return Outcome::Skipped(e.to_string()),
        };
        if StackFacts::read(&self.stack).needs_recovery() {
            if let Err(e) = self.recover(&layout, &api).await {
                return Outcome::RecoveryFailed(e.to_string());
            }
            warn!("an interrupted move to OpenBao was rolled back; trying again");
        }
        if !StackFacts::read(&self.stack).can_migrate() {
            return Outcome::NotNeeded;
        }
        if api.seal_status().await.is_ok() {
            return Outcome::Skipped("a secrets server is already answering; the move waits for the next start".into());
        }
        let (keys, root) = match (layout.unseal_keys(), layout.root_token()) {
            (Ok(keys), Ok(root)) if keys.len() >= usize::from(THRESHOLD) => (keys, root),
            _ => return Outcome::Skipped("the sealed init data or unseal keys cannot be read".into()),
        };
        let config = std::fs::read_to_string(self.stack.join(CONFIG_FILE)).ok();
        let Some(openbao_config) = config.as_deref().and_then(crate::openbao_config::file_storage_to_raft) else {
            return Outcome::Skipped("the secrets server config has no file storage to move".into());
        };
        let stamp = chrono::Local::now().format("%Y%m%d-%H%M%S").to_string();
        let mut run = Run { m: self, layout, api, servers: Vec::new() };
        let result = run.forward(&stamp, &openbao_config, keys, root).await;
        match result {
            Ok(Some(report)) => Outcome::Migrated(report),
            Ok(None) => Outcome::Crashed,
            Err(e) => {
                run.stop_servers();
                match run.layout.roll_back() {
                    Ok(()) => Outcome::RolledBack(e.to_string()),
                    Err(r) => Outcome::RecoveryFailed(format!("{e}; putting the former storage back failed: {r}")),
                }
            }
        }
    }

    /// Stops the servers an interrupted move left running, waits for the
    /// address to go quiet and puts the former storage back.
    async fn recover(&self, layout: &Layout, api: &api::Api) -> Result<()> {
        if let Ok(marker) = layout.marker() {
            marker.pids.into_iter().for_each(|pid| server::stop_pid(pid, &self.stack));
        }
        api.wait_silent(self.ready_timeout).await?;
        layout.roll_back()?;
        Ok(())
    }
}

struct Run<'a> {
    m: &'a Migration,
    layout: Layout,
    api: api::Api,
    servers: Vec<Child>,
}

impl Run<'_> {
    fn start(&mut self, engine: Engine) -> Result<()> {
        let child = server::start(engine, &self.m.stack)?;
        self.layout.record_pid(child.id())?;
        self.servers.push(child);
        Ok(())
    }

    fn stop_servers(&mut self) {
        self.servers.iter_mut().for_each(server::stop);
        self.servers.clear();
    }

    fn crash_at(&self, stage: Stage) -> bool {
        self.m.fault == Some(Fault::CrashAt(stage))
    }

    /// The move itself; `Ok(None)` when a rehearsal stopped it as a crash.
    async fn forward(&mut self, stamp: &str, openbao_config: &str, mut keys: Vec<String>, mut root: String) -> Result<Option<Report>> {
        self.layout.begin(stamp)?;
        self.start(Engine::LegacyVault)?;
        self.api.wait_answering(self.m.ready_timeout).await?;
        self.api.unseal(&keys[..usize::from(THRESHOLD)]).await?;
        keys.iter_mut().for_each(tree::wipe_string);
        let old = self.api.with_token(&root);
        tree::wipe_string(&mut root);
        let mounts = old.kv_mounts().await?;
        let snapshot = tree::export(&old, &mounts).await?;
        let mut policies = Vec::new();
        for name in old.policy_names().await? {
            let text = old.policy(&name).await?;
            policies.push((name, text));
        }
        let before = snapshot.counts();
        info!("secrets read from the former storage: {before:?}, {} policies", policies.len());
        if self.crash_at(Stage::Exported) {
            return Ok(None);
        }

        self.stop_servers();
        self.api.wait_silent(self.m.ready_timeout).await?;
        self.layout.set_aside_storage(stamp)?;
        self.layout.set_aside_files(stamp)?;
        self.layout.write_config(openbao_config)?;
        if self.crash_at(Stage::StorageSetAside) {
            return Ok(None);
        }

        self.start(Engine::OpenBao)?;
        let (initialized, _) = self.api.wait_answering(self.m.ready_timeout).await?;
        if initialized {
            return Err(anyhow!("OpenBao started on a storage that is already initialised"));
        }
        let init = self.api.init(SHARES, THRESHOLD).await?;
        self.layout.write_init(&init, THRESHOLD)?;
        self.api.unseal(&init.keys_b64[..usize::from(THRESHOLD)]).await?;
        let new = self.api.with_token(&init.root_token);
        drop(init);
        if self.crash_at(Stage::OpenBaoInitialised) {
            return Ok(None);
        }

        for mount in &mounts {
            new.enable_kv(mount).await?;
        }
        for (name, text) in &policies {
            new.put_policy(name, text).await?;
        }
        let kill_after = match self.m.fault {
            Some(Fault::KillOpenBaoAfterWrites(n)) => Some(n),
            _ => None,
        };
        let servers = &mut self.servers;
        tree::import(&new, &snapshot, |written| {
            if kill_after == Some(written) {
                servers.iter_mut().for_each(server::stop);
            }
        })
        .await?;
        let back = tree::export(&new, &mounts).await?;
        tree::verify(&snapshot, &back)?;
        let after = back.counts();

        // The former server token is set aside; the regular start makes a
        // new scoped token on OpenBao.
        self.layout.finish()?;
        let openbao_pid = self.servers.last().map_or(0, Child::id);
        self.servers.clear();
        info!("secrets moved to OpenBao: {after:?}");
        Ok(Some(Report { before, after, policies: policies.len(), openbao_pid }))
    }
}
