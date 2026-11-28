//! Keeps the sidecar's scoped Vault token working across long idle periods.
//!
//! The root token is discarded after init (#2498), so a lapsed scoped token
//! cannot simply be re-created. At every Vault recovery the stored token is
//! checked: a valid one is renewed; a rejected or missing one is re-issued
//! with a one-time root token from Vault's `operator generate-root` flow,
//! fed with the stored unseal keys. That root token creates the new scoped
//! token and is revoked at once; it is never written anywhere. No token, OTP
//! or key is ever logged.

use anyhow::Result;
use log::{error, info, warn};
use std::path::Path;

/// What Vault said about the stored token.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TokenState {
    Valid,
    /// Vault answered and refused it (expired, revoked or unknown).
    Rejected,
    /// Vault did not answer usefully (down, sealed, network error).
    Unreachable,
    /// No token is stored.
    Missing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Action {
    Renew,
    Reissue,
    KeepAsIs,
    NoKeys,
}

pub(super) fn decide(state: TokenState, have_keys: bool) -> Action {
    match state {
        TokenState::Valid => Action::Renew,
        TokenState::Unreachable => Action::KeepAsIs,
        TokenState::Rejected | TokenState::Missing if have_keys => Action::Reissue,
        TokenState::Rejected | TokenState::Missing => Action::NoKeys,
    }
}

/// Read a failed `token lookup`: only an explicit refusal means the token is bad.
pub(super) fn classify_lookup(success: bool, stderr: &str) -> TokenState {
    if success {
        return TokenState::Valid;
    }
    let text = stderr.to_ascii_lowercase();
    let refused = ["permission denied", "code: 403", "bad token", "invalid token", "token not found"];
    if text.contains("sealed") {
        TokenState::Unreachable
    } else if refused.iter().any(|marker| text.contains(marker)) {
        TokenState::Rejected
    } else {
        TokenState::Unreachable
    }
}

/// The log line for a failed re-issue. `step` names the stage, never a value.
pub(super) fn reissue_failed_message(step: &str) -> String {
    format!(
        "The server's Vault token is no longer valid and could not be re-issued ({step}). \
         The Vault data and its unseal keys are untouched. Restart the app once Vault is \
         running to try again; if this repeats, back up the whole stack folder (its \
         conf/vault/credentials holds the unseal keys) before any reinstall."
    )
}

pub(super) const NO_KEYS_MESSAGE: &str = "The server's Vault token is missing or no longer valid and \
    no unseal keys are stored to re-issue it; the stack must be set up again (first-run setup) \
    after backing up its data folder.";

/// The Vault operations recovery needs; the CLI implements them, tests fake them.
pub(super) trait VaultOps {
    fn lookup(&self, token: &str) -> TokenState;
    fn renew(&self, token: &str) -> bool;
    /// A one-time root token, or the name of the step that failed.
    fn generate_root(&self, keys: &[String]) -> std::result::Result<String, String>;
    fn create_scoped(&self, root: &str) -> std::result::Result<String, String>;
    fn revoke_self(&self, root: &str) -> bool;
    fn store(&self, token: &str) -> std::result::Result<(), String>;
}

fn reissue<V: VaultOps>(vault: &V, keys: &[String]) -> std::result::Result<String, String> {
    let root = vault.generate_root(keys)?;
    let created = vault.create_scoped(&root);
    if vault.revoke_self(&root) {
        info!("The one-time Vault root token was revoked");
    } else {
        error!(
            "The one-time Vault root token could not be revoked. It was never stored; an \
             administrator can find it by its accessor (`token lookup -accessor`) and revoke it"
        );
    }
    let scoped = created?;
    vault.store(&scoped)?;
    Ok(scoped)
}

/// The scoped token the sidecar should use after this recovery.
pub(super) fn recover<V: VaultOps>(vault: &V, stored: Option<String>, keys: &[String]) -> Result<String> {
    let state = stored.as_deref().map_or(TokenState::Missing, |token| vault.lookup(token));
    match (decide(state, !keys.is_empty()), stored) {
        (Action::Renew, Some(token)) => {
            if vault.renew(&token) {
                info!("Scoped Vault token renewed");
            } else {
                warn!("Scoped Vault token is valid but its renewal failed; it is kept");
            }
            Ok(token)
        }
        (Action::KeepAsIs, Some(token)) => {
            warn!("Vault did not answer the token check; the stored token is kept");
            Ok(token)
        }
        (Action::Reissue, _) => {
            warn!("The stored Vault token is missing or no longer valid; re-issuing it");
            let token = reissue(vault, keys).map_err(|step| {
                let message = reissue_failed_message(&step);
                error!("{message}");
                anyhow::anyhow!(message)
            })?;
            info!("A new scoped Vault token was issued and stored");
            Ok(token)
        }
        (Action::NoKeys, _) => {
            error!("{NO_KEYS_MESSAGE}");
            Err(anyhow::anyhow!(NO_KEYS_MESSAGE))
        }
        (Action::Renew | Action::KeepAsIs, None) => {
            let message = "No scoped Vault token is stored and Vault is not answering; restart the app";
            error!("{message}");
            Err(anyhow::anyhow!(message))
        }
    }
}

/// Check, renew or re-issue the stored scoped token after Vault is unsealed,
/// and put the result into `VAULT_TOKEN` for this process.
pub fn recover_scoped_token(
    vault_addr: &str,
    ca_cert: &Path,
    vault_bin: &Path,
    unseal_keys: &[String],
) -> Result<String> {
    use super::secret_store as secrets;
    let cli = super::token_recovery_cli::VaultCli { addr: vault_addr, ca_cert, bin: vault_bin };
    let stored = secrets::read(secrets::VAULT_TOKEN_SECRET)
        .ok()
        .filter(|token| !token.trim().is_empty());
    let token = recover(&cli, stored, unseal_keys)?;
    std::env::set_var("VAULT_TOKEN", &token);
    Ok(token)
}

#[cfg(test)]
#[path = "token_recovery_tests.rs"]
mod tests;
