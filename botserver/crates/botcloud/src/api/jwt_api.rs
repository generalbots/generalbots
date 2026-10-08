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

pub(crate) fn base64_url_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    // Convert URL-safe base64 to standard base64
    let raw = input.replace('-', "+").replace('_', "/");
    let raw = match raw.len() % 4 {
        2 => raw + "==",
        3 => raw + "=",
        0 => raw,
        _ => return Err("invalid base64 input length"),
    };
    // Decode base64 manually (no external dep needed)
    let chars: Vec<char> = raw.chars().collect();
    let mut out = Vec::with_capacity(chars.len() / 4 * 3);
    let alphabet: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '=' { break; }
        let mut vals = [0u8; 4];
        let mut valid = 0usize;
        for j in 0..4 {
            if i + j >= chars.len() || chars[i + j] == '=' { break; }
            if let Some(pos) = alphabet.iter().position(|&a| a as char == chars[i + j]) {
                vals[valid] = pos as u8;
                valid += 1;
            }
        }
        if valid == 0 { break; }
        let n = ((vals[0] as u32) << 18)
            | (if valid > 1 { (vals[1] as u32) << 12 } else { 0 })
            | (if valid > 2 { (vals[2] as u32) << 6 } else { 0 })
            | (if valid > 3 { vals[3] as u32 } else { 0 });
        out.push((n >> 16) as u8);
        if valid > 2 { out.push((n >> 8) as u8); }
        if valid > 3 { out.push(n as u8); }
        i += 4;
    }
    Ok(out)
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
