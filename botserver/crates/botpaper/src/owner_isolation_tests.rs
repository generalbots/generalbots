//! Cross-user isolation for Paper storage (#2137).
//!
//! The owner comes from the verified identity, so every list/load/delete only
//! sees the `users/{owner}/papers/` prefix of the caller.

use crate::state::{DbPool, PaperState, S3DeleteFn, S3GetFn, S3ListFn, S3PutFn};
use crate::storage::{
    delete_document_from_drive, list_documents_from_drive, load_document_from_drive,
    save_document_to_drive,
};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;

type Store = Arc<Mutex<HashMap<String, Vec<u8>>>>;

fn pool() -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new("postgres://localhost/unused");
    Pool::builder().max_size(1).build_unchecked(manager)
}

fn paper_state(store: &Store) -> Arc<PaperState> {
    let s3_put: S3PutFn = {
        let store = store.clone();
        Arc::new(move |bucket, key, data, _content_type| {
            let store = store.clone();
            let bucket = bucket.to_string();
            let key = key.to_string();
            Box::pin(async move {
                store
                    .lock()
                    .map_err(|e| e.to_string())?
                    .insert(format!("{bucket}/{key}"), data);
                Ok(())
            })
                as std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>
        })
    };

    let s3_get: S3GetFn = {
        let store = store.clone();
        Arc::new(move |bucket, key| {
            let store = store.clone();
            let bucket = bucket.to_string();
            let key = key.to_string();
            Box::pin(async move {
                let objects = store.lock().map_err(|e| e.to_string())?;
                objects
                    .get(&format!("{bucket}/{key}"))
                    .cloned()
                    .ok_or_else(|| "404 Not Found".to_string())
            })
                as std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<Vec<u8>, String>> + Send>,
                >
        })
    };

    let s3_delete: S3DeleteFn = {
        let store = store.clone();
        Arc::new(move |bucket, key| {
            let store = store.clone();
            let bucket = bucket.to_string();
            let key = key.to_string();
            Box::pin(async move {
                store
                    .lock()
                    .map_err(|e| e.to_string())?
                    .remove(&format!("{bucket}/{key}"));
                Ok(())
            })
                as std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>>
        })
    };

    let s3_list: S3ListFn = {
        let store = store.clone();
        Arc::new(move |bucket, prefix| {
            let store = store.clone();
            let bucket = bucket.to_string();
            let prefix = prefix.to_string();
            Box::pin(async move {
                let needle = format!("{bucket}/{prefix}");
                let objects = store.lock().map_err(|e| e.to_string())?;
                let mut keys: Vec<String> = objects
                    .keys()
                    .filter(|key| key.starts_with(&needle))
                    .map(|key| key[bucket.len() + 1..].to_string())
                    .collect();
                keys.sort();
                Ok(keys)
            })
                as std::pin::Pin<
                    Box<dyn std::future::Future<Output = Result<Vec<String>, String>> + Send>,
                >
        })
    };

    let call_llm: crate::state::CallLlmFn = Arc::new(|_prompt, _context| {
        Box::pin(async { Err("no llm".to_string()) })
            as std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>>
    });

    Arc::new(PaperState {
        conn: pool(),
        bucket_name: "test-bot.gbai".to_string(),
        s3_put,
        s3_get,
        s3_delete,
        s3_list,
        call_llm,
    })
}

async fn seed(state: &Arc<PaperState>, owner: &str, id: &str, title: &str) {
    save_document_to_drive(state, owner, id, title, "# body", false)
        .await
        .expect("seed document");
}

#[tokio::test]
async fn a_users_list_excludes_another_users_documents() {
    let store: Store = Arc::default();
    let state = paper_state(&store);
    seed(&state, "user-a", "doc-a", "Alice doc").await;
    seed(&state, "user-b", "doc-b", "Bob doc").await;

    let alice = list_documents_from_drive(&state, "user-a")
        .await
        .expect("alice list");
    assert_eq!(alice.len(), 1);
    assert_eq!(alice[0].id, "doc-a");

    let bob = list_documents_from_drive(&state, "user-b")
        .await
        .expect("bob list");
    assert_eq!(bob.len(), 1);
    assert_eq!(bob[0].id, "doc-b");
}

#[tokio::test]
async fn another_users_document_cannot_be_loaded() {
    let store: Store = Arc::default();
    let state = paper_state(&store);
    seed(&state, "user-a", "doc-a", "Alice doc").await;

    let loaded = load_document_from_drive(&state, "user-b", "doc-a")
        .await
        .expect("load is not an error");
    assert!(loaded.is_none());
}

#[tokio::test]
async fn deleting_another_users_document_is_not_found() {
    let store: Store = Arc::default();
    let state = paper_state(&store);
    seed(&state, "user-a", "doc-a", "Alice doc").await;

    let error = delete_document_from_drive(&state, "user-b", "doc-a")
        .await
        .expect_err("bob must not delete alice's document");
    assert_eq!(error, "Document not found");

    let still_there = load_document_from_drive(&state, "user-a", "doc-a")
        .await
        .expect("alice load")
        .expect("alice document survives");
    assert_eq!(still_there.id, "doc-a");
}

#[tokio::test]
async fn deleting_a_missing_document_is_not_found() {
    let store: Store = Arc::default();
    let state = paper_state(&store);
    seed(&state, "user-a", "doc-a", "Alice doc").await;

    let error = delete_document_from_drive(&state, "user-a", "nope")
        .await
        .expect_err("missing id");
    assert_eq!(error, "Document not found");
}

async fn as_alice(
    mut request: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> axum::response::Response {
    request
        .extensions_mut()
        .insert(crate::auth::PaperUser::new("user-a", "Alice"));
    next.run(request).await
}

fn new_document_app(state: Arc<PaperState>, identity: bool) -> axum::Router {
    let router = axum::Router::new()
        .route(
            crate::PAPER_NEW,
            axum::routing::post(crate::handlers::handle_new_document),
        )
        .with_state(state);
    if identity {
        router.layer(axum::middleware::from_fn(as_alice))
    } else {
        router
    }
}

#[tokio::test]
async fn creating_a_document_without_identity_is_unauthorized() {
    let store: Store = Arc::default();
    let app = new_document_app(paper_state(&store), false);
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(crate::PAPER_NEW)
                .body(axum::body::Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), axum::http::StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn creating_a_document_uses_the_verified_owner() {
    let store: Store = Arc::default();
    let state = paper_state(&store);
    let app = new_document_app(state.clone(), true);
    let response = app
        .oneshot(
            axum::http::Request::builder()
                .method("POST")
                .uri(crate::PAPER_NEW)
                .body(axum::body::Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), axum::http::StatusCode::OK);

    // The document was written under the verified owner's prefix only.
    let alice = list_documents_from_drive(&state, "user-a")
        .await
        .expect("alice list");
    assert_eq!(alice.len(), 1);
    let bob = list_documents_from_drive(&state, "user-b")
        .await
        .expect("bob list");
    assert!(bob.is_empty());
}
