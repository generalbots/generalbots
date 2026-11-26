//! Which secrets-server binary the local stack runs.
//!
//! The bundled secrets server is OpenBao (`bao`, MPL-2.0). The component keeps
//! the id `vault`, the `VAULT_*` environment and the `bin/vault`, `data/vault`
//! and `conf/vault` paths; only the program differs. A stack whose
//! `data/vault` was written by the former HashiCorp Vault binary keeps that
//! binary until its secrets are moved to OpenBao, because OpenBao cannot read
//! that storage. This module is the one place that decides between them.

use std::path::{Path, PathBuf};

/// Program directory of the secrets component, relative to the stack.
pub const BIN_DIR: &str = "bin/vault";
/// Storage directory of the secrets component, relative to the stack.
pub const DATA_DIR: &str = "data/vault";
/// Written once every secret of the former Vault storage is in OpenBao.
pub const MIGRATED_MARKER: &str = "conf/vault/openbao-migrated";
/// Present while the move to OpenBao is under way.
pub const MIGRATING_MARKER: &str = "conf/vault/openbao-migrating";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Engine {
    /// OpenBao, the bundled secrets server.
    OpenBao,
    /// The former HashiCorp Vault binary, kept for storage it wrote.
    LegacyVault,
}

impl Engine {
    /// Program name without extension.
    #[must_use]
    pub const fn binary_name(self) -> &'static str {
        match self {
            Self::OpenBao => "bao",
            Self::LegacyVault => "vault",
        }
    }

    /// Program file name on this platform.
    #[must_use]
    pub fn file_name(self) -> String {
        if cfg!(windows) {
            format!("{}.exe", self.binary_name())
        } else {
            self.binary_name().to_string()
        }
    }

    /// Program path inside `stack`.
    #[must_use]
    pub fn path_in(self, stack: &Path) -> PathBuf {
        stack.join(BIN_DIR).join(self.file_name())
    }
}

/// What the stack holds on disk, as far as the choice of binary goes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StackFacts {
    pub legacy_binary: bool,
    pub openbao_binary: bool,
    pub data_present: bool,
    pub migrated: bool,
    pub migrating: bool,
}

impl StackFacts {
    #[must_use]
    pub fn read(stack: &Path) -> Self {
        let data = stack.join(DATA_DIR);
        Self {
            legacy_binary: Engine::LegacyVault.path_in(stack).is_file(),
            openbao_binary: Engine::OpenBao.path_in(stack).is_file(),
            data_present: dir_has_entries(&data),
            migrated: stack.join(MIGRATED_MARKER).is_file(),
            migrating: stack.join(MIGRATING_MARKER).is_file(),
        }
    }

    /// The storage was written by the former Vault binary and its secrets
    /// are not in OpenBao yet. An interrupted move counts as pending: its
    /// recovery puts the former storage back first.
    #[must_use]
    pub const fn legacy_data_pending(&self) -> bool {
        self.legacy_binary && !self.migrated && (self.data_present || self.migrating)
    }

    /// A move to OpenBao was started and never finished: roll it back
    /// before anything starts the secrets server.
    #[must_use]
    pub const fn needs_recovery(&self) -> bool {
        self.migrating && !self.migrated
    }

    /// The former storage can be moved now (both programs are present).
    #[must_use]
    pub const fn can_migrate(&self) -> bool {
        self.legacy_data_pending() && self.openbao_binary
    }

    /// The binary that must serve this stack.
    #[must_use]
    pub const fn engine(&self) -> Engine {
        if self.legacy_data_pending() {
            Engine::LegacyVault
        } else {
            Engine::OpenBao
        }
    }
}

fn dir_has_entries(dir: &Path) -> bool {
    std::fs::read_dir(dir).is_ok_and(|mut entries| entries.next().is_some())
}

/// The binary that serves `stack` now.
#[must_use]
pub fn active_engine(stack: &Path) -> Engine {
    StackFacts::read(stack).engine()
}

/// The secrets CLI to run for `stack` (`bin/vault/bao[.exe]`, or the former
/// `vault[.exe]` while its storage is not migrated).
#[must_use]
pub fn cli_path(stack: &Path) -> PathBuf {
    active_engine(stack).path_in(stack)
}

/// Program name for an in-container install (`/opt/gbo/bin/<name>`).
pub const CONTAINER_CLI: &str = "/opt/gbo/bin/bao";

/// Placeholder in the component's start command for the program name
/// (without extension), resolved with [`active_engine`] when it is started.
pub const CLI_PLACEHOLDER: &str = "{{SECRETS_CLI}}";

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(legacy: bool, data: bool, migrated: bool, migrating: bool) -> StackFacts {
        StackFacts {
            legacy_binary: legacy,
            openbao_binary: true,
            data_present: data,
            migrated,
            migrating,
        }
    }

    #[test]
    fn a_fresh_stack_runs_openbao() {
        assert_eq!(facts(false, false, false, false).engine(), Engine::OpenBao);
        assert_eq!(facts(false, true, false, false).engine(), Engine::OpenBao);
    }

    #[test]
    fn unmigrated_vault_storage_keeps_the_former_binary() {
        let f = facts(true, true, false, false);
        assert!(f.legacy_data_pending());
        assert_eq!(f.engine(), Engine::LegacyVault);
    }

    #[test]
    fn migrated_storage_runs_openbao() {
        assert_eq!(facts(true, true, true, false).engine(), Engine::OpenBao);
        assert_eq!(facts(true, true, true, true).engine(), Engine::OpenBao);
        assert_eq!(facts(true, false, false, false).engine(), Engine::OpenBao);
    }

    #[test]
    fn an_interrupted_move_is_rolled_back_and_retried() {
        // Crash after `data/vault` was renamed: no data, marker present.
        let crashed = facts(true, false, false, true);
        assert!(crashed.needs_recovery());
        assert_eq!(crashed.engine(), Engine::LegacyVault);
        assert!(crashed.can_migrate());
        // Crash between the two markers: the move is done.
        let finished = facts(true, true, true, true);
        assert!(!finished.needs_recovery());
        assert!(!finished.can_migrate());
    }

    #[test]
    fn without_openbao_the_former_binary_keeps_serving() {
        let mut f = facts(true, true, false, false);
        f.openbao_binary = false;
        assert!(!f.can_migrate());
        assert_eq!(f.engine(), Engine::LegacyVault);
    }

    #[test]
    fn paths_stay_in_the_vault_component_folder() {
        let stack = Path::new("stack");
        let bao = Engine::OpenBao.path_in(stack);
        assert!(bao.starts_with(stack.join("bin/vault")));
        let name = bao.file_name().map(|n| n.to_string_lossy().into_owned());
        assert_eq!(name.as_deref(), Some(if cfg!(windows) { "bao.exe" } else { "bao" }));
        assert_eq!(Engine::LegacyVault.binary_name(), "vault");
    }

    #[test]
    fn the_start_command_names_the_engine_of_the_stack() {
        let components = crate::installer::PackageManager::all_components();
        let vault = components.get("vault");
        let cmd = vault.map(|c| c.effective_exec_cmd().to_string()).unwrap_or_default();
        assert!(cmd.contains(CLI_PLACEHOLDER), "{cmd}");
        let render = |engine: Engine| cmd.replace(CLI_PLACEHOLDER, engine.binary_name());
        let program = if cfg!(windows) { "bao.exe server" } else { "bao server" };
        assert!(render(Engine::OpenBao).contains(program));
        assert!(render(Engine::LegacyVault).contains(&program.replace("bao", "vault")));
        assert_eq!(vault.and_then(|c| c.binary_name.as_deref()), Some("bao"));
    }

    #[test]
    fn facts_are_read_from_disk() {
        let Ok(dir) = tempfile::tempdir() else { return };
        let stack = dir.path();
        assert_eq!(StackFacts::read(stack), StackFacts::default());
        let made = std::fs::create_dir_all(stack.join(BIN_DIR))
            .and_then(|()| std::fs::create_dir_all(stack.join(DATA_DIR).join("core")))
            .and_then(|()| std::fs::write(Engine::LegacyVault.path_in(stack), b"x"));
        assert!(made.is_ok());
        assert_eq!(active_engine(stack), Engine::LegacyVault);
        assert!(std::fs::create_dir_all(stack.join("conf/vault"))
            .and_then(|()| std::fs::write(stack.join(MIGRATED_MARKER), b"1"))
            .is_ok());
        assert_eq!(active_engine(stack), Engine::OpenBao);
        assert_eq!(cli_path(stack), Engine::OpenBao.path_in(stack));
    }
}
