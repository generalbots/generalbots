//! Offline identity/isolation tests for the sheet handlers (#2127).
//!
//! The sheet API used to answer every anonymous request as the shared
//! `default-user` namespace, so anyone could list, read and delete workbooks
//! without a token. These tests pin the two properties that replaced it:
//!
//! * a handler invoked without a verified identity answers `401` and touches
//!   no storage;
//! * two verified users never see each other's workbooks, because every
//!   read/write is scoped by the verified user id.
//!
//! No server, database or Drive is involved: `MemDrive` is an in-memory
//! `DriveOps`, so the handlers run exactly as they do in production.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use axum::extract::{Extension, State};
use axum::http::StatusCode;
use axum::Json;

use crate::auth::SheetUser;
use crate::handlers::crud::{handle_delete_sheet, handle_list_sheets, handle_new_sheet,
    handle_save_sheet};
use crate::state::{DriveOps, SheetState};
use botsheet_core::sessions::SessionStore;
use botsheet_core::types::{LoadQuery, SaveRequest};

/// In-memory Drive: key -> bytes.
#[derive(Default)]
struct MemDrive(Mutex<HashMap<String, Vec<u8>>>);

impl MemDrive {
    fn keys(&self) -> Vec<String> {
        let map = match self.0.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        let mut keys: Vec<String> = map.keys().cloned().collect();
        keys.sort();
        keys
    }
}

#[async_trait]
impl DriveOps for MemDrive {
    async fn put_object(
        &self,
        _bucket: &str,
        key: &str,
        body: Vec<u8>,
        _content_type: &str,
    ) -> Result<(), String> {
        let mut map = match self.0.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        map.insert(key.to_string(), body);
        Ok(())
    }

    async fn get_object(&self, _bucket: &str, key: &str) -> Result<Vec<u8>, String> {
        let map = match self.0.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        map.get(key)
            .cloned()
            .ok_or_else(|| format!("404 no such object {key}"))
    }

    async fn list_objects(&self, _bucket: &str, prefix: &str) -> Result<Vec<String>, String> {
        Ok(self
            .keys()
            .into_iter()
            .filter(|key| key.starts_with(prefix))
            .collect())
    }

    async fn delete_object(&self, _bucket: &str, key: &str) -> Result<(), String> {
        let mut map = match self.0.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        map.remove(key);
        Ok(())
    }
}

fn state() -> (Arc<SheetState>, Arc<MemDrive>) {
    let drive = Arc::new(MemDrive::default());
    let state = Arc::new(SheetState {
        drive: Some(drive.clone()),
        on_save: None,
        sessions: SessionStore::default(),
    });
    (state, drive)
}

fn alice() -> SheetUser {
    SheetUser::verified("u-alice", "Alice")
}

fn bob() -> SheetUser {
    SheetUser::verified("u-bob", "Bob")
}

fn save_request(name: &str) -> SaveRequest {
    SaveRequest {
        id: None,
        name: Some(name.to_string()),
        worksheets: Vec::new(),
        action: None,
    }
}

#[tokio::test]
async fn anonymous_list_is_unauthorized() {
    let (state, drive) = state();
    let error = handle_list_sheets(State(state), None)
        .await
        .expect_err("anonymous listing must be refused");
    assert_eq!(error.0, StatusCode::UNAUTHORIZED);
    assert!(drive.keys().is_empty(), "a refused request must not read or write");
}

#[tokio::test]
async fn anonymous_save_is_unauthorized_and_writes_nothing() {
    let (state, drive) = state();
    let error = handle_save_sheet(State(state), None, Json(save_request("Budget")))
        .await
        .expect_err("anonymous save must be refused");
    assert_eq!(error.0, StatusCode::UNAUTHORIZED);
    assert!(
        drive.keys().is_empty(),
        "no object may be created for an unauthenticated caller"
    );
}

#[tokio::test]
async fn anonymous_new_sheet_is_unauthorized() {
    let (state, _drive) = state();
    let error = handle_new_sheet(State(state), None)
        .await
        .expect_err("anonymous new sheet must be refused");
    assert_eq!(error.0, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn no_handler_silently_uses_the_legacy_default_namespace() {
    let (state, drive) = state();
    // Every entry point an anonymous caller used to reach.
    let _ = handle_new_sheet(State(state.clone()), None).await;
    let _ = handle_list_sheets(State(state.clone()), None).await;
    let _ = handle_save_sheet(State(state), None, Json(save_request("Ghost"))).await;
    assert!(
        drive.keys().iter().all(|key| !key.contains("default-user")),
        "nothing may be stored in the shared legacy namespace: {:?}",
        drive.keys()
    );
}

#[tokio::test]
async fn two_users_cannot_see_each_others_sheets() {
    let (state, _drive) = state();

    let saved = handle_save_sheet(
        State(state.clone()),
        Some(Extension(alice())),
        Json(save_request("Alice budget")),
    )
    .await
    .expect("alice saves her own sheet");
    let sheet_id = saved.0.id.clone();

    let alice_list = handle_list_sheets(State(state.clone()), Some(Extension(alice())))
        .await
        .expect("alice lists");
    assert_eq!(alice_list.0.len(), 1, "alice sees her sheet");

    let bob_list = handle_list_sheets(State(state.clone()), Some(Extension(bob())))
        .await
        .expect("bob lists");
    assert!(
        bob_list.0.is_empty(),
        "bob must not see alice's workbook: {:?}",
        bob_list.0.iter().map(|m| &m.name).collect::<Vec<_>>()
    );

    let bob_load = crate::handlers::crud::handle_load_sheet(
        State(state.clone()),
        Some(Extension(bob())),
        axum::http::HeaderMap::new(),
        axum::extract::Query(LoadQuery {
            id: Some(sheet_id.clone()),
        }),
    )
    .await;
    assert!(
        bob_load.is_err(),
        "bob must not load a workbook that lives in alice's namespace"
    );

    let bob_delete = handle_delete_sheet(
        State(state.clone()),
        Some(Extension(bob())),
        Json(LoadQuery {
            id: Some(sheet_id.clone()),
        }),
    )
    .await;
    assert!(bob_delete.is_ok(), "delete is scoped to bob's own namespace");

    let alice_after = handle_list_sheets(State(state), Some(Extension(alice())))
        .await
        .expect("alice lists again");
    assert_eq!(
        alice_after.0.len(),
        1,
        "bob's delete must not remove alice's workbook"
    );
}
