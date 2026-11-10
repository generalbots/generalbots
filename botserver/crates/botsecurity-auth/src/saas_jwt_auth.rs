//! SaaS cloud-JWT authentication provider.
//!
//! The cloud login flow (`botcloud::api::handle_login`) mints HS256 JWTs
//! signed with the persisted `saas_jwt_secret` carrying `sub`/`email`/
//! `org_id`/`branch_id` claims (issue #736). These tokens have no
//! `iss`/`aud`/`token_type` claims, so the generic `LocalJwtAuthProvider`
//! (which enforces them) rejects them — the suite middleware then degraded
//! every cloud-authenticated request to an anonymous "session-user",
//! leaving CRM/Drive apps scoped to a random default branch. This provider
//! validates the SaaS JWT signature + expiry only.
//!
//! #2112 — the verification itself lives in
//! [`botsecurity_core::verified_jwt`], the same function the tenant resolver
//! uses, so the login path and every `*_from_claims` helper agree on the
//! secret, algorithm and expiry rules.

use crate::auth::{AuthError, AuthenticatedUser, Role};
use crate::auth_provider::AuthProvider;
use async_trait::async_trait;
use botsecurity_core::verified_jwt::verify_hs256_jwt;
use uuid::Uuid;

pub struct SaasJwtAuthProvider {
    secret: String,
}

impl SaasJwtAuthProvider {
    pub fn new(secret: impl Into<String>) -> Self {
        Self {
            secret: secret.into(),
        }
    }
}

#[async_trait]
impl AuthProvider for SaasJwtAuthProvider {
    fn name(&self) -> &str {
        "saas-jwt"
    }

    fn priority(&self) -> i32 {
        50
    }

    fn is_enabled(&self) -> bool {
        !self.secret.is_empty()
    }

    fn supports_token_type(&self, token: &str) -> bool {
        token.split('.').count() == 3
    }

    async fn authenticate(&self, token: &str) -> Result<AuthenticatedUser, AuthError> {
        log::info!("saas-jwt provider validating token");
        let claims = verify_hs256_jwt(token, &self.secret).map_err(|reason| {
            log::warn!("saas-jwt token rejected: {reason}");
            AuthError::InvalidToken
        })?;

        let user_id = Uuid::parse_str(&claims.sub).unwrap_or_else(|_| {
            Uuid::new_v5(&Uuid::NAMESPACE_DNS, format!("zitadel:{}", claims.sub).as_bytes())
        });
        let email = claims.email.unwrap_or_default();
        let username = email.split('@').next().unwrap_or("user").to_string();

        let mut user = AuthenticatedUser::new(user_id, username)
            .with_email(email)
            .with_role(Role::User);

        if let Some(org) = claims.org_id {
            user = user.with_organization(org);
        }

        Ok(user)
    }

    async fn authenticate_api_key(&self, _api_key: &str) -> Result<AuthenticatedUser, AuthError> {
        Err(AuthError::InvalidApiKey)
    }
}