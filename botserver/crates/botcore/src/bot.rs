#[derive(diesel::QueryableByName, Debug)]
struct BotRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: uuid::Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
}

pub fn get_default_bot(conn: &mut diesel::PgConnection) -> (uuid::Uuid, String) {
    use diesel::RunQueryDsl;
    let result: Option<BotRow> = diesel::sql_query(
        "SELECT id, name FROM bots ORDER BY created_at ASC LIMIT 1"
    )
    .get_result(conn)
    .ok();
    match result {
        Some(row) => (row.id, row.name),
        None => (uuid::Uuid::nil(), "default".to_string()),
    }
}

pub fn get_bot_config(_bot_id: &str) -> Option<serde_json::Value> {
    None
}
