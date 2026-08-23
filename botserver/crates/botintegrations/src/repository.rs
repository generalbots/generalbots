use chrono::{DateTime, Utc};
use diesel::prelude::*;
use uuid::Uuid;

use crate::error::IntegrationError;
use crate::models::ConnectionEvent;
use crate::rows::{ConnectionRow, EventRow};
use crate::scope::ConnectionScope;

/// Audit event payload handed to [`record_event`]; metadata is sanitized
/// through the allowlist filter before insertion.
pub struct NewConnectionEvent<'a> {
    pub connection_id: Option<Uuid>,
    pub actor_user_id: Uuid,
    pub event_type: &'a str,
    pub outcome: &'a str,
    pub risk_level: &'a str,
    pub metadata: &'a serde_json::Value,
}

/// Inserts a new active connection row. The caller supplies the generated
/// connection id together with the pre-built canonical vault path.
pub struct NewConnectionInsert<'a> {
    pub connection_id: Uuid,
    pub provider_slug: &'a str,
    pub display_name: &'a str,
    pub auth_kind: &'a str,
    pub vault_path: &'a str,
    pub granted_scopes: &'a serde_json::Value,
    pub configuration: &'a serde_json::Value,
    pub visibility: &'a str,
    pub expires_at: Option<DateTime<Utc>>,
}

pub fn list(
    conn: &mut PgConnection,
    scope: &ConnectionScope,
    include_revoked: bool,
) -> Result<Vec<ConnectionRow>, IntegrationError> {
    let statement = if include_revoked {
        "SELECT * FROM integration_connections \
         WHERE org_id = $1 AND branch_id = $2 AND bot_id = $3 AND owner_user_id = $4 \
         ORDER BY created_at DESC LIMIT 500"
    } else {
        "SELECT * FROM integration_connections \
         WHERE org_id = $1 AND branch_id = $2 AND bot_id = $3 AND owner_user_id = $4 \
         AND status <> 'revoked' \
         ORDER BY created_at DESC LIMIT 500"
    };
    Ok(diesel::sql_query(statement)
        .bind::<diesel::sql_types::Uuid, _>(scope.org_id)
        .bind::<diesel::sql_types::Uuid, _>(scope.branch_id)
        .bind::<diesel::sql_types::Uuid, _>(scope.bot_id)
        .bind::<diesel::sql_types::Uuid, _>(scope.owner_user_id())
        .load(conn)?)
}

pub fn get(
    conn: &mut PgConnection,
    scope: &ConnectionScope,
    connection_id: Uuid,
) -> Result<Option<ConnectionRow>, IntegrationError> {
    Ok(diesel::sql_query(
        "SELECT * FROM integration_connections \
         WHERE id = $5 AND org_id = $1 AND branch_id = $2 AND bot_id = $3 AND owner_user_id = $4 \
         LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(scope.org_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.branch_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.bot_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.owner_user_id())
    .bind::<diesel::sql_types::Uuid, _>(connection_id)
    .get_result(conn)
    .optional()?)
}

pub fn find_active_by_provider(
    conn: &mut PgConnection,
    scope: &ConnectionScope,
    provider_slug: &str,
) -> Result<Option<ConnectionRow>, IntegrationError> {
    Ok(diesel::sql_query(
        "SELECT * FROM integration_connections \
         WHERE provider_slug = $5 AND status = 'active' \
         AND org_id = $1 AND branch_id = $2 AND bot_id = $3 \
           AND (owner_user_id = $4 OR visibility = 'branch') \
         ORDER BY CASE WHEN owner_user_id = $4 THEN 0 ELSE 1 END \
         LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(scope.org_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.branch_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.bot_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.owner_user_id())
    .bind::<diesel::sql_types::Text, _>(provider_slug)
    .get_result(conn)
    .optional()?)
}

pub fn insert(
    conn: &mut PgConnection,
    scope: &ConnectionScope,
    new_connection: &NewConnectionInsert<'_>,
) -> Result<(), IntegrationError> {
    diesel::sql_query(
        "INSERT INTO integration_connections \
         (id, org_id, branch_id, bot_id, owner_user_id, provider_slug, display_name, auth_kind, status, vault_path, granted_scopes, visibility, configuration, expires_at) \
         VALUES ($5, $1, $2, $3, $4, $6, $7, $8, 'active', $9, $10, $12, $11, $13)",
    )
    .bind::<diesel::sql_types::Uuid, _>(scope.org_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.branch_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.bot_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.owner_user_id())
    .bind::<diesel::sql_types::Uuid, _>(new_connection.connection_id)
    .bind::<diesel::sql_types::Text, _>(new_connection.provider_slug)
    .bind::<diesel::sql_types::Text, _>(new_connection.display_name)
    .bind::<diesel::sql_types::Text, _>(new_connection.auth_kind)
    .bind::<diesel::sql_types::Text, _>(new_connection.vault_path)
    .bind::<diesel::sql_types::Jsonb, _>(new_connection.granted_scopes)
    .bind::<diesel::sql_types::Text, _>(new_connection.visibility)
    .bind::<diesel::sql_types::Jsonb, _>(new_connection.configuration)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(
        new_connection.expires_at,
    )
    .execute(conn)?;
    Ok(())
}

/// Appends an audit event after sanitizing the metadata payload through the
/// strict allowlist filter (`crate::metadata::sanitize_metadata`).
pub fn record_event(
    conn: &mut PgConnection,
    scope: &ConnectionScope,
    event: &NewConnectionEvent<'_>,
) -> Result<(), IntegrationError> {
    diesel::sql_query(
        "INSERT INTO integration_connection_events \
         (connection_id, org_id, branch_id, bot_id, owner_user_id, actor_user_id, event_type, outcome, risk_level, metadata) \
         VALUES ($5, $1, $2, $3, $4, $6, $7, $8, $9, $10)",
    )
    .bind::<diesel::sql_types::Uuid, _>(scope.org_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.branch_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.bot_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.owner_user_id())
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(event.connection_id)
    .bind::<diesel::sql_types::Uuid, _>(event.actor_user_id)
    .bind::<diesel::sql_types::Text, _>(event.event_type)
    .bind::<diesel::sql_types::Text, _>(event.outcome)
    .bind::<diesel::sql_types::Text, _>(event.risk_level)
    .bind::<diesel::sql_types::Jsonb, _>(&crate::metadata::sanitize_metadata(event.metadata))
    .execute(conn)?;
    Ok(())
}

pub fn mark_tested(
    conn: &mut PgConnection,
    scope: &ConnectionScope,
    connection_id: Uuid,
    test_status: &str,
) -> Result<bool, IntegrationError> {
    let affected = diesel::sql_query(
        "UPDATE integration_connections \
         SET last_tested_at = NOW(), last_test_status = $5, updated_at = NOW() \
         WHERE id = $6 AND org_id = $1 AND branch_id = $2 AND bot_id = $3 AND owner_user_id = $4",
    )
    .bind::<diesel::sql_types::Uuid, _>(scope.org_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.branch_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.bot_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.owner_user_id())
    .bind::<diesel::sql_types::Text, _>(test_status)
    .bind::<diesel::sql_types::Uuid, _>(connection_id)
    .execute(conn)?;
    Ok(affected > 0)
}

pub fn mark_revoked(
    conn: &mut PgConnection,
    scope: &ConnectionScope,
    connection_id: Uuid,
) -> Result<bool, IntegrationError> {
    let affected = diesel::sql_query(
        "UPDATE integration_connections \
         SET status = 'revoked', revoked_at = NOW(), updated_at = NOW() \
         WHERE id = $5 AND status <> 'revoked' \
         AND org_id = $1 AND branch_id = $2 AND bot_id = $3 AND owner_user_id = $4",
    )
    .bind::<diesel::sql_types::Uuid, _>(scope.org_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.branch_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.bot_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.owner_user_id())
    .bind::<diesel::sql_types::Uuid, _>(connection_id)
    .execute(conn)?;
    Ok(affected > 0)
}

pub fn increment_credential_version(
    conn: &mut PgConnection,
    scope: &ConnectionScope,
    connection_id: Uuid,
) -> Result<i64, IntegrationError> {
    #[derive(QueryableByName)]
    struct VersionRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        credential_version: i64,
    }
    let row: VersionRow = diesel::sql_query(
        "UPDATE integration_connections \
         SET credential_version = credential_version + 1, last_refreshed_at = NOW(), updated_at = NOW() \
         WHERE id = $5 AND org_id = $1 AND branch_id = $2 AND bot_id = $3 AND owner_user_id = $4 \
         RETURNING credential_version",
    )
    .bind::<diesel::sql_types::Uuid, _>(scope.org_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.branch_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.bot_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.owner_user_id())
    .bind::<diesel::sql_types::Uuid, _>(connection_id)
    .get_result(conn)?;
    Ok(row.credential_version)
}

pub fn delete_row(
    conn: &mut PgConnection,
    scope: &ConnectionScope,
    connection_id: Uuid,
) -> Result<bool, IntegrationError> {
    let affected = diesel::sql_query(
        "DELETE FROM integration_connections \
         WHERE id = $5 AND org_id = $1 AND branch_id = $2 AND bot_id = $3 AND owner_user_id = $4",
    )
    .bind::<diesel::sql_types::Uuid, _>(scope.org_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.branch_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.bot_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.owner_user_id())
    .bind::<diesel::sql_types::Uuid, _>(connection_id)
    .execute(conn)?;
    Ok(affected > 0)
}

pub fn list_events(
    conn: &mut PgConnection,
    scope: &ConnectionScope,
    connection_id: Uuid,
) -> Result<Vec<ConnectionEvent>, IntegrationError> {
    Ok(diesel::sql_query(
        "SELECT id, connection_id, event_type, outcome, risk_level, metadata, created_at \
         FROM integration_connection_events \
         WHERE connection_id = $5 AND org_id = $1 AND branch_id = $2 AND bot_id = $3 AND owner_user_id = $4 \
         ORDER BY created_at DESC LIMIT 200",
    )
    .bind::<diesel::sql_types::Uuid, _>(scope.org_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.branch_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.bot_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.owner_user_id())
    .bind::<diesel::sql_types::Uuid, _>(connection_id)
    .load::<EventRow>(conn)?
    .into_iter()
    .map(Into::into)
    .collect())
}
