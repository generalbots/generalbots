// aggregate query helpers
use super::*;
use crate::db;

pub(crate) async fn count_nfe() -> Result<i64, String> {
    let pool = db::pool().map_err(|(c, msg)| format!("{c}: {msg}"))?;
    let mut conn = pool.get().map_err(|e| format!("pool get: {e}"))?;
    #[derive(diesel::QueryableByName)]
    struct R { #[diesel(sql_type = diesel::sql_types::BigInt)] c: i64 }
    let r: R = diesel::sql_query("SELECT COUNT(*) AS c FROM brazil_nfe")
        .get_result(&mut conn).map_err(|e| format!("query: {e}"))?;
    Ok(r.c)
}

pub(crate) async fn count_nfse() -> Result<i64, String> {
    let pool = db::pool().map_err(|(c, msg)| format!("{c}: {msg}"))?;
    let mut conn = pool.get().map_err(|e| format!("pool get: {e}"))?;
    #[derive(diesel::QueryableByName)]
    struct R { #[diesel(sql_type = diesel::sql_types::BigInt)] c: i64 }
    let r: R = diesel::sql_query("SELECT COUNT(*) AS c FROM brazil_nfse")
        .get_result(&mut conn).map_err(|e| format!("query: {e}"))?;
    Ok(r.c)
}

pub(crate) async fn count_cte() -> Result<i64, String> {
    let pool = db::pool().map_err(|(c, msg)| format!("{c}: {msg}"))?;
    let mut conn = pool.get().map_err(|e| format!("pool get: {e}"))?;
    #[derive(diesel::QueryableByName)]
    struct R { #[diesel(sql_type = diesel::sql_types::BigInt)] c: i64 }
    let r: R = diesel::sql_query("SELECT COUNT(*) AS c FROM brazil_cte")
        .get_result(&mut conn).map_err(|e| format!("query: {e}"))?;
    Ok(r.c)
}

pub(crate) async fn sum_nfe_total() -> Result<String, String> {
    let pool = db::pool().map_err(|(c, msg)| format!("{c}: {msg}"))?;
    let mut conn = pool.get().map_err(|e| format!("pool get: {e}"))?;
    #[derive(diesel::QueryableByName)]
    struct R { #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Numeric>)] s: Option<rust_decimal::Decimal> }
    let r: R = diesel::sql_query("SELECT COALESCE(SUM(total), 0) AS s FROM brazil_nfe WHERE status = 'authorized'")
        .get_result(&mut conn).map_err(|e| format!("query: {e}"))?;
    Ok(r.s.map(|d| d.to_string()).unwrap_or_else(|| "0".to_string()))
}
