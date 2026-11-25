//! Boot step before the secrets server starts: a stack that still runs the
//! former HashiCorp Vault binary gets the OpenBao program, then its storage
//! is moved to OpenBao. Any failure leaves the former Vault serving.

use botcorepkg::openbao_migration::{Layout, Migration, Outcome};
use log::{error, info, warn};
use std::path::Path;

pub async fn prepare(stack: &Path) {
    match botcorepkg::openbao_fetch::ensure_openbao_binary(stack).await {
        Ok(true) => info!("OpenBao program placed for the secrets server"),
        Ok(false) => {}
        Err(e) => warn!("OpenBao could not be placed; the former Vault keeps serving: {e}"),
    }
    let addr = std::env::var("VAULT_ADDR")
        .ok()
        .filter(|a| !a.is_empty())
        .unwrap_or_else(|| botcoresecrets::local_vault_addr().to_string());
    if !addr.starts_with("http://") {
        // The move speaks plain HTTP to the loopback desktop listener; a
        // TLS/mTLS stack keeps its former binary until it is moved by hand.
        return;
    }
    match Migration::new(stack.to_path_buf(), addr).run().await {
        Outcome::NotNeeded | Outcome::Crashed => {}
        Outcome::Migrated(report) => {
            info!("secrets moved to OpenBao: {:?}, {} policies", report.after, report.policies);
            // `load_early` set the former server token; the new root token
            // serves until the regular start makes a new scoped one.
            if let Ok(root) = Layout::new(stack).root_token() {
                std::env::set_var("VAULT_TOKEN", root);
            }
        }
        Outcome::Skipped(why) => info!("the move to OpenBao waits: {why}"),
        Outcome::RolledBack(why) => {
            warn!("the move to OpenBao failed and was rolled back; the former Vault keeps serving: {why}");
        }
        Outcome::RecoveryFailed(why) => {
            error!("the former Vault storage could not be put back and needs attention: {why}");
        }
    }
}
