use super::*;

pub(super) fn error_create(intent: &str, e: &dyn std::error::Error) -> Json<CreateAndExecuteResponse> {
    let msg = err_msg("create_and_execute", e);
    warn!("create failed for intent: {intent}");
    Json(CreateAndExecuteResponse {
        success: false,
        task_id: String::new(),
        status: "failed".to_string(),
        message: msg.clone(),
        app_url: None,
        created_resources: Vec::new(),
        pending_items: Vec::new(),
        error: Some(msg),
    })
}
/// Insert an `auto_tasks` row recording a created automation. The row is what
/// makes an AutoTask item traceable: the list UI reads these rows, and the
/// ✏️ action opens the `.bas` it produced.
pub(crate) fn record_auto_task(
    pool: &DbPool,
    bot_id: Uuid,
    title: &str,
    intent: &str,
    status: &str,
    script_path: Option<&str>,
) {
    // branch_id is NOT NULL (6.5.23); resolve it from the bot's row.
    let outcome = pool.get().ok().map(|mut conn| {
        let branch_id: Option<Uuid> = sql_query("SELECT branch_id AS id FROM bots WHERE id = $1")
            .bind::<DieselUuid, _>(bot_id)
            .get_result::<BotBranchRow>(&mut conn)
            .ok()
            .and_then(|r| r.id);
        sql_query(
            "INSERT INTO auto_tasks (bot_id, branch_id, title, intent, status, basic_program, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, NOW(), NOW()) RETURNING id",
        )
        .bind::<DieselUuid, _>(bot_id)
        .bind::<NullableSql<DieselUuid>, _>(branch_id)
        .bind::<Text, _>(title)
        .bind::<Text, _>(intent)
        .bind::<Text, _>(status)
        .bind::<NullableSql<Text>, _>(script_path)
        .get_result::<TaskIdRow>(&mut conn)
    });
    match outcome {
        Some(Ok(row)) => info!("Recorded auto_tasks row {} for '{title}'", row.id),
        Some(Err(e)) => warn!("Failed to record auto_tasks row for '{title}': {e}"),
        None => warn!("DB pool unavailable — auto_tasks row for '{title}' not recorded"),
    }
}
