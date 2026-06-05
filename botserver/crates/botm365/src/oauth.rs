//! Microsoft 365 OAuth2 flow types.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

/// Microsoft Graph scopes used by the connector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum M365Scope {
    /// Read user profile.
    UserRead,
    /// Read SharePoint sites.
    SitesReadAll,
    /// Read SharePoint lists.
    ListsReadAll,
    /// Read files in OneDrive / SharePoint.
    FilesReadAll,
    /// Read calendar events.
    CalendarsRead,
}

impl M365Scope {
    /// Returns the scope string (`offline_access …`).
    pub fn as_scope_string(scopes: &[Self]) -> String {
        scopes
            .iter()
            .map(|s| match s {
                Self::UserRead => "User.Read",
                Self::SitesReadAll => "Sites.Read.All",
                Self::ListsReadAll => "Lists.Read.All",
                Self::FilesReadAll => "Files.Read.All",
                Self::CalendarsRead => "Calendars.Read",
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// OAuth2 token returned by Microsoft.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct M365Token {
    /// Bearer access token.
    pub access_token: String,
    /// Refresh token (used to obtain a new access token silently).
    pub refresh_token: Option<String>,
    /// When the access token expires.
    pub expires_at: DateTime<Utc>,
    /// Scopes granted by the resource owner.
    pub scopes: Vec<M365Scope>,
    /// Tenant ID (Azure AD).
    pub tenant_id: String,
}

impl M365Token {
    /// Returns true if the token is expired or about to expire.
    pub fn is_expired(&self, now: DateTime<Utc>) -> bool {
        now + Duration::minutes(5) >= self.expires_at
    }
}

/// High-level OAuth2 authorization-code flow orchestrator.
///
/// The actual HTTP exchange lives behind an injected `http_post` callback
/// so the type can be unit-tested without a network.
#[derive(Debug, Clone)]
pub struct M365OAuthFlow {
    /// Microsoft Entra tenant ID.
    pub tenant_id: String,
    /// Application (client) ID.
    pub client_id: String,
    /// Client secret (or `None` for public clients with PKCE).
    pub client_secret: Option<String>,
    /// Redirect URI registered in Azure AD.
    pub redirect_uri: String,
}

impl M365OAuthFlow {
    /// Build the `/authorize` URL the user should be redirected to.
    pub fn authorize_url(&self, state: &str, scopes: &[M365Scope]) -> String {
        format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize\
?client_id={}&response_type=code&redirect_uri={}&response_mode=query\
&scope={}&state={}",
            self.tenant_id,
            urlencoding(&self.client_id),
            urlencoding(&self.redirect_uri),
            urlencoding(&M365Scope::as_scope_string(scopes)),
            urlencoding(state),
        )
    }

    /// Build the token-exchange request body (callers POST it to
    /// `/{tenant}/oauth2/v2.0/token`).
    pub fn token_exchange_body(&self, code: &str) -> String {
        let mut s = format!(
            "client_id={}&grant_type=authorization_code&code={}&redirect_uri={}",
            urlencoding(&self.client_id),
            urlencoding(code),
            urlencoding(&self.redirect_uri),
        );
        if let Some(secret) = &self.client_secret {
            s.push_str("&client_secret=");
            s.push_str(&urlencoding(secret));
        }
        s
    }
}

/// Errors raised by the OAuth flow.
#[derive(Debug, thiserror::Error, Serialize, Deserialize)]
pub enum M365Error {
    /// Microsoft returned an error response.
    #[error("microsoft oauth: {0}")]
    Oauth(String),
    /// Token refresh failed.
    #[error("refresh failed: {0}")]
    Refresh(String),
    /// Missing scope for the requested operation.
    #[error("missing scope: {0:?}")]
    MissingScope(M365Scope),
}

fn urlencoding(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            _ => {
                out.push_str(&format!("%{:02X}", b));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_string_joins() {
        let s = M365Scope::as_scope_string(&[M365Scope::UserRead, M365Scope::FilesReadAll]);
        assert_eq!(s, "User.Read Files.Read.All");
    }

    #[test]
    fn authorize_url_includes_state() {
        let f = M365OAuthFlow {
            tenant_id: "tid".to_string(),
            client_id: "cid".to_string(),
            client_secret: None,
            redirect_uri: "https://app/cb".to_string(),
        };
        let url = f.authorize_url("xyz", &[M365Scope::SitesReadAll]);
        assert!(url.contains("state=xyz"));
        assert!(url.contains("client_id=cid"));
    }

    #[test]
    fn token_exchange_body_includes_secret() {
        let f = M365OAuthFlow {
            tenant_id: "tid".to_string(),
            client_id: "cid".to_string(),
            client_secret: Some("s".to_string()),
            redirect_uri: "https://app/cb".to_string(),
        };
        let body = f.token_exchange_body("abc");
        assert!(body.contains("client_secret=s"));
    }
}
