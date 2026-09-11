//! OAuth2 token handling for mail accounts.
//!
//! Microsoft 365, Outlook.com and Gmail no longer accept a static password over
//! IMAP/SMTP, so those mailboxes authenticate with an OAuth2 access token
//! obtained from the provider. This module owns the provider endpoints, the
//! consent URL, the authorization-code exchange and the refresh flow.
//!
//! Client credentials are read from the environment:
//!
//! * `MAIL_OAUTH_MICROSOFT_CLIENT_ID` / `MAIL_OAUTH_MICROSOFT_CLIENT_SECRET`
//! * `MAIL_OAUTH_GOOGLE_CLIENT_ID` / `MAIL_OAUTH_GOOGLE_CLIENT_SECRET`
//!
//! The redirect URI must be registered with the provider and point back at the
//! callback route (`/api/email/oauth/callback`).

use base64::{engine::general_purpose, Engine as _};
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use serde::Deserialize;
use uuid::Uuid;

/// Access tokens are treated as expired this long before their stated expiry,
/// so a token that is about to lapse is refreshed instead of failing a sync.
const EXPIRY_SKEW_SECS: i64 = 60;

/// Providers that expose mail over OAuth2.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MailOAuthProvider {
    Microsoft,
    Google,
}

impl MailOAuthProvider {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Microsoft => "microsoft",
            Self::Google => "google",
        }
    }

    /// Parses the provider identifier stored on the account.
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.to_ascii_lowercase().as_str() {
            "microsoft" | "m365" | "outlook" | "office365" | "exchange" => Some(Self::Microsoft),
            "google" | "gmail" => Some(Self::Google),
            _ => None,
        }
    }

    fn authorize_url(self) -> &'static str {
        match self {
            Self::Microsoft => {
                "https://login.microsoftonline.com/common/oauth2/v2.0/authorize"
            }
            Self::Google => "https://accounts.google.com/o/oauth2/v2/auth",
        }
    }

    fn token_url(self) -> &'static str {
        match self {
            Self::Microsoft => "https://login.microsoftonline.com/common/oauth2/v2.0/token",
            Self::Google => "https://oauth2.googleapis.com/token",
        }
    }

    /// Mail scopes. Microsoft requires the legacy Outlook resource scopes for
    /// IMAP/SMTP, Google requires the full mail scope.
    fn scopes(self) -> &'static str {
        match self {
            Self::Microsoft => {
                "offline_access https://outlook.office.com/IMAP.AccessAsUser.All \
                 https://outlook.office.com/SMTP.Send"
            }
            Self::Google => "https://mail.google.com/",
        }
    }

    #[must_use]
    pub fn default_imap_server(self) -> &'static str {
        match self {
            Self::Microsoft => "outlook.office365.com",
            Self::Google => "imap.gmail.com",
        }
    }

    #[must_use]
    pub fn default_smtp_server(self) -> &'static str {
        match self {
            Self::Microsoft => "smtp.office365.com",
            Self::Google => "smtp.gmail.com",
        }
    }

    /// Reads the client credentials for this provider from the environment.
    #[must_use]
    pub fn credentials(self) -> Option<(String, String)> {
        let (id_key, secret_key) = match self {
            Self::Microsoft => (
                "MAIL_OAUTH_MICROSOFT_CLIENT_ID",
                "MAIL_OAUTH_MICROSOFT_CLIENT_SECRET",
            ),
            Self::Google => (
                "MAIL_OAUTH_GOOGLE_CLIENT_ID",
                "MAIL_OAUTH_GOOGLE_CLIENT_SECRET",
            ),
        };
        let client_id = std::env::var(id_key).ok()?;
        let client_secret = std::env::var(secret_key).ok()?;
        if client_id.is_empty() || client_secret.is_empty() {
            return None;
        }
        Some((client_id, client_secret))
    }
}

/// Tokens returned by an authorization-code exchange or a refresh.
#[derive(Debug, Clone)]
pub struct TokenSet {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_at: DateTime<Utc>,
}

impl TokenSet {
    /// Whether the access token is expired or close enough to expiry that it
    /// should be refreshed before use.
    #[must_use]
    pub fn needs_refresh(&self) -> bool {
        Utc::now() + Duration::seconds(EXPIRY_SKEW_SECS) >= self.expires_at
    }
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    #[serde(default)]
    access_token: Option<String>,
    #[serde(default)]
    refresh_token: Option<String>,
    #[serde(default)]
    expires_in: Option<i64>,
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    error_description: Option<String>,
}

impl TokenResponse {
    fn into_token_set(self) -> Result<TokenSet, String> {
        if let Some(error) = self.error {
            let detail = self.error_description.unwrap_or_default();
            return Err(format!("OAuth2 error: {error} {detail}").trim().to_string());
        }
        let access_token = self
            .access_token
            .ok_or_else(|| "OAuth2 response contained no access token".to_string())?;
        let lifetime = self.expires_in.unwrap_or(3600).max(60);
        Ok(TokenSet {
            access_token,
            refresh_token: self.refresh_token,
            expires_at: Utc::now() + Duration::seconds(lifetime),
        })
    }
}

/// Builds the provider consent URL the user is redirected to.
#[must_use]
pub fn consent_url(
    provider: MailOAuthProvider,
    client_id: &str,
    redirect_uri: &str,
    state: &str,
) -> String {
    let mut query = url::form_urlencoded::Serializer::new(String::new());
    query.append_pair("client_id", client_id);
    query.append_pair("response_type", "code");
    query.append_pair("redirect_uri", redirect_uri);
    query.append_pair("response_mode", "query");
    query.append_pair("scope", provider.scopes());
    query.append_pair("state", state);
    // Microsoft returns a refresh token only when offline access is requested
    // explicitly; Google uses the same parameter to force a consent prompt.
    query.append_pair("prompt", "consent");
    format!("{}?{}", provider.authorize_url(), query.finish())
}

/// Exchanges an authorization code for tokens.
pub async fn exchange_code(
    provider: MailOAuthProvider,
    client_id: &str,
    client_secret: &str,
    code: &str,
    redirect_uri: &str,
) -> Result<TokenSet, String> {
    let form = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("code", code),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
    ];
    post_token_request(provider.token_url(), &form).await
}

/// Exchanges a refresh token for a fresh access token.
pub async fn refresh_access_token(
    provider: MailOAuthProvider,
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<TokenSet, String> {
    let form = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("refresh_token", refresh_token),
        ("grant_type", "refresh_token"),
        ("scope", provider.scopes()),
    ];
    post_token_request(provider.token_url(), &form).await
}

/// Authentication mechanism an account uses against IMAP and SMTP.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MailMechanism {
    /// `LOGIN` / `AUTH PLAIN` with a password.
    Password,
    /// `AUTHENTICATE XOAUTH2` with a bearer token.
    Xoauth2,
}

/// Authentication material resolved for a mailbox.
#[derive(Debug, Clone)]
pub struct MailboxCredentials {
    pub username: String,
    /// Password or bearer token, depending on `mechanism`.
    pub secret: String,
    pub mechanism: MailMechanism,
}

impl MailboxCredentials {
    /// Whether the secret is a bearer token rather than a password.
    #[must_use]
    pub fn is_oauth2(&self) -> bool {
        matches!(self.mechanism, MailMechanism::Xoauth2)
    }
}

#[derive(Debug, QueryableByName)]
struct AccountAuthRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    username: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    password_encrypted: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    auth_mode: String,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    oauth_provider: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    refresh_token_encrypted: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Text>)]
    access_token_encrypted: Option<String>,
    #[diesel(sql_type = diesel::sql_types::Nullable<diesel::sql_types::Timestamptz>)]
    token_expires_at: Option<DateTime<Utc>>,
}

/// Loads the authentication material for an account.
///
/// Password accounts decrypt the stored password. OAuth2 accounts reuse the
/// stored access token while it is valid and otherwise exchange the refresh
/// token for a new one, persisting the result so the next use reuses it. Shared
/// by the mailbox sync worker and the SMTP send path.
pub fn resolve_credentials(
    conn: &mut diesel::PgConnection,
    account_id: Uuid,
) -> Result<MailboxCredentials, String> {
    let account = diesel::sql_query(
        "SELECT username, password_encrypted, auth_mode, oauth_provider, \
                refresh_token_encrypted, access_token_encrypted, token_expires_at \
         FROM user_email_accounts WHERE id = $1",
    )
    .bind::<diesel::sql_types::Uuid, _>(account_id)
    .get_result::<AccountAuthRow>(conn)
    .map_err(|e| format!("Email account not found: {e}"))?;

    if account.auth_mode != "oauth2" {
        return Ok(MailboxCredentials {
            username: account.username,
            secret: decode_secret(&account.password_encrypted)?,
            mechanism: MailMechanism::Password,
        });
    }

    let provider_name = account.oauth_provider.clone().unwrap_or_default();
    let provider = MailOAuthProvider::parse(&provider_name)
        .ok_or_else(|| format!("Unknown OAuth2 provider '{provider_name}'"))?;

    let stored_token = account
        .access_token_encrypted
        .as_deref()
        .map(decode_secret)
        .transpose()?;
    let token_is_usable = matches!(
        (stored_token.as_ref(), account.token_expires_at),
        (Some(_), Some(expiry)) if expiry > Utc::now() + Duration::seconds(EXPIRY_SKEW_SECS)
    );
    if let Some(token) = stored_token.filter(|_| token_is_usable) {
        return Ok(MailboxCredentials {
            username: account.username,
            secret: token,
            mechanism: MailMechanism::Xoauth2,
        });
    }

    let refresh_token = decode_secret(
        account
            .refresh_token_encrypted
            .as_deref()
            .ok_or_else(|| "OAuth2 account has no refresh token; reconnect the mailbox".to_string())?,
    )?;
    let (client_id, client_secret) = provider.credentials().ok_or_else(|| {
        format!(
            "No OAuth2 client credentials configured for {}",
            provider.as_str()
        )
    })?;

    let tokens = refresh_token_blocking(provider, &client_id, &client_secret, &refresh_token)?;
    persist_tokens(conn, account_id, &tokens)?;

    Ok(MailboxCredentials {
        username: account.username,
        secret: tokens.access_token,
        mechanism: MailMechanism::Xoauth2,
    })
}

/// Refreshes a token from a synchronous context, which is where the sync worker
/// and the send path run.
fn refresh_token_blocking(
    provider: MailOAuthProvider,
    client_id: &str,
    client_secret: &str,
    refresh_token: &str,
) -> Result<TokenSet, String> {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .map_err(|e| format!("Could not build a runtime for the token refresh: {e}"))?;
    runtime.block_on(refresh_access_token(
        provider,
        client_id,
        client_secret,
        refresh_token,
    ))
}

/// Stores a refreshed token set on the account row.
fn persist_tokens(
    conn: &mut diesel::PgConnection,
    account_id: Uuid,
    tokens: &TokenSet,
) -> Result<(), String> {
    let access_token = encode_secret(&tokens.access_token);
    let refresh_token = tokens.refresh_token.as_deref().map(encode_secret);

    diesel::sql_query(
        "UPDATE user_email_accounts SET access_token_encrypted = $1, token_expires_at = $2, \
         refresh_token_encrypted = COALESCE($3, refresh_token_encrypted) WHERE id = $4",
    )
    .bind::<diesel::sql_types::Text, _>(access_token)
    .bind::<diesel::sql_types::Timestamptz, _>(tokens.expires_at)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Text>, _>(refresh_token.as_deref())
    .bind::<diesel::sql_types::Uuid, _>(account_id)
    .execute(conn)
    .map_err(|e| format!("Failed to store the refreshed OAuth2 tokens: {e}"))?;
    Ok(())
}

/// Encodes a password or token for storage.
///
/// This matches the encoding already used for `password_encrypted`; moving both
/// to real encryption at rest is tracked separately, since changing the format
/// requires migrating every stored credential.
#[must_use]
pub fn encode_secret(value: &str) -> String {
    general_purpose::STANDARD.encode(value.as_bytes())
}

/// Decodes a stored password or token.
fn decode_secret(value: &str) -> Result<String, String> {
    let bytes = general_purpose::STANDARD
        .decode(value)
        .map_err(|e| format!("Stored credential could not be decoded: {e}"))?;
    String::from_utf8(bytes).map_err(|e| format!("Stored credential is not valid UTF-8: {e}"))
}

async fn post_token_request(url: &str, form: &[(&str, &str)]) -> Result<TokenSet, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("Could not build the OAuth2 HTTP client: {e}"))?;

    let response = client
        .post(url)
        .form(form)
        .send()
        .await
        .map_err(|e| format!("OAuth2 token request failed: {e}"))?;

    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| format!("OAuth2 token response could not be read: {e}"))?;

    let parsed: TokenResponse = serde_json::from_str(&body)
        .map_err(|e| format!("OAuth2 token response was not valid JSON ({status}): {e}"))?;

    parsed.into_token_set()
}
