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

use crate::auth::{AuthError, AuthenticatedUser, Role};
use crate::auth_provider::AuthProvider;
use async_trait::async_trait;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde_json::Value;
use std::collections::HashSet;
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
        let mut validation = Validation::new(Algorithm::HS256);
        validation.validate_exp = true;
        validation.required_spec_claims = HashSet::from(["exp".to_string()]);
        // No iss/aud enforcement: cloud JWTs are minted claim-light.

        let data = decode::<Value>(token, &DecodingKey::from_secret(self.secret.as_bytes()), &validation)
            .map_err(|e| {
                log::warn!("saas-jwt token rejected: {e}");
                AuthError::InvalidToken
            })?;

        let claims = data.claims;
        let sub = claims
            .get("sub")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        let email = claims
            .get("email")
            .and_then(|v| v.as_str())
            .unwrap_or_default();

        let user_id = Uuid::parse_str(sub).unwrap_or_else(|_| {
            Uuid::new_v5(&Uuid::NAMESPACE_DNS, format!("zitadel:{sub}").as_bytes())
        });
        let username = email.split('@').next().unwrap_or("user").to_string();

        let mut user = AuthenticatedUser::new(user_id, username)
            .with_email(email.to_string())
            .with_role(Role::User);

        if let Some(org) = claims
            .get("org_id")
            .and_then(|v| v.as_str())
            .and_then(|s| Uuid::parse_str(s).ok())
        {
            user = user.with_organization(org);
        }

        Ok(user)
    }

    async fn authenticate_api_key(&self, _api_key: &str) -> Result<AuthenticatedUser, AuthError> {
        Err(AuthError::InvalidApiKey)
    }
}