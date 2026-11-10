//! Plan collaboration surface (issue #2136).
//!
//! Before this fix the socket invented a guest identity when no token was
//! present, so an anonymous visitor could watch (and post into) any plan
//! channel. There is no guest path now: the upgrade requires a verified
//! credential bound to the plan, and the presence/typing readers require the
//! same authorization as the plan itself.

use super::{handle_get_plan_collaborators, handle_get_plan_presence, handle_get_plan_typing};
use crate::auth::PlanError;
use crate::test_support::headers_for;
use crate::Plan;
use axum::http::{HeaderMap, StatusCode};
use chrono::Utc;
use std::collections::HashMap;
use uuid::Uuid;

const OWNER: &str = "collab-owner";
const STRANGER: &str = "collab-stranger";
const BRANCH: Uuid = Uuid::from_u128(0x3333);

async fn seed_plan() -> String {
    let id = Uuid::new_v4().to_string();
    let plan = Plan {
        id: id.clone(),
        title: "Colaboração".to_string(),
        description: None,
        owner_id: OWNER.to_string(),
        org_id: None,
        branch_id: Some(BRANCH.to_string()),
        members: Vec::new(),
        tasks: HashMap::new(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    crate::get_store().write().await.insert(id.clone(), plan);
    id
}

#[tokio::test]
async fn the_presence_readers_refuse_an_anonymous_caller() {
    let plan_id = seed_plan().await;
    let empty = HeaderMap::new();
    let collaborators = handle_get_plan_collaborators(empty.clone(), axum::extract::Path(plan_id.clone()))
        .await
        .expect_err("anonymous collaborators");
    assert_eq!(collaborators.status(), StatusCode::UNAUTHORIZED);
    let presence = handle_get_plan_presence(empty.clone(), axum::extract::Path(plan_id.clone()))
        .await
        .expect_err("anonymous presence");
    assert_eq!(presence.status(), StatusCode::UNAUTHORIZED);
    let typing = handle_get_plan_typing(empty, axum::extract::Path(plan_id))
        .await
        .expect_err("anonymous typing");
    assert_eq!(typing.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn the_presence_readers_answer_the_owner() {
    let plan_id = seed_plan().await;
    let headers = headers_for(OWNER, Some(BRANCH));
    let collaborators =
        handle_get_plan_collaborators(headers.clone(), axum::extract::Path(plan_id.clone()))
            .await
            .expect("owner collaborators");
    let presence = handle_get_plan_presence(headers, axum::extract::Path(plan_id))
        .await
        .expect("owner presence");
    // The JSON shape the plan UI reads is unchanged.
    assert!(collaborators.0.get("count").is_some());
    assert!(presence.0.get("users").is_some());
}

#[tokio::test]
async fn a_stranger_cannot_read_the_collaboration_state() {
    let plan_id = seed_plan().await;
    let headers = headers_for(STRANGER, Some(BRANCH));
    let refusal = handle_get_plan_presence(headers, axum::extract::Path(plan_id))
        .await
        .expect_err("stranger");
    assert_eq!(refusal, PlanError::NotFound);
    assert_eq!(refusal.status(), StatusCode::NOT_FOUND);
}
