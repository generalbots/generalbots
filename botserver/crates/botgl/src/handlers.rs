use crate::types::{
    BalanceSheetRow, CreateAccountRequest, CreateJournalEntryRequest, GlAccount, GlJournalEntry,
    IncomeStatementRow, TrialBalanceRow,
};
use axum::{extract::State, http::StatusCode, routing::get, Json, Router};
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

pub type DbPool = diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<PgConnection>>;

#[derive(Clone)]
pub struct GlState {
    pub pool: DbPool,
}

pub fn configure_gl_routes() -> Router<Arc<GlState>> {
    Router::new()
        .route("/api/erp/gl/accounts", get(list_accounts).post(create_account))
        .route("/api/erp/gl/accounts/:id", get(get_account))
        .route("/api/erp/gl/entries", get(list_entries).post(create_entry))
        .route("/api/erp/gl/trial-balance", get(trial_balance))
        .route("/api/erp/gl/income-statement", get(income_statement))
        .route("/api/erp/gl/balance-sheet", get(balance_sheet))
}

async fn list_accounts(
    State(state): State<Arc<GlState>>,
) -> Result<Json<Vec<GlAccount>>, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let accounts = diesel::sql_query(
        "SELECT id, branch_id, code, name, account_type, parent_id, is_active, created_at \
         FROM gl_accounts ORDER BY code",
    )
    .load::<GlAccountDbRow>(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|r| GlAccount {
        id: r.id,
        branch_id: r.branch_id,
        code: r.code,
        name: r.name,
        account_type: r.account_type,
        parent_id: r.parent_id,
        is_active: r.is_active,
        created_at: r.created_at,
    })
    .collect();
    Ok(Json(accounts))
}

async fn create_account(
    State(state): State<Arc<GlState>>,
    Json(payload): Json<CreateAccountRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let id = Uuid::new_v4();

    diesel::sql_query(
        "INSERT INTO gl_accounts (id, branch_id, code, name, account_type, parent_id, is_active) \
         VALUES ($1, '00000000-0000-0000-0000-000000000000', $2, $3, $4, $5, true)",
    )
    .bind::<diesel::sql_types::Uuid, _>(&id)
    .bind::<diesel::sql_types::Text, _>(&payload.code)
    .bind::<diesel::sql_types::Text, _>(&payload.name)
    .bind::<diesel::sql_types::Text, _>(&payload.account_type)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(&payload.parent_id)
    .execute(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"id": id})))
}

async fn get_account(
    State(state): State<Arc<GlState>>,
    axum::extract::Path(id): axum::extract::Path<Uuid>,
) -> Result<Json<GlAccount>, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let row = diesel::sql_query(
        "SELECT id, branch_id, code, name, account_type, parent_id, is_active, created_at \
         FROM gl_accounts WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(&id)
    .get_result::<GlAccountDbRow>(&mut conn)
    .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(GlAccount {
        id: row.id,
        branch_id: row.branch_id,
        code: row.code,
        name: row.name,
        account_type: row.account_type,
        parent_id: row.parent_id,
        is_active: row.is_active,
        created_at: row.created_at,
    }))

}

async fn list_entries(
    State(state): State<Arc<GlState>>,
) -> Result<Json<Vec<GlJournalEntry>>, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let entries = diesel::sql_query(
        "SELECT id, branch_id, entry_date, description, reference_type, reference_id, \
         status, created_by, posted_at, created_at \
         FROM gl_journal_entries ORDER BY entry_date DESC",
    )
    .load::<GlEntryDbRow>(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|r| GlJournalEntry {
        id: r.id,
        branch_id: r.branch_id,
        entry_date: r.entry_date,
        description: r.description,
        reference_type: r.reference_type,
        reference_id: r.reference_id,
        status: r.status,
        created_by: r.created_by,
        posted_at: r.posted_at,
        created_at: r.created_at,
    })
    .collect();
    Ok(Json(entries))
}

async fn create_entry(
    State(state): State<Arc<GlState>>,
    Json(payload): Json<CreateJournalEntryRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let entry_id = Uuid::new_v4();

    conn.transaction(|tx| {
        diesel::sql_query(
            "INSERT INTO gl_journal_entries (id, branch_id, entry_date, description, reference_type, reference_id, status) \
             VALUES ($1, '00000000-0000-0000-0000-000000000000', $2, $3, $4, $5, 'draft')",
        )
        .bind::<diesel::sql_types::Uuid, _>(&entry_id)
        .bind::<diesel::sql_types::Date, _>(&payload.entry_date)
        .bind::<diesel::sql_types::Text, _>(&payload.description)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&payload.reference_type)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(&payload.reference_id)
        .execute(tx)?;

        for line in &payload.lines {
            diesel::sql_query(
                "INSERT INTO gl_journal_lines (id, entry_id, account_id, debit, credit, description) \
                 VALUES ($1, $2, $3, $4, $5, $6)",
            )
            .bind::<diesel::sql_types::Uuid, _>(&Uuid::new_v4())
            .bind::<diesel::sql_types::Uuid, _>(&entry_id)
            .bind::<diesel::sql_types::Uuid, _>(&line.account_id)
            .bind::<diesel::sql_types::Numeric, _>(&line.debit)
            .bind::<diesel::sql_types::Numeric, _>(&line.credit)
            .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(&line.description)
            .execute(tx)?;
        }

        Ok::<_, diesel::result::Error>(())
    })
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(serde_json::json!({"id": entry_id})))
}

async fn trial_balance(
    State(state): State<Arc<GlState>>,
) -> Result<Json<Vec<TrialBalanceRow>>, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let rows = diesel::sql_query(
        "SELECT a.code, a.name, a.account_type, \
         COALESCE(SUM(l.debit), 0) as debit_total, \
         COALESCE(SUM(l.credit), 0) as credit_total, \
         COALESCE(SUM(l.debit - l.credit), 0) as balance \
         FROM gl_accounts a \
         LEFT JOIN gl_journal_lines l ON l.account_id = a.id \
         LEFT JOIN gl_journal_entries e ON e.id = l.entry_id AND e.status = 'posted' \
         GROUP BY a.id, a.code, a.name, a.account_type \
         ORDER BY a.code",
    )
    .load::<TrialBalanceDbRow>(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|r| TrialBalanceRow {
        account_code: r.code,
        account_name: r.name,
        account_type: r.account_type,
        debit_total: r.debit_total,
        credit_total: r.credit_total,
        balance: r.balance,
    })
    .collect();
    Ok(Json(rows))
}

async fn income_statement(
    State(state): State<Arc<GlState>>,
) -> Result<Json<Vec<IncomeStatementRow>>, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let rows = diesel::sql_query(
        "SELECT a.code, a.name, \
         COALESCE(SUM(CASE WHEN a.account_type IN ('revenue','income') THEN l.credit - l.debit \
                           WHEN a.account_type = 'expense' THEN l.debit - l.credit ELSE 0 END), 0) as amount \
         FROM gl_accounts a \
         LEFT JOIN gl_journal_lines l ON l.account_id = a.id \
         LEFT JOIN gl_journal_entries e ON e.id = l.entry_id AND e.status = 'posted' \
         WHERE a.account_type IN ('revenue','income','expense') \
         GROUP BY a.id, a.code, a.name \
         ORDER BY a.code",
    )
    .load::<IncomeStDbRow>(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|r| IncomeStatementRow {
        account_code: r.code,
        account_name: r.name,
        amount: r.amount,
    })
    .collect();
    Ok(Json(rows))
}

async fn balance_sheet(
    State(state): State<Arc<GlState>>,
) -> Result<Json<Vec<BalanceSheetRow>>, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::SERVICE_UNAVAILABLE)?;
    let rows = diesel::sql_query(
        "SELECT a.code, a.name, a.account_type, \
         COALESCE(SUM(l.debit - l.credit), 0) as balance \
         FROM gl_accounts a \
         LEFT JOIN gl_journal_lines l ON l.account_id = a.id \
         LEFT JOIN gl_journal_entries e ON e.id = l.entry_id AND e.status = 'posted' \
         WHERE a.account_type IN ('asset','liability','equity') \
         GROUP BY a.id, a.code, a.name, a.account_type \
         ORDER BY a.code",
    )
    .load::<BalanceSheetDbRow>(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .into_iter()
    .map(|r| BalanceSheetRow {
        account_code: r.code,
        account_name: r.name,
        account_type: r.account_type,
        balance: r.balance,
    })
    .collect();
    Ok(Json(rows))
}

// Diesel queryable rows
#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct GlAccountDbRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Text)]
    code: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    account_type: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    parent_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Bool)]
    is_active: bool,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct GlEntryDbRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    branch_id: Uuid,
    #[diesel(sql_type = diesel::sql_types::Date)]
    entry_date: chrono::NaiveDate,
    #[diesel(sql_type = diesel::sql_types::Text)]
    description: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    reference_type: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    reference_id: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Uuid>)]
    created_by: Option<Uuid>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    posted_at: Option<chrono::DateTime<chrono::Utc>>,
    #[diesel(sql_type = diesel::sql_types::Timestamptz)]
    created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct TrialBalanceDbRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    code: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    account_type: String,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    debit_total: rust_decimal::Decimal,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    credit_total: rust_decimal::Decimal,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    balance: rust_decimal::Decimal,
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct IncomeStDbRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    code: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    amount: rust_decimal::Decimal,
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct BalanceSheetDbRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    code: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    name: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    account_type: String,
    #[diesel(sql_type = diesel::sql_types::Numeric)]
    balance: rust_decimal::Decimal,
}
