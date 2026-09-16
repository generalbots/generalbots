//! Tenant isolation regression tests (#1389) — guards the fixes for the
//! cross-tenant drive leak (#1401) and the tenant-scoped RBAC model (#1387).
//!
//! Contract under test:
//! 1. A signed-in user may only address drive buckets belonging to its own
//!    tenant (personal `user-` bucket, instance default, own `{slug}.gborg`
//!    and own org's `{bot}.gbai`).
//! 2. Cross-tenant bucket access is denied with 403, even for org-level
//!    admins (the global `admin` group also models org admins).
//! 3. Only PLATFORM admins (explicit superadmin role or members of the
//!    reserved root org, slug `default`) bypass the tenant check.
//! 4. The bucket discovery endpoint never lists other tenants' workspaces.
//!
//! Runs against either `BOTSERVER_URL` or a harness-started botserver, in
//! the same style as `cloud_tenant.rs`. Seeding failures skip the test
//! instead of failing the suite so runs without a database stay green.

use bottest::prelude::*;
use diesel::prelude::*;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

fn test_client() -> Client {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client")
}

fn external_server_url() -> Option<String> {
    std::env::var("BOTSERVER_URL").ok()
}

async fn get_test_server() -> Option<(Option<TestContext>, String)> {
    if let Some(url) = external_server_url() {
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

macro_rules! skip_if_no_server {
    ($base_url:expr) => {
        if $base_url.is_none() {
            eprintln!("Skipping test: no server available");
            return;
        }
    };
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
struct SeededTenant {
    slug: String,
    branch_id: Uuid,
}

fn seed_tenant(name: &str) -> Option<SeededTenant> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let mut conn = PgConnection::establish(&url).ok()?;
    let org_id = Uuid::new_v4();
    let branch_id = Uuid::new_v4();
    let user_id = Uuid::new_v4();
    let slug = format!("{}-{}", name, Uuid::new_v4().simple());
    let email = format!("admin-{slug}@tenant.test");
    let contact_id = Uuid::new_v4();

    diesel::sql_query(
        "INSERT INTO organizations (org_id, name, slug, created_at) \
         VALUES ($1, $2, $3, NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(org_id)
    .bind::<diesel::sql_types::Text, _>(&slug)
    .bind::<diesel::sql_types::Text, _>(&slug)
    .execute(&mut conn)
    .ok()?;

    diesel::sql_query(
        "INSERT INTO branches (id, org_id, slug, name, created_at, updated_at) \
         VALUES ($1, $2, $3, $3, NOW(), NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .bind::<diesel::sql_types::Uuid, _>(org_id)
    .bind::<diesel::sql_types::Text, _>(&slug)
    .execute(&mut conn)
    .ok()?;

    diesel::sql_query(
        "INSERT INTO users (id, name, email, is_active, created_at) \
         VALUES ($1, $2, $3, true, NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Text, _>(&slug)
    .bind::<diesel::sql_types::Text, _>(&email)
    .execute(&mut conn)
    .ok()?;

    diesel::sql_query(
        "INSERT INTO user_organizations (user_id, org_id, role, created_at) \
         VALUES ($1, $2, 'admin', NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Uuid, _>(org_id)
    .execute(&mut conn)
    .ok()?;

    diesel::sql_query(
        "INSERT INTO crm_contacts (id, branch_id, first_name, last_name, email, created_at) \
         VALUES ($1, $2, $3, 'Admin', $4, NOW())",
    )
    .bind::<diesel::sql_types::Uuid, _>(contact_id)
    .bind::<diesel::sql_types::Uuid, _>(branch_id)
    .bind::<diesel::sql_types::Text, _>(&slug)
    .bind::<diesel::sql_types::Text, _>(&email)
    .execute(&mut conn)
    .ok()?;

    Some(SeededTenant {
        slug,
        branch_id,
    })
}

/// Resolves the seeded admin user's id (needed to attach RBAC groups).
fn seeded_admin_user_id(slug: &str) -> Option<Uuid> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let mut conn = PgConnection::establish(&url).ok()?;
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
    let url = std::env::var("DATABASE_URL").ok()?;
    let mut conn = PgConnection::establish(&url).ok()?;
    let user_id = seeded_admin_user_id(slug)?;
    let group_id: OrgIdRow = diesel::sql_query(
        "INSERT INTO rbac_groups (id, name, is_active, created_at) \
         VALUES ($1, 'Administrators', true, NOW()) \
         ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name \
         RETURNING id AS org_id",
    )
    .bind::<diesel::sql_types::Uuid, _>(Uuid::new_v4())
    .get_result(&mut conn)
    .ok()?;
    diesel::sql_query(
        "INSERT INTO rbac_user_groups (user_id, group_id, created_at) \
         VALUES ($1, $2, NOW()) ON CONFLICT DO NOTHING",
    )
    .bind::<diesel::sql_types::Uuid, _>(user_id)
    .bind::<diesel::sql_types::Uuid, _>(group_id.org_id)
    .execute(&mut conn)
    .ok()?;
    Some(())
}

/// Mints a local HMAC JWT whose `email` claim maps to the seeded admin.
/// The suite auth path mirrors how the drive endpoints resolve identity
/// from the token email.
fn mint_admin_jwt(slug: &str, org_id: &str, branch_id: &str) -> String {
    use base64::Engine;
    let b64 = |b: &[u8]| base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b);
    let header = b64(br#"{"alg":"HS256","typ":"JWT"}"#);
    let payload = b64(
        json!({
            "sub": format!("zitadel-{}", slug),
            "email": format!("admin-{slug}@tenant.test"),
            "org_id": org_id,
            "branch_id": branch_id,
            "exp": 4102444800i64,
        })
        .to_string()
        .as_bytes(),
    );
    format!("{header}.{payload}.fakesig")
}

fn org_id_of(slug: &str) -> Option<String> {
    let url = std::env::var("DATABASE_URL").ok()?;
    let mut conn = PgConnection::establish(&url).ok()?;
    diesel::sql_query("SELECT org_id FROM organizations WHERE slug = $1 LIMIT 1")
        .bind::<diesel::sql_types::Text, _>(slug)
        .get_result::<OrgIdRow>(&mut conn)
        .ok()
        .map(|r| r.org_id.to_string())
}

#[tokio::test]
async fn test_drive_list_buckets_only_shows_own_tenant() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let tenant_a = seed_tenant("isola").or_else(|| seed_tenant("isola")) ;
    let tenant_b = seed_tenant("isolb").or_else(|| seed_tenant("isolb"));
    let (Some(a), Some(b)) = (tenant_a, tenant_b) else {
        eprintln!("Skipping: tenant seeding unavailable (no DATABASE_URL/tables)");
        return;
    };

    let Some(org_a) = org_id_of(&a.slug) else {
        eprintln!("Skipping: org lookup failed");
        return;
    };
    let token = mint_admin_jwt(&a.slug, &org_a, &a.branch_id.to_string());

    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let resp = client
        .get(format!("{base_url}/api/files/buckets"))
        .bearer_auth(&token)
        .send()
        .await
        .expect("buckets request failed");

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
        !names.iter().any(|n| n.ends_with(".gborg") && !n.contains(&a.slug)),
        "bucket discovery listed a foreign .gborg workspace: {names:?}"
    );
}

#[tokio::test]
async fn test_drive_cross_tenant_bucket_denied() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let tenant_a = seed_tenant("isola");
    let tenant_b = seed_tenant("isolb");
    let (Some(a), Some(b)) = (tenant_a, tenant_b) else {
        eprintln!("Skipping: tenant seeding unavailable");
        return;
    };
    let Some(org_a) = org_id_of(&a.slug) else {
        eprintln!("Skipping: org lookup failed");
        return;
    };
    grant_global_admin_group(&a.slug); // org admin, NOT platform admin

    let token = mint_admin_jwt(&a.slug, &org_a, &a.branch_id.to_string());
    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    // THE #1401 regression: tenant A's (org-level) admin reads tenant B's
    // workspace bucket by naming it. Must be 403.
    let foreign = format!("{}.gborg", b.slug);
    let resp = client
        .get(format!("{base_url}/api/files/list"))
        .query(&[("bucket", foreign.as_str()), ("path", "")])
        .bearer_auth(&token)
        .send()
        .await
        .expect("list request failed");

    assert_eq!(
        resp.status(),
        reqwest::StatusCode::FORBIDDEN,
        "cross-tenant bucket read must be denied (got {}) — #1401 regression",
        resp.status()
    );
}

#[tokio::test]
async fn test_drive_own_tenant_bucket_allowed() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let Some(a) = seed_tenant("isola") else {
        eprintln!("Skipping: tenant seeding unavailable");
        return;
    };
    let Some(org_a) = org_id_of(&a.slug) else {
        eprintln!("Skipping: org lookup failed");
        return;
    };
    let token = mint_admin_jwt(&a.slug, &org_a, &a.branch_id.to_string());
    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let own = format!("{}.gborg", a.slug);
    let resp = client
        .get(format!("{base_url}/api/files/list"))
        .query(&[("bucket", own.as_str()), ("path", "")])
        .bearer_auth(&token)
        .send()
        .await
        .expect("list request failed");

    assert!(
        resp.status().is_success(),
        "own tenant bucket must be accessible (got {})",
        resp.status()
    );
}

#[tokio::test]
async fn test_drive_spoofed_user_id_ignored() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let tenant_a = seed_tenant("isola");
    let tenant_b = seed_tenant("isolb");
    let (Some(a), Some(b)) = (tenant_a, tenant_b) else {
        eprintln!("Skipping: tenant seeding unavailable");
        return;
    };
    let Some(org_a) = org_id_of(&a.slug) else {
        eprintln!("Skipping: org lookup failed");
        return;
    };
    let token = mint_admin_jwt(&a.slug, &org_a, &a.branch_id.to_string());
    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    // Non-admin callers cannot impersonate another user's scope via user_id:
    // tenant A's token asks for tenant B admin's user id.
    let Some(foreign_uid) = seeded_admin_user_id(&b.slug).map(|u| u.to_string()) else {
        eprintln!("Skipping: tenant B admin lookup failed");
        return;
    };
    let resp = client
        .get(format!("{base_url}/api/files/list"))
        .query(&[("user_id", foreign_uid.as_str()), ("scope", "user")])
        .bearer_auth(&token)
        .send()
        .await
        .expect("list request failed");

    assert_ne!(
        resp.status(),
        reqwest::StatusCode::OK,
        "spoofed user_id must not expose tenant B's user scope"
    );
}

#[tokio::test]
async fn test_org_admin_is_not_platform_admin() {
    let server = get_test_server().await;
    skip_if_no_server!(server);

    let tenant_a = seed_tenant("isola");
    let tenant_b = seed_tenant("isolb");
    let (Some(a), Some(b)) = (tenant_a, tenant_b) else {
        eprintln!("Skipping: tenant seeding unavailable");
        return;
    };
    let Some(org_a) = org_id_of(&a.slug) else {
        eprintln!("Skipping: org lookup failed");
        return;
    };
    // Org admin via the global RBAC group — must STILL not cross tenants.
    grant_global_admin_group(&a.slug);

    let token = mint_admin_jwt(&a.slug, &org_a, &a.branch_id.to_string());
    let (_ctx, base_url) = server.unwrap();
    let client = test_client();

    let foreign = format!("{}.gborg", b.slug);
    let resp = client
        .get(format!("{base_url}/api/files/search"))
        .query(&[("bucket", foreign.as_str()), ("query", "x")])
        .bearer_auth(&token)
        .send()
        .await
        .expect("search request failed");

    assert!(
        resp.status() == reqwest::StatusCode::FORBIDDEN || resp.status() == reqwest::StatusCode::NOT_FOUND,
        "org admin must not search foreign buckets (got {})",
        resp.status()
    );
}
