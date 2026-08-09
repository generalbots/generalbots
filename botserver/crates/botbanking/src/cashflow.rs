//! Cash-flow sheet import into the banking ledger (issue #724).
//!
//! `POST /api/banking/imports/cashflow` imports the rows of a month's
//! cash-flow spreadsheet into the existing `bank_transactions` table with
//! `reconciled = false`, so they flow into the standard reconciliation
//! pipeline later. Only valid rows are imported (unparseable dates/amounts
//! are reported, never silently dropped). The sheet is read by explicit
//! Drive key (generic object access — no folder heuristics) or passed
//! directly as rows.
//!
//! `GET /api/banking/diagnosis` returns the cash-flow
//! result for the whole branch (an import affects the global cash flow, so
//! the diagnosis is never per-import): a summary plus the transaction
//! detail, along with the tax rates the tax engine resolves. It is pure
//! data — the LLM turns it into the diagnosis text and can run tax
//! experiments with the user through the calculate endpoint.

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::Json;
use botcore::shared::state::AppState;
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

const IMPORT_BANK_MARKER: &str = "cashflow-import";

#[derive(Debug, Deserialize, Serialize)]
pub struct CashflowRow {
    pub date: String,
    pub description: Option<String>,
    pub amount: String,
    #[serde(rename = "type")]
    pub kind: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CashflowImportRequest {
    /// Drive key of the sheet (e.g. `financeiro/fluxo-caixa-2026-08.csv`).
    /// When present the file is read from the bot's bucket; otherwise `rows`
    /// must be supplied.
    pub file_key: Option<String>,
    /// Optional YYYY-MM period; when absent it is derived from the rows
    /// (or defaults to the current month).
    pub period: Option<String>,
    /// Rows to import (used when `file_key` is absent).
    pub rows: Option<Vec<CashflowRow>>,
    /// Optional bot id; when absent the default bot is used.
    pub bot_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CashflowImportResponse {
    pub imported: usize,
    pub skipped_invalid: usize,
    pub invalid: Vec<serde_json::Value>,
    pub period: String,
    pub revenue: f64,
    pub expenses: f64,
    pub net: f64,
}

#[derive(Debug, Deserialize)]
pub struct DiagnosisQuery {
    pub period: Option<String>,
    pub bot_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DiagnosisRow {
    pub date: String,
    pub description: String,
    pub amount: f64,
    pub category: String,
    pub reconciled: bool,
    pub bank: String,
}

#[derive(Debug, Serialize)]
pub struct DiagnosisSummary {
    pub entries: usize,
    pub revenue: f64,
    pub expenses: f64,
    pub net: f64,
    pub pending_reconciliation: usize,
}

#[derive(Debug, Serialize)]
pub struct DiagnosisResponse {
    pub diagnosed: bool,
    pub period: Option<String>,
    pub scope: serde_json::Value,
    pub summary: DiagnosisSummary,
    pub detail: Vec<DiagnosisRow>,
    pub tax_rates: serde_json::Value,
    pub tax_rates_source: String,
    pub reason: Option<String>,
}

#[derive(diesel::QueryableByName, Clone)]
pub struct BotScopeRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    pub branch_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    pub name: String,
}

pub fn resolve_bot_scope(
    pool: &botcore::shared::utils::DbPool,
    bot_id: &Uuid,
    caller_branch: Option<Uuid>,
) -> Option<BotScopeRow> {
    let mut conn = pool.get().ok()?;
    let scope = if *bot_id == Uuid::nil() {
        diesel::sql_query(
            "SELECT id, branch_id, name FROM bots \
             WHERE is_default_for_branch = true ORDER BY created_at ASC LIMIT 1",
        )
        .get_result(&mut conn)
        .ok()
    } else {
        diesel::sql_query("SELECT id, branch_id, name FROM bots WHERE id = $1")
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .get_result(&mut conn)
            .ok()
    };
    // Authorize the resolved scope against the caller's tenant (issue #734):
    // a client-supplied bot_id may only reference a bot that belongs to the
    // caller's branch. Anonymous/internal callers pass None (server-resolved).
    match caller_branch {
        Some(branch) => scope.filter(|s: &BotScopeRow| s.branch_id == branch),
        None => scope,
    }
}

fn parse_br_amount(s: &str) -> Option<f64> {
    let cleaned: String = s
        .replace(['R', '$', ' '], "")
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == ',' || *c == '.' || *c == '-')
        .collect();
    let parsed = if cleaned.contains(',') && cleaned.contains('.') {
        let last_comma = cleaned.rfind(',').map(|i| i + 1).unwrap_or(0);
        let last_dot = cleaned.rfind('.').map(|i| i + 1).unwrap_or(0);
        let sep = if last_comma > last_dot { ',' } else { '.' };
        if sep == ',' {
            cleaned.replace('.', "").replace(',', ".")
        } else {
            cleaned.replace(',', "")
        }
    } else if cleaned.contains(',') {
        cleaned.replace(',', ".")
    } else {
        cleaned
    };
    parsed.parse::<f64>().ok()
}

fn parse_br_date(s: &str) -> Option<NaiveDate> {
    let trimmed = s.trim();
    if let Ok(d) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
        return Some(d);
    }
    if let Ok(d) = NaiveDate::parse_from_str(trimmed, "%d/%m/%Y") {
        return Some(d);
    }
    NaiveDate::parse_from_str(trimmed, "%Y/%m/%d").ok()
}

fn is_expense(kind: &str, amount: f64) -> bool {
    amount < 0.0
        || kind.contains("despesa")
        || kind.contains("saida")
        || kind.contains("debito")
}

fn round2(v: f64) -> f64 {
    (v * 100.0).round() / 100.0
}

/// Validates and normalizes one sheet row into a signed bank-transaction row.
fn normalize_row(row: &CashflowRow) -> Result<(NaiveDate, String, f64, String, String), serde_json::Value> {
    let date = parse_br_date(&row.date).ok_or_else(|| {
        serde_json::json!({ "row": row, "reason": format!("invalid date '{}'", row.date) })
    })?;
    let raw_amount = parse_br_amount(&row.amount).ok_or_else(|| {
        serde_json::json!({ "row": row, "reason": format!("invalid amount '{}'", row.amount) })
    })?;
    let kind = row
        .kind
        .clone()
        .unwrap_or_default()
        .to_lowercase();
    let negative = is_expense(&kind, raw_amount);
    let amount = if negative { -raw_amount.abs() } else { raw_amount.abs() };
    let category = if negative { "despesa".to_string() } else { "receita".to_string() };
    let description = row
        .description
        .clone()
        .unwrap_or_default()
        .trim()
        .to_string();
    Ok((date, description, amount, category, kind))
}

/// Imports valid cash-flow rows into `bank_transactions` (reconciled=false)
/// so the existing reconciliation pipeline can process them later.
pub async fn import_cashflow(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Json(req): Json<CashflowImportRequest>,
) -> Result<Json<CashflowImportResponse>, (StatusCode, Json<serde_json::Value>)> {
    let err = |code: StatusCode, msg: &str| (code, Json(serde_json::json!({ "error": msg })));
    let bot_id = req
        .bot_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::nil());
    let drive = state.drive.as_ref();
    let caller_branch = botcore::shared::tenant::branch_from_claims(&headers);
    match cashflow_import_inner(
        &state.conn,
        drive,
        &bot_id,
        caller_branch,
        req.file_key.as_deref(),
        req.rows,
        req.period,
    )
    .await
    {
        Ok(resp) => Ok(Json(resp)),
        Err(msg) => Err(err(StatusCode::BAD_REQUEST, &msg)),
    }
}

/// Shared business logic for the cash-flow import (issue #724). Used by the
/// REST handler and by the LLM api-command catalog so chat can adjust a
/// month's entries over any channel.
pub async fn cashflow_import_inner(
    pool: &botcore::shared::utils::DbPool,
    drive: Option<&Arc<dyn botlib::traits::DriveRepository>>,
    bot_id: &Uuid,
    caller_branch: Option<Uuid>,
    file_key: Option<&str>,
    rows: Option<Vec<CashflowRow>>,
    period: Option<String>,
) -> Result<CashflowImportResponse, String> {
    let scope = resolve_bot_scope(pool, bot_id, caller_branch)
        .ok_or_else(|| "Bot not found".to_string())?;

    // Sheet content: explicit drive key or supplied rows.
    let csv_text = if let Some(file_key) = file_key {
        let drive = drive.ok_or_else(|| "Drive unavailable".to_string())?;
        let bucket = format!("{}.gbai", scope.name);
        let key = format!("{}.gbdrive/{}", scope.name, file_key.trim_start_matches('/'));
        let bytes = drive
            .get_object(&bucket, &key)
            .await
            .map_err(|e| format!("Sheet not found: {e}"))?;
        Some(String::from_utf8_lossy(&bytes).to_string())
    } else {
        None
    };

    let rows: Vec<CashflowRow> = if let Some(rows) = rows {
        rows
    } else if let Some(text) = csv_text {
        parse_csv_rows(&text)
    } else {
        return Err("Provide 'rows' or 'file_key'".to_string());
    };

    let mut normalized: Vec<(NaiveDate, String, f64, String)> = Vec::new();
    let mut invalid: Vec<serde_json::Value> = Vec::new();
    for row in &rows {
        match normalize_row(row) {
            Ok((date, description, amount, category, _kind)) => {
                normalized.push((date, description, amount, category));
            }
            Err(invalid_row) => invalid.push(invalid_row),
        }
    }

    let period = period
        .filter(|p| !p.trim().is_empty())
        .or_else(|| {
            normalized
                .first()
                .map(|(date, _, _, _)| date.format("%Y-%m").to_string())
        })
        .unwrap_or_else(|| Utc::now().format("%Y-%m").to_string());

    let mut conn = pool.get().map_err(|e| format!("Pool error: {e}"))?;

    // Idempotent per period: replace the previous import of the same marker.
    let marker = format!("{IMPORT_BANK_MARKER}:{period}");
    diesel::sql_query(
        "DELETE FROM bank_transactions WHERE bot_id = $1 AND branch_id = $2 AND bank = $3",
    )
    .bind::<diesel::sql_types::Uuid, _>(scope.id)
    .bind::<diesel::sql_types::Uuid, _>(scope.branch_id)
    .bind::<diesel::sql_types::Text, _>(&marker)
    .execute(&mut conn)
    .map_err(|e| format!("Cleanup error: {e}"))?;

    let mut revenue = 0.0f64;
    let mut expenses = 0.0f64;
    for (date, description, amount, category) in &normalized {
        if *amount >= 0.0 {
            revenue += amount;
        } else {
            expenses += amount.abs();
        }
        diesel::sql_query(
            "INSERT INTO bank_transactions \
             (id, bot_id, branch_id, bank, account, transaction_date, description, amount, category, reconciled, created_at) \
             VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, false, NOW())",
        )
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Uuid, _>(scope.id)
        .bind::<diesel::sql_types::Uuid, _>(scope.branch_id)
        .bind::<diesel::sql_types::Text, _>(&marker)
        .bind::<diesel::sql_types::Text, _>("cashflow")
        .bind::<diesel::sql_types::Date, _>(*date)
        .bind::<diesel::sql_types::Text, _>(description)
        .bind::<diesel::sql_types::Double, _>(*amount)
        .bind::<diesel::sql_types::Text, _>(category)
        .execute(&mut conn)
        .map_err(|e| format!("Insert error: {e}"))?;
    }

    Ok(CashflowImportResponse {
        imported: normalized.len(),
        skipped_invalid: invalid.len(),
        invalid,
        period,
        revenue: round2(revenue),
        expenses: round2(expenses),
        net: round2(revenue - expenses),
    })
}

/// Returns the cash-flow result for the whole branch (issue #724). The
/// diagnosis runs on the global `bank_transactions` data of the bot's
/// branch — an import changes the cash flow, so it is never scoped to a
/// single import. The response has both a summary and the transaction
/// detail, plus the resolved tax rates so the LLM can reason and experiment.
pub async fn cashflow_diagnosis(
    State(state): State<Arc<AppState>>,
    headers: axum::http::HeaderMap,
    Query(query): Query<DiagnosisQuery>,
) -> Result<Json<DiagnosisResponse>, (StatusCode, Json<serde_json::Value>)> {
    let err = |code: StatusCode, msg: &str| (code, Json(serde_json::json!({ "error": msg })));
    let bot_id = query
        .bot_id
        .as_deref()
        .and_then(|s| Uuid::parse_str(s).ok())
        .unwrap_or(Uuid::nil());
    let caller_branch = botcore::shared::tenant::branch_from_claims(&headers);
    match cashflow_diagnosis_inner(&state.conn, &bot_id, caller_branch, query.period.as_deref()).await {
        Ok(resp) => Ok(Json(resp)),
        Err(msg) => Err(err(StatusCode::BAD_REQUEST, &msg)),
    }
}

/// Shared business logic for the cash-flow diagnosis (#724). Used by the
/// REST handler and by the LLM api-command catalog so chat can answer
/// financial-health questions over WhatsApp.
pub async fn cashflow_diagnosis_inner(
    pool: &botcore::shared::utils::DbPool,
    bot_id: &Uuid,
    caller_branch: Option<Uuid>,
    period_q: Option<&str>,
) -> Result<DiagnosisResponse, String> {
    let scope = resolve_bot_scope(pool, bot_id, caller_branch)
        .ok_or_else(|| "Bot not found".to_string())?;

    let period = period_q.map(|p| p.to_string()).filter(|p| !p.trim().is_empty());

    let mut conn = pool.get().map_err(|e| format!("Pool error: {e}"))?;

    #[derive(diesel::QueryableByName)]
    struct SumRow {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        n: i64,
        #[diesel(sql_type = diesel::sql_types::Double)]
        revenue: f64,
        #[diesel(sql_type = diesel::sql_types::Double)]
        expenses: f64,
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        pending: i64,
    }
    let (sum_sql, sum_bind): (&str, Option<String>) = match &period {
        Some(p) => (
            "SELECT count(*) AS n, \
             COALESCE(SUM(CASE WHEN amount >= 0 THEN amount ELSE 0 END), 0)::float8 AS revenue, \
             COALESCE(SUM(CASE WHEN amount < 0 THEN abs(amount) ELSE 0 END), 0)::float8 AS expenses, \
             COALESCE(SUM(CASE WHEN reconciled = false THEN 1 ELSE 0 END), 0) AS pending \
             FROM bank_transactions \
             WHERE bot_id = $1 AND branch_id = $2 AND to_char(transaction_date, 'YYYY-MM') = $3",
            Some(p.clone()),
        ),
        None => (
            "SELECT count(*) AS n, \
             COALESCE(SUM(CASE WHEN amount >= 0 THEN amount ELSE 0 END), 0)::float8 AS revenue, \
             COALESCE(SUM(CASE WHEN amount < 0 THEN abs(amount) ELSE 0 END), 0)::float8 AS expenses, \
             COALESCE(SUM(CASE WHEN reconciled = false THEN 1 ELSE 0 END), 0) AS pending \
             FROM bank_transactions \
             WHERE bot_id = $1 AND branch_id = $2",
            None,
        ),
    };

    let sums: Option<SumRow> = match &sum_bind {
        Some(p) => diesel::sql_query(sum_sql)
            .bind::<diesel::sql_types::Uuid, _>(scope.id)
            .bind::<diesel::sql_types::Uuid, _>(scope.branch_id)
            .bind::<diesel::sql_types::Text, _>(p.as_str())
            .get_result(&mut conn)
            .ok(),
        None => diesel::sql_query(sum_sql)
            .bind::<diesel::sql_types::Uuid, _>(scope.id)
            .bind::<diesel::sql_types::Uuid, _>(scope.branch_id)
            .get_result(&mut conn)
            .ok(),
    };
    let sums = sums.ok_or_else(|| "No banking data found".to_string())?;

    let (detail_sql, detail_bind) = match &period {
        Some(p) => (
            "SELECT transaction_date, description, amount::float8 AS amount, category, reconciled, bank \
             FROM bank_transactions \
             WHERE bot_id = $1 AND branch_id = $2 AND to_char(transaction_date, 'YYYY-MM') = $3 \
             ORDER BY transaction_date, created_at LIMIT 1000",
            Some(p.clone()),
        ),
        None => (
            "SELECT transaction_date, description, amount::float8 AS amount, category, reconciled, bank \
             FROM bank_transactions \
             WHERE bot_id = $1 AND branch_id = $2 \
             ORDER BY transaction_date, created_at DESC LIMIT 1000",
            None,
        ),
    };

    #[derive(diesel::QueryableByName)]
    struct DetailRow {
        #[diesel(sql_type = diesel::sql_types::Date)]
        transaction_date: NaiveDate,
        #[diesel(sql_type = diesel::sql_types::Text)]
        description: String,
        #[diesel(sql_type = diesel::sql_types::Double)]
        amount: f64,
        #[diesel(sql_type = diesel::sql_types::Text)]
        category: String,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        reconciled: bool,
        #[diesel(sql_type = diesel::sql_types::Text)]
        bank: String,
    }
    let detail: Vec<DetailRow> = match &detail_bind {
        Some(p) => {
            let rows = diesel::sql_query(detail_sql)
                .bind::<diesel::sql_types::Uuid, _>(scope.id)
                .bind::<diesel::sql_types::Uuid, _>(scope.branch_id)
                .bind::<diesel::sql_types::Text, _>(p.as_str())
                .load::<DetailRow>(&mut conn);
            match rows {
                Ok(rows) => rows,
                Err(e) => {
                    log::error!("banking diagnosis detail query failed: {e}");
                    Vec::new()
                }
            }
        }
        None => {
            let rows = diesel::sql_query(detail_sql)
                .bind::<diesel::sql_types::Uuid, _>(scope.id)
                .bind::<diesel::sql_types::Uuid, _>(scope.branch_id)
                .load::<DetailRow>(&mut conn);
            match rows {
                Ok(rows) => rows,
                Err(e) => {
                    log::error!("banking diagnosis detail query failed: {e}");
                    Vec::new()
                }
            }
        }
    };

    let rows: Vec<DiagnosisRow> = detail
        .iter()
        .map(|r| DiagnosisRow {
            date: r.transaction_date.format("%Y-%m-%d").to_string(),
            description: r.description.clone(),
            amount: r.amount,
            category: r.category.clone(),
            reconciled: r.reconciled,
            bank: r.bank.clone(),
        })
        .collect();

    let loaded_rates = bottax::storage::load_rates_from_billing(&mut conn, &scope.branch_id);
    let rates = loaded_rates.unwrap_or_default();
    let source = if loaded_rates.is_some() {
        "billing_tax_rates"
    } else {
        "default"
    };

    Ok(DiagnosisResponse {
        diagnosed: true,
        period,
        scope: serde_json::json!({
            "bot": scope.id,
            "branch": scope.branch_id,
            "name": scope.name,
        }),
        summary: DiagnosisSummary {
            entries: sums.n as usize,
            revenue: round2(sums.revenue),
            expenses: round2(sums.expenses),
            net: round2(sums.revenue - sums.expenses),
            pending_reconciliation: sums.pending as usize,
        },
        detail: rows,
        tax_rates: serde_json::json!({
            "irpj_pct": rates.irpj_pct.to_string(),
            "csll_pct": rates.csll_pct.to_string(),
            "pis_cofins_pct": rates.pis_cofins_pct.to_string(),
            "iss_pct": rates.iss_pct.to_string(),
        }),
        tax_rates_source: source.to_string(),
        reason: None,
    })
}

fn parse_csv_rows(csv: &str) -> Vec<CashflowRow> {
    let mut rows: Vec<CashflowRow> = Vec::new();
    let mut lines = csv.lines();
    let header_line = match lines.next() {
        Some(h) => h,
        None => return rows,
    };
    let headers: Vec<String> = header_line
        .split(',')
        .map(|s| s.trim().trim_matches('"').to_lowercase())
        .collect();
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        let values: Vec<String> = line
            .split(',')
            .map(|s| s.trim().trim_matches('"').to_string())
            .collect();
        let get = |h: &str| -> Option<String> {
            match headers.iter().position(|x| x == h) {
                Some(idx) => values.get(idx).cloned(),
                None => None,
            }
        };
        let date = get("date")
            .or_else(|| get("data"))
            .or_else(|| get("vencimento"))
            .unwrap_or_default();
        let description = get("description")
            .or_else(|| get("descricao"))
            .or_else(|| get("historico"))
            .unwrap_or_default();
        let amount = get("amount").or_else(|| get("valor")).unwrap_or_default();
        let kind = get("type").or_else(|| get("tipo")).unwrap_or_default();
        if date.is_empty() || amount.is_empty() {
            continue;
        }
        rows.push(CashflowRow {
            date,
            description: Some(description),
            amount,
            kind: Some(kind),
            category: None,
        });
    }
    rows
}
