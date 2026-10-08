use super::*;

/// True when the host provably stays inside a trusted perimeter: loopback,
/// RFC1918/ULA private addresses or link-local. DNS hostnames cannot be
/// verified as private without resolution (SSRF risk) and always need https.
pub(crate) fn directory_url_allows_password(dir_url: &str, allow_insecure_http: bool) -> bool {
    if allow_insecure_http {
        return true;
    }
    let parsed = match url::Url::parse(dir_url) {
        Ok(p) => p,
        Err(_) => return false,
    };
    let host = parsed.host_str().unwrap_or("");
    let h = host.trim_start_matches('[').trim_end_matches(']');
    let private = h == "localhost"
        || h.starts_with("127.")
        || h.parse::<std::net::IpAddr>().is_ok_and(|addr| match addr {
            std::net::IpAddr::V4(v4) => v4.is_loopback() || v4.is_private() || v4.is_link_local(),
            std::net::IpAddr::V6(v6) => {
                v6.is_loopback() || (v6.segments()[0] & 0xfe00) == 0xfc00 || v6.is_unicast_link_local()
            }
        });
    parsed.scheme() == "https" || (parsed.scheme() == "http" && private)
}

/// Provision the local `users` row for a freshly created directory identity.
/// The id is the canonical UUIDv5 of `zitadel:{directory_user_id}` (same
/// derivation `resolve_login_subject` and RBAC use), so login's stable-subject
/// resolution finds the row on the very first sign-in. Without it, password
/// verification succeeds but no subject can be resolved and the login 401s
/// forever (#1365).
/// Zitadel default password complexity (both in prod and dev builds): at least
/// 8 characters with uppercase, lowercase, digit and symbol. Mirrored here so
/// signup can reject a non-compliant password before any account state is
/// written — the directory error `Password must contain symbol (DOMAIN-ZDLwA)`
/// is what an unsignable account looked like. Returns the human-readable
/// violation, or `None` when the password satisfies the policy.
pub(crate) fn password_policy_violation(password: &str) -> Option<String> {
    if password.chars().count() < 8 {
        return Some("Password must be at least 8 characters long.".to_string());
    }
    if !password.chars().any(|c| c.is_ascii_uppercase()) {
        return Some("Password must contain an uppercase letter.".to_string());
    }
    if !password.chars().any(|c| c.is_ascii_lowercase()) {
        return Some("Password must contain a lowercase letter.".to_string());
    }
    if !password.chars().any(|c| c.is_ascii_digit()) {
        return Some("Password must contain a digit.".to_string());
    }
    if !password.chars().any(|c| !c.is_ascii_alphanumeric()) {
        return Some("Password must contain a symbol (e.g. !@#$%).".to_string());
    }
    None
}

pub(crate) fn provision_user_row(conn: &mut diesel::PgConnection, zitadel_user_id: &str, email: &str) {
    let derived = Uuid::new_v5(&Uuid::NAMESPACE_DNS, format!("zitadel:{zitadel_user_id}").as_bytes());
    let username = email.split('@').next().unwrap_or(email).to_string();
    let result = diesel::sql_query(
        "INSERT INTO users (id, username, email, password_hash, created_at, updated_at, is_active) \
         VALUES ($1, $2, $3, '', NOW(), NOW(), true) \
         ON CONFLICT (id) DO NOTHING",
    )
    .bind::<diesel::sql_types::Uuid, _>(derived)
    .bind::<diesel::sql_types::Text, _>(username)
    .bind::<diesel::sql_types::Text, _>(email)
    .execute(conn);
    match result {
        Ok(n) if n > 0 => tracing::info!("Provisioned users row {derived} for directory identity {zitadel_user_id}"),
        Ok(_) => {}
        Err(e) => tracing::warn!("users row provisioning failed for {email}: {e} (login still resolvable by email)"),
    }
}

