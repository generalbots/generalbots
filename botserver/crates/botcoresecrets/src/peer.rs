//! Peer-network settings (`secret/gbo/peer`, #1968).
//!
//! The peer node gets the same treatment as Drive/ALM/email: the canonical
//! place for its values is Vault, not the systemd unit or a checked-in file.
//! This module only *reads* — merging with environment overrides and defaults
//! belongs to the caller (`PeerConfig::load`), so the precedence rule lives in
//! exactly one place.
//!
//! Keys: `enabled`, `node-id`, `node-name`, `advertise-addr`, `data-dir`,
//! `heartbeat-ttl-secs`, `token`, `signing-key`. Values are never logged.

use crate::manager::SecretsManager;
use crate::paths::SecretPaths;
use std::collections::BTreeMap;

/// Read `secret/gbo/peer` synchronously.
///
/// Returns `None` when Vault is not configured or the read fails; the caller
/// then falls back to the `GB_PEER_*` variables, the legacy `~/.prod` file and
/// the built-in defaults. An empty map and `None` mean the same thing to the
/// caller: Vault did not supply a value.
pub fn peer_secret_sync() -> Option<BTreeMap<String, String>> {
    let manager = SecretsManager::get_clone().ok()?;
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let result = match rt {
            Ok(rt) => rt.block_on(async move {
                manager
                    .get_secret(SecretPaths::PEER)
                    .await
                    .ok()
                    .map(|secrets| secrets.into_iter().collect::<BTreeMap<_, _>>())
            }),
            Err(error) => {
                log::warn!("peer settings: could not start Vault runtime: {error}");
                None
            }
        };
        let _ = tx.send(result);
    });
    rx.recv_timeout(std::time::Duration::from_secs(5))
        .ok()
        .flatten()
}
