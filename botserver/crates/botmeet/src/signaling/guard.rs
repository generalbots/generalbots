//! Room-membership guard for the meet signaling WebSocket (#2734).
//!
//! The signaling route used to be globally authenticated only: any logged-in
//! user could upgrade `/ws/meet/{room_id}`, self-assert any `participant_id`
//! in `Join`, receive other rooms' events and forge screen-share/routing
//! signals. This module performs the membership check *before* the upgrade
//! and binds the connection identity to the verified `AuthenticatedUser`.

use axum::{
    extract::{ws::WebSocketUpgrade, Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use log::warn;
use std::sync::Arc;
use uuid::Uuid;

use botcore::shared::state::AppState;
use botsecurity::auth::AuthenticatedUser;

use super::{handle_authenticated_signaling_socket, new_signaling_rooms, SignalingRooms};

/// A single `meeting_participants` fact reduced to its two security-relevant
/// columns: the room and the verified user that belongs to it.
pub(crate) type RoomMembership = (Uuid, Uuid);

/// Pure membership predicate: the caller may join `room_id` only when its
/// **verified** `user_id` appears in that room's membership rows.
///
/// Keeping this pure (no DB, no async, no I/O) makes the security decision
/// directly testable, including the two-room isolation cases in the unit
/// tests below.
pub(crate) fn is_room_member(members: &[RoomMembership], room_id: Uuid, user_id: Uuid) -> bool {
    members
        .iter()
        .any(|(room, member)| *room == room_id && *member == user_id)
}

/// Pure routing predicate: a signal may be forwarded to `target_id` only when
/// that id belongs to the *same* room. Ids from a different room are rejected
/// before the in-memory lookup, so cross-room routing is never possible.
pub(crate) fn target_in_room(participant_ids: &[String], target_id: &str) -> bool {
    participant_ids.iter().any(|id| id == target_id)
}

/// Identity bound to a signaling connection by the server. It is derived
/// from `AuthenticatedUser` at upgrade time; the client-supplied
/// `participant_id`/`participant_name` in `Join` is never trusted.
#[derive(Debug, Clone)]
pub(crate) struct ParticipantIdentity {
    pub(crate) id: String,
    pub(crate) name: String,
}

async fn resolve_room_id(state: &Arc<AppState>, room: &str) -> Result<Option<Uuid>, String> {
    if let Ok(id) = Uuid::parse_str(room) {
        return Ok(Some(id));
    }

    use botschema::meeting_rooms::dsl::{id as room_pk, meeting_rooms, room_code};
    use diesel::prelude::*;

    let pool = state.conn.clone();
    let code = room.to_string();
    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("db pool: {e}"))?;
        meeting_rooms
            .filter(room_code.eq(code))
            .select(room_pk)
            .first::<Uuid>(&mut conn)
            .optional()
            .map_err(|e| format!("room lookup: {e}"))
    })
    .await
    .map_err(|e| format!("room lookup task: {e}"))?
}

async fn load_room_members(
    state: &Arc<AppState>,
    room: Uuid,
) -> Result<Vec<RoomMembership>, String> {
    use botschema::meeting_participants::dsl::{
        meeting_participants, room_id as room_col, user_id as user_col,
    };
    use diesel::prelude::*;

    let pool = state.conn.clone();
    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| format!("db pool: {e}"))?;
        let rows = meeting_participants
            .filter(room_col.eq(room))
            .filter(user_col.is_not_null())
            .select((room_col, user_col))
            .load::<(Uuid, Option<Uuid>)>(&mut conn)
            .map_err(|e| format!("membership query: {e}"))?;
        Ok(rows
            .into_iter()
            .filter_map(|(r, user)| user.map(|user| (r, user)))
            .collect())
    })
    .await
    .map_err(|e| format!("membership task: {e}"))?
}

fn forbidden(message: &str) -> Response {
    (
        StatusCode::FORBIDDEN,
        Json(serde_json::json!({ "error": message })),
    )
        .into_response()
}

fn server_error(e: &str, context: &str) -> Response {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(serde_json::json!({
            "error": botsecurity::error_sanitizer::log_and_sanitize_json(e, context, None)
        })),
    )
        .into_response()
}

/// Guarded WebSocket entry point used by the route in `configure.rs`.
///
/// The upgrade is refused unless the caller is an active member of the room
/// (looked up in `meeting_participants`, by room UUID or invite/room code).
/// On success the participant id/name are bound to the verified identity and
/// passed to the signaling loop, so a client can no longer self-assert them.
pub async fn authenticated_signaling_websocket(
    ws: WebSocketUpgrade,
    Path(room_id): Path<String>,
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
) -> Response {
    let room = match resolve_room_id(&state, &room_id).await {
        Ok(Some(room)) => room,
        Ok(None) => return forbidden("Unknown meeting room"),
        Err(e) => return server_error(&e, "botmeet::signaling::resolve_room"),
    };

    let members = match load_room_members(&state, room).await {
        Ok(members) => members,
        Err(e) => return server_error(&e, "botmeet::signaling::load_room_members"),
    };

    if !is_room_member(&members, room, user.user_id) {
        warn!(
            "Rejected signaling join for room {room_id}: user {} is not a member",
            user.user_id
        );
        return forbidden("Not a member of this meeting room");
    }

    let id = user.user_id.to_string();
    let name = if user.username.is_empty() {
        user.email.clone().unwrap_or_else(|| id.clone())
    } else {
        user.username.clone()
    };
    let identity = ParticipantIdentity { id, name };

    let rooms = state
        .extensions
        .get::<SignalingRooms>()
        .await
        .map(|arc| (*arc).clone())
        .unwrap_or_else(new_signaling_rooms);

    ws.on_upgrade(move |socket| {
        handle_authenticated_signaling_socket(socket, room_id, state, rooms, identity)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOM_A: &str = "11111111-1111-1111-1111-111111111111";
    const ROOM_B: &str = "22222222-2222-2222-2222-222222222222";
    const USER_A: &str = "aaaaaaaa-aaaa-aaaa-aaaa-aaaaaaaaaaaa";
    const USER_B: &str = "bbbbbbbb-bbbb-bbbb-bbbb-bbbbbbbbbbbb";

    fn room(uuid_str: &str) -> Uuid {
        Uuid::parse_str(uuid_str).expect("test UUID must parse")
    }

    #[test]
    fn member_of_room_is_accepted() {
        let members = vec![(room(ROOM_A), room(USER_A))];
        assert!(is_room_member(&members, room(ROOM_A), room(USER_A)));
    }

    #[test]
    fn member_of_a_different_room_is_rejected() {
        // Two-room isolation: USER_A belongs to ROOM_A only. Asking about
        // ROOM_B must never grant membership, even though the user is a
        // legitimate participant elsewhere.
        let members = vec![(room(ROOM_A), room(USER_A)), (room(ROOM_B), room(USER_B))];
        assert!(!is_room_member(&members, room(ROOM_B), room(USER_A)));
        assert!(!is_room_member(&members, room(ROOM_A), room(USER_B)));
    }

    #[test]
    fn unknown_user_is_rejected() {
        let members = vec![(room(ROOM_A), room(USER_A))];
        assert!(!is_room_member(&members, room(ROOM_A), room(USER_B)));
    }

    #[test]
    fn empty_membership_list_rejects_everyone() {
        assert!(!is_room_member(&[], room(ROOM_A), room(USER_A)));
    }

    #[test]
    fn target_in_same_room_is_routable() {
        let ids = vec![room(USER_A).to_string(), room(USER_B).to_string()];
        assert!(target_in_room(&ids, &room(USER_B).to_string()));
    }

    #[test]
    fn target_from_other_room_is_not_routable() {
        // A target id that exists only in ROOM_B must be rejected when
        // routing a signal inside ROOM_A.
        let room_a_ids = vec![room(USER_A).to_string()];
        assert!(!target_in_room(&room_a_ids, &room(USER_B).to_string()));
    }
}
