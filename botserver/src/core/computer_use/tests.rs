use super::*;
use serde_json::json;
use uuid::Uuid;

use super::*;

fn user() -> Uuid {
    Uuid::new_v4()
}

/// The suite collects each proposed action once, and only for the user and
/// conversation it belongs to.
#[tokio::test]
async fn pending_is_scoped_and_delivered_once() {
    let owner = user();
    let session = format!("s-{}", Uuid::new_v4());
    let action = create(&session, owner, "browser", json!({"action": "open_url"})).await;
    let other = user();

    assert!(pending_for(&session, other, "browser").await.is_empty());
    assert!(pending_for(&session, owner, "terminal").await.is_empty());

    let collected = pending_for(&session, owner, "browser").await;
    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0].id, action.id);
    assert!(
        pending_for(&session, owner, "browser").await.is_empty(),
        "a reconnecting tab must not receive the same consent card twice"
    );
}

#[tokio::test]
async fn results_are_scoped_to_the_owner() {
    let owner = user();
    let session = format!("s-{}", Uuid::new_v4());
    let action = create(
        &session,
        owner,
        "browser",
        json!({"action": "extract_text"}),
    )
    .await;

    assert!(complete(action.id, user(), json!({"status": "ok"}))
        .await
        .is_none());
    assert!(get(action.id, user()).await.is_none());

    let done = complete(action.id, owner, json!({"status": "ok", "text": "hello"}))
        .await
        .expect("the owner can record the outcome");
    assert_eq!(done.status, "completed");
    assert_eq!(done.result.unwrap()["text"], "hello");

    let denied = create(
        &session,
        owner,
        "browser",
        json!({"action": "click_element"}),
    )
    .await;
    let denied = complete(denied.id, owner, json!({"status": "denied"}))
        .await
        .expect("denial is recorded");
    assert_eq!(denied.status, "denied");
}

#[tokio::test]
async fn terminal_outcome_is_queryable_by_id() {
    let owner = user();
    let session = format!("s-{}", Uuid::new_v4());
    let recorded = record_terminal(
        &session,
        owner,
        json!({"command": "echo hi"}),
        json!({"status": "ok", "stdout": "hi\n", "exit_code": 0}),
    )
    .await;
    assert_eq!(recorded.status, "completed");
    assert_eq!(recorded.kind, "terminal_exec");
    let found = get(recorded.id, owner)
        .await
        .expect("recorded action is readable");
    assert_eq!(found.result.unwrap()["stdout"], "hi\n");
}

/// #1644 — a URL in an explicit "open/read" request becomes a browser
/// proposal; ordinary text with no verb or no URL stays with the model.
#[test]
fn explicit_url_requests_become_proposals() {
    let proposal = propose_from_text("please open https://example.com/docs.")
        .expect("a URL with an open verb must propose an action");
    assert_eq!(proposal["action"], "open_url");
    assert_eq!(proposal["url"], "https://example.com/docs");
    assert!(propose_from_text("what is the weather?").is_none());
    assert!(propose_from_text("open the settings window").is_none());
}

#[tokio::test]
async fn execution_requires_a_stored_approval() {
    let owner = user();
    let session = format!("s-{}", Uuid::new_v4());
    let action = create(&session, owner, "browser", json!({"action": "open_url"})).await;
    assert!(!is_approved(action.id, owner).await);
    assert!(approve(action.id, user()).await.is_none());
    let approved = approve(action.id, owner).await.expect("owner can approve");
    assert!(approved.approved_at.is_some());
    assert!(is_approved(action.id, owner).await);
}

#[tokio::test]
async fn completed_outcomes_are_reported_once() {
    let owner = user();
    let session = format!("s-{}", Uuid::new_v4());
    let action = create(&session, owner, "browser", json!({"action": "open_url"})).await;
    complete(
        action.id,
        owner,
        json!({"status": "ok", "title": "Example"}),
    )
    .await;
    let first = take_completed_for_prompt(&session, owner).await;
    assert_eq!(first.len(), 1);
    assert!(
        take_completed_for_prompt(&session, owner).await.is_empty(),
        "the same outcome must not enter two turns"
    );
}

#[test]
fn explicit_terminal_requests_become_proposals() {
    let proposal = propose_terminal_from_text("run `ls -la` in the workspace")
        .expect("a backticked command must propose a terminal action");
    assert_eq!(proposal["command"], "ls -la");
    let plain = propose_terminal_from_text("execute echo hello")
        .expect("a plain command after the verb is accepted");
    assert_eq!(plain["command"], "echo hello");
    assert!(propose_terminal_from_text("what is the weather?").is_none());
    assert!(propose_terminal_from_text("run ").is_none());
}

#[test]
fn live_terminal_tool_requests_are_left_to_the_model() {
    assert!(propose_terminal_from_text(
        "Run this with terminal_run in a live terminal: for i in 1 2 3; do echo tick $i; done"
    )
    .is_none());
}

#[test]
fn command_after_colon_excludes_the_follow_up_request() {
    let proposal = propose_terminal_from_text(
        "Run this shell command with vm_exec: ls /nope. Report the stdout/stderr \
             and the exact exit code; do not claim success.",
    )
    .expect("a named command must propose a terminal action");
    assert_eq!(proposal["command"], "ls /nope");

    let chained = propose_terminal_from_text("Use vm_exec to run the shell command: date; ls -la")
        .expect("a chained command must be preserved");
    assert_eq!(chained["command"], "date; ls -la");
}
