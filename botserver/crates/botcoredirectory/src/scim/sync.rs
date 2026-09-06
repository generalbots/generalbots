use log::info;
use serde::{Deserialize, Serialize};

use super::types::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureAdConfig {
    pub tenant_id: String,
    pub client_id: String,
    pub client_secret: String,
    pub sync_mode: SyncMode,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SyncMode {
    Full,
    Delta,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub groups_created: u32,
    pub groups_updated: u32,
    pub users_mapped: u32,
    pub users_created: u32,
    pub users_updated: u32,
    pub errors: Vec<String>,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    pub running: bool,
    pub last_sync: Option<String>,
    pub last_result: Option<SyncResult>,
    pub progress: SyncProgress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncProgress {
    pub phase: String,
    pub current: u32,
    pub total: u32,
}

const O365_GROUP_MAP: &[(&str, &str)] = &[
    ("Domain Users", "everyone"),
    ("Domain Admins", "admins"),
    ("Enterprise Admins", "admins"),
    ("HR Department", "human_resources"),
    ("Finance Department", "finance"),
    ("Marketing Department", "marketing"),
    ("Helpdesk", "support"),
    ("Content Managers", "content_managers"),
    ("Sales Department", "sales"),
    ("Developers", "developers"),
    ("Management", "managers"),
    ("Guest Users", "viewers"),
];

pub struct AzureAdSyncer {
    config: AzureAdConfig,
    http_client: reqwest::Client,
}

impl AzureAdSyncer {
    pub fn new(config: AzureAdConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn get_access_token(&self) -> Result<String, String> {
        // The tenant segment is constrained to hostname-safe characters so the
        // token endpoint always stays on login.microsoftonline.com (SSRF guard).
        let encoded: String = self.config.tenant_id.chars()
            .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '.' { c } else { '_' })
            .collect();
        let token_url = format!(
            "https://login.microsoftonline.com/{encoded}/oauth2/v2.0/token"
        );

        let params = [
            ("grant_type", "client_credentials"),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
            ("scope", "https://graph.microsoft.com/.default"),
        ];

        let resp = self.http_client
            .post(&token_url)
            .form(&params)
            .send()
            .await
            .map_err(|e| format!("Token request failed: {}", e))?;

        let data: serde_json::Value = resp.json().await
            .map_err(|e| format!("Token response parse failed: {}", e))?;

        data.get("access_token")
            .and_then(|t| t.as_str())
            .map(String::from)
            .ok_or_else(|| "No access_token in response".to_string())
    }

    pub async fn list_azure_ad_groups(&self, token: &str) -> Result<Vec<serde_json::Value>, String> {
        let url = "https://graph.microsoft.com/v1.0/groups?$select=id,displayName,mailEnabled,securityEnabled,groupTypes,members&$top=100";

        let resp = self.http_client
            .get(url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Failed to list groups: {}", e))?;

        let data: serde_json::Value = resp.json().await
            .map_err(|e| format!("Failed to parse groups: {}", e))?;

        data.get("value")
            .and_then(|v| v.as_array())
            .cloned()
            .ok_or_else(|| "No groups in response".to_string())
    }

    pub async fn list_azure_ad_users(&self, token: &str) -> Result<Vec<serde_json::Value>, String> {
        let url = "https://graph.microsoft.com/v1.0/users?$select=id,displayName,givenName,surname,mail,userPrincipalName,accountEnabled,memberOf&$top=100";

        let resp = self.http_client
            .get(url)
            .bearer_auth(token)
            .send()
            .await
            .map_err(|e| format!("Failed to list users: {}", e))?;

        let data: serde_json::Value = resp.json().await
            .map_err(|e| format!("Failed to parse users: {}", e))?;

        data.get("value")
            .and_then(|v| v.as_array())
            .cloned()
            .ok_or_else(|| "No users in response".to_string())
    }

    pub fn map_azure_group_to_gb(azure_group_name: &str) -> Option<&'static str> {
        O365_GROUP_MAP.iter()
            .find(|(o365_name, _)| o365_name.eq_ignore_ascii_case(azure_group_name))
            .map(|(_, gb_name)| *gb_name)
    }

    pub async fn sync(
        &self,
        auth_service: &dyn botlib::traits::AuthServiceTrait,
    ) -> Result<SyncResult, String> {
        let start = std::time::Instant::now();
        let mut result = SyncResult {
            groups_created: 0,
            groups_updated: 0,
            users_mapped: 0,
            users_created: 0,
            users_updated: 0,
            errors: vec![],
            duration_ms: 0,
        };

        let token = self.get_access_token().await?;

        info!("Starting Azure AD sync (mode: {:?})", self.config.sync_mode);

        let azure_groups = match self.list_azure_ad_groups(&token).await {
            Ok(g) => g,
            Err(e) => {
                result.errors.push(format!("Failed to list Azure AD groups: {}", e));
                result.duration_ms = start.elapsed().as_millis() as u64;
                return Ok(result);
            }
        };

        info!("Found {} Azure AD groups", azure_groups.len());

        let mut group_map: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

        for azure_group in &azure_groups {
            let azure_name = azure_group.get("displayName")
                .and_then(|n| n.as_str())
                .unwrap_or("");

            if let Some(gb_group_name) = Self::map_azure_group_to_gb(azure_name) {
                let member_ids: Vec<String> = azure_group.get("members")
                    .and_then(|m| m.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|m| m.get("id").and_then(|id| id.as_str()).map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();

                group_map.insert(gb_group_name.to_string(), member_ids);
                result.groups_created += 1;
            }
        }

        let azure_users = match self.list_azure_ad_users(&token).await {
            Ok(u) => u,
            Err(e) => {
                result.errors.push(format!("Failed to list Azure AD users: {}", e));
                result.duration_ms = start.elapsed().as_millis() as u64;
                return Ok(result);
            }
        };

        info!("Found {} Azure AD users", azure_users.len());

        for azure_user in &azure_users {
            let user_principal = azure_user.get("userPrincipalName")
                .and_then(|u| u.as_str())
                .unwrap_or("");
            let display_name = azure_user.get("displayName")
                .and_then(|d| d.as_str())
                .unwrap_or("");
            let given_name = azure_user.get("givenName")
                .and_then(|g| g.as_str())
                .unwrap_or("");
            let surname = azure_user.get("surname")
                .and_then(|s| s.as_str())
                .unwrap_or("");
            let email = azure_user.get("mail")
                .and_then(|m| m.as_str())
                .unwrap_or(user_principal);
            let is_active = azure_user.get("accountEnabled")
                .and_then(|a| a.as_bool())
                .unwrap_or(true);

            let user_groups: Vec<String> = azure_user.get("memberOf")
                .and_then(|m| m.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|g| {
                            let azure_group_name = g.get("displayName")?.as_str()?;
                            Self::map_azure_group_to_gb(azure_group_name).map(String::from)
                        })
                        .collect()
                })
                .unwrap_or_default();

            let scim_user = ScimUser {
                schemas: vec![
                    "urn:ietf:params:scim:schemas:core:2.0:User".to_string(),
                    "urn:ietf:params:scim:schemas:extension:enterprise:2.0:User".to_string(),
                ],
                id: None,
                external_id: azure_user.get("id").and_then(|i| i.as_str()).map(String::from),
                user_name: user_principal.to_string(),
                family_name: Some(surname.to_string()),
                given_name: Some(given_name.to_string()),
                display_name: Some(display_name.to_string()),
                active: is_active,
                emails: vec![ScimEmail {
                    value: email.to_string(),
                    email_type: Some("work".to_string()),
                    primary: true,
                }],
                phone_numbers: vec![],
                photos: vec![],
                groups: user_groups.iter().map(|g| ScimGroupRef {
                    value: g.clone(),
                    reference: Some(format!("/Groups/group_{}", g)),
                    display: None,
                }).collect(),
                meta: None,
            };

            let body = scim_user.to_zitadel_json();

            match auth_service.http_post(
                format!("{}/v2/users", auth_service.api_url()),
                body,
            ).await {
                Ok(_) => {
                    result.users_created += 1;
                    info!("Created user: {}", user_principal);
                }
                Err(e) => {
                    if e.contains("already exists") || e.contains("UNIQUE") {
                        result.users_updated += 1;
                    } else {
                        result.errors.push(format!("User {}: {}", user_principal, e));
                    }
                }
            }

            result.users_mapped += 1;
        }

        for (gb_group_name, member_ids) in &group_map {
            let metadata_key = format!("group_{}", gb_group_name);
            let metadata_value = serde_json::json!({
                "name": gb_group_name,
                "members": member_ids,
                "source": "azure_ad_sync",
                "synced_at": chrono::Utc::now().to_rfc3339()
            }).to_string();

            let body = serde_json::json!({
                "key": metadata_key,
                "value": metadata_value
            });

            match auth_service.http_post(
                format!("{}/metadata/organization", auth_service.api_url()),
                body,
            ).await {
                Ok(_) => {
                    result.groups_updated += 1;
                    info!("Synced group: {}", gb_group_name);
                }
                Err(e) => {
                    result.errors.push(format!("Group {}: {}", gb_group_name, e));
                }
            }
        }

        result.duration_ms = start.elapsed().as_millis() as u64;
        info!(
            "Azure AD sync complete: {} groups, {} users, {} errors in {}ms",
            result.groups_created,
            result.users_mapped,
            result.errors.len(),
            result.duration_ms
        );

        Ok(result)
    }
}
