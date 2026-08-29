/* =============================================================================
 * ui_fragments/minutes_app_forms.rs — POST handlers and ensure_*_table
 * =============================================================================*/
use super::*;
use crate::db;

pub(super) async fn complete_action(Path(id): Path<String>) -> Result<Html<String>, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id '{id}': {e}")))?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool: {e}")))?;
    ensure_actions_table(&mut conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    diesel::sql_query("UPDATE minutes_actions SET status = 'done' WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(parsed)
        .execute(&mut conn)
        .map_err(db::map_diesel_err)?;
    Ok(Html(r#"<span class="gb-ok">✓ Concluída</span>"#.to_string()))
}

pub(super) async fn update_document(Path(id): Path<String>) -> Result<Html<String>, (StatusCode, String)> {
    let parsed = Uuid::parse_str(&id)
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid id '{id}': {e}")))?;
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool: {e}")))?;
    ensure_documents_table(&mut conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    diesel::sql_query("UPDATE minutes_documents SET updated_at = NOW() WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(parsed)
        .execute(&mut conn)
        .map_err(db::map_diesel_err)?;
    Ok(Html(r#"<span class="gb-ok">✓ Atualizado</span>"#.to_string()))
}

pub(super) fn ensure_actions_table(conn: &mut diesel::PgConnection) -> Result<(), String> {
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS minutes_actions (
            id UUID PRIMARY KEY,
            title TEXT NOT NULL,
            owner TEXT NOT NULL DEFAULT '',
            due DATE,
            priority TEXT NOT NULL DEFAULT 'medium',
            status TEXT NOT NULL DEFAULT 'open',
            notes TEXT,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(conn)
    .map_err(|e| format!("{e}"))?;
    Ok(())
}

pub(super) fn ensure_documents_table(conn: &mut diesel::PgConnection) -> Result<(), String> {
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS minutes_documents (
            id UUID PRIMARY KEY,
            meeting_id TEXT,
            title TEXT NOT NULL,
            content TEXT NOT NULL DEFAULT '',
            version BIGINT NOT NULL DEFAULT 1,
            status TEXT NOT NULL DEFAULT 'draft',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(conn)
    .map_err(|e| format!("{e}"))?;
    Ok(())
}
