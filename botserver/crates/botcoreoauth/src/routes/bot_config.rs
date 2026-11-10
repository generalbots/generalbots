use super::*;

/// Reads the bot-level OAuth configuration from Vault.
pub(super) async fn get_bot_config(state: &OAuthState_) -> HashMap<String, String> {
    let conn = state.conn.clone();
    let active_bot_id: Uuid = match tokio::task::spawn_blocking(move || {
        use diesel::prelude::*;
        use crate::schema::bots;

        let mut db_conn = conn.get().ok()?;
        bots::dsl::bots
            .filter(bots::dsl::is_active.eq(true))
            .select(bots::dsl::id)
            .first(&mut db_conn)
            .optional()
            .ok()?
    })
    .await
    {
        Ok(Some(id)) => id,
        _ => return HashMap::new(),
    };

    read_bot_config_from_vault(&active_bot_id).unwrap_or_default()
}

pub(super) fn read_bot_config_from_vault(bot_id: &Uuid) -> Option<HashMap<String, String>> {
    use botcoresecrets::manager::SecretsManager;
    let sm = SecretsManager::get_clone().ok()?;
    if !sm.is_enabled() {
        return None;
    }
    let path = format!("gbo/{}/{}/{}", uuid::Uuid::nil(), uuid::Uuid::nil(), bot_id);
    let sm_clone = sm.clone();
    let path_clone = path.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let result = if let Ok(rt) = rt {
            rt.block_on(async move { sm_clone.get_secret(&path_clone).await.ok() })
        } else {
            None
        };
        let _ = tx.send(result);
    });
    rx.recv().ok().flatten()
}
