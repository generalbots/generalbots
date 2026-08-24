use crate::models::ConnectionRow;
use crate::knowledge::{Container, KnowledgeConnector, RawItem};
use crate::registry;
use axum::http::StatusCode;
use botcore::shared::state::AppState;
use botcore::shared::utils::DbPool;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde_json::{Map, Value};
use std::sync::Arc;
use uuid::Uuid;

/// Hard ceiling on items indexed per sync run across all containers.
pub const MAX_ITEMS_PER_SYNC: i64 = 2000;

const SELECT_CONNECTION_SQL: &str =
    "SELECT * FROM connector_connections WHERE id = $1";
const UPSERT_ITEM_SQL: &str = "INSERT INTO indexed_items \
    (id, connection_id, external_id, title, body_tsv, vector_ref, acl, container, external_url, updated_at) \
    VALUES ($1, $2, $3, $4, $5, NULL, $6::jsonb, $7, $8, COALESCE($9, NOW())) \
    ON CONFLICT (connection_id, external_id) DO UPDATE SET \
    title = EXCLUDED.title, body_tsv = EXCLUDED.body_tsv, acl = EXCLUDED.acl, \
    container = EXCLUDED.container, external_url = EXCLUDED.external_url, \
    updated_at = EXCLUDED.updated_at, deleted_at = NULL";
const MARK_MISSING_DELETED_SQL: &str = "UPDATE indexed_items SET deleted_at = NOW() \
    WHERE connection_id = $1 AND container = $2 AND deleted_at IS NULL \
    AND NOT (external_id <> ALL (SELECT jsonb_array_elements_text($3::jsonb)))";
const PERSIST_SYNC_SQL: &str = "UPDATE connector_connections \
    SET cursors = $1::jsonb, last_sync_at = NOW(), updated_at = NOW() WHERE id = $2";

#[derive(Debug, Clone, serde::Serialize)]
pub struct SyncOutcome {
    pub connection_id: Uuid,
    pub kind: String,
    pub containers_scanned: usize,
    pub items_upserted: i64,
    pub items_marked_deleted: i64,
    pub budget_exhausted: bool,
}

/// Synchronize one connector connection: iterate every upstream container,
/// upsert discovered items and tombstone externals that disappeared.
///
/// Spawn-safe: blocking HTTP iteration and Diesel I/O run inside
/// `tokio::task::spawn_blocking`; credential resolution is awaited before that.
pub async fn sync_connection(
    state: &Arc<AppState>,
    connection_id: Uuid,
) -> Result<SyncOutcome, (StatusCode, String)> {
    let pool = state.conn.clone();
    let mut conn = pool
        .get()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("DB pool: {e}")))?;
    let row: ConnectionRow = diesel::sql_query(SELECT_CONNECTION_SQL)
        .bind::<diesel::sql_types::Uuid, _>(connection_id)
        .get_result(&mut conn)
        .optional()
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Load connection: {e}")))?
        .ok_or_else(|| {
            (StatusCode::NOT_FOUND, format!("Connector connection {connection_id} not found"))
        })?;
    drop(conn);

    if row.status != "connected" {
        return Err((StatusCode::CONFLICT, format!("Connection is '{}' and cannot sync", row.status)));
    }

    let credentials =
        registry::resolve_credentials(&row.vault_token_ref).await.map_err(|e| {
            tracing::error!("botconnectors: credential resolution failed for {connection_id}: {e}");
            (StatusCode::INTERNAL_SERVER_ERROR, "Connector credentials unavailable".to_string())
        })?;

    let outcome = tokio::task::spawn_blocking(move || run_sync(pool, &row, credentials))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Sync task join: {e}")))?
        .map_err(|e| {
            tracing::error!("botconnectors: sync of {connection_id} failed: {e}");
            (StatusCode::BAD_GATEWAY, format!("Connector sync failed: {e}"))
        })?;
    Ok(outcome)
}

fn run_sync(
    pool: DbPool,
    row: &ConnectionRow,
    credentials: Value,
) -> Result<SyncOutcome, String> {
    let connector: Arc<dyn KnowledgeConnector> =
        registry::connector_for_kind(&row.kind).ok_or_else(|| format!("Unknown connector kind '{}'", row.kind))?;
    let mut db = pool.get().map_err(|e| format!("DB pool: {e}"))?;

    let containers = connector.list_containers(&credentials)?;
    let mut cursors: Map<String, Value> =
        row.cursors.as_object().cloned().unwrap_or_default();

    let mut budget = MAX_ITEMS_PER_SYNC;
    let mut budget_exhausted = false;
    let mut items_upserted: i64 = 0;
    let mut items_marked_deleted: i64 = 0;

    for container in &containers {
        if budget <= 0 {
            budget_exhausted = true;
            tracing::warn!(
                "botconnectors: sync budget of {MAX_ITEMS_PER_SYNC} exhausted for connection {}",
                row.id
            );
            break;
        }
        let previous_cursor = cursor_since(cursors.get(&container.id));
        let items = connector.iter_items(&credentials, container, previous_cursor)?;

        let mut seen: Vec<String> = Vec::new();
        let mut max_updated = previous_cursor;
        for item in &items {
            if budget <= 0 {
                budget_exhausted = true;
                tracing::warn!(
                    "botconnectors: container '{}' truncated at sync budget for connection {}",
                    container.id,
                    row.id
                );
                break;
            }
            upsert_item(&mut db, row.id, item)?;
            seen.push(item.external_id.clone());
            if let Some(ts) = item.updated_at {
                max_updated = Some(match max_updated {
                    Some(prev) => prev.max(ts),
                    None => ts,
                });
            }
            budget -= 1;
            items_upserted += 1;
        }

        // Tombstone only when the container was fully iterated; a truncated
        // pass must never mark unscanned externals as deleted.
        if !budget_exhausted {
            items_marked_deleted += mark_missing_deleted(&mut db, row.id, container, &seen)?;
        }

        if let Some(cursor) = max_updated {
            cursors.insert(container.id.clone(), Value::String(cursor.to_rfc3339()));
        }
    }

    diesel::sql_query(PERSIST_SYNC_SQL)
        .bind::<diesel::sql_types::Text, _>(Value::Object(cursors).to_string())
        .bind::<diesel::sql_types::Uuid, _>(row.id)
        .execute(&mut db)
        .map_err(|e| format!("Persist cursors: {e}"))?;

    Ok(SyncOutcome {
        connection_id: row.id,
        kind: row.kind.clone(),
        containers_scanned: containers.len(),
        items_upserted,
        items_marked_deleted,
        budget_exhausted,
    })
}

fn cursor_since(raw: Option<&Value>) -> Option<DateTime<Utc>> {
    let text = raw.and_then(Value::as_str).filter(|s| !s.is_empty())?;
    DateTime::parse_from_rfc3339(text).ok().map(|t| t.with_timezone(&Utc))
}

fn upsert_item(db: &mut PgConnection, connection_id: Uuid, item: &RawItem) -> Result<(), String> {
    let acl_json = Value::Array(
        item.acl_principals.iter().map(|p| Value::String(p.clone())).collect(),
    );
    diesel::sql_query(UPSERT_ITEM_SQL)
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Uuid, _>(connection_id)
        .bind::<diesel::sql_types::Text, _>(&item.external_id)
        .bind::<diesel::sql_types::Text, _>(&item.title)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&item.body)
        .bind::<diesel::sql_types::Text, _>(acl_json.to_string())
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&item.container_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&item.external_url)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(item.updated_at)
        .execute(db)
        .map_err(|e| format!("Upsert item '{}': {e}", item.external_id))?;
    Ok(())
}

fn mark_missing_deleted(
    db: &mut PgConnection,
    connection_id: Uuid,
    container: &Container,
    seen: &[String],
) -> Result<i64, String> {
    // An empty `seen` from a successful iteration means every previously
    // indexed external of this container vanished upstream.
    let affected = if seen.is_empty() {
        diesel::sql_query(
            "UPDATE indexed_items SET deleted_at = NOW() \
             WHERE connection_id = $1 AND container = $2 AND deleted_at IS NULL",
        )
        .bind::<diesel::sql_types::Uuid, _>(connection_id)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&container.id)
        .execute(db)
        .map_err(|e| format!("Mark all missing for container '{}': {e}", container.id))?
    } else {
        let seen_json = Value::Array(seen.iter().map(|id| Value::String(id.clone())).collect());
        diesel::sql_query(MARK_MISSING_DELETED_SQL)
            .bind::<diesel::sql_types::Uuid, _>(connection_id)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&container.id)
            .bind::<diesel::sql_types::Text, _>(seen_json.to_string())
            .execute(db)
            .map_err(|e| format!("Mark missing deleted for container '{}': {e}", container.id))?
    };
    Ok(affected as i64)
}
