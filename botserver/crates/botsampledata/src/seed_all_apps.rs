//! Demo-data seeding for the remaining suite apps.
//!
//! The base `db.rs` seeds people, CRM, tickets, billing, tasks, calendar,
//! research, compliance, goals, workspaces, social, marketing, o365 and drive.
//! This module extends coverage to the rest of the harvested app surface so the
//! chat command palette and `api.exec` return real data for every application:
//! products, analytics/dashboards, meet + minutes, learn, project, canvas,
//! attendant, plan/OKRs, database, integrations, sources, monitoring.
//!
//! Two kinds of tables are seeded:
//!   * Central-schema tables (products, meeting_rooms, canvases, ...) that are
//!     always present — seeded unconditionally.
//!   * Crate-owned tables (sales_deals, hr_employees, brazil_nfe, ...) that are
//!     created lazily when their owning crate initializes — seeded only when the
//!     table exists (guarded by `to_regclass`), so a missing table never aborts
//!     the rest of the demo data.
//!
//! Every statement is idempotent (guarded by a count check), matching the
//! existing `botsampledata` contract, and carries the NOT-NULL columns the real
//! schema requires (`org_id`, `bot_id`, `branch_id` as applicable).

use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Integer, Nullable, Text, Uuid as SqlUuid};
use uuid::Uuid;

use super::db::Scopes;

/// Count helper reusing the parent module's semantics.
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
        4 => sql_query(sql)
            .bind::<Text, _>(binds[0])
            .bind::<Text, _>(binds[1])
            .bind::<Text, _>(binds[2])
            .bind::<Text, _>(binds[3])
            .get_result::<C>(conn),
        _ => return Err("count: too many binds".to_string()),
    };
    r.map(|r| r.n).map_err(|e| e.to_string())
}

/// Whether a table exists in the public schema (prevents seeding crate-owned
/// tables that have not been created yet).
fn table_exists(conn: &mut diesel::PgConnection, name: &str) -> bool {
    #[derive(diesel::QueryableByName)]
    struct Exists {
        #[diesel(sql_type = diesel::sql_types::Bool)]
        exists: bool,
    }
    let r: Result<Exists, _> = sql_query("SELECT to_regclass($1) IS NOT NULL AS exists")
        .bind::<Text, _>(format!("public.{name}"))
        .get_result(conn);
    r.map(|r| r.exists).unwrap_or(false)
}

/// Seeds every additional app. Failures are logged and isolated per domain.
pub fn seed(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let domains: Vec<(&str, fn(&mut diesel::PgConnection, &Scopes) -> Result<(), String>)> = vec![
        ("products", seed_products),
        ("dashboards", seed_dashboards),
        ("meet", seed_meet),
        ("learn", seed_learn),
        ("project", seed_project),
        ("canvas", seed_canvas),
        ("attendant", seed_attendant),
        ("okr", seed_okr),
        ("database", seed_database),
        ("sales", seed_sales),
        ("hr", seed_hr),
        ("tax", seed_tax),
        ("kyc", seed_kyc),
        ("fraud", seed_fraud),
    ];

    for (name, seed_fn) in domains {
        match seed_fn(conn, s) {
            Ok(()) => log::info!("botsampledata: {name} seeded"),
            Err(e) => log::error!("botsampledata: {name} seeding failed: {e}"),
        }
    }
    Ok(())
}

fn seed_products(conn: &mut diesel::PgConnection, _s: &Scopes) -> Result<(), String> {
    // Products handler resolves branch = nil in SaaS/admin mode; seed the nil
    // scope (global catalog). Live schema requires price/currency/tax_rate/unit.
    let scope_branch = Uuid::nil();
    let scope_str = scope_branch.to_string();
    let catalog: &[(&str, &str, &str, &str, f64, i32)] = &[
        ("hosting-basic", "Hosting Basic", "Shared web hosting plan", "plan", 9.99, 100),
        ("hosting-pro", "Hosting Pro", "VPS hosting with dedicated resources", "infrastructure", 19.99, 50),
        ("ssl-cert", "SSL Certificate", "Annual SSL certificate", "communication", 21.99, 200),
        ("support-hour", "Support Hour", "Specialized support hour", "service", 49.90, -1),
    ];
    for (sku, name, desc, ptype, price, stock) in catalog {
        let n = count(
            conn,
            "SELECT count(*) AS n FROM products WHERE branch_id::text = $1 AND sku = $2",
            &[&scope_str, sku],
        )?;
        if n == 0 {
            sql_query(
                "INSERT INTO products (id, branch_id, sku, name, description, category, product_type, price, currency, tax_rate, unit, stock_quantity, low_stock_threshold, is_active, images, attributes, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'BRL', 16.33, 'unit', $9, 10, true, '{}'::jsonb, '{}'::jsonb, NOW(), NOW())",
            )
            .bind::<SqlUuid, _>(Uuid::new_v4())
            .bind::<SqlUuid, _>(scope_branch)
            .bind::<Text, _>(sku)
            .bind::<Text, _>(name)
            .bind::<Text, _>(desc)
            .bind::<Text, _>(ptype)
            .bind::<Text, _>(ptype)
            .bind::<diesel::sql_types::Double, _>(price)
            .bind::<Integer, _>(*stock)
            .execute(conn)
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn seed_dashboards(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let branch = s.branch_id;
    let user = s.user_id;
    let n = count(conn, "SELECT count(*) AS n FROM dashboards WHERE branch_id::text = $1 AND name = 'Executive Overview'", &[&s.branch_str])?;
    if n == 0 {
        sql_query(
            "INSERT INTO dashboards (id, org_id, bot_id, branch_id, owner_id, name, description, layout, refresh_interval, is_public, is_template, tags, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, 'Executive Overview', 'Company KPIs', '{}'::jsonb, 300, true, false, ARRAY['kpi','finance'], NOW(), NOW())",
        )
        .bind::<SqlUuid, _>(Uuid::new_v4())
        .bind::<SqlUuid, _>(s.org_id)
        .bind::<SqlUuid, _>(s.bot_id)
        .bind::<SqlUuid, _>(branch)
        .bind::<SqlUuid, _>(user)
        .execute(conn)
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn seed_meet(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let branch = s.branch_id;
    let user = s.user_id;
    let n = count(conn, "SELECT count(*) AS n FROM meeting_rooms WHERE branch_id::text = $1 AND name = 'Weekly Sync'", &[&s.branch_str])?;
    if n == 0 {
        let room = Uuid::new_v4();
        sql_query(
            "INSERT INTO meeting_rooms (id, org_id, bot_id, branch_id, room_code, name, description, created_by, max_participants, is_recording, is_transcribing, status, settings, created_at, updated_at)
             VALUES ($1, $2, $3, $4, 'WEEKLY', 'Weekly Sync', 'Team standup', $5, 12, true, true, 'active', '{}'::jsonb, NOW(), NOW())",
        )
        .bind::<SqlUuid, _>(room)
        .bind::<SqlUuid, _>(s.org_id)
        .bind::<SqlUuid, _>(s.bot_id)
        .bind::<SqlUuid, _>(branch)
        .bind::<SqlUuid, _>(user)
        .execute(conn)
        .map_err(|e| e.to_string())?;

        let rec_n = count(conn, "SELECT count(*) AS n FROM meeting_recordings WHERE room_id::text = $1", &[&room.to_string()])?;
        if rec_n == 0 {
            sql_query(
                "INSERT INTO meeting_recordings (id, room_id, org_id, bot_id, branch_id, recording_type, file_url, file_size, duration_seconds, status, started_at, metadata, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, 'video', 'drive://meet/weekly.mp4', 52428800, 1800, 'completed', NOW() - INTERVAL '1 day', '{}'::jsonb, NOW(), NOW())",
            )
            .bind::<SqlUuid, _>(Uuid::new_v4())
            .bind::<SqlUuid, _>(room)
            .bind::<SqlUuid, _>(s.org_id)
            .bind::<SqlUuid, _>(s.bot_id)
            .bind::<SqlUuid, _>(branch)
            .execute(conn)
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn seed_learn(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let n = count(conn, "SELECT count(*) AS n FROM learn_courses WHERE title = 'Onboarding Fundamentals'", &[])?;
    if n == 0 {
        sql_query(
            "INSERT INTO learn_courses (id, organization_id, title, description, category, difficulty, duration_minutes, is_mandatory, due_days, is_published, created_by, created_at, updated_at)
             VALUES ($1, $2, 'Onboarding Fundamentals', 'Company onboarding course', 'Onboarding', 'beginner', 45, true, 14, true, $3, NOW(), NOW())",
        )
        .bind::<SqlUuid, _>(Uuid::new_v4())
        .bind::<Nullable<SqlUuid>, _>(Some(s.org_id))
        .bind::<Nullable<SqlUuid>, _>(Some(s.user_id))
        .execute(conn)
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn seed_project(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let n = count(conn, "SELECT count(*) AS n FROM projects WHERE name = 'Website Redesign'", &[])?;
    if n == 0 {
        let project_id = Uuid::new_v4();
        // Live schema: `projects.org` is varchar; project_type/deploy_target/
        // environment are NOT NULL.
        sql_query(
            "INSERT INTO projects (id, org, name, project_type, deploy_target, environment, status, description, created_at, updated_at)
             VALUES ($1, $2, 'Website Redesign', 'web', 'production', 'dev', 'active', 'Refresh the corporate website', NOW(), NOW())",
        )
        .bind::<SqlUuid, _>(project_id)
        .bind::<Text, _>(&s.org_str)
        .execute(conn)
        .map_err(|e| e.to_string())?;

        let t = count(conn, "SELECT count(*) AS n FROM project_tasks WHERE project_id::text = $1", &[&project_id.to_string()])?;
        if t == 0 {
            for (idx, name) in ["Design mockups", "Content migration", "QA pass"].iter().enumerate() {
                sql_query(
                    "INSERT INTO project_tasks (id, project_id, name, task_type, start_date, end_date, duration_days, percent_complete, status, priority, assigned_to, wbs, created_at, updated_at)
                     VALUES ($1, $2, $3, 'milestone', CURRENT_DATE, CURRENT_DATE + INTERVAL '7 days', 7, 0, 'not_started', 'medium', ARRAY[]::uuid[], $4, NOW(), NOW())",
                )
                .bind::<SqlUuid, _>(Uuid::new_v4())
                .bind::<SqlUuid, _>(project_id)
                .bind::<Text, _>(name)
                .bind::<Text, _>(&format!("1.{}", idx + 1))
                .execute(conn)
                .map_err(|e| e.to_string())?;
            }
        }
    }
    Ok(())
}

fn seed_canvas(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let branch = s.branch_id;
    let user = s.user_id;
    let n = count(conn, "SELECT count(*) AS n FROM canvases WHERE branch_id::text = $1 AND name = 'Product Roadmap Board'", &[&s.branch_str])?;
    if n == 0 {
        sql_query(
            "INSERT INTO canvases (id, org_id, bot_id, branch_id, name, description, width, height, is_public, is_template, created_by, created_at, updated_at)
             VALUES ($1, $2, $3, $4, 'Product Roadmap Board', 'Planning whiteboard', 1920, 1080, true, false, $5, NOW(), NOW())",
        )
        .bind::<SqlUuid, _>(Uuid::new_v4())
        .bind::<SqlUuid, _>(s.org_id)
        .bind::<SqlUuid, _>(s.bot_id)
        .bind::<SqlUuid, _>(branch)
        .bind::<SqlUuid, _>(user)
        .execute(conn)
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn seed_attendant(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let branch = s.branch_id;
    let n = count(conn, "SELECT count(*) AS n FROM attendant_queues WHERE branch_id::text = $1 AND name = 'Support'", &[&s.branch_str])?;
    if n == 0 {
        sql_query(
            "INSERT INTO attendant_queues (id, org_id, bot_id, branch_id, name, description, priority, max_wait_minutes, auto_assign, working_hours, is_active, created_at, updated_at)
             VALUES ($1, $2, $3, $4, 'Support', 'Customer support queue', 1, 10, true, '{}'::jsonb, true, NOW(), NOW())",
        )
        .bind::<SqlUuid, _>(Uuid::new_v4())
        .bind::<SqlUuid, _>(s.org_id)
        .bind::<SqlUuid, _>(s.bot_id)
        .bind::<SqlUuid, _>(branch)
        .execute(conn)
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn seed_okr(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let branch = s.branch_id;
    let user = s.user_id;
    let n = count(conn, "SELECT count(*) AS n FROM okr_objectives WHERE branch_id::text = $1 AND title = 'Grow revenue 20%'", &[&s.branch_str])?;
    if n == 0 {
        sql_query(
            "INSERT INTO okr_objectives (id, org_id, bot_id, branch_id, owner_id, title, description, period, period_start, period_end, status, progress, visibility, weight, tags, created_at, updated_at)
             VALUES ($1, $2, $3, $4, $5, 'Grow revenue 20%', 'Increase quarterly revenue', 'Q3', CURRENT_DATE, CURRENT_DATE + INTERVAL '90 days', 'on_track', 0, 'company', 1.0, ARRAY['growth'], NOW(), NOW())",
        )
        .bind::<SqlUuid, _>(Uuid::new_v4())
        .bind::<SqlUuid, _>(s.org_id)
        .bind::<SqlUuid, _>(s.bot_id)
        .bind::<SqlUuid, _>(branch)
        .bind::<SqlUuid, _>(user)
        .execute(conn)
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn seed_database(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    let user = s.user_id;
    let n = count(conn, "SELECT count(*) AS n FROM database_saved_queries WHERE name = 'Recent customers'", &[])?;
    if n == 0 {
        sql_query(
            "INSERT INTO database_saved_queries (id, bot_id, branch_id, user_id, name, query_text, description, is_shared, created_at, updated_at)
             VALUES ($1, $2, $3, $4, 'Recent customers', 'SELECT * FROM crm_contacts ORDER BY created_at DESC LIMIT 10;', 'Most recently added contacts', true, NOW(), NOW())",
        )
        .bind::<SqlUuid, _>(Uuid::new_v4())
        .bind::<SqlUuid, _>(s.bot_id)
        .bind::<SqlUuid, _>(s.branch_id)
        .bind::<SqlUuid, _>(user)
        .execute(conn)
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn seed_sales(conn: &mut diesel::PgConnection, _s: &Scopes) -> Result<(), String> {
    if !table_exists(conn, "sales_deals") {
        log::info!("botsampledata: sales_deals table not yet created; skipping sales seed");
        return Ok(());
    }
    let deals: &[(&str, f64, &str, f64)] = &[
        ("Acme Corporation expansion", 25000.0, "proposal", 0.6),
        ("Beta Inc onboarding", 12000.0, "negotiation", 0.8),
        ("Gamma renewal", 8000.0, "closed_won", 1.0),
    ];
    for (title, value, stage, prob) in deals {
        let n = count(conn, "SELECT count(*) AS n FROM sales_deals WHERE title = $1", &[title])?;
        if n == 0 {
            sql_query(
                "INSERT INTO sales_deals (id, title, value, stage, status, probability, created_at)
                 VALUES ($1, $2, $3, $4, 'open', $5, NOW())",
            )
            .bind::<SqlUuid, _>(Uuid::new_v4())
            .bind::<Text, _>(title)
            .bind::<diesel::sql_types::Double, _>(value)
            .bind::<Text, _>(stage)
            .bind::<diesel::sql_types::Double, _>(prob)
            .execute(conn)
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn seed_hr(conn: &mut diesel::PgConnection, _s: &Scopes) -> Result<(), String> {
    if !table_exists(conn, "hr_employees") {
        log::info!("botsampledata: hr_employees table not yet created; skipping hr seed");
        return Ok(());
    }
    let n = count(conn, "SELECT count(*) AS n FROM hr_employees WHERE email = 'joana.hr@example.com'", &[])?;
    if n == 0 {
        sql_query(
            "INSERT INTO hr_employees (id, name, email, department, role, status, hired_at)
             VALUES ($1, 'Joana Lima', 'joana.hr@example.com', 'People', 'HR Analyst', 'active', NOW() - INTERVAL '200 days')",
        )
        .bind::<SqlUuid, _>(Uuid::new_v4())
        .execute(conn)
        .map_err(|e| e.to_string())?;
    }
    if table_exists(conn, "hr_recruitment") {
        let n2 = count(conn, "SELECT count(*) AS n FROM hr_recruitment WHERE position = 'Backend Engineer'", &[])?;
        if n2 == 0 {
            sql_query(
                "INSERT INTO hr_recruitment (id, position, department, status, candidates, opened_at)
                 VALUES ($1, 'Backend Engineer', 'Engineering', 'open', 4, NOW())",
            )
            .bind::<SqlUuid, _>(Uuid::new_v4())
            .execute(conn)
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn seed_tax(conn: &mut diesel::PgConnection, _s: &Scopes) -> Result<(), String> {
    if table_exists(conn, "brazil_nfe") {
        let n = count(conn, "SELECT count(*) AS n FROM brazil_nfe WHERE number = 'NFe-2026-0001'", &[])?;
        if n == 0 {
            sql_query(
                "INSERT INTO brazil_nfe (id, number, series, emitter_cnpj, recipient_cnpj, total, status, created_at)
                 VALUES ($1, 'NFe-2026-0001', '1', '00000000000191', '00000000000283', 1250.00, 'authorized', NOW() - INTERVAL '5 days')",
            )
            .bind::<SqlUuid, _>(Uuid::new_v4())
            .execute(conn)
            .map_err(|e| e.to_string())?;
        }
    }
    if table_exists(conn, "brazil_nfse") {
        let n2 = count(conn, "SELECT count(*) AS n FROM brazil_nfse WHERE number = 'NFS-e-2026-001'", &[])?;
        if n2 == 0 {
            sql_query(
                "INSERT INTO brazil_nfse (id, number, service_code, provider_cnpj, total, status, created_at)
                 VALUES ($1, 'NFS-e-2026-001', '0101', '00000000000191', 480.00, 'issued', NOW() - INTERVAL '3 days')",
            )
            .bind::<SqlUuid, _>(Uuid::new_v4())
            .execute(conn)
            .map_err(|e| e.to_string())?;
        }
    }
    Ok(())
}

fn seed_kyc(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    // identity_kyc_workflows has a NOT NULL profile_id FK -> identity_profiles.
    let mut profile_id = Uuid::new_v4();
    let profile_n = count(conn, "SELECT count(*) AS n FROM identity_profiles WHERE legal_name = 'Carlos Demo'", &[])?;
    if profile_n == 0 {
        sql_query(
            "INSERT INTO identity_profiles (id, bot_id, person_id, branch_id, legal_name, tax_id, kyc_status, created_at, updated_at)
             VALUES ($1, $2, $3, $4, 'Carlos Demo', '000.000.000-00', 'pending', NOW(), NOW())",
        )
        .bind::<SqlUuid, _>(profile_id)
        .bind::<SqlUuid, _>(s.bot_id)
        .bind::<SqlUuid, _>(s.user_id)
        .bind::<SqlUuid, _>(s.branch_id)
        .execute(conn)
        .map_err(|e| e.to_string())?;
    } else {
        // Reuse an existing profile id.
        #[derive(diesel::QueryableByName)]
        struct P {
            #[diesel(sql_type = diesel::sql_types::Uuid)]
            id: Uuid,
        }
        let existing: Option<P> = sql_query(
            "SELECT id FROM identity_profiles WHERE legal_name = 'Carlos Demo' LIMIT 1",
        )
        .get_result(conn)
        .ok();
        if let Some(p) = existing {
            profile_id = p.id;
        }
    }

    let n = count(conn, "SELECT count(*) AS n FROM identity_kyc_workflows WHERE workflow_name = 'Standard Onboarding'", &[])?;
    if n == 0 {
        sql_query(
            "INSERT INTO identity_kyc_workflows (id, bot_id, profile_id, branch_id, workflow_name, current_step, steps_completed, total_steps, status, started_at)
             VALUES ($1, $2, $3, $4, 'Standard Onboarding', 'document_upload', '[]'::jsonb, 4, 'in_progress', NOW() - INTERVAL '1 day')",
        )
        .bind::<SqlUuid, _>(Uuid::new_v4())
        .bind::<SqlUuid, _>(s.bot_id)
        .bind::<SqlUuid, _>(profile_id)
        .bind::<SqlUuid, _>(s.branch_id)
        .execute(conn)
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn seed_fraud(conn: &mut diesel::PgConnection, s: &Scopes) -> Result<(), String> {
    if !table_exists(conn, "fraud_transactions") {
        log::info!("botsampledata: fraud_transactions table not yet; skipping fraud seed");
        return Ok(());
    }
    let n = count(conn, "SELECT count(*) AS n FROM fraud_transactions WHERE amount = 350.00", &[])?;
    if n == 0 {
        sql_query(
            "INSERT INTO fraud_transactions (id, user_id, amount, currency, status, risk_score, created_at)
             VALUES ($1, $2, 350.00, 'BRL', 'flagged', 82, NOW() - INTERVAL '2 hours')",
        )
        .bind::<SqlUuid, _>(Uuid::new_v4())
        .bind::<SqlUuid, _>(s.user_id)
        .execute(conn)
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}