use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use botlib::kb_scope::KbTenantScope;
use diesel::prelude::*;
use log::{info, warn};
use rhai::{Dynamic, Engine};
use std::sync::Arc;
use uuid::Uuid;

#[derive(QueryableByName)]
struct BotNameResult {
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: Uuid,
}

#[derive(QueryableByName)]
struct KbCollectionResult {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    folder_path: String,
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

/// Current KB ACL facts from the trusted group tables. `$1` = kb_id, `$2` =
/// user_id. Splitting the verdict into facts lets the pure predicate below be
/// reused at attach time and at query time (#2820).
const KB_ACCESS_FACTS_SQL: &str = "SELECT
    NOT EXISTS (SELECT 1 FROM kb_group_associations kga WHERE kga.kb_id = kc.id) AS is_public,
    EXISTS (
        SELECT 1 FROM kb_group_associations kga
        JOIN rbac_user_groups rug ON rug.group_id = kga.group_id
        WHERE kga.kb_id = kc.id AND rug.user_id = $2
    ) AS user_in_group
    FROM kb_collections kc WHERE kc.id = $1";

#[derive(QueryableByName)]
struct KbAccessFacts {
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_public: bool,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    user_in_group: bool,
}

/// Pure permission recheck predicate: a KB is accessible when it has no group
/// restrictions or the caller currently belongs to an authorized group.
pub fn kb_access_allowed(is_public: bool, user_in_group: bool) -> bool {
    is_public || user_in_group
}

/// Query-time ACL recheck against the trusted group tables. Call this before
/// returning KB content so a revoked membership hides already-attached KBs
/// (#2820); attach-time checks alone leave stale authorization.
pub fn user_can_access_kb(
    conn_pool: &botbasic_types::types::DbPool,
    kb_id: Uuid,
    user_id: Uuid,
) -> Result<bool, String> {
    let mut conn = conn_pool
        .get()
        .map_err(|e| format!("Failed to get DB connection: {}", e))?;
    let facts = diesel::sql_query(KB_ACCESS_FACTS_SQL)
        .bind::<diesel::sql_types::Uuid, _>(kb_id)
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .get_result::<KbAccessFacts>(&mut conn)
        .map_err(|e| format!("Failed to recheck KB access: {}", e))?;
    Ok(kb_access_allowed(facts.is_public, facts.user_in_group))
}

pub fn register_use_kb_keyword(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    let state_clone = state;
    let session_clone = user;

    let session_clone_for_syntax = session_clone.clone();
    let state_clone_for_syntax = state_clone.clone();

    if let Err(e) =
        engine.register_custom_syntax(["USE", "KB", "$expr$"], true, move |context, inputs| {
            let kb_name = context.eval_expression_tree(&inputs[0])?.to_string();

            info!(
                "USE KB keyword executed - KB: {}, Session: {}",
                kb_name, session_clone_for_syntax.id
            );

            let session_id = session_clone_for_syntax.id;
            let bot_id = session_clone_for_syntax.bot_id;
            let user_id = Uuid::parse_str(&session_clone_for_syntax.user_id).unwrap_or(Uuid::nil());
            let conn = state_clone_for_syntax.db_pool().clone();
            let kb_name_clone = kb_name.clone();

            let result = std::thread::spawn(move || {
                add_kb_to_session(conn, session_id, bot_id, user_id, &kb_name_clone)
            })
            .join();

            match result {
                Ok(Ok(_)) => {
                    info!(
                        " KB '{}' added to session {}",
                        kb_name, session_clone_for_syntax.id
                    );
                    Ok(Dynamic::UNIT)
                }
                Ok(Err(e)) => {
                    log::error!("Failed to add KB '{}': {}", kb_name, e);
                    Err(format!("USE_KB failed: {}", e).into())
                }
                Err(e) => {
                    log::error!("Thread panic in USE_KB: {:?}", e);
                    Err("USE_KB failed: thread panic".into())
                }
            }
        })
    {
        log::error!("Failed to register USE KB syntax: {e}");
    }

    let session_clone2 = session_clone.clone();
    let state_clone2 = state_clone.clone();

    engine.register_fn("use_kb", move |kb_name: &str| -> Dynamic {
        info!(
            "use_kb function called - KB: {}, Session: {}",
            kb_name, session_clone2.id
        );

        let session_id = session_clone2.id;
        let bot_id = session_clone2.bot_id;
        let user_id = Uuid::parse_str(&session_clone2.user_id).unwrap_or(Uuid::nil());
        let conn = state_clone2.db_pool().clone();
        let kb_name_clone = kb_name.to_string();

        let join_result = std::thread::spawn(move || {
            add_kb_to_session(conn, session_id, bot_id, user_id, &kb_name_clone)
        })
        .join();

        log::info!(
            "use_kb AFTER JOIN kb {} result={:?} join_ok={}",
            kb_name,
            join_result
                .as_ref()
                .map(|r| r.as_ref().map(|_| "ok").unwrap_or("err")),
            join_result.is_ok()
        );

        match join_result {
            Ok(Ok(_)) => {
                info!(" use_kb '{}' added to session {}", kb_name, session_id);
                Dynamic::UNIT
            }
            Ok(Err(e)) => {
                log::error!("Failed to add KB '{}': {}", kb_name, e);
                Dynamic::from(format!("USE_KB failed: {}", e))
            }
            Err(e) => {
                log::error!("Thread panic in USE_KB: {:?}", e);
                Dynamic::from("USE_KB failed: thread panic")
            }
        }
    });

    engine.register_fn("USE_KB", move |kb_name: &str| -> Dynamic {
        info!(
            "USE_KB function called - KB: {}, Session: {}",
            kb_name, session_clone.id
        );

        let session_id = session_clone.id;
        let bot_id = session_clone.bot_id;
        let user_id = Uuid::parse_str(&session_clone.user_id).unwrap_or(Uuid::nil());
        let conn = state_clone.db_pool().clone();
        let kb_name_clone = kb_name.to_string();

        let join_result = std::thread::spawn(move || {
            add_kb_to_session(conn, session_id, bot_id, user_id, &kb_name_clone)
        })
        .join();

        log::info!(
            "USE_KB AFTER JOIN kb {} result={:?} join_ok={}",
            kb_name,
            join_result
                .as_ref()
                .map(|r| r.as_ref().map(|_| "ok").unwrap_or("err")),
            join_result.is_ok()
        );

        match join_result {
            Ok(Ok(_)) => {
                info!(" USE_KB '{}' added to session {}", kb_name, session_id);
                Dynamic::UNIT
            }
            Ok(Err(e)) => {
                log::error!("Failed to add KB '{}': {}", kb_name, e);
                Dynamic::from(format!("USE_KB failed: {}", e))
            }
            Err(e) => {
                log::error!("Thread panic in USE_KB: {:?}", e);
                Dynamic::from("USE_KB failed: thread panic")
            }
        }
    });
}

pub fn add_kb_to_session(
    conn_pool: botbasic_types::types::DbPool,
    session_id: Uuid,
    bot_id: Uuid,
    user_id: Uuid,
    kb_name: &str,
) -> Result<(), String> {
    let mut conn = conn_pool
        .get()
        .map_err(|e| format!("Failed to get DB connection: {}", e))?;

    let bot_result: BotNameResult =
        diesel::sql_query("SELECT name, branch_id FROM bots WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .get_result(&mut conn)
            .map_err(|e| format!("Failed to get bot name: {}", e))?;
    let bot_name = bot_result.name;
    // The KB collection name is always derived from the server-resolved
    // bot/branch pair, never from a caller-supplied name (issue #2818).
    let Some(scope) = KbTenantScope::new(bot_id, bot_result.branch_id) else {
        return Err(format!("Cannot resolve KB tenant scope for bot {bot_id}"));
    };
    let qdrant_collection = scope.collection_name(kb_name);

    let kb_exists: Option<KbCollectionResult> = diesel::sql_query(
        "SELECT id, folder_path FROM kb_collections WHERE bot_id = $1 AND name = $2",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Text, _>(kb_name)
    .get_result(&mut conn)
    .optional()
    .map_err(|e| format!("Failed to check KB existence: {}", e))?;

    let kb_folder_path = if let Some(kb_result) = kb_exists {
        let facts = diesel::sql_query(KB_ACCESS_FACTS_SQL)
            .bind::<diesel::sql_types::Uuid, _>(kb_result.id)
            .bind::<diesel::sql_types::Uuid, _>(user_id)
            .get_result::<KbAccessFacts>(&mut conn)
            .map_err(|e| format!("Failed to check KB access: {}", e))?;

        if !kb_access_allowed(facts.is_public, facts.user_in_group) {
            return Err(format!("Access denied for KB '{}'", kb_name));
        }

        kb_result.folder_path
    } else {
        let default_path = format!("work/{}/{}.gbkb/{}", bot_name, bot_name, kb_name);
        let kb_id = Uuid::new_v4();

        log::warn!(
            "KB '{}' not found in kb_collections for bot {}. Using default path: {}, collection: {}",
            kb_name, bot_name, default_path, qdrant_collection
        );

        diesel::sql_query(
            "INSERT INTO kb_collections (id, bot_id, name, folder_path, qdrant_collection, document_count)
            VALUES ($1, $2, $3, $4, $5, 0)
            ON CONFLICT (bot_id, name) DO UPDATE SET
            folder_path = EXCLUDED.folder_path,
            qdrant_collection = EXCLUDED.qdrant_collection"
        )
        .bind::<diesel::sql_types::Uuid, _>(kb_id)
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .bind::<diesel::sql_types::Text, _>(kb_name)
        .bind::<diesel::sql_types::Text, _>(&default_path)
        .bind::<diesel::sql_types::Text, _>(&qdrant_collection)
        .execute(&mut conn)
        .ok();

        default_path
    };

    let tool_name: Option<String> = None;

    // Ensure session exists in user_sessions for FK constraint on session_kb_associations
    if let Err(e) = diesel::sql_query(
        "INSERT INTO user_sessions (id, user_id, bot_id, branch_id, title, answer_mode, context_data, created_at, updated_at)
         VALUES ($1, $2, $3, $4, 'KB Session', 1, '{}'::jsonb, NOW(), NOW())
         ON CONFLICT (id) DO NOTHING"
    )
    .bind::<diesel::sql_types::Uuid, _>(session_id)
    .bind::<diesel::sql_types::Text, _>(user_id.to_string())
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Uuid, _>(scope.branch_id)
    .execute(&mut conn)
    {
        warn!("Failed to ensure session {} in user_sessions: {}", session_id, e);
    }

    let assoc_id = Uuid::new_v4();
    if let Err(e) = diesel::sql_query(
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
    {
        if e.to_string().contains("foreign key constraint") {
            warn!("KB '{}' deferred for session {} (session not yet in DB)", kb_name, session_id);
        } else {
            return Err(format!("Failed to add KB association: {}", e));
        }
    } else {
        info!(
            " Added KB '{}' to session {} (collection: {}, path: {})",
            kb_name, session_id, qdrant_collection, kb_folder_path
        );
    }

    Ok(())
}

pub fn get_active_kbs_for_session(
    conn_pool: &botbasic_types::types::DbPool,
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

#[cfg(test)]
mod kb_access_tests {
    use super::kb_access_allowed;

    #[test]
    fn public_kb_needs_no_group_membership() {
        assert!(kb_access_allowed(true, false));
        assert!(kb_access_allowed(true, true));
    }

    #[test]
    fn restricted_kb_requires_a_current_group_membership() {
        assert!(kb_access_allowed(false, true));
        assert!(!kb_access_allowed(false, false));
    }

    #[test]
    fn revoking_membership_denies_an_already_attached_kb() {
        // At attach time the user was in an authorized group.
        assert!(kb_access_allowed(false, true));
        // Query time after the group row is removed: same KB, no live grant.
        assert!(!kb_access_allowed(false, false));
    }
}
