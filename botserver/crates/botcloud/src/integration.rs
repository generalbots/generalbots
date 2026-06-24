//! SaaS Integration → CRM, ERP (GL) and Subscriptions.
//!
//! Connects the SaaS checkout flow with:
//! - **CRM**: creates contact and deal (opportunity) in pipeline
//! - **ERP (GL)**: posts entry to general ledger when invoice is paid
//! - **Subscription**: creates recurring record in `billing_recurring`
//! - **Signup**: creates tenant, org, branch, bot, MinIO bucket, trial subscription, workspace

use chrono::{Utc, Duration};
use diesel::deserialize::QueryableByName;
use diesel::sql_types::{Date, Nullable, Numeric, Text, Timestamptz, Uuid as DieselUuid};
use diesel::RunQueryDsl;
use std::process::Command;
use std::io::Write;
use uuid::Uuid;

/// Helper struct to read Uuid from raw SQL queries.
#[derive(QueryableByName, Debug)]
struct IdRow {
    #[diesel(sql_type = DieselUuid)]
    id: Uuid,
}

use crate::{DbPool, SaasConfig};

/// Creates a contact in CRM (`crm_contacts`) and returns the generated ID.
pub fn create_crm_contact(
    pool: &DbPool,
    org_id: Uuid,
    bot_id: Uuid,
    name: &str,
    email: &str,
    pass_hash: Option<&str>,
) -> Result<Uuid, String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let id = Uuid::new_v4();
    let parts: Vec<&str> = name.splitn(2, ' ').collect();
    let first_name = parts.first().unwrap_or(&"");
    let last_name = parts.get(1);

    diesel::sql_query(
        r#"INSERT INTO crm_contacts (id, org_id, bot_id, first_name, last_name, email, pass_hash, status, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, $6, $7, 'active', NOW(), NOW())"#,
    )
    .bind::<DieselUuid, _>(id)
    .bind::<DieselUuid, _>(org_id)
    .bind::<DieselUuid, _>(bot_id)
    .bind::<Text, _>(first_name)
    .bind::<Nullable<Text>, _>(last_name.map(|s| s.to_string()))
    .bind::<Text, _>(email)
    .bind::<Nullable<Text>, _>(pass_hash.map(|s| s.to_string()))
    .execute(&mut conn)
    .map_err(|e| format!("Insert crm_contact: {e}"))?;

    Ok(id)
}

/// Creates a deal (opportunity) in CRM (`crm_deals`) linked to contact and invoice.
pub fn create_crm_deal(
    pool: &DbPool,
    org_id: Uuid,
    bot_id: Uuid,
    contact_id: Uuid,
    _invoice_id: Uuid,
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

/// Marks a deal as won and records the closing date.
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

/// Creates an organization in the `organizations` table and returns the ID.
pub fn create_organization(pool: &DbPool, name: &str, domain: Option<&str>) -> Result<Uuid, String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let id = Uuid::new_v4();
    let slug = name.to_lowercase().replace(' ', "-");

    diesel::sql_query(
        r#"INSERT INTO organizations (org_id, name, slug, domain, created_at, updated_at)
           VALUES ($1, $2, $3, $4, NOW(), NOW())"#,
    )
    .bind::<DieselUuid, _>(id)
    .bind::<Text, _>(name)
    .bind::<Text, _>(&slug)
    .bind::<Nullable<Text>, _>(domain)
    .execute(&mut conn)
    .map_err(|e| format!("Insert organization: {e}"))?;

    Ok(id)
}

/// Creates a general ledger (GL) entry for a paid invoice.
///
/// Structure:
/// - Debit: Accounts Receivable (or Cash, depending on method)
/// - Credit: Subscription Revenue
pub fn create_gl_entry_for_invoice(
    pool: &DbPool,
    invoice_id: Uuid,
    total: f64,
    customer_name: &str,
) -> Result<Uuid, String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let entry_id = Uuid::new_v4();
    let now = Utc::now();

    // Find or create default GL accounts
    let ar_account_id = get_or_create_gl_account(&mut conn, "1.1.01", "Contas a Receber - Assinaturas", "asset")?;
    let revenue_account_id = get_or_create_gl_account(&mut conn, "4.1.01", "Receita de Assinatura SaaS", "revenue")?;

    let entry_date = now.date_naive();
    let description = format!("Fatura paga: {customer_name}");

    conn.build_transaction().read_write().run::<_, diesel::result::Error, _>(|tx| {
        diesel::sql_query(
            "INSERT INTO gl_journal_entries (id, bot_id, entry_date, description, reference_type, reference_id, status, created_at) \
             VALUES ($1, '00000000-0000-0000-0000-000000000000', $2, $3, 'invoice', $4, 'posted', $5)",
        )
        .bind::<DieselUuid, _>(entry_id)
        .bind::<Date, _>(entry_date)
        .bind::<Text, _>(&description)
        .bind::<DieselUuid, _>(invoice_id)
        .bind::<Timestamptz, _>(now)
        .execute(tx)?;

        // Debit: Accounts Receivable
        diesel::sql_query(
            "INSERT INTO gl_journal_lines (id, entry_id, account_id, debit, credit, description) \
             VALUES ($1, $2, $3, $4, 0, $5)",
        )
        .bind::<DieselUuid, _>(Uuid::new_v4())
        .bind::<DieselUuid, _>(entry_id)
        .bind::<DieselUuid, _>(ar_account_id)
        .bind::<Numeric, _>(botbilling::api_models::bd(total))
        .bind::<Text, _>("Recebimento assinatura SaaS")
        .execute(tx)?;

        // Credit: Subscription Revenue
        diesel::sql_query(
            "INSERT INTO gl_journal_lines (id, entry_id, account_id, debit, credit, description) \
             VALUES ($1, $2, $3, 0, $4, $5)",
        )
        .bind::<DieselUuid, _>(Uuid::new_v4())
        .bind::<DieselUuid, _>(entry_id)
        .bind::<DieselUuid, _>(revenue_account_id)
        .bind::<Numeric, _>(botbilling::api_models::bd(total))
        .bind::<Text, _>("Reconhecimento de receita SaaS")
        .execute(tx)?;

        Ok(())
    })
    .map_err(|e| format!("Transaction: {e}"))?;

    Ok(entry_id)
}

fn get_or_create_gl_account(
    conn: &mut diesel::PgConnection,
    code: &str,
    name: &str,
    account_type: &str,
) -> Result<Uuid, String> {
    // Try to find existing account
    let existing: Option<Uuid> = diesel::sql_query(
        "SELECT id FROM gl_accounts WHERE code = $1",
    )
    .bind::<Text, _>(code)
    .get_result::<IdRow>(conn)
    .ok()
    .map(|r| r.id);

    if let Some(id) = existing {
        return Ok(id);
    }

    // Create new account
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

/// Creates a recurring subscription record (`billing_recurring`) for the bot.
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

/// Returns the default tenant ID, creating it if it doesn't exist.
pub fn get_or_create_default_tenant(pool: &DbPool) -> Result<Uuid, String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;

    let existing: Option<Uuid> = diesel::sql_query(
        "SELECT id FROM tenants WHERE slug = 'default' LIMIT 1",
    )
    .get_result::<IdRow>(&mut conn)
    .ok()
    .map(|r| r.id);

    if let Some(id) = existing {
        return Ok(id);
    }

    let id = Uuid::new_v4();
    diesel::sql_query(
        "INSERT INTO tenants (id, name, slug, is_active, created_at, updated_at) \
         VALUES ($1, 'Default Tenant', 'default', true, NOW(), NOW())",
    )
    .bind::<DieselUuid, _>(id)
    .execute(&mut conn)
    .map_err(|e| format!("Insert tenant: {e}"))?;

    Ok(id)
}

/// Links an organization to a tenant.
pub fn link_org_to_tenant(pool: &DbPool, org_id: Uuid, tenant_id: Uuid) -> Result<(), String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;

    diesel::sql_query(
        "UPDATE organizations SET tenant_id = $1 WHERE org_id = $2",
    )
    .bind::<DieselUuid, _>(tenant_id)
    .bind::<DieselUuid, _>(org_id)
    .execute(&mut conn)
    .map_err(|e| format!("Link org to tenant: {e}"))?;

    Ok(())
}

/// Creates a branch under an organization and tenant.
pub fn create_branch(pool: &DbPool, org_id: Uuid, tenant_id: Uuid, name: &str) -> Result<Uuid, String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let id = Uuid::new_v4();
    let slug = name.to_lowercase().replace(' ', "-");

    diesel::sql_query(
        r#"INSERT INTO branches (id, org_id, tenant_id, slug, name, is_active, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, true, NOW(), NOW())"#,
    )
    .bind::<DieselUuid, _>(id)
    .bind::<DieselUuid, _>(org_id)
    .bind::<DieselUuid, _>(tenant_id)
    .bind::<Text, _>(&slug)
    .bind::<Text, _>(name)
    .execute(&mut conn)
    .map_err(|e| format!("Insert branch: {e}"))?;

    Ok(id)
}

/// Creates a bot record in the database and returns (bot_id, org_slug).
pub fn create_bot(pool: &DbPool, org_id: Uuid, branch_id: Uuid, _tenant_id: Uuid, name: &str) -> Result<(Uuid, String), String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let bot_id = Uuid::new_v4();
    let slug = name.to_lowercase().replace(' ', "-");
    let org_slug = slug.clone();
    let now = Utc::now();

    diesel::sql_query(
        r#"INSERT INTO bots
           (id, name, slug, org_id, branch_id, tenant_id, is_default_for_branch,
            is_active, created_at, updated_at, llm_provider, llm_config,
            context_provider, context_config, is_public, database_name)
           VALUES ($1, $2, $3, $4, $5, $6, true,
                   true, $7, $7, 'openai', '{}'::jsonb,
                    'openai', '{}'::jsonb, true, $8)"#,
    )
    .bind::<DieselUuid, _>(bot_id)
    .bind::<Text, _>(name)
    .bind::<Text, _>(&slug)
    .bind::<DieselUuid, _>(org_id)
    .bind::<DieselUuid, _>(branch_id)
    .bind::<DieselUuid, _>(Uuid::nil())
    .bind::<Timestamptz, _>(now)
    .bind::<Nullable<Text>, _>(Some(format!("cloud_{}", slug)))
    .execute(&mut conn)
    .map_err(|e| format!("Insert bot: {e}"))?;

    Ok((bot_id, org_slug))
}

/// Creates MinIO `.gborg` bucket for the org and uploads default start.bas inside.
/// Structure: {org_slug}.gborg / {bot_slug}.gbai / {bot_slug}.gbdialog / start.bas
/// Requires mc alias to be pre-configured (e.g., via Vault credentials).
pub fn create_bot_bucket(config: &SaasConfig, org_slug: &str, bot_slug: &str, bot_name: &str) -> Result<(), String> {
    let mc = &config.mc_path;
    let alias = &config.mc_alias;
    let org_bucket = format!("{alias}/{org_slug}.gborg");

    let mb = Command::new(mc)
        .args(["mb", &org_bucket, "--ignore-existing"])
        .output()
        .map_err(|e| format!("mc mb failed: {e}"))?;

    if !mb.status.success() {
        let stderr = String::from_utf8_lossy(&mb.stderr);
        return Err(format!("Failed to create bucket {org_bucket}: {stderr}"));
    }

    let start_bas_content = format!("TALK \"Olá! Sou o {bot_name}. Como posso ajudar?\"");
    let remote_path = format!("{org_bucket}/{bot_slug}.gbai/{bot_slug}.gbdialog/start.bas");

    let tmpfile = format!("/tmp/start_{bot_slug}.bas");
    let mut f = std::fs::File::create(&tmpfile)
        .map_err(|e| format!("Failed to create temp file: {e}"))?;
    f.write_all(start_bas_content.as_bytes())
        .map_err(|e| format!("Failed to write temp file: {e}"))?;
    drop(f);

    let cp = Command::new(mc)
        .args(["cp", &tmpfile, &remote_path])
        .output()
        .map_err(|e| format!("mc cp failed: {e}"))?;

    let _ = std::fs::remove_file(&tmpfile);

    if !cp.status.success() {
        let stderr = String::from_utf8_lossy(&cp.stderr);
        return Err(format!("Failed to upload start.bas to {remote_path}: {stderr}"));
    }

    Ok(())
}

/// Creates a trial subscription in billing_recurring for the given plan and trial days.
pub fn create_trial_subscription(
    pool: &DbPool,
    org_id: Uuid,
    bot_id: Uuid,
    _branch_id: Uuid,
    customer_name: &str,
    customer_email: &str,
    plan: &str,
    trial_days: i32,
) -> Result<Uuid, String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let sub_id = Uuid::new_v4();
    let now = Utc::now();
    let trial_end = now.date_naive() + Duration::days(trial_days as i64);

    diesel::sql_query(
        r#"INSERT INTO billing_recurring
           (id, org_id, bot_id, customer_name, customer_email, status, frequency, interval_count,
            amount, currency, description, next_invoice_date, start_date, last_invoice_id,
            invoices_generated, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, 'trialing', 'monthly', 1, 0.0, 'USD',
                   $6, $7, $8, NULL, 0, $9, $9)"#,
    )
    .bind::<DieselUuid, _>(sub_id)
    .bind::<DieselUuid, _>(org_id)
    .bind::<DieselUuid, _>(bot_id)
    .bind::<Text, _>(customer_name)
    .bind::<Nullable<Text>, _>(Some(customer_email.to_string()))
    .bind::<Nullable<Text>, _>(Some(format!("{plan} - {trial_days} Day Trial")))
    .bind::<Date, _>(trial_end)
    .bind::<Date, _>(now.date_naive())
    .bind::<Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(|e| format!("Insert trial subscription: {e}"))?;

    Ok(sub_id)
}

/// Creates an active free subscription in billing_recurring.
/// No trial, no amount — starts immediately as 'active' with $0.00.
pub fn create_free_subscription(
    pool: &DbPool,
    org_id: Uuid,
    bot_id: Uuid,
    _branch_id: Uuid,
    customer_name: &str,
    customer_email: &str,
) -> Result<Uuid, String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let sub_id = Uuid::new_v4();
    let now = Utc::now();

    diesel::sql_query(
        r#"INSERT INTO billing_recurring
           (id, org_id, bot_id, customer_name, customer_email, status, frequency, interval_count,
            amount, currency, description, next_invoice_date, start_date, last_invoice_id,
            invoices_generated, created_at, updated_at)
           VALUES ($1, $2, $3, $4, $5, 'active', 'monthly', 1, 0.0, 'USD',
                   'Free Plan', $8, $6, NULL, 0, $7, $7)"#,
    )
    .bind::<DieselUuid, _>(sub_id)
    .bind::<DieselUuid, _>(org_id)
    .bind::<DieselUuid, _>(bot_id)
    .bind::<Text, _>(customer_name)
    .bind::<Nullable<Text>, _>(Some(customer_email.to_string()))
    .bind::<Date, _>(now.date_naive())
    .bind::<Date, _>(now.date_naive())
    .bind::<Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(|e| format!("Insert free subscription: {e}"))?;

    Ok(sub_id)
}

/// Creates a default cloud workspace for the organization.
pub fn create_cloud_workspace(pool: &DbPool, org_id: Uuid, name: &str) -> Result<Uuid, String> {
    let mut conn = pool.get().map_err(|e| format!("DB pool: {e}"))?;
    let id = Uuid::new_v4();
    let now = Utc::now();

    diesel::sql_query(
        r#"INSERT INTO cloud_workspaces (id, org_id, name, description, icon, created_at, updated_at)
           VALUES ($1, $2, $3, $4, 'default', $5, $5)"#,
    )
    .bind::<DieselUuid, _>(id)
    .bind::<DieselUuid, _>(org_id)
    .bind::<Text, _>(name)
    .bind::<Nullable<Text>, _>(Some(format!("Default workspace for {}", name)))
    .bind::<Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(|e| format!("Insert cloud_workspace: {e}"))?;

    Ok(id)
}
