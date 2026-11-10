use super::*;

pub(crate) fn jwt_sign(header: &str, payload: &str, secret: &[u8]) -> Result<String, String> {
    use hmac::{Hmac, Mac};
    use sha2::Sha256;
    type HmacSha256 = Hmac<Sha256>;
    let mut mac = HmacSha256::new_from_slice(secret)
        .map_err(|e| format!("HMAC init: {e}"))?;
    let signing_input = format!("{header}.{payload}");
    mac.update(signing_input.as_bytes());
    let signature = base64_url_encode(&mac.finalize().into_bytes());
    Ok(format!("{signing_input}.{signature}"))
}

/// Decodes one JWT segment (URL-safe base64, RFC 7515 §2).
///
/// #2069 — this used to be a hand-rolled decoder that silently *skipped* every
/// character outside the alphabet, so `"eyJ!x"` decoded as if the `!` were not
/// there: a malformed token produced bytes no encoder ever emitted. Padding is
/// optional (the encoder emits unpadded segments), and everything else — a
/// stray character, an impossible length, a padded byte in the middle — is an
/// error. Every caller treats `Err` as "no identity", so a stricter decoder
/// can only deny tokens, never grant a scope.
pub(crate) fn base64_url_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    use base64::Engine as _;

    let unpadded = input.trim_end_matches('=');
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(unpadded)
        .map_err(|_| "invalid base64url input")
}

pub fn get_branch_id_from_jwt(
    headers: &HeaderMap,
    conn: &mut diesel::PgConnection,
) -> Result<Option<Uuid>, String> {
    crate::branch_scope::resolve_branch_id(headers, conn)
}

/// Check if the authenticated user is the SaaS super-admin.
/// Delegates to the canonical platform-admin helper (#1387): explicit
/// membership in the reserved root org (slug = 'default', the first/
/// default bot's org owning the seeded catalog) via user_organizations ∪
/// crm_contacts→branches, or the dev bootstrap admin (admin@localhost).
pub fn is_super_admin(
    headers: &HeaderMap,
    conn: &mut diesel::PgConnection,
) -> Result<bool, String> {
    botsecurity_auth::platform_admin::is_platform_admin(conn, headers)
}

// ─────────────────────────────────────────────────────────────────────────────
// Organizations
