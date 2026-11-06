use std::sync::Arc;

use botcore::shared::state::AppState;
use diesel::prelude::*;
use uuid::Uuid;

/// #1700 — first-token latency is recorded once per turn at the first content
/// chunk, so the server-held-thread claim can be measured instead of asserted.
pub(super) fn record(
    state: &Arc<AppState>,
    bot_uuid: Uuid,
    session_id: Uuid,
    llm_model: &str,
    ttft_ms: i64,
) {
    log::info!("llm_ttft: {} ms (model {})", ttft_ms, llm_model);
    let pool = state.conn.clone();
    let model = llm_model.to_string();
    tokio::task::spawn_blocking(move || {
        let Ok(mut conn) = pool.get() else { return };
        let _ = diesel::sql_query(
            "INSERT INTO llm_metrics (id, request_id, session_id, bot_id, model, request_type, input_tokens, output_tokens, total_tokens, latency_ms, ttft_ms, cached, success, estimated_cost, timestamp, metadata) \
             VALUES (gen_random_uuid(), gen_random_uuid(), $1, $2, $3, 'chat', 0, 0, 0, $4, $4, false, true, 0, NOW(), '{}'::jsonb)",
        )
        .bind::<diesel::sql_types::Uuid, _>(session_id)
        .bind::<diesel::sql_types::Uuid, _>(bot_uuid)
        .bind::<diesel::sql_types::Text, _>(&model)
        .bind::<diesel::sql_types::BigInt, _>(ttft_ms)
        .execute(&mut conn);
    });
}
