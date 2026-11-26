use axum::extract::Json;
use botsecurity_auth::auth_api::types::AuthenticatedUser;
use diesel::prelude::*;
use serde_json::Value;
use uuid::Uuid;

use crate::error::IntegrationError;

#[derive(diesel::QueryableByName)]
struct DefaultBotRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
}

/// #2960 — an org-scoped caller must never receive a bot id from another
/// tenant: the integrations app feeds that id straight into
/// `/api/bots/{id}/integration-connections`, where `resolve_scope` correctly
/// answers 403. Selecting the first active bot across ALL organizations made
/// a signed-in user of another org open the seeded `default` bot and see a
/// spurious 403 on every connected-services refresh.
const DEFAULT_BOT_FOR_ORG_SQL: &str =
    "SELECT id, name FROM bots WHERE is_active = TRUE AND org_id = $1 \
     ORDER BY created_at ASC LIMIT 1";

/// Org-less resolution is kept only for principals that legitimately span
/// organizations (platform admins) or run a single-user desktop install.
const DEFAULT_BOT_ANY_SQL: &str =
    "SELECT id, name FROM bots WHERE is_active = TRUE ORDER BY created_at ASC LIMIT 1";

/// Resolve the shared pool, mapping the tuple error of `db::pool()` onto the
/// control-plane error type without leaking the raw message.
fn shared_pool() -> Result<&'static botcore::shared::utils::DbPool, IntegrationError> {
    crate::db::pool().map_err(|(status, message)| {
        log::error!("integration context: database pool unavailable ({status}): {message}");
        IntegrationError::Storage("database pool unavailable".to_string())
    })
}

/// The verified org claim of the caller, when present. A nil uuid is the
/// "no org scope" marker, not an organization id.
fn caller_org(user: &AuthenticatedUser) -> Option<Uuid> {
    user.organization_id.filter(|org| *org != Uuid::nil())
}

/// Whether an org-less caller may resolve the global first-active bot.
/// Roleless authenticated users must not be handed a foreign bot id; the
/// platform admin and the `no_security` desktop owner legitimately span orgs.
fn may_resolve_any_bot(user: &AuthenticatedUser) -> bool {
    user.is_admin()
        || user.is_super_admin()
        || user.metadata.get("desktop_local").map(String::as_str) == Some("true")
}

fn default_bot_row(
    pool: &botcore::shared::utils::DbPool,
    user: &AuthenticatedUser,
) -> Result<Option<DefaultBotRow>, IntegrationError> {
    let mut conn = pool.get()?;
    match caller_org(user) {
        Some(org) => Ok(diesel::sql_query(DEFAULT_BOT_FOR_ORG_SQL)
            .bind::<diesel::sql_types::Uuid, _>(org)
            .get_result::<DefaultBotRow>(&mut conn)
            .optional()?),
        None if may_resolve_any_bot(user) => Ok(diesel::sql_query(DEFAULT_BOT_ANY_SQL)
            .get_result::<DefaultBotRow>(&mut conn)
            .optional()?),
        // Fail closed with the documented "no workspace" payload instead of
        // an unscoped bot id that the connection routes would reject.
        None => Ok(None),
    }
}

/// Build the response from a server-resolved row only. The client never
/// supplies the bot id; `connections_url` is derived from the same row.
fn context_payload(bot: Option<DefaultBotRow>) -> Value {
    let Some(bot) = bot else {
        return serde_json::json!({
            "bot_id": Value::Null,
            "bot_name": "",
            "connections_url": Value::Null,
            "reason": "no_active_bot",
        });
    };
    serde_json::json!({
        "bot_id": bot.id,
        "bot_name": bot.name,
        "connections_url": format!("/api/bots/{}/integration-connections", bot.id),
        "reason": Value::Null,
    })
}

/// GET /api/apps/integrations/context
///
/// Resolves the default active bot for the caller's organization so suite
/// apps can build tenant-scoped connection URLs without hardcoding bot ids.
/// Same resolution approach as the WhatsApp channel bootstrap: first active
/// bot by creation order, but never across a verified org boundary (#2960).
/// Vault-independent by design - this handler reads only the database and
/// never touches the secrets manager.
///
/// #2100 — an installation with no active bot (a fresh desktop profile)
/// answers `200` with the documented no-workspace payload
/// `{ "bot_id": null, "reason": "no_active_bot" }`. It used to answer `404`,
/// which the Integrations app reported as "Status unavailable" instead of the
/// honest "no bot/workspace yet".
pub async fn context(user: AuthenticatedUser) -> Result<Json<Value>, IntegrationError> {
    let pool = shared_pool()?;
    let bot = default_bot_row(pool, &user)?;
    if bot.is_some() {
        log::debug!("integration context resolved for caller {}", user.user_id);
    } else {
        log::debug!(
            "integration context: no active bot for caller {}",
            user.user_id
        );
    }
    Ok(Json(context_payload(bot)))
}

#[cfg(test)]
mod context_scope_tests {
    use super::*;

    fn row() -> DefaultBotRow {
        DefaultBotRow {
            id: Uuid::parse_str("f47ac10b-58cc-4372-a567-0e02b2c3d480").expect("fixture bot id"),
            name: "default".to_string(),
        }
    }

    #[test]
    fn caller_org_ignores_missing_and_nil_claims() {
        let user = AuthenticatedUser::new(Uuid::new_v4(), "member@example.com".to_string());
        assert_eq!(caller_org(&user), None);
        assert_eq!(
            caller_org(&user.clone().with_organization(Uuid::nil())),
            None
        );
        let org = Uuid::new_v4();
        assert_eq!(caller_org(&user.with_organization(org)), Some(org));
    }

    #[test]
    fn only_cross_org_principals_may_resolve_a_global_default() {
        let member = AuthenticatedUser::new(Uuid::new_v4(), "member@example.com".to_string());
        assert!(!may_resolve_any_bot(&member));
        let admin = member.clone().with_role(botsecurity_auth::Role::Admin);
        assert!(may_resolve_any_bot(&admin));
        let desktop = AuthenticatedUser::desktop_local();
        assert!(may_resolve_any_bot(&desktop));
    }

    #[test]
    fn scoped_query_binds_the_caller_org_and_the_fallback_is_org_free() {
        assert!(DEFAULT_BOT_FOR_ORG_SQL.contains("org_id = $1"));
        assert!(DEFAULT_BOT_FOR_ORG_SQL.contains("is_active = TRUE"));
        assert!(DEFAULT_BOT_FOR_ORG_SQL.contains("ORDER BY created_at ASC LIMIT 1"));
        assert!(!DEFAULT_BOT_ANY_SQL.contains("org_id"));
    }

    #[test]
    fn payload_never_exposes_a_bot_the_server_did_not_resolve() {
        let empty = context_payload(None);
        assert!(empty["bot_id"].is_null());
        assert!(empty["connections_url"].is_null());
        assert_eq!(empty["reason"], "no_active_bot");

        let resolved = context_payload(Some(row()));
        assert_eq!(resolved["bot_name"], "default");
        assert_eq!(
            resolved["connections_url"],
            "/api/bots/f47ac10b-58cc-4372-a567-0e02b2c3d480/integration-connections"
        );
        assert!(resolved["reason"].is_null());
    }
}
