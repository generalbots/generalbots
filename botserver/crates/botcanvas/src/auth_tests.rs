use super::*;
use axum::http::{request::Parts, Request};

fn parts() -> Parts {
    Request::builder()
        .uri("/api/canvas/collaborators")
        .body(())
        .expect("request builder is valid for a test")
        .into_parts()
        .0
}

#[tokio::test]
async fn a_missing_identity_is_rejected() {
    let mut request_parts = parts();
    let result = CanvasUser::from_request_parts(&mut request_parts, &()).await;
    assert_eq!(result.err(), Some(CanvasUnauthorized));
}

#[tokio::test]
async fn an_anonymous_identity_is_rejected() {
    let mut request_parts = parts();
    request_parts.extensions.insert(CanvasUser::new(Uuid::nil()));
    let result = CanvasUser::from_request_parts(&mut request_parts, &()).await;
    assert_eq!(result.err(), Some(CanvasUnauthorized));
}

#[tokio::test]
async fn a_verified_identity_is_accepted() {
    let user_id = Uuid::new_v4();
    let mut request_parts = parts();
    request_parts.extensions.insert(CanvasUser::new(user_id));
    let caller = CanvasUser::from_request_parts(&mut request_parts, &())
        .await
        .expect("verified identity must be accepted");
    assert_eq!(caller.user_id, user_id);
    assert!(caller.is_verified());
}
