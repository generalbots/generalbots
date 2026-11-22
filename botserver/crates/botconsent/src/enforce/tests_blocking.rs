use std::collections::HashMap;
use std::time::Instant;

use uuid::Uuid;

use crate::models::{ConsentScope, Decision, VM_ALL_CLASS};

use super::blocking::{audit_view, check_offer, is_blocking, waits_in};
use super::resolve::resolve_in_memory;
use super::{PendingMap, PendingRequest, ResolvedOutcome};

fn vm_req(id: &str, class: &str, thread: Option<&str>, protected: bool) -> PendingRequest {
    PendingRequest {
        request_id: id.to_string(),
        user_id: Uuid::nil(),
        app_id: "vm:chat:t1".to_string(),
        action_class: class.to_string(),
        detail: serde_json::json!({
            "blocking": true,
            "protected": protected,
            "command": "ls -la",
            "audit": { "command": "ls -la", "hash": "abc" },
        }),
        thread_id: thread.map(str::to_string),
        run_id: None,
        risk_tier: "medium".to_string(),
    }
}

fn pending_with(req: PendingRequest) -> (PendingMap, PendingMap) {
    let mut pending: PendingMap = HashMap::new();
    pending.insert(req.request_id.clone(), (req, Instant::now()));
    (pending, HashMap::new())
}

#[test]
fn blocking_allow_once_is_never_parked_for_another_call() {
    let (mut pending, mut approved) = pending_with(vm_req("r1", "vm_command", Some("t1"), false));
    let outcome = resolve_in_memory(&mut pending, &mut approved, "r1", Decision::AllowOnce, Instant::now())
        .expect("allow once resolves");
    assert!(matches!(outcome, ResolvedOutcome::ConsumeOnce(_)));
    assert!(approved.is_empty(), "a blocking approval must not be reusable");
}

#[test]
fn allow_all_becomes_a_conversation_grant_of_vm_all() {
    let (mut pending, mut approved) = pending_with(vm_req("r2", "vm_file_write", Some("t1"), false));
    let outcome = resolve_in_memory(&mut pending, &mut approved, "r2", Decision::AllowAll, Instant::now())
        .expect("allow all resolves");
    match outcome {
        ResolvedOutcome::PersistGrant(req, scope) => {
            assert_eq!(req.action_class, VM_ALL_CLASS);
            assert_eq!(scope, ConsentScope::Conversation);
        }
        other => panic!("unexpected outcome {other:?}"),
    }
}

#[test]
fn allow_all_is_refused_outside_the_vm_and_keeps_the_request() {
    let (mut pending, mut approved) = pending_with(vm_req("r3", "vm_drive_save", Some("t1"), false));
    let refused = resolve_in_memory(&mut pending, &mut approved, "r3", Decision::AllowAll, Instant::now());
    assert!(refused.is_err());
    assert!(pending.contains_key("r3"), "a refused choice leaves the card answerable");
    let once = resolve_in_memory(&mut pending, &mut approved, "r3", Decision::AllowConversation, Instant::now());
    assert!(matches!(once, Ok(ResolvedOutcome::PersistGrant(_, ConsentScope::Conversation))));
}

#[test]
fn protected_actions_offer_only_once_or_deny() {
    let req = vm_req("r4", "vm_command", Some("t1"), true);
    assert!(check_offer(&req, Decision::AllowOnce).is_ok());
    assert!(check_offer(&req, Decision::Deny).is_ok());
    assert!(check_offer(&req, Decision::AllowConversation).is_err());
    assert!(check_offer(&req, Decision::AllowAll).is_err());
}

#[test]
fn conversation_choices_need_a_thread_and_vm_requests_refuse_legacy_scopes() {
    let req = vm_req("r5", "vm_command", None, false);
    assert!(check_offer(&req, Decision::AllowConversation).is_err());
    assert!(check_offer(&req, Decision::AllowAll).is_err());
    let threaded = vm_req("r6", "vm_command", Some("t1"), false);
    for legacy in [Decision::AllowRun, Decision::AllowWorkspace, Decision::Always] {
        assert!(check_offer(&threaded, legacy).is_err());
    }
}

#[test]
fn audit_view_prefers_the_redacted_copy() {
    let req = vm_req("r7", "vm_command", Some("t1"), false);
    assert!(is_blocking(&req));
    assert_eq!(audit_view(&req.detail), serde_json::json!({ "command": "ls -la", "hash": "abc" }));
    let plain = serde_json::json!({ "name": "cmd" });
    assert_eq!(audit_view(&plain), plain);
}

#[test]
fn an_open_card_is_found_by_its_conversation_and_run() {
    let mut req = vm_req("r1", "vm_command", Some("t1"), false);
    req.run_id = Some("run-a".to_string());
    let (pending, _) = pending_with(req);
    assert!(waits_in(&pending, "t1", Some("run-a")));
    assert!(waits_in(&pending, "t1", None));
    assert!(!waits_in(&pending, "t1", Some("run-b")));
    assert!(!waits_in(&pending, "t2", Some("run-a")));
}

#[test]
fn a_card_that_does_not_block_a_turn_is_not_waiting() {
    let mut req = vm_req("r1", "vm_command", Some("t1"), false);
    req.detail = serde_json::json!({ "command": "ls" });
    let (pending, _) = pending_with(req);
    assert!(!waits_in(&pending, "t1", None));
    assert!(!waits_in(&HashMap::new(), "t1", None));
}
