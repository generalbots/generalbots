use axum::http::StatusCode;
use rust_decimal::Decimal;
use diesel::RunQueryDsl;

use crate::db;

pub fn ensure_schema_sync() -> Result<(), (StatusCode, String)> {
    let pool = db::pool()?;
    let mut conn = pool.get().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Pool error: {e}")))?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS brazil_nfe (
            id UUID PRIMARY KEY,
            number TEXT NOT NULL,
            series TEXT NOT NULL DEFAULT '1',
            emitter_cnpj TEXT NOT NULL DEFAULT '',
            recipient_cnpj TEXT NOT NULL DEFAULT '',
            total NUMERIC(18,2) NOT NULL DEFAULT 0,
            status VARCHAR(30) NOT NULL DEFAULT 'pending',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            authorized_at TIMESTAMPTZ,
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS brazil_nfse (
            id UUID PRIMARY KEY,
            number TEXT NOT NULL,
            service_code TEXT NOT NULL DEFAULT '',
            provider_cnpj TEXT NOT NULL DEFAULT '',
            total NUMERIC(18,2) NOT NULL DEFAULT 0,
            status VARCHAR(30) NOT NULL DEFAULT 'pending',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS brazil_cte (
            id UUID PRIMARY KEY,
            number TEXT NOT NULL,
            sender_cnpj TEXT NOT NULL DEFAULT '',
            recipient_cnpj TEXT NOT NULL DEFAULT '',
            modality TEXT NOT NULL DEFAULT 'normal',
            total NUMERIC(18,2) NOT NULL DEFAULT 0,
            status VARCHAR(30) NOT NULL DEFAULT 'pending',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS brazil_sped (
            id UUID PRIMARY KEY,
            period TEXT NOT NULL,
            kind TEXT NOT NULL DEFAULT 'fiscal',
            status VARCHAR(30) NOT NULL DEFAULT 'open',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
        )",
    )
    .execute(&mut conn)
    .map_err(db::map_diesel_err)?;
    Ok(())
}

pub fn parse_decimal(s: &str) -> Result<Decimal, (StatusCode, String)> {
    s.parse::<Decimal>()
        .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid decimal '{s}': {e}")))
}

/// Loads composite tax rates from the existing `billing_tax_rates` fiscal
/// model (branch-scoped, active rows only). Rates are stored as percentages.
/// A nil branch resolves to the default bot's branch so anonymous/bot-driven
/// calls (chat tools) still pick up the seeded rates.
pub fn load_rates_from_billing(
    conn: &mut diesel::PgConnection,
    branch_id: &uuid::Uuid,
) -> Option<crate::calculator::TaxRates> {
    #[derive(diesel::QueryableByName)]
    struct RateRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        name: String,
        #[diesel(sql_type = diesel::sql_types::Numeric)]
        rate: Decimal,
    }
    let rows: Vec<RateRow> = if *branch_id == uuid::Uuid::nil() {
        diesel::sql_query(
            "SELECT name, rate FROM billing_tax_rates \
             WHERE is_active = true AND ( \
               branch_id = (SELECT branch_id FROM bots WHERE is_default_for_branch = true ORDER BY created_at ASC LIMIT 1) OR \
               branch_id IS NULL)",
        )
        .load(conn)
        .ok()?
    } else {
        diesel::sql_query(
            "SELECT name, rate FROM billing_tax_rates \
             WHERE is_active = true AND (branch_id = $1 OR branch_id IS NULL)",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .load(conn)
        .ok()?
    };

    if rows.is_empty() {
        return None;
    }
    let mut rates = crate::calculator::TaxRates::default();
    for row in rows {
        rates.set(&row.name, row.rate);
    }
    Some(rates)
}
