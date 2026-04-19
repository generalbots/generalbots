// Microsoft Graph API client extracted from external_sync.rs
use crate::contacts::external_sync::{ExternalContact, ExternalEmail, ExternalPhone};
use reqwest::Client;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct MicrosoftClient {
    pub client: Client,
}

impl MicrosoftClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn list_contacts(
        &self,
        access_token: &str,
        skip: Option<String>,
    ) -> Result<(Vec<ExternalContact>, Option<String>), MicrosoftError> {
        let mut url = "https://graph.microsoft.com/v1.0/me/contacts?$select=id,displayName,givenName,surname,emailAddresses,mobilePhone,companyName,jobTitle".to_string();

        if let Some(s) = skip {
            url.push_str(&format!("&$skip={}", s));
        }

        let response = self
            .client
            .get(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| MicrosoftError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(MicrosoftError::ApiError(format!(
                "Failed to list contacts: {}",
                response.status()
            )));
        }

        #[derive(Deserialize)]
        struct MsContactsResponse {
            value: Vec<MsContact>,
            #[serde(rename = "@odata.nextLink")]
            next_link: Option<String>,
        }

        let data: MsContactsResponse = response
            .json()
            .await
            .map_err(|e| MicrosoftError::ParseError(e.to_string()))?;

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

    pub async fn fetch_contacts(&self, access_token: &str) -> Result<Vec<ExternalContact>, MicrosoftError> {
        let mut all_contacts = Vec::new();
        let mut cursor: Option<String> = None;

        loop {
            let (contacts, next_cursor) = self.list_contacts(access_token, cursor.as_deref()).await?;
            all_contacts.extend(contacts);

            if next_cursor.is_none() {
                break;
            }
            cursor = next_cursor;

            if all_contacts.len() > 10000 {
                log::warn!("Reached contact fetch limit");
                break;
            }
        }

        Ok(all_contacts)
    }

    pub async fn create_contact(&self, access_token: &str, contact: &ExternalContact) -> Result<String, MicrosoftError> {
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
            .map_err(|e| MicrosoftError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(MicrosoftError::ApiError(format!("Create contact failed: {} - {}", status, body)));
        }

        #[derive(Deserialize)]
        struct CreateResponse {
            id: String,
        }

        let data: CreateResponse = response.json().await
            .map_err(|e| MicrosoftError::ParseError(e.to_string()))?;

        Ok(data.id)
    }

    pub async fn update_contact(&self, access_token: &str, contact_id: &str, contact: &ExternalContact) -> Result<(), MicrosoftError> {
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
            .map_err(|e| MicrosoftError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(MicrosoftError::ApiError(format!("Update contact failed: {}", status)));
        }

        Ok(())
    }

    pub async fn delete_contact(&self, access_token: &str, contact_id: &str) -> Result<(), MicrosoftError> {
        let url = format!("https://graph.microsoft.com/v1.0/me/contacts/{}", contact_id);

        let response = self.client
            .delete(&url)
            .bearer_auth(access_token)
            .send()
            .await
            .map_err(|e| MicrosoftError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            return Err(MicrosoftError::ApiError(format!("Delete contact failed: {}", status)));
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum MicrosoftError {
    NetworkError(String),
    ApiError(String),
    ParseError(String),
}

impl std::fmt::Display for MicrosoftError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NetworkError(e) => write!(f, "Network error: {e}"),
            Self::ApiError(e) => write!(f, "API error: {e}"),
            Self::ParseError(e) => write!(f, "Parse error: {e}"),
        }
    }
}

impl std::error::Error for MicrosoftError {}

#[derive(Debug, Clone, Deserialize)]
struct MsContact {
    id: String,
    given_name: Option<String>,
    surname: Option<String>,
    display_name: Option<String>,
    email_addresses: Option<Vec<MsEmailAddress>>,
    mobile_phone: Option<String>,
    business_phones: Option<Vec<String>>,
    company_name: Option<String>,
    job_title: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct MsEmailAddress {
    address: String,
    name: Option<String>,
}
