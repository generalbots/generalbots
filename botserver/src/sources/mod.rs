pub use botsources::*;

use botsources::state::ConfigManagerOps;
use std::sync::Arc;

pub struct SourcesConfigManager {
    inner: botcore::config::ConfigManager,
}

impl ConfigManagerOps for SourcesConfigManager {
    fn get_config(
        &self,
        bot_id: &uuid::Uuid,
        key: &str,
        default: Option<&str>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        self.inner.get_config(bot_id, key, default)
    }

    fn set_config(
        &self,
        bot_id: &uuid::Uuid,
        key: &str,
        value: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        self.inner.set_config(bot_id, key, value)
    }
}

pub fn make_sources_state(conn: diesel::r2d2::Pool<diesel::r2d2::ConnectionManager<diesel::PgConnection>>) -> Arc<botsources::state::AppState> {
    Arc::new(botsources::state::AppState {
        conn,
        config_manager: Arc::new(SourcesConfigManager {
            inner: botcore::config::ConfigManager::new(
                conn.clone(),
            ),
        }),
        get_default_bot: None,
        get_work_path: None,
        get_keywords: None,
        mcp_loader: None,
    })
}
