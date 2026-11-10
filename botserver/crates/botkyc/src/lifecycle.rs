//! Pure decision core of the KYC surface.
//!
//! The biometric crate owns the state machine ([`botbiometry::kyc::KycState`]);
//! this module maps a requested wire status onto it, refuses to reopen a
//! terminal case, and reads the workflow's `steps_completed` JSONB, so the
//! HTTP handlers only translate an outcome into a status code.

use botbiometry::kyc::KycState;
use serde_json::Value;

/// Resolves a requested wire status, rejecting anything the state machine does
/// not know. The comparison is case-sensitive: the wire format is lowercase.
pub fn resolve_requested(status: &str) -> Result<KycState, String> {
    KycState::from_status(status).ok_or_else(|| format!("Unknown KYC status '{status}'"))
}

/// Rejects a transition that would reopen a terminal case.
///
/// The stored status is parsed leniently: a row written before the state
/// machine existed (or by an older build) carries a status this crate does not
/// know, and such a row must stay editable instead of blocking every update.
/// Re-applying the same terminal status is a no-op, not a conflict.
pub fn check_transition(current: &str, requested: KycState) -> Result<(), String> {
    let Some(current_state) = KycState::from_status(current) else {
        return Ok(());
    };
    if current_state.is_terminal() && current_state != requested {
        return Err(format!(
            "terminal ({}) and cannot transition to '{}'",
            current_state.as_status(),
            requested.as_status()
        ));
    }
    Ok(())
}

/// Documents completed in a workflow's `steps_completed` JSONB array.
///
/// Only string elements count: the column is written by several builds, so a
/// nested object or a number must not become a document name.
pub fn documents_from_steps(steps: &Value) -> Vec<String> {
    steps
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
#[path = "lifecycle_tests.rs"]
mod lifecycle_tests;
