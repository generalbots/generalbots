//! Boot step before the secrets server starts: a stack that still runs the
//! former HashiCorp Vault binary gets the OpenBao program, then its storage
//! is moved to OpenBao. Any failure leaves the former Vault serving.

use botcorepkg::openbao_migration::{Layout, Migration, Outcome};
use log::Level;
use std::path::Path;

/// What the boot step did, for the log once logging is up (the step runs
/// before it). The lines name steps and counts only, never a secret.
#[derive(Debug, Default)]
pub struct Notes(Vec<(Level, String)>);

impl Notes {
    fn push(&mut self, level: Level, text: String) {
        self.0.push((level, text));
    }

    /// Writes the lines to the log.
    pub fn emit(self) {
        for (level, text) in self.0 {
            log::log!(target: "openbao_boot", level, "{text}");
        }
    }
}

/// Runs at the very start of the server process, before any secrets client
/// is made: a client made earlier would keep the former token and read the
/// built-in defaults instead of the real secrets. Does nothing for a remote
/// secrets server or a stack without the former binary.
pub async fn prepare(stack: &Path) -> Notes {
    let mut notes = Notes::default();
    let configured = std::env::var("VAULT_ADDR").unwrap_or_default();
    if !configured.is_empty() && !configured.contains("127.0.0.1") && !configured.contains("localhost") {
        return notes;
    }
    if !botcorepkg::secrets_engine::StackFacts::read(stack).legacy_binary {
        return notes;
    }
    match botcorepkg::openbao_fetch::ensure_openbao_binary(stack).await {
        Ok(true) => notes.push(Level::Info, "OpenBao program placed for the secrets server".into()),
        Ok(false) => {}
        Err(e) => notes.push(Level::Warn, format!("OpenBao could not be placed; the former Vault keeps serving: {e}")),
    }
    let addr = if configured.is_empty() { botcoresecrets::local_vault_addr().to_string() } else { configured };
    if !addr.starts_with("http://") {
        // The move speaks plain HTTP to the loopback desktop listener; a
        // TLS/mTLS stack keeps its former binary until it is moved by hand.
        return notes;
    }
    match Migration::new(stack.to_path_buf(), addr).run().await {
        Outcome::NotNeeded | Outcome::Crashed => {}
        Outcome::Migrated(report) => {
            notes.push(
                Level::Info,
                format!(
                    "secrets moved to OpenBao: before {:?}, after {:?}, {} policies",
                    report.before, report.after, report.policies
                ),
            );
            // The former server token was set aside; the new root token
            // serves until the regular start makes a new scoped one.
            if let Ok(root) = Layout::new(stack).root_token() {
                std::env::set_var("VAULT_TOKEN", root);
            }
        }
        Outcome::Skipped(why) => notes.push(Level::Info, format!("the move to OpenBao waits: {why}")),
        Outcome::RolledBack(why) => notes.push(
            Level::Warn,
            format!("the move to OpenBao failed and was rolled back; the former Vault keeps serving: {why}"),
        ),
        Outcome::RecoveryFailed(why) => notes.push(
            Level::Error,
            format!("the former Vault storage could not be put back and needs attention: {why}"),
        ),
    }
    notes
}
