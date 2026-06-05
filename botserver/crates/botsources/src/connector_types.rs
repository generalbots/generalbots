use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectorType {
    Salesforce,
    Sap,
    Totvs,
    Bling,
    Shopify,
    WooCommerce,
    MySql,
    Postgres,
    RestApi,
    GraphQl,
    GoogleSheets,
    Csv,
    SharePoint,
    Custom(String),
}

impl std::fmt::Display for ConnectorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Salesforce => write!(f, "salesforce"),
            Self::Sap => write!(f, "sap"),
            Self::Totvs => write!(f, "totvs"),
            Self::Bling => write!(f, "bling"),
            Self::Shopify => write!(f, "shopify"),
            Self::WooCommerce => write!(f, "woocommerce"),
            Self::MySql => write!(f, "mysql"),
            Self::Postgres => write!(f, "postgres"),
            Self::RestApi => write!(f, "rest_api"),
            Self::GraphQl => write!(f, "graphql"),
            Self::GoogleSheets => write!(f, "google_sheets"),
            Self::Csv => write!(f, "csv"),
            Self::SharePoint => write!(f, "sharepoint"),
            Self::Custom(s) => write!(f, "{s}"),
        }
    }
}

impl From<&str> for ConnectorType {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "salesforce" => Self::Salesforce,
            "sap" => Self::Sap,
            "totvs" => Self::Totvs,
            "bling" => Self::Bling,
            "shopify" => Self::Shopify,
            "woocommerce" => Self::WooCommerce,
            "mysql" => Self::MySql,
            "postgres" => Self::Postgres,
            "rest_api" | "rest" => Self::RestApi,
            "graphql" | "graph_ql" => Self::GraphQl,
            "google_sheets" | "sheets" => Self::GoogleSheets,
            "csv" => Self::Csv,
            "sharepoint" => Self::SharePoint,
            _ => Self::Custom(s.to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    None,
    ApiKey,
    Bearer,
    Basic,
    OAuth2,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncDirection {
    Pull,
    Push,
    Bidirectional,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub auth_type: AuthType,
    pub api_key: Option<String>,
    pub api_key_header: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub oauth2_client_id: Option<String>,
    pub oauth2_client_secret: Option<String>,
    pub oauth2_token_url: Option<String>,
    pub oauth2_scopes: Option<Vec<String>>,
    pub base_url: Option<String>,
    pub extra_headers: Option<Vec<(String, String)>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    pub external_field: String,
    pub internal_field: String,
    pub transform: Option<String>,
    pub is_primary_key: bool,
    pub default_value: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Option<serde_json::Value>,
    pub auth_type: AuthType,
    pub sync_direction: SyncDirection,
    pub field_mapping: Vec<FieldMapping>,
    pub schedule: Option<String>,
    pub pagination: Option<PaginationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationConfig {
    pub pagination_type: PaginationType,
    pub page_param: Option<String>,
    pub limit_param: Option<String>,
    pub limit: Option<i32>,
    pub results_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PaginationType {
    Offset,
    Page,
    Cursor,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorConfig {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub connector_type: ConnectorType,
    pub description: Option<String>,
    pub auth_config: AuthConfig,
    pub endpoints: Vec<EndpointConfig>,
    pub schedule: Option<String>,
    pub is_active: bool,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_sync_status: Option<String>,
    pub error_log: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SyncStatus {
    Success,
    Partial,
    Failed,
    Running,
    Pending,
}

impl std::fmt::Display for SyncStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Success => write!(f, "success"),
            Self::Partial => write!(f, "partial"),
            Self::Failed => write!(f, "failed"),
            Self::Running => write!(f, "running"),
            Self::Pending => write!(f, "pending"),
        }
    }
}

impl From<&str> for SyncStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "success" => Self::Success,
            "partial" => Self::Partial,
            "failed" => Self::Failed,
            "running" => Self::Running,
            _ => Self::Pending,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateConnectorRequest {
    pub name: String,
    pub connector_type: String,
    pub description: Option<String>,
    pub auth_config: AuthConfig,
    pub endpoints: Option<Vec<EndpointConfig>>,
    pub schedule: Option<String>,
    pub bot_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConnectorRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub auth_config: Option<AuthConfig>,
    pub endpoints: Option<Vec<EndpointConfig>>,
    pub schedule: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredSchema {
    pub object_name: String,
    pub fields: Vec<SchemaField>,
    pub total_count: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaField {
    pub name: String,
    pub field_type: String,
    pub required: bool,
    pub max_length: Option<i32>,
    pub values: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub status: SyncStatus,
    pub records_synced: i64,
    pub records_failed: i64,
    pub duration_ms: i64,
    pub error_message: Option<String>,
    pub endpoint_results: Vec<EndpointSyncResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointSyncResult {
    pub endpoint_name: String,
    pub status: SyncStatus,
    pub records_synced: i64,
    pub records_failed: i64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncLog {
    pub id: Uuid,
    pub connector_id: Uuid,
    pub endpoint_id: Option<Uuid>,
    pub status: SyncStatus,
    pub records_synced: i64,
    pub records_failed: i64,
    pub duration_ms: i64,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub connector_type: ConnectorType,
    pub icon: String,
    pub auth_type: AuthType,
    pub auth_help: String,
    pub default_endpoints: Vec<EndpointConfig>,
    pub default_schedule: Option<String>,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorRow {
    pub id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub connector_type: String,
    pub description: Option<String>,
    pub auth_config: serde_json::Value,
    pub schedule: Option<String>,
    pub is_active: bool,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_sync_status: Option<String>,
    pub error_log: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointRow {
    pub id: Uuid,
    pub connector_id: Uuid,
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Option<serde_json::Value>,
    pub sync_direction: String,
    pub field_mapping: serde_json::Value,
    pub schedule: Option<String>,
    pub pagination: Option<serde_json::Value>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncLogRow {
    pub id: Uuid,
    pub connector_id: Uuid,
    pub endpoint_id: Option<Uuid>,
    pub status: String,
    pub records_synced: i64,
    pub records_failed: i64,
    pub duration_ms: i64,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}
