use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::shared::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub layout: DashboardLayout,
    pub widgets: Vec<Widget>,
    pub data_sources: Vec<DataSourceRef>,
    pub filters: Vec<DashboardFilter>,
    pub refresh_interval: Option<i32>,
    pub is_public: bool,
    pub is_template: bool,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardLayout {
    pub columns: i32,
    pub row_height: i32,
    pub gap: i32,
    pub responsive_breakpoints: Option<ResponsiveBreakpoints>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsiveBreakpoints {
    pub mobile: i32,
    pub tablet: i32,
    pub desktop: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub id: Uuid,
    pub widget_type: WidgetType,
    pub title: String,
    pub position: WidgetPosition,
    pub config: WidgetConfig,
    pub data_query: Option<DataQuery>,
    pub style: Option<WidgetStyle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WidgetType {
    LineChart,
    BarChart,
    PieChart,
    DonutChart,
    AreaChart,
    ScatterPlot,
    Heatmap,
    Table,
    Kpi,
    Gauge,
    Map,
    Text,
    Image,
    Iframe,
    Filter,
    DateRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetConfig {
    pub chart_config: Option<ChartConfig>,
    pub table_config: Option<TableConfig>,
    pub kpi_config: Option<KpiConfig>,
    pub map_config: Option<MapConfig>,
    pub text_content: Option<String>,
    pub image_url: Option<String>,
    pub iframe_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    pub x_axis: String,
    pub y_axis: Vec<String>,
    pub series: Vec<ChartSeries>,
    pub legend_position: Option<String>,
    pub show_labels: bool,
    pub stacked: bool,
    pub colors: Option<Vec<String>>,
    pub animations: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSeries {
    pub name: String,
    pub field: String,
    pub color: Option<String>,
    pub series_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableConfig {
    pub columns: Vec<TableColumn>,
    pub page_size: i32,
    pub sortable: bool,
    pub filterable: bool,
    pub export_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableColumn {
    pub field: String,
    pub header: String,
    pub width: Option<i32>,
    pub format: Option<ColumnFormat>,
    pub sortable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ColumnFormat {
    Text,
    Number,
    Currency,
    Percentage,
    Date,
    DateTime,
    Boolean,
    Link,
    Image,
    Progress,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiConfig {
    pub value_field: String,
    pub comparison_field: Option<String>,
    pub comparison_type: Option<ComparisonType>,
    pub format: ColumnFormat,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
    pub thresholds: Option<KpiThresholds>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonType {
    PreviousPeriod,
    PreviousYear,
    Target,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KpiThresholds {
    pub good: f64,
    pub warning: f64,
    pub bad: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapConfig {
    pub latitude_field: String,
    pub longitude_field: String,
    pub value_field: Option<String>,
    pub label_field: Option<String>,
    pub map_style: String,
    pub zoom: i32,
    pub center: Option<MapCenter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapCenter {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetStyle {
    pub background_color: Option<String>,
    pub border_color: Option<String>,
    pub border_radius: Option<i32>,
    pub padding: Option<i32>,
    pub font_size: Option<i32>,
    pub text_color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQuery {
    pub source_id: Uuid,
    pub query_type: QueryType,
    pub sql: Option<String>,
    pub table: Option<String>,
    pub fields: Option<Vec<String>>,
    pub filters: Option<Vec<QueryFilter>>,
    pub group_by: Option<Vec<String>>,
    pub order_by: Option<Vec<OrderBy>>,
    pub limit: Option<i32>,
    pub aggregations: Option<Vec<Aggregation>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QueryType {
    Sql,
    Table,
    Api,
    Realtime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryFilter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum FilterOperator {
    Equals,
    NotEquals,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    Contains,
    StartsWith,
    EndsWith,
    In,
    NotIn,
    Between,
    IsNull,
    IsNotNull,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBy {
    pub field: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Aggregation {
    pub field: String,
    pub function: AggregateFunction,
    pub alias: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AggregateFunction {
    Sum,
    Avg,
    Min,
    Max,
    Count,
    CountDistinct,
    First,
    Last,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardFilter {
    pub id: Uuid,
    pub name: String,
    pub field: String,
    pub filter_type: DashboardFilterType,
    pub default_value: Option<serde_json::Value>,
    pub options: Option<Vec<FilterOption>>,
    pub linked_widgets: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DashboardFilterType {
    Text,
    Number,
    Date,
    DateRange,
    Select,
    MultiSelect,
    Checkbox,
    Slider,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterOption {
    pub value: serde_json::Value,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceRef {
    pub id: Uuid,
    pub name: String,
    pub source_type: DataSourceType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSource {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub source_type: DataSourceType,
    pub connection: DataSourceConnection,
    pub schema: Option<DataSourceSchema>,
    pub refresh_schedule: Option<String>,
    pub last_sync: Option<DateTime<Utc>>,
    pub status: DataSourceStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataSourceType {
    Postgresql,
    Mysql,
    Sqlserver,
    Oracle,
    Mongodb,
    Bigquery,
    Snowflake,
    Redshift,
    Elasticsearch,
    RestApi,
    GraphqlApi,
    Csv,
    Excel,
    GoogleSheets,
    Airtable,
    InternalTables,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceConnection {
    pub host: Option<String>,
    pub port: Option<i32>,
    pub database: Option<String>,
    pub username: Option<String>,
    pub password_vault_key: Option<String>,
    pub ssl: Option<bool>,
    pub url: Option<String>,
    pub api_key_vault_key: Option<String>,
    pub headers: Option<HashMap<String, String>>,
    pub connection_string_vault_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSourceSchema {
    pub tables: Vec<TableSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableSchema {
    pub name: String,
    pub columns: Vec<ColumnSchema>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnSchema {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub primary_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataSourceStatus {
    Active,
    Inactive,
    Error,
    Syncing,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationalQuery {
    pub id: Uuid,
    pub dashboard_id: Option<Uuid>,
    pub user_id: Uuid,
    pub natural_language: String,
    pub generated_query: Option<DataQuery>,
    pub result_widget: Option<Widget>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ListDashboardsQuery {
    pub owner_id: Option<Uuid>,
    pub tag: Option<String>,
    pub is_template: Option<bool>,
    pub search: Option<String>,
    pub limit: Option<i32>,
    pub offset: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDashboardRequest {
    pub name: String,
    pub description: Option<String>,
    pub layout: Option<DashboardLayout>,
    pub is_public: Option<bool>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateDashboardRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub layout: Option<DashboardLayout>,
    pub is_public: Option<bool>,
    pub refresh_interval: Option<i32>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct AddWidgetRequest {
    pub widget_type: WidgetType,
    pub title: String,
    pub position: WidgetPosition,
    pub config: WidgetConfig,
    pub data_query: Option<DataQuery>,
    pub style: Option<WidgetStyle>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateWidgetRequest {
    pub title: Option<String>,
    pub position: Option<WidgetPosition>,
    pub config: Option<WidgetConfig>,
    pub data_query: Option<DataQuery>,
    pub style: Option<WidgetStyle>,
}

#[derive(Debug, Deserialize)]
pub struct CreateDataSourceRequest {
    pub name: String,
    pub description: Option<String>,
    pub source_type: DataSourceType,
    pub connection: DataSourceConnection,
}

#[derive(Debug, Deserialize)]
pub struct ConversationalQueryRequest {
    pub query: String,
    pub data_source_id: Option<Uuid>,
    pub context: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ConversationalQueryResponse {
    pub query: ConversationalQuery,
    pub data: Option<serde_json::Value>,
    pub suggested_visualization: Option<WidgetType>,
    pub explanation: String,
}

#[derive(Debug, Serialize)]
pub struct WidgetData {
    pub widget_id: Uuid,
    pub data: serde_json::Value,
    pub fetched_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct DashboardsService {}

impl DashboardsService {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn list_dashboards(
        &self,
        _organization_id: Uuid,
        _query: &ListDashboardsQuery,
    ) -> Result<Vec<Dashboard>, DashboardsError> {
        Ok(vec![])
    }

    pub async fn create_dashboard(
        &self,
        organization_id: Uuid,
        owner_id: Uuid,
        req: CreateDashboardRequest,
    ) -> Result<Dashboard, DashboardsError> {
        let now = Utc::now();
        Ok(Dashboard {
            id: Uuid::new_v4(),
            organization_id,
            owner_id,
            name: req.name,
            description: req.description,
            layout: req.layout.unwrap_or(DashboardLayout {
                columns: 12,
                row_height: 80,
                gap: 16,
                responsive_breakpoints: None,
            }),
            widgets: vec![],
            data_sources: vec![],
            filters: vec![],
            refresh_interval: None,
            is_public: req.is_public.unwrap_or(false),
            is_template: false,
            tags: req.tags.unwrap_or_default(),
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn get_dashboard(
        &self,
        _organization_id: Uuid,
        _dashboard_id: Uuid,
    ) -> Result<Option<Dashboard>, DashboardsError> {
        Ok(None)
    }

    pub async fn update_dashboard(
        &self,
        _organization_id: Uuid,
        _dashboard_id: Uuid,
        _req: UpdateDashboardRequest,
    ) -> Result<Dashboard, DashboardsError> {
        Err(DashboardsError::NotFound("Dashboard not found".to_string()))
    }

    pub async fn delete_dashboard(
        &self,
        _organization_id: Uuid,
        _dashboard_id: Uuid,
    ) -> Result<(), DashboardsError> {
        Ok(())
    }

    pub async fn duplicate_dashboard(
        &self,
        organization_id: Uuid,
        dashboard_id: Uuid,
        owner_id: Uuid,
        new_name: String,
    ) -> Result<Dashboard, DashboardsError> {
        let original = self.get_dashboard(organization_id, dashboard_id).await?;
        match original {
            Some(mut dash) => {
                dash.id = Uuid::new_v4();
                dash.name = new_name;
                dash.owner_id = owner_id;
                dash.is_template = false;
                dash.created_at = Utc::now();
                dash.updated_at = Utc::now();
                Ok(dash)
            }
            None => Err(DashboardsError::NotFound("Dashboard not found".to_string())),
        }
    }

    pub async fn add_widget(
        &self,
        _organization_id: Uuid,
        _dashboard_id: Uuid,
        req: AddWidgetRequest,
    ) -> Result<Widget, DashboardsError> {
        Ok(Widget {
            id: Uuid::new_v4(),
            widget_type: req.widget_type,
            title: req.title,
            position: req.position,
            config: req.config,
            data_query: req.data_query,
            style: req.style,
        })
    }

    pub async fn update_widget(
        &self,
        _organization_id: Uuid,
        _dashboard_id: Uuid,
        _widget_id: Uuid,
        _req: UpdateWidgetRequest,
    ) -> Result<Widget, DashboardsError> {
        Err(DashboardsError::NotFound("Widget not found".to_string()))
    }

    pub async fn delete_widget(
        &self,
        _organization_id: Uuid,
        _dashboard_id: Uuid,
        _widget_id: Uuid,
    ) -> Result<(), DashboardsError> {
        Ok(())
    }

    pub async fn get_widget_data(
        &self,
        _organization_id: Uuid,
        _dashboard_id: Uuid,
        widget_id: Uuid,
    ) -> Result<WidgetData, DashboardsError> {
        Ok(WidgetData {
            widget_id,
            data: serde_json::json!([]),
            fetched_at: Utc::now(),
        })
    }

    pub async fn list_data_sources(
        &self,
        _organization_id: Uuid,
    ) -> Result<Vec<DataSource>, DashboardsError> {
        Ok(get_builtin_data_sources())
    }

    pub async fn create_data_source(
        &self,
        organization_id: Uuid,
        req: CreateDataSourceRequest,
    ) -> Result<DataSource, DashboardsError> {
        let now = Utc::now();
        Ok(DataSource {
            id: Uuid::new_v4(),
            organization_id,
            name: req.name,
            description: req.description,
            source_type: req.source_type,
            connection: req.connection,
            schema: None,
            refresh_schedule: None,
            last_sync: None,
            status: DataSourceStatus::Active,
            created_at: now,
            updated_at: now,
        })
    }

    pub async fn test_data_source(
        &self,
        _organization_id: Uuid,
        _data_source_id: Uuid,
    ) -> Result<bool, DashboardsError> {
        Ok(true)
    }

    pub async fn sync_data_source(
        &self,
        _organization_id: Uuid,
        _data_source_id: Uuid,
    ) -> Result<DataSource, DashboardsError> {
        Err(DashboardsError::NotFound("Data source not found".to_string()))
    }

    pub async fn delete_data_source(
        &self,
        _organization_id: Uuid,
        _data_source_id: Uuid,
    ) -> Result<(), DashboardsError> {
        Ok(())
    }

    pub async fn conversational_query(
        &self,
        _organization_id: Uuid,
        user_id: Uuid,
        req: ConversationalQueryRequest,
    ) -> Result<ConversationalQueryResponse, DashboardsError> {
        let query = ConversationalQuery {
            id: Uuid::new_v4(),
            dashboard_id: None,
            user_id,
            natural_language: req.query.clone(),
            generated_query: None,
            result_widget: None,
            created_at: Utc::now(),
        };

        let (suggested_viz, explanation) = self.analyze_query_intent(&req.query);

        Ok(ConversationalQueryResponse {
            query,
            data: Some(serde_json::json!([])),
            suggested_visualization: Some(suggested_viz),
            explanation,
        })
    }

    fn analyze_query_intent(&self, query: &str) -> (WidgetType, String) {
        let query_lower = query.to_lowercase();

        if query_lower.contains("trend") || query_lower.contains("over time") || query_lower.contains("timeline") {
            (WidgetType::LineChart, "Showing data as a line chart to visualize trends over time".to_string())
        } else if query_lower.contains("compare") || query_lower.contains("by category") || query_lower.contains("breakdown") {
            (WidgetType::BarChart, "Using a bar chart to compare values across categories".to_string())
        } else if query_lower.contains("distribution") || query_lower.contains("percentage") || query_lower.contains("share") {
            (WidgetType::PieChart, "Displaying distribution as a pie chart".to_string())
        } else if query_lower.contains("total") || query_lower.contains("count") || query_lower.contains("sum") || query_lower.contains("kpi") {
            (WidgetType::Kpi, "Showing as a KPI card for quick insight".to_string())
        } else if query_lower.contains("table") || query_lower.contains("list") || query_lower.contains("details") {
            (WidgetType::Table, "Presenting data in a table format for detailed view".to_string())
        } else if query_lower.contains("map") || query_lower.contains("location") || query_lower.contains("geographic") {
            (WidgetType::Map, "Visualizing geographic data on a map".to_string())
        } else if query_lower.contains("gauge") || query_lower.contains("progress") || query_lower.contains("target") {
            (WidgetType::Gauge, "Showing progress toward a target as a gauge".to_string())
        } else {
            (WidgetType::BarChart, "Defaulting to bar chart for general visualization".to_string())
        }
    }

    pub async fn save_query_as_widget(
        &self,
        _organization_id: Uuid,
        _dashboard_id: Uuid,
        _query_id: Uuid,
    ) -> Result<Widget, DashboardsError> {
        Err(DashboardsError::NotFound("Query not found".to_string()))
    }

    pub async fn get_templates(
        &self,
        _organization_id: Uuid,
    ) -> Result<Vec<Dashboard>, DashboardsError> {
        Ok(vec![])
    }

    pub async fn export_dashboard(
        &self,
        _organization_id: Uuid,
        _dashboard_id: Uuid,
        format: ExportFormat,
    ) -> Result<Vec<u8>, DashboardsError> {
        match format {
            ExportFormat::Pdf => Ok(vec![]),
            ExportFormat::Png => Ok(vec![]),
            ExportFormat::Json => Ok(vec![]),
        }
    }
}

impl Default for DashboardsService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Pdf,
    Png,
    Json,
}

fn get_builtin_data_sources() -> Vec<DataSource> {
    let now = Utc::now();
    vec![
        DataSource {
            id: Uuid::new_v4(),
            organization_id: Uuid::nil(),
            name: "Internal Tables".to_string(),
            description: Some("Data from GB app tables".to_string()),
            source_type: DataSourceType::InternalTables,
            connection: DataSourceConnection {
                host: None,
                port: None,
                database: None,
                username: None,
                password_vault_key: None,
                ssl: None,
                url: None,
                api_key_vault_key: None,
                headers: None,
                connection_string_vault_key: None,
            },
            schema: None,
            refresh_schedule: None,
            last_sync: Some(now),
            status: DataSourceStatus::Active,
            created_at: now,
            updated_at: now,
        },
    ]
}

#[derive(Debug, thiserror::Error)]
pub enum DashboardsError {
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Unauthorized: {0}")]
    Unauthorized(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Database error: {0}")]
    Database(String),
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Query error: {0}")]
    Query(String),
}

impl IntoResponse for DashboardsError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        let (status, message) = match &self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            Self::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::Database(msg) | Self::Connection(msg) | Self::Query(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

pub async fn handle_list_dashboards(
    State(_state): State<Arc<AppState>>,
    Query(query): Query<ListDashboardsQuery>,
) -> Result<Json<Vec<Dashboard>>, DashboardsError> {
    let service = DashboardsService::new();
    let org_id = Uuid::nil();
    let dashboards = service.list_dashboards(org_id, &query).await?;
    Ok(Json(dashboards))
}

pub async fn handle_create_dashboard(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreateDashboardRequest>,
) -> Result<Json<Dashboard>, DashboardsError> {
    let service = DashboardsService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    let dashboard = service.create_dashboard(org_id, user_id, req).await?;
    Ok(Json(dashboard))
}

pub async fn handle_get_dashboard(
    State(_state): State<Arc<AppState>>,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Json<Option<Dashboard>>, DashboardsError> {
    let service = DashboardsService::new();
    let org_id = Uuid::nil();
    let dashboard = service.get_dashboard(org_id, dashboard_id).await?;
    Ok(Json(dashboard))
}

pub async fn handle_update_dashboard(
    State(_state): State<Arc<AppState>>,
    Path(dashboard_id): Path<Uuid>,
    Json(req): Json<UpdateDashboardRequest>,
) -> Result<Json<Dashboard>, DashboardsError> {
    let service = DashboardsService::new();
    let org_id = Uuid::nil();
    let dashboard = service.update_dashboard(org_id, dashboard_id, req).await?;
    Ok(Json(dashboard))
}

pub async fn handle_delete_dashboard(
    State(_state): State<Arc<AppState>>,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, DashboardsError> {
    let service = DashboardsService::new();
    let org_id = Uuid::nil();
    service.delete_dashboard(org_id, dashboard_id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_add_widget(
    State(_state): State<Arc<AppState>>,
    Path(dashboard_id): Path<Uuid>,
    Json(req): Json<AddWidgetRequest>,
) -> Result<Json<Widget>, DashboardsError> {
    let service = DashboardsService::new();
    let org_id = Uuid::nil();
    let widget = service.add_widget(org_id, dashboard_id, req).await?;
    Ok(Json(widget))
}

pub async fn handle_update_widget(
    State(_state): State<Arc<AppState>>,
    Path((dashboard_id, widget_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateWidgetRequest>,
) -> Result<Json<Widget>, DashboardsError> {
    let service = DashboardsService::new();
    let org_id = Uuid::nil();
    let widget = service.update_widget(org_id, dashboard_id, widget_id, req).await?;
    Ok(Json(widget))
}

pub async fn handle_delete_widget(
    State(_state): State<Arc<AppState>>,
    Path((dashboard_id, widget_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, DashboardsError> {
    let service = DashboardsService::new();
    let org_id = Uuid::nil();
    service.delete_widget(org_id, dashboard_id, widget_id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_get_widget_data(
    State(_state): State<Arc<AppState>>,
    Path((dashboard_id, widget_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<WidgetData>, DashboardsError> {
    let service = DashboardsService::new();
    let org_id = Uuid::nil();
    let data = service.get_widget_data(org_id, dashboard_id, widget_id).await?;
    Ok(Json(data))
}

pub async fn handle_list_data_sources(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<DataSource>>, DashboardsError> {
    let service = DashboardsService::new();
    let org_id = Uuid::nil();
    let sources = service.list_data_sources(org_id).await?;
    Ok(Json(sources))
}

pub async fn handle_create_data_source(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<CreateDataSourceRequest>,
) -> Result<Json<DataSource>, DashboardsError> {
    let service = DashboardsService::new();
    let org_id = Uuid::nil();
    let source = service.create_data_source(org_id, req).await?;
    Ok(Json(source))
}

pub async fn handle_test_data_source(
    State(_state): State<Arc<AppState>>,
    Path(source_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, DashboardsError> {
    let service = DashboardsService::new();
    let org_id = Uuid::nil();
    let success = service.test_data_source(org_id, source_id).await?;
    Ok(Json(serde_json::json!({ "success": success })))
}

pub async fn handle_delete_data_source(
    State(_state): State<Arc<AppState>>,
    Path(source_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, DashboardsError> {
    let service = DashboardsService::new();
    let org_id = Uuid::nil();
    service.delete_data_source(org_id, source_id).await?;
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_conversational_query(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<ConversationalQueryRequest>,
) -> Result<Json<ConversationalQueryResponse>, DashboardsError> {
    let service = DashboardsService::new();
    let org_id = Uuid::nil();
    let user_id = Uuid::nil();
    let response = service.conversational_query(org_id, user_id, req).await?;
    Ok(Json(response))
}

pub async fn handle_get_templates(
    State(_state): State<Arc<AppState>>,
) -> Result<Json<Vec<Dashboard>>, DashboardsError> {
    let service = DashboardsService::new();
    let org_id = Uuid::nil();
    let templates = service.get_templates(org_id).await?;
    Ok(Json(templates))
}

pub fn configure_dashboards_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/dashboards", get(handle_list_dashboards))
        .route("/api/dashboards", post(handle_create_dashboard))
        .route("/api/dashboards/templates", get(handle_get_templates))
        .route("/api/dashboards/:id", get(handle_get_dashboard))
        .route("/api/dashboards/:id", put(handle_update_dashboard))
        .route("/api/dashboards/:id", delete(handle_delete_dashboard))
        .route("/api/dashboards/:id/widgets", post(handle_add_widget))
        .route(
            "/api/dashboards/:id/widgets/:widget_id",
            put(handle_update_widget),
        )
        .route(
            "/api/dashboards/:id/widgets/:widget_id",
            delete(handle_delete_widget),
        )
        .route(
            "/api/dashboards/:id/widgets/:widget_id/data",
            get(handle_get_widget_data),
        )
        .route("/api/dashboards/sources", get(handle_list_data_sources))
        .route("/api/dashboards/sources", post(handle_create_data_source))
        .route(
            "/api/dashboards/sources/:id/test",
            post(handle_test_data_source),
        )
        .route(
            "/api/dashboards/sources/:id",
            delete(handle_delete_data_source),
        )
        .route("/api/dashboards/query", post(handle_conversational_query))
}
