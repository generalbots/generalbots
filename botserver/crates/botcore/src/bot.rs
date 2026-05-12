pub fn get_default_bot(_conn: &mut diesel::PgConnection) -> (uuid::Uuid, String) {
    (uuid::Uuid::nil(), "default".to_string())
}

pub fn get_bot_config(_bot_id: &str) -> Option<serde_json::Value> {
    None
}
