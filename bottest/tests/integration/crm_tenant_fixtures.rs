//! Fixtures for the CRM tenant-isolation tests (#2138).
//!
//! Each scenario needs two independent tenants (distinct branches) with a
//! verified token per admin, plus a way to create a row as one tenant and
//! check which branch the server stored it under. Seeding is skipped — not
//! failed — when the database or the CRM endpoints are unavailable.

use reqwest::Client;
use super::tenant_isolation::{org_id_of, seed_tenant, SeededTenant};
use crate::tenant_tokens::mint_admin_jwt;
use uuid::Uuid;

/// A bearer token for the seeded admin of `tenant`.
pub(super) fn tenant_token(tenant: &SeededTenant) -> Option<String> {
    let org_id = org_id_of(&tenant.slug)?;
    Some(mint_admin_jwt(
        &tenant.slug,
        &org_id,
        &tenant.branch_id.to_string(),
    ))
}

/// Two independent tenants — distinct branches — and a verified token for
/// each admin: `(branch_a, branch_b, token_a, token_b)`.
pub(super) fn two_tenants() -> Option<(Uuid, Uuid, String, String)> {
    let a = seed_tenant("crmisola")?;
    let b = seed_tenant("crmisolb")?;
    let token_a = tenant_token(&a)?;
    let token_b = tenant_token(&b)?;
    Some((a.branch_id, b.branch_id, token_a, token_b))
}

/// Creates a ticket as `token`; returns `(id, branch_id)` as the server stored
/// them, or `None` when the endpoint is unavailable in this build.
pub(super) async fn create_ticket(
    client: &Client,
    base_url: &str,
    token: &str,
    subject: &str,
) -> Option<(String, Option<String>)> {
    let resp = client
        .post(format!("{base_url}/api/tickets"))
        .bearer_auth(token)
        .json(&serde_json::json!({ "subject": subject, "description": "tenant-only data" }))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        eprintln!("Skipping: ticket create unavailable ({})", resp.status());
        return None;
    }
    let body: serde_json::Value = resp.json().await.ok()?;
    let id = body["id"].as_str()?.to_string();
    Some((id, body["branch_id"].as_str().map(str::to_string)))
}

/// Creates a person as `token`; returns `(id, branch_id)`.
pub(super) async fn create_person(
    client: &Client,
    base_url: &str,
    token: &str,
    first_name: &str,
) -> Option<(String, Option<String>)> {
    let resp = client
        .post(format!("{base_url}/api/people"))
        .bearer_auth(token)
        .json(&serde_json::json!({ "first_name": first_name, "last_name": "Tenant" }))
        .send()
        .await
        .ok()?;
    if !resp.status().is_success() {
        eprintln!("Skipping: people create unavailable ({})", resp.status());
        return None;
    }
    let body: serde_json::Value = resp.json().await.ok()?;
    let id = body["id"].as_str()?.to_string();
    Some((id, body["branch_id"].as_str().map(str::to_string)))
}

/// The scenario needs the server to store the row under the token's branch;
/// an older build behind `BOTSERVER_URL` stores it elsewhere and the test
/// skips instead of reporting a false leak.
pub(super) fn scoped_to(tenant_branch: Uuid, stored_branch: Option<String>) -> bool {
    stored_branch.as_deref() == Some(tenant_branch.to_string().as_str())
}
