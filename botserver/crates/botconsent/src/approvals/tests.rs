//! Test module root. Workflow regressions and #2901 security tests live in
//! the `tests/` submodule directory so each file stays under 450 lines.

use chrono::{Duration, Utc};
use uuid::Uuid;

use super::engine;
use super::store::ApprovalStore;
use super::types::{
    ApprovalStatus, ApprovalStep, CallerPrincipal, DecisionKind, Delegation, SlaEventKind,
    StepKind, StepMode,
};

mod security;
mod workflow;

const TENANT_A: &str = "tenant-a";
const TENANT_B: &str = "tenant-b";

fn step(
    id: &str,
    mode: StepMode,
    kind: StepKind,
    approvers: &[&str],
    sla_secs: i64,
) -> ApprovalStep {
    ApprovalStep {
        id: id.to_string(),
        title: id.to_string(),
        mode,
        kind,
        approvers: approvers.iter().map(|name| (*name).to_string()).collect(),
        groups: Vec::new(),
        sla_secs,
        escalate_to: Some("chief-of-staff".to_string()),
        quorum: 0,
    }
}

fn contract_route() -> Vec<ApprovalStep> {
    vec![
        step(
            "reviewers",
            StepMode::Parallel,
            StepKind::Review,
            &["alice", "bob"],
            0,
        ),
        step("cfo", StepMode::Sequential, StepKind::Approval, &["cfo"], 0),
        step(
            "esign",
            StepMode::Sequential,
            StepKind::Signature,
            &["signer"],
            0,
        ),
    ]
}

fn user(n: u8) -> Uuid {
    let mut bytes = [0u8; 16];
    bytes[15] = n;
    Uuid::from_bytes(bytes)
}

fn store_with_contract() -> (ApprovalStore, uuid::Uuid) {
    let store = ApprovalStore::new();
    let record = store
        .create(
            TENANT_A,
            "contract-q3",
            "contract",
            "Q3 contract",
            "author",
            "v7 content",
        )
        .map_err(|e| e)
        .expect("create");
    store
        .set_route(TENANT_A, record.id, contract_route(), "Please review", None)
        .map_err(|e| e)
        .expect("route");
    (store, record.id)
}
