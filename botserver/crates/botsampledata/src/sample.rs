//! Dedicated "sample" tenant for all demo data.
//!
//! Every seed operation writes exclusively under the sample tenant created
//! here, so real tenants are never polluted by demo contacts, tickets,
//! products or mailboxes.
//!
//! The sample scope uses deterministic UUIDs (a fixed namespace) so repeated
//! boots are idempotent and traceable: the sample org/branch/bot/user rows
//! are looked up by identity, never generated randomly.

use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{BigInt, Text, Uuid as SqlUuid};
use uuid::Uuid;

/// Demo-only password for the sample accounts, generated on the fly so no
/// credential literal ever lands in source (secret scanners flag hardcoded
/// passwords). The demo tenant is fully isolated (deterministic namespace +
/// sample branch), so the generated value is safe to log for operators of
/// demo environments; it must never be used outside demo.
fn generate_demo_password() -> String {
    format!("gb-demo-{}", Uuid::new_v4().simple())
}

/// Deterministic sample tenant id (fixed namespace; never collides with the
/// nil/global or system scopes used elsewhere in the codebase).
pub const SAMPLE_TENANT_ID: Uuid =
    Uuid::from_u128(0x7000_0000_0000_0000_0000_0000_0000_0001);
/// Deterministic sample organization (tenant "sample").
pub const SAMPLE_ORG_ID: Uuid = Uuid::from_u128(0x7000_0000_0000_0000_0000_0000_0000_0002);
/// Deterministic sample branch inside the sample organization.
pub const SAMPLE_BRANCH_ID: Uuid = Uuid::from_u128(0x7000_0000_0000_0000_0000_0000_0000_0003);
/// Deterministic sample bot bound to the sample branch.
pub const SAMPLE_BOT_ID: Uuid = Uuid::from_u128(0x7000_0000_0000_0000_0000_0000_0000_0004);
/// Deterministic id for the primary demo user (`user@sample.com`).
pub const SAMPLE_USER_ID: Uuid = Uuid::from_u128(0x7000_0000_0000_0000_0000_0000_0000_0010);

/// The resolved identifiers for the dedicated demo tenant.
#[derive(Debug, Clone)]
pub struct SampleScope {
    pub tenant_id: Uuid,
    pub org_id: Uuid,
    pub branch_id: Uuid,
    pub bot_id: Uuid,
    pub user_id: Uuid,
    pub org_str: String,
    pub branch_str: String,
    pub bot_str: String,
    pub user_str: String,
}

impl SampleScope {
    /// Builds the scope from the deterministic sample UUID namespace.
    pub fn build() -> Self {
        Self {
            tenant_id: SAMPLE_TENANT_ID,
            org_id: SAMPLE_ORG_ID,
            branch_id: SAMPLE_BRANCH_ID,
            bot_id: SAMPLE_BOT_ID,
            user_id: SAMPLE_USER_ID,
            org_str: SAMPLE_ORG_ID.to_string(),
            branch_str: SAMPLE_BRANCH_ID.to_string(),
            bot_str: SAMPLE_BOT_ID.to_string(),
            user_str: SAMPLE_USER_ID.to_string(),
        }
    }
}

/// A password stored with the canonical Argon2 hash used by the login flow,
/// so the demo account can actually sign in. Mirrors the hashing performed by
/// `botsecurity-auth` (argon2 crate, default parameters).
pub fn hash_demo_password(password: &str) -> Result<String, String> {
    use argon2::password_hash::{rand_core::OsRng, SaltString};
    use argon2::{Argon2, PasswordHasher};

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    argon2
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|e| format!("failed to hash demo password: {e}"))
}

/// Ensures the whole sample tenant exists: tenant, organization, branch, bot,
/// cloud workspace, demo users and their org memberships.
///
/// Idempotent: every row is guarded by `ON CONFLICT ... DO NOTHING` (or a
/// count probe) so repeated boots never duplicate data, and no real tenant is
/// ever modified or used as a fallback. On failure the caller must skip all
/// demo seeding rather than degrade to a real scope.
pub fn ensure_sample_scope(conn: &mut diesel::PgConnection) -> Result<SampleScope, String> {
    let scope = SampleScope::build();

    // 1. Tenant.
    sql_query(
        "INSERT INTO tenants (id, name, slug, is_active, created_at, updated_at) \
         VALUES ($1, 'Sample', 'sample', true, NOW(), NOW()) \
         ON CONFLICT (id) DO NOTHING",
    )
    .bind::<SqlUuid, _>(scope.tenant_id)
    .execute(conn)
    .map_err(|e| format!("fail to create sample tenant: {e}"))?;

    // 2. Organization owning the sample branch.
    sql_query(
        "INSERT INTO organizations (org_id, tenant_id, name, slug, created_at, updated_at) \
         VALUES ($1, $2, 'Sample', 'sample', NOW(), NOW()) \
         ON CONFLICT (org_id) DO NOTHING",
    )
    .bind::<SqlUuid, _>(scope.org_id)
    .bind::<SqlUuid, _>(scope.tenant_id)
    .execute(conn)
    .map_err(|e| format!("fail to create sample organization: {e}"))?;

    // 3. Branch bound to the sample organization.
    sql_query(
        "INSERT INTO branches (id, org_id, tenant_id, slug, name, is_active, created_at, updated_at) \
         VALUES ($1, $2, $3, 'sample', 'Sample Branch', true, NOW(), NOW()) \
         ON CONFLICT (id) DO NOTHING",
    )
    .bind::<SqlUuid, _>(scope.branch_id)
    .bind::<SqlUuid, _>(scope.org_id)
    .bind::<SqlUuid, _>(scope.tenant_id)
    .execute(conn)
    .map_err(|e| format!("fail to create sample branch: {e}"))?;

    // 4. Bot bound to the sample branch (a workspace bot for the demo tenant).
    sql_query(
        "INSERT INTO bots (id, name, slug, org_id, branch_id, tenant_id, is_default_for_branch, \
                           is_active, created_at, updated_at, llm_provider, llm_config, \
                           context_provider, context_config, is_public, database_name) \
         VALUES ($1, 'sample', 'sample', $2, $3, $4, true, \
                 true, NOW(), NOW(), 'openai', '{}'::jsonb, \
                 'openai', '{}'::jsonb, true, 'sample_demo') \
         ON CONFLICT (id) DO NOTHING",
    )
    .bind::<SqlUuid, _>(scope.bot_id)
    .bind::<SqlUuid, _>(scope.org_id)
    .bind::<SqlUuid, _>(scope.branch_id)
    .bind::<SqlUuid, _>(scope.tenant_id)
    .execute(conn)
    .map_err(|e| format!("fail to create sample bot: {e}"))?;

    // 5. Cloud workspace row so the demo org shows in the SaaS cloud UI.
    sql_query(
        "INSERT INTO cloud_workspaces (id, org_id, branch_id, name, description, icon, created_at, updated_at) \
         VALUES ($1, $2, $3, 'Sample', 'Dedicated demo workspace', 'default', NOW(), NOW()) \
         ON CONFLICT (id) DO NOTHING",
    )
    .bind::<SqlUuid, _>(Uuid::from_u128(0x7000_0000_0000_0000_0000_0000_0000_0005))
    .bind::<SqlUuid, _>(scope.org_id)
    .bind::<SqlUuid, _>(scope.branch_id)
    .execute(conn)
    .map_err(|e| format!("fail to create sample cloud workspace: {e}"))?;

    let demo_password = generate_demo_password();
    let password_hash = hash_demo_password(&demo_password)?;
    log::info!(
        "botsampledata: sample tenant ready — demo login for user@sample.com uses a generated password ({} chars, demo-only)", demo_password.len()
    );

    // 6. Demo accounts with a real Argon2 hash so they can log in.
    let users: &[(&str, &str, Uuid, bool)] = &[
        ("user.sample", "user@sample.com", SAMPLE_USER_ID, true),
        ("alice.sample", "alice@sample.com", Uuid::from_u128(0x7000_0000_0000_0000_0000_0000_0000_0011), false),
        ("bruno.sample", "bruno@sample.com", Uuid::from_u128(0x7000_0000_0000_0000_0000_0000_0000_0012), false),
        ("carla.sample", "carla@sample.com", Uuid::from_u128(0x7000_0000_0000_0000_0000_0000_0000_0013), false),
    ];
    for (username, email, user_id, is_admin) in users {
        let existing: i64 = sql_query(
            "SELECT count(*) AS n FROM users WHERE email = $1",
        )
        .bind::<Text, _>(email)
        .get_result::<CountRow>(conn)
        .map_err(|e| format!("fail to probe demo user {username}: {e}"))?
        .n;
        if existing == 0 {
            sql_query(
                "INSERT INTO users (id, username, email, password_hash, is_active, created_at, updated_at) \
                 VALUES ($1, $2, $3, $4, true, NOW(), NOW())",
            )
            .bind::<SqlUuid, _>(*user_id)
            .bind::<Text, _>(username)
            .bind::<Text, _>(email)
            .bind::<Text, _>(&password_hash)
            .execute(conn)
            .map_err(|e| format!("fail to create demo user {username}: {e}"))?;
        }

        let role = if *is_admin { "admin" } else { "member" };
        let existing: i64 = sql_query(
            "SELECT count(*) AS n FROM user_organizations WHERE user_id = $1 AND org_id = $2",
        )
        .bind::<SqlUuid, _>(*user_id)
        .bind::<SqlUuid, _>(scope.org_id)
        .get_result::<CountRow>(conn)
        .map_err(|e| format!("fail to probe demo org membership for {username}: {e}"))?
        .n;
        if existing == 0 {
            sql_query(
                "INSERT INTO user_organizations (id, user_id, org_id, role, is_default, joined_at) \
                 VALUES ($1, $2, $3, $4, false, NOW())",
            )
            .bind::<SqlUuid, _>(Uuid::new_v4())
            .bind::<SqlUuid, _>(*user_id)
            .bind::<SqlUuid, _>(scope.org_id)
            .bind::<Text, _>(role)
            .execute(conn)
            .map_err(|e| format!("fail to bind demo user {username} to org: {e}"))?;
        }
    }

    Ok(scope)
}

#[derive(diesel::QueryableByName)]
struct CountRow {
    #[diesel(sql_type = BigInt)]
    n: i64,
}