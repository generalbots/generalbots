/* =============================================================================
 * ui_fragments/minutes_app_forms.rs — POST handlers and ensure_*_table
 * =============================================================================*/
use super::*;
use crate::db;

#[derive(Deserialize)]
pub(super) struct MeetingForm {
    pub title: String,
    pub date: String,
    pub time: String,
    pub duration: u64,
    pub location: Option<String>,
}

pub(super) async fn schedule_meeting(Form(f): Form<MeetingForm>) -> Result<Html<String>, (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool: {e}")))?;
    ensure_meetings_table(&mut conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let id = Uuid::new_v4();
    diesel::sql_query(
        "INSERT INTO minutes_meetings (id, title, date, duration_minutes, location, status, created_at)
         VALUES ($1, $2, $3, $4, $5, 'scheduled', NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(&f.title)
    .bind::<diesel::sql_types::Text, _>(&f.date)
    .bind::<diesel::sql_types::BigInt, _>(f.duration as i64)
    .bind::<diesel::sql_types::Text, _>(f.location.as_deref().unwrap_or("Online"))
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Html(format!(
        r##"<div class="gb-form-result">
<strong>✓ Reunião agendada</strong> — {title} em {date} às {time}
<div hx-get="/suite/minutes/fragments/upcoming" hx-trigger="load delay:200ms" hx-swap="outerHTML" hx-target="#min-upcoming"></div>
</div>"##,
        title = htmx_escape(&f.title),
        date = htmx_escape(&f.date),
        time = htmx_escape(&f.time),
    )))
}

#[derive(Deserialize)]
pub struct ActionForm {
    pub title: String,
    pub owner: String,
    pub due: Option<String>,
    pub priority: Option<String>,
    pub notes: Option<String>,
}

pub(super) async fn create_action(Form(f): Form<ActionForm>) -> Result<Html<String>, (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool: {e}")))?;
    ensure_actions_table(&mut conn).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;
    let id = Uuid::new_v4();
    let due_date = match f.due.as_deref() {
        Some(s) if !s.is_empty() => Some(chrono::NaiveDate::parse_from_str(s, "%Y-%m-%d")
            .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid date: {e}")))?),
        _ => None,
    };
    diesel::sql_query(
        "INSERT INTO minutes_actions (id, title, owner, due, priority, status, notes, created_at)
         VALUES ($1, $2, $3, $4, $5, 'open', $6, NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Text, _>(&f.title)
    .bind::<diesel::sql_types::Text, _>(&f.owner)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Date>, _>(due_date)
    .bind::<diesel::sql_types::Text, _>(f.priority.as_deref().unwrap_or("medium"))
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(f.notes.as_deref())
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(Html(r#"<div class="gb-form-result"><strong>✓ Ação criada</strong></div>"#.to_string()))
}

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

pub(super) async fn sign_document(Path(id): Path<String>) -> Result<Html<String>, (StatusCode, String)> {
    Ok(Html(format!(
        r##"<div class="gb-form-result"><strong>✓ Assinatura registrada</strong> para o documento {id}</div>"##,
        id = htmx_escape(&id),
    )))
}

pub(super) fn ensure_meetings_table(conn: &mut diesel::PgConnection) -> Result<(), String> {
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS minutes_meetings (
            id UUID PRIMARY KEY,
            title TEXT NOT NULL,
            date TEXT NOT NULL,
            duration_minutes BIGINT NOT NULL DEFAULT 30,
            location TEXT NOT NULL DEFAULT 'Online',
            status TEXT NOT NULL DEFAULT 'scheduled',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(conn)
    .map_err(|e| format!("{e}"))?;
    Ok(())
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
