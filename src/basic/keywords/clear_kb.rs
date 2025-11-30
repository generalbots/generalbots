use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use diesel::prelude::*;
use log::{error, info};
use rhai::{Dynamic, Engine, EvalAltResult};
use std::sync::Arc;
use uuid::Uuid;

#[derive(QueryableByName)]
struct CountResult {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

/// Register CLEAR KB keyword
/// Removes one or all Knowledge Bases from the current session's context
/// Usage:
///   CLEAR KB "kbname"  - Remove specific KB
///   CLEAR KB           - Remove all KBs
pub fn register_clear_kb_keyword(
    engine: &mut Engine,
    state: Arc<AppState>,
    session: Arc<UserSession>,
) -> Result<(), Box<EvalAltResult>> {
    // CLEAR KB with argument - remove specific KB
    let state_clone = Arc::clone(&state);
    let session_clone = Arc::clone(&session);
    // Register with spaces: CLEAR KB "kbname"
    engine.register_custom_syntax(&["CLEAR", "KB", "$expr$"], true, move |context, inputs| {
        let kb_name = context.eval_expression_tree(&inputs[0])?.to_string();

        info!(
            "CLEAR KB keyword executed - KB: {}, Session: {}",
            kb_name, session_clone.id
        );

        let session_id = session_clone.id;
        let conn = state_clone.conn.clone();
        let kb_name_clone = kb_name.clone();

        let result =
            std::thread::spawn(move || clear_specific_kb(conn, session_id, &kb_name_clone)).join();

        match result {
            Ok(Ok(_)) => {
                info!(
                    "✅ KB '{}' removed from session {}",
                    kb_name, session_clone.id
                );
                Ok(Dynamic::UNIT)
            }
            Ok(Err(e)) => {
                error!("Failed to clear KB '{}': {}", kb_name, e);
                Err(format!("CLEAR KB failed: {}", e).into())
            }
            Err(e) => {
                error!("Thread panic in CLEAR KB: {:?}", e);
                Err("CLEAR KB failed: thread panic".into())
            }
        }
    })?;

    // CLEAR KB without argument - remove all KBs
    let state_clone2 = Arc::clone(&state);
    let session_clone2 = Arc::clone(&session);
    // Register with spaces: CLEAR KB (no argument)
    engine.register_custom_syntax(&["CLEAR", "KB"], true, move |_context, _inputs| {
        info!(
            "CLEAR KB (all) keyword executed - Session: {}",
            session_clone2.id
        );

        let session_id = session_clone2.id;
        let conn = state_clone2.conn.clone();

        let result = std::thread::spawn(move || clear_all_kbs(conn, session_id)).join();

        match result {
            Ok(Ok(count)) => {
                // Get the remaining active KB count
                let remaining_count =
                    get_active_kb_count(&state_clone2.conn, session_clone2.id).unwrap_or(0);
                info!(
                    "Successfully cleared {} KB associations for session {}, {} remaining active",
                    count, session_clone2.id, remaining_count
                );
                Ok(Dynamic::UNIT)
            }
            Ok(Err(e)) => {
                error!("Failed to clear all KBs: {}", e);
                Err(format!("CLEAR KB failed: {}", e).into())
            }
            Err(e) => {
                error!("Thread panic in CLEAR KB: {:?}", e);
                Err("CLEAR KB failed: thread panic".into())
            }
        }
    })?;

    Ok(())
}

/// Clear a specific KB from session
fn clear_specific_kb(
    conn_pool: crate::shared::utils::DbPool,
    session_id: Uuid,
    kb_name: &str,
) -> Result<(), String> {
    let mut conn = conn_pool
        .get()
        .map_err(|e| format!("Failed to get DB connection: {}", e))?;

    // Mark KB as inactive (soft delete)
    let rows_affected = diesel::sql_query(
        "UPDATE session_kb_associations
         SET is_active = false
         WHERE session_id = $1 AND kb_name = $2 AND is_active = true",
    )
    .bind::<diesel::sql_types::Uuid, _>(session_id)
    .bind::<diesel::sql_types::Text, _>(kb_name)
    .execute(&mut conn)
    .map_err(|e| format!("Failed to clear KB: {}", e))?;

    // Get the remaining active KB count after clearing
    let remaining_count = get_active_kb_count(&conn_pool, session_id).unwrap_or(0);

    if rows_affected == 0 {
        info!(
            "KB '{}' was not active in session {} or not found",
            kb_name, session_id
        );
    } else {
        info!(
            "✅ Cleared KB '{}' from session {}, {} KB(s) remaining active",
            kb_name, session_id, remaining_count
        );
    }

    Ok(())
}

/// Clear all KBs from session
fn clear_all_kbs(
    conn_pool: crate::shared::utils::DbPool,
    session_id: Uuid,
) -> Result<usize, String> {
    let mut conn = conn_pool
        .get()
        .map_err(|e| format!("Failed to get DB connection: {}", e))?;

    // Mark all KBs as inactive
    let rows_affected = diesel::sql_query(
        "UPDATE session_kb_associations
         SET is_active = false
         WHERE session_id = $1 AND is_active = true",
    )
    .bind::<diesel::sql_types::Uuid, _>(session_id)
    .execute(&mut conn)
    .map_err(|e| format!("Failed to clear all KBs: {}", e))?;

    if rows_affected > 0 {
        info!(
            "✅ Cleared {} active KBs from session {}",
            rows_affected, session_id
        );
    } else {
        info!("No active KBs to clear in session {}", session_id);
    }

    Ok(rows_affected)
}

/// Get count of active KBs for a session
pub fn get_active_kb_count(
    conn_pool: &crate::shared::utils::DbPool,
    session_id: Uuid,
) -> Result<i64, String> {
    let mut conn = conn_pool
        .get()
        .map_err(|e| format!("Failed to get DB connection: {}", e))?;

    let result: CountResult = diesel::sql_query(
        "SELECT COUNT(*) as count
         FROM session_kb_associations
         WHERE session_id = $1 AND is_active = true",
    )
    .bind::<diesel::sql_types::Uuid, _>(session_id)
    .get_result(&mut conn)
    .map_err(|e| format!("Failed to get KB count: {}", e))?;

    Ok(result.count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear_kb_syntax() {
        let mut engine = Engine::new();

        // Test CLEAR_KB with argument
        assert!(engine
            .register_custom_syntax(&["CLEAR_KB", "$expr$"], true, |_, _| Ok(Dynamic::UNIT))
            .is_ok());

        // Test CLEAR_KB without argument
        assert!(engine
            .register_custom_syntax(&["CLEAR_KB"], true, |_, _| Ok(Dynamic::UNIT))
            .is_ok());
    }
}
