use super::*;

/// Require specific role - returns 403 if role not present
pub fn require_role_middleware(
    required_role: &'static str,
) -> impl Fn(Request<Body>, Next) -> MiddlewareFuture + Clone + Send {
    move |request: Request<Body>, next: Next| {
        Box::pin(async move {
            let user = request
                .extensions()
                .get::<AuthenticatedUser>()
                .cloned()
                .unwrap_or_else(AuthenticatedUser::anonymous);

            if !user.has_role(required_role) {
                return Err(ResponseError::new(
                    ForbiddenResponse::new(&format!("Required role: {required_role}")).into_response(),
                ));
            }

            Ok(next.run(request).await)
        })
    }
}
/// Require organization context - returns 400 if no organization
pub async fn require_organization_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, ResponseError> {
    let has_org = request.extensions().get::<OrganizationContext>().is_some();

    if !has_org {
        return Err(ResponseError::new(
            BadRequestResponse::new("Organization context required").into_response(),
        ));
    }

    Ok(next.run(request).await)
}
/// Require admin role within organization
pub async fn require_org_admin_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, ResponseError> {
    let context = request.extensions().get::<RequestContext>().cloned();

    let is_admin = context
        .as_ref()
        .map(|c| c.user.is_admin || c.user.is_super_admin || c.has_org_permission("admin"))
        .unwrap_or(false);

    if !is_admin {
        return Err(ResponseError::new(
            ForbiddenResponse::new("Organization admin access required").into_response(),
        ));
    }

    Ok(next.run(request).await)
}

// ============================================================================
// Extractors
// ============================================================================
/// Extract bot ID from request
pub(super) fn extract_bot_id(request: &Request<Body>) -> Option<Uuid> {
    // Try header
    if let Some(header) = request.headers().get("X-Bot-Id") {
        if let Ok(s) = header.to_str() {
            if let Ok(id) = Uuid::parse_str(s) {
                return Some(id);
            }
        }
    }

    // Try path
    let path = request.uri().path();
    if path.contains("/bots/") {
        let parts: Vec<&str> = path.split('/').collect();
        for (i, part) in parts.iter().enumerate() {
            if *part == "bots" && i + 1 < parts.len() {
                if let Ok(id) = Uuid::parse_str(parts[i + 1]) {
                    return Some(id);
                }
            }
        }
    }

    None
}
/// Extract conversation ID from request
pub(super) fn extract_conversation_id(request: &Request<Body>) -> Option<Uuid> {
    // Try header
    if let Some(header) = request.headers().get("X-Conversation-Id") {
        if let Ok(s) = header.to_str() {
            if let Ok(id) = Uuid::parse_str(s) {
                return Some(id);
            }
        }
    }

    // Try path
    let path = request.uri().path();
    if path.contains("/conversations/") {
        let parts: Vec<&str> = path.split('/').collect();
        for (i, part) in parts.iter().enumerate() {
            if *part == "conversations" && i + 1 < parts.len() {
                if let Ok(id) = Uuid::parse_str(parts[i + 1]) {
                    return Some(id);
                }
            }
        }
    }

    None
}
/// Extract client IP from request headers
pub(super) fn extract_client_ip(request: &Request<Body>) -> Option<String> {
    // Try common proxy headers
    for header_name in &[
        "X-Forwarded-For",
        "X-Real-IP",
        "CF-Connecting-IP",
        "True-Client-IP",
    ] {
        if let Some(header) = request.headers().get(*header_name) {
            if let Ok(value) = header.to_str() {
                // X-Forwarded-For can contain multiple IPs
                let ip = value.split(',').next().map(|s| s.trim().to_string());
                if ip.is_some() {
                    return ip;
                }
            }
        }
    }

    None
}
/// Extract and validate user from authorization header
pub(super) async fn extract_and_validate_user(
    request: &Request<Body>,
    state: &ContextMiddlewareState,
) -> Result<AuthenticatedUser, AuthError> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
        .ok_or(AuthError::MissingToken)?;

    let token = if let Some(stripped) = auth_header.strip_prefix("Bearer ") {
        stripped
    } else {
        return Err(AuthError::InvalidFormat);
    };

    // Validate JWT token
    let claims = validate_jwt(token, &state.jwt_secret)?;

    let user_id =
        Uuid::parse_str(&claims.sub).map_err(|_| AuthError::InvalidToken("Invalid user ID".to_string()))?;

    let user = AuthenticatedUser::new(user_id).with_email(claims.sub.clone());

    Ok(AuthenticatedUser {
        token_claims: Some(claims),
        ..user
    })
}
/// Key material for the request JWT verifier.
///
/// The variant *is* the algorithm: a deployment configures exactly one of the
/// two, and the verifier never falls back from one to the other. The previous
/// code tried HS256 first and then RS256 with the same bytes when the secret
/// looked like a PEM, which is the classic HS256/RS256 confusion hole — an
/// attacker who knows the RSA public key could sign HS256 with it and pass the
/// first branch (#2125).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum JwtVerifierKey {
    /// Opaque shared secret for HMAC-SHA256.
    Hs256(Vec<u8>),
    /// RSA public key in PEM form for RS256.
    Rs256(Vec<u8>),
}

impl JwtVerifierKey {
    /// Derives the configured algorithm from the material's shape. PEM
    /// material is only ever an RSA public key; anything else is an HMAC
    /// secret. The choice is made once, from configuration, never per token.
    pub(super) fn from_secret(secret: &str) -> Result<Self, AuthError> {
        if secret.trim().is_empty() {
            return Err(AuthError::InvalidToken(
                "JWT verification key is not configured".to_string(),
            ));
        }
        if secret.contains("-----BEGIN") {
            return Ok(Self::Rs256(secret.as_bytes().to_vec()));
        }
        Ok(Self::Hs256(secret.as_bytes().to_vec()))
    }

    pub(super) fn algorithm(&self) -> Algorithm {
        match self {
            Self::Hs256(_) => Algorithm::HS256,
            Self::Rs256(_) => Algorithm::RS256,
        }
    }

    fn decoding_key(&self) -> Result<DecodingKey, AuthError> {
        match self {
            Self::Hs256(secret) => Ok(DecodingKey::from_secret(secret)),
            Self::Rs256(pem) => DecodingKey::from_rsa_pem(pem)
                .map_err(|e| AuthError::InvalidToken(format!("Invalid RSA public key: {e}"))),
        }
    }
}

/// Validate JWT token and extract claims using jsonwebtoken crate.
pub(super) fn validate_jwt(token: &str, secret: &str) -> Result<TokenClaims, AuthError> {
    validate_jwt_with_key(token, &JwtVerifierKey::from_secret(secret)?)
}

/// Validates `token` against exactly one algorithm and one key. A token whose
/// header advertises a different algorithm — `none`, RS256 against an HMAC
/// key, HS256 against an RSA key — is rejected before the signature is
/// compared, because `Validation` only accepts `key.algorithm()`.
pub(super) fn validate_jwt_with_key(
    token: &str,
    key: &JwtVerifierKey,
) -> Result<TokenClaims, AuthError> {
    let mut validation = Validation::new(key.algorithm());
    validation.validate_exp = true;
    validation.validate_nbf = false;
    validation.set_required_spec_claims(&["sub", "exp"]);

    let decoding_key = key.decoding_key()?;

    match decode::<TokenClaims>(token, &decoding_key, &validation) {
        Ok(token_data) => Ok(token_data.claims),
        Err(e) => Err(match e.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::TokenExpired,
            jsonwebtoken::errors::ErrorKind::InvalidSignature => {
                AuthError::InvalidToken(format!("Invalid signature: {e}"))
            }
            jsonwebtoken::errors::ErrorKind::InvalidToken => {
                AuthError::InvalidToken("Malformed token".to_string())
            }
            jsonwebtoken::errors::ErrorKind::InvalidIssuer => {
                AuthError::InvalidToken("Invalid issuer".to_string())
            }
            jsonwebtoken::errors::ErrorKind::InvalidAudience => {
                AuthError::InvalidToken("Invalid audience".to_string())
            }
            jsonwebtoken::errors::ErrorKind::InvalidSubject => {
                AuthError::InvalidToken("Invalid subject".to_string())
            }
            jsonwebtoken::errors::ErrorKind::MissingRequiredClaim(claim) => {
                AuthError::InvalidToken(format!("Missing required claim: {claim}"))
            }
            _ => AuthError::InvalidToken(format!("Token validation failed: {e}")),
        }),
    }
}
