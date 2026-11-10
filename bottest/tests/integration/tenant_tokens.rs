//! JWT minting for the tenant-isolation suite (#2112).
//!
//! Tenant claims are only trusted when the HS256 signature verifies against
//! the server's `saas_jwt_secret`, so every fixture token here is signed with
//! the same secret the running server resolves. `mint_forged_jwt` keeps the
//! claims intact and breaks only the signature: it must never grant a tenant.

use base64::Engine;
use hmac::{Hmac, Mac};
use serde_json::json;
use sha2::Sha256;

/// Mints a local HS256 JWT whose `email` claim maps to the seeded admin.
fn sign(claims: serde_json::Value) -> String {
    let b64 = |b: &[u8]| base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(b);
    let header = b64(br#"{"alg":"HS256","typ":"JWT"}"#);
    let payload = b64(claims.to_string().as_bytes());
    let signing_input = format!("{header}.{payload}");
    let secret = botsecurity_core::verified_jwt::resolve_saas_jwt_secret().expect("saas jwt secret");
    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).expect("hmac accepts any key");
    mac.update(signing_input.as_bytes());
    format!("{signing_input}.{}", b64(&mac.finalize().into_bytes()))
}

fn claims(slug: &str, org_id: &str, branch_id: &str) -> serde_json::Value {
    json!({
        "sub": format!("zitadel-{}", slug),
        "email": format!("admin-{slug}@tenant.test"),
        "org_id": org_id,
        "branch_id": branch_id,
        "exp": 4102444800i64,
    })
}

/// A valid token for the seeded admin of `slug`.
pub fn mint_admin_jwt(slug: &str, org_id: &str, branch_id: &str) -> String {
    sign(claims(slug, org_id, branch_id))
}

/// The same claims with a signature the server cannot verify.
pub fn mint_forged_jwt(slug: &str, org_id: &str, branch_id: &str) -> String {
    let mut parts: Vec<String> = sign(claims(slug, org_id, branch_id))
        .split('.')
        .map(str::to_string)
        .collect();
    parts[2] = "fakesig".to_string();
    parts.join(".")
}
