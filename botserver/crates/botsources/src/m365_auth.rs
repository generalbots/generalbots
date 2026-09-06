use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum OAuthFlow {
    AuthorizationCode,
    ClientCredentials,
    DeviceCode,
    OnBehalfOf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct M365Credentials {
    pub tenant_id: String,
    pub client_id: String,
    pub client_secret: Option<String>,
    pub redirect_uri: Option<String>,
    pub scopes: Vec<String>,
    pub authority: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct M365Token {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
    pub token_type: String,
    pub expires_at: DateTime<Utc>,
    pub scope: String,
    pub issued_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationUrlRequest {
    pub state: String,
    pub code_challenge: Option<String>,
    pub code_challenge_method: Option<String>,
    pub prompt: Option<String>,
    pub login_hint: Option<String>,
}

pub struct M365OAuthClient {
    pub credentials: M365Credentials,
    pub http_client: reqwest::blocking::Client,
}

impl M365OAuthClient {
    pub fn new(credentials: M365Credentials) -> Self {
        Self {
            credentials,
            http_client: reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_default(),
        }
    }

    pub fn authorization_url(&self, req: AuthorizationUrlRequest) -> String {
        let scopes = self.credentials.scopes.join(" ");
        let mut url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize?\
             client_id={}&response_type=code&redirect_uri={}&response_mode=query&\
             scope={}&state={}",
            self.credentials.tenant_id,
            urlencoding(&self.credentials.client_id),
            urlencoding(self.credentials.redirect_uri.as_deref().unwrap_or("")),
            urlencoding(&scopes),
            urlencoding(&req.state),
        );
        if let Some(cc) = &req.code_challenge {
            url.push_str(&format!("&code_challenge={}", urlencoding(cc)));
        }
        if let Some(method) = &req.code_challenge_method {
            url.push_str(&format!("&code_challenge_method={}", urlencoding(method)));
        }
        if let Some(prompt) = &req.prompt {
            url.push_str(&format!("&prompt={}", urlencoding(prompt)));
        }
        if let Some(lh) = &req.login_hint {
            url.push_str(&format!("&login_hint={}", urlencoding(lh)));
        }
        url
    }

    /// Keeps the host pinned to the Microsoft login endpoint regardless of
    /// tenant input, so credentials cannot be redirected to another server.
    fn token_endpoint(tenant: &str) -> String {
        let encoded: String = tenant
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '.' { c } else { '_' })
            .collect();
        format!("https://login.microsoftonline.com/{encoded}/oauth2/v2.0/token")
    }

    pub fn exchange_code(&self, code: &str) -> Result<M365Token, String> {
        let url = Self::token_endpoint(&self.credentials.tenant_id);
        let mut form = vec![
            ("client_id", self.credentials.client_id.clone()),
            ("scope", self.credentials.scopes.join(" ")),
            ("code", code.to_string()),
            ("redirect_uri", self.credentials.redirect_uri.clone().unwrap_or_default()),
            ("grant_type", "authorization_code".to_string()),
        ];
        if let Some(secret) = &self.credentials.client_secret {
            form.push(("client_secret", secret.clone()));
        }
        let resp = self
            .http_client
            .post(&url)
            .form(&form)
            .send()
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }
        let body: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
        Self::parse_token(body)
    }

    pub fn refresh(&self, refresh_token: &str) -> Result<M365Token, String> {
        let url = Self::token_endpoint(&self.credentials.tenant_id);
        let mut form = vec![
            ("client_id", self.credentials.client_id.clone()),
            ("scope", self.credentials.scopes.join(" ")),
            ("refresh_token", refresh_token.to_string()),
            ("grant_type", "refresh_token".to_string()),
        ];
        if let Some(secret) = &self.credentials.client_secret {
            form.push(("client_secret", secret.clone()));
        }
        let resp = self
            .http_client
            .post(&url)
            .form(&form)
            .send()
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }
        let body: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
        Self::parse_token(body)
    }

    pub fn client_credentials(&self) -> Result<M365Token, String> {
        let url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            self.credentials.tenant_id
        );
        let secret = self
            .credentials
            .client_secret
            .clone()
            .ok_or_else(|| "client_secret required for client_credentials flow".to_string())?;
        let form = vec![
            ("client_id", self.credentials.client_id.clone()),
            ("client_secret", secret),
            ("scope", self.credentials.scopes.join(" ")),
            ("grant_type", "client_credentials".to_string()),
        ];
        let resp = self
            .http_client
            .post(&url)
            .form(&form)
            .send()
            .map_err(|e| e.to_string())?;
        if !resp.status().is_success() {
            return Err(format!("HTTP {}", resp.status()));
        }
        let body: serde_json::Value = resp.json().map_err(|e| e.to_string())?;
        Self::parse_token(body)
    }

    fn parse_token(body: serde_json::Value) -> Result<M365Token, String> {
        let access = body
            .get("access_token")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "missing access_token".to_string())?;
        let expires_in = body
            .get("expires_in")
            .and_then(|v| v.as_i64())
            .unwrap_or(3600);
        let now = Utc::now();
        Ok(M365Token {
            access_token: access.to_string(),
            refresh_token: body
                .get("refresh_token")
                .and_then(|v| v.as_str())
                .map(String::from),
            id_token: body
                .get("id_token")
                .and_then(|v| v.as_str())
                .map(String::from),
            token_type: body
                .get("token_type")
                .and_then(|v| v.as_str())
                .unwrap_or("Bearer")
                .to_string(),
            expires_at: now + Duration::seconds(expires_in),
            scope: body
                .get("scope")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string(),
            issued_at: now,
        })
    }

    pub fn is_expired(token: &M365Token) -> bool {
        Utc::now() >= token.expires_at - Duration::seconds(60)
    }

    pub fn ensure_valid(&self, token: &M365Token) -> Result<M365Token, String> {
        if Self::is_expired(token) {
            if let Some(rt) = &token.refresh_token {
                return self.refresh(rt);
            }
            return Err("Token expired and no refresh token available".to_string());
        }
        Ok(token.clone())
    }
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
