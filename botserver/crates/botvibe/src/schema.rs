//! Schema bootstrap helper for the Vibe crates.
//!
//! Postgres rejects multiple DDL statements in a single prepared statement
//! ("cannot insert multiple commands into a prepared statement"), so the
//! multi-statement schema constants cannot go through `diesel::sql_query`
//! as one string. This splits each schema on `;` and executes the statements
//! sequentially, making `ensure_schema` work on a fresh database.

use diesel::r2d2::{ConnectionManager, PooledConnection};
use diesel::RunQueryDsl;
use diesel::PgConnection;

pub type PgConn = PooledConnection<ConnectionManager<PgConnection>>;

/// Executes every non-empty statement of `schema` against `conn`.
pub fn ensure_schema_sql(conn: &mut PgConn, schema: &str, ctx: &str) -> Result<(), String> {
    for stmt in schema.split(';') {
        let statement = stmt.trim();
        if statement.is_empty() {
            continue;
        }
        diesel::sql_query(statement)
            .execute(conn)
            .map_err(|e| format!("{ctx}: {e}"))?;
    }
    Ok(())
}