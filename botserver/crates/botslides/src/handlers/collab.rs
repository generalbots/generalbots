use crate::auth::SlidesUser;
use crate::storage::{load_presentation_by_id, DriveOps};
use crate::types::{
    CollaborationCursor, CollaborationSelection, ListCursorsResponse, ListSelectionsResponse,
    UpdateCursorRequest, UpdateSelectionRequest,
};
use crate::SlidesState;
use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    Json,
};
use botsecurity_auth::auth_api::types::AuthenticatedUser;
use chrono::Utc;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, RwLock};
use uuid::Uuid;

static CURSORS: LazyLock<RwLock<HashMap<String, Vec<CollaborationCursor>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

static SELECTIONS: LazyLock<RwLock<HashMap<String, Vec<CollaborationSelection>>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

// Resolve the caller's identity from the auth extension (mirrors the WebSocket
// path, which already distinguishes users). Falls back to a stable default for
// anonymous/session-less callers so cursors never collide silently.
fn collab_user_id(user: &AuthenticatedUser) -> String {
    if let Some(ref email) = user.email {
        if !email.is_empty() && email != "session-user" {
            return email.clone();
        }
    }
    if user.user_id.is_nil() {
        "default".to_string()
    } else {
        user.user_id.to_string()
    }
}

fn collab_user_name(user: &AuthenticatedUser) -> String {
    if let Some(ref email) = user.email {
        if !email.is_empty() && email != "session-user" {
            return email.split('@').next().unwrap_or(email).to_string();
        }
    }
    if !user.username.is_empty() {
        user.username.clone()
    } else {
        "User".to_string()
    }
}

fn collab_user_color(user_id: &str) -> String {
    const PALETTE: [&str; 12] = [
        "#FF6B6B", "#4ECDC4", "#45B7D1", "#96CEB4", "#FFEAA7", "#DDA0DD",
        "#98D8C8", "#F7DC6F", "#BB8FCE", "#85C1E9", "#F1948A", "#82E0AA",
    ];
    let mut h: u32 = 0;
    for b in user_id.bytes() {
        h = h.wrapping_mul(31).wrapping_add(b as u32);
    }
    PALETTE[(h % PALETTE.len() as u32) as usize].to_string()
}

/// Why a collab write was refused before it reached the in-memory maps (#2739).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CollabScopeError {
    /// The client supplied a key that is not a UUID.
    InvalidPresentationId,
    /// The caller has no usable owner scope.
    NoOwnerScope,
}

/// Pure validation/scoping of a collab key (#2739).
///
/// Cursor and selection maps used to be keyed by any string the client sent,
/// so an arbitrary key could be injected into another deck's collaboration
/// state. Only a UUID the caller can own is accepted; the canonical UUID is
/// returned so equivalent spellings of the same id share one key.
fn collab_presentation_scope(
    presentation_id: &str,
    owner_id: &str,
) -> Result<String, CollabScopeError> {
    let id =
        Uuid::parse_str(presentation_id).map_err(|_| CollabScopeError::InvalidPresentationId)?;
    let owner = owner_id.trim();
    if owner.is_empty() || owner == "default-user" {
        return Err(CollabScopeError::NoOwnerScope);
    }
    Ok(id.to_string())
}

/// Authorizes an owner against the stored deck before any collab write (#2739).
///
/// A malformed id is a `400`, a missing owner a `401`, and a deck the caller
/// does not own (or that does not exist) a `404` — never a silent map write.
async fn authorize_collab_deck<D: DriveOps>(
    state: &SlidesState<D>,
    owner_id: &str,
    presentation_id: &str,
) -> Result<String, (StatusCode, Json<serde_json::Value>)> {
    let canonical_id = collab_presentation_scope(presentation_id, owner_id).map_err(|reason| {
        let (status, message) = match reason {
            CollabScopeError::InvalidPresentationId => {
                (StatusCode::BAD_REQUEST, "Invalid presentation id")
            }
            CollabScopeError::NoOwnerScope => (StatusCode::UNAUTHORIZED, "Authentication required"),
        };
        (status, Json(serde_json::json!({ "error": message })))
    })?;

    let drive = state.drive.as_ref().ok_or_else(|| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(serde_json::json!({ "error": "Drive not configured" })),
        )
    })?;

    // The path scopes the read to `owner_id` and the stored deck re-checks its
    // owner, so a caller cannot reach a deck that is not theirs (#2137).
    load_presentation_by_id(drive, owner_id, &canonical_id)
        .await
        .map_err(|_| {
            (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({ "error": "Presentation not found" })),
            )
        })?;

    Ok(canonical_id)
}

pub async fn handle_update_cursor<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    user_scope: SlidesUser,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<UpdateCursorRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let presentation_id =
        authorize_collab_deck(&state, user_scope.owner_id(), &req.presentation_id).await?;
    let user_id = collab_user_id(&user);

    let cursor = CollaborationCursor {
        user_id: user_id.clone(),
        user_name: collab_user_name(&user),
        user_color: collab_user_color(&user_id),
        slide_index: req.slide_index,
        element_id: req.element_id,
        x: req.x,
        y: req.y,
        last_activity: Utc::now(),
    };

    if let Ok(mut cursors) = CURSORS.write() {
        let presentation_cursors = cursors.entry(presentation_id).or_default();
        presentation_cursors.retain(|c| c.user_id != user_id);
        presentation_cursors.push(cursor);
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_update_selection<D: DriveOps>(
    State(state): State<Arc<SlidesState<D>>>,
    user_scope: SlidesUser,
    Extension(user): Extension<AuthenticatedUser>,
    Json(req): Json<UpdateSelectionRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    let presentation_id =
        authorize_collab_deck(&state, user_scope.owner_id(), &req.presentation_id).await?;
    let user_id = collab_user_id(&user);

    let selection = CollaborationSelection {
        user_id: user_id.clone(),
        user_name: collab_user_name(&user),
        user_color: collab_user_color(&user_id),
        slide_index: req.slide_index,
        element_ids: req.element_ids,
    };

    if let Ok(mut selections) = SELECTIONS.write() {
        let presentation_selections = selections.entry(presentation_id).or_default();
        presentation_selections.retain(|s| s.user_id != user_id);
        presentation_selections.push(selection);
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_list_cursors<D: DriveOps>(
    State(_state): State<Arc<SlidesState<D>>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ListCursorsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let presentation_id = params.get("presentation_id").cloned().unwrap_or_default();

    let cursors = if let Ok(cursors_map) = CURSORS.read() {
        cursors_map
            .get(&presentation_id)
            .cloned()
            .unwrap_or_default()
    } else {
        vec![]
    };

    Ok(Json(ListCursorsResponse { cursors }))
}

pub async fn handle_list_selections<D: DriveOps>(
    State(_state): State<Arc<SlidesState<D>>>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<ListSelectionsResponse>, (StatusCode, Json<serde_json::Value>)> {
    let presentation_id = params.get("presentation_id").cloned().unwrap_or_default();

    let selections = if let Ok(selections_map) = SELECTIONS.read() {
        selections_map
            .get(&presentation_id)
            .cloned()
            .unwrap_or_default()
    } else {
        vec![]
    };

    Ok(Json(ListSelectionsResponse { selections }))
}

#[cfg(test)]
mod tests {
    use super::{collab_presentation_scope, CollabScopeError};

    const ID: &str = "8f14e45f-ceea-467f-a1a5-c4d1d2f4a9d6";

    #[test]
    fn valid_uuid_is_accepted_and_canonicalized() {
        let upper = ID.to_uppercase();
        assert_eq!(
            collab_presentation_scope(&upper, "user-a"),
            Ok(ID.to_string())
        );
        assert_eq!(collab_presentation_scope(ID, "user-a"), Ok(ID.to_string()));
    }

    #[test]
    fn arbitrary_or_empty_key_is_rejected() {
        for key in ["", "current", "../other-deck", "not-a-uuid", "12345"] {
            assert_eq!(
                collab_presentation_scope(key, "user-a"),
                Err(CollabScopeError::InvalidPresentationId),
                "key {key:?} must be rejected"
            );
        }
    }

    #[test]
    fn shared_or_empty_owner_is_rejected() {
        for owner in ["", "   ", "default-user"] {
            assert_eq!(
                collab_presentation_scope(ID, owner),
                Err(CollabScopeError::NoOwnerScope),
                "owner {owner:?} must be rejected"
            );
        }
    }
}
