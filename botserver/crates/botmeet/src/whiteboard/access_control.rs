//! Pure, DB-free authorization helpers for the whiteboard service (#2740).
//!
//! Callers resolve *verified* conversation memberships from the database first
//! and hand the resulting UUIDs to [`WhiteboardPrincipal::can_access_conversation`].
//! Keeping that decision pure makes it unit-testable without a running server,
//! so the router stays unmounted until the guards are proven (see the note in
//! `super`).

use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Uuid as DieselUuid;
use std::sync::Arc;
use uuid::Uuid;

use botcore::shared::state::AppState;
use botlib::db_pool::DbPool;
use botsecurity::auth::AuthenticatedUser;

/// Caller identity presented at the whiteboard handshake/export boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhiteboardPrincipal {
    pub user_id: Uuid,
    pub username: String,
    pub authenticated: bool,
}

impl WhiteboardPrincipal {
    pub fn from_authenticated(user: &AuthenticatedUser) -> Self {
        Self {
            user_id: user.user_id,
            username: user.username.clone(),
            authenticated: user.is_authenticated(),
        }
    }

    /// Cheap preconditions checked before any database work.
    pub fn has_verified_identity(&self, conversation_id: Uuid) -> bool {
        self.authenticated && !self.user_id.is_nil() && !conversation_id.is_nil()
    }

    /// True only for an authenticated caller whose verified membership set
    /// contains the target conversation. `member_conversations` MUST come from
    /// the membership store, never from client input; a foreign or nil
    /// conversation id is therefore always refused.
    pub fn can_access_conversation(
        &self,
        conversation_id: Uuid,
        member_conversations: &[Uuid],
    ) -> bool {
        self.has_verified_identity(conversation_id)
            && member_conversations.contains(&conversation_id)
    }
}

/// A live socket must inherit the handshake identity. A caller that injects a
/// different `user_id` into a whiteboard message is attempting identity
/// spoofing and must be ignored.
pub fn socket_identity_matches(claimed_user_id: Uuid, handshake_user_id: Uuid) -> bool {
    !handshake_user_id.is_nil() && claimed_user_id == handshake_user_id
}

#[derive(diesel::QueryableByName)]
struct MemberConversationRow {
    #[diesel(sql_type = DieselUuid)]
    conversation_id: Uuid,
}

/// Conversation ids where `user_id` is an active member or the owner. Errors
/// propagate so callers fail closed rather than granting access on a DB error.
pub(crate) fn verified_member_conversations(
    pool: &DbPool,
    user_id: Uuid,
) -> Result<Vec<Uuid>, String> {
    let mut conn = pool.get().map_err(|e| format!("db pool: {e}"))?;
    let rows: Vec<MemberConversationRow> = sql_query(
        "SELECT conversation_id FROM meet_conversation_members
          WHERE user_id = $1 AND status = 'active'
          UNION
         SELECT id AS conversation_id FROM meet_conversations
          WHERE created_by = $1",
    )
    .bind::<DieselUuid, _>(user_id)
    .get_results(&mut conn)
    .map_err(|e| format!("membership query: {e}"))?;
    Ok(rows.into_iter().map(|r| r.conversation_id).collect())
}

/// Full handshake/export gate: resolve verified memberships, then apply the
/// pure predicate. Any database failure denies access.
pub(crate) async fn caller_can_access(
    state: &Arc<AppState>,
    principal: &WhiteboardPrincipal,
    conversation_id: Uuid,
) -> bool {
    if !principal.has_verified_identity(conversation_id) {
        return false;
    }
    let pool = state.conn.clone();
    let user_id = principal.user_id;
    let lookup =
        tokio::task::spawn_blocking(move || verified_member_conversations(&pool, user_id)).await;
    match lookup {
        Ok(Ok(member_conversations)) => {
            principal.can_access_conversation(conversation_id, &member_conversations)
        }
        Ok(Err(e)) => {
            log::error!("Whiteboard membership lookup failed: {e}");
            false
        }
        Err(e) => {
            log::error!("Whiteboard membership lookup task failed: {e}");
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn principal(user_id: Uuid, authenticated: bool) -> WhiteboardPrincipal {
        WhiteboardPrincipal {
            user_id,
            username: "tester".to_string(),
            authenticated,
        }
    }

    #[test]
    fn member_of_conversation_is_allowed() {
        let user_id = Uuid::new_v4();
        let conversation_id = Uuid::new_v4();
        let p = principal(user_id, true);
        assert!(p.can_access_conversation(conversation_id, &[conversation_id]));
    }

    #[test]
    fn foreign_conversation_is_refused() {
        let user_id = Uuid::new_v4();
        let member = Uuid::new_v4();
        let foreign = Uuid::new_v4();
        let p = principal(user_id, true);
        assert!(!p.can_access_conversation(foreign, &[member]));
    }

    #[test]
    fn unauthenticated_and_nil_identities_are_refused() {
        let user_id = Uuid::new_v4();
        let conversation_id = Uuid::new_v4();
        assert!(
            !principal(user_id, false).can_access_conversation(conversation_id, &[conversation_id])
        );
        assert!(!principal(Uuid::nil(), true)
            .can_access_conversation(conversation_id, &[conversation_id]));
        assert!(!principal(user_id, true).can_access_conversation(Uuid::nil(), &[Uuid::nil()]));
    }

    #[test]
    fn anonymous_authenticated_user_is_refused() {
        let conversation_id = Uuid::new_v4();
        let principal = WhiteboardPrincipal::from_authenticated(&AuthenticatedUser::anonymous());
        assert!(!principal.can_access_conversation(conversation_id, &[conversation_id]));
    }

    #[test]
    fn empty_membership_set_is_refused() {
        let user_id = Uuid::new_v4();
        let conversation_id = Uuid::new_v4();
        assert!(!principal(user_id, true).can_access_conversation(conversation_id, &[]));
    }

    #[test]
    fn socket_identity_must_match_handshake() {
        let handshake = Uuid::new_v4();
        assert!(socket_identity_matches(handshake, handshake));
        assert!(!socket_identity_matches(Uuid::new_v4(), handshake));
        assert!(!socket_identity_matches(handshake, Uuid::nil()));
    }
}
