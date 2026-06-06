use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use chrono::Utc;
use diesel::sql_types::{BigInt, Nullable, Text, Timestamptz};
use diesel::QueryableByName;
use rhai::{Dynamic, Engine, EvalAltResult};
use serde_json::{json, Value};
use std::sync::Arc;
use std::time::Duration;
use uuid::Uuid;

/// Chatbot handoff BASIC keywords for issue #621.
///
/// Provides: TRANSFER TO HUMAN, ESCALATE, REQUEST AGENT,
/// ASSIGN AGENT, COMPLETE HANDOFF.
///
/// Persistence model:
/// - handoff_queue: a single table holding all open handoff tickets with
///   status, priority, assigned agent, topic and full JSON metadata.
/// - handoff_events: append-only audit log for escalations, assignments,
///   completions and CSAT ratings.
/// - Redis is used for fast agent presence tracking
///   (presence:{agent_id} = "online" with TTL).
pub fn register_chatbot_handoff_keywords(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    register_transfer_to_human(state.clone(), user.clone(), engine);
    register_escalate(state.clone(), user.clone(), engine);
    register_request_agent(state.clone(), user.clone(), engine);
    register_assign_agent(state.clone(), user.clone(), engine);
    register_complete_handoff(state, user, engine);
}

fn runtime_error(message: impl Into<String>) -> Box<EvalAltResult> {
    Box::new(EvalAltResult::ErrorRuntime(message.into().into(), rhai::Position::NONE))
}

fn ensure_schema(pool: &botlib::db_pool::DbPool) -> Result<(), String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS handoff_queue (
            id UUID PRIMARY KEY,
            bot_id UUID NOT NULL,
            session_id UUID,
            user_id UUID,
            topic TEXT NOT NULL,
            reason TEXT,
            priority TEXT NOT NULL DEFAULT 'normal',
            status TEXT NOT NULL DEFAULT 'queued',
            agent_id UUID,
            metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(|e| format!("handoff_queue: {e}"))?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS handoff_events (
            id UUID PRIMARY KEY,
            handoff_id UUID NOT NULL,
            kind TEXT NOT NULL,
            payload JSONB NOT NULL DEFAULT '{}'::jsonb,
            actor_id UUID,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )",
    )
    .execute(&mut conn)
    .map_err(|e| format!("handoff_events: {e}"))?;
    Ok(())
}

#[derive(QueryableByName, Debug)]
struct QueuePosition {
    #[diesel(sql_type = BigInt)]
    position: i64,
}

#[derive(QueryableByName, Debug)]
struct AgentId {
    #[diesel(sql_type = Nullable<diesel::sql_types::Uuid>)]
    agent_id: Option<Uuid>,
}

fn queue_position(
    pool: &botlib::db_pool::DbPool,
    bot_id: Uuid,
    handoff_id: Uuid,
) -> Result<i64, String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let rows: Vec<QueuePosition> = diesel::sql_query(
        "SELECT COUNT(*) AS position
           FROM handoff_queue
          WHERE bot_id = $1
            AND status = 'queued'
            AND created_at <= (SELECT created_at FROM handoff_queue WHERE id = $2)",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Uuid, _>(handoff_id)
    .get_results(&mut conn)
    .map_err(|e| format!("queue position: {e}"))?;
    Ok(rows.first().map(|r| r.position).unwrap_or(1))
}

fn insert_queue_entry(
    pool: &botlib::db_pool::DbPool,
    bot_id: Uuid,
    session_id: Option<Uuid>,
    user_id: Option<Uuid>,
    topic: &str,
    reason: Option<&str>,
    priority: &str,
    metadata: &Value,
) -> Result<Uuid, String> {
    ensure_schema(pool)?;
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let id = Uuid::new_v4();
    let body = serde_json::to_string(metadata).map_err(|e| format!("Serialize: {e}"))?;
    diesel::sql_query(
        "INSERT INTO handoff_queue
            (id, bot_id, session_id, user_id, topic, reason, priority, status, metadata)
         VALUES ($1, $2, $3, $4, $5, $6, $7, 'queued', $8::jsonb)",
    )
    .bind::<diesel::sql_types::Uuid, _>(id)
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(session_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(user_id)
    .bind::<diesel::sql_types::Text, _>(topic)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(reason)
    .bind::<diesel::sql_types::Text, _>(priority)
    .bind::<diesel::sql_types::Text, _>(body)
    .execute(&mut conn)
    .map_err(|e| format!("insert queue: {e}"))?;
    Ok(id)
}

fn update_handoff_status(
    pool: &botlib::db_pool::DbPool,
    handoff_id: Uuid,
    status: &str,
    agent_id: Option<Uuid>,
) -> Result<(), String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    diesel::sql_query(
        "UPDATE handoff_queue
            SET status = $1,
                agent_id = COALESCE($2, agent_id),
                updated_at = NOW()
          WHERE id = $3",
    )
    .bind::<diesel::sql_types::Text, _>(status)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(agent_id)
    .bind::<diesel::sql_types::Uuid, _>(handoff_id)
    .execute(&mut conn)
    .map_err(|e| format!("update handoff: {e}"))?;
    Ok(())
}

fn fetch_handoff(
    pool: &botlib::db_pool::DbPool,
    handoff_id: Uuid,
) -> Result<Option<Uuid>, String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let rows: Vec<AgentId> = diesel::sql_query(
        "SELECT agent_id FROM handoff_queue WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(handoff_id)
    .get_results(&mut conn)
    .map_err(|e| format!("fetch handoff: {e}"))?;
    Ok(rows.first().and_then(|r| r.agent_id))
}

fn append_event(
    pool: &botlib::db_pool::DbPool,
    handoff_id: Uuid,
    kind: &str,
    payload: &Value,
    actor_id: Option<Uuid>,
) -> Result<(), String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let body = serde_json::to_string(payload).map_err(|e| format!("Serialize: {e}"))?;
    diesel::sql_query(
        "INSERT INTO handoff_events (id, handoff_id, kind, payload, actor_id)
         VALUES ($1, $2, $3, $4::jsonb, $5)",
    )
    .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
    .bind::<diesel::sql_types::Uuid, _>(handoff_id)
    .bind::<diesel::sql_types::Text, _>(kind)
    .bind::<diesel::sql_types::Text, _>(body)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(actor_id)
    .execute(&mut conn)
    .map_err(|e| format!("append event: {e}"))?;
    Ok(())
}

fn set_agent_presence(state: &Arc<dyn BasicRuntime>, agent_id: Uuid, online: bool) {
    if let Some(client) = state.cache_client() {
        let key = format!("presence:{agent_id}");
        let value = if online { "online" } else { "offline" };
        let client_clone = Arc::clone(&client);
        let _ = std::thread::Builder::new()
            .name("presence".into())
            .spawn(move || {
                if let Ok(mut conn) = client_clone.get_connection() {
                    use redis::Commands;
                    if online {
                        let _: Result<(), _> = conn.set_ex(&key, value, 300);
                    } else {
                        let _: Result<(), _> = conn.del(&key);
                    }
                }
            })
            .map(|h| h.join().ok())
            .ok();
    }
}

fn execute_with_timeout<F, T>(work: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::Builder::new()
        .name("handoff-worker".into())
        .spawn(move || {
            let outcome = work();
            let _ = tx.send(outcome);
        })
        .map_err(|e| format!("Spawn: {e}"))?;
    rx.recv_timeout(Duration::from_secs(15))
        .map_err(|_| "Operation timed out after 15s".to_string())
}

fn bot_id_from_session(user: &UserSession) -> Uuid {
    user.bot_id.unwrap_or_else(Uuid::nil)
}

fn user_id_from_session(user: &UserSession) -> Option<Uuid> {
    user.user_id
}

fn priority_int(priority: &str) -> i32 {
    match priority.to_ascii_lowercase().as_str() {
        "low" | "baixa" => 1,
        "normal" | "media" | "média" => 5,
        "high" | "alta" => 8,
        "urgent" | "critica" | "crítica" => 10,
        _ => 5,
    }
}

fn register_transfer_to_human(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    let state_clone = Arc::clone(&state);
    engine
        .register_custom_syntax(
            ["TRANSFER", "TO", "HUMAN", "REASON", "$expr$"],
            false,
            move |context, inputs| {
                let reason = context.eval_expression_tree(&inputs[0])?.to_string();
                let state = Arc::clone(&state_clone);
                let user_clone = user.clone();
                let reason_clone = reason.clone();
                let outcome = execute_with_timeout(move || -> Result<Value, String> {
                    let pool = state.db_pool();
                    let bot_id = bot_id_from_session(&user_clone);
                    let user_id = user_id_from_session(&user_clone);
                    let session_id = user_clone.session_id;
                    let topic = format!("User {user_id:?} request");
                    let metadata = json!({
                        "channel": "bot",
                        "session_id": session_id,
                    });
                    let handoff_id = insert_queue_entry(
                        pool,
                        bot_id,
                        Some(session_id),
                        user_id,
                        &topic,
                        Some(&reason_clone),
                        "normal",
                        &metadata,
                    )?;
                    append_event(
                        pool,
                        handoff_id,
                        "transfer",
                        &json!({ "reason": reason_clone.clone() }),
                        user_id,
                    )?;
                    let pos = queue_position(pool, bot_id, handoff_id)?;
                    Ok(json!({
                        "kind": "transfer_to_human",
                        "handoff_id": handoff_id.to_string(),
                        "reason": reason_clone,
                        "queue_position": pos,
                        "estimated_wait_seconds": pos * 60,
                        "queued_at": Utc::now().to_rfc3339(),
                    }))
                });
                match outcome {
                    Ok(v) => Ok(Dynamic::from(v.to_string())),
                    Err(e) => Err(runtime_error(format!("TRANSFER TO HUMAN: {e}"))),
                }
            },
        )
        .map_err(|e| runtime_error(format!("TRANSFER TO HUMAN registration: {e}")))
        .ok();
}

fn register_escalate(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    engine
        .register_custom_syntax(
            ["ESCALATE", "$expr$", "PRIORITY", "$expr$"],
            false,
            move |context, inputs| {
                let issue = context.eval_expression_tree(&inputs[0])?.to_string();
                let priority = context.eval_expression_tree(&inputs[1])?.to_string();
                let state = Arc::clone(&state_clone);
                let user_clone = user.clone();
                let issue_clone = issue.clone();
                let priority_clone = priority.clone();
                let outcome = execute_with_timeout(move || -> Result<Value, String> {
                    let pool = state.db_pool();
                    let bot_id = bot_id_from_session(&user_clone);
                    let user_id = user_id_from_session(&user_clone);
                    let session_id = user_clone.session_id;
                    let metadata = json!({
                        "channel": "bot",
                        "session_id": session_id,
                        "issue": issue_clone,
                    });
                    let handoff_id = insert_queue_entry(
                        pool,
                        bot_id,
                        Some(session_id),
                        user_id,
                        &issue_clone,
                        Some(&issue_clone),
                        &priority_clone,
                        &metadata,
                    )?;
                    append_event(
                        pool,
                        handoff_id,
                        "escalation",
                        &json!({
                            "priority": priority_clone,
                            "priority_rank": priority_int(&priority_clone),
                        }),
                        user_id,
                    )?;
                    Ok(json!({
                        "kind": "escalation",
                        "handoff_id": handoff_id.to_string(),
                        "issue": issue_clone,
                        "priority": priority_clone,
                        "priority_rank": priority_int(&priority_clone),
                        "escalated": true,
                    }))
                });
                match outcome {
                    Ok(v) => Ok(Dynamic::from(v.to_string())),
                    Err(e) => Err(runtime_error(format!("ESCALATE: {e}"))),
                }
            },
        )
        .map_err(|e| runtime_error(format!("ESCALATE registration: {e}")))
        .ok();
}

fn register_request_agent(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    engine
        .register_custom_syntax(
            ["REQUEST", "AGENT", "TOPIC", "$expr$"],
            false,
            move |context, inputs| {
                let topic = context.eval_expression_tree(&inputs[0])?.to_string();
                let state = Arc::clone(&state_clone);
                let user_clone = user.clone();
                let topic_clone = topic.clone();
                let outcome = execute_with_timeout(move || -> Result<Value, String> {
                    let pool = state.db_pool();
                    let bot_id = bot_id_from_session(&user_clone);
                    let user_id = user_id_from_session(&user_clone);
                    let session_id = user_clone.session_id;
                    let metadata = json!({ "channel": "bot", "session_id": session_id });
                    let handoff_id = insert_queue_entry(
                        pool,
                        bot_id,
                        Some(session_id),
                        user_id,
                        &topic_clone,
                        Some(&topic_clone),
                        "normal",
                        &metadata,
                    )?;
                    append_event(
                        pool,
                        handoff_id,
                        "request",
                        &json!({ "topic": topic_clone }),
                        user_id,
                    )?;
                    let pos = queue_position(pool, bot_id, handoff_id)?;
                    Ok(json!({
                        "kind": "request_agent",
                        "handoff_id": handoff_id.to_string(),
                        "topic": topic_clone,
                        "agent_id": null,
                        "queue_position": pos,
                        "waiting": true,
                    }))
                });
                match outcome {
                    Ok(v) => Ok(Dynamic::from(v.to_string())),
                    Err(e) => Err(runtime_error(format!("REQUEST AGENT: {e}"))),
                }
            },
        )
        .map_err(|e| runtime_error(format!("REQUEST AGENT registration: {e}")))
        .ok();
}

fn register_assign_agent(state: Arc<dyn BasicRuntime>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    engine
        .register_custom_syntax(
            ["ASSIGN", "AGENT", "$expr$", "TO", "$expr$"],
            false,
            move |context, inputs| {
                let agent = context.eval_expression_tree(&inputs[0])?.to_string();
                let session_str = context.eval_expression_tree(&inputs[1])?.to_string();
                let state = Arc::clone(&state_clone);
                let user_clone = user.clone();
                let agent_clone = agent.clone();
                let outcome = execute_with_timeout(move || -> Result<Value, String> {
                    let agent_id = Uuid::parse_str(&agent_clone)
                        .map_err(|e| format!("agent must be UUID: {e}"))?;
                    let handoff_id = Uuid::parse_str(&session_str)
                        .map_err(|e| format!("session must be UUID: {e}"))?;
                    let pool = state.db_pool();
                    let user_id = user_id_from_session(&user_clone);
                    update_handoff_status(pool, handoff_id, "assigned", Some(agent_id))?;
                    append_event(
                        pool,
                        handoff_id,
                        "assigned",
                        &json!({ "agent_id": agent_id }),
                        user_id,
                    )?;
                    set_agent_presence(&state, agent_id, true);
                    Ok(json!({
                        "kind": "assign_agent",
                        "agent": agent_id.to_string(),
                        "session": handoff_id.to_string(),
                        "assigned": true,
                    }))
                });
                match outcome {
                    Ok(v) => Ok(Dynamic::from(v.to_string())),
                    Err(e) => Err(runtime_error(format!("ASSIGN AGENT: {e}"))),
                }
            },
        )
        .map_err(|e| runtime_error(format!("ASSIGN AGENT registration: {e}")))
        .ok();
}

fn register_complete_handoff(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    let state_clone = Arc::clone(&state);
    engine
        .register_custom_syntax(
            ["COMPLETE", "HANDOFF", "$expr$", "RATING", "$expr$"],
            false,
            move |context, inputs| {
                let session_str = context.eval_expression_tree(&inputs[0])?.to_string();
                let rating_str = context.eval_expression_tree(&inputs[1])?.to_string();
                let state = Arc::clone(&state_clone);
                let user_clone = user.clone();
                let outcome = execute_with_timeout(move || -> Result<Value, String> {
                    let handoff_id = Uuid::parse_str(&session_str)
                        .map_err(|e| format!("session must be UUID: {e}"))?;
                    let rating: f64 = rating_str
                        .parse()
                        .map_err(|e| format!("rating must be numeric: {e}"))?;
                    let pool = state.db_pool();
                    let user_id = user_id_from_session(&user_clone);
                    let prior_agent = fetch_handoff(pool, handoff_id)?;
                    update_handoff_status(pool, handoff_id, "completed", prior_agent)?;
                    append_event(
                        pool,
                        handoff_id,
                        "completed",
                        &json!({ "csat": rating }),
                        user_id,
                    )?;
                    if let Some(agent_id) = prior_agent {
                        set_agent_presence(&state, agent_id, false);
                    }
                    Ok(json!({
                        "kind": "complete_handoff",
                        "session": handoff_id.to_string(),
                        "rating": rating,
                        "agent": prior_agent.map(|a| a.to_string()),
                        "completed": true,
                    }))
                });
                match outcome {
                    Ok(v) => Ok(Dynamic::from(v.to_string())),
                    Err(e) => Err(runtime_error(format!("COMPLETE HANDOFF: {e}"))),
                }
            },
        )
        .map_err(|e| runtime_error(format!("COMPLETE HANDOFF registration: {e}")))
        .ok();
}
