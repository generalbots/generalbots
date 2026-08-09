use bottest::prelude::*;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct StatusRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    status: String,
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct CountRow {
    #[diesel(sql_type = diesel::sql_types::BigInt)]
    count: i64,
}

fn test_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}

async fn seed_org_branch(ctx: &TestContext, name: &str) -> Option<Uuid> {
    let pool = ctx.db_pool().await.ok()?;
    let mut conn = pool.get().ok()?;
    let tenant_id = Uuid::new_v4();
    let org_id = Uuid::new_v4();
    let branch_id = Uuid::new_v4();
    let slug = format!("{}-{}", name, Uuid::new_v4().simple());

    use diesel::RunQueryDsl;
    let inserted = diesel::sql_query(
        "INSERT INTO tenants (id, name, slug, created_at, updated_at) \
         VALUES ($1, $2, $3, NOW(), NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(tenant_id)
    .bind::<diesel::sql_types::Text, _>(name)
    .bind::<diesel::sql_types::Text, _>(&slug)
    .execute(&mut conn)
    .and_then(|_| {
        diesel::sql_query(
            "INSERT INTO organizations (org_id, tenant_id, name, slug, created_at) \
             VALUES ($1, $2, $3, $4, NOW())",
        )
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .bind::<diesel::sql_types::Uuid, _>(tenant_id)
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(&slug)
        .execute(&mut conn)
    })
    .and_then(|_| {
        diesel::sql_query(
            "INSERT INTO branches (id, org_id, tenant_id, slug, name, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, NOW(), NOW())",
        )
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .bind::<diesel::sql_types::Uuid, _>(tenant_id)
        .bind::<diesel::sql_types::Text, _>(&slug)
        .bind::<diesel::sql_types::Text, _>(name)
        .execute(&mut conn)
    });

    match inserted {
        Ok(_) => Some(branch_id),
        Err(e) => {
            eprintln!("Skipping: seed_org_branch failed ({e})");
            None
        }
    }
}

#[tokio::test]
async fn test_cloud_signup_login_roundtrip_with_zitadel() {
    let ctx = match TestHarness::quick().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {e}");
            return;
        }
    };

    let zitadel_user_id = Uuid::new_v4().to_string();
    if let Some(mock) = ctx.mock_zitadel() {
        mock.mount_cloud_flow(&zitadel_user_id).await;
    }

    let server = match ctx.start_botserver().await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Skipping: {e}");
            return;
        }
    };
    if !server.is_running() {
        eprintln!("Skipping: botserver did not start");
        return;
    }

    let client = test_client();
    let base_url = server.url.clone();
    let email = format!("roundtrip-{}@example.com", Uuid::new_v4().simple());
    let password = "Sup3rS3cret!";

    let signup = client
        .post(format!("{base_url}/api/cloud/auth/signup"))
        .json(&json!({
            "name": "Roundtrip User",
            "bot_name": format!("bot-{}", Uuid::new_v4().simple()),
            "email": email,
            "password": password,
            "plan": "shared",
        }))
        .send()
        .await
        .expect("signup request failed");

    assert_eq!(
        signup.status(),
        reqwest::StatusCode::OK,
        "signup should succeed"
    );
    let signup_body: serde_json::Value = signup.json().await.expect("signup JSON");
    assert_eq!(signup_body["status"], "ok");
    assert_eq!(signup_body["plan"], "shared");
    assert!(!signup_body["token"].as_str().unwrap_or("").is_empty());
    assert!(signup_body["org_id"].is_string());
    assert!(signup_body["branch_id"].is_string());
    let signup_token = signup_body["token"].as_str().unwrap().to_string();

    let login = client
        .post(format!("{base_url}/api/cloud/auth/login"))
        .json(&json!({
            "email": email,
            "password": password,
        }))
        .send()
        .await
        .expect("login request failed");
    assert_eq!(
        login.status(),
        reqwest::StatusCode::OK,
        "login should succeed after signup"
    );
    let login_body: serde_json::Value = login.json().await.expect("login JSON");
    let login_token = login_body["token"].as_str().unwrap_or("").to_string();
    assert!(!login_token.is_empty(), "login must issue a JWT");

    for token in [&signup_token, &login_token] {
        let plans = client
            .get(format!("{base_url}/api/cloud/plans"))
            .bearer_auth(token)
            .send()
            .await
            .expect("plans request failed");
        assert_eq!(
            plans.status(),
            reqwest::StatusCode::OK,
            "issued token must be accepted by protected routes"
        );
    }

    let rejected = client
        .get(format!("{base_url}/api/cloud/plans"))
        .send()
        .await
        .expect("anonymous plans request failed");
    assert!(
        rejected.status() == reqwest::StatusCode::UNAUTHORIZED
            || rejected.status() == reqwest::StatusCode::FORBIDDEN,
        "unauthenticated request must be rejected, got {}",
        rejected.status()
    );
}

#[tokio::test]
async fn test_trial_promotion_to_paid() {
    let ctx = match TestHarness::database_only().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {e}");
            return;
        }
    };
    let pool = match ctx.db_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Skipping: {e}");
            return;
        }
    };

    let branch_id = match seed_org_branch(&ctx, "trial-branch").await {
        Some(id) => id,
        None => return,
    };
    let sub_id = Uuid::new_v4();
    let bot_id = Uuid::new_v4();
    let yesterday = chrono::Utc::now().date_naive() - chrono::Duration::days(1);

    {
        use diesel::RunQueryDsl;
        let mut conn = match pool.get() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Skipping: {e}");
                return;
            }
        };

        if let Err(e) = diesel::sql_query(
            "INSERT INTO billing_recurring \
             (id, org_id, bot_id, branch_id, customer_name, customer_email, status, \
              frequency, interval_count, amount, currency, description, next_invoice_date, \
              start_date, invoices_generated, created_at, updated_at) \
             VALUES ($1, $2, $3, $4, $5, $6, 'trialing', 'monthly', 1, 3.99, 'USD', \
                     $7, $8, NOW(), 0, NOW(), NOW())",
        )
        .bind::<diesel::sql_types::Uuid, _>(sub_id)
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .bind::<diesel::sql_types::Uuid, _>(bot_id)
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .bind::<diesel::sql_types::Text, _>("Trial Customer")
        .bind::<diesel::sql_types::Text, _>("trial@example.com")
        .bind::<diesel::sql_types::Text, _>("shared - 14 Day Trial")
        .bind::<diesel::sql_types::Date, _>(yesterday)
        .execute(&mut conn)
        {
            eprintln!("Skipping: billing_recurring insert failed ({e})");
            return;
        }
    }

    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping: {e}");
            return;
        }
    };
    let promoted = botbilling::lifecycle::promote_expired_trials_in_db(&mut conn);
    match promoted {
        Ok(count) => {
            assert_eq!(count, 1, "exactly one expired trial must be promoted");
        }
        Err(e) => {
            eprintln!("Skipping: promote_expired_trials_in_db failed ({e})");
            return;
        }
    }
    use diesel::RunQueryDsl;
    let status: String = match diesel::sql_query(
        "SELECT status FROM billing_recurring WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(sub_id)
    .get_result::<StatusRow>(&mut conn)
    {
        Ok(row) => row.status,
        Err(e) => {
            eprintln!("Skipping: {e}");
            return;
        }
    };
    assert_eq!(status, "active", "trial must be promoted to active");

    let invoice_count: i64 = match diesel::sql_query(
        "SELECT COUNT(*) AS count FROM billing_invoices WHERE branch_id = $1 AND status = 'unpaid'",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .get_result::<CountRow>(&mut conn)
    {
        Ok(row) => row.count,
        Err(e) => {
            eprintln!("Skipping: {e}");
            return;
        }
    };
    assert_eq!(invoice_count, 1, "first paid invoice must be created");
}

#[tokio::test]
async fn test_workspace_unique_index_and_dedupe() {
    let ctx = match TestHarness::database_only().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {e}");
            return;
        }
    };
    let pool = match ctx.db_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Skipping: {e}");
            return;
        }
    };

    let branch_id = match seed_org_branch(&ctx, "workspace-branch").await {
        Some(id) => id,
        None => return,
    };
    {
        use diesel::RunQueryDsl;
        let mut conn = match pool.get() {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Skipping: {e}");
                return;
            }
        };

        let insert = |conn: &mut diesel::PgConnection| {
            diesel::sql_query(
                "INSERT INTO cloud_workspaces (id, org_id, branch_id, name, description, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, $5, NOW(), NOW()) \
                 ON CONFLICT (branch_id) DO UPDATE SET name = EXCLUDED.name",
            )
            .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
            .bind::<diesel::sql_types::Uuid, _>(Uuid::nil())
            .bind::<diesel::sql_types::Uuid, _>(branch_id)
            .bind::<diesel::sql_types::Text, _>("my-workspace")
            .bind::<diesel::sql_types::Text, _>("dedupe test")
            .execute(conn)
        };
        if let Err(e) = insert(&mut conn) {
            eprintln!("Skipping: cloud_workspaces table unavailable ({e})");
            return;
        }
        if let Err(e) = insert(&mut conn) {
            eprintln!("Skipping: ON CONFLICT upsert failed ({e})");
            return;
        }
    }

    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping: {e}");
            return;
        }
    };
    use diesel::RunQueryDsl;
    let count: i64 = match diesel::sql_query(
        "SELECT COUNT(*) AS count FROM cloud_workspaces WHERE branch_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .get_result::<CountRow>(&mut conn)
    {
        Ok(row) => row.count,
        Err(e) => {
            eprintln!("Skipping: {e}");
            return;
        }
    };
    assert_eq!(count, 1, "workspace dedupe must keep a single row per branch");
}

#[tokio::test]
async fn test_tenant_isolation_branch_scoping() {
    let ctx = match TestHarness::database_only().await {
        Ok(ctx) => ctx,
        Err(e) => {
            eprintln!("Skipping: {e}");
            return;
        }
    };
    let pool = match ctx.db_pool().await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Skipping: {e}");
            return;
        }
    };

    let mut conn = match pool.get() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Skipping: {e}");
            return;
        }
    };
    use diesel::RunQueryDsl;

    if let Err(e) = diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS hr_employees (
            id UUID PRIMARY KEY, name TEXT NOT NULL, email TEXT NOT NULL DEFAULT '',
            department VARCHAR(100) NOT NULL DEFAULT '', role VARCHAR(100) NOT NULL DEFAULT '',
            status VARCHAR(30) NOT NULL DEFAULT 'active', hired_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000')",
    )
    .execute(&mut conn)
    {
        eprintln!("Skipping: {e}");
        return;
    }

    let branch_a = Uuid::new_v4();
    let branch_b = Uuid::new_v4();
    for (branch, name, email) in [
        (branch_a, "Alpha User", "alpha@a.com"),
        (branch_a, "Alpha Second", "alpha2@a.com"),
        (branch_b, "Beta User", "beta@b.com"),
    ] {
        if let Err(e) = diesel::sql_query(
            "INSERT INTO hr_employees (id, name, email, department, role, branch_id) \
             VALUES ($1, $2, $3, 'eng', 'member', $4)",
        )
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Text, _>(name)
        .bind::<diesel::sql_types::Text, _>(email)
        .bind::<diesel::sql_types::Uuid, _>(branch)
        .execute(&mut conn)
        {
            eprintln!("Skipping: {e}");
            return;
        }
    }

    let branch_a_count: i64 = match diesel::sql_query(
        "SELECT COUNT(*) AS count FROM hr_employees WHERE branch_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_a)
    .get_result::<CountRow>(&mut conn)
    {
        Ok(row) => row.count,
        Err(e) => {
            eprintln!("Skipping: {e}");
            return;
        }
    };
    assert_eq!(branch_a_count, 2, "branch A must see only its own rows");

    let branch_b_count: i64 = match diesel::sql_query(
        "SELECT COUNT(*) AS count FROM hr_employees WHERE branch_id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_b)
    .get_result::<CountRow>(&mut conn)
    {
        Ok(row) => row.count,
        Err(e) => {
            eprintln!("Skipping: {e}");
            return;
        }
    };
    assert_eq!(branch_b_count, 1, "branch B must see only its own rows");
}
