use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::{delete, get, post, put},
    Json, Router,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

use crate::bot::get_default_bot;
use crate::core::shared::schema::{
    conversational_queries, dashboard_data_sources, dashboard_filters, dashboard_widgets,
    dashboards,
};
use crate::shared::state::AppState;

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = dashboards)]
pub struct DbDashboard {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub owner_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub layout: serde_json::Value,
    pub refresh_interval: Option<i32>,
    pub is_public: bool,
    pub is_template: bool,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = dashboard_widgets)]
pub struct DbWidget {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub widget_type: String,
    pub title: String,
    pub position_x: i32,
    pub position_y: i32,
    pub width: i32,
    pub height: i32,
    pub config: serde_json::Value,
    pub data_query: Option<serde_json::Value>,
    pub style: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, AsChangeset, Serialize, Deserialize)]
#[diesel(table_name = dashboard_data_sources)]
pub struct DbDataSource {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub source_type: String,
    pub connection: serde_json::Value,
    pub schema_definition: serde_json::Value,
    pub refresh_schedule: Option<String>,
    pub last_sync: Option<DateTime<Utc>>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = dashboard_filters)]
pub struct DbFilter {
    pub id: Uuid,
    pub dashboard_id: Uuid,
    pub name: String,
    pub field: String,
    pub filter_type: String,
    pub default_value: Option<serde_json::Value>,
    pub options: serde_json::Value,
    pub linked_widgets: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Queryable, Insertable, Serialize, Deserialize)]
#[diesel(table_name = conversational_queries)]
pub struct DbConversationalQuery {
    pub id: Uuid,
    pub org_id: Uuid,
    pub bot_id: Uuid,
    pub dashboard_id: Option<Uuid>,
    pub user_id: Uuid,
    pub natural_language: String,
    pub generated_query: Option<String>,
    pub result_widget_config: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

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

impl Default for DashboardLayout {
    fn default() -> Self {
        Self {
            columns: 12,
            row_height: 80,
            gap: 16,
            responsive_breakpoints: None,
        }
    }
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

impl std::fmt::Display for WidgetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::LineChart => "line_chart",
            Self::BarChart => "bar_chart",
            Self::PieChart => "pie_chart",
            Self::DonutChart => "donut_chart",
            Self::AreaChart => "area_chart",
            Self::ScatterPlot => "scatter_plot",
            Self::Heatmap => "heatmap",
            Self::Table => "table",
            Self::Kpi => "kpi",
            Self::Gauge => "gauge",
            Self::Map => "map",
            Self::Text => "text",
            Self::Image => "image",
            Self::Iframe => "iframe",
            Self::Filter => "filter",
            Self::DateRange => "date_range",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for WidgetType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "line_chart" => Ok(Self::LineChart),
            "bar_chart" => Ok(Self::BarChart),
            "pie_chart" => Ok(Self::PieChart),
            "donut_chart" => Ok(Self::DonutChart),
            "area_chart" => Ok(Self::AreaChart),
            "scatter_plot" => Ok(Self::ScatterPlot),
            "heatmap" => Ok(Self::Heatmap),
            "table" => Ok(Self::Table),
            "kpi" => Ok(Self::Kpi),
            "gauge" => Ok(Self::Gauge),
            "map" => Ok(Self::Map),
            "text" => Ok(Self::Text),
            "image" => Ok(Self::Image),
            "iframe" => Ok(Self::Iframe),
            "filter" => Ok(Self::Filter),
            "date_range" => Ok(Self::DateRange),
            _ => Err(format!("Unknown widget type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    pub x_axis: Option<String>,
    pub y_axis: Option<String>,
    pub series: Vec<ChartSeries>,
    pub legend_position: Option<String>,
    pub show_labels: bool,
    pub stacked: bool,
    pub colors: Vec<String>,
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
    pub format: Option<String>,
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
    pub map_style: Option<String>,
    pub zoom: Option<i32>,
    pub center: Option<MapCenter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapCenter {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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
    pub source_id: Option<Uuid>,
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
    pub alias: Option<String>,
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
    pub options: Vec<FilterOption>,
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

impl std::fmt::Display for DashboardFilterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Text => "text",
            Self::Number => "number",
            Self::Date => "date",
            Self::DateRange => "date_range",
            Self::Select => "select",
            Self::MultiSelect => "multi_select",
            Self::Checkbox => "checkbox",
            Self::Slider => "slider",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for DashboardFilterType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "text" => Ok(Self::Text),
            "number" => Ok(Self::Number),
            "date" => Ok(Self::Date),
            "date_range" => Ok(Self::DateRange),
            "select" => Ok(Self::Select),
            "multi_select" => Ok(Self::MultiSelect),
            "checkbox" => Ok(Self::Checkbox),
            "slider" => Ok(Self::Slider),
            _ => Err(format!("Unknown filter type: {s}")),
        }
    }
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

impl std::fmt::Display for DataSourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Postgresql => "postgresql",
            Self::Mysql => "mysql",
            Self::Sqlserver => "sqlserver",
            Self::Oracle => "oracle",
            Self::Mongodb => "mongodb",
            Self::Bigquery => "bigquery",
            Self::Snowflake => "snowflake",
            Self::Redshift => "redshift",
            Self::Elasticsearch => "elasticsearch",
            Self::RestApi => "rest_api",
            Self::GraphqlApi => "graphql_api",
            Self::Csv => "csv",
            Self::Excel => "excel",
            Self::GoogleSheets => "google_sheets",
            Self::Airtable => "airtable",
            Self::InternalTables => "internal_tables",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for DataSourceType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "postgresql" => Ok(Self::Postgresql),
            "mysql" => Ok(Self::Mysql),
            "sqlserver" => Ok(Self::Sqlserver),
            "oracle" => Ok(Self::Oracle),
            "mongodb" => Ok(Self::Mongodb),
            "bigquery" => Ok(Self::Bigquery),
            "snowflake" => Ok(Self::Snowflake),
            "redshift" => Ok(Self::Redshift),
            "elasticsearch" => Ok(Self::Elasticsearch),
            "rest_api" => Ok(Self::RestApi),
            "graphql_api" => Ok(Self::GraphqlApi),
            "csv" => Ok(Self::Csv),
            "excel" => Ok(Self::Excel),
            "google_sheets" => Ok(Self::GoogleSheets),
            "airtable" => Ok(Self::Airtable),
            "internal_tables" => Ok(Self::InternalTables),
            _ => Err(format!("Unknown data source type: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
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

impl std::fmt::Display for DataSourceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Active => "active",
            Self::Inactive => "inactive",
            Self::Error => "error",
            Self::Syncing => "syncing",
        };
        write!(f, "{s}")
    }
}

impl std::str::FromStr for DataSourceStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Self::Active),
            "inactive" => Ok(Self::Inactive),
            "error" => Ok(Self::Error),
            "syncing" => Ok(Self::Syncing),
            _ => Err(format!("Unknown status: {s}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationalQuery {
    pub id: Uuid,
    pub dashboard_id: Option<Uuid>,
    pub user_id: Uuid,
    pub natural_language: String,
    pub generated_query: Option<String>,
    pub result_widget: Option<Widget>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct ListDashboardsQuery {
    pub owner_id: Option<Uuid>,
    pub tag: Option<String>,
    pub is_template: Option<bool>,
    pub search: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
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
    pub context: Option<HashMap<String, String>>,
}

#[derive(Debug, Serialize)]
pub struct ConversationalQueryResponse {
    pub query: ConversationalQuery,
    pub data: Option<serde_json::Value>,
    pub suggested_visualization: Option<WidgetType>,
    pub explanation: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct WidgetData {
    pub widget_id: Uuid,
    pub data: serde_json::Value,
    pub fetched_at: DateTime<Utc>,
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
    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for DashboardsError {
    fn into_response(self) -> axum::response::Response {
        use axum::http::StatusCode;
        let (status, message) = match &self {
            Self::NotFound(msg) => (StatusCode::NOT_FOUND, msg.clone()),
            Self::Unauthorized(msg) => (StatusCode::UNAUTHORIZED, msg.clone()),
            Self::Validation(msg) => (StatusCode::BAD_REQUEST, msg.clone()),
            Self::Database(msg) | Self::Connection(msg) | Self::Query(msg) | Self::Internal(msg) => {
                (StatusCode::INTERNAL_SERVER_ERROR, msg.clone())
            }
        };
        (status, Json(serde_json::json!({ "error": message }))).into_response()
    }
}

fn db_dashboard_to_dashboard(
    db: DbDashboard,
    widgets: Vec<Widget>,
    filters: Vec<DashboardFilter>,
) -> Dashboard {
    let layout: DashboardLayout =
        serde_json::from_value(db.layout).unwrap_or_default();

    Dashboard {
        id: db.id,
        organization_id: db.org_id,
        owner_id: db.owner_id,
        name: db.name,
        description: db.description,
        layout,
        widgets,
        data_sources: vec![],
        filters,
        refresh_interval: db.refresh_interval,
        is_public: db.is_public,
        is_template: db.is_template,
        tags: db.tags,
        created_at: db.created_at,
        updated_at: db.updated_at,
    }
}

fn db_widget_to_widget(db: DbWidget) -> Widget {
    let widget_type: WidgetType = db.widget_type.parse().unwrap_or(WidgetType::Text);
    let config: WidgetConfig = serde_json::from_value(db.config).unwrap_or_default();
    let data_query: Option<DataQuery> = db.data_query.and_then(|v| serde_json::from_value(v).ok());
    let style: Option<WidgetStyle> = serde_json::from_value(db.style).ok();

    Widget {
        id: db.id,
        widget_type,
        title: db.title,
        position: WidgetPosition {
            x: db.position_x,
            y: db.position_y,
            width: db.width,
            height: db.height,
        },
        config,
        data_query,
        style,
    }
}

fn db_filter_to_filter(db: DbFilter) -> DashboardFilter {
    let filter_type: DashboardFilterType = db.filter_type.parse().unwrap_or(DashboardFilterType::Text);
    let options: Vec<FilterOption> = serde_json::from_value(db.options).unwrap_or_default();
    let linked_widgets: Vec<Uuid> = serde_json::from_value(db.linked_widgets).unwrap_or_default();

    DashboardFilter {
        id: db.id,
        name: db.name,
        field: db.field,
        filter_type,
        default_value: db.default_value,
        options,
        linked_widgets,
    }
}

fn db_data_source_to_data_source(db: DbDataSource) -> DataSource {
    let source_type: DataSourceType = db.source_type.parse().unwrap_or(DataSourceType::InternalTables);
    let connection: DataSourceConnection = serde_json::from_value(db.connection).unwrap_or_default();
    let schema: Option<DataSourceSchema> = serde_json::from_value(db.schema_definition).ok();
    let status: DataSourceStatus = db.status.parse().unwrap_or(DataSourceStatus::Inactive);

    DataSource {
        id: db.id,
        organization_id: db.org_id,
        name: db.name,
        description: db.description,
        source_type,
        connection,
        schema,
        refresh_schedule: db.refresh_schedule,
        last_sync: db.last_sync,
        status,
        created_at: db.created_at,
        updated_at: db.updated_at,
    }
}

pub async fn handle_list_dashboards(
    State(state): State<Arc<AppState>>,
    Query(query): Query<ListDashboardsQuery>,
) -> Result<Json<Vec<Dashboard>>, DashboardsError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| DashboardsError::Database(e.to_string()))?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let limit = query.limit.unwrap_or(50);
        let offset = query.offset.unwrap_or(0);

        let mut db_query = dashboards::table
            .filter(dashboards::bot_id.eq(bot_id))
            .into_boxed();

        if let Some(owner_id) = query.owner_id {
            db_query = db_query.filter(dashboards::owner_id.eq(owner_id));
        }

        if let Some(is_template) = query.is_template {
            db_query = db_query.filter(dashboards::is_template.eq(is_template));
        }

        if let Some(ref search) = query.search {
            let term = format!("%{search}%");
            db_query = db_query.filter(dashboards::name.ilike(term));
        }

        let db_dashboards: Vec<DbDashboard> = db_query
            .order(dashboards::created_at.desc())
            .offset(offset)
            .limit(limit)
            .load(&mut conn)
            .map_err(|e| DashboardsError::Database(e.to_string()))?;

        let mut result_dashboards = Vec::new();
        for db_dash in db_dashboards {
            let dash_id = db_dash.id;
            let widgets_db: Vec<DbWidget> = dashboard_widgets::table
                .filter(dashboard_widgets::dashboard_id.eq(dash_id))
                .load(&mut conn)
                .unwrap_or_default();
            let filters_db: Vec<DbFilter> = dashboard_filters::table
                .filter(dashboard_filters::dashboard_id.eq(dash_id))
                .load(&mut conn)
                .unwrap_or_default();

            let widgets: Vec<Widget> = widgets_db.into_iter().map(db_widget_to_widget).collect();
            let filters: Vec<DashboardFilter> = filters_db.into_iter().map(db_filter_to_filter).collect();

            result_dashboards.push(db_dashboard_to_dashboard(db_dash, widgets, filters));
        }

        Ok::<_, DashboardsError>(result_dashboards)
    })
    .await
    .map_err(|e| DashboardsError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_create_dashboard(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateDashboardRequest>,
) -> Result<Json<Dashboard>, DashboardsError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| DashboardsError::Database(e.to_string()))?;
        let (bot_id, org_id) = get_default_bot(&mut conn);
        let now = Utc::now();

        let layout = req.layout.unwrap_or_default();
        let layout_json = serde_json::to_value(&layout).unwrap_or_default();

        let db_dashboard = DbDashboard {
            id: Uuid::new_v4(),
            org_id,
            bot_id,
            owner_id: Uuid::nil(),
            name: req.name,
            description: req.description,
            layout: layout_json,
            refresh_interval: None,
            is_public: req.is_public.unwrap_or(false),
            is_template: false,
            tags: req.tags.unwrap_or_default(),
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(dashboards::table)
            .values(&db_dashboard)
            .execute(&mut conn)
            .map_err(|e| DashboardsError::Database(e.to_string()))?;

        Ok::<_, DashboardsError>(db_dashboard_to_dashboard(db_dashboard, vec![], vec![]))
    })
    .await
    .map_err(|e| DashboardsError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_get_dashboard(
    State(state): State<Arc<AppState>>,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Json<Option<Dashboard>>, DashboardsError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| DashboardsError::Database(e.to_string()))?;

        let db_dash: Option<DbDashboard> = dashboards::table
            .find(dashboard_id)
            .first(&mut conn)
            .optional()
            .map_err(|e| DashboardsError::Database(e.to_string()))?;

        match db_dash {
            Some(db) => {
                let widgets_db: Vec<DbWidget> = dashboard_widgets::table
                    .filter(dashboard_widgets::dashboard_id.eq(dashboard_id))
                    .load(&mut conn)
                    .unwrap_or_default();
                let filters_db: Vec<DbFilter> = dashboard_filters::table
                    .filter(dashboard_filters::dashboard_id.eq(dashboard_id))
                    .load(&mut conn)
                    .unwrap_or_default();

                let widgets: Vec<Widget> = widgets_db.into_iter().map(db_widget_to_widget).collect();
                let filters: Vec<DashboardFilter> = filters_db.into_iter().map(db_filter_to_filter).collect();

                Ok::<_, DashboardsError>(Some(db_dashboard_to_dashboard(db, widgets, filters)))
            }
            None => Ok(None),
        }
    })
    .await
    .map_err(|e| DashboardsError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_update_dashboard(
    State(state): State<Arc<AppState>>,
    Path(dashboard_id): Path<Uuid>,
    Json(req): Json<UpdateDashboardRequest>,
) -> Result<Json<Dashboard>, DashboardsError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| DashboardsError::Database(e.to_string()))?;

        let mut db_dash: DbDashboard = dashboards::table
            .find(dashboard_id)
            .first(&mut conn)
            .map_err(|_| DashboardsError::NotFound("Dashboard not found".to_string()))?;

        if let Some(name) = req.name {
            db_dash.name = name;
        }
        if let Some(description) = req.description {
            db_dash.description = Some(description);
        }
        if let Some(layout) = req.layout {
            db_dash.layout = serde_json::to_value(&layout).unwrap_or_default();
        }
        if let Some(is_public) = req.is_public {
            db_dash.is_public = is_public;
        }
        if let Some(refresh_interval) = req.refresh_interval {
            db_dash.refresh_interval = Some(refresh_interval);
        }
        if let Some(tags) = req.tags {
            db_dash.tags = tags;
        }
        db_dash.updated_at = Utc::now();

        diesel::update(dashboards::table.find(dashboard_id))
            .set(&db_dash)
            .execute(&mut conn)
            .map_err(|e| DashboardsError::Database(e.to_string()))?;

        let widgets_db: Vec<DbWidget> = dashboard_widgets::table
            .filter(dashboard_widgets::dashboard_id.eq(dashboard_id))
            .load(&mut conn)
            .unwrap_or_default();
        let filters_db: Vec<DbFilter> = dashboard_filters::table
            .filter(dashboard_filters::dashboard_id.eq(dashboard_id))
            .load(&mut conn)
            .unwrap_or_default();

        let widgets: Vec<Widget> = widgets_db.into_iter().map(db_widget_to_widget).collect();
        let filters: Vec<DashboardFilter> = filters_db.into_iter().map(db_filter_to_filter).collect();

        Ok::<_, DashboardsError>(db_dashboard_to_dashboard(db_dash, widgets, filters))
    })
    .await
    .map_err(|e| DashboardsError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_delete_dashboard(
    State(state): State<Arc<AppState>>,
    Path(dashboard_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, DashboardsError> {
    let pool = state.conn.clone();

    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| DashboardsError::Database(e.to_string()))?;

        let deleted = diesel::delete(dashboards::table.find(dashboard_id))
            .execute(&mut conn)
            .map_err(|e| DashboardsError::Database(e.to_string()))?;

        if deleted == 0 {
            return Err(DashboardsError::NotFound("Dashboard not found".to_string()));
        }

        Ok::<_, DashboardsError>(())
    })
    .await
    .map_err(|e| DashboardsError::Internal(e.to_string()))??;

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_add_widget(
    State(state): State<Arc<AppState>>,
    Path(dashboard_id): Path<Uuid>,
    Json(req): Json<AddWidgetRequest>,
) -> Result<Json<Widget>, DashboardsError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| DashboardsError::Database(e.to_string()))?;
        let now = Utc::now();

        let db_widget = DbWidget {
            id: Uuid::new_v4(),
            dashboard_id,
            widget_type: req.widget_type.to_string(),
            title: req.title,
            position_x: req.position.x,
            position_y: req.position.y,
            width: req.position.width,
            height: req.position.height,
            config: serde_json::to_value(&req.config).unwrap_or_default(),
            data_query: req.data_query.and_then(|q| serde_json::to_value(&q).ok()),
            style: serde_json::to_value(&req.style.unwrap_or_default()).unwrap_or_default(),
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(dashboard_widgets::table)
            .values(&db_widget)
            .execute(&mut conn)
            .map_err(|e| DashboardsError::Database(e.to_string()))?;

        Ok::<_, DashboardsError>(db_widget_to_widget(db_widget))
    })
    .await
    .map_err(|e| DashboardsError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_update_widget(
    State(state): State<Arc<AppState>>,
    Path((dashboard_id, widget_id)): Path<(Uuid, Uuid)>,
    Json(req): Json<UpdateWidgetRequest>,
) -> Result<Json<Widget>, DashboardsError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| DashboardsError::Database(e.to_string()))?;

        let mut db_widget: DbWidget = dashboard_widgets::table
            .filter(dashboard_widgets::id.eq(widget_id))
            .filter(dashboard_widgets::dashboard_id.eq(dashboard_id))
            .first(&mut conn)
            .map_err(|_| DashboardsError::NotFound("Widget not found".to_string()))?;

        if let Some(title) = req.title {
            db_widget.title = title;
        }
        if let Some(position) = req.position {
            db_widget.position_x = position.x;
            db_widget.position_y = position.y;
            db_widget.width = position.width;
            db_widget.height = position.height;
        }
        if let Some(config) = req.config {
            db_widget.config = serde_json::to_value(&config).unwrap_or_default();
        }
        if let Some(data_query) = req.data_query {
            db_widget.data_query = serde_json::to_value(&data_query).ok();
        }
        if let Some(style) = req.style {
            db_widget.style = serde_json::to_value(&style).unwrap_or_default();
        }
        db_widget.updated_at = Utc::now();

        diesel::update(dashboard_widgets::table.find(widget_id))
            .set(&db_widget)
            .execute(&mut conn)
            .map_err(|e| DashboardsError::Database(e.to_string()))?;

        Ok::<_, DashboardsError>(db_widget_to_widget(db_widget))
    })
    .await
    .map_err(|e| DashboardsError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_delete_widget(
    State(state): State<Arc<AppState>>,
    Path((dashboard_id, widget_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<serde_json::Value>, DashboardsError> {
    let pool = state.conn.clone();

    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| DashboardsError::Database(e.to_string()))?;

        let deleted = diesel::delete(
            dashboard_widgets::table
                .filter(dashboard_widgets::id.eq(widget_id))
                .filter(dashboard_widgets::dashboard_id.eq(dashboard_id)),
        )
        .execute(&mut conn)
        .map_err(|e| DashboardsError::Database(e.to_string()))?;

        if deleted == 0 {
            return Err(DashboardsError::NotFound("Widget not found".to_string()));
        }

        Ok::<_, DashboardsError>(())
    })
    .await
    .map_err(|e| DashboardsError::Internal(e.to_string()))??;

    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_get_widget_data(
    State(_state): State<Arc<AppState>>,
    Path((_dashboard_id, widget_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<WidgetData>, DashboardsError> {
    Ok(Json(WidgetData {
        widget_id,
        data: serde_json::json!([]),
        fetched_at: Utc::now(),
    }))
}

pub async fn handle_list_data_sources(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<DataSource>>, DashboardsError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| DashboardsError::Database(e.to_string()))?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let db_sources: Vec<DbDataSource> = dashboard_data_sources::table
            .filter(dashboard_data_sources::bot_id.eq(bot_id))
            .order(dashboard_data_sources::created_at.desc())
            .load(&mut conn)
            .map_err(|e| DashboardsError::Database(e.to_string()))?;

        let sources: Vec<DataSource> = db_sources.into_iter().map(db_data_source_to_data_source).collect();
        Ok::<_, DashboardsError>(sources)
    })
    .await
    .map_err(|e| DashboardsError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_create_data_source(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateDataSourceRequest>,
) -> Result<Json<DataSource>, DashboardsError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| DashboardsError::Database(e.to_string()))?;
        let (bot_id, org_id) = get_default_bot(&mut conn);
        let now = Utc::now();

        let db_source = DbDataSource {
            id: Uuid::new_v4(),
            org_id,
            bot_id,
            name: req.name,
            description: req.description,
            source_type: req.source_type.to_string(),
            connection: serde_json::to_value(&req.connection).unwrap_or_default(),
            schema_definition: serde_json::json!({}),
            refresh_schedule: None,
            last_sync: None,
            status: "active".to_string(),
            created_at: now,
            updated_at: now,
        };

        diesel::insert_into(dashboard_data_sources::table)
            .values(&db_source)
            .execute(&mut conn)
            .map_err(|e| DashboardsError::Database(e.to_string()))?;

        Ok::<_, DashboardsError>(db_data_source_to_data_source(db_source))
    })
    .await
    .map_err(|e| DashboardsError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_test_data_source(
    State(_state): State<Arc<AppState>>,
    Path(_source_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, DashboardsError> {
    Ok(Json(serde_json::json!({ "success": true })))
}

pub async fn handle_delete_data_source(
    State(state): State<Arc<AppState>>,
    Path(source_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, DashboardsError> {
    let pool = state.conn.clone();

    tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| DashboardsError::Database(e.to_string()))?;

        diesel::delete(dashboard_data_sources::table.find(source_id))
            .execute(&mut conn)
            .map_err(|e| DashboardsError::Database(e.to_string()))?;

        Ok::<_, DashboardsError>(())
    })
    .await
    .map_err(|e| DashboardsError::Internal(e.to_string()))??;

    Ok(Json(serde_json::json!({ "success": true })))
}

fn analyze_query_intent(query: &str) -> (WidgetType, String) {
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

pub async fn handle_conversational_query(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ConversationalQueryRequest>,
) -> Result<Json<ConversationalQueryResponse>, DashboardsError> {
    let pool = state.conn.clone();
    let query_text = req.query.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| DashboardsError::Database(e.to_string()))?;
        let (bot_id, org_id) = get_default_bot(&mut conn);
        let now = Utc::now();

        let db_query = DbConversationalQuery {
            id: Uuid::new_v4(),
            org_id,
            bot_id,
            dashboard_id: None,
            user_id: Uuid::nil(),
            natural_language: query_text.clone(),
            generated_query: None,
            result_widget_config: None,
            created_at: now,
        };

        diesel::insert_into(conversational_queries::table)
            .values(&db_query)
            .execute(&mut conn)
            .map_err(|e| DashboardsError::Database(e.to_string()))?;

        let (suggested_viz, explanation) = analyze_query_intent(&query_text);

        let conv_query = ConversationalQuery {
            id: db_query.id,
            dashboard_id: None,
            user_id: db_query.user_id,
            natural_language: db_query.natural_language,
            generated_query: None,
            result_widget: None,
            created_at: db_query.created_at,
        };

        Ok::<_, DashboardsError>(ConversationalQueryResponse {
            query: conv_query,
            data: Some(serde_json::json!([])),
            suggested_visualization: Some(suggested_viz),
            explanation,
        })
    })
    .await
    .map_err(|e| DashboardsError::Internal(e.to_string()))??;

    Ok(Json(result))
}

pub async fn handle_get_templates(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Dashboard>>, DashboardsError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool.get().map_err(|e| DashboardsError::Database(e.to_string()))?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let db_dashboards: Vec<DbDashboard> = dashboards::table
            .filter(dashboards::bot_id.eq(bot_id))
            .filter(dashboards::is_template.eq(true))
            .order(dashboards::created_at.desc())
            .load(&mut conn)
            .map_err(|e| DashboardsError::Database(e.to_string()))?;

        let templates: Vec<Dashboard> = db_dashboards
            .into_iter()
            .map(|db| db_dashboard_to_dashboard(db, vec![], vec![]))
            .collect();

        Ok::<_, DashboardsError>(templates)
    })
    .await
    .map_err(|e| DashboardsError::Internal(e.to_string()))??;

    Ok(Json(result))
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
