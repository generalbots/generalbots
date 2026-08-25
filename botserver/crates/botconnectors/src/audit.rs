use chrono::Utc;
use diesel::prelude::*;
use serde_json::json;
use tracing::warn;
use uuid::Uuid;

/// Best-effort audit trail for permissioned search queries.
///
/// Writes one structured tracing line and one `consent_audit` row with
/// outcome `'query'`. Persistence failures never fail the caller — they are
/// logged and swallowed, since auditing must not break serving.
///
/// The raw query text is never persisted; only `q_hash` (a caller-computed
/// digest of the query) is stored.
pub fn audit_query(
    conn: &mut PgConnection,
    user_id: Option<Uuid>,
    q_hash: &str,
    sources_hit: &[String],
    items_returned: usize,
) {
    let request = json!({
        "q_hash": q_hash,
        "sources": sources_hit,
        "items_returned": items_returned,
        "at": Utc::now().to_rfc3339(),
    });

    tracing::info!(
        target: "botconnectors::audit",
        user_id = ?user_id,
        q_hash = %q_hash,
        sources = ?sources_hit,
        items_returned = items_returned,
        "connector search executed"
    );

    let result = diesel::sql_query(
        "INSERT INTO consent_audit (user_id, request, outcome) VALUES ($1, $2::jsonb, 'query')",
    )
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(user_id)
    .bind::<diesel::sql_types::Text, _>(request.to_string())
    .execute(conn);

    if let Err(e) = result {
        warn!("botconnectors: consent_audit insert failed (query not audited in DB): {e}");
    }
}
