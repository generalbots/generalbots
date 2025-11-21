use crate::basic::compiler::AstNode;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use diesel::prelude::*;
use log::{error, info, warn};
use rhai::{Dynamic, Engine, EvalAltResult, Position};
use std::sync::Arc;
use uuid::Uuid;

/// Register ADD_KB keyword
/// Adds a Knowledge Base to the current session's context
/// Usage: ADD_KB "kbname"
/// Example: ADD_KB "circular" or ADD_KB kbname (where kbname is a variable)
pub fn register_add_kb_keyword(
    engine: &mut Engine,
    state: Arc<AppState>,
    session: Arc<UserSession>,
) -> Result<(), Box<EvalAltResult>> {
    let state_clone = Arc::clone(&state);
    let session_clone = Arc::clone(&session);

    engine.register_custom_syntax(&["ADD_KB", "$expr$"], true, move |context, inputs| {
        let kb_name = context.eval_expression_tree(&inputs[0])?.to_string();

        info!(
            "ADD_KB keyword executed - KB: {}, Session: {}",
            kb_name, session_clone.id
        );

        let session_id = session_clone.id;
        let bot_id = session_clone.bot_id;
        let conn = state_clone.conn.clone();

        // Execute in blocking context since we're working with database
        let result =
            std::thread::spawn(move || add_kb_to_session(conn, session_id, bot_id, &kb_name))
                .join();

        match result {
            Ok(Ok(_)) => {
                info!("✅ KB '{}' added to session {}", kb_name, session_clone.id);
                Ok(Dynamic::UNIT)
            }
            Ok(Err(e)) => {
                error!("Failed to add KB '{}': {}", kb_name, e);
                Err(format!("ADD_KB failed: {}", e).into())
            }
            Err(e) => {
                error!("Thread panic in ADD_KB: {:?}", e);
                Err("ADD_KB failed: thread panic".into())
            }
        }
    })?;

    Ok(())
}

/// Add KB to session in database
fn add_kb_to_session(
    conn_pool: crate::shared::utils::DbPool,
    session_id: Uuid,
    bot_id: Uuid,
    kb_name: &str,
) -> Result<(), String> {
    let mut conn = conn_pool
        .get()
        .map_err(|e| format!("Failed to get DB connection: {}", e))?;

    // Get bot name to construct KB path
    let bot_name: String = diesel::sql_query("SELECT name FROM bots WHERE id = $1")
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .get_result::<(String,)>(&mut conn)
        .map_err(|e| format!("Failed to get bot name: {}", e))?
        .0;

    // Check if KB collection exists
    let kb_exists: Option<(String, String)> = diesel::sql_query(
        "SELECT folder_path, qdrant_collection FROM kb_collections WHERE bot_id = $1 AND name = $2",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Text, _>(kb_name)
    .get_result::<(String, String)>(&mut conn)
    .optional()
    .map_err(|e| format!("Failed to check KB existence: {}", e))?;

    let (kb_folder_path, qdrant_collection) = if let Some((path, collection)) = kb_exists {
        (path, collection)
    } else {
        // KB doesn't exist in database, construct default path
        let default_path = format!("work/{}/{}.gbkb/{}", bot_name, bot_name, kb_name);
        let default_collection = format!("{}_{}", bot_name, kb_name);

        warn!(
            "KB '{}' not found in kb_collections for bot {}. Using default path: {}",
            kb_name, bot_name, default_path
        );

        // Optionally create KB collection entry
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
        .ok(); // Ignore errors if it already exists

        (default_path, default_collection)
    };

    // Get the tool name from call stack if available
    let tool_name = std::env::var("CURRENT_TOOL_NAME").ok();

    // Add or update KB association for this session
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
        "✅ Added KB '{}' to session {} (collection: {}, path: {})",
        kb_name, session_id, qdrant_collection, kb_folder_path
    );

    Ok(())
}

/// Get all active KBs for a session
pub fn get_active_kbs_for_session(
    conn_pool: &crate::shared::utils::DbPool,
    session_id: Uuid,
) -> Result<Vec<(String, String, String)>, String> {
    let mut conn = conn_pool
        .get()
        .map_err(|e| format!("Failed to get DB connection: {}", e))?;

    let results: Vec<(String, String, String)> = diesel::sql_query(
        "SELECT kb_name, kb_folder_path, qdrant_collection
         FROM session_kb_associations
         WHERE session_id = $1 AND is_active = true
         ORDER BY added_at DESC",
    )
    .bind::<diesel::sql_types::Uuid, _>(session_id)
    .load(&mut conn)
    .map_err(|e| format!("Failed to get active KBs: {}", e))?;

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_kb_syntax() {
        let mut engine = Engine::new();
        // This would normally use real state and session
        // For now just test that the syntax can be registered
        assert!(engine
            .register_custom_syntax(&["ADD_KB", "$expr$"], true, |_, _| Ok(Dynamic::UNIT))
            .is_ok());
    }
}
