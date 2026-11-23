//! #2796 — authorization for `GET /api/chat/models`. The `bot` query param is
//! an untrusted selector; these helpers decide whether the caller may read a
//! resolved bot's model metadata. The rule mirrors the bot tree's membership
//! check (platform admins see all, everyone else public + own-org bots).

use axum::http::StatusCode;
use axum::response::{IntoResponse, Json, Response};
use uuid::Uuid;

/// Pure predicate: platform admins may read any bot; other callers only public
/// bots or bots of an organization they belong to.
pub(super) fn bot_visible_to_caller(
    is_platform_admin: bool,
    bot_is_public: bool,
    bot_org_id: Option<Uuid>,
    caller_org_ids: &[Uuid],
) -> bool {
    is_platform_admin
        || bot_is_public
        || bot_org_id.is_some_and(|org| caller_org_ids.contains(&org))
}

/// Mirrors `bot_tree::caller_is_platform_admin`: an explicit super admin, or a
/// member of the reserved root org `default`, may see every tenant's bots.
pub(super) fn root_org_member(conn: &mut diesel::pg::PgConnection, email: &str) -> bool {
    use diesel::RunQueryDsl;

    #[derive(diesel::QueryableByName)]
    #[diesel(check_for_backend(diesel::pg::Pg))]
    struct CountRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        count: i64,
    }

    diesel::sql_query(
        "SELECT COUNT(*) AS count FROM user_organizations uo \
         JOIN organizations o ON o.org_id = uo.org_id \
         JOIN users u ON u.id = uo.user_id \
         WHERE lower(u.email) = lower($1) AND o.slug = 'default'",
    )
    .bind::<diesel::sql_types::Text, _>(email)
    .get_result::<CountRow>(conn)
    .map(|row| row.count > 0)
    .unwrap_or(false)
}

pub(super) fn chat_models_not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"error": "Bot not found"})),
    )
        .into_response()
}

#[cfg(test)]
mod tests {
    use super::bot_visible_to_caller;
    use uuid::Uuid;

    /// #2796 — a two-tenant boundary: tenant A's member reaches tenant A's
    /// private bot, never tenant B's; public bots and platform admins are the
    /// only other ways across the boundary.
    #[test]
    fn models_access_is_scoped_to_caller_tenants() {
        let tenant_a = Uuid::new_v4();
        let tenant_b = Uuid::new_v4();
        let a_member = vec![tenant_a];
        let b_member = vec![tenant_b];

        // Same tenant: allowed. Cross tenant: denied.
        assert!(bot_visible_to_caller(
            false,
            false,
            Some(tenant_a),
            &a_member
        ));
        assert!(!bot_visible_to_caller(
            false,
            false,
            Some(tenant_a),
            &b_member
        ));

        // Public bot is readable by either tenant and by anonymous callers.
        assert!(bot_visible_to_caller(
            false,
            true,
            Some(tenant_a),
            &b_member
        ));
        assert!(bot_visible_to_caller(false, true, Some(tenant_a), &[]));

        // Platform admin reaches any tenant's private bot; an unauthenticated
        // caller with no public flag and no membership is rejected.
        assert!(bot_visible_to_caller(
            true,
            false,
            Some(tenant_b),
            &a_member
        ));
        assert!(!bot_visible_to_caller(false, false, Some(tenant_b), &[]));
    }
}
