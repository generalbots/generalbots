use crate::core::bot::get_default_bot;
use crate::core::shared::state::AppState;
use axum::Router;
use std::sync::Arc;

pub use botsources::{
    configure_sources_api_routes as crate_configure_sources_routes,
    AppState as SourcesAppState,
    ConfigManagerOps, DbPool as SourcesDbPool, GetDefaultBotFn as SourcesGetDefaultBotFn,
    GetKeywordsFn, GetWorkPathFn, McpCsvLoaderFn, McpCsvLoaderOps, McpCsvRowData,
    McpLoadResult, McpServerInfo, McpToolInfo,
};

struct BotserverConfigManager {
    pool: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>,
}

impl botsources::ConfigManagerOps for BotserverConfigManager {
    fn get_config(
        &self,
        bot_id: &uuid::Uuid,
        key: &str,
        default: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        use diesel::prelude::*;
        use crate::core::shared::models::schema::bot_configuration::dsl::*;
        let mut conn = self.pool.get()?;
        let result = bot_configuration
            .filter(config_key.eq(key))
            .filter(crate::core::shared::models::schema::bot_configuration::dsl::bot_id.eq(*bot_id))
            .select(config_value)
            .first::<String>(&mut conn)
            .ok();
        Ok(result.unwrap_or_else(|| default.unwrap_or("").to_string()))
    }

    fn set_config(
        &self,
        _bot_id: &uuid::Uuid,
        _key: &str,
        _value: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

fn make_sources_state(app_state: &Arc<AppState>) -> SourcesAppState {
    SourcesAppState {
        conn: app_state.conn.clone(),
        config_manager: Arc::new(BotserverConfigManager { pool: app_state.conn.clone() }),
        get_default_bot: Some(Arc::new(|conn| get_default_bot(conn))),
        get_work_path: Some(Arc::new(|| "/opt/gbo/work".to_string())),
        get_keywords: Some(Arc::new(|| Vec::new())),
        mcp_loader: None,
    }
}

pub fn configure_sources_routes(app_state: Arc<AppState>) -> Router {
    crate_configure_sources_routes()
        .with_state(Arc::new(make_sources_state(&app_state)))
}

pub fn configure_sources_ui_routes(app_state: Arc<AppState>) -> Router {
    botsources::ui::configure_sources_ui_routes()
        .with_state(Arc::new(make_sources_state(&app_state)))
}
