use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::skill_packages)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PackageRow {
    pub id: Uuid,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub latest_version: Option<String>,
    pub publisher_org_id: Option<Uuid>,
    pub publisher_name: Option<String>,
    pub visibility: String,
    pub review_status: String,
    pub downloads: i64,
    pub icon_glyph: Option<String>,
    pub tags: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::skill_versions)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct VersionRow {
    pub id: Uuid,
    pub package_id: Uuid,
    pub version: String,
    pub manifest: serde_json::Value,
    pub object_key: String,
    pub changelog: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::skill_installs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InstallRow {
    pub id: Uuid,
    pub package_id: Uuid,
    pub version_id: Uuid,
    pub org_id: Option<Uuid>,
    pub branch_id: Option<Uuid>,
    pub bot_id: Uuid,
    pub installed_by: Option<Uuid>,
    pub status: String,
    pub installed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishBody {
    pub slug: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub tags: serde_json::Value,
    #[serde(default)]
    pub icon_glyph: Option<String>,
    pub version: String,
    #[serde(default)]
    pub changelog: Option<String>,
    #[serde(default)]
    pub manifest: serde_json::Value,
    pub content_base64: String,
    #[serde(default)]
    pub visibility: Option<String>,
}

impl PublishBody {
    pub fn effective_visibility(&self) -> &str {
        match self.visibility.as_deref() {
            Some("private") => "private",
            _ => "public",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct InstallBody {
    pub bot_id: Uuid,
    #[serde(default)]
    pub version: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_roundtrips_through_serde() {
        let body = PublishBody {
            slug: "csv-cleaner".to_string(),
            name: "CSV Cleaner".to_string(),
            description: Some("Cleans CSV files".to_string()),
            tags: serde_json::json!(["data", "files"]),
            icon_glyph: Some("\u{1F9F9}".to_string()),
            version: "1.0.0".to_string(),
            changelog: None,
            manifest: serde_json::json!({
                "entry": "clean_csv.bas",
                "scripts": ["clean_csv.bas", "summary_clean.bas"],
                "prompts": [],
                "permissions": ["file.read", "file.write"],
            }),
            content_base64: "YWJj".to_string(),
            visibility: Some("public".to_string()),
        };

        let wire = serde_json::to_string(&body).unwrap();
        let parsed: PublishBody = serde_json::from_str(&wire).unwrap();
        assert_eq!(parsed, body);
        assert_eq!(
            parsed.manifest.get("permissions"),
            Some(&serde_json::json!(["file.read", "file.write"]))
        );
    }

    #[test]
    fn install_body_supports_pinned_version() {
        let body: InstallBody =
            serde_json::from_value(serde_json::json!({ "bot_id": Uuid::nil(), "version": "1.2.3" })).unwrap();
        assert_eq!(body.version.as_deref(), Some("1.2.3"));
        let minimal: InstallBody = serde_json::from_value(serde_json::json!({ "bot_id": Uuid::nil() })).unwrap();
        assert!(minimal.version.is_none());
    }
}
