//! Daily briefing builder (#1191).
//!
//! Enriches the `daily-briefing` proactivity trigger so the emitted card
//! contains real per-app history (recent Vibe runs grouped by project, with
//! tool/LLM usage from telemetry) plus a short relevance summary distilled
//! by the configured LLM. Falls back to the data-only summary when the LLM
//! is unavailable so the card is never empty.

use crate::types::DbPool;
use diesel::prelude::*;

#[derive(diesel::QueryableByName, Debug)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct RunSummaryRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    project_name: String,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    total_runs: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    completed: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    failed: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    tool_calls: i64,
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    total_tokens: i64,
}

/// Builds the daily briefing text.
///
/// Queries the last 24h of runs (per project/app), counts tool calls and
/// LLM tokens from telemetry, then asks the configured LLM what is most
/// relevant. Returns a plain-text briefing safe to render in a card.
pub async fn build_daily_briefing(pool: &DbPool) -> String {
    let history = match per_app_history(pool) {
        Ok(h) => h,
        Err(e) => {
            log::warn!("daily briefing: history query failed: {e}");
            return "Daily briefing: no run history available yet.".to_string();
        }
    };

    let mut lines = Vec::new();
    lines.push("📋 Daily briefing — last 24h".to_string());
    if history.is_empty() {
        lines.push("No Vibe runs in the last 24 hours.".to_string());
        return lines.join("\n");
    }
    lines.push(String::new());
    for app in &history {
        lines.push(format!(
            "• {name}: {total} run(s) — {completed} ok, {failed} failed, {tools} tool call(s), {tokens} LLM tokens",
            name = app.project_name,
            total = app.total_runs,
            completed = app.completed,
            failed = app.failed,
            tools = app.tool_calls,
            tokens = app.total_tokens,
        ));
    }
    lines.push(String::new());

    // Ask the LLM what is most relevant from its point of view.
    if let Some(highlights) = llm_relevance(&history).await {
        lines.push("✨ Most relevant from the LLM:".to_string());
        lines.push(highlights);
    }
    lines.join("\n")
}

/// Groups the last 24h of runs per project with tool/token usage.
fn per_app_history(pool: &DbPool) -> Result<Vec<RunSummaryRow>, String> {
    let mut conn = pool.get().map_err(|e| format!("pool get: {e}"))?;
    let rows = diesel::sql_query(
        "SELECT \
             COALESCE(config->>'project_name', 'default') AS project_name, \
             COUNT(*) AS total_runs, \
             COUNT(*) FILTER (WHERE state = 'completed') AS completed, \
             COUNT(*) FILTER (WHERE state = 'failed') AS failed, \
             (SELECT COUNT(*) FROM vibe_telemetry t \
               WHERE t.run_id IN (SELECT run_id FROM vibe_runs r \
                                  WHERE r.created_at > NOW() - INTERVAL '24 hours')) AS tool_calls, \
             COALESCE((SELECT SUM(COALESCE(t2.tokens_used, 0)) FROM vibe_telemetry t2 \
               WHERE t2.run_id IN (SELECT run_id FROM vibe_runs r2 \
                                   WHERE r2.created_at > NOW() - INTERVAL '24 hours')), 0) AS total_tokens \
         FROM vibe_runs \
         WHERE created_at > NOW() - INTERVAL '24 hours' \
         GROUP BY project_name \
         ORDER BY total_runs DESC \
         LIMIT 10",
    )
    .load::<RunSummaryRow>(&mut conn)
    .map_err(|e| format!("query: {e}"))?;
    Ok(rows)
}

/// Distills the most relevant items from the LLM's point of view.
async fn llm_relevance(history: &[RunSummaryRow]) -> Option<String> {
    let settings = crate::llm_client::resolve_llm(None, None, None);
    if settings.url.is_empty() || settings.key.is_empty() {
        log::debug!("daily briefing: no LLM configured; skipping relevance summary");
        return None;
    }
    let mut summary = String::new();
    for app in history {
        summary.push_str(&format!(
            "- {}: {} runs ({} ok, {} failed), {} tool calls\n",
            app.project_name, app.total_runs, app.completed, app.failed, app.tool_calls,
        ));
    }
    let system = "You are a concise engineering assistant. From the Vibe agent run summary below, \
                  pick the 2-3 most relevant items worth a human's attention today (failed runs, \
                  heavy activity, anomalies). Reply in 2 short bullet lines max, plain text, no markdown headers.";
    match crate::llm_client::chat_completion(&settings, system, &summary).await {
        Ok(text) => {
            let trimmed = text.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        }
        Err(e) => {
            log::warn!("daily briefing: LLM relevance summary failed: {e}");
            None
        }
    }
}

/// Convenience for the emit closure: refresh the briefing for a pool.
pub fn build_daily_briefing_blocking(pool: &DbPool) -> String {
    let pool = pool.clone();
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        let result = if let Ok(rt) = rt {
            rt.block_on(async move { build_daily_briefing(&pool).await })
        } else {
            "Daily briefing: unavailable.".to_string()
        };
        let _ = tx.send(result);
    });
    rx.recv_timeout(std::time::Duration::from_secs(45))
        .unwrap_or_else(|_| "Daily briefing: generation timed out.".to_string())
}
