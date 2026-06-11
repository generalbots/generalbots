//! Integração SaaS → CRM, ERP (GL) e Assinaturas.
//!
//! Conecta o fluxo de checkout do SaaS com:
//! - **CRM**: cria contact e deal (oportunidade) no pipeline
//! - **ERP (GL)**: lança entrada no razão contábil quando fatura é paga
//! - **Subscription**: cria registro de recorrência em `billing_recurring`

use chrono::Utc;
use diesel::sql_types::{Date, Int4, Nullable, Numeric, Text, Timestamptz, Uuid as DieselUuid};
use diesel::{Connection, RunQueryDsl};
use uuid::Uuid;

use crate::DbPool;

/// Cria um contato no CRM (`crm_contacts`) e retorna o ID gerado.
pub fn create_crm_contact(
    pool: &DbPool,
    org_id: Uuid,
    bot_id: Uuid,
    name: &str,
    email: &str,
) -> Result<Uuid, String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let id = Uuid::new_v4();
    let parts: Vec<&str> = name.splitn(2, ' ').collect();
    let first_name = parts.first().unwrap_or(&"");
    let last_name = parts.get(1);

    diesel::sql_query(
        r#"INSERT INTO crm_contacts (id, org_id, bot_id, first_name, last_name, email, status, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, 'active', NOW(), NOW())"#,
    )
    .bind::<DieselUuid, _>(id)
    .bind::<DieselUuid, _>(org_id)
    .bind::<DieselUuid, _>(bot_id)
    .bind::<Text, _>(first_name)
    .bind::<Nullable<Text>, _>(last_name.map(|s| s.to_string()))
    .bind::<Text, _>(email)
    .execute(&mut conn)
    .map_err(|e| format!("Insert crm_contact: {e}"))?;

    Ok(id)
}

/// Cria um deal (oportunidade) no CRM (`crm_deals`) vinculado ao contato e à fatura.
pub fn create_crm_deal(
    pool: &DbPool,
    org_id: Uuid,
    bot_id: Uuid,
    contact_id: Uuid,
    invoice_id: Uuid,
    deal_name: &str,
    value: f64,
    currency: &str,
) -> Result<Uuid, String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let id = Uuid::new_v4();
    let now = Utc::now();

    diesel::sql_query(
        r#"INSERT INTO crm_deals (id, org_id, bot_id, contact_id, name, title, value, currency, stage, probability, owner_id, created_at, updated_at, deal_date)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'qualification', 10, $9, $10, $10, $10::date)"#,
    )
    .bind::<DieselUuid, _>(id)
    .bind::<DieselUuid, _>(org_id)
    .bind::<DieselUuid, _>(bot_id)
    .bind::<DieselUuid, _>(contact_id)
    .bind::<Text, _>(deal_name)
    .bind::<Text, _>(deal_name)
    .bind::<Numeric, _>(botbilling::api_models::bd(value))
    .bind::<Text, _>(currency)
    .bind::<DieselUuid, _>(Uuid::nil())
    .bind::<Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(|e| format!("Insert crm_deal: {e}"))?;

    Ok(id)
}

/// Marca um deal como ganho (won) e registra a data de fechamento.
pub fn win_crm_deal(
    pool: &DbPool,
    org_id: Uuid,
    deal_name: &str,
) -> Result<(), String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let now = Utc::now();

    diesel::sql_query(
        r#"UPDATE crm_deals SET stage = 'won', won = true, probability = 100,
           actual_close_date = $1::date, closed_at = $1, updated_at = $1
           WHERE org_id = $2 AND name = $3 AND (won IS NULL OR won = false) AND stage <> 'lost'"#,
    )
    .bind::<Timestamptz, _>(now)
    .bind::<DieselUuid, _>(org_id)
    .bind::<Text, _>(deal_name)
    .execute(&mut conn)
    .map_err(|e| format!("Win crm_deal: {e}"))?;

    Ok(())
}

/// Cria uma organização na tabela `organizations` e retorna o ID.
pub fn create_organization(pool: &DbPool, name: &str) -> Result<Uuid, String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let id = Uuid::new_v4();
    let slug = name.to_lowercase().replace(' ', "-");

    diesel::sql_query(
        r#"INSERT INTO organizations (org_id, name, slug, created_at, updated_at)
           VALUES ($1, $2, $3, NOW(), NOW())"#,
    )
    .bind::<DieselUuid, _>(id)
    .bind::<Text, _>(name)
    .bind::<Text, _>(&slug)
    .execute(&mut conn)
    .map_err(|e| format!("Insert organization: {e}"))?;

    Ok(id)
}

/// Cria um lançamento contábil (GL) para uma fatura paga.
///
/// Estrutura:
/// - Débito: Contas a Receber (ou Caixa, conforme o método)
/// - Crédito: Receita de Assinatura
pub fn create_gl_entry_for_invoice(
    pool: &DbPool,
    invoice_id: Uuid,
    total: f64,
    customer_name: &str,
) -> Result<Uuid, String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let entry_id = Uuid::new_v4();
    let now = Utc::now();

    // Busca ou cria contas GL padrão
    let ar_account_id = get_or_create_gl_account(&mut conn, "1.1.01", "Contas a Receber - Assinaturas", "asset")?;
    let revenue_account_id = get_or_create_gl_account(&mut conn, "4.1.01", "Receita de Assinatura SaaS", "revenue")?;

    let entry_date = now.date_naive();
    let description = format!("Fatura paga: {customer_name}");

    conn.transaction(|tx| {
        diesel::sql_query(
            "INSERT INTO gl_journal_entries (id, bot_id, entry_date, description, reference_type, reference_id, status, created_at) \
             VALUES ($1, '00000000-0000-0000-0000-000000000000', $2, $3, 'invoice', $4, 'posted', $5)",
        )
        .bind::<DieselUuid, _>(entry_id)
        .bind::<Date, _>(entry_date)
        .bind::<Text, _>(&description)
        .bind::<DieselUuid, _>(invoice_id)
        .bind::<Timestamptz, _>(now)
        .execute(tx)
        .map_err(|e| format!("Insert gl_entry: {e}"))?;

        // Débito: Contas a Receber
        diesel::sql_query(
            "INSERT INTO gl_journal_lines (id, entry_id, account_id, debit, credit, description) \
             VALUES ($1, $2, $3, $4, 0, $5)",
        )
        .bind::<DieselUuid, _>(Uuid::new_v4())
        .bind::<DieselUuid, _>(entry_id)
        .bind::<DieselUuid, _>(ar_account_id)
        .bind::<Numeric, _>(botbilling::api_models::bd(total))
        .bind::<Text, _>("Recebimento assinatura SaaS")
        .execute(tx)
        .map_err(|e| format!("Insert gl_line debit: {e}"))?;

        // Crédito: Receita de Assinatura
        diesel::sql_query(
            "INSERT INTO gl_journal_lines (id, entry_id, account_id, debit, credit, description) \
             VALUES ($1, $2, $3, 0, $4, $5)",
        )
        .bind::<DieselUuid, _>(Uuid::new_v4())
        .bind::<DieselUuid, _>(entry_id)
        .bind::<DieselUuid, _>(revenue_account_id)
        .bind::<Numeric, _>(botbilling::api_models::bd(total))
        .bind::<Text, _>("Reconhecimento de receita SaaS")
        .execute(tx)
        .map_err(|e| format!("Insert gl_line credit: {e}"))?;

        Ok::<_, String>(())
    })?;

    Ok(entry_id)
}

fn get_or_create_gl_account(
    conn: &mut diesel::PgConnection,
    code: &str,
    name: &str,
    account_type: &str,
) -> Result<Uuid, String> {
    // Tenta buscar conta existente
    let existing: Option<Uuid> = diesel::sql_query(
        "SELECT id FROM gl_accounts WHERE code = $1",
    )
    .bind::<Text, _>(code)
    .get_result(conn)
    .ok();

    if let Some(id) = existing {
        return Ok(id);
    }

    // Cria nova conta
    let id = Uuid::new_v4();
    diesel::sql_query(
        "INSERT INTO gl_accounts (id, bot_id, code, name, account_type, is_active, created_at) \
         VALUES ($1, '00000000-0000-0000-0000-000000000000', $2, $3, $4, true, NOW())",
    )
    .bind::<DieselUuid, _>(id)
    .bind::<Text, _>(code)
    .bind::<Text, _>(name)
    .bind::<Text, _>(account_type)
    .execute(conn)
    .map_err(|e| format!("Create gl_account: {e}"))?;

    Ok(id)
}

/// Cria um registro de assinatura recorrente (`billing_recurring`) para o bot.
pub fn create_billing_subscription(
    pool: &DbPool,
    org_id: Uuid,
    bot_id: Uuid,
    customer_name: &str,
    customer_email: &str,
    plan: &str,
    total: f64,
    currency: &str,
    invoice_id: Uuid,
    period: &str,
) -> Result<Uuid, String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let id = Uuid::new_v4();
    let now = Utc::now();
    let frequency = if period == "yearly" { "yearly" } else { "monthly" };

    let interval_count = 1i32;
    let next_invoice = if frequency == "monthly" {
        now.date_naive() + chrono::Duration::days(30)
    } else {
        now.date_naive() + chrono::Duration::days(365)
    };

    diesel::sql_query(
        r#"INSERT INTO billing_recurring
           (id, org_id, bot_id, customer_name, customer_email, status, frequency, interval_count,
            amount, currency, description, next_invoice_date, start_date, last_invoice_id,
            invoices_generated, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, 'active', $6, $7, $8, $9, $10, $11, $12, $13, 1, $14, $14)"#,
    )
    .bind::<DieselUuid, _>(id)
    .bind::<DieselUuid, _>(org_id)
    .bind::<DieselUuid, _>(bot_id)
    .bind::<Text, _>(customer_name)
    .bind::<Nullable<Text>, _>(Some(customer_email.to_string()))
    .bind::<Text, _>(frequency)
    .bind::<diesel::sql_types::Int4, _>(interval_count)
    .bind::<Numeric, _>(botbilling::api_models::bd(total))
    .bind::<Text, _>(currency)
    .bind::<Nullable<Text>, _>(Some(format!("Assinatura {plan} - {frequency}")))
    .bind::<Date, _>(next_invoice)
    .bind::<Date, _>(now.date_naive())
    .bind::<DieselUuid, _>(invoice_id)
    .bind::<Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(|e| format!("Insert billing_recurring: {e}"))?;

    Ok(id)
}
