






pub use crate::basic::keywords::mcp_directory::{
    generate_example_configs, McpConnectionConfig, McpCsvLoader, McpCsvRow, McpLoadError,
    McpLoadResult, McpServerConfig, McpToolConfig,
};


pub use crate::basic::keywords::mcp_directory::{
    McpDirectoryScanResult, McpDirectoryScanner, McpScanError,
};

pub use crate::basic::keywords::mcp_client::{
    McpCapabilities, McpClient, McpConnection, McpRequest, McpResponse, McpServer, McpServerStatus,
    McpServerType, McpTool, ToolRiskLevel,
};


pub fn get_server_type_icon(server_type: &str) -> &'static str {
    match server_type.to_lowercase().as_str() {
        "filesystem" => "📁",
        "database" => "🗄️",
        "github" => "🐙",
        "web" | "http" => "🌐",
        "email" => "📧",
        "slack" => "💬",
        "teams" => "👥",
        "analytics" => "📊",
        "search" => "🔍",
        "storage" => "💾",
        "compute" => "⚡",
        "custom" => "🔧",
        _ => "🔌",
    }
}


pub fn get_risk_level_class(risk_level: &ToolRiskLevel) -> &'static str {
    match risk_level {
        ToolRiskLevel::Safe => "risk-safe",
        ToolRiskLevel::Low => "risk-low",
        ToolRiskLevel::Medium => "risk-medium",
        ToolRiskLevel::High => "risk-high",
        ToolRiskLevel::Critical => "risk-critical",
    }
}


pub fn get_risk_level_name(risk_level: &ToolRiskLevel) -> &'static str {
    match risk_level {
        ToolRiskLevel::Safe => "Safe",
        ToolRiskLevel::Low => "Low",
        ToolRiskLevel::Medium => "Medium",
        ToolRiskLevel::High => "High",
        ToolRiskLevel::Critical => "Critical",
    }
}


pub fn create_loader(work_path: &str, bot_id: &str) -> McpCsvLoader {
    McpCsvLoader::new(work_path, bot_id)
}


pub fn load_servers(work_path: &str, bot_id: &str) -> McpLoadResult {
    let loader = McpCsvLoader::new(work_path, bot_id);
    loader.load()
}


pub fn csv_exists(work_path: &str, bot_id: &str) -> bool {
    let loader = McpCsvLoader::new(work_path, bot_id);
    loader.csv_exists()
}


pub fn get_csv_path(work_path: &str, bot_id: &str) -> std::path::PathBuf {
    let loader = McpCsvLoader::new(work_path, bot_id);
    loader.get_csv_path()
}
