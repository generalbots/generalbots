use axum::body::Body;
use tracing::{debug, warn};

use super::auth_api_utils::{
    ExtractedAuthData, is_jwt_format, validate_session_sync,
};
use super::config::AuthConfig;
use super::error::AuthError;
use super::types::AuthenticatedUser;
use crate::auth_provider::AuthProviderRegistry;

pub async fn authenticate_with_extracted_data(
    data: ExtractedAuthData,
    config: &AuthConfig,
    registry: &AuthProviderRegistry,
) -> Result<AuthenticatedUser, AuthError> {
    if let Some(key) = data.api_key {
        let mut user = registry.authenticate_api_key(&key).await?;
        if let Some(bid) = data.bot_id {
            user = user.with_current_bot(bid);
        }
        return Ok(user);
    }

    if let Some(token) = data.bearer_token {
        debug!("Authenticating bearer token (length={})", token.len());

        // Check if token is JWT format - if so, try providers first
        if is_jwt_format(&token) {
            debug!("Token appears to be JWT format, trying JWT providers");
            match registry.authenticate_token(&token).await {
                Ok(mut user) => {
                    debug!("JWT authentication successful for user: {}", user.user_id);
                    if let Some(bid) = data.bot_id {
                        user = user.with_current_bot(bid);
                    }
                    return Ok(user);
                }
                Err(e) => {
                    debug!(
                        "JWT authentication failed: {:?}, falling back to session validation",
                        e
                    );
                }
            }
        } else {
            debug!("Token is not JWT format, treating as session ID");
        }

        // Treat token as session ID (Zitadel session or other)
        match validate_session_sync(&token) {
            Ok(mut user) => {
                debug!("Session validation successful");
                if let Some(bid) = data.bot_id {
                    user = user.with_current_bot(bid);
                }
                return Ok(user);
            }
            Err(e) => {
                warn!("Session validation failed: {:?}", e);
                return Err(e);
            }
        }
    }

    if let Some(token) = data.query_token {
        debug!("Authenticating query token (length={})", token.len());

        // Browser WS clients may send the raw bearer token or the JWT itself.
        let token = token
            .strip_prefix(&config.bearer_prefix)
            .map(|s| s.to_string())
            .or_else(|| {
                if token.starts_with("Bearer ") {
                    None
                } else {
                    Some(token.clone())
                }
            });

        if let Some(token) = token {
            if is_jwt_format(&token) {
                match registry.authenticate_token(&token).await {
                    Ok(mut user) => {
                        debug!(
                            "Query-token JWT authentication successful for user: {}",
                            user.user_id
                        );
                        if let Some(bid) = data.bot_id {
                            user = user.with_current_bot(bid);
                        }
                        return Ok(user);
                    }
                    Err(e) => {
                        debug!(
                            "Query-token JWT authentication failed: {:?}, falling back to session validation",
                            e
                        );
                    }
                }
            }

            match validate_session_sync(&token) {
                Ok(mut u) => {
                    debug!("Query-token session validation successful");
                    if let Some(bid) = data.bot_id {
                        u = u.with_current_bot(bid);
                    }
                    return Ok(u);
                }
                Err(e) => {
                    warn!("Query-token session validation failed: {:?}", e);
                }
            }
        }
    }

    if let Some(sid) = data.session_id {
        let mut user = validate_session_sync(&sid)?;
        if let Some(bid) = data.bot_id {
            user = user.with_current_bot(bid);
        }
        return Ok(user);
    }

    if let Some(uid) = data.user_id_header {
        let mut user = AuthenticatedUser::new(uid, "header-user".to_string());
        if let Some(bid) = data.bot_id {
            user = user.with_current_bot(bid);
        }
        return Ok(user);
    }

    if !config.require_auth {
        return Ok(AuthenticatedUser::anonymous());
    }

    Err(AuthError::MissingToken)
}

pub async fn extract_user_with_providers(
    request: &axum::http::Request<Body>,
    config: &AuthConfig,
    registry: &AuthProviderRegistry,
) -> Result<AuthenticatedUser, AuthError> {
    let extracted = ExtractedAuthData::from_request(request, config);
    authenticate_with_extracted_data(extracted, config, registry).await
}

