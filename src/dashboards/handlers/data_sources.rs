use axum::{
    extract::{Path, State},
    Json,
};
use chrono::Utc;
use diesel::prelude::*;
use std::sync::Arc;
use uuid::Uuid;

use crate::bot::get_default_bot;
use crate::core::shared::schema::{conversational_queries, dashboard_data_sources};
use crate::shared::state::AppState;

use crate::dashboards::error::DashboardsError;
use crate::dashboards::storage::{db_data_source_to_data_source, DbConversationalQuery, DbDataSource};
use crate::dashboards::types::{
    ConversationalQuery, ConversationalQueryRequest, ConversationalQueryResponse,
    CreateDataSourceRequest, DataSource, WidgetType,
};

pub async fn handle_list_data_sources(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<DataSource>>, DashboardsError> {
    let pool = state.conn.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| DashboardsError::Database(e.to_string()))?;
        let (bot_id, _) = get_default_bot(&mut conn);

        let db_sources: Vec<DbDataSource> = dashboard_data_sources::table
            .filter(dashboard_data_sources::bot_id.eq(bot_id))
            .order(dashboard_data_sources::created_at.desc())
            .load(&mut conn)
            .map_err(|e| DashboardsError::Database(e.to_string()))?;

        let sources: Vec<DataSource> = db_sources
            .into_iter()
            .map(db_data_source_to_data_source)
            .collect();
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
        let mut conn = pool
            .get()
            .map_err(|e| DashboardsError::Database(e.to_string()))?;
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
        let mut conn = pool
            .get()
            .map_err(|e| DashboardsError::Database(e.to_string()))?;

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

    if query_lower.contains("trend")
        || query_lower.contains("over time")
        || query_lower.contains("timeline")
    {
        (
            WidgetType::LineChart,
            "Showing data as a line chart to visualize trends over time".to_string(),
        )
    } else if query_lower.contains("compare")
        || query_lower.contains("by category")
        || query_lower.contains("breakdown")
    {
        (
            WidgetType::BarChart,
            "Using a bar chart to compare values across categories".to_string(),
        )
    } else if query_lower.contains("distribution")
        || query_lower.contains("percentage")
        || query_lower.contains("share")
    {
        (
            WidgetType::PieChart,
            "Displaying distribution as a pie chart".to_string(),
        )
    } else if query_lower.contains("total")
        || query_lower.contains("count")
        || query_lower.contains("sum")
        || query_lower.contains("kpi")
    {
        (
            WidgetType::Kpi,
            "Showing as a KPI card for quick insight".to_string(),
        )
    } else if query_lower.contains("table")
        || query_lower.contains("list")
        || query_lower.contains("details")
    {
        (
            WidgetType::Table,
            "Presenting data in a table format for detailed view".to_string(),
        )
    } else if query_lower.contains("map")
        || query_lower.contains("location")
        || query_lower.contains("geographic")
    {
        (
            WidgetType::Map,
            "Visualizing geographic data on a map".to_string(),
        )
    } else if query_lower.contains("gauge")
        || query_lower.contains("progress")
        || query_lower.contains("target")
    {
        (
            WidgetType::Gauge,
            "Showing progress toward a target as a gauge".to_string(),
        )
    } else {
        (
            WidgetType::BarChart,
            "Defaulting to bar chart for general visualization".to_string(),
        )
    }
}

pub async fn handle_conversational_query(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ConversationalQueryRequest>,
) -> Result<Json<ConversationalQueryResponse>, DashboardsError> {
    let pool = state.conn.clone();
    let query_text = req.query.clone();

    let result = tokio::task::spawn_blocking(move || {
        let mut conn = pool
            .get()
            .map_err(|e| DashboardsError::Database(e.to_string()))?;
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
