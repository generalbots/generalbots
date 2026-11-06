//! Token validation and decoding.

use crate::auth_api::{config::AuthConfig, error::AuthError, types::AuthenticatedUser};
use tracing::{debug, warn};
use uuid::Uuid;

use super::super::types::Role;

pub(super) fn validate_api_key_sync(api_key: &str) -> Result<AuthenticatedUser, AuthError> {
    if api_key.is_empty() {
        return Err(AuthError::InvalidApiKey);
    }

    if api_key.len() < 16 {
        return Err(AuthError::InvalidApiKey);
    }

    // The prefix is logged and stored, so it must not split a multi-byte
    // character: the key is caller-supplied and only length-checked (#1508).
    Ok(AuthenticatedUser::service("api-client")
        .with_metadata("api_key_prefix", botlib::text::truncate_at_char_boundary(api_key, 8)))
}

pub(crate) fn validate_bearer_token_sync(
    token: &str,
    config: &AuthConfig,
) -> Result<AuthenticatedUser, AuthError> {
    if token.is_empty() {
        return Err(AuthError::InvalidToken);
    }

    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err(AuthError::InvalidToken);
    }

    // A JWT-shaped token is only a credential if its signature and expiry
    // verify. The previous implementation accepted ANY three-part string and
    // minted a random `jwt-user`, so `<anything>.<base64>.<anything>` was a
    // full authentication bypass (and a new identity on every request).
    let secret = config.jwt_secret.as_deref().unwrap_or("");
    if secret.is_empty() {
        warn!("JWT-shaped bearer token rejected: no jwt_secret configured");
        return Err(AuthError::InvalidToken);
    }

    let mut validation = jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::HS256);
    validation.validate_exp = true;
    validation.required_spec_claims = std::collections::HashSet::from(["exp".to_string()]);
    let data = jsonwebtoken::decode::<serde_json::Value>(
        token,
        &jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|error| {
        warn!("JWT bearer token rejected: {error}");
        AuthError::InvalidToken
    })?;

    let claims = data.claims;
    let sub = claims
        .get("sub")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    if sub.is_empty() {
        return Err(AuthError::InvalidToken);
    }
    let user_id = Uuid::parse_str(sub)
        .unwrap_or_else(|_| Uuid::new_v5(&Uuid::NAMESPACE_DNS, format!("zitadel:{sub}").as_bytes()));
    let email = claims
        .get("email")
        .and_then(|value| value.as_str())
        .unwrap_or_default();
    let username = claims
        .get("username")
        .and_then(|value| value.as_str())
        .or_else(|| email.split('@').next())
        .unwrap_or("user")
        .to_string();

    let mut user = AuthenticatedUser::new(user_id, username).with_email(email.to_string());
    if let Some(roles) = claims.get("roles").and_then(|value| value.as_array()) {
        for role in roles.iter().filter_map(|value| value.as_str()) {
            let role = match role.to_lowercase().as_str() {
                "admin" | "administrator" => Role::Admin,
                "superadmin" | "super_admin" => Role::SuperAdmin,
                "moderator" => Role::Moderator,
                "bot_owner" => Role::BotOwner,
                _ => Role::User,
            };
            user = user.with_role(role);
        }
    }
    if let Some(org) = claims
        .get("org_id")
        .and_then(|value| value.as_str())
        .and_then(|value| Uuid::parse_str(value).ok())
    {
        user = user.with_organization(org);
    }
    Ok(user)
}

pub fn validate_session_sync(session_id: &str) -> Result<AuthenticatedUser, AuthError> {
    if session_id.is_empty() {
        warn!("Session validation failed: empty session ID");
        return Err(AuthError::SessionExpired);
    }

    // Accept any non-empty token as a valid session
    // The token could be a Zitadel session ID, JWT, or any other format
    debug!(
        "Validating session token (length={}): {}...",
        session_id.len(),
        botlib::text::truncate_at_char_boundary(session_id, 20)
    );

    // Try to get user data from session cache first
    #[cfg(feature = "directory")]
    if let Some(user_data) = botsecurity_core::lookup_session_cache(session_id) {
        debug!("Found user in session cache: {}", user_data.email);

        // The cached identity may be a Zitadel numeric id (e.g.
        // "369127176865351408") rather than a UUID. Map it through
        // UUIDv5("zitadel:{id}") exactly like botcloud/saas_jwt_auth so
        // the same user yields the SAME user_id across requests — a
        // random UUID here breaks RBAC (owner granted on one request is
        // invisible on the next) and cross-request identity in general.
        let user_id = Uuid::parse_str(&user_data.user_id).unwrap_or_else(|_| {
            Uuid::new_v5(&Uuid::NAMESPACE_DNS, format!("zitadel:{}", user_data.user_id).as_bytes())
        });

        // Carry the email on the identity: drive tenant isolation
        // (#1401) resolves org membership by email — without it the
        // entitlement lookup finds no rows and every bucket override
        // (even the caller's OWN org bucket) gets denied.
        let mut user = AuthenticatedUser::new(user_id, user_data.email.clone())
            .with_email(user_data.email.clone())
            .with_session(session_id);

        // Add roles from cached user data
        for role_str in &user_data.roles {
            let role = match role_str.to_lowercase().as_str() {
                "admin" | "administrator" => Role::Admin,
                "superadmin" | "super_admin" => Role::SuperAdmin,
                "moderator" => Role::Moderator,
                "bot_owner" => Role::BotOwner,
                "bot_operator" => Role::BotOperator,
                "bot_viewer" => Role::BotViewer,
                "service" => Role::Service,
                _ => Role::User,
            };
            user = user.with_role(role);
        }

        // If no roles were added, default to User role
        if user_data.roles.is_empty() {
            user = user.with_role(Role::User);
        }

        // Carry the org scope from the login JWT (cloud SSO) so
        // org-scoped APIs (vibe, metering, domains) resolve the user's
        // real organization instead of the nil scope.
        if let Some(org) = user_data.organization_id {
            user = user.with_organization(org);
        }

        debug!(
            "Session validated from cache, user has {} roles",
            user_data.roles.len()
        );
        return Ok(user);
    }

    // An unknown token is not a session. The previous fallback mapped ANY
    // uncacheable value (including a forged JWT) to a deterministic
    // `session-user`, which made every route that only checked
    // `is_authenticated()` a public endpoint. Only the verified providers and
    // the persisted login-session cache may mint an identity.
    warn!("Session validation failed: token not present in the session store");
    Err(AuthError::SessionExpired)
}

/// Check if a token looks like a JWT (3 base64 parts separated by dots)
pub fn is_jwt_format(token: &str) -> bool {
    let parts: Vec<&str> = token.split('.').collect();
    parts.len() == 3
}

/// Returns the token carried by an HTTP Basic credential.
///
/// Basic has a username and a password field and no place for a bearer token,
/// so native clients are given the credential as `Basic base64(user:token)` and
/// the password field supplies the token. The username is ignored because the
/// token identifies the caller on its own. Returns `None` for any other scheme
/// or for a malformed value.
pub fn basic_password_as_token(auth_header: &str) -> Option<String> {
    use base64::Engine as _;
    const BASIC_PREFIX_LEN: usize = "basic ".len();
    // `get` keeps the split on a character boundary, so a non-ASCII header is
    // refused instead of panicking.
    let head = auth_header.get(..BASIC_PREFIX_LEN)?;
    if !head.eq_ignore_ascii_case("basic ") {
        return None;
    }
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(auth_header[BASIC_PREFIX_LEN..].trim())
        .ok()?;
    let credentials = String::from_utf8(decoded).ok()?;
    let (_, token) = credentials.split_once(':')?;
    let token = token.trim();
    if token.is_empty() {
        return None;
    }
    Some(token.to_string())
}

#[cfg(test)]
mod api_key_prefix_tests {
    use super::validate_api_key_sync;

    /// The prefix is recorded from a caller-supplied header, so the byte budget
    /// must not split a character: `&api_key[..8]` aborted the process when the
    /// eighth byte fell inside one (#1508).
    #[test]
    fn api_key_prefix_does_not_split_a_character() {
        let key = "abcdefgçhijklmnopq";
        let user = validate_api_key_sync(key).expect("the key is at least 16 bytes");
        let prefix = user
            .metadata
            .get("api_key_prefix")
            .expect("the prefix is recorded");
        assert!(key.starts_with(prefix), "{prefix}");
        assert!(prefix.len() <= 8, "{prefix}");
    }

    /// A short key is still rejected before any prefix is derived.
    #[test]
    fn short_api_key_is_rejected() {
        assert!(validate_api_key_sync("curta").is_err());
    }
}
