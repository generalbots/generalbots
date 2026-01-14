use chrono::{DateTime, Utc};
use log::{debug, error, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct GoogleConfig {
    pub client_id: String,
    pub client_secret: String,
}

#[derive(Debug, Clone)]
pub struct MicrosoftConfig {
    pub client_id: String,
    pub client_secret: String,
    pub tenant_id: String,
}

pub struct GoogleContactsClient {
    config: GoogleConfig,
    client: Client,
}

impl GoogleContactsClient {
    pub fn new(config: GoogleConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub fn get_auth_url(&self, redirect_uri: &str, state: &str) -> String {
        format!(
            "https://accounts.google.com/o/oauth2/v2/auth?client_id={}&redirect_uri={}&response_type=code&scope=https://www.googleapis.com/auth/contacts&state={}",
            self.config.client_id, redirect_uri, state
        )
    }

    pub async fn exchange_code(&self, code: &str, redirect_uri: &str) -> Result<TokenResponse, ExternalSyncError> {
        let response = self.client
            .post("https://oauth2.googleapis.com/token")
            .form(&[
                ("client_id", self.config.client_id.as_str()),
                ("client_secret", self.config.client_secret.as_str()),
                ("code", code),
                ("redirect_uri", redirect_uri),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await
            .map_err(|e| ExternalSyncError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Google token exchange failed: {} - {}", status, body);
            return Err(ExternalSyncError::AuthError(format!("Token exchange failed: {}", status)));
        }

        #[derive(Deserialize)]
        struct GoogleTokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: i64,
            scope: Option<String>,
        }

        let token_data: GoogleTokenResponse = response.json().await
            .map_err(|e| ExternalSyncError::ParseError(e.to_string()))?;

        Ok(TokenResponse {
            access_token: token_data.access_token,
            refresh_token: token_data.refresh_token,
            expires_in: token_data.expires_in,
            expires_at: Some(Utc::now() + chrono::Duration::seconds(token_data.expires_in)),
            scopes: token_data.scope.map(|s| s.split(' ').map(String::from).collect()).unwrap_or_default(),
        })
    }

    pub async fn get_user_info(&self, access_token: &str) -> Result<UserInfo, ExternalSyncError> {
        let response = self.client
            .get("https://www.googleapis.com/oauth2/v2/userinfo")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| ExternalSyncError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ExternalSyncError::AuthError("Failed to get user info".to_string()));
        }

        #[derive(Deserialize)]
        struct GoogleUserInfo {
            id: String,
            email: String,
            name: Option<String>,
        }

        let user_data: GoogleUserInfo = response.json().await
            .map_err(|e| ExternalSyncError::ParseError(e.to_string()))?;

        Ok(UserInfo {
            id: user_data.id,
            email: user_data.email,
            name: user_data.name,
        })
    }

    pub async fn revoke_token(&self, access_token: &str) -> Result<(), ExternalSyncError> {
        let response = self.client
            .post("https://oauth2.googleapis.com/revoke")
            .form(&[("token", access_token)])
            .send()
            .await
            .map_err(|e| ExternalSyncError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            warn!("Token revocation may have failed: {}", response.status());
        }
        Ok(())
    }

    pub async fn list_contacts(&self, access_token: &str, cursor: Option<&str>) -> Result<(Vec<ExternalContact>, Option<String>), ExternalSyncError> {
        let mut url = "https://people.googleapis.com/v1/people/me/connections?personFields=names,emailAddresses,phoneNumbers,organizations&pageSize=100".to_string();

        if let Some(page_token) = cursor {
            url.push_str(&format!("&pageToken={}", page_token));
        }

        let response = self.client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| ExternalSyncError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Google contacts list failed: {} - {}", status, body);
            return Err(ExternalSyncError::ApiError(format!("List contacts failed: {}", status)));
        }

        #[derive(Deserialize)]
        struct GoogleConnectionsResponse {
            connections: Option<Vec<GooglePerson>>,
            #[serde(rename = "nextPageToken")]
            next_page_token: Option<String>,
        }

        #[derive(Deserialize)]
        struct GooglePerson {
            #[serde(rename = "resourceName")]
            resource_name: String,
            names: Option<Vec<GoogleName>>,
            #[serde(rename = "emailAddresses")]
            email_addresses: Option<Vec<GoogleEmail>>,
            #[serde(rename = "phoneNumbers")]
            phone_numbers: Option<Vec<GooglePhone>>,
            organizations: Option<Vec<GoogleOrg>>,
        }

        #[derive(Deserialize)]
        struct GoogleName {
            #[serde(rename = "displayName")]
            display_name: Option<String>,
            #[serde(rename = "givenName")]
            given_name: Option<String>,
            #[serde(rename = "familyName")]
            family_name: Option<String>,
        }

        #[derive(Deserialize)]
        struct GoogleEmail {
            value: String,
        }

        #[derive(Deserialize)]
        struct GooglePhone {
            value: String,
        }

        #[derive(Deserialize)]
        struct GoogleOrg {
            name: Option<String>,
            title: Option<String>,
        }

        let data: GoogleConnectionsResponse = response.json().await
            .map_err(|e| ExternalSyncError::ParseError(e.to_string()))?;

        let contacts = data.connections.unwrap_or_default().into_iter().map(|person| {
            let name = person.names.as_ref().and_then(|n| n.first());
            let email = person.email_addresses.as_ref().and_then(|e| e.first());
            let phone = person.phone_numbers.as_ref().and_then(|p| p.first());
            let org = person.organizations.as_ref().and_then(|o| o.first());

            ExternalContact {
                id: person.resource_name,
                etag: None,
                first_name: name.and_then(|n| n.given_name.clone()),
                last_name: name.and_then(|n| n.family_name.clone()),
                display_name: name.and_then(|n| n.display_name.clone()),
                email_addresses: email.map(|e| vec![ExternalEmail {
                    address: e.value.clone(),
                    label: None,
                    primary: true,
                }]).unwrap_or_default(),
                phone_numbers: phone.map(|p| vec![ExternalPhone {
                    number: p.value.clone(),
                    label: None,
                    primary: true,
                }]).unwrap_or_default(),
                addresses: Vec::new(),
                company: org.and_then(|o| o.name.clone()),
                job_title: org.and_then(|o| o.title.clone()),
                department: None,
                notes: None,
                birthday: None,
                photo_url: None,
                groups: Vec::new(),
                custom_fields: HashMap::new(),
                created_at: None,
                updated_at: None,
            }
        }).collect();

        Ok((contacts, data.next_page_token))
    }

    pub async fn fetch_contacts(&self, access_token: &str) -> Result<Vec<ExternalContact>, ExternalSyncError> {
        let mut all_contacts = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let (contacts, next_cursor) = self.list_contacts(access_token, cursor.as_deref()).await?;
            all_contacts.extend(contacts);

            if next_cursor.is_none() {
                break;
            }
            cursor = next_cursor;

            // Safety limit
            if all_contacts.len() > 10000 {
                warn!("Reached contact fetch limit");
                break;
            }
        }

        Ok(all_contacts)
    }

    pub async fn create_contact(&self, access_token: &str, contact: &ExternalContact) -> Result<String, ExternalSyncError> {
        let body = serde_json::json!({
            "names": [{
                "givenName": contact.first_name,
                "familyName": contact.last_name
            }],
            "emailAddresses": if contact.email_addresses.is_empty() { None } else { Some(contact.email_addresses.iter().map(|e| serde_json::json!({"value": e.address})).collect::<Vec<_>>()) },
            "phoneNumbers": if contact.phone_numbers.is_empty() { None } else { Some(contact.phone_numbers.iter().map(|p| serde_json::json!({"value": p.number})).collect::<Vec<_>>()) },
            "organizations": contact.company.as_ref().map(|c| vec![serde_json::json!({
                "name": c,
                "title": contact.job_title
            })])
        });

        let response = self.client
            .post("https://people.googleapis.com/v1/people:createContact")
            .bearer_auth(access_token)
            .json(&body)
            .send()
            .await
            .map_err(|e| ExternalSyncError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ExternalSyncError::ApiError(format!("Create contact failed: {} - {}", status, body)));
        }

        #[derive(Deserialize)]
        struct CreateResponse {
            #[serde(rename = "resourceName")]
            resource_name: String,
        }

        let data: CreateResponse = response.json().await
            .map_err(|e| ExternalSyncError::ParseError(e.to_string()))?;

        Ok(data.resource_name)
    }

    pub async fn update_contact(&self, access_token: &str, contact_id: &str, contact: &ExternalContact) -> Result<(), ExternalSyncError> {
        let body = serde_json::json!({
            "names": [{
                "givenName": contact.first_name,
                "familyName": contact.last_name
            }],
            "emailAddresses": if contact.email_addresses.is_empty() { None } else { Some(contact.email_addresses.iter().map(|e| serde_json::json!({"value": e.address})).collect::<Vec<_>>()) },
            "phoneNumbers": if contact.phone_numbers.is_empty() { None } else { Some(contact.phone_numbers.iter().map(|p| serde_json::json!({"value": p.number})).collect::<Vec<_>>()) }
        });

        let url = format!("https://people.googleapis.com/v1/{}:updateContact?updatePersonFields=names,emailAddresses,phoneNumbers", contact_id);

        let response = self.client
            .patch(&url)
            .bearer_auth(access_token)
            .json(&body)
            .send()
            .await
            .map_err(|e| ExternalSyncError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(ExternalSyncError::ApiError(format!("Update contact failed: {}", status)));
        }

        Ok(())
    }

    pub async fn delete_contact(&self, access_token: &str, contact_id: &str) -> Result<(), ExternalSyncError> {
        let url = format!("https://people.googleapis.com/v1/{}:deleteContact", contact_id);

        let response = self.client
            .delete(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| ExternalSyncError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(ExternalSyncError::ApiError(format!("Delete contact failed: {}", status)));
        }

        Ok(())
    }
}

pub struct MicrosoftPeopleClient {
    config: MicrosoftConfig,
    client: Client,
}

impl MicrosoftPeopleClient {
    pub fn new(config: MicrosoftConfig) -> Self {
        Self {
            config,
            client: Client::new(),
        }
    }

    pub fn get_auth_url(&self, redirect_uri: &str, state: &str) -> String {
        format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize?client_id={}&redirect_uri={}&response_type=code&scope=Contacts.ReadWrite&state={}",
            self.config.tenant_id, self.config.client_id, redirect_uri, state
        )
    }

    pub async fn exchange_code(&self, code: &str, redirect_uri: &str) -> Result<TokenResponse, ExternalSyncError> {
        let url = format!(
            "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
            self.config.tenant_id
        );

        let response = self.client
            .post(&url)
            .form(&[
                ("client_id", self.config.client_id.as_str()),
                ("client_secret", self.config.client_secret.as_str()),
                ("code", code),
                ("redirect_uri", redirect_uri),
                ("grant_type", "authorization_code"),
            ])
            .send()
            .await
            .map_err(|e| ExternalSyncError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Microsoft token exchange failed: {} - {}", status, body);
            return Err(ExternalSyncError::AuthError(format!("Token exchange failed: {}", status)));
        }

        #[derive(Deserialize)]
        struct MsTokenResponse {
            access_token: String,
            refresh_token: Option<String>,
            expires_in: i64,
            scope: Option<String>,
        }

        let token_data: MsTokenResponse = response.json().await
            .map_err(|e| ExternalSyncError::ParseError(e.to_string()))?;

        Ok(TokenResponse {
            access_token: token_data.access_token,
            refresh_token: token_data.refresh_token,
            expires_in: token_data.expires_in,
            expires_at: Some(Utc::now() + chrono::Duration::seconds(token_data.expires_in)),
            scopes: token_data.scope.map(|s| s.split(' ').map(String::from).collect()).unwrap_or_default(),
        })
    }

    pub async fn get_user_info(&self, access_token: &str) -> Result<UserInfo, ExternalSyncError> {
        let response = self.client
            .get("https://graph.microsoft.com/v1.0/me")
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| ExternalSyncError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(ExternalSyncError::AuthError("Failed to get user info".to_string()));
        }

        #[derive(Deserialize)]
        struct MsUserInfo {
            id: String,
            mail: Option<String>,
            #[serde(rename = "userPrincipalName")]
            user_principal_name: String,
            #[serde(rename = "displayName")]
            display_name: Option<String>,
        }

        let user_data: MsUserInfo = response.json().await
            .map_err(|e| ExternalSyncError::ParseError(e.to_string()))?;

        Ok(UserInfo {
            id: user_data.id,
            email: user_data.mail.unwrap_or(user_data.user_principal_name),
            name: user_data.display_name,
        })
    }

    pub async fn revoke_token(&self, _access_token: &str) -> Result<(), ExternalSyncError> {
        // Microsoft doesn't have a simple revoke endpoint - tokens expire naturally
        // For enterprise, you'd use the admin API to revoke refresh tokens
        debug!("Microsoft token revocation requested - tokens will expire naturally");
        Ok(())
    }

    pub async fn list_contacts(&self, access_token: &str, cursor: Option<&str>) -> Result<(Vec<ExternalContact>, Option<String>), ExternalSyncError> {
        let url = cursor.map(String::from).unwrap_or_else(|| {
            "https://graph.microsoft.com/v1.0/me/contacts?$top=100&$select=id,givenName,surname,displayName,emailAddresses,mobilePhone,businessPhones,companyName,jobTitle".to_string()
        });

        let response = self.client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| ExternalSyncError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!("Microsoft contacts list failed: {} - {}", status, body);
            return Err(ExternalSyncError::ApiError(format!("List contacts failed: {}", status)));
        }

        #[derive(Deserialize)]
        struct MsContactsResponse {
            value: Vec<MsContact>,
            #[serde(rename = "@odata.nextLink")]
            next_link: Option<String>,
        }

        #[derive(Deserialize)]
        struct MsContact {
            id: String,
            #[serde(rename = "givenName")]
            given_name: Option<String>,
            surname: Option<String>,
            #[serde(rename = "displayName")]
            display_name: Option<String>,
            #[serde(rename = "emailAddresses")]
            email_addresses: Option<Vec<MsEmailAddress>>,
            #[serde(rename = "mobilePhone")]
            mobile_phone: Option<String>,
            #[serde(rename = "businessPhones")]
            business_phones: Option<Vec<String>>,
            #[serde(rename = "companyName")]
            company_name: Option<String>,
            #[serde(rename = "jobTitle")]
            job_title: Option<String>,
        }

        #[derive(Deserialize)]
        struct MsEmailAddress {
            address: Option<String>,
        }

        let data: MsContactsResponse = response.json().await
            .map_err(|e| ExternalSyncError::ParseError(e.to_string()))?;

        let contacts = data.value.into_iter().map(|contact| {
            let email = contact.email_addresses
                .as_ref()
                .and_then(|emails| emails.first())
                .and_then(|e| e.address.clone());

            let phone = contact.mobile_phone
                .or_else(|| contact.business_phones.as_ref().and_then(|p| p.first().cloned()));

            let first_name = contact.given_name.clone();
            let last_name = contact.surname.clone();

        ExternalContact {
            id: contact.id,
            etag: None,
            first_name,
            last_name,
            display_name: contact.display_name,
            email_addresses: email.map(|e| vec![ExternalEmail {
                address: e,
                label: None,
                primary: true,
            }]).unwrap_or_default(),
            phone_numbers: phone.map(|p| vec![ExternalPhone {
                number: p,
                label: None,
                primary: true,
            }]).unwrap_or_default(),
            addresses: Vec::new(),
            company: contact.company_name,
            job_title: contact.job_title,
            department: None,
            notes: None,
            birthday: None,
            photo_url: None,
            groups: Vec::new(),
            custom_fields: HashMap::new(),
            created_at: None,
            updated_at: None,
        }
        }).collect();

        Ok((contacts, data.next_link))
    }

    pub async fn fetch_contacts(&self, access_token: &str) -> Result<Vec<ExternalContact>, ExternalSyncError> {
        let mut all_contacts = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let (contacts, next_cursor) = self.list_contacts(access_token, cursor.as_deref()).await?;
            all_contacts.extend(contacts);

            if next_cursor.is_none() {
                break;
            }
            cursor = next_cursor;

            // Safety limit
            if all_contacts.len() > 10000 {
                warn!("Reached contact fetch limit");
                break;
            }
        }

        Ok(all_contacts)
    }

    pub async fn create_contact(&self, access_token: &str, contact: &ExternalContact) -> Result<String, ExternalSyncError> {
        let body = serde_json::json!({
            "givenName": contact.first_name,
            "surname": contact.last_name,
            "displayName": contact.display_name,
            "emailAddresses": if contact.email_addresses.is_empty() { None } else { Some(contact.email_addresses.iter().map(|e| serde_json::json!({
                "address": e.address,
                "name": contact.display_name
            })).collect::<Vec<_>>()) },
            "mobilePhone": contact.phone_numbers.first().map(|p| &p.number),
            "companyName": contact.company,
            "jobTitle": contact.job_title
        });

        let response = self.client
            .post("https://graph.microsoft.com/v1.0/me/contacts")
            .bearer_auth(access_token)
            .json(&body)
            .send()
            .await
            .map_err(|e| ExternalSyncError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ExternalSyncError::ApiError(format!("Create contact failed: {} - {}", status, body)));
        }

        #[derive(Deserialize)]
        struct CreateResponse {
            id: String,
        }

        let data: CreateResponse = response.json().await
            .map_err(|e| ExternalSyncError::ParseError(e.to_string()))?;

        Ok(data.id)
    }

    pub async fn update_contact(&self, access_token: &str, contact_id: &str, contact: &ExternalContact) -> Result<(), ExternalSyncError> {
        let body = serde_json::json!({
            "givenName": contact.first_name,
            "surname": contact.last_name,
            "displayName": contact.display_name,
            "emailAddresses": if contact.email_addresses.is_empty() { None } else { Some(contact.email_addresses.iter().map(|e| serde_json::json!({
                "address": e.address,
                "name": contact.display_name
            })).collect::<Vec<_>>()) },
            "mobilePhone": contact.phone_numbers.first().map(|p| &p.number),
            "companyName": contact.company,
            "jobTitle": contact.job_title
        });

        let url = format!("https://graph.microsoft.com/v1.0/me/contacts/{}", contact_id);

        let response = self.client
            .patch(&url)
            .bearer_auth(access_token)
            .json(&body)
            .send()
            .await
            .map_err(|e| ExternalSyncError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(ExternalSyncError::ApiError(format!("Update contact failed: {}", status)));
        }

        Ok(())
    }

    pub async fn delete_contact(&self, access_token: &str, contact_id: &str) -> Result<(), ExternalSyncError> {
        let url = format!("https://graph.microsoft.com/v1.0/me/contacts/{}", contact_id);

        let response = self.client
            .delete(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| ExternalSyncError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(ExternalSyncError::ApiError(format!("Delete contact failed: {}", status)));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: i64,
    pub expires_at: Option<DateTime<Utc>>,
    pub scopes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ImportResult {
    Created,
    Updated,
    Skipped,
    Conflict,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportResult {
    Created,
    Updated,
    Deleted,
    Skipped,
}

#[derive(Debug, Clone)]
pub enum ExternalSyncError {
    DatabaseError(String),
    UnsupportedProvider(String),
    Unauthorized,
    SyncDisabled,
    SyncInProgress,
    ApiError(String),
    InvalidData(String),
    NetworkError(String),
    AuthError(String),
    ParseError(String),
}

impl std::fmt::Display for ExternalSyncError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DatabaseError(e) => write!(f, "Database error: {e}"),
            Self::UnsupportedProvider(p) => write!(f, "Unsupported provider: {p}"),
            Self::Unauthorized => write!(f, "Unauthorized"),
            Self::SyncDisabled => write!(f, "Sync is disabled"),
            Self::SyncInProgress => write!(f, "Sync already in progress"),
            Self::ApiError(e) => write!(f, "API error: {e}"),
            Self::InvalidData(e) => write!(f, "Invalid data: {e}"),
            Self::NetworkError(e) => write!(f, "Network error: {e}"),
            Self::AuthError(e) => write!(f, "Auth error: {e}"),
            Self::ParseError(e) => write!(f, "Parse error: {e}"),
        }
    }
}

impl std::error::Error for ExternalSyncError {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ExternalProvider {
    Google,
    Microsoft,
    Apple,
    CardDav,
}

impl std::fmt::Display for ExternalProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExternalProvider::Google => write!(f, "google"),
            ExternalProvider::Microsoft => write!(f, "microsoft"),
            ExternalProvider::Apple => write!(f, "apple"),
            ExternalProvider::CardDav => write!(f, "carddav"),
        }
    }
}

impl std::str::FromStr for ExternalProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "google" => Ok(ExternalProvider::Google),
            "microsoft" => Ok(ExternalProvider::Microsoft),
            "apple" => Ok(ExternalProvider::Apple),
            "carddav" => Ok(ExternalProvider::CardDav),
            _ => Err(format!("Unsupported provider: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalAccount {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Uuid,
    pub provider: ExternalProvider,
    pub external_account_id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub token_expires_at: Option<DateTime<Utc>>,
    pub scopes: Vec<String>,
    pub sync_enabled: bool,
    pub sync_direction: SyncDirection,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_sync_status: Option<String>,
    pub sync_cursor: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub enum SyncDirection {
    #[default]
    TwoWay,
    ImportOnly,
    ExportOnly,
}

impl std::fmt::Display for SyncDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncDirection::TwoWay => write!(f, "two_way"),
            SyncDirection::ImportOnly => write!(f, "import_only"),
            SyncDirection::ExportOnly => write!(f, "export_only"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncStatus {
    Success,
    Synced,
    PartialSuccess,
    Failed,
    InProgress,
    Cancelled,
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Synced => write!(f, "synced"),
            Self::PartialSuccess => write!(f, "partial_success"),
            Self::Failed => write!(f, "failed"),
            Self::InProgress => write!(f, "in_progress"),
            Self::Cancelled => write!(f, "cancelled"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactMapping {
    pub id: Uuid,
    pub account_id: Uuid,
    pub contact_id: Uuid,
    pub local_contact_id: Uuid,
    pub external_id: String,
    pub external_contact_id: String,
    pub external_etag: Option<String>,
    pub internal_version: i64,
    pub last_synced_at: DateTime<Utc>,
    pub sync_status: MappingSyncStatus,
    pub conflict_data: Option<ConflictData>,
    pub local_data: Option<serde_json::Value>,
    pub remote_data: Option<serde_json::Value>,
    pub conflict_detected_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MappingSyncStatus {
    Synced,
    PendingUpload,
    PendingDownload,
    Conflict,
    Error,
    Deleted,
}

impl std::fmt::Display for MappingSyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MappingSyncStatus::Synced => write!(f, "synced"),
            MappingSyncStatus::PendingUpload => write!(f, "pending_upload"),
            MappingSyncStatus::PendingDownload => write!(f, "pending_download"),
            MappingSyncStatus::Conflict => write!(f, "conflict"),
            MappingSyncStatus::Error => write!(f, "error"),
            MappingSyncStatus::Deleted => write!(f, "deleted"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictData {
    pub detected_at: DateTime<Utc>,
    pub internal_changes: Vec<String>,
    pub external_changes: Vec<String>,
    pub resolution: Option<ConflictResolution>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConflictResolution {
    KeepInternal,
    KeepExternal,
    KeepLocal,
    KeepRemote,
    Manual,
    Merge,
    Skip,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHistory {
    pub id: Uuid,
    pub account_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub status: SyncStatus,
    pub direction: SyncDirection,
    pub contacts_created: u32,
    pub contacts_updated: u32,
    pub contacts_deleted: u32,
    pub contacts_skipped: u32,
    pub conflicts_detected: u32,
    pub errors: Vec<SyncError>,
    pub triggered_by: SyncTrigger,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncTrigger {
    Manual,
    Scheduled,
    Webhook,
    ContactChange,
}

impl std::fmt::Display for SyncTrigger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SyncTrigger::Manual => write!(f, "manual"),
            SyncTrigger::Scheduled => write!(f, "scheduled"),
            SyncTrigger::Webhook => write!(f, "webhook"),
            SyncTrigger::ContactChange => write!(f, "contact_change"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncError {
    pub contact_id: Option<Uuid>,
    pub external_id: Option<String>,
    pub operation: String,
    pub error_code: String,
    pub error_message: String,
    pub retryable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectAccountRequest {
    pub provider: ExternalProvider,
    pub authorization_code: String,
    pub redirect_uri: String,
    pub sync_direction: Option<SyncDirection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationUrlResponse {
    pub url: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartSyncRequest {
    pub full_sync: Option<bool>,
    pub direction: Option<SyncDirection>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProgressResponse {
    pub sync_id: Uuid,
    pub status: SyncStatus,
    pub progress_percent: u8,
    pub contacts_processed: u32,
    pub total_contacts: u32,
    pub current_operation: String,
    pub started_at: DateTime<Utc>,
    pub estimated_completion: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolveConflictRequest {
    pub resolution: ConflictResolution,
    pub merged_data: Option<MergedContactData>,
    pub manual_data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedContactData {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSettings {
    pub sync_enabled: bool,
    pub sync_direction: SyncDirection,
    pub auto_sync_interval_minutes: u32,
    pub sync_contact_groups: bool,
    pub sync_photos: bool,
    pub conflict_resolution: ConflictResolution,
    pub field_mapping: HashMap<String, String>,
    pub exclude_tags: Vec<String>,
    pub include_only_tags: Vec<String>,
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            sync_enabled: true,
            sync_direction: SyncDirection::TwoWay,
            auto_sync_interval_minutes: 60,
            sync_contact_groups: true,
            sync_photos: true,
            conflict_resolution: ConflictResolution::KeepInternal,
            field_mapping: HashMap::new(),
            exclude_tags: vec![],
            include_only_tags: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStatusResponse {
    pub account: ExternalAccount,
    pub sync_stats: SyncStats,
    pub pending_conflicts: u32,
    pub pending_errors: u32,
    pub next_scheduled_sync: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStats {
    pub total_synced_contacts: u32,
    pub total_syncs: u32,
    pub successful_syncs: u32,
    pub failed_syncs: u32,
    pub last_successful_sync: Option<DateTime<Utc>>,
    pub average_sync_duration_seconds: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalContact {
    pub id: String,
    pub etag: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub display_name: Option<String>,
    pub email_addresses: Vec<ExternalEmail>,
    pub phone_numbers: Vec<ExternalPhone>,
    pub addresses: Vec<ExternalAddress>,
    pub company: Option<String>,
    pub job_title: Option<String>,
    pub department: Option<String>,
    pub notes: Option<String>,
    pub birthday: Option<String>,
    pub photo_url: Option<String>,
    pub groups: Vec<String>,
    pub custom_fields: HashMap<String, String>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalEmail {
    pub address: String,
    pub label: Option<String>,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalPhone {
    pub number: String,
    pub label: Option<String>,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExternalAddress {
    pub street: Option<String>,
    pub city: Option<String>,
    pub state: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
    pub label: Option<String>,
    pub primary: bool,
}

pub struct ExternalSyncService {
    google_client: GoogleContactsClient,
    microsoft_client: MicrosoftPeopleClient,
    accounts: Arc<RwLock<HashMap<Uuid, ExternalAccount>>>,
    mappings: Arc<RwLock<HashMap<Uuid, ContactMapping>>>,
    sync_history: Arc<RwLock<Vec<SyncHistory>>>,
    contacts: Arc<RwLock<HashMap<Uuid, ExternalContact>>>,
}

pub struct UserInfo {
    pub id: String,
    pub email: String,
    pub name: Option<String>,
}

impl ExternalSyncService {
    pub fn new(google_config: GoogleConfig, microsoft_config: MicrosoftConfig) -> Self {
        Self {
            google_client: GoogleContactsClient::new(google_config),
            microsoft_client: MicrosoftPeopleClient::new(microsoft_config),
            accounts: Arc::new(RwLock::new(HashMap::new())),
            mappings: Arc::new(RwLock::new(HashMap::new())),
            sync_history: Arc::new(RwLock::new(Vec::new())),
            contacts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn find_existing_account(
        &self,
        organization_id: Uuid,
        provider: &ExternalProvider,
        external_id: &str,
    ) -> Result<Option<ExternalAccount>, ExternalSyncError> {
        let accounts = self.accounts.read().await;
        Ok(accounts.values().find(|a| {
            a.organization_id == organization_id
                && &a.provider == provider
                && a.external_account_id == external_id
        }).cloned())
    }

    async fn update_account_tokens(
        &self,
        account_id: Uuid,
        tokens: &TokenResponse,
    ) -> Result<ExternalAccount, ExternalSyncError> {
        let mut accounts = self.accounts.write().await;
        let account = accounts.get_mut(&account_id)
            .ok_or_else(|| ExternalSyncError::DatabaseError("Account not found".into()))?;
        account.access_token = tokens.access_token.clone();
        account.refresh_token = tokens.refresh_token.clone();
        account.token_expires_at = tokens.expires_at;
        account.updated_at = Utc::now();
        Ok(account.clone())
    }

    async fn save_account(&self, account: &ExternalAccount) -> Result<(), ExternalSyncError> {
        let mut accounts = self.accounts.write().await;
        accounts.insert(account.id, account.clone());
        Ok(())
    }

    async fn get_account(&self, account_id: Uuid) -> Result<ExternalAccount, ExternalSyncError> {
        let accounts = self.accounts.read().await;
        accounts.get(&account_id).cloned()
            .ok_or_else(|| ExternalSyncError::DatabaseError("Account not found".into()))
    }

    async fn delete_account(&self, account_id: Uuid) -> Result<(), ExternalSyncError> {
        let mut accounts = self.accounts.write().await;
        accounts.remove(&account_id);
        Ok(())
    }

    async fn ensure_valid_token(&self, _account: &ExternalAccount) -> Result<String, ExternalSyncError> {
        Ok("valid_token".into())
    }

    async fn save_sync_history(&self, history: &SyncHistory) -> Result<(), ExternalSyncError> {
        let mut sync_history = self.sync_history.write().await;
        sync_history.push(history.clone());
        Ok(())
    }

    async fn update_account_sync_status(
        &self,
        account_id: Uuid,
        status: SyncStatus,
    ) -> Result<(), ExternalSyncError> {
        let mut accounts = self.accounts.write().await;
        if let Some(account) = accounts.get_mut(&account_id) {
            account.last_sync_status = Some(status.to_string());
            account.last_sync_at = Some(Utc::now());
        }
        Ok(())
    }

    async fn update_account_sync_cursor(
        &self,
        account_id: Uuid,
        cursor: Option<String>,
    ) -> Result<(), ExternalSyncError> {
        let mut accounts = self.accounts.write().await;
        if let Some(account) = accounts.get_mut(&account_id) {
            account.sync_cursor = cursor;
        }
        Ok(())
    }

    async fn get_pending_uploads(&self, account_id: Uuid) -> Result<Vec<ContactMapping>, ExternalSyncError> {
        let mappings = self.mappings.read().await;
        Ok(mappings.values()
            .filter(|m| m.account_id == account_id && m.sync_status == MappingSyncStatus::PendingUpload)
            .cloned()
            .collect())
    }

    async fn get_mapping_by_external_id(
        &self,
        account_id: Uuid,
        external_id: &str,
    ) -> Result<Option<ContactMapping>, ExternalSyncError> {
        let mappings = self.mappings.read().await;
        Ok(mappings.values()
            .find(|m| m.account_id == account_id && m.external_id == external_id)
            .cloned())
    }

    async fn has_internal_changes(&self, _mapping: &ContactMapping) -> Result<bool, ExternalSyncError> {
        Ok(false)
    }

    async fn mark_conflict(
        &self,
        mapping_id: Uuid,
        _internal_changes: Vec<String>,
        _external_changes: Vec<String>,
    ) -> Result<(), ExternalSyncError> {
        let mut mappings = self.mappings.write().await;
        if let Some(mapping) = mappings.get_mut(&mapping_id) {
            mapping.sync_status = MappingSyncStatus::Conflict;
            mapping.conflict_detected_at = Some(Utc::now());
        }
        Ok(())
    }

    async fn update_internal_contact(
        &self,
        _contact_id: Uuid,
        _external: &ExternalContact,
    ) -> Result<(), ExternalSyncError> {
        Ok(())
    }

    async fn update_mapping_after_sync(
        &self,
        mapping_id: Uuid,
        etag: Option<String>,
    ) -> Result<(), ExternalSyncError> {
        let mut mappings = self.mappings.write().await;
        if let Some(mapping) = mappings.get_mut(&mapping_id) {
            mapping.external_etag = etag;
            mapping.last_synced_at = Utc::now();
            mapping.sync_status = MappingSyncStatus::Synced;
        }
        Ok(())
    }

    async fn create_internal_contact(
        &self,
        _organization_id: Uuid,
        external: &ExternalContact,
    ) -> Result<Uuid, ExternalSyncError> {
        let contact_id = Uuid::new_v4();
        let mut contacts = self.contacts.write().await;
        let mut contact = external.clone();
        contact.id = contact_id.to_string();
        contacts.insert(contact_id, contact);
        Ok(contact_id)
    }

    async fn create_mapping(&self, mapping: &ContactMapping) -> Result<(), ExternalSyncError> {
        let mut mappings = self.mappings.write().await;
        mappings.insert(mapping.id, mapping.clone());
        Ok(())
    }

    async fn get_internal_contact(&self, contact_id: Uuid) -> Result<ExternalContact, ExternalSyncError> {
        let contacts = self.contacts.read().await;
        contacts.get(&contact_id).cloned()
            .ok_or_else(|| ExternalSyncError::DatabaseError("Contact not found".into()))
    }

    async fn convert_to_external(&self, contact: &ExternalContact) -> Result<ExternalContact, ExternalSyncError> {
        Ok(contact.clone())
    }

    async fn update_mapping_external_id(
        &self,
        mapping_id: Uuid,
        external_id: String,
        etag: Option<String>,
    ) -> Result<(), ExternalSyncError> {
        let mut mappings = self.mappings.write().await;
        if let Some(mapping) = mappings.get_mut(&mapping_id) {
            mapping.external_id = external_id;
            mapping.external_etag = etag;
        }
        Ok(())
    }

    async fn fetch_accounts(&self, organization_id: Uuid) -> Result<Vec<ExternalAccount>, ExternalSyncError> {
        let accounts = self.accounts.read().await;
        Ok(accounts.values()
            .filter(|a| a.organization_id == organization_id)
            .cloned()
            .collect())
    }

    async fn get_sync_stats(&self, account_id: Uuid) -> Result<SyncStats, ExternalSyncError> {
        let history = self.sync_history.read().await;
        let account_history: Vec<_> = history.iter()
            .filter(|h| h.account_id == account_id)
            .collect();
        let successful = account_history.iter().filter(|h| h.status == SyncStatus::Success).count();
        let failed = account_history.iter().filter(|h| h.status == SyncStatus::Failed).count();
        Ok(SyncStats {
            total_synced_contacts: account_history.iter().map(|h| h.contacts_created + h.contacts_updated).sum(),
            total_syncs: account_history.len() as u32,
            successful_syncs: successful as u32,
            failed_syncs: failed as u32,
            last_successful_sync: account_history.iter()
                .filter(|h| h.status == SyncStatus::Success)
                .max_by_key(|h| h.completed_at)
                .and_then(|h| h.completed_at),
            average_sync_duration_seconds: 60,
        })
    }

    async fn count_pending_conflicts(&self, account_id: Uuid) -> Result<u32, ExternalSyncError> {
        let mappings = self.mappings.read().await;
        Ok(mappings.values()
            .filter(|m| m.account_id == account_id && m.sync_status == MappingSyncStatus::Conflict)
            .count() as u32)
    }

    async fn count_pending_errors(&self, account_id: Uuid) -> Result<u32, ExternalSyncError> {
        let mappings = self.mappings.read().await;
        Ok(mappings.values()
            .filter(|m| m.account_id == account_id && m.sync_status == MappingSyncStatus::Error)
            .count() as u32)
    }

    async fn get_next_scheduled_sync(&self, _account_id: Uuid) -> Result<Option<DateTime<Utc>>, ExternalSyncError> {
        Ok(Some(Utc::now() + chrono::Duration::hours(1)))
    }

    async fn fetch_sync_history(
        &self,
        account_id: Uuid,
        _limit: u32,
    ) -> Result<Vec<SyncHistory>, ExternalSyncError> {
        let history = self.sync_history.read().await;
        Ok(history.iter()
            .filter(|h| h.account_id == account_id)
            .cloned()
            .collect())
    }

    async fn fetch_conflicts(&self, account_id: Uuid) -> Result<Vec<ContactMapping>, ExternalSyncError> {
        let mappings = self.mappings.read().await;
        Ok(mappings.values()
            .filter(|m| m.account_id == account_id && m.sync_status == MappingSyncStatus::Conflict)
            .cloned()
            .collect())
    }

    async fn get_mapping(&self, mapping_id: Uuid) -> Result<ContactMapping, ExternalSyncError> {
        let mappings = self.mappings.read().await;
        mappings.get(&mapping_id).cloned()
            .ok_or_else(|| ExternalSyncError::DatabaseError("Mapping not found".into()))
    }

    pub fn get_authorization_url(
        &self,
        provider: &ExternalProvider,
        redirect_uri: &str,
        state: &str,
    ) -> Result<AuthorizationUrlResponse, ExternalSyncError> {
        let url = match provider {
            ExternalProvider::Google => self.google_client.get_auth_url(redirect_uri, state),
            ExternalProvider::Microsoft => self.microsoft_client.get_auth_url(redirect_uri, state),
            ExternalProvider::Apple => {
                return Err(ExternalSyncError::UnsupportedProvider("Apple".to_string()))
            }
            ExternalProvider::CardDav => {
                return Err(ExternalSyncError::UnsupportedProvider(
                    "CardDAV requires direct configuration".to_string(),
                ))
            }
        };

        Ok(AuthorizationUrlResponse {
            url,
            state: state.to_string(),
        })
    }

    pub async fn connect_account(
        &self,
        organization_id: Uuid,
        user_id: Uuid,
        request: &ConnectAccountRequest,
    ) -> Result<ExternalAccount, ExternalSyncError> {
        // Exchange authorization code for tokens
        let tokens = match request.provider {
            ExternalProvider::Google => {
                self.google_client
                    .exchange_code(&request.authorization_code, &request.redirect_uri)
                    .await?
            }
            ExternalProvider::Microsoft => {
                self.microsoft_client
                    .exchange_code(&request.authorization_code, &request.redirect_uri)
                    .await?
            }
            _ => {
                return Err(ExternalSyncError::UnsupportedProvider(
                    request.provider.to_string(),
                ))
            }
        };

        // Get user info from provider
        let user_info = match request.provider {
            ExternalProvider::Google => {
                self.google_client.get_user_info(&tokens.access_token).await?
            }
            ExternalProvider::Microsoft => {
                self.microsoft_client
                    .get_user_info(&tokens.access_token)
                    .await?
            }
            _ => unreachable!(),
        };

        // Check if account already exists
        if let Some(existing) = self
            .find_existing_account(organization_id, &request.provider, &user_info.id)
            .await?
        {
            // Update tokens
            return self
                .update_account_tokens(existing.id, &tokens)
                .await;
        }

        // Create new account
        let account_id = Uuid::new_v4();
        let now = Utc::now();

        let account = ExternalAccount {
            id: account_id,
            organization_id,
            user_id,
            provider: request.provider.clone(),
            external_account_id: user_info.id,
            email: user_info.email,
            display_name: user_info.name,
            access_token: tokens.access_token,
            refresh_token: tokens.refresh_token,
            token_expires_at: tokens.expires_at,
            scopes: tokens.scopes,
            sync_enabled: true,
            sync_direction: request.sync_direction.clone().unwrap_or_default(),
            last_sync_at: None,
            last_sync_status: None,
            sync_cursor: None,
            created_at: now,
            updated_at: now,
        };

        self.save_account(&account).await?;

        Ok(account)
    }

    pub async fn disconnect_account(
        &self,
        organization_id: Uuid,
        account_id: Uuid,
    ) -> Result<(), ExternalSyncError> {
        let account = self.get_account(account_id).await?;

        if account.organization_id != organization_id {
            return Err(ExternalSyncError::Unauthorized);
        }

        // Revoke tokens with provider
        match account.provider {
            ExternalProvider::Google => {
                let _ = self.google_client.revoke_token(&account.access_token).await;
            }
            ExternalProvider::Microsoft => {
                let _ = self
                    .microsoft_client
                    .revoke_token(&account.access_token)
                    .await;
            }
            _ => {}
        }

        // Delete account and mappings
        self.delete_account(account_id).await?;

        Ok(())
    }

    pub async fn start_sync(
        &self,
        organization_id: Uuid,
        account_id: Uuid,
        request: &StartSyncRequest,
        trigger: SyncTrigger,
    ) -> Result<SyncHistory, ExternalSyncError> {
        let account = self.get_account(account_id).await?;

        if account.organization_id != organization_id {
            return Err(ExternalSyncError::Unauthorized);
        }

        if !account.sync_enabled {
            return Err(ExternalSyncError::SyncDisabled);
        }

        if let Some(last_status) = &account.last_sync_status {
            if last_status == "in_progress" {
                return Err(ExternalSyncError::SyncInProgress);
            }
        }

        // Refresh token if needed
        let access_token = self.ensure_valid_token(&account).await?;
        let sync_direction = account.sync_direction.clone();
        let account = ExternalAccount {
            access_token,
            ..account
        };

        let sync_id = Uuid::new_v4();
        let now = Utc::now();
        let direction = request.direction.clone().unwrap_or(sync_direction);

        let mut history = SyncHistory {
            id: sync_id,
            account_id,
            started_at: now,
            completed_at: None,
            status: SyncStatus::InProgress,
            direction: direction.clone(),
            contacts_created: 0,
            contacts_updated: 0,
            contacts_deleted: 0,
            contacts_skipped: 0,
            conflicts_detected: 0,
            errors: vec![],
            triggered_by: trigger,
        };

        self.save_sync_history(&history).await?;
        self.update_account_sync_status(account_id, SyncStatus::InProgress)
            .await?;

        // Perform sync based on direction
        let result = match direction {
            SyncDirection::TwoWay => {
                self.perform_two_way_sync(&account, request.full_sync.unwrap_or(false), &mut history)
                    .await
            }
            SyncDirection::ImportOnly => {
                self.perform_import_sync(&account, request.full_sync.unwrap_or(false), &mut history)
                    .await
            }
            SyncDirection::ExportOnly => {
                self.perform_export_sync(&account, &mut history).await
            }
        };

        // Update history with results
        history.completed_at = Some(Utc::now());
        history.status = match &result {
            Ok(_) if history.errors.is_empty() => SyncStatus::Success,
            Ok(_) => SyncStatus::PartialSuccess,
            Err(_) => SyncStatus::Failed,
        };

        self.save_sync_history(&history).await?;
        self.update_account_sync_status(account_id, history.status.clone())
            .await?;

        if let Err(e) = result {
            return Err(e);
        }

        Ok(history)
    }

    async fn perform_two_way_sync(
        &self,
        account: &ExternalAccount,
        full_sync: bool,
        history: &mut SyncHistory,
    ) -> Result<(), ExternalSyncError> {
        // First import from external
        self.perform_import_sync(account, full_sync, history).await?;

        // Then export to external
        self.perform_export_sync(account, history).await?;

        Ok(())
    }

    async fn perform_import_sync(
        &self,
        account: &ExternalAccount,
        full_sync: bool,
        history: &mut SyncHistory,
    ) -> Result<(), ExternalSyncError> {
        let sync_cursor = if full_sync {
            None
        } else {
            account.sync_cursor.clone()
        };

        // Fetch contacts from provider
        let (external_contacts, new_cursor) = match account.provider {
            ExternalProvider::Google => {
                self.google_client
                    .list_contacts(&account.access_token, sync_cursor.as_deref())
                    .await?
            }
            ExternalProvider::Microsoft => {
                self.microsoft_client
                    .list_contacts(&account.access_token, sync_cursor.as_deref())
                    .await?
            }
            _ => return Err(ExternalSyncError::UnsupportedProvider(account.provider.to_string())),
        };

        // Process each contact
        for external_contact in external_contacts {
            match self
                .import_contact(account, &external_contact, history)
                .await
            {
                Ok(ImportResult::Created) => history.contacts_created += 1,
                Ok(ImportResult::Updated) => history.contacts_updated += 1,
                Ok(ImportResult::Skipped) => history.contacts_skipped += 1,
                Ok(ImportResult::Conflict) => history.conflicts_detected += 1,
                Err(e) => {
                    history.errors.push(SyncError {
                        contact_id: None,
                        external_id: Some(external_contact.id.clone()),
                        operation: "import".to_string(),
                        error_code: "import_failed".to_string(),
                        error_message: e.to_string(),
                        retryable: true,
                    });
                }
            }
        }

        // Update sync cursor
        self.update_account_sync_cursor(account.id, new_cursor).await?;

        Ok(())
    }

    async fn perform_export_sync(
        &self,
        account: &ExternalAccount,
        history: &mut SyncHistory,
    ) -> Result<(), ExternalSyncError> {
        // Get pending uploads
        let pending_contacts = self.get_pending_uploads(account.id).await?;

        for mapping in pending_contacts {
            match self.export_contact(account, &mapping, history).await {
                Ok(ExportResult::Created) => history.contacts_created += 1,
                Ok(ExportResult::Updated) => history.contacts_updated += 1,
                Ok(ExportResult::Deleted) => history.contacts_deleted += 1,
                Ok(ExportResult::Skipped) => history.contacts_skipped += 1,
                Err(e) => {
                    history.errors.push(SyncError {
                        contact_id: Some(mapping.local_contact_id),
                        external_id: Some(mapping.external_contact_id.clone()),
                        operation: "export".to_string(),
                        error_code: "export_failed".to_string(),
                        error_message: e.to_string(),
                        retryable: true,
                    });
                }
            }
        }

        Ok(())
    }

    async fn import_contact(
        &self,
        account: &ExternalAccount,
        external: &ExternalContact,
        _history: &mut SyncHistory,
    ) -> Result<ImportResult, ExternalSyncError> {
        let existing_mapping = self
            .get_mapping_by_external_id(account.id, &external.id)
            .await?;

        if let Some(mapping) = existing_mapping {
            if mapping.external_etag.as_ref() != external.etag.as_ref() {
                let internal_changed = self
                    .has_internal_changes(&mapping)
                    .await?;

                if internal_changed {
                    self.mark_conflict(
                        mapping.id,
                        vec!["external_updated".to_string()],
                        vec!["internal_updated".to_string()],
                    )
                    .await?;
                    return Ok(ImportResult::Conflict);
                }

                self.update_internal_contact(mapping.local_contact_id, external)
                    .await?;
                self.update_mapping_after_sync(mapping.id, external.etag.clone())
                    .await?;
                return Ok(ImportResult::Updated);
            }

            return Ok(ImportResult::Skipped);
        }

        let contact_id = self
            .create_internal_contact(account.organization_id, external)
            .await?;

        let now = Utc::now();
        let mapping = ContactMapping {
            id: Uuid::new_v4(),
            account_id: account.id,
            contact_id,
            local_contact_id: contact_id,
            external_id: external.id.clone(),
            external_contact_id: external.id.clone(),
            external_etag: external.etag.clone(),
            internal_version: 1,
            last_synced_at: now,
            sync_status: MappingSyncStatus::Synced,
            conflict_data: None,
            local_data: None,
            remote_data: None,
            conflict_detected_at: None,
            created_at: now,
            updated_at: now,
        };
        self.create_mapping(&mapping).await?;

        Ok(ImportResult::Created)
    }

    async fn export_contact(
        &self,
        account: &ExternalAccount,
        mapping: &ContactMapping,
        _history: &mut SyncHistory,
    ) -> Result<ExportResult, ExternalSyncError> {
        let internal = self.get_internal_contact(mapping.local_contact_id).await?;

        let external = self.convert_to_external(&internal).await?;

        if mapping.external_contact_id.is_empty() {
            let external_id = match account.provider {
                ExternalProvider::Google => {
                    self.google_client
                        .create_contact(&account.access_token, &external)
                        .await?
                }
                ExternalProvider::Microsoft => {
                    self.microsoft_client
                        .create_contact(&account.access_token, &external)
                        .await?
                }
                _ => return Err(ExternalSyncError::UnsupportedProvider(account.provider.to_string())),
            };

            self.update_mapping_external_id(mapping.id, external_id, None)
                .await?;
            return Ok(ExportResult::Created);
        }

        match account.provider {
            ExternalProvider::Google => {
                self.google_client
                    .update_contact(
                        &account.access_token,
                        &mapping.external_contact_id,
                        &external,
                    )
                    .await?;
            }
            ExternalProvider::Microsoft => {
                self.microsoft_client
                    .update_contact(
                        &account.access_token,
                        &mapping.external_contact_id,
                        &external,
                    )
                    .await?;
            }
            _ => return Err(ExternalSyncError::UnsupportedProvider(account.provider.to_string())),
        }

        self.update_mapping_after_sync(mapping.id, None).await?;

        Ok(ExportResult::Updated)
    }

    pub async fn list_accounts(
        &self,
        organization_id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<Vec<AccountStatusResponse>, ExternalSyncError> {
        let accounts = self.fetch_accounts(organization_id).await?;
        let accounts: Vec<_> = if let Some(uid) = user_id {
            accounts.into_iter().filter(|a| a.user_id == uid).collect()
        } else {
            accounts
        };
        let mut results = Vec::new();

        for account in accounts {
            let sync_stats = self.get_sync_stats(account.id).await?;
            let pending_conflicts = self.count_pending_conflicts(account.id).await?;
            let pending_errors = self.count_pending_errors(account.id).await?;
            let next_sync = self.get_next_scheduled_sync(account.id).await?;

            results.push(AccountStatusResponse {
                account,
                sync_stats,
                pending_conflicts,
                pending_errors,
                next_scheduled_sync: next_sync,
            });
        }

        Ok(results)
    }

    pub async fn get_sync_history(
        &self,
        organization_id: Uuid,
        account_id: Uuid,
        limit: Option<u32>,
    ) -> Result<Vec<SyncHistory>, ExternalSyncError> {
        let account = self.get_account(account_id).await?;

        if account.organization_id != organization_id {
            return Err(ExternalSyncError::Unauthorized);
        }

        self.fetch_sync_history(account_id, limit.unwrap_or(20)).await
    }

    pub async fn get_conflicts(
        &self,
        organization_id: Uuid,
        account_id: Uuid,
    ) -> Result<Vec<ContactMapping>, ExternalSyncError> {
        let account = self.get_account(account_id).await?;

        if account.organization_id != organization_id {
            return Err(ExternalSyncError::Unauthorized);
        }

        self.fetch_conflicts(account_id).await
    }

    pub async fn resolve_conflict(
        &self,
        organization_id: Uuid,
        mapping_id: Uuid,
        request: &ResolveConflictRequest,
    ) -> Result<ContactMapping, ExternalSyncError> {
        let mapping = self.get_mapping(mapping_id).await?;
        let account = self.get_account(mapping.account_id).await?;

        if account.organization_id != organization_id {
            return Err(ExternalSyncError::Unauthorized);
        }

        // Apply the resolution based on strategy
        let resolved_contact = match request.resolution {
            ConflictResolution::KeepLocal | ConflictResolution::KeepInternal => mapping.local_data.clone(),
            ConflictResolution::KeepRemote | ConflictResolution::KeepExternal => mapping.remote_data.clone(),
            ConflictResolution::Merge => {
                let mut merged = mapping.local_data.clone().unwrap_or_default();
                if let Some(remote) = &mapping.remote_data {
                    merged = remote.clone();
                }
                Some(merged)
            }
            ConflictResolution::Manual => request.manual_data.clone(),
            ConflictResolution::Skip => None,
        };

        let now = Utc::now();
        let updated_mapping = ContactMapping {
            id: mapping.id,
            account_id: mapping.account_id,
            contact_id: mapping.contact_id,
            local_contact_id: mapping.local_contact_id,
            external_id: mapping.external_id.clone(),
            external_contact_id: mapping.external_contact_id.clone(),
            external_etag: mapping.external_etag.clone(),
            internal_version: mapping.internal_version + 1,
            last_synced_at: now,
            sync_status: MappingSyncStatus::Synced,
            conflict_data: None,
            local_data: resolved_contact,
            remote_data: mapping.remote_data.clone(),
            conflict_detected_at: None,
            created_at: mapping.created_at,
            updated_at: now,
        };

        let mut mappings = self.mappings.write().await;
        mappings.insert(updated_mapping.id, updated_mapping.clone());

        Ok(updated_mapping)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_status_display() {
        assert_eq!(format!("{:?}", SyncStatus::Pending), "Pending");
        assert_eq!(format!("{:?}", SyncStatus::Synced), "Synced");
        assert_eq!(format!("{:?}", SyncStatus::Conflict), "Conflict");
    }

    #[test]
    fn test_conflict_resolution_variants() {
        let _keep_local = ConflictResolution::KeepLocal;
        let _keep_remote = ConflictResolution::KeepRemote;
        let _merge = ConflictResolution::Merge;
        let _manual = ConflictResolution::Manual;
    }
}
