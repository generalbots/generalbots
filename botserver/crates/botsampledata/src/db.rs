//! Database demo-data seeding.
//!
//! All inserts are guarded so re-running is safe. Scope notes:
//!   * Real default branch (a188d531): CRM, tickets, billing, research,
//!     workspaces, o365, drive.
//!   * Nil scope (`00000000-...`): people, social, compliance, goals,
//!     calendar, campaigns, lists — these handlers resolve scope to nil in
//!     suite-admin mode.

use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Integer, Nullable, Text, Uuid as SqlUuid};
use uuid::Uuid;

/// Scopes resolved at seed time from the live database so demo data lands on
/// the instance's actual default org/branch/bot (works in dev and prod).
pub struct Scopes {
    pub org_id: Uuid,
    pub branch_id: Uuid,
    pub bot_id: Uuid,
    pub user_id: Uuid,
    pub org_str: String,
    pub branch_str: String,
    pub bot_str: String,
    pub user_str: String,
}

impl Scopes {
    fn new(conn: &mut diesel::PgConnection) -> Result<Self, String> {
        #[derive(diesel::QueryableByName)]
        struct DefaultBotRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            branch_id: Uuid,
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            bot_id: Uuid,
        }
        // Default bot is the first bot flagged is_default_for_branch.
        let default_bot: Option<DefaultBotRow> = sql_query(
            "SELECT b.branch_id, b.id AS bot_id FROM bots b \
             WHERE b.is_default_for_branch = true ORDER BY b.created_at ASC LIMIT 1",
        )
        .get_result(conn)
        .ok();

        let (branch_id, bot_id) = default_bot
            .map(|r| (r.branch_id, r.bot_id))
            .unwrap_or_else(|| {
                (
                    Uuid::parse_str("0de59833-b46b-4546-b816-d0f73f59d8c0").unwrap_or_default(),
                    Uuid::parse_str("f47ac10b-58cc-4372-a567-0e02b2c3d480").unwrap_or_default(),
                )
            });

        // Org owning that branch.
        #[derive(diesel::QueryableByName)]
        struct OrgRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            org_id: Uuid,
        }
        let org_id: Option<OrgRow> = sql_query(
            "SELECT org_id FROM branches WHERE id = $1 LIMIT 1",
        )
        .bind::<SqlUuid, _>(branch_id)
        .get_result(conn)
        .ok();

        let org_id = org_id.map(|r| r.org_id).unwrap_or_else(Uuid::nil);

        // Resolve the demo user id: prefer the users row matching the demo
        // email (in prod this is the UUIDv5(zitadel:{id}) mapping used by the
        // suite session), falling back to the fixed sample id.
        #[derive(diesel::QueryableByName)]
        struct UserRow {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            user_id: Uuid,
        }
        let user_id: Option<UserRow> = sql_query(
            "SELECT id AS user_id FROM users WHERE email = 'sample@example.com' LIMIT 1",
        )
        .get_result(conn)
        .ok();
        let user_id = user_id
            .map(|r| r.user_id)
            .unwrap_or_else(|| {
                Uuid::parse_str("10000000-0000-0000-0000-000000000001").unwrap_or_default()
            });

        Ok(Scopes {
            org_id,
            branch_id,
            bot_id,
            user_id,
            org_str: org_id.to_string(),
            branch_str: branch_id.to_string(),
            bot_str: bot_id.to_string(),
            user_str: user_id.to_string(),
        })
    }
}

/// Seed every domain. Each domain runs independently: a failure in one app
/// (e.g. a legacy/missing table on an un-migrated prod schema) is logged and
/// the remaining domains still seed.
pub fn seed(conn: &mut diesel::PgConnection) -> Result<(), String> {
    let scopes = Scopes::new(conn)?;
    ensure_user(conn, &scopes)?;

    let domains: Vec<(&str, fn(&mut diesel::PgConnection, &Scopes) -> Result<(), String>)> = vec![
        ("people", seed_people),
        ("crm", seed_crm),
        ("tickets", seed_tickets),
        ("billing", seed_billing),
        ("tasks", seed_tasks),
        ("calendar", seed_calendar),
        ("research", seed_research),
        ("compliance", seed_compliance),
        ("goals", seed_goals),
        ("workspaces", seed_workspaces),
        ("social", seed_social),
        ("marketing", seed_marketing),
        ("o365", seed_m365),
        ("drive", seed_drive),
    ];

    for (name, seed_fn) in domains {
        match seed_fn(conn, &scopes) {
            Ok(()) => log::info!("botsampledata: {name} seeded"),
            Err(e) => log::error!("botsampledata: {name} seeding failed: {e}"),
        }
    }

    Ok(())
}

fn ensure_user(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    sql_query(
        "INSERT INTO users (id, username, email, password_hash, is_active, created_at, updated_at)
         VALUES ($1, 'sample.user', 'sample@example.com', 'x', true, NOW(), NOW())
         ON CONFLICT (id) DO NOTHING",
    )
    .bind::<SqlUuid, _>(s.user_id)
    .execute(conn)
    .map_err(|e| e.to_string())?;
    Ok(())
}

fn count(conn: &mut diesel::PgConnection, sql: &str, binds: &[&str]) -> Result<i64, String> {
    #[derive(diesel::QueryableByName)]
    struct C {
        #[diesel(sql_type = diesel::sql_types::BigInt)]
        n: i64,
    }
    let r = match binds.len() {
        0 => sql_query(sql).get_result::<C>(conn),
        1 => sql_query(sql)
            .bind::<Text, _>(binds[0])
            .get_result::<C>(conn),
        2 => sql_query(sql)
            .bind::<Text, _>(binds[0])
            .bind::<Text, _>(binds[1])
            .get_result::<C>(conn),
        3 => sql_query(sql)
            .bind::<Text, _>(binds[0])
            .bind::<Text, _>(binds[1])
            .bind::<Text, _>(binds[2])
            .get_result::<C>(conn),
        _ => return Err("count: too many binds".to_string()),
    };
    r.map(|r| r.n).map_err(|e| e.to_string())
}

#[derive(diesel::QueryableByName)]
struct UuidRowNamed {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
}

fn seed_people(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let nil = Uuid::nil();
    let user = s.user_id;

    for (dept, code) in [("Engineering", "ENG"), ("Marketing", "MKT")] {
        let n = count(conn, "SELECT count(*) AS n FROM people_departments WHERE org_id::text = $1 AND code = $2", &["00000000-0000-0000-0000-000000000000", code])?;
        if n == 0 {
            sql_query("INSERT INTO people_departments (id, org_id, bot_id, name, description, code, is_active, created_at, updated_at, branch_id)
                       VALUES ($1, $2, $2, $3, $3 || ' dept', $4, true, NOW(), NOW(), $2)")
                .bind::<SqlUuid, _>(Uuid::new_v4())
                .bind::<SqlUuid, _>(nil)
                .bind::<Text, _>(dept)
                .bind::<Text, _>(code)
                .execute(conn)
                .map_err(|e| e.to_string())?;
        }
    }

    let people: &[(&str, &str, &str, &str, &str)] = &[
        ("Alice", "Sample", "alice.sample@example.com", "Software Engineer", "Engineering"),
        ("Bruno", "Demo", "bruno.demo@example.com", "Product Manager", "Marketing"),
        ("Carla", "Test", "carla.test@example.com", "Designer", "Engineering"),
    ];
    for (first, last, email, title, dept) in people {
        let n = count(conn, "SELECT count(*) AS n FROM people WHERE org_id::text = $1 AND email = $2", &["00000000-0000-0000-0000-000000000000", email])?;
        if n == 0 {
            sql_query(
                "INSERT INTO people (id, org_id, bot_id, user_id, first_name, last_name, email, phone, job_title, department, office_location, hire_date, timezone, locale, skills, social_links, custom_fields, created_at, updated_at, branch_id)
                 VALUES ($1, $2, $2, $3, $4, $5, $6, '+1 555 0100', $7, $8, 'Remote', '2024-03-01', 'UTC', 'en-US', ARRAY['rust'], '{}', '{}', NOW(), NOW(), $2)",
            )
                .bind::<SqlUuid, _>(Uuid::new_v4())
                .bind::<SqlUuid, _>(nil)
                .bind::<Nullable<SqlUuid>, _>(Some(user))
                .bind::<Text, _>(first)
                .bind::<Text, _>(last)
                .bind::<Text, _>(email)
                .bind::<Text, _>(title)
                .bind::<Text, _>(dept)
                .execute(conn)
                .map_err(|e| e.to_string())?;
        }
    }

    let n = count(conn, "SELECT count(*) AS n FROM people_skills WHERE org_id::text = $1 AND name = 'Rust'", &["00000000-0000-0000-0000-000000000000"])?;
    if n == 0 {
        sql_query("INSERT INTO people_skills (id, org_id, bot_id, name, category, description, is_active, created_at, branch_id)
                   VALUES ($1, $2, $2, 'Rust', 'Engineering', 'Systems programming', true, NOW(), $2)")
            .bind::<SqlUuid, _>(Uuid::new_v4())
            .bind::<SqlUuid, _>(nil)
            .execute(conn)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn seed_crm(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let branch = s.branch_id;
    let bot = s.bot_id;
    let user = s.user_id;

    let account_id;
    let n = count(conn, "SELECT count(*) AS n FROM crm_accounts WHERE org_id::text = $1 AND name = 'Acme Corp'", &[&s.branch_str])?;
    if n == 0 {
        let row = sql_query(
            "INSERT INTO crm_accounts (id, org_id, bot_id, name, industry, website, phone, created_at, updated_at, branch_id)
             VALUES ($1, $2, $3, 'Acme Corp', 'Technology', 'https://acme.example.com', '+1 555 1000', NOW(), NOW(), $2)
             RETURNING id",
        )
        .bind::<SqlUuid, _>(Uuid::new_v4())
        .bind::<SqlUuid, _>(branch)
        .bind::<SqlUuid, _>(bot)
        .get_result::<UuidRowNamed>(conn)
        .map(|r| r.id)
        .map_err(|e| e.to_string())?;
        account_id = row;
    } else {
        account_id = sql_query("SELECT id FROM crm_accounts WHERE org_id = $1 AND name = 'Acme Corp' LIMIT 1")
            .bind::<SqlUuid, _>(branch)
            .get_result::<UuidRowNamed>(conn)
            .map(|r| r.id)
            .map_err(|e| e.to_string())?;
    }

    let n = count(conn, "SELECT count(*) AS n FROM crm_accounts WHERE org_id::text = $1 AND name = 'Globex Inc'", &[&s.branch_str])?;
    if n == 0 {
        sql_query("INSERT INTO crm_accounts (id, org_id, bot_id, name, industry, website, phone, created_at, updated_at, branch_id)
                   VALUES ($1, $2, $3, 'Globex Inc', 'Retail', 'https://globex.example.com', '+1 555 2000', NOW(), NOW(), $2)")
            .bind::<SqlUuid, _>(Uuid::new_v4())
            .bind::<SqlUuid, _>(branch)
            .bind::<SqlUuid, _>(bot)
            .execute(conn)
            .map_err(|e| e.to_string())?;
    }

    let contact_id;
    let n = count(conn, "SELECT count(*) AS n FROM crm_contacts WHERE org_id::text = $1 AND email = 'john.doe@acme.example.com'", &[&s.branch_str])?;
    if n == 0 {
        let row = sql_query(
            "INSERT INTO crm_contacts (id, org_id, bot_id, first_name, last_name, email, phone, company, job_title, source, status, tags, custom_fields, city, state, country, notes, owner_id, created_at, updated_at, branch_id)
             VALUES ($1, $2, $3, 'John', 'Doe', 'john.doe@acme.example.com', '+1 555 1100', 'Acme Corp', 'CTO', 'web', 'active', ARRAY['enterprise','warm'], '{}', 'Austin', 'TX', 'US', 'Primary technical contact', $4, NOW(), NOW(), $2)
             RETURNING id",
        )
        .bind::<SqlUuid, _>(Uuid::new_v4())
        .bind::<SqlUuid, _>(branch)
        .bind::<SqlUuid, _>(bot)
        .bind::<Nullable<SqlUuid>, _>(Some(user))
        .get_result::<UuidRowNamed>(conn)
        .map(|r| r.id)
        .map_err(|e| e.to_string())?;
        contact_id = row;
    } else {
        contact_id = sql_query("SELECT id FROM crm_contacts WHERE org_id = $1 AND email = 'john.doe@acme.example.com' LIMIT 1")
            .bind::<SqlUuid, _>(branch)
            .get_result::<UuidRowNamed>(conn)
            .map(|r| r.id)
            .map_err(|e| e.to_string())?;
    }

    let n = count(conn, "SELECT count(*) AS n FROM crm_contacts WHERE org_id::text = $1 AND email = 'jane.roe@globex.example.com'", &[&s.branch_str])?;
    if n == 0 {
        sql_query("INSERT INTO crm_contacts (id, org_id, bot_id, first_name, last_name, email, phone, company, job_title, source, status, tags, custom_fields, city, state, country, owner_id, created_at, updated_at, branch_id)
                   VALUES ($1, $2, $3, 'Jane', 'Roe', 'jane.roe@globex.example.com', '+1 555 2100', 'Globex Inc', 'Head of Ops', 'referral', 'active', ARRAY['retail','new'], '{}', 'Seattle', 'WA', 'US', $4, NOW(), NOW(), $2)")
            .bind::<SqlUuid, _>(Uuid::new_v4())
            .bind::<SqlUuid, _>(branch)
            .bind::<SqlUuid, _>(bot)
            .bind::<Nullable<SqlUuid>, _>(Some(user))
            .execute(conn)
            .map_err(|e| e.to_string())?;
    }

    let deals: &[(&str, &str, f64, &str, i32)] = &[
        ("Acme expansion", "Acme expansion", 45000.0, "proposal", 60),
        ("Globex pilot", "Globex pilot", 12000.0, "discovery", 20),
    ];
    for (title, name, value, stage, probability) in deals {
        let n = count(conn, "SELECT count(*) AS n FROM crm_deals WHERE org_id::text = $1 AND title = $2", &[&s.branch_str, title])?;
        if n == 0 {
            sql_query(
                "INSERT INTO crm_deals (id, org_id, bot_id, contact_id, account_id, owner_id, title, name, description, value, currency, stage, probability, won, source, created_at, updated_at, branch_id)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'Demo deal', $9, 'USD', $10, $11, false, 'web', NOW(), NOW(), $2)",
            )
                .bind::<SqlUuid, _>(Uuid::new_v4())
                .bind::<SqlUuid, _>(branch)
                .bind::<SqlUuid, _>(bot)
                .bind::<Nullable<SqlUuid>, _>(Some(contact_id))
                .bind::<Nullable<SqlUuid>, _>(Some(account_id))
                .bind::<Nullable<SqlUuid>, _>(Some(user))
                .bind::<Text, _>(title)
                .bind::<Text, _>(name)
                .bind::<diesel::sql_types::Double, _>(value)
                .bind::<Text, _>(stage)
                .bind::<Integer, _>(*probability)
                .execute(conn)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn seed_tickets(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let org = s.org_id;
    let bot = s.bot_id;
    let branch = s.branch_id;
    let user = s.user_id;

    for (cat, color, icon) in [("Incident", "#ef4444", "alert"), ("Request", "#22c55e", "hand")] {
        let n = count(conn, "SELECT count(*) AS n FROM ticket_categories WHERE org_id::text = $1 AND name = $2", &[&s.org_str, cat])?;
        if n == 0 {
            sql_query("INSERT INTO ticket_categories (id, org_id, bot_id, name, description, color, icon, sort_order, is_active, created_at, branch_id)
                       VALUES ($1, $2, $3, $4, $4 || ' category', $5, $6, 1, true, NOW(), $7)")
                .bind::<SqlUuid, _>(Uuid::new_v4())
                .bind::<SqlUuid, _>(org)
                .bind::<SqlUuid, _>(bot)
                .bind::<Text, _>(cat)
                .bind::<Text, _>(color)
                .bind::<Text, _>(icon)
                .bind::<SqlUuid, _>(branch)
                .execute(conn)
                .map_err(|e| e.to_string())?;
        }
    }

    let tickets: &[(&str, &str, &str, &str, &str, &str)] = &[
        ("TK-1001", "Cannot access dashboard", "Dashboard returns 502 after login", "open", "high", "alice.sample@example.com"),
        ("TK-1002", "New hire onboarding", "Please provision access for new hire", "in_progress", "medium", "bruno.demo@example.com"),
        ("TK-1003", "Export report missing columns", "Monthly export is missing created_at", "resolved", "low", "carla.test@example.com"),
    ];
    for (num, subject, desc, status, priority, requester) in tickets {
        let n = count(conn, "SELECT count(*) AS n FROM support_tickets WHERE org_id::text = $1 AND ticket_number = $2", &[&s.org_str, num])?;
        if n == 0 {
            sql_query(
                "INSERT INTO support_tickets (id, org_id, bot_id, ticket_number, subject, description, status, priority, category, source, requester_id, requester_email, requester_name, assignee_id, tags, created_at, updated_at, branch_id)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'Incident', 'web', $9, $10, 'Sample User', $9, ARRAY['sample'], NOW(), NOW(), $11)",
            )
                .bind::<SqlUuid, _>(Uuid::new_v4())
                .bind::<SqlUuid, _>(org)
                .bind::<SqlUuid, _>(bot)
                .bind::<Text, _>(num)
                .bind::<Text, _>(subject)
                .bind::<Text, _>(desc)
                .bind::<Text, _>(status)
                .bind::<Text, _>(priority)
                .bind::<Nullable<SqlUuid>, _>(Some(user))
                .bind::<Text, _>(requester)
                .bind::<SqlUuid, _>(branch)
                .execute(conn)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn seed_billing(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let org = s.org_id;
    let bot = s.bot_id;
    let branch = s.branch_id;

    let products: &[(&str, &str, &str, f64, &str)] = &[
        ("vps-small", "VPS Small", "4 vCPU / 8 GB RAM / 100 GB NVMe", 9.99, "infrastructure"),
        ("gpu-basic", "GPU Basic", "RTX 3060 12 GB VRAM", 39.99, "infrastructure"),
        ("domain-com", "Domain .com", "Annual .com domain", 21.99, "communication"),
        ("print-pla-100g", "PLA Filament 1kg", "PLA filament 1.75mm black", 19.99, "print3d"),
    ];
    for (sku, name, desc, price, category) in products {
        let n = count(conn, "SELECT count(*) AS n FROM products WHERE org_id::text = $1 AND sku = $2", &[&s.org_str, sku])?;
        if n == 0 {
            sql_query(
                "INSERT INTO products (id, org_id, bot_id, sku, name, description, category, product_type, price, cost, currency, tax_rate, unit, stock_quantity, low_stock_threshold, is_active, attributes, created_at, updated_at, branch_id)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, 'service', $8, $8 * 0.4, 'USD', 0, 'unit', -1, 0, true, '{}'::jsonb, NOW(), NOW(), $9)",
            )
                .bind::<SqlUuid, _>(Uuid::new_v4())
                .bind::<SqlUuid, _>(org)
                .bind::<SqlUuid, _>(bot)
                .bind::<Text, _>(sku)
                .bind::<Text, _>(name)
                .bind::<Text, _>(desc)
                .bind::<Text, _>(category)
                .bind::<diesel::sql_types::Double, _>(price)
                .bind::<SqlUuid, _>(branch)
                .execute(conn)
                .map_err(|e| e.to_string())?;
        }
    }

    let n = count(conn, "SELECT count(*) AS n FROM billing_invoices WHERE org_id::text = $1 AND invoice_number = 'INV-2026-0001'", &[&s.branch_str])?;
    if n == 0 {
        sql_query(
            "INSERT INTO billing_invoices (id, org_id, bot_id, invoice_number, customer_id, customer_name, customer_email, status, issue_date, due_date, subtotal, tax_rate, tax_amount, total, amount_paid, amount_due, currency, notes, created_at, updated_at, branch_id)
             VALUES ($1, $2, $3, 'INV-2026-0001', NULL, 'Acme Corp', 'billing@acme.example.com', 'paid', '2026-07-01', '2026-07-31', 2500, 0, 0, 2500, 2500, 0, 'USD', 'Sample paid invoice', NOW(), NOW(), $2)",
        )
            .bind::<SqlUuid, _>(Uuid::new_v4())
            .bind::<SqlUuid, _>(branch)
            .bind::<SqlUuid, _>(bot)
            .execute(conn)
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn seed_tasks(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let branch = s.branch_id;
    let user = s.user_id;

    let tasks: &[(&str, &str, &str)] = &[
        ("Design onboarding flow", "UX flow for new user onboarding", "in_progress"),
        ("Fix login token refresh", "Token refresh loop after idle", "todo"),
    ];
    for (title, desc, status) in tasks {
        let n = count(conn, "SELECT count(*) AS n FROM tasks WHERE branch_id::text = $1 AND title = $2", &[&s.branch_str, title])?;
        if n == 0 {
            sql_query(
                "INSERT INTO tasks (id, title, description, status, priority, assignee_id, reporter_id, due_date, tags, estimated_hours, actual_hours, progress, created_at, updated_at, branch_id)
                 VALUES ($1, $2, $3, $4, 'high', $5, $5, '2026-08-20', ARRAY['demo'], 12, 6, 50, NOW(), NOW(), $6)",
            )
                .bind::<SqlUuid, _>(Uuid::new_v4())
                .bind::<Text, _>(title)
                .bind::<Text, _>(desc)
                .bind::<Text, _>(status)
                .bind::<Nullable<SqlUuid>, _>(Some(user))
                .bind::<SqlUuid, _>(branch)
                .execute(conn)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn seed_calendar(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let nil = Uuid::nil();
    let user = s.user_id;

    let cal_id;
    let n = count(conn, "SELECT count(*) AS n FROM calendars WHERE org_id::text = $1 AND owner_id::text = $2", &["00000000-0000-0000-0000-000000000000", &s.user_str])?;
    if n == 0 {
        cal_id = sql_query(
            "INSERT INTO calendars (id, org_id, bot_id, owner_id, name, description, color, timezone, is_primary, is_visible, is_shared, created_at, branch_id)
             VALUES ($1, $2, $2, $3, 'Work', 'Primary work calendar', '#3b82f6', 'UTC', true, true, false, NOW(), $2) RETURNING id",
        )
        .bind::<SqlUuid, _>(Uuid::new_v4())
        .bind::<SqlUuid, _>(nil)
        .bind::<SqlUuid, _>(user)
        .get_result::<UuidRowNamed>(conn)
        .map(|r| r.id)
        .map_err(|e| e.to_string())?;
    } else {
        cal_id = sql_query("SELECT id FROM calendars WHERE org_id = $1 AND owner_id = $2 LIMIT 1")
            .bind::<SqlUuid, _>(nil)
            .bind::<SqlUuid, _>(user)
            .get_result::<UuidRowNamed>(conn)
            .map(|r| r.id)
            .map_err(|e| e.to_string())?;
    }

    let events: &[&str] = &["Sprint Planning", "Product Review"];
    for title in events {
        let n = count(conn, "SELECT count(*) AS n FROM calendar_events WHERE org_id::text = $1 AND title = $2", &["00000000-0000-0000-0000-000000000000", title])?;
        if n == 0 {
            sql_query(
                "INSERT INTO calendar_events (id, org_id, bot_id, calendar_id, owner_id, title, description, location, start_time, end_time, status, visibility, busy_status, reminders, attendees, created_at, updated_at, branch_id)
                 VALUES ($1, $2, $2, $3, $4, $5, 'Demo event', 'Virtual', NOW() + INTERVAL '1 day', NOW() + INTERVAL '1 day' + INTERVAL '1 hour', 'confirmed', 'default', 'busy', '[]', '[]', NOW(), NOW(), $2)",
            )
                .bind::<SqlUuid, _>(Uuid::new_v4())
                .bind::<SqlUuid, _>(nil)
                .bind::<SqlUuid, _>(cal_id)
                .bind::<SqlUuid, _>(user)
                .bind::<Text, _>(title)
                .execute(conn)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn seed_research(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let bot = s.bot_id;
    let user = s.user_id;

    for (name, folder, count_docs) in [("Market Research", "/kb/market", 12), ("Product Docs", "/kb/product", 24)] {
        let n = count(conn, "SELECT count(*) AS n FROM kb_collections WHERE bot_id::text = $1 AND name = $2", &[&s.bot_str, name])?;
        if n == 0 {
            sql_query(
                "INSERT INTO kb_collections (id, bot_id, name, folder_path, qdrant_collection, document_count, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, 'default_demo_' || $3, $5, NOW(), NOW())",
            )
                .bind::<SqlUuid, _>(Uuid::new_v4())
                .bind::<SqlUuid, _>(bot)
                .bind::<Text, _>(name)
                .bind::<Text, _>(folder)
                .bind::<Integer, _>(count_docs)
                .execute(conn)
                .map_err(|e| e.to_string())?;
        }
    }

    // Seed documents inside the collections so the universal search endpoint
    // has indexed knowledge-base content to return for the research app.
    let docs: &[(&str, &str, &str)] = &[
        ("Market Research", "/kb/market/enterprise-rag-2025.md", "Enterprise RAG adoption trends and budgets for 2025"),
        ("Market Research", "/kb/market/on-prem-gpu-inference.md", "Cost analysis of on-premises GPU inference clusters"),
        ("Market Research", "/kb/market/competitor-analysis-q2.md", "Competitor feature comparison for conversational AI platforms"),
        ("Product Docs", "/kb/product/api-reference.md", "Complete REST API reference for the General Bots platform"),
        ("Product Docs", "/kb/product/getting-started.md", "Quick start guide covering installation and first bot creation"),
        ("Product Docs", "/kb/product/whatsapp-integration.md", "Connecting WhatsApp Business API to a bot in five steps"),
    ];
    for (collection, file_path, summary) in docs {
        let n = count(
            conn,
            "SELECT count(*) AS n FROM kb_documents WHERE bot_id::text = $1 AND file_path = $2",
            &[&s.bot_str, file_path],
        )?;
        if n == 0 {
            sql_query(
                "INSERT INTO kb_documents (id, bot_id, collection_name, file_path, file_size, file_hash, first_published_at, last_modified_at, fail_count, metadata, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, 2048, md5($4), NOW(), NOW(), 0, jsonb_build_object('summary', $5), NOW(), NOW())",
            )
                .bind::<SqlUuid, _>(Uuid::new_v4())
                .bind::<SqlUuid, _>(bot)
                .bind::<Text, _>(collection)
                .bind::<Text, _>(file_path)
                .bind::<Text, _>(summary)
                .execute(conn)
                .map_err(|e| e.to_string())?;
        }
    }

    for query in ["enterprise rag adoption 2025", "on-prem gpu inference"] {
        let n = count(conn, "SELECT count(*) AS n FROM research_searches WHERE user_id::text = $1 AND query = $2", &[&s.user_str, query])?;
        if n == 0 {
            sql_query("INSERT INTO research_searches (id, user_id, query, created_at) VALUES ($1, $2, $3, NOW())")
                .bind::<SqlUuid, _>(Uuid::new_v4())
                .bind::<SqlUuid, _>(user)
                .bind::<Text, _>(query)
                .execute(conn)
                .map_err(|e| e.to_string())?;
        }
    }

    Ok(())
}

fn seed_compliance(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let nil = Uuid::nil();
    let user = s.user_id;

    let checks: &[(&str, &str, i32)] = &[
        ("SOC2", "compliant", 94),
        ("ISO27001", "non_compliant", 40),
    ];
    for (framework, status, score) in checks {
        let n = count(conn, "SELECT count(*) AS n FROM compliance_checks WHERE branch_id::text = $1 AND check_type = $2", &["00000000-0000-0000-0000-000000000000", framework])?;
        if n == 0 {
            sql_query(
                "INSERT INTO compliance_checks (id, branch_id, check_type, status, target_type, target_id, result, checked_at, checked_by, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, 'control', NULL, jsonb_build_object('score', $5), NOW(), $6, NOW(), NOW())",
            )
                .bind::<SqlUuid, _>(Uuid::new_v4())
                .bind::<SqlUuid, _>(nil)
                .bind::<Text, _>(framework)
                .bind::<Text, _>(status)
                .bind::<Integer, _>(*score)
                .bind::<Nullable<SqlUuid>, _>(Some(user))
                .execute(conn)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn seed_goals(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let nil = Uuid::nil();
    let user = s.user_id;

    let obj_id;
    let n = count(conn, "SELECT count(*) AS n FROM okr_objectives WHERE org_id::text = $1 AND title = 'Ship v1.0'", &["00000000-0000-0000-0000-000000000000"])?;
    if n == 0 {
        obj_id = sql_query(
            "INSERT INTO okr_objectives (id, org_id, bot_id, owner_id, title, description, period, period_start, period_end, status, progress, visibility, tags, created_at, updated_at, branch_id)
             VALUES ($1, $2, $2, $3, 'Ship v1.0', 'Deliver the v1 platform', 'Q3-2026', '2026-07-01', '2026-09-30', 'active', 45, 'team', ARRAY['v1','launch'], NOW(), NOW(), $2) RETURNING id",
        )
        .bind::<SqlUuid, _>(Uuid::new_v4())
        .bind::<SqlUuid, _>(nil)
        .bind::<SqlUuid, _>(user)
        .get_result::<UuidRowNamed>(conn)
        .map(|r| r.id)
        .map_err(|e| e.to_string())?;
    } else {
        obj_id = sql_query("SELECT id FROM okr_objectives WHERE org_id = $1 AND title = 'Ship v1.0' LIMIT 1")
            .bind::<SqlUuid, _>(nil)
            .get_result::<UuidRowNamed>(conn)
            .map(|r| r.id)
            .map_err(|e| e.to_string())?;
    }

    for (title, current) in [("Complete all 12 milestones", 5), ("Reach 95% test coverage", 72)] {
        let n = count(conn, "SELECT count(*) AS n FROM okr_key_results WHERE org_id::text = $1 AND objective_id::text = $2 AND title = $3", &["00000000-0000-0000-0000-000000000000", &obj_id.to_string(), title])?;
        if n == 0 {
            sql_query(
                "INSERT INTO okr_key_results (id, org_id, bot_id, objective_id, owner_id, title, metric_type, start_value, target_value, current_value, unit, status, due_date, created_at, updated_at, branch_id)
                 VALUES ($1, $2, $2, $3, $4, $5, 'count', 0, 100, $6, 'units', 'in_progress', '2026-09-30', NOW(), NOW(), $2)",
            )
                .bind::<SqlUuid, _>(Uuid::new_v4())
                .bind::<SqlUuid, _>(nil)
                .bind::<SqlUuid, _>(obj_id)
                .bind::<SqlUuid, _>(user)
                .bind::<Text, _>(title)
                .bind::<Integer, _>(current)
                .execute(conn)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn seed_workspaces(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let org = s.org_id;
    let bot = s.bot_id;
    let branch = s.branch_id;
    let user = s.user_id;

    let ws_id;
    let n = count(conn, "SELECT count(*) AS n FROM aiworkspaces WHERE branch_id::text = $1 AND name = 'Product Workspace'", &[&s.branch_str])?;
    if n == 0 {
        ws_id = sql_query(
            "INSERT INTO aiworkspaces (id, org_id, bot_id, name, description, icon_type, icon_value, settings, created_by, created_at, updated_at, branch_id)
             VALUES ($1, $2, $3, 'Product Workspace', 'Shared product documentation', 'emoji', '🚀', '{}', $4, NOW(), NOW(), $5) RETURNING id",
        )
        .bind::<SqlUuid, _>(Uuid::new_v4())
        .bind::<SqlUuid, _>(org)
        .bind::<SqlUuid, _>(bot)
        .bind::<SqlUuid, _>(user)
        .bind::<SqlUuid, _>(branch)
        .get_result::<UuidRowNamed>(conn)
        .map(|r| r.id)
        .map_err(|e| e.to_string())?;
    } else {
        ws_id = sql_query("SELECT id FROM aiworkspaces WHERE branch_id = $1 AND name = 'Product Workspace' LIMIT 1")
            .bind::<SqlUuid, _>(branch)
            .get_result::<UuidRowNamed>(conn)
            .map(|r| r.id)
            .map_err(|e| e.to_string())?;
    }

    for (title, icon) in [("Home", "🏠"), ("Roadmap", "🗺️")] {
        let n = count(conn, "SELECT count(*) AS n FROM aiworkspace_pages WHERE workspace_id::text = $1 AND title = $2", &[&ws_id.to_string(), title])?;
        if n == 0 {
            sql_query(
                "INSERT INTO aiworkspace_pages (id, workspace_id, parent_id, title, icon_type, icon_value, content, properties, is_template, is_public, public_edit, position, created_by, last_edited_by, created_at, updated_at)
                 VALUES ($1, $2, NULL, $3, 'emoji', $4, '[]', '{}', false, false, false, 0, $5, $5, NOW(), NOW())",
            )
                .bind::<SqlUuid, _>(Uuid::new_v4())
                .bind::<SqlUuid, _>(ws_id)
                .bind::<Text, _>(title)
                .bind::<Text, _>(icon)
                .bind::<SqlUuid, _>(user)
                .execute(conn)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn seed_social(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let nil = Uuid::nil();
    let user = s.user_id;

    let community_id;
    let n = count(conn, "SELECT count(*) AS n FROM social_communities WHERE org_id::text = $1 AND slug = 'company-announcements'", &["00000000-0000-0000-0000-000000000000"])?;
    if n == 0 {
        community_id = sql_query(
            "INSERT INTO social_communities (id, org_id, bot_id, name, slug, description, visibility, join_policy, owner_id, member_count, post_count, is_official, is_featured, settings, created_at, updated_at, branch_id)
             VALUES ($1, $2, $2, 'Company Announcements', 'company-announcements', 'Internal announcements', 'public', 'open', $3, 3, 2, true, false, '{}', NOW(), NOW(), $2) RETURNING id",
        )
        .bind::<SqlUuid, _>(Uuid::new_v4())
        .bind::<SqlUuid, _>(nil)
        .bind::<SqlUuid, _>(user)
        .get_result::<UuidRowNamed>(conn)
        .map(|r| r.id)
        .map_err(|e| e.to_string())?;
    } else {
        community_id = sql_query("SELECT id FROM social_communities WHERE org_id = $1 AND slug = 'company-announcements' LIMIT 1")
            .bind::<SqlUuid, _>(nil)
            .get_result::<UuidRowNamed>(conn)
            .map(|r| r.id)
            .map_err(|e| e.to_string())?;
    }

    for content in ["Welcome to the new platform! 🎉", "v1.0 is shipping this quarter"] {
        let n = count(conn, "SELECT count(*) AS n FROM social_posts WHERE org_id::text = $1 AND content = $2", &["00000000-0000-0000-0000-000000000000", content])?;
        if n == 0 {
            sql_query(
                "INSERT INTO social_posts (id, org_id, bot_id, author_id, community_id, content, content_type, hashtags, visibility, comment_count, reaction_counts, created_at, branch_id)
                 VALUES ($1, $2, $2, $3, $4, $5, 'text', ARRAY['demo'], 'public', 1, '{\"like\":1}', NOW(), $2)",
            )
                .bind::<SqlUuid, _>(Uuid::new_v4())
                .bind::<SqlUuid, _>(nil)
                .bind::<SqlUuid, _>(user)
                .bind::<Nullable<SqlUuid>, _>(Some(community_id))
                .bind::<Text, _>(content)
                .execute(conn)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn seed_marketing(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let nil = Uuid::nil();

    let campaigns: &[(&str, &str)] = &[("Q3 Launch Campaign", "email"), ("WhatsApp Promo", "whatsapp")];
    for (name, ctype) in campaigns {
        let n = count(conn, "SELECT count(*) AS n FROM marketing_campaigns WHERE branch_id::text = $1 AND name = $2", &["00000000-0000-0000-0000-000000000000", name])?;
        if n == 0 {
            sql_query(
                "INSERT INTO marketing_campaigns (id, branch_id, name, campaign_type, status, starts_at, ends_at, budget, metrics, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, 'draft', NOW(), NOW() + INTERVAL '14 days', 500, '{}', NOW(), NOW())",
            )
                .bind::<SqlUuid, _>(Uuid::new_v4())
                .bind::<SqlUuid, _>(nil)
                .bind::<Text, _>(name)
                .bind::<Text, _>(ctype)
                .execute(conn)
                .map_err(|e| e.to_string())?;
        }
    }

    for name in ["All Active Contacts", "Warm Leads"] {
        let n = count(conn, "SELECT count(*) AS n FROM marketing_lists WHERE branch_id::text = $1 AND name = $2", &["00000000-0000-0000-0000-000000000000", name])?;
        if n == 0 {
            sql_query(
                "INSERT INTO marketing_lists (id, branch_id, name, list_type, description, query_text, member_count, contact_count, is_dynamic, criteria, created_at, updated_at)
                 VALUES ($1, $2, $3, 'static', 'Sample list', NULL, 0, 0, false, '{}', NOW(), NOW())",
            )
                .bind::<SqlUuid, _>(Uuid::new_v4())
                .bind::<SqlUuid, _>(nil)
                .bind::<Text, _>(name)
                .execute(conn)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn seed_m365(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let bot = s.bot_id;
    let org = s.org_id;

    for (site, list, title) in [("site-engineering", "list-docs", "Engineering Docs"), ("site-marketing", "list-assets", "Marketing Assets")] {
        let n = count(conn, "SELECT count(*) AS n FROM m365_sharepoint_items WHERE site_id = $1 AND list_id::text = $2", &[site, list])?;
        if n == 0 {
            sql_query(
                "INSERT INTO m365_sharepoint_items (id, bot_id, organization_id, site_id, list_id, item_id, title, fields, author, modified_at, synced_at)
                 VALUES ($1, $2, $3, $4, $5, 'item-1', $6, '{\"doc_count\": 10}', 'Sample User', NOW(), NOW())",
            )
                .bind::<SqlUuid, _>(Uuid::new_v4())
                .bind::<SqlUuid, _>(bot)
                .bind::<SqlUuid, _>(org)
                .bind::<Text, _>(site)
                .bind::<Text, _>(list)
                .bind::<Text, _>(title)
                .execute(conn)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn seed_drive(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let branch = s.branch_id;
    let user = s.user_id;

    for (path, ftype, name, mime) in [
        ("/reports/q3-summary.pdf", "pdf", "q3-summary.pdf", "application/pdf"),
        ("/exports/customers.csv", "csv", "customers.csv", "text/csv"),
    ] {
        let n = count(conn, "SELECT count(*) AS n FROM drive_files WHERE branch_id::text = $1 AND user_id::text = $2 AND file_path = $3", &[&s.branch_str, &s.user_str, path])?;
        if n == 0 {
            sql_query(
                "INSERT INTO drive_files (id, file_path, file_type, last_modified, file_size, indexed, user_id, scope, branch_id, path, name, mime_type, created_at, updated_at)
                 VALUES ($1, $2, $3, NOW(), 245760, true, $4, 'user', $5, $6, $7, $8, NOW(), NOW())",
            )
                .bind::<SqlUuid, _>(Uuid::new_v4())
                .bind::<Text, _>(path)
                .bind::<Text, _>(ftype)
                .bind::<SqlUuid, _>(user)
                .bind::<SqlUuid, _>(branch)
                .bind::<Text, _>(path.rsplit('/').next().map(|p| format!("/{}", p)).unwrap_or_default())
                .bind::<Text, _>(name)
                .bind::<Text, _>(mime)
                .execute(conn)
                .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}
