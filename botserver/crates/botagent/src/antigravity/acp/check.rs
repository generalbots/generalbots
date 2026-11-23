//! The per-version check of `agy acp`: the protocol version the bridge
//! speaks, a session that asks before acting (a saved auto mode is switched
//! back to one that asks, or the turn is refused), and the version's record
//! of having acted without asking. The check needs no account and runs once
//! per CLI version and workspace in this server process; a turn that sees
//! an action finish unasked marks that version unsafe from then on.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock, PoisonError};

use serde_json::{json, Value};

use crate::duplex::RpcClient;

use super::proto::{self, AgentInfo, PROTOCOL_VERSION};
use super::{AcpError, REQUEST_WAIT};

/// `vm name → (version, asks)`.
fn results() -> &'static Mutex<HashMap<String, (String, bool)>> {
    static RESULTS: OnceLock<Mutex<HashMap<String, (String, bool)>>> = OnceLock::new();
    RESULTS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn remember(vm_name: &str, version: &str, asks: bool) {
    results()
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(vm_name.to_string(), (version.to_string(), asks));
}

fn cached(vm_name: &str, version: &str) -> Option<bool> {
    results()
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(vm_name)
        .filter(|(checked, _)| checked == version)
        .map(|(_, asks)| *asks)
}

/// A bridged turn saw an action run unasked: refuse this version from now on.
pub(super) fn mark_unsafe(vm_name: &str, version: &str) {
    remember(vm_name, version, false);
}

/// The answer to `initialize` is one the bridge can drive, and the version
/// was not refused before.
pub(super) fn ensure(vm_name: &str, info: &AgentInfo) -> Result<(), AcpError> {
    if info.protocol != PROTOCOL_VERSION {
        return Err(AcpError::NoAcp(format!("protocol version {} is not supported", info.protocol)));
    }
    match cached(vm_name, &info.version) {
        Some(false) => Err(AcpError::Unsafe { version: info.version.clone() }),
        Some(true) => Ok(()),
        None => {
            tracing::info!(
                "antigravity bridge: agy {} speaks ACP {} (images {}, files {}, resume {}, load {})",
                info.version,
                info.protocol,
                info.images,
                info.embedded_context,
                info.resume_session,
                info.load_session
            );
            remember(vm_name, &info.version, true);
            Ok(())
        }
    }
}

/// A session opened in a mode that acts without asking is switched to a
/// mode that asks; when none is offered or the switch fails, the turn is
/// refused instead of letting the agent act unasked.
pub(super) async fn ensure_asking_mode(
    client: &RpcClient,
    session_id: &str,
    opened: &Value,
    info: &AgentInfo,
) -> Result<(), AcpError> {
    let Some(auto) = proto::auto_mode(opened) else {
        return Ok(());
    };
    let Some(asking) = proto::asking_mode(opened) else {
        tracing::error!("antigravity bridge: session mode {auto} acts without asking and no other mode is offered");
        return Err(AcpError::Unsafe { version: info.version.clone() });
    };
    let params = json!({"sessionId": session_id, "modeId": asking});
    match client.request("session/set_mode", params, REQUEST_WAIT).await {
        Ok(_) => {
            tracing::info!("antigravity bridge: session mode {auto} switched to {asking}");
            Ok(())
        }
        Err(error) => {
            tracing::error!("antigravity bridge: session mode {auto} could not be switched ({error})");
            Err(AcpError::Unsafe { version: info.version.clone() })
        }
    }
}
