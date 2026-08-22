use botcore::shared::utils::DbPool;
use botsecurity_auth::auth_api::types::AuthenticatedUser;
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::IntegrationError;

/// Fully resolved tenant scope for every integration connection operation.
///
/// The scope is derived exclusively from the server-minted authenticated
/// user extension plus the database row of the referenced bot. Client
/// supplied headers or query parameters are never trusted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectionScope {
    pub user_id: Uuid,
    pub org_id: Uuid,
    pub branch_id: Uuid,
    pub bot_id: Uuid,
}

impl ConnectionScope {
    /// The connection owner is always the authenticated caller.
    pub fn owner_user_id(&self) -> Uuid {
        self.user_id
    }
}

#[derive(diesel::QueryableByName)]
struct BotScopeRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    bot_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    org_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: Uuid,
}

/// Resolves the tenant scope for `bot_id` from the database and validates it
/// against the authenticated caller.
///
/// A single SQL statement joins `bots` to `branches` (the organizations table
/// keys on `org_id`, so it needs no join). The bot and its branch must both be
/// active, the branch must belong to the bot organization, and when the caller
/// carries an organization id it must match the bot organization. Any failure
/// to establish the organization is treated as a scope violation (fail closed).
pub fn resolve_scope(
    pool: &DbPool,
    authenticated_user: &AuthenticatedUser,
    bot_id: Uuid,
) -> Result<ConnectionScope, IntegrationError> {
    let caller_org = authenticated_user
        .organization_id
        .filter(|org| *org != Uuid::nil())
        .ok_or(IntegrationError::UnauthorizedScope)?;

    let mut conn = pool.get()?;
    let row: BotScopeRow = diesel::sql_query(
        "SELECT b.id AS bot_id, b.org_id, b.branch_id \
         FROM bots b \
         INNER JOIN branches br ON br.id = b.branch_id AND br.is_active = TRUE AND br.org_id = b.org_id \
         WHERE b.id = $1 AND b.is_active = TRUE \
         LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .get_result(&mut conn)
    .optional()?
    .ok_or(IntegrationError::NotFound)?;

    if row.org_id != caller_org {
        return Err(IntegrationError::UnauthorizedScope);
    }

    Ok(ConnectionScope {
        user_id: authenticated_user.user_id,
        org_id: row.org_id,
        branch_id: row.branch_id,
        bot_id: row.bot_id,
    })
}
