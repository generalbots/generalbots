//! Tenant isolation regression tests (#1387/#1388/#1389, #1401, #1644/#1645).
//! Cross-tenant access is denied even for org admins; seeding failures skip.
//! #2138 covers CRM in `crm_tenant_isolation.rs`; #2547 pins cache keys.

use crate::tenant_tokens::{mint_admin_jwt, mint_forged_jwt};
use bottest::prelude::*;
use diesel::prelude::*;
use reqwest::{Client, Method, StatusCode};
use std::time::Duration;
use uuid::Uuid;

pub(super) fn test_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}

/// Starts/reuses a botserver; keep the returned `TestContext` alive.
pub(super) async fn get_test_server() -> Option<(Option<TestContext>, String)> {
    if let Ok(url) = std::env::var("BOTSERVER_URL") {
        let probe = Client::builder()
            .timeout(Duration::from_secs(2))
            .build()
            .ok()?;
        if probe.get(&url).send().await.is_ok() {
            return Some((None, url));
        }
    }
    let ctx = TestHarness::quick().await.ok()?;
    let server = ctx.start_botserver().await.ok()?;
    if server.is_running() {
        Some((Some(ctx), server.url.clone()))
    } else {
        None
    }
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct UserIdRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    id: Uuid,
}

#[derive(diesel::QueryableByName)]
#[diesel(check_for_backend(diesel::pg::Pg))]
struct OrgIdRow {
    #[diesel(sql_type = diesel::sql_types::Uuid)]
    org_id: Uuid,
}

/// One tenant: org (slug = bucket prefix) + branch + admin user bound to the
/// org via `user_organizations` and to the branch via `crm_contacts`, so the
/// entitlement lookup finds it through either path.
#[derive(Debug)]
pub(super) struct SeededTenant {
    pub(super) slug: String,
    pub(super) branch_id: Uuid,
}

fn db() -> Option<PgConnection> {
    PgConnection::establish(&std::env::var("DATABASE_URL").ok()?).ok()
}

pub(super) fn seed_tenant(name: &str) -> Option<SeededTenant> {
    let mut conn = db()?;
    let org_id = Uuid::new_v4();
    let branch_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let slug = format!("{}-{}", name, Uuid::new_v4().simple());
    let email = format!("admin-{slug}@tenant.test");

    // Schema drift guard: `users.name` was dropped (username is the column
    // now) and user_organizations/crm_contacts gained NOT NULL columns, so the
    // old fixture failed silently and every assertion below was skipped.
    diesel::sql_query(
        "INSERT INTO organizations (org_id, name, slug, created_at) VALUES ($1, $2, $3, NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(org_id)
    .bind::<diesel::sql_types::Text, _>(&slug)
    .bind::<diesel::sql_types::Text, _>(&slug)
    .execute(&mut conn)
    .ok()?;
    diesel::sql_query("INSERT INTO branches (id, org_id, slug, name, created_at, updated_at) VALUES ($1, $2, $3, $3, NOW(), NOW())")
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .bind::<diesel::sql_types::Text, _>(&slug)
        .execute(&mut conn).ok()?;
    diesel::sql_query("INSERT INTO users (id, username, email, password_hash, is_active, is_admin, created_at, updated_at) VALUES ($1, $2, $3, 'x', true, false, NOW(), NOW())")
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .bind::<diesel::sql_types::Text, _>(&slug)
        .bind::<diesel::sql_types::Text, _>(&email)
        .execute(&mut conn).ok()?;
    diesel::sql_query("INSERT INTO user_organizations (id, user_id, org_id, role, is_default, joined_at) VALUES ($1, $2, $3, 'admin', true, NOW())")
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .bind::<diesel::sql_types::Uuid, _>(org_id)
        .execute(&mut conn).ok()?;
    // `crm_contacts.org_id` is a legacy FK to `branches`, not to
    // `organizations`; the entitlement lookup reads branch_id.
    diesel::sql_query("INSERT INTO crm_contacts (id, org_id, branch_id, bot_id, status, first_name, last_name, email, created_at, updated_at) VALUES ($1, $2, $2, (SELECT id FROM bots ORDER BY created_at LIMIT 1), 'active', $3, 'Admin', $4, NOW(), NOW())")
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .bind::<diesel::sql_types::Uuid, _>(branch_id)
        .bind::<diesel::sql_types::Text, _>(&slug)
        .bind::<diesel::sql_types::Text, _>(&email)
        .execute(&mut conn).ok()?;

    Some(SeededTenant { slug, branch_id })
}

/// Resolves the seeded admin user's id (needed to attach RBAC groups).
fn seeded_admin_user_id(slug: &str) -> Option<Uuid> {
    let mut conn = db()?;
    diesel::sql_query("SELECT id FROM users WHERE email = $1 LIMIT 1")
        .bind::<diesel::sql_types::Text, _>(format!("admin-{slug}@tenant.test"))
        .get_result::<UserIdRow>(&mut conn)
        .ok()
        .map(|r| r.id)
}

/// Puts the seeded admin in a global RBAC group named "Administrators" — the
/// same way prod models ORG admins. The suite then asserts this does NOT
/// grant platform powers (#1387).
fn grant_global_admin_group(slug: &str) -> Option<()> {
    let mut conn = db()?;
    let user_id = seeded_admin_user_id(slug)?;
    let group_id: OrgIdRow = diesel::sql_query("INSERT INTO rbac_groups (id, name, display_name, is_active, created_at, updated_at) VALUES ($1, 'Administrators', 'Administrators', true, NOW(), NOW()) ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name RETURNING id AS org_id")
        .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
        .get_result(&mut conn).ok()?;
    diesel::sql_query("INSERT INTO rbac_user_groups (user_id, group_id, created_at) VALUES ($1, $2, NOW()) ON CONFLICT DO NOTHING")
        .bind::<diesel::sql_types::Uuid, _>(user_id)
        .bind::<diesel::sql_types::Uuid, _>(group_id.org_id)
        .execute(&mut conn).ok()?;
    Some(())
}

pub(super) fn org_id_of(slug: &str) -> Option<String> {
    let mut conn = db()?;
    diesel::sql_query("SELECT org_id FROM organizations WHERE slug = $1 LIMIT 1")
        .bind::<diesel::sql_types::Text, _>(slug)
        .get_result::<OrgIdRow>(&mut conn)
        .ok()
        .map(|r| r.org_id.to_string())
}

type Ctx = Option<TestContext>;
type One = (Ctx, String, Client, SeededTenant, String);
type Two = (
    Ctx,
    String,
    Client,
    SeededTenant,
    SeededTenant,
    String,
    String,
);

/// Seed one/two tenants with tokens; keep `TestContext` (first field) alive.
async fn one_tenant(name: &str) -> Option<One> {
    let (ctx, base_url) = get_test_server().await?;
    let tenant = seed_tenant(name).or_else(|| seed_tenant(name))?;
    let token = mint_admin_jwt(
        &tenant.slug,
        &org_id_of(&tenant.slug)?,
        &tenant.branch_id.to_string(),
    );
    Some((ctx, base_url, test_client(), tenant, token))
}

async fn two_tenants(name_a: &str, name_b: &str) -> Option<Two> {
    let (ctx, base_url) = get_test_server().await?;
    let a = seed_tenant(name_a).or_else(|| seed_tenant(name_a))?;
    let b = seed_tenant(name_b).or_else(|| seed_tenant(name_b))?;
    let token_a = mint_admin_jwt(&a.slug, &org_id_of(&a.slug)?, &a.branch_id.to_string());
    let token_b = mint_admin_jwt(&b.slug, &org_id_of(&b.slug)?, &b.branch_id.to_string());
    Some((ctx, base_url, test_client(), a, b, token_a, token_b))
}

async fn send_get(client: &Client, url: String, token: &str) -> reqwest::Response {
    client
        .get(url)
        .bearer_auth(token)
        .send()
        .await
        .expect("request failed")
}

async fn send_post(
    client: &Client,
    url: String,
    token: &str,
    body: serde_json::Value,
) -> reqwest::Response {
    client
        .post(url)
        .bearer_auth(token)
        .json(&body)
        .send()
        .await
        .expect("request failed")
}

#[tokio::test]
async fn test_drive_list_buckets_only_shows_own_tenant() {
    let Some((_ctx, base_url, client, a, b, token, ..)) = two_tenants("isola", "isolb").await
    else {
        eprintln!("Skipping: two-tenant setup unavailable");
        return;
    };

    let resp = send_get(&client, format!("{base_url}/api/files/buckets"), &token).await;
    if !resp.status().is_success() {
        eprintln!("Skipping: buckets endpoint unavailable ({})", resp.status());
        return;
    }
    let body: serde_json::Value = resp.json().await.expect("buckets JSON");
    let names: Vec<String> = body
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|b| b["name"].as_str().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default();

    assert!(
        !names.iter().any(|n| n.contains(&b.slug)),
        "bucket discovery leaked tenant B ({b:?}) to tenant A: {names:?}"
    );
    assert!(
        !names
            .iter()
            .any(|n| n.ends_with(".gborg") && !n.contains(&a.slug)),
        "bucket discovery listed a foreign .gborg workspace: {names:?}"
    );
}

#[tokio::test]
async fn test_drive_cross_tenant_bucket_denied() {
    let Some((_ctx, base_url, client, a, b, token, ..)) = two_tenants("isola", "isolb").await
    else {
        eprintln!("Skipping: two-tenant setup unavailable");
        return;
    };
    grant_global_admin_group(&a.slug); // org admin, NOT platform admin

    // THE #1401 regression: tenant A's (org-level) admin reads tenant B's
    // workspace bucket by naming it. Must be 403.
    let foreign = format!("{}.gborg", b.slug);
    let url = format!("{base_url}/api/files/list?bucket={foreign}&path=");
    let resp = send_get(&client, url, &token).await;

    assert_eq!(
        resp.status(),
        StatusCode::FORBIDDEN,
        "cross-tenant bucket read must be denied (got {}) — #1401 regression",
        resp.status()
    );
}

#[tokio::test]
async fn test_drive_own_tenant_bucket_allowed() {
    let Some((_ctx, base_url, client, a, token)) = one_tenant("isola").await else {
        eprintln!("Skipping: one-tenant setup unavailable");
        return;
    };

    let own = format!("{}.gborg", a.slug);
    let url = format!("{base_url}/api/files/list?bucket={own}&path=");
    let resp = send_get(&client, url, &token).await;

    assert!(
        resp.status().is_success(),
        "own tenant bucket must be accessible (got {})",
        resp.status()
    );
}

#[tokio::test]
async fn test_drive_spoofed_user_id_ignored() {
    let Some((_ctx, base_url, client, _, b, token, ..)) = two_tenants("isola", "isolb").await
    else {
        eprintln!("Skipping: two-tenant setup unavailable");
        return;
    };

    // Non-admin callers cannot impersonate another user's scope via user_id:
    // tenant A's token asks for tenant B admin's user id.
    let Some(foreign_uid) = seeded_admin_user_id(&b.slug).map(|u| u.to_string()) else {
        eprintln!("Skipping: tenant B admin lookup failed");
        return;
    };
    let url = format!("{base_url}/api/files/list?user_id={foreign_uid}&scope=user");
    let resp = send_get(&client, url, &token).await;

    assert_ne!(
        resp.status(),
        StatusCode::OK,
        "spoofed user_id must not expose tenant B's user scope"
    );
}

#[tokio::test]
async fn test_org_admin_is_not_platform_admin() {
    let Some((_ctx, base_url, client, a, b, token, ..)) = two_tenants("isola", "isolb").await
    else {
        eprintln!("Skipping: two-tenant setup unavailable");
        return;
    };
    // Org admin via the global RBAC group — must STILL not cross tenants.
    grant_global_admin_group(&a.slug);

    let foreign = format!("{}.gborg", b.slug);
    let url = format!("{base_url}/api/files/search?bucket={foreign}&query=x");
    let resp = send_get(&client, url, &token).await;

    assert!(
        resp.status() == StatusCode::FORBIDDEN || resp.status() == StatusCode::NOT_FOUND,
        "org admin must not search foreign buckets (got {})",
        resp.status()
    );
}

/// #2112 — a token whose signature does not verify must never resolve a
/// tenant, even when its payload names a real branch of the same org.
#[tokio::test]
async fn test_forged_token_cannot_read_a_tenant_branch() {
    let Some((_ctx, base_url, client, a, _)) = one_tenant("isola").await else {
        eprintln!("Skipping: one-tenant setup unavailable");
        return;
    };
    let Some(org_a) = org_id_of(&a.slug) else {
        eprintln!("Skipping: org lookup failed");
        return;
    };
    let forged = mint_forged_jwt(&a.slug, &org_a, &a.branch_id.to_string());

    let own = format!("{}.gborg", a.slug);
    let forged_url = format!("{base_url}/api/files/list?bucket={own}&path=");
    let resp = send_get(&client, forged_url, &forged).await;

    assert!(
        !resp.status().is_success(),
        "a forged JWT must not read a tenant branch (got {}) — #2112 regression",
        resp.status()
    );
}

/// #1644/#1645 — a computer-use approval is a server-side record owned by the
/// stable user id derived from the verified JWT `sub`. Two tenants have
/// distinct subjects, so tenant A must not read, approve or execute tenant B's
/// action. The owner keeps full access. Skips when the surface is unavailable.
#[tokio::test]
async fn test_computer_use_approvals_are_cross_tenant_scoped() {
    let Some((_ctx, base_url, client, _, _, token_a, token_b)) =
        two_tenants("isola", "isolb").await
    else {
        eprintln!("Skipping: two-tenant setup unavailable");
        return;
    };

    let created = send_post(
        &client,
        format!("{base_url}/api/computer/actions"),
        &token_b,
        json!({
            "session_id": "tenant-b-session",
            "kind": "browser",
            "action": { "action": "open_url", "url": "https://example.invalid" },
        }),
    )
    .await;
    if !created.status().is_success() {
        eprintln!("Skipping: computer-use endpoint unavailable");
        return;
    }
    let body: serde_json::Value = created.json().await.expect("action JSON");
    let action_id = body["action_id"].as_str().unwrap_or_default().to_string();
    if action_id.is_empty() {
        eprintln!("Skipping: action id missing from response");
        return;
    }

    let base = format!("{base_url}/api/computer/actions/{action_id}");
    // Tenant A must not see, approve or execute tenant B's pending action.
    for (method, suffix, body) in [
        (Method::GET, "", None),
        (
            Method::POST,
            "/approve",
            Some(json!({ "action_id": action_id })),
        ),
        (Method::POST, "/execute", Some(json!({ "approved": true }))),
    ] {
        let mut req = client
            .request(method, format!("{base}{suffix}"))
            .bearer_auth(&token_a);
        if let Some(json) = body {
            req = req.json(&json);
        }
        let resp = req.send().await.expect("cross-tenant request failed");
        assert_eq!(
            resp.status(),
            StatusCode::NOT_FOUND,
            "cross-tenant access must be denied"
        );
    }

    // The owner still reads and approves its own action.
    let own = send_get(&client, base.clone(), &token_b).await;
    assert_eq!(own.status(), StatusCode::OK, "owner must read its action");
    let approved = send_post(
        &client,
        format!("{base}/approve"),
        &token_b,
        json!({ "action_id": action_id }),
    )
    .await;
    assert_eq!(
        approved.status(),
        StatusCode::OK,
        "owner must approve its action"
    );
}

/// #2547 — two orgs running a bot with the SAME name must never share a
/// suggestion/context key; pre-field sessions keep the stable legacy key.
#[test]
fn test_cache_keys_do_not_leak_between_orgs_with_same_bot_name() {
    let bot = Uuid::from_u128(7);
    let session = Uuid::from_u128(8);
    let key = |org: Uuid, kind: &str| {
        botlib::key_utils::scoped_key(Some(org), &[kind, &bot.to_string(), &session.to_string()])
    };
    assert_ne!(
        key(Uuid::from_u128(1), "suggestions"),
        key(Uuid::from_u128(2), "suggestions")
    );
    assert_ne!(
        key(Uuid::from_u128(1), "context"),
        key(Uuid::from_u128(2), "context")
    );
    assert_eq!(
        botlib::key_utils::scoped_key(None, &["suggestions"]),
        botlib::key_utils::legacy_key(&["suggestions"])
    );
}
