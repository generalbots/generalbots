//! #2452 — org-wide share-link inventory.
//!
//! `GET /api/collab/links` is resource-scoped, so an admin cannot answer "what
//! external links exist in this org?". This module adds a read-only inventory
//! over the same `resource_links` rows and the same `LinkItem` DTO. It does not
//! add a bulk revoke: the returned `token` is the identifier accepted by the
//! existing `DELETE /api/collab/links/:token` handler, so a single screen can
//! list every link and revoke any row one token at a time.
//!
//! Tenant scope is derived only from verified claims: `AuthenticatedUser`
//! supplies the admin flags, the org id and the caller identity. No org id is
//! read from the query string or the JSON body.

use super::*;

/// Admin scope: links minted by the caller (the admin may be represented only
/// by a JWT identity) plus links minted by any user who is a member of the
/// caller's authenticated org. Membership is verified in the database, never
/// taken from the request. `$1` is the org id, `$2` the caller identity.
const ORGANIZATION_LINKS_SQL: &str = "SELECT token, role, expires_at, created_at, allow_download FROM resource_links l \
     WHERE (l.created_by = $2 OR EXISTS ( \
         SELECT 1 FROM users u \
         JOIN user_organizations uo ON uo.user_id = u.id \
         WHERE (u.id::text = l.created_by OR LOWER(u.email) = LOWER(l.created_by)) \
           AND uo.org_id = $1 \
     )) ORDER BY created_at DESC";

/// Ordinary scope: only links the caller minted. There is deliberately no org
/// predicate here — a non-admin must never see another user's links, even when
/// both users belong to the same organization.
const OWN_LINKS_SQL: &str = "SELECT token, role, expires_at, created_at, allow_download FROM resource_links \
     WHERE created_by = $1 ORDER BY created_at DESC";

/// The scope an inventory request is allowed to use. `None` means the caller
/// has no usable scope and the handler must fail closed with 403.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LinkInventoryScope<'a> {
    Organization { org_id: uuid::Uuid },
    Own { created_by: &'a str },
}

/// Pure scope projection. Admins get the org-wide view only when their
/// verified principal carries a real (non-nil) organization; platform admins
/// without an org scope are denied rather than silently given global access.
/// Non-admins are restricted to their own links regardless of any org claim.
pub(crate) fn link_inventory_scope(
    is_admin: bool,
    org_id: Option<uuid::Uuid>,
    created_by: &str,
) -> Option<LinkInventoryScope<'_>> {
    if is_admin {
        match org_id {
            Some(org_id) if org_id != uuid::Uuid::nil() => {
                Some(LinkInventoryScope::Organization { org_id })
            }
            _ => None,
        }
    } else {
        Some(LinkInventoryScope::Own { created_by })
    }
}

/// `GET /api/collab/links/all` — list links visible to the caller's scope.
/// The response is the existing `LinkItem` shape; each `token` can be passed to
/// `DELETE /api/collab/links/:token` to revoke that one link.
pub async fn list_all_links(
    State(state): State<Arc<AppState>>,
    Extension(user): Extension<AuthenticatedUser>,
    request_locale: botcore::i18n::RequestLocale,
) -> Result<Json<Vec<LinkItem>>, (StatusCode, Json<serde_json::Value>)> {
    let locale = request_locale.locale();
    let created_by = collab_user_id(&user);
    let is_admin = user.is_admin() || user.is_super_admin();
    let Some(scope) = link_inventory_scope(is_admin, user.organization_id, &created_by) else {
        return Err(err(
            locale,
            StatusCode::FORBIDDEN,
            "error-collab-owner-only",
        ));
    };
    let mut conn = state
        .conn
        .get()
        .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let rows = match scope {
        LinkInventoryScope::Organization { org_id } => diesel::sql_query(ORGANIZATION_LINKS_SQL)
            .bind::<SqlUuid, _>(org_id)
            .bind::<Text, _>(&created_by)
            .load::<LinkRow>(&mut conn),
        LinkInventoryScope::Own { created_by } => diesel::sql_query(OWN_LINKS_SQL)
            .bind::<Text, _>(created_by)
            .load::<LinkRow>(&mut conn),
    }
    .map_err(|e| err_db(locale, StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(
        rows.into_iter()
            .map(|r| LinkItem {
                token: r.token,
                role: r.role,
                expires_at: r.expires_at.map(|d| d.to_rfc3339()),
                created_at: r.created_at.to_rfc3339(),
                allow_download: r.allow_download,
            })
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn admin_scope_uses_the_authenticated_org_and_fails_closed_without_it() {
        let org = uuid::Uuid::from_u128(42);
        assert_eq!(
            link_inventory_scope(true, Some(org), "alice@example.test"),
            Some(LinkInventoryScope::Organization { org_id: org })
        );
        assert_eq!(link_inventory_scope(true, None, "alice@example.test"), None);
        assert_eq!(
            link_inventory_scope(true, Some(uuid::Uuid::nil()), "alice@example.test"),
            None
        );
        // The org id is bound from the principal, never from a request field.
        assert!(ORGANIZATION_LINKS_SQL.contains("uo.org_id = $1"));
        assert!(ORGANIZATION_LINKS_SQL.contains("l.created_by = $2"));
        assert!(ORGANIZATION_LINKS_SQL.contains("JOIN user_organizations"));
        assert!(!ORGANIZATION_LINKS_SQL.contains("$3"));
    }

    #[test]
    fn ordinary_callers_only_see_links_they_minted() {
        let org = uuid::Uuid::from_u128(7);
        let creator = "bob@example.test";
        assert_eq!(
            link_inventory_scope(false, Some(org), creator),
            Some(LinkInventoryScope::Own {
                created_by: creator
            })
        );
        assert!(OWN_LINKS_SQL.contains("WHERE created_by = $1"));
        assert!(!OWN_LINKS_SQL.contains("user_organizations"));
        assert!(!OWN_LINKS_SQL.contains("org_id"));
        // Both branches project the same existing LinkRow/LinkItem shape.
        for sql in [ORGANIZATION_LINKS_SQL, OWN_LINKS_SQL] {
            assert!(sql.contains(
                "SELECT token, role, expires_at, created_at, allow_download FROM resource_links"
            ));
        }
    }
}
