use botcore::shared::state::AppState;
use botschema::conversation_autotasks as table;
use chrono::Utc;
use diesel::prelude::*;

use super::{STATUS_ORPHANED, STATUS_RUNNING, STATUS_SCHEDULED};

/// Marks every live row as orphaned at boot (#1613). Lazy reconciliation in
/// `list` promotes it back as soon as the workspace confirms the entry, so a
/// job that survived a restart is reported correctly on first open.
pub fn reconcile_on_boot(state: &AppState) {
    let mut conn = match state.conn.get() {
        Ok(conn) => conn,
        Err(error) => {
            log::warn!("conversation autotask reconciliation skipped: {error}");
            return;
        }
    };
    let touched = diesel::update(table::table)
        .filter(table::status.eq_any(vec![STATUS_RUNNING, STATUS_SCHEDULED]))
        .set((
            table::status.eq(STATUS_ORPHANED),
            table::updated_at.eq(Utc::now()),
        ))
        .execute(&mut conn);
    match touched {
        Ok(count) if count > 0 => {
            log::info!("conversation autotasks: {count} row(s) pending reconciliation")
        }
        Ok(_) => {}
        Err(error) => log::warn!("conversation autotask reconciliation failed: {error}"),
    }
}
