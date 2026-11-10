//! Dataset manifest and the tenant-scoped probes the suite runs against it.
//!
//! `scripts/gen_integrity_dataset.py` writes the SQL and a manifest holding, per
//! table and per scope id, the expected row count and the digest
//! `md5(string_agg(id::text, ',' ORDER BY id::text))`. The probes below recompute
//! exactly that, so a row that moved to another tenant changes a checksum instead
//! of hiding in a count that still adds up.

use std::collections::BTreeMap;
use std::path::Path;

use diesel::prelude::*;
use diesel::sql_types::{BigInt, Text, Uuid as SqlUuid};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub seed: i64,
    pub scale: f64,
    pub sql_file: String,
    pub total_rows: i64,
    pub orgs: Vec<Org>,
    pub tables: BTreeMap<String, TableExpectation>,
}

#[derive(Debug, Deserialize)]
pub struct Org {
    pub slug: String,
    pub org_id: String,
    pub tenant_id: String,
    pub branch_ids: Vec<String>,
    pub bot_ids: Vec<String>,
    pub user_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct TableExpectation {
    /// One of `org_id`, `branch_id`, `bot_id`, `session_branch`, `ticket_branch`.
    pub scope: String,
    pub rows: i64,
    pub null_rows: i64,
    pub null_checksum: Option<String>,
    pub scopes: BTreeMap<String, ScopeExpectation>,
}

#[derive(Debug, Deserialize)]
pub struct ScopeExpectation {
    pub rows: i64,
    pub checksum: String,
}

#[derive(QueryableByName)]
pub struct Probe {
    #[diesel(sql_type = BigInt)]
    pub row_count: i64,
    #[diesel(sql_type = Text)]
    pub id_checksum: String,
}

#[derive(QueryableByName)]
pub struct RowCount {
    #[diesel(sql_type = BigInt)]
    pub row_count: i64,
}

/// Reads `integrity_manifest.json` from the dataset directory.
pub fn load(dir: &Path) -> Result<Manifest, String> {
    let path = dir.join("integrity_manifest.json");
    let text =
        std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
}

/// `FROM` target for a scoped probe; join-based scopes reach the owner through
/// the parent row, exactly like the handlers do.
pub fn from_clause(table: &str, alias: &str, kind: &str) -> Result<String, String> {
    match kind {
        "org_id" | "branch_id" | "bot_id" => Ok(format!("{table} AS {alias}")),
        "session_branch" => Ok(format!(
            "{table} AS {alias} \
             JOIN user_sessions AS us_{alias} ON us_{alias}.id = {alias}.session_id"
        )),
        "ticket_branch" => Ok(format!(
            "{table} AS {alias} \
             JOIN support_tickets AS st_{alias} ON st_{alias}.id = {alias}.ticket_id"
        )),
        other => Err(format!("unknown scope kind `{other}`")),
    }
}

/// Column carrying the tenant/branch/bot the row belongs to.
pub fn scope_expr(alias: &str, kind: &str) -> Result<String, String> {
    match kind {
        "org_id" | "branch_id" | "bot_id" => Ok(format!("{alias}.{kind}")),
        "session_branch" => Ok(format!("us_{alias}.branch_id")),
        "ticket_branch" => Ok(format!("st_{alias}.branch_id")),
        other => Err(format!("unknown scope kind `{other}`")),
    }
}

fn query(conn: &mut PgConnection, sql: String) -> Result<Probe, String> {
    diesel::sql_query(sql)
        .get_result::<Probe>(conn)
        .map_err(|e| format!("scoped probe failed: {e}"))
}

/// Rows and id checksum visible through a single tenant scope.
pub fn probe(
    conn: &mut PgConnection,
    table: &str,
    kind: &str,
    scope_id: Uuid,
) -> Result<Probe, String> {
    let sql = format!(
        "SELECT count(*) AS row_count, \
                coalesce(md5(string_agg(t.id::text, ',' ORDER BY t.id::text)), '') AS id_checksum \
         FROM {} WHERE {} = $1",
        from_clause(table, "t", kind)?,
        scope_expr("t", kind)?
    );
    diesel::sql_query(sql)
        .bind::<SqlUuid, _>(scope_id)
        .get_result::<Probe>(conn)
        .map_err(|e| format!("probe {table} by {kind}: {e}"))
}

/// Rows with no tenant at all — the nil-branch / global fallback bucket.
pub fn null_probe(conn: &mut PgConnection, table: &str, kind: &str) -> Result<Probe, String> {
    let sql = format!(
        "SELECT count(*) AS row_count, \
                coalesce(md5(string_agg(t.id::text, ',' ORDER BY t.id::text)), '') AS id_checksum \
         FROM {} WHERE {} IS NULL",
        from_clause(table, "t", kind)?,
        scope_expr("t", kind)?
    );
    query(conn, sql)
}

/// How many rows a query scoped to `own` sees that actually belong to `foreign`.
/// Must always be zero: that is the cross-tenant read the issue asks us to prove.
pub fn leaked_rows(
    conn: &mut PgConnection,
    table: &str,
    kind: &str,
    own: Uuid,
    foreign: Uuid,
) -> Result<i64, String> {
    let sql = format!(
        "SELECT count(*) AS row_count FROM {} WHERE {} = $1 \
         AND t.id IN (SELECT t2.id FROM {} WHERE {} = $2)",
        from_clause(table, "t", kind)?,
        scope_expr("t", kind)?,
        from_clause(table, "t2", kind)?,
        scope_expr("t2", kind)?
    );
    diesel::sql_query(sql)
        .bind::<SqlUuid, _>(own)
        .bind::<SqlUuid, _>(foreign)
        .get_result::<RowCount>(conn)
        .map(|row| row.row_count)
        .map_err(|e| format!("leak probe {table} by {kind}: {e}"))
}

/// Total rows in a table, unscoped — the denominator of the partition check.
pub fn total_rows(conn: &mut PgConnection, table: &str) -> Result<i64, String> {
    diesel::sql_query(format!("SELECT count(*) AS row_count FROM {table}"))
        .get_result::<RowCount>(conn)
        .map(|row| row.row_count)
        .map_err(|e| format!("count {table}: {e}"))
}

/// `true` when a query bound to `id` returns at least one row — used to prove the
/// dataset really is a nested org -> branch -> bot tree in the database.
pub fn exists_with(conn: &mut PgConnection, sql: &str, id: Uuid) -> Result<bool, String> {
    diesel::sql_query(sql.to_string())
        .bind::<SqlUuid, _>(id)
        .get_result::<RowCount>(conn)
        .map(|row| row.row_count > 0)
        .map_err(|e| format!("existence probe failed: {e}"))
}

/// Every scope id in the manifest for one table, in deterministic order.
pub fn scope_ids(expectation: &TableExpectation) -> Vec<(Uuid, &ScopeExpectation)> {
    expectation
        .scopes
        .iter()
        .filter_map(|(raw, expected)| match Uuid::parse_str(raw) {
            Ok(id) => Some((id, expected)),
            Err(_) => None,
        })
        .collect()
}

/// Verifies counts and checksums for every table/scope in the manifest.
/// Returns one line per mismatch so a failure lists all of them at once.
pub fn verify(conn: &mut PgConnection, manifest: &Manifest) -> Vec<String> {
    let mut failures = Vec::new();
    for (table, expectation) in &manifest.tables {
        let kind = expectation.scope.as_str();
        let mut scoped_total = 0_i64;
        for (scope_id, expected) in scope_ids(expectation) {
            match probe(conn, table, kind, scope_id) {
                Ok(found) => {
                    scoped_total += found.row_count;
                    if found.row_count != expected.rows {
                        failures.push(format!(
                            "{table} scope {scope_id}: {}/{} rows visible (leak or loss)",
                            found.row_count, expected.rows
                        ));
                    } else if found.id_checksum != expected.checksum {
                        failures.push(format!(
                            "{table} scope {scope_id}: id checksum {} != {}",
                            found.id_checksum, expected.checksum
                        ));
                    }
                }
                Err(e) => failures.push(format!("{table} scope {scope_id}: {e}")),
            }
        }
        match null_probe(conn, table, kind) {
            Ok(found) => {
                if found.row_count != expectation.null_rows {
                    failures.push(format!(
                        "{table}: {} unscoped rows, manifest says {}",
                        found.row_count, expectation.null_rows
                    ));
                }
                if let Some(expected) = &expectation.null_checksum {
                    if found.id_checksum != *expected {
                        failures.push(format!(
                            "{table}: unscoped id checksum {} != {expected}",
                            found.id_checksum
                        ));
                    }
                }
                scoped_total += found.row_count;
            }
            Err(e) => failures.push(format!("{table}: {e}")),
        }
        match total_rows(conn, table) {
            Ok(total) if total != scoped_total => failures.push(format!(
                "{table}: {total} rows but the scopes account for {scoped_total} \
                 — rows carry a scope the manifest does not know"
            )),
            Ok(_) => {}
            Err(e) => failures.push(e),
        }
        let expected_total = expectation.rows + expectation.null_rows;
        if expected_total != scoped_total {
            failures.push(format!(
                "{table}: manifest expects {expected_total} rows, the probes account for {scoped_total}"
            ));
        }
    }
    failures
}
