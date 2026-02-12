use crate::core::shared::models::UserSession;
use crate::core::shared::state::AppState;
use diesel::prelude::*;
use log::{error, info, warn};
use rhai::{Dynamic, Engine, EvalAltResult};
use std::sync::Arc;
use uuid::Uuid;

#[derive(QueryableByName)]
struct BotNameResult {
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
}

#[derive(QueryableByName)]
struct KbCollectionResult {
    #[diesel(sql_type = diesel::sql_types::Text)]
    folder_path: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    qdrant_collection: String,
}

#[derive(QueryableByName, Debug, Clone)]
pub struct ActiveKbResult {
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub kb_name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub kb_folder_path: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub qdrant_collection: String,
}

pub fn register_use_kb_keyword(
    engine: &mut Engine,
    state: Arc<AppState>,
    session: Arc<UserSession>,
) -> Result<(), Box<EvalAltResult>> {
    let state_clone = Arc::clone(&state);
    let session_clone = Arc::clone(&session);

    engine.register_custom_syntax(["USE", "KB", "$expr$"], true, move |context, inputs| {
        let kb_name = context.eval_expression_tree(&inputs[0])?.to_string();

        info!(
            "USE KB keyword executed - KB: {}, Session: {}",
            kb_name, session_clone.id
        );

        let session_id = session_clone.id;
        let bot_id = session_clone.bot_id;
        let conn = state_clone.conn.clone();
        let kb_name_clone = kb_name.clone();

        let result =
            std::thread::spawn(move || add_kb_to_session(conn, session_id, bot_id, &kb_name_clone))
                .join();

        match result {
            Ok(Ok(_)) => {
                info!(" KB '{}' added to session {}", kb_name, session_clone.id);
                Ok(Dynamic::UNIT)
            }
            Ok(Err(e)) => {
                error!("Failed to add KB '{}': {}", kb_name, e);
                Err(format!("USE_KB failed: {}", e).into())
            }
            Err(e) => {
                error!("Thread panic in USE_KB: {:?}", e);
                Err("USE_KB failed: thread panic".into())
            }
        }
    })?;

    Ok(())
}

fn add_kb_to_session(
    conn_pool: crate::core::shared::utils::DbPool,
    session_id: Uuid,
    bot_id: Uuid,
    kb_name: &str,
) -> Result<(), String> {
    let mut conn = conn_pool
        .get()
        .map_err(|e| format!("Failed to get DB connection: {}", e))?;

    let bot_result: BotNameResult = diesel::sql_query("SELECT name FROM bots WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .get_result(&mut conn)
        .map_err(|e| format!("Failed to get bot name: {}", e))?;
    let bot_name = bot_result.name;

    let kb_exists: Option<KbCollectionResult> = diesel::sql_query(
        "SELECT folder_path, qdrant_collection FROM kb_collections WHERE bot_id = $1 AND name = $2",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Text, _>(kb_name)
    .get_result(&mut conn)
    .optional()
    .map_err(|e| format!("Failed to check KB existence: {}", e))?;

    let (kb_folder_path, qdrant_collection) = if let Some(kb_result) = kb_exists {
        (kb_result.folder_path, kb_result.qdrant_collection)
    } else {
        let default_path = format!("work/{}/{}.gbkb/{}", bot_name, bot_name, kb_name);
        let default_collection = format!("{}_{}", bot_name, kb_name);

        warn!(
            "KB '{}' not found in kb_collections for bot {}. Using default path: {}",
            kb_name, bot_name, default_path
        );

        let kb_id = Uuid::new_v4();
        diesel::sql_query(
            "INSERT INTO kb_collections (id, bot_id, name, folder_path, qdrant_collection, document_count)
             VALUES ($1, $2, $3, $4, $5, 0)
             ON CONFLICT (bot_id, name) DO NOTHING"
        )
        .bind::<diesel::sql_types::Uuid, _>(kb_id)
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .bind::<diesel::sql_types::Text, _>(kb_name)
        .bind::<diesel::sql_types::Text, _>(&default_path)
        .bind::<diesel::sql_types::Text, _>(&default_collection)
        .execute(&mut conn)
        .ok();

        (default_path, default_collection)
    };

    let tool_name: Option<String> = None;

    let assoc_id = Uuid::new_v4();
    diesel::sql_query(
        "INSERT INTO session_kb_associations (id, session_id, bot_id, kb_name, kb_folder_path, qdrant_collection, added_by_tool, is_active)
         VALUES ($1, $2, $3, $4, $5, $6, $7, true)
         ON CONFLICT (session_id, kb_name)
         DO UPDATE SET
            is_active = true,
            added_at = NOW(),
            added_by_tool = EXCLUDED.added_by_tool"
    )
    .bind::<diesel::sql_types::Uuid, _>(assoc_id)
    .bind::<diesel::sql_types::Uuid, _>(session_id)
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Text, _>(kb_name)
    .bind::<diesel::sql_types::Text, _>(&kb_folder_path)
    .bind::<diesel::sql_types::Text, _>(&qdrant_collection)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(tool_name.as_deref())
    .execute(&mut conn)
    .map_err(|e| format!("Failed to add KB association: {}", e))?;

    info!(
        " Added KB '{}' to session {} (collection: {}, path: {})",
        kb_name, session_id, qdrant_collection, kb_folder_path
    );

    Ok(())
}

pub fn get_active_kbs_for_session(
    conn_pool: &crate::core::shared::utils::DbPool,
    session_id: Uuid,
) -> Result<Vec<(String, String, String)>, String> {
    let mut conn = conn_pool
        .get()
        .map_err(|e| format!("Failed to get DB connection: {}", e))?;

    let results: Vec<ActiveKbResult> = diesel::sql_query(
        "SELECT kb_name, kb_folder_path, qdrant_collection
         FROM session_kb_associations
         WHERE session_id = $1 AND is_active = true
         ORDER BY added_at DESC",
    )
    .bind::<diesel::sql_types::Uuid, _>(session_id)
    .load(&mut conn)
    .map_err(|e| format!("Failed to get active KBs: {}", e))?;

    Ok(results
        .into_iter()
        .map(|r| (r.kb_name, r.kb_folder_path, r.qdrant_collection))
        .collect())
}
