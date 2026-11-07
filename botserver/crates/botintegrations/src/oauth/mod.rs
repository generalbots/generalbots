pub(crate) use crate::error::IntegrationError;
pub(crate) use crate::repository;
pub(crate) use crate::scope::{resolve_scope, ConnectionScope};
pub(crate) use crate::state::IntegrationState;
pub(crate) use axum::extract::{Path, Query, State};
pub(crate) use axum::http::{header, StatusCode};
pub(crate) use axum::response::{IntoResponse, Redirect, Response};
pub(crate) use axum::Json;
pub(crate) use base64::Engine as _;
pub(crate) use botsecurity_auth::auth_api::types::AuthenticatedUser;
pub(crate) use botsecurity_core::response_error::ResponseError;
pub(crate) use diesel::prelude::*;
pub(crate) use hmac::{Hmac, Mac};
pub(crate) use serde::Deserialize;
pub(crate) use serde_json::{json, Value};
pub(crate) use sha2::Sha256;
pub(crate) use std::sync::Arc;
pub(crate) use uuid::Uuid;

const STATE_MAX_AGE_SECS: i64 = 600;

pub(crate) struct OAuthProviderConfig {
    pub(crate) authorize_url: &'static str,
    pub(crate) token_url: &'static str,
    pub(crate) scopes: &'static str,
    pub(crate) basic_client_auth: bool,
}

#[derive(Deserialize)]
pub struct StartQuery {
    return_to: Option<String>,
}

#[derive(Deserialize)]
pub struct CallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

/// Resolves a provider's OAuth client, preferring the tenant/user-configured
/// Vault entry and falling back to the managed public desktop client.
///
/// Managed ids are public, not secrets: they are injected at build/release
/// time (or supplied at runtime) through `botlib::managed_oauth`. Self-hosters
/// that registered their own app keep working because the Vault lookup wins.
pub(crate) async fn load_client_or_managed(
    state: &IntegrationState,
    scope: &ConnectionScope,
    provider: &str,
) -> Result<(Value, String), IntegrationError> {
    match state
        .vault
        .load_oauth_client(scope.org_id, scope.branch_id, scope.bot_id, provider)
        .await
    {
        Ok(client) => Ok(client),
        Err(_) => {
            let client_id =
                botlib::managed_oauth::client_id_for(provider).ok_or(IntegrationError::NotFound)?;
            Ok((
                json!({ "client_id": client_id }),
                format!("managed public client ({provider})"),
            ))
        }
    }
}

mod build_authorize_url;
mod callback;
mod provider_config;

pub use build_authorize_url::*;
pub use callback::*;
pub(crate) use provider_config::*;
