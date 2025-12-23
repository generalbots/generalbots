use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use diesel::prelude::*;
use log::{error, trace};
use rhai::{Dynamic, Engine};
use std::sync::Arc;
use uuid::Uuid;

/// Registers user memory keywords for cross-session memory persistence.
/// Unlike bot memory, user memory persists across all sessions and bots for a specific user.
pub fn register_user_memory_keywords(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    set_user_memory_keyword(state.clone(), user.clone(), engine);
    get_user_memory_keyword(state.clone(), user.clone(), engine);
    remember_user_fact_keyword(state.clone(), user.clone(), engine);
    get_user_facts_keyword(state.clone(), user.clone(), engine);
    clear_user_memory_keyword(state.clone(), user.clone(), engine);
}

/// SET USER MEMORY key, value
/// Stores a key-value pair that persists across all sessions for this user
pub fn set_user_memory_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["SET", "USER", "MEMORY", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let key = context.eval_expression_tree(&inputs[0])?.to_string();
                let value = context.eval_expression_tree(&inputs[1])?.to_string();
                let state_for_spawn = Arc::clone(&state_clone);
                let user_clone_spawn = user_clone.clone();
                let key_clone = key.clone();
                let value_clone = value.clone();

                tokio::spawn(async move {
                    if let Err(e) = set_user_memory_async(
                        &state_for_spawn,
                        user_clone_spawn.user_id,
                        &key_clone,
                        &value_clone,
                        "preference",
                    )
                    .await
                    {
                        error!("Failed to set user memory: {}", e);
                    } else {
                        trace!(
                            "Set user memory for key: {} with value length: {}",
                            key_clone,
                            value_clone.len()
                        );
                    }
                });

                Ok(Dynamic::UNIT)
            },
        )
        .expect("Failed to register SET USER MEMORY syntax");
}

/// GET USER MEMORY("key")
/// Retrieves a value from user's cross-session memory
pub fn get_user_memory_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine.register_fn("GET USER MEMORY", move |key_param: String| -> String {
        let state = Arc::clone(&state_clone);
        let conn_result = state.conn.get();

        if let Ok(mut conn) = conn_result {
            get_user_memory_sync(&mut conn, user_clone.user_id, &key_param).unwrap_or_default()
        } else {
            String::new()
        }
    });
}

/// REMEMBER USER FACT "fact about user"
/// Stores a learned fact about the user for future reference
pub fn remember_user_fact_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["REMEMBER", "USER", "FACT", "$expr$"],
            false,
            move |context, inputs| {
                let fact = context.eval_expression_tree(&inputs[0])?.to_string();
                let state_for_spawn = Arc::clone(&state_clone);
                let user_clone_spawn = user_clone.clone();
                let fact_clone = fact.clone();

                tokio::spawn(async move {
                    if let Err(e) = add_user_fact_async(
                        &state_for_spawn,
                        user_clone_spawn.user_id,
                        &fact_clone,
                    )
                    .await
                    {
                        error!("Failed to remember user fact: {}", e);
                    } else {
                        trace!("Remembered user fact: {}", fact_clone);
                    }
                });

                Ok(Dynamic::UNIT)
            },
        )
        .expect("Failed to register REMEMBER USER FACT syntax");
}

/// GET USER FACTS()
/// Retrieves all learned facts about the user
pub fn get_user_facts_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine.register_fn("GET USER FACTS", move || -> rhai::Array {
        let state = Arc::clone(&state_clone);
        let conn_result = state.conn.get();

        if let Ok(mut conn) = conn_result {
            get_user_facts_sync(&mut conn, user_clone.user_id)
                .unwrap_or_default()
                .into_iter()
                .map(Dynamic::from)
                .collect()
        } else {
            rhai::Array::new()
        }
    });
}

/// CLEAR USER MEMORY
/// Clears all user memory (preferences and facts)
pub fn clear_user_memory_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(&["CLEAR", "USER", "MEMORY"], false, move |_context, _inputs| {
            let state_for_spawn = Arc::clone(&state_clone);
            let user_clone_spawn = user_clone.clone();

            tokio::spawn(async move {
                if let Err(e) = clear_user_memory_async(&state_for_spawn, user_clone_spawn.user_id).await {
                    error!("Failed to clear user memory: {}", e);
                } else {
                    trace!("Cleared all user memory for user: {}", user_clone_spawn.user_id);
                }
            });

            Ok(Dynamic::UNIT)
        })
        .expect("Failed to register CLEAR USER MEMORY syntax");
}

// Database Operations

/// Async function to set user memory
async fn set_user_memory_async(
    state: &AppState,
    user_id: Uuid,
    key: &str,
    value: &str,
    memory_type: &str,
) -> Result<(), String> {
    let mut conn = state
        .conn
        .get()
        .map_err(|e| format!("Failed to acquire database connection: {}", e))?;

    let now = chrono::Utc::now();
    let new_id = Uuid::new_v4();

    // Use raw SQL for upsert since we need to handle the user_memories table
    diesel::sql_query(
        "INSERT INTO user_memories (id, user_id, key, value, memory_type, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, $5, $6, $7) \
         ON CONFLICT (user_id, key) DO UPDATE SET \
         value = EXCLUDED.value, \
         memory_type = EXCLUDED.memory_type, \
         updated_at = EXCLUDED.updated_at",
    )
    .bind::<diesel::sql_types::Uuid, _>(new_id)
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Text, _>(key)
    .bind::<diesel::sql_types::Text, _>(value)
    .bind::<diesel::sql_types::Text, _>(memory_type)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(|e| format!("Failed to set user memory: {}", e))?;

    Ok(())
}

/// Sync function to get user memory (for use in registered functions)
fn get_user_memory_sync(
    conn: &mut diesel::PgConnection,
    user_id: Uuid,
    key: &str,
) -> Result<String, String> {
    #[derive(QueryableByName)]
    struct MemoryValue {
        #[diesel(sql_type = diesel::sql_types::Text)]
        value: String,
    }

    let result: Option<MemoryValue> = diesel::sql_query(
        "SELECT value FROM user_memories WHERE user_id = $1 AND key = $2 LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Text, _>(key)
    .get_result(conn)
    .optional()
    .map_err(|e| format!("Failed to get user memory: {}", e))?;

    Ok(result.map(|r| r.value).unwrap_or_default())
}

/// Async function to add a user fact
async fn add_user_fact_async(
    state: &AppState,
    user_id: Uuid,
    fact: &str,
) -> Result<(), String> {
    let mut conn = state
        .conn
        .get()
        .map_err(|e| format!("Failed to acquire database connection: {}", e))?;

    let now = chrono::Utc::now();
    let new_id = Uuid::new_v4();
    let fact_key = format!("fact_{}", Uuid::new_v4());

    diesel::sql_query(
        "INSERT INTO user_memories (id, user_id, key, value, memory_type, created_at, updated_at) \
         VALUES ($1, $2, $3, $4, 'fact', $5, $6)",
    )
    .bind::<diesel::sql_types::Uuid, _>(new_id)
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Text, _>(&fact_key)
    .bind::<diesel::sql_types::Text, _>(fact)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(|e| format!("Failed to add user fact: {}", e))?;

    Ok(())
}

/// Sync function to get all user facts
fn get_user_facts_sync(
    conn: &mut diesel::PgConnection,
    user_id: Uuid,
) -> Result<Vec<String>, String> {
    #[derive(QueryableByName)]
    struct FactValue {
        #[diesel(sql_type = diesel::sql_types::Text)]
        value: String,
    }

    let results: Vec<FactValue> = diesel::sql_query(
        "SELECT value FROM user_memories WHERE user_id = $1 AND memory_type = 'fact' ORDER BY created_at DESC",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .load(conn)
    .map_err(|e| format!("Failed to get user facts: {}", e))?;

    Ok(results.into_iter().map(|r| r.value).collect())
}

/// Async function to clear all user memory
async fn clear_user_memory_async(state: &AppState, user_id: Uuid) -> Result<(), String> {
    let mut conn = state
        .conn
        .get()
        .map_err(|e| format!("Failed to acquire database connection: {}", e))?;

    diesel::sql_query("DELETE FROM user_memories WHERE user_id = $1")
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .execute(&mut conn)
        .map_err(|e| format!("Failed to clear user memory: {}", e))?;

    Ok(())
}

// Tests
