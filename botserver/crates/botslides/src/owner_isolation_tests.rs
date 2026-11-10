//! Cross-user isolation for deck storage (#2137).
//!
//! Every read and write resolves the owner from the verified identity, so a
//! caller can only reach the decks under `users/{their id}/presentations/`.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::routing::get;
use axum::Router;
use crate::auth::SlidesUser;
use crate::handlers::handle_new_presentation;
use crate::storage::{
    create_new_presentation, delete_presentation_from_drive, list_presentations_from_drive,
    load_presentation_by_id, save_presentation_to_drive, DriveOps,
};
use crate::SlidesState;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

/// Minimal in-memory Drive: enough of the object API to exercise the deck
/// paths without a database, a bucket or a network.
#[derive(Clone, Default)]
struct MemoryDrive {
    objects: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl MemoryDrive {
    fn object_key(bucket: &str, key: &str) -> String {
        format!("{bucket}/{key}")
    }
}

impl DriveOps for MemoryDrive {
    async fn put_object(
        &self,
        bucket: &str,
        key: &str,
        body: Vec<u8>,
        _content_type: &str,
    ) -> Result<(), String> {
        let mut objects = self.objects.lock().map_err(|e| e.to_string())?;
        objects.insert(Self::object_key(bucket, key), body);
        Ok(())
    }

    async fn get_object(&self, bucket: &str, key: &str) -> Result<Vec<u8>, String> {
        let objects = self.objects.lock().map_err(|e| e.to_string())?;
        objects
            .get(&Self::object_key(bucket, key))
            .cloned()
            .ok_or_else(|| "404 Not Found".to_string())
    }

    async fn list_objects(&self, bucket: &str, prefix: &str) -> Result<Vec<String>, String> {
        let needle = Self::object_key(bucket, prefix);
        let objects = self.objects.lock().map_err(|e| e.to_string())?;
        let mut keys: Vec<String> = objects
            .keys()
            .filter(|key| key.starts_with(&needle))
            .map(|key| key[needle.len()..].to_string())
            .collect();
        keys.sort();
        Ok(keys)
    }

    async fn delete_object(&self, bucket: &str, key: &str) -> Result<(), String> {
        let mut objects = self.objects.lock().map_err(|e| e.to_string())?;
        objects.remove(&Self::object_key(bucket, key));
        Ok(())
    }
}

async fn seed(drive: &MemoryDrive, owner: &str, name: &str) -> crate::types::Presentation {
    let mut deck = create_new_presentation(owner);
    deck.name = name.to_string();
    save_presentation_to_drive(drive, owner, &deck)
        .await
        .expect("seed deck");
    deck
}

#[tokio::test]
async fn a_users_list_excludes_another_users_decks() {
    let drive = MemoryDrive::default();
    let alice_deck = seed(&drive, "user-a", "Alice deck").await;
    let bob_deck = seed(&drive, "user-b", "Bob deck").await;

    let alice = list_presentations_from_drive(&drive, "user-a")
        .await
        .expect("alice list");
    assert_eq!(alice.len(), 1);
    assert_eq!(alice[0].id, alice_deck.id);

    let bob = list_presentations_from_drive(&drive, "user-b")
        .await
        .expect("bob list");
    assert_eq!(bob.len(), 1);
    assert_eq!(bob[0].id, bob_deck.id);
}

#[tokio::test]
async fn another_users_deck_cannot_be_loaded() {
    let drive = MemoryDrive::default();
    let alice_deck = seed(&drive, "user-a", "Alice deck").await;

    let error = load_presentation_by_id(&drive, "user-b", &alice_deck.id)
        .await
        .expect_err("bob must not load alice's deck");
    assert_eq!(error, "Presentation not found");
}

#[tokio::test]
async fn deleting_another_users_deck_is_not_found() {
    let drive = MemoryDrive::default();
    let alice_deck = seed(&drive, "user-a", "Alice deck").await;

    let error = delete_presentation_from_drive(&drive, "user-b", &alice_deck.id)
        .await
        .expect_err("bob must not delete alice's deck");
    assert_eq!(error, "Presentation not found");

    // The deck is untouched for its owner.
    let still_there = load_presentation_by_id(&drive, "user-a", &alice_deck.id)
        .await
        .expect("alice deck survives");
    assert_eq!(still_there.name, "Alice deck");
}

#[tokio::test]
async fn saving_under_another_owner_never_touches_the_original() {
    let drive = MemoryDrive::default();
    let alice_deck = seed(&drive, "user-a", "Alice deck").await;

    let mut copy = alice_deck.clone();
    copy.name = "Hijacked".to_string();
    save_presentation_to_drive(&drive, "user-b", &copy)
        .await
        .expect("bob writes into his own namespace");

    // The copy carries alice's owner id, so it is not served to bob either.
    let bob_copy = load_presentation_by_id(&drive, "user-b", &alice_deck.id).await;
    assert!(bob_copy.is_err());

    let original = load_presentation_by_id(&drive, "user-a", &alice_deck.id)
        .await
        .expect("alice deck unchanged");
    assert_eq!(original.name, "Alice deck");
}

#[tokio::test]
async fn a_deck_copied_into_the_wrong_prefix_is_not_served() {
    let drive = MemoryDrive::default();
    let mut deck = create_new_presentation("user-a");
    deck.owner_id = "user-a".to_string();
    save_presentation_to_drive(&drive, "user-b", &deck)
        .await
        .expect("write mismatched deck");

    let error = load_presentation_by_id(&drive, "user-b", &deck.id)
        .await
        .expect_err("owner mismatch is not served");
    assert_eq!(error, "Presentation not found");
}

fn slides_state(drive: MemoryDrive) -> std::sync::Arc<SlidesState<MemoryDrive>> {
    let manager =
        diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new("postgres://localhost/unused");
    let conn = diesel::r2d2::Pool::builder()
        .max_size(1)
        .build_unchecked(manager);
    std::sync::Arc::new(SlidesState {
        drive: Some(drive),
        conn: std::sync::Arc::new(conn),
        get_default_bot: std::sync::Arc::new(|_conn: &mut diesel::PgConnection| {
            (uuid::Uuid::nil(), "test-bot".to_string())
        }),
    })
}

fn new_deck_app(state: std::sync::Arc<SlidesState<MemoryDrive>>, identity: bool) -> Router {
    let router = Router::new()
        .route(
            "/api/slides/new",
            get(handle_new_presentation::<MemoryDrive>),
        )
        .with_state(state);
    if identity {
        router.layer(axum::middleware::from_fn(as_alice))
    } else {
        router
    }
}

async fn as_alice(
    mut request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    request
        .extensions_mut()
        .insert(SlidesUser::new("user-a", "Alice"));
    next.run(request).await
}

#[tokio::test]
async fn creating_a_deck_without_identity_is_unauthorized() {
    let app = new_deck_app(slides_state(MemoryDrive::default()), false);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/slides/new")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn creating_a_deck_uses_the_verified_owner() {
    let app = new_deck_app(slides_state(MemoryDrive::default()), true);
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/slides/new")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);

    let body = axum::body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("body");
    let deck: crate::types::Presentation = serde_json::from_slice(&body).expect("deck json");
    assert_eq!(deck.owner_id, "user-a");
}
