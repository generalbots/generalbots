pub use botsources::*;

use std::sync::Arc;

use botsources::state::McpLoadError;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;

use botbasic_ai::keywords::mcp_client::McpServer;
use botbasic_ai::keywords::mcp_directory::McpCsvLoader;

pub struct SourcesConfigManager {
    pub inner: botcore::config::ConfigManager,
}

impl botsources::state::ConfigManagerOps for SourcesConfigManager {
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

/// Adapts the bot's on-disk MCP directory (`{bot}.gbai/mcp.csv`) to the
/// sources service trait (#1675). The loader was previously left unwired, so
/// the product's MCP list/add always reported an empty catalog.
struct DirectoryMcpLoader {
    inner: McpCsvLoader,
}

fn server_type_label(server: &McpServer) -> String {
    match &server.server_type {
        botbasic_ai::keywords::mcp_client::McpServerType::Custom(name) => name.clone(),
        other => format!("{other:?}").to_lowercase(),
    }
}

fn server_info(server: &McpServer) -> McpServerInfo {
    McpServerInfo {
            id: server.id.clone(),
            name: server.name.clone(),
            description: server.description.clone(),
            server_type: server_type_label(server),
            status: match &server.status {
                botbasic_ai::keywords::mcp_client::McpServerStatus::Active => "active",
                botbasic_ai::keywords::mcp_client::McpServerStatus::Inactive => "inactive",
                botbasic_ai::keywords::mcp_client::McpServerStatus::Connecting => "connecting",
                botbasic_ai::keywords::mcp_client::McpServerStatus::Maintenance => "maintenance",
                botbasic_ai::keywords::mcp_client::McpServerStatus::Error(_) => "error",
                botbasic_ai::keywords::mcp_client::McpServerStatus::Unknown => "unknown",
            }
            .to_string(),
            enabled: matches!(
                &server.status,
                botbasic_ai::keywords::mcp_client::McpServerStatus::Active
            ),
            tools: server
                .tools
                .iter()
                .map(|tool| McpToolInfo {
                    name: tool.name.clone(),
                    description: tool.description.clone(),
                    risk_level: format!("{:?}", tool.risk_level).to_lowercase(),
                    requires_approval: tool.requires_approval,
                })
                .collect(),
    }
}

fn to_sources_result(result: botbasic_ai::keywords::mcp_directory::McpDirectoryScanResult) -> McpLoadResult {
    McpLoadResult {
        file_path: result.file_path,
        servers: result.servers.iter().map(server_info).collect(),
        lines_processed: result.lines_processed,
        errors: result
            .errors
            .into_iter()
            .map(|error| McpLoadError {
                line: error.line,
                message: error.message,
                recoverable: error.recoverable,
            })
            .collect(),
    }
}

impl McpCsvLoaderOps for DirectoryMcpLoader {
    fn load(&self) -> McpLoadResult {
        to_sources_result(self.inner.load())
    }

    fn load_server(&self, name: &str) -> Option<McpServerInfo> {
        self.inner.load_server(name).map(|server| server_info(&server))
    }

    fn add_server(&self, row: &McpCsvRowData) -> Result<(), String> {
        let row = botbasic_ai::keywords::mcp_directory::McpCsvRow {
            name: row.name.clone(),
            connection_type: row.connection_type.clone(),
            command: row.command.clone(),
            args: row.args.clone(),
            description: row.description.clone(),
            enabled: row.enabled,
            auth_type: row.auth_type.clone(),
            auth_env: row.auth_env.clone(),
            risk_level: row.risk_level.clone(),
            requires_approval: row.requires_approval,
        };
        self.inner.add_server(&row).map_err(|error| error.to_string())
    }

    fn remove_server(&self, name: &str) -> Result<bool, String> {
        self.inner.remove_server(name).map_err(|error| error.to_string())
    }

    fn csv_exists(&self) -> bool {
        self.inner.csv_exists()
    }
}

pub fn make_sources_state(
    conn: Pool<ConnectionManager<PgConnection>>,
) -> Arc<botsources::state::AppState> {
    let config_manager = botcore::config::ConfigManager::new(conn.clone());
    let mcp_loader: botsources::state::McpCsvLoaderFn =
        Arc::new(|work_path: &str, bot_id: &str| {
            Box::new(DirectoryMcpLoader {
                inner: McpCsvLoader::new(work_path, bot_id),
            })
        });
    Arc::new(botsources::state::AppState {
        conn,
        config_manager: Arc::new(SourcesConfigManager {
            inner: config_manager,
        }),
        get_default_bot: Some(Arc::new(|_conn: &mut PgConnection| {
            (uuid::Uuid::nil(), "default".to_string())
        })),
        get_work_path: Some(Arc::new(botcore::shared::utils::get_work_path)),
        get_keywords: None,
        mcp_loader: Some(mcp_loader),
    })
}
