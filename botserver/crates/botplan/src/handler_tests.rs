//! Plan REST authorization (issue #2136).
//!
//! These tests call the handlers directly and assert the observable contract:
//! a plan belonging to somebody else (or to another tenant) answers **404** on
//! read/update/delete, an unauthenticated call answers **401**, a member may
//! write tasks but not administer membership (**403**), and `GET` never creates
//! a plan as a side effect.

use super::*;
use crate::test_support::headers_for;
use crate::PlanError;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use axum::Json;

const OWNER: &str = "rest-owner";
const MEMBER: &str = "rest-member";
const INTRUDER: &str = "rest-intruder";
const BRANCH: Uuid = Uuid::from_u128(0x4444);
const OTHER_BRANCH: Uuid = Uuid::from_u128(0x5555);

async fn create_plan_for(sub: &str, branch: Option<Uuid>) -> Plan {
    let response = handle_create_plan(
        headers_for(sub, branch),
        Json(CreatePlanRequest {
            title: Some("Plano de teste".to_string()),
            description: None,
        }),
    )
    .await
    .expect("create plan");
    serde_json::from_value(response.0["plan"].clone()).expect("plan json")
}

async fn add_task(plan_id: &str, sub: &str, branch: Option<Uuid>) -> String {
    let response = handle_create_task(
        headers_for(sub, branch),
        Json(CreateTaskRequest {
            plan_id: plan_id.to_string(),
            title: "Tarefa".to_string(),
            ..Default::default()
        }),
    )
    .await
    .expect("create task");
    response.0["task"]["id"]
        .as_str()
        .expect("task id")
        .to_string()
}

#[tokio::test]
async fn the_owner_reads_their_own_plan() {
    let plan = create_plan_for(OWNER, Some(BRANCH)).await;
    assert_eq!(plan.owner_id, OWNER);
    assert_eq!(plan.branch_id.as_deref(), Some(BRANCH.to_string().as_str()));
    assert!(Uuid::parse_str(&plan.id).is_ok(), "ids are server-minted");

    let response = handle_get_plan(headers_for(OWNER, Some(BRANCH)), Path(plan.id.clone()))
        .await
        .expect("owner read");
    assert_eq!(response.0["ok"], serde_json::json!(true));
    assert_eq!(response.0["plan"]["owner_id"], serde_json::json!(OWNER));
}

#[tokio::test]
async fn plan_ids_are_server_generated_and_distinct() {
    let first = create_plan_for(OWNER, Some(BRANCH)).await;
    let second = create_plan_for(OWNER, Some(BRANCH)).await;
    assert_ne!(first.id, second.id);
    assert_ne!(first.id, "default", "the client no longer chooses the id");
}

#[tokio::test]
async fn an_anonymous_caller_is_refused() {
    let plan = create_plan_for(OWNER, Some(BRANCH)).await;
    let refusal = handle_get_plan(HeaderMap::new(), Path(plan.id))
        .await
        .expect_err("anonymous read");
    assert_eq!(refusal.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn get_never_creates_a_plan() {
    let unknown = Uuid::new_v4().to_string();
    let refusal = handle_get_plan(headers_for(OWNER, Some(BRANCH)), Path(unknown.clone()))
        .await
        .expect_err("unknown plan");
    assert_eq!(refusal, PlanError::NotFound);
    assert_eq!(refusal.status(), StatusCode::NOT_FOUND);
    assert!(
        load_plan(&unknown).await.is_none(),
        "GET must not insert a plan as a side effect"
    );
}

#[tokio::test]
async fn a_foreign_plan_is_not_found_on_read() {
    let plan = create_plan_for(OWNER, Some(BRANCH)).await;
    let refusal = handle_get_plan(headers_for(INTRUDER, Some(BRANCH)), Path(plan.id))
        .await
        .expect_err("foreign read");
    assert_eq!(refusal.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_foreign_plan_is_not_found_on_update() {
    let plan = create_plan_for(OWNER, Some(BRANCH)).await;
    let task_id = add_task(&plan.id, OWNER, Some(BRANCH)).await;
    let refusal = handle_update_task(
        headers_for(INTRUDER, Some(BRANCH)),
        Json(UpdateTaskRequest {
            plan_id: plan.id.clone(),
            task_id,
            title: Some("hijacked".to_string()),
            ..Default::default()
        }),
    )
    .await
    .expect_err("foreign update");
    assert_eq!(refusal.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_foreign_plan_is_not_found_on_delete() {
    let plan = create_plan_for(OWNER, Some(BRANCH)).await;
    let task_id = add_task(&plan.id, OWNER, Some(BRANCH)).await;
    let task_refusal = handle_delete_task(
        headers_for(INTRUDER, Some(BRANCH)),
        Json(DeleteTaskRequest {
            plan_id: plan.id.clone(),
            task_id,
        }),
    )
    .await
    .expect_err("foreign task delete");
    assert_eq!(task_refusal.status(), StatusCode::NOT_FOUND);

    let plan_refusal = handle_delete_plan(headers_for(INTRUDER, Some(BRANCH)), Path(plan.id))
        .await
        .expect_err("foreign plan delete");
    assert_eq!(plan_refusal.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn the_owner_updates_and_deletes_their_tasks_and_plan() {
    let plan = create_plan_for(OWNER, Some(BRANCH)).await;
    let task_id = add_task(&plan.id, OWNER, Some(BRANCH)).await;

    let updated = handle_update_task(
        headers_for(OWNER, Some(BRANCH)),
        Json(UpdateTaskRequest {
            plan_id: plan.id.clone(),
            task_id: task_id.clone(),
            status: Some("done".to_string()),
            ..Default::default()
        }),
    )
    .await
    .expect("owner update");
    assert_eq!(updated.0["task"]["status"], serde_json::json!("done"));

    let deleted = handle_delete_task(
        headers_for(OWNER, Some(BRANCH)),
        Json(DeleteTaskRequest {
            plan_id: plan.id.clone(),
            task_id,
        }),
    )
    .await
    .expect("owner delete task");
    assert_eq!(deleted.0["ok"], serde_json::json!(true));

    let removed = handle_delete_plan(headers_for(OWNER, Some(BRANCH)), Path(plan.id.clone()))
        .await
        .expect("owner delete plan");
    assert_eq!(removed.0["deleted"], serde_json::json!(true));
    assert!(load_plan(&plan.id).await.is_none());
}

#[tokio::test]
async fn a_member_writes_tasks_but_cannot_administer_membership() {
    let plan = create_plan_for(OWNER, Some(BRANCH)).await;
    handle_add_member(
        headers_for(OWNER, Some(BRANCH)),
        Path(plan.id.clone()),
        Json(AddMemberRequest {
            user_id: MEMBER.to_string(),
        }),
    )
    .await
    .expect("owner adds member");

    add_task(&plan.id, MEMBER, Some(BRANCH)).await;

    let refusal = handle_add_member(
        headers_for(MEMBER, Some(BRANCH)),
        Path(plan.id.clone()),
        Json(AddMemberRequest {
            user_id: INTRUDER.to_string(),
        }),
    )
    .await
    .expect_err("member cannot add members");
    assert_eq!(refusal, PlanError::Forbidden("owner_only"));
    assert_eq!(refusal.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn another_tenant_gets_not_found() {
    let plan = create_plan_for(OWNER, Some(BRANCH)).await;
    let refusal = handle_get_plan(headers_for(INTRUDER, Some(OTHER_BRANCH)), Path(plan.id.clone()))
        .await
        .expect_err("other tenant");
    assert_eq!(refusal.status(), StatusCode::NOT_FOUND);
    let owner_elsewhere =
        handle_get_plan(headers_for(OWNER, Some(OTHER_BRANCH)), Path(plan.id))
            .await
            .expect_err("owner of another branch");
    assert_eq!(owner_elsewhere.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn a_task_cannot_be_created_on_a_plan_that_does_not_exist() {
    let unknown = Uuid::new_v4().to_string();
    let refusal = handle_create_task(
        headers_for(OWNER, Some(BRANCH)),
        Json(CreateTaskRequest {
            plan_id: unknown.clone(),
            title: "Órfã".to_string(),
            ..Default::default()
        }),
    )
    .await
    .expect_err("no implicit plan creation");
    assert_eq!(refusal.status(), StatusCode::NOT_FOUND);
    assert!(load_plan(&unknown).await.is_none());
}
