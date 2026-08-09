//! PostgreSQL persistence for Vibe runs (Issue #793).
//!
//! Executes the `VIBE_SCHEMA` DDL and stores runs in the `vibe_runs` table
//! so run state, tool-call history and metrics survive server restarts.
//! The in-memory run map in the API layer remains the authority for live
//! runs; this store is a write-through audit layer plus a query fallback.

use crate::types::{DbPool, VibeRun, VibeRunState};
use diesel::prelude::*;

/// Executes the DDL from `types::VIBE_SCHEMA` against the pool (idempotent).
pub fn ensure_vibe_schema(pool: &DbPool) -> Result<(), String> {
    let mut conn = pool
        .get()
        .map_err(|e| format!("vibe schema: pool get failed: {e}"))?;
    for statement in crate::types::VIBE_SCHEMA
        .split(';')
        .filter(|s| !s.trim().is_empty())
    {
        diesel::sql_query(statement)
            .execute(&mut conn)
            .map_err(|e| format!("vibe schema statement failed: {e}"))?;
    }
    Ok(())
}

/// Write-through + read-back store for `vibe_runs`.
pub struct VibeRunStore {
    pool: DbPool,
}

impl VibeRunStore {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }

    /// Inserts or updates a run row, keeping the JSONB columns in sync.
    pub fn save_run(&self, run: &VibeRun) -> Result<(), String> {
        let mut conn = self
            .pool
            .get()
            .map_err(|e| format!("run store: pool get failed: {e}"))?;
        let config = serde_json::to_value(&run.config)
            .map_err(|e| format!("run store: config serialize: {e}"))?;
        let tool_calls = serde_json::to_value(&run.tool_calls)
            .map_err(|e| format!("run store: tool_calls serialize: {e}"))?;
        let state = serde_json::to_string(&run.state)
            .map_err(|e| format!("run store: state serialize: {e}"))?;
        let state = state.trim_matches('"');
        diesel::sql_query(
            "INSERT INTO vibe_runs \
             (run_id, bot_id, session_id, user_id, state, use_case, config, intent, \
              tool_calls, created_at, updated_at, completed_at, error) \
             VALUES ($1, $2, $3, $4, $5::varchar, $6::varchar, $7, $8, $9, $10, $11, $12, $13) \
             ON CONFLICT (run_id) DO UPDATE SET \
               state = EXCLUDED.state, config = EXCLUDED.config, tool_calls = EXCLUDED.tool_calls, \
               updated_at = EXCLUDED.updated_at, completed_at = EXCLUDED.completed_at, \
               error = EXCLUDED.error",
        )
        .bind::<diesel::sql_types::Uuid, _>(run.run_id)
        .bind::<diesel::sql_types::Uuid, _>(run.bot_id)
        .bind::<diesel::sql_types::Uuid, _>(run.session_id)
        .bind::<diesel::sql_types::Uuid, _>(run.user_id)
        .bind::<diesel::sql_types::Text, _>(state)
        .bind::<diesel::sql_types::Text, _>(run.use_case.to_string())
        .bind::<diesel::sql_types::Jsonb, _>(config)
        .bind::<diesel::sql_types::Text, _>(&run.intent)
        .bind::<diesel::sql_types::Jsonb, _>(tool_calls)
        .bind::<diesel::sql_types::Timestamptz, _>(run.created_at)
        .bind::<diesel::sql_types::Timestamptz, _>(run.updated_at)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>, _>(run.completed_at)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(run.error.clone())
        .execute(&mut conn)
        .map_err(|e| format!("run store: save failed: {e}"))?;
        Ok(())
    }

    /// Loads a run by id, or `None` when no row exists.
    pub fn get_run(&self, run_id: uuid::Uuid) -> Option<VibeRun> {
        let mut conn = self.pool.get().ok()?;
        let row = diesel::sql_query(
            "SELECT run_id, bot_id, session_id, user_id, state, use_case, \
                    config::text AS config, intent, tool_calls::text AS tool_calls, \
                    created_at::text AS created_at, updated_at::text AS updated_at, \
                    completed_at::text AS completed_at, error \
             FROM vibe_runs WHERE run_id = $1",
        )
        .bind::<diesel::sql_types::Uuid, _>(run_id)
        .get_result::<RunRow>(&mut conn)
        .ok()?;
        row.to_vibe_run().ok()
    }

    /// Lists stored runs, newest first, bounded by `limit`.
    pub fn list_runs(&self, limit: i64) -> Vec<VibeRun> {
        let mut conn = match self.pool.get() {
            Ok(conn) => conn,
            Err(_) => return Vec::new(),
        };
        let rows = diesel::sql_query(
            "SELECT run_id, bot_id, session_id, user_id, state, use_case, \
                    config::text AS config, intent, tool_calls::text AS tool_calls, \
                    created_at::text AS created_at, updated_at::text AS updated_at, \
                    completed_at::text AS completed_at, error \
             FROM vibe_runs ORDER BY created_at DESC LIMIT $1",
        )
        .bind::<diesel::sql_types::BigInt, _>(limit)
        .load::<RunRow>(&mut conn)
        .unwrap_or_default();
        rows.iter().filter_map(|r| r.to_vibe_run().ok()).collect()
    }
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct RunRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    run_id: uuid::Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    bot_id: uuid::Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    session_id: uuid::Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    user_id: uuid::Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    state: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    use_case: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    config: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    intent: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    tool_calls: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    created_at: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    updated_at: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    completed_at: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    error: Option<String>,
}

impl RunRow {
    fn to_vibe_run(&self) -> Result<VibeRun, String> {
        let config = serde_json::from_str::<crate::types::VibeRunConfig>(&self.config)
            .map_err(|e| format!("run store: config parse: {e}"))?;
        let tool_calls: Vec<crate::types::VibeToolCall> =
            serde_json::from_str(&self.tool_calls)
                .map_err(|e| format!("run store: tool_calls parse: {e}"))?;
        let state: VibeRunState = serde_json::from_str(&format!("\"{}\"", self.state))
            .map_err(|e| format!("run store: state parse: {e}"))?;
        let parse_ts = |s: &str| -> Result<chrono::DateTime<chrono::Utc>, String> {
            chrono::DateTime::parse_from_rfc3339(s)
                .map(|d| d.with_timezone(&chrono::Utc))
                .map_err(|e| format!("run store: timestamp parse: {e}"))
        };
        Ok(VibeRun {
            run_id: self.run_id,
            bot_id: self.bot_id,
            session_id: self.session_id,
            user_id: self.user_id,
            state,
            use_case: crate::types::VibeUseCase::try_from_str(&self.use_case)
                .unwrap_or(crate::types::VibeUseCase::SoftwareDevelopment),
            config,
            intent: self.intent.clone(),
            tool_calls,
            created_at: parse_ts(&self.created_at)?,
            updated_at: parse_ts(&self.updated_at)?,
            completed_at: self
                .completed_at
                .as_deref()
                .map(parse_ts)
                .transpose()?,
            error: self.error.clone(),
        })
    }
}

/// Helper trait used to map stored text back to a use case.
impl crate::types::VibeUseCase {
    fn try_from_str(s: &str) -> Option<Self> {
        match s {
            "software_development" => Some(Self::SoftwareDevelopment),
            "customer_support" => Some(Self::CustomerSupport),
            "financial_analysis" => Some(Self::FinancialAnalysis),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{VibeRun, VibeRunConfig};

    #[test]
    fn row_round_trip_reconstructs_run() {
        let run = VibeRun::new(
            uuid::Uuid::nil(),
            uuid::Uuid::nil(),
            uuid::Uuid::nil(),
            "intent".to_string(),
            VibeRunConfig::default(),
        );
        let row = RunRow {
            run_id: run.run_id,
            bot_id: run.bot_id,
            session_id: run.session_id,
            user_id: run.user_id,
            state: "pending".to_string(),
            use_case: "software_development".to_string(),
            config: serde_json::to_string(&run.config).expect("config"),
            intent: run.intent.clone(),
            tool_calls: "[]".to_string(),
            created_at: run.created_at.to_rfc3339(),
            updated_at: run.updated_at.to_rfc3339(),
            completed_at: run.completed_at.map(|t| t.to_rfc3339()),
            error: None,
        };
        let rebuilt = row.to_vibe_run().expect("rebuild");
        assert_eq!(rebuilt.run_id, run.run_id);
        assert_eq!(rebuilt.state, VibeRunState::Pending);
        assert_eq!(rebuilt.intent, "intent");
        assert!(rebuilt.tool_calls.is_empty());
    }

    #[test]
    fn use_case_round_trips() {
        assert_eq!(
            crate::types::VibeUseCase::try_from_str("financial_analysis"),
            Some(crate::types::VibeUseCase::FinancialAnalysis)
        );
        assert_eq!(crate::types::VibeUseCase::try_from_str("x"), None);
    }
}