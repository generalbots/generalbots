//! #2949 — route/caller parity guard for the Campaigns suite app.
//!
//! `GET /api/instagram/campaigns` was declared in `botinstagram` but the live
//! server mounted only the webhook/send router, so the app's call returned 404.
//! These tests drive the production assembly seam and assert that the path the
//! frontend actually calls is a registered, non-404 route, with an
//! unregistered path as a non-vacuity control.

use super::instagram_router_with_state;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use botinstagram::{ChannelState, INSTAGRAM_CAMPAIGNS_PATH};
use std::path::PathBuf;
use std::sync::Arc;
use tower::ServiceExt;

type NoopStream = Arc<
    dyn Fn(
            botlib::models::UserMessage,
            tokio::sync::mpsc::Sender<botlib::models::BotResponse>,
        ) -> tokio::task::JoinHandle<Result<(), String>>
        + Send
        + Sync,
>;

fn test_state() -> Arc<ChannelState> {
    let stream_response: NoopStream = Arc::new(
        |_: botlib::models::UserMessage,
         _: tokio::sync::mpsc::Sender<botlib::models::BotResponse>| {
            tokio::spawn(async { Ok::<(), String>(()) })
        },
    );
    Arc::new(ChannelState {
        get_config: Arc::new(|_: &str, _: &str, _: Option<&str>| Ok(String::new())),
        stream_response,
        attendant_broadcast: None,
    })
}

async fn status(uri: &str) -> StatusCode {
    instagram_router_with_state(test_state())
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(uri)
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("router response")
        .status()
}

#[tokio::test]
async fn campaigns_list_path_is_mounted_not_404() {
    assert_eq!(status(INSTAGRAM_CAMPAIGNS_PATH).await, StatusCode::OK);
}

#[tokio::test]
async fn unregistered_instagram_path_still_404s() {
    assert_eq!(
        status("/api/instagram/definitely-not-a-route").await,
        StatusCode::NOT_FOUND
    );
}

#[test]
fn suite_campaigns_caller_matches_registered_route() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let caller = root.join("../botui/ui/suite/js/campaigns/01_state.js");
    let source = std::fs::read_to_string(&caller).expect("campaigns app state source");
    let declared = source
        .lines()
        .find_map(|line| {
            let value = line.trim().strip_prefix("instagramUrl:")?;
            Some(
                value
                    .trim()
                    .trim_end_matches(',')
                    .trim_matches(['"', '\''])
                    .to_string(),
            )
        })
        .expect("instagramUrl in 01_state.js");
    assert_eq!(
        declared, INSTAGRAM_CAMPAIGNS_PATH,
        "the Campaigns app calls {declared} but the router registers {INSTAGRAM_CAMPAIGNS_PATH}"
    );
}
