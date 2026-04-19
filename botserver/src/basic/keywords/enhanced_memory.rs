use crate::core::shared::models::{bot_memories, bot_shared_memory, BotSharedMemory};
use crate::core::shared::state::AppState;
use crate::basic::UserSession;
use diesel::prelude::*;
use rhai::{Dynamic, Engine};
use std::sync::Arc;
use uuid::Uuid;

pub fn register_bot_share_memory(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    if let Err(e) = engine.register_custom_syntax(
        ["BOT", "SHARE", "MEMORY", "$string$", "WITH", "$string$"],
        false,
        move |context, inputs| {
            let memory_key = context.eval_expression_tree(&inputs[0])?.to_string();
            let target_bot_name = context.eval_expression_tree(&inputs[1])?.to_string();
            
            let state_for_spawn = Arc::clone(&state_clone);
            let user_clone_spawn = user_clone.clone();
            
            tokio::spawn(async move {
                if let Err(e) = share_bot_memory(&state_for_spawn, &user_clone_spawn, &memory_key, &target_bot_name).await {
                    log::error!("Failed to share memory {memory_key} with {target_bot_name}: {e}");
                }
            });

            Ok(Dynamic::UNIT)
        },
    ) {
        log::warn!("Failed to register BOT SHARE MEMORY syntax: {e}");
    }
}

pub fn register_bot_sync_memory(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user;

    if let Err(e) = engine.register_custom_syntax(
        ["BOT", "SYNC", "MEMORY", "FROM", "$string$"],
        false,
        move |context, inputs| {
            let source_bot_name = context.eval_expression_tree(&inputs[0])?.to_string();
            
            let state_for_spawn = Arc::clone(&state_clone);
            let user_clone_spawn = user_clone.clone();
            
            tokio::spawn(async move {
                if let Err(e) = sync_bot_memory(&state_for_spawn, &user_clone_spawn, &source_bot_name).await {
                    log::error!("Failed to sync memory from {source_bot_name}: {e}");
                }
            });

            Ok(Dynamic::UNIT)
        },
    ) {
        log::warn!("Failed to register BOT SYNC MEMORY syntax: {e}");
    }
}

async fn share_bot_memory(
    state: &Arc<AppState>,
    user: &UserSession,
    memory_key: &str,
    target_bot_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = state.conn.get()?;
    
    let source_bot_uuid = Uuid::parse_str(&user.bot_id.to_string())?;
    
    let target_bot_uuid = find_bot_by_name(&mut conn, target_bot_name)?;
    
    let memory_value = bot_memories::table
        .filter(bot_memories::bot_id.eq(source_bot_uuid))
        .filter(bot_memories::key.eq(memory_key))
        .select(bot_memories::value)
        .first(&mut conn)
        .unwrap_or_default();
    
    let shared_memory = BotSharedMemory {
        id: Uuid::new_v4(),
        source_bot_id: source_bot_uuid,
        target_bot_id: target_bot_uuid,
        memory_key: memory_key.to_string(),
        memory_value,
        shared_at: chrono::Utc::now(),
    };
    
    diesel::insert_into(bot_shared_memory::table)
        .values(&shared_memory)
        .on_conflict((bot_shared_memory::target_bot_id, bot_shared_memory::memory_key))
        .do_update()
        .set((
            bot_shared_memory::memory_value.eq(&shared_memory.memory_value),
            bot_shared_memory::shared_at.eq(chrono::Utc::now()),
        ))
        .execute(&mut conn)?;
    
    Ok(())
}

async fn sync_bot_memory(
    state: &Arc<AppState>,
    user: &UserSession,
    source_bot_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut conn = state.conn.get()?;
    
    let target_bot_uuid = Uuid::parse_str(&user.bot_id.to_string())?;
    let source_bot_uuid = find_bot_by_name(&mut conn, source_bot_name)?;
    
    let shared_memories: Vec<BotSharedMemory> = bot_shared_memory::table
        .filter(bot_shared_memory::source_bot_id.eq(source_bot_uuid))
        .filter(bot_shared_memory::target_bot_id.eq(target_bot_uuid))
        .load(&mut conn)?;
    
    for shared_memory in shared_memories {
        diesel::insert_into(bot_memories::table)
            .values((
                bot_memories::id.eq(Uuid::new_v4()),
                bot_memories::bot_id.eq(target_bot_uuid),
                bot_memories::key.eq(&shared_memory.memory_key),
                bot_memories::value.eq(&shared_memory.memory_value),
                bot_memories::created_at.eq(chrono::Utc::now()),
                bot_memories::updated_at.eq(chrono::Utc::now()),
            ))
            .on_conflict((bot_memories::bot_id, bot_memories::key))
            .do_update()
            .set((
                bot_memories::value.eq(&shared_memory.memory_value),
                bot_memories::updated_at.eq(chrono::Utc::now()),
            ))
            .execute(&mut conn)?;
    }
    
    Ok(())
}

fn find_bot_by_name(
    conn: &mut PgConnection,
    bot_name: &str,
) -> Result<Uuid, Box<dyn std::error::Error + Send + Sync>> {
    use crate::core::shared::models::bots;
    
    let bot_id: Uuid = bots::table
        .filter(bots::name.eq(bot_name))
        .select(bots::id)
        .first(conn)
        .map_err(|_| format!("Bot not found: {bot_name}"))?;
    
    Ok(bot_id)
}
