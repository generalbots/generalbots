//! Who may answer a request, when a waiting request is withdrawn, and what a
//! deleted conversation takes with it.

use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::oneshot::error::TryRecvError;
use uuid::Uuid;

use crate::models::Decision;

use super::blocking::{turn_superseded, withdraw_thread};
use super::resolve::{check_owner, resolve, Resolver, NOT_OWNER};
use super::{ConsentService, PendingRequest};

fn owner() -> Uuid {
    Uuid::from_u128(1)
}

fn vm_req(id: &str, thread: &str) -> PendingRequest {
    PendingRequest {
        request_id: id.to_string(),
        user_id: owner(),
        app_id: format!("vm:chat:{thread}"),
        action_class: "vm_command".to_string(),
        detail: serde_json::json!({ "blocking": true, "protected": false }),
        thread_id: Some(thread.to_string()),
        run_id: Some("run-1".to_string()),
        risk_tier: "medium".to_string(),
    }
}

/// A service whose pool never connects: the paths under test return before
/// any database access.
fn service() -> Arc<ConsentService> {
    let manager = diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new("postgres://unused.invalid/none");
    let pool = diesel::r2d2::Pool::builder()
        .connection_timeout(Duration::from_millis(50))
        .build_unchecked(manager);
    Arc::new(ConsentService::new(pool))
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_time()
        .build()
        .expect("test runtime")
}

#[test]
fn only_the_owner_may_answer() {
    let req = vm_req("r1", "t1");
    let me = Resolver { user_id: owner(), thread_id: None };
    let other = Resolver { user_id: Uuid::from_u128(2), thread_id: Some("t1") };
    assert!(check_owner(&req, Decision::AllowOnce, me).is_ok());
    for decision in [Decision::AllowOnce, Decision::Deny, Decision::AllowAll] {
        let refused = check_owner(&req, decision, other).expect_err("another user is refused");
        assert!(refused.contains(NOT_OWNER));
    }
}

#[test]
fn conversation_choices_come_from_the_conversation_that_asked() {
    let req = vm_req("r1", "t1");
    let elsewhere = Resolver { user_id: owner(), thread_id: Some("t2") };
    let here = Resolver { user_id: owner(), thread_id: Some("t1") };
    assert!(check_owner(&req, Decision::AllowConversation, elsewhere).is_err());
    assert!(check_owner(&req, Decision::AllowAll, elsewhere).is_err());
    // Once and deny cover only this action, wherever the card is answered.
    assert!(check_owner(&req, Decision::AllowOnce, elsewhere).is_ok());
    assert!(check_owner(&req, Decision::Deny, elsewhere).is_ok());
    assert!(check_owner(&req, Decision::AllowConversation, here).is_ok());
    assert!(check_owner(&req, Decision::AllowAll, here).is_ok());
}

#[test]
fn a_refused_decision_leaves_the_request_pending_and_its_waiter_waiting() {
    let rt = runtime();
    rt.block_on(async {
        let svc = service();
        let (tx, mut rx) = tokio::sync::oneshot::channel();
        svc.waiters.lock().await.insert("r1".to_string(), tx);
        svc.pending
            .lock()
            .await
            .insert("r1".to_string(), (vm_req("r1", "t1"), Instant::now()));
        let other = Resolver { user_id: Uuid::from_u128(2), thread_id: Some("t1") };
        let refused = resolve(&svc, "r1", Decision::AllowAll, other).await;
        assert!(refused.is_err_and(|e| e.contains(NOT_OWNER)));
        assert!(svc.pending.lock().await.contains_key("r1"));
        assert!(svc.waiters.lock().await.contains_key("r1"));
        assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)), "the waiting action was not woken");
    });
}

#[test]
fn deleting_a_conversation_withdraws_only_its_requests() {
    let rt = runtime();
    rt.block_on(async {
        let svc = service();
        let mut waiting = std::collections::HashMap::new();
        for (id, thread) in [("a", "t1"), ("b", "t1"), ("c", "t2")] {
            let (tx, rx) = tokio::sync::oneshot::channel::<Decision>();
            svc.waiters.lock().await.insert(id.to_string(), tx);
            svc.pending.lock().await.insert(id.to_string(), (vm_req(id, thread), Instant::now()));
            waiting.insert(id, rx);
        }
        assert_eq!(withdraw_thread(&svc, "t1").await, 2);
        let pending = svc.pending.lock().await;
        assert!(pending.contains_key("c") && !pending.contains_key("a") && !pending.contains_key("b"));
        for (id, rx) in &mut waiting {
            let expected_closed = *id != "c";
            let closed = matches!(rx.try_recv(), Err(TryRecvError::Closed));
            assert_eq!(closed, expected_closed, "request {id}: a withdrawn action is told no, the other waits");
        }
    });
}

#[test]
fn a_request_lives_as_long_as_its_turn() {
    // Still its turn, even after the person sent another (queued) message.
    assert!(!turn_superseded(Some("run-1"), Some("run-1"), false, false));
    // The turn was stopped while the card was open.
    assert!(turn_superseded(Some("run-1"), Some("run-1"), true, false));
    // A stop flag left from an earlier turn does not withdraw it.
    assert!(!turn_superseded(Some("run-1"), Some("run-1"), true, true));
    // A new message started another turn: the old card is withdrawn.
    assert!(turn_superseded(Some("run-1"), Some("run-2"), false, false));
    assert!(turn_superseded(None, Some("run-2"), false, false));
    assert!(!turn_superseded(None, None, false, false));
}

#[test]
fn a_deleted_conversation_matches_its_own_grants_only() {
    let thread = Uuid::from_u128(7);
    let pattern = crate::store::conversation_vm_app_pattern(thread);
    assert_eq!(pattern, format!("vm:%:{thread}"));
    assert!(!pattern.contains('_'));
}
