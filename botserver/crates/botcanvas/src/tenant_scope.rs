//! Tenant and ownership enforcement for the canvas collaboration REST layer
//! (#2735).
//!
//! Every handler resolves the caller's tenant from `get_bot_context` — never
//! from the request — and passes the loaded canvas through the pure
//! [`authorize_canvas_action`] predicate. A canvas that is not in that tenant
//! is indistinguishable from one that does not exist (`404`); a caller who is
//! neither the canvas owner nor an editor cannot mutate (`403`).

use super::*;

pub(crate) type CanvasApiError = (StatusCode, String);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanvasAction {
    ListCollaborators,
    AddCollaborator,
    RemoveCollaborator,
    ListComments,
    CreateComment,
    ResolveComment,
    ListVersions,
    CreateVersion,
}

impl CanvasAction {
    fn is_mutation(self) -> bool {
        matches!(
            self,
            Self::AddCollaborator
                | Self::RemoveCollaborator
                | Self::CreateComment
                | Self::ResolveComment
                | Self::CreateVersion
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CanvasAuthzError {
    NotFound,
    Forbidden,
}

/// Pure ownership predicate: both tenant columns must match the verified
/// context, and an unresolved (nil) context can never own anything.
pub(crate) fn canvas_in_scope(canvas: &DbCanvas, org_id: Uuid, bot_id: Uuid) -> bool {
    !org_id.is_nil()
        && !bot_id.is_nil()
        && canvas.org_id == org_id
        && canvas.bot_id == bot_id
}

/// Only an explicit owner/editor grant authorizes a mutation. Unknown or
/// viewer permissions fail closed.
fn permission_allows_mutation(permission: Option<&str>) -> bool {
    let normalized = permission.map(|value| value.trim().to_ascii_lowercase());
    matches!(normalized.as_deref(), Some("owner" | "edit" | "editor"))
}

/// Owner (creator) or a stored editor grant on this canvas. Grants from
/// another canvas or another user never count.
fn may_mutate(canvas: &DbCanvas, caller: Uuid, grants: &[DbCanvasCollaborator]) -> bool {
    if caller.is_nil() {
        return false;
    }
    if !canvas.created_by.is_nil() && canvas.created_by == caller {
        return true;
    }
    grants.iter().any(|grant| {
        grant.canvas_id == canvas.id
            && grant.user_id == caller
            && permission_allows_mutation(Some(&grant.permission))
    })
}

/// The single authorization decision shared by all collaboration handlers.
pub(crate) fn authorize_canvas_action(
    canvas: Option<&DbCanvas>,
    action: CanvasAction,
    caller: Uuid,
    grants: &[DbCanvasCollaborator],
    org_id: Uuid,
    bot_id: Uuid,
) -> Result<(), CanvasAuthzError> {
    let Some(canvas) = canvas else {
        return Err(CanvasAuthzError::NotFound);
    };
    if !canvas_in_scope(canvas, org_id, bot_id) {
        return Err(CanvasAuthzError::NotFound);
    }
    if action.is_mutation() && !may_mutate(canvas, caller, grants) {
        return Err(CanvasAuthzError::Forbidden);
    }
    Ok(())
}

pub(crate) fn authorization_error(error: CanvasAuthzError) -> CanvasApiError {
    match error {
        CanvasAuthzError::NotFound => (StatusCode::NOT_FOUND, "Canvas not found".to_string()),
        CanvasAuthzError::Forbidden => (
            StatusCode::FORBIDDEN,
            "Canvas owner or editor permission required".to_string(),
        ),
    }
}

/// Loads a canvas only inside the verified tenant context. Foreign or unknown
/// ids both produce `404`, so a caller cannot probe another tenant's rows.
pub(crate) fn load_scoped_canvas(
    conn: &mut diesel::PgConnection,
    canvas_id: Uuid,
    org_id: Uuid,
    bot_id: Uuid,
) -> Result<DbCanvas, CanvasApiError> {
    let canvas: DbCanvas = canvases::table
        .filter(canvases::id.eq(canvas_id))
        .filter(canvases::org_id.eq(org_id))
        .filter(canvases::bot_id.eq(bot_id))
        .first(conn)
        .map_err(|_| authorization_error(CanvasAuthzError::NotFound))?;
    Ok(canvas)
}

pub(crate) fn load_canvas_grants(
    conn: &mut diesel::PgConnection,
    canvas_id: Uuid,
) -> Result<Vec<DbCanvasCollaborator>, CanvasApiError> {
    canvas_collaborators::table
        .filter(canvas_collaborators::canvas_id.eq(canvas_id))
        .load(conn)
        .map_err(|error| (StatusCode::INTERNAL_SERVER_ERROR, format!("Query error: {error}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    const ORG_A: Uuid = Uuid::from_u128(0x1000);
    const BOT_A: Uuid = Uuid::from_u128(0x1001);
    const ORG_B: Uuid = Uuid::from_u128(0x2000);
    const BOT_B: Uuid = Uuid::from_u128(0x2001);
    const ALICE: Uuid = Uuid::from_u128(0xa11ce);
    const BOB: Uuid = Uuid::from_u128(0xb0b);

    fn canvas(org_id: Uuid, bot_id: Uuid, owner: Uuid) -> DbCanvas {
        let now = Utc::now();
        DbCanvas {
            id: Uuid::new_v4(),
            branch_id: bot_id,
            name: "board".to_string(),
            description: None,
            width: 1920,
            height: 1080,
            background_color: None,
            thumbnail_url: None,
            is_public: false,
            is_template: false,
            created_by: owner,
            created_at: now,
            updated_at: now,
            org_id,
            bot_id,
        }
    }

    fn grant(canvas_id: Uuid, user_id: Uuid, permission: &str) -> DbCanvasCollaborator {
        DbCanvasCollaborator {
            id: Uuid::new_v4(),
            canvas_id,
            user_id,
            permission: permission.to_string(),
            added_by: Some(ALICE),
            added_at: Utc::now(),
        }
    }

    fn decide(
        target: &DbCanvas,
        action: CanvasAction,
        caller: Uuid,
        grants: &[DbCanvasCollaborator],
        org_id: Uuid,
        bot_id: Uuid,
    ) -> Result<(), CanvasAuthzError> {
        authorize_canvas_action(Some(target), action, caller, grants, org_id, bot_id)
    }

    #[test]
    fn a_canvas_belongs_only_to_its_own_tenant() {
        let own = canvas(ORG_A, BOT_A, ALICE);
        let foreign = canvas(ORG_B, BOT_B, BOB);
        assert!(canvas_in_scope(&own, ORG_A, BOT_A));
        assert!(!canvas_in_scope(&own, ORG_B, BOT_B));
        assert!(!canvas_in_scope(&foreign, ORG_A, BOT_A));
        assert!(!canvas_in_scope(&own, Uuid::nil(), BOT_A));
        assert!(!canvas_in_scope(&own, ORG_A, Uuid::nil()));
    }

    #[test]
    fn list_is_scoped_for_both_tenants() {
        let own = canvas(ORG_A, BOT_A, ALICE);
        let foreign = canvas(ORG_B, BOT_B, BOB);
        assert_eq!(decide(&own, CanvasAction::ListCollaborators, ALICE, &[], ORG_A, BOT_A), Ok(()));
        assert_eq!(
            decide(&own, CanvasAction::ListCollaborators, BOB, &[], ORG_B, BOT_B),
            Err(CanvasAuthzError::NotFound)
        );
        assert_eq!(
            decide(&foreign, CanvasAction::ListCollaborators, ALICE, &[], ORG_A, BOT_A),
            Err(CanvasAuthzError::NotFound)
        );
    }

    #[test]
    fn add_requires_owner_or_editor_and_ignores_a_forged_grantee() {
        let own = canvas(ORG_A, BOT_A, ALICE);
        let forged_grantee = BOB;
        assert_ne!(forged_grantee, ALICE, "the body may name a different user");
        // The caller is always the verified id, never the body's user_id.
        assert_eq!(decide(&own, CanvasAction::AddCollaborator, ALICE, &[], ORG_A, BOT_A), Ok(()));
        assert_eq!(
            decide(&own, CanvasAction::AddCollaborator, Uuid::nil(), &[], ORG_A, BOT_A),
            Err(CanvasAuthzError::Forbidden)
        );
        let viewer = [grant(own.id, BOB, "view")];
        assert_eq!(
            decide(&own, CanvasAction::AddCollaborator, BOB, &viewer, ORG_A, BOT_A),
            Err(CanvasAuthzError::Forbidden)
        );
        let editor = [grant(own.id, BOB, "editor")];
        assert_eq!(decide(&own, CanvasAction::AddCollaborator, BOB, &editor, ORG_A, BOT_A), Ok(()));
        // Grants on another canvas never confer permission here.
        let other = [grant(Uuid::new_v4(), BOB, "owner")];
        assert_eq!(
            decide(&own, CanvasAction::AddCollaborator, BOB, &other, ORG_A, BOT_A),
            Err(CanvasAuthzError::Forbidden)
        );
    }

    #[test]
    fn comment_read_write_and_resolve_are_scoped() {
        let own = canvas(ORG_A, BOT_A, ALICE);
        let foreign = canvas(ORG_B, BOT_B, BOB);
        // Reads are open to any authenticated user inside the tenant; only
        // mutations require owner/editor. Foreign-tenant rows are always 404.
        assert_eq!(decide(&own, CanvasAction::ListComments, ALICE, &[], ORG_A, BOT_A), Ok(()));
        assert_eq!(decide(&own, CanvasAction::ListComments, BOB, &[], ORG_A, BOT_A), Ok(()));
        assert_eq!(
            decide(&foreign, CanvasAction::ListComments, ALICE, &[], ORG_A, BOT_A),
            Err(CanvasAuthzError::NotFound)
        );
        for action in [CanvasAction::CreateComment, CanvasAction::ResolveComment] {
            assert_eq!(decide(&own, action, ALICE, &[], ORG_A, BOT_A), Ok(()));
            assert_eq!(
                decide(&foreign, action, ALICE, &[], ORG_A, BOT_A),
                Err(CanvasAuthzError::NotFound)
            );
            assert_eq!(
                decide(&own, action, BOB, &[], ORG_A, BOT_A),
                Err(CanvasAuthzError::Forbidden)
            );
        }
    }

    #[test]
    fn version_read_and_create_are_scoped() {
        let own = canvas(ORG_A, BOT_A, ALICE);
        let foreign = canvas(ORG_B, BOT_B, BOB);
        // Listing versions is a read (open to in-tenant users); creating one
        // is a mutation (owner/editor only).
        assert_eq!(decide(&own, CanvasAction::ListVersions, ALICE, &[], ORG_A, BOT_A), Ok(()));
        assert_eq!(decide(&own, CanvasAction::ListVersions, BOB, &[], ORG_A, BOT_A), Ok(()));
        assert_eq!(decide(&foreign, CanvasAction::ListVersions, BOB, &[], ORG_B, BOT_B), Ok(()));
        assert_eq!(
            decide(&foreign, CanvasAction::ListVersions, ALICE, &[], ORG_A, BOT_A),
            Err(CanvasAuthzError::NotFound)
        );
        assert_eq!(decide(&own, CanvasAction::CreateVersion, ALICE, &[], ORG_A, BOT_A), Ok(()));
        assert_eq!(decide(&foreign, CanvasAction::CreateVersion, BOB, &[], ORG_B, BOT_B), Ok(()));
        assert_eq!(
            decide(&foreign, CanvasAction::CreateVersion, ALICE, &[], ORG_A, BOT_A),
            Err(CanvasAuthzError::NotFound)
        );
        assert_eq!(
            decide(&own, CanvasAction::CreateVersion, BOB, &[], ORG_A, BOT_A),
            Err(CanvasAuthzError::Forbidden)
        );
    }

    #[test]
    fn removing_a_collaborator_uses_the_same_ownership_rule() {
        let own = canvas(ORG_A, BOT_A, ALICE);
        assert_eq!(
            decide(&own, CanvasAction::RemoveCollaborator, BOB, &[], ORG_A, BOT_A),
            Err(CanvasAuthzError::Forbidden)
        );
        assert_eq!(decide(&own, CanvasAction::RemoveCollaborator, ALICE, &[], ORG_A, BOT_A), Ok(()));
    }

    #[test]
    fn status_mapping_distinguishes_foreign_from_unauthorized() {
        assert_eq!(authorization_error(CanvasAuthzError::NotFound).0, StatusCode::NOT_FOUND);
        assert_eq!(authorization_error(CanvasAuthzError::Forbidden).0, StatusCode::FORBIDDEN);
    }
}
