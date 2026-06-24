use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimUser {
    pub schemas: Vec<String>,
    pub id: Option<String>,
    pub external_id: Option<String>,
    pub user_name: String,
    pub family_name: Option<String>,
    pub given_name: Option<String>,
    pub display_name: Option<String>,
    pub active: bool,
    #[serde(rename = "emails")]
    pub emails: Vec<ScimEmail>,
    #[serde(rename = "phoneNumbers")]
    pub phone_numbers: Vec<ScimPhone>,
    #[serde(rename = "photos")]
    pub photos: Vec<ScimPhoto>,
    #[serde(rename = "groups")]
    pub groups: Vec<ScimGroupRef>,
    #[serde(rename = "meta")]
    pub meta: Option<ScimMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimEmail {
    pub value: String,
    #[serde(rename = "type")]
    pub email_type: Option<String>,
    pub primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimPhone {
    pub value: String,
    #[serde(rename = "type")]
    pub phone_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimPhoto {
    pub value: String,
    #[serde(rename = "type")]
    pub photo_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimGroupRef {
    pub value: String,
    #[serde(rename = "$ref")]
    pub reference: Option<String>,
    pub display: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimMeta {
    pub resource_type: String,
    pub created: Option<String>,
    pub last_modified: Option<String>,
    pub location: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimGroup {
    pub schemas: Vec<String>,
    pub id: Option<String>,
    pub external_id: Option<String>,
    pub display_name: String,
    #[serde(rename = "members")]
    pub members: Vec<ScimMember>,
    #[serde(rename = "meta")]
    pub meta: Option<ScimMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimMember {
    pub value: String,
    #[serde(rename = "$ref")]
    pub reference: Option<String>,
    pub display: Option<String>,
    #[serde(rename = "type")]
    pub member_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimListResponse {
    pub schemas: Vec<String>,
    pub total_results: u32,
    pub start_index: u32,
    pub items_per_page: u32,
    pub resources: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScimError {
    pub schemas: Vec<String>,
    pub scim_type: Option<String>,
    pub detail: String,
    pub status: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScimPatchOp {
    pub schemas: Vec<String>,
    pub operations: Vec<ScimPatchOperation>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ScimPatchOperation {
    pub op: String,
    pub path: Option<String>,
    pub value: serde_json::Value,
}

impl ScimUser {
    pub fn from_zitadel_user(user: &serde_json::Value, groups: Vec<ScimGroupRef>) -> Self {
        let user_name = user.get("userName")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let family_name = user.get("name")
            .and_then(|n| n.get("familyName"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let given_name = user.get("name")
            .and_then(|n| n.get("givenName"))
            .and_then(|v| v.as_str())
            .map(String::from);

        let display_name = user.get("displayName")
            .and_then(|v| v.as_str())
            .map(String::from);

        let emails = user.get("emails")
            .and_then(|e| e.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|e| {
                        Some(ScimEmail {
                            value: e.get("value")?.as_str()?.to_string(),
                            email_type: e.get("type").and_then(|t| t.as_str()).map(String::from),
                            primary: e.get("primary").and_then(|p| p.as_bool()).unwrap_or(false),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let state = user.get("state")
            .and_then(|s| s.as_str())
            .unwrap_or("active");

        ScimUser {
            schemas: vec![
                "urn:ietf:params:scim:schemas:core:2.0:User".to_string(),
                "urn:ietf:params:scim:schemas:extension:enterprise:2.0:User".to_string(),
            ],
            id: user.get("userId").and_then(|v| v.as_str()).map(String::from),
            external_id: user.get("id").and_then(|v| v.as_str()).map(String::from),
            user_name,
            family_name,
            given_name,
            display_name,
            active: state == "active",
            emails,
            phone_numbers: vec![],
            photos: vec![],
            groups,
            meta: Some(ScimMeta {
                resource_type: "User".to_string(),
                created: user.get("creationDate").and_then(|v| v.as_str()).map(String::from),
                last_modified: user.get("changeDate").and_then(|v| v.as_str()).map(String::from),
                location: None,
                version: None,
            }),
        }
    }

    pub fn to_zitadel_json(&self) -> serde_json::Value {
        let mut profile = serde_json::Map::new();
        if let Some(ref gn) = self.given_name {
            profile.insert("givenName".to_string(), serde_json::json!(gn));
        }
        if let Some(ref fn_) = self.family_name {
            profile.insert("familyName".to_string(), serde_json::json!(fn_));
        }
        if let Some(ref dn) = self.display_name {
            profile.insert("displayName".to_string(), serde_json::json!(dn));
        }

        let mut emails = serde_json::Map::new();
        if let Some(e) = self.emails.first() {
            emails.insert("email".to_string(), serde_json::json!(e.value));
            emails.insert("isVerified".to_string(), serde_json::json!(true));
        }

        serde_json::json!({
            "userName": self.user_name,
            "profile": profile,
            "emails": emails,
            "phone": {},
        })
    }
}

impl ScimGroup {
    pub fn from_metadata(key: &str, value: &serde_json::Value, members: Vec<ScimMember>) -> Self {
        let display_name = value.get("name")
            .and_then(|n| n.as_str())
            .unwrap_or(key)
            .to_string();

        ScimGroup {
            schemas: vec!["urn:ietf:params:scim:schemas:core:2.0:Group".to_string()],
            id: Some(key.to_string()),
            external_id: None,
            display_name,
            members,
            meta: Some(ScimMeta {
                resource_type: "Group".to_string(),
                created: value.get("created_at").and_then(|v| v.as_str()).map(String::from),
                last_modified: value.get("updated_at").and_then(|v| v.as_str()).map(String::from),
                location: None,
                version: None,
            }),
        }
    }
}
