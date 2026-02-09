use diesel::prelude::*;
use log::{debug, info, warn};
use serde_json::{json, Value};
use std::path::Path;
use uuid::Uuid;

use crate::shared::utils::DbPool;

/// Structure to hold tool information loaded from .mcp.json files
#[derive(Debug, Clone)]
struct ToolInfo {
    name: String,
    description: String,
    parameters: Vec<ToolParameter>,
}

#[derive(Debug, Clone)]
struct ToolParameter {
    name: String,
    param_type: String,
    description: String,
    required: bool,
    example: Option<String>,
}

/// Loads tools for a bot and returns them formatted for OpenAI API
pub fn get_session_tools(
    db_pool: &DbPool,
    bot_name: &str,
    session_id: &Uuid,
) -> Result<Vec<Value>, Box<dyn std::error::Error + Send + Sync>> {
    use crate::shared::models::schema::{bots, session_tool_associations};

    // Get bot_id (we use the query to verify the bot exists)
    let mut conn = db_pool.get()?;
    let _bot_id: Uuid = bots::table
        .filter(bots::name.eq(bot_name))
        .select(bots::id)
        .first(&mut *conn)
        .map_err(|e| format!("Failed to get bot_id for bot '{}': {}", bot_name, e))?;

    // Get tool names associated with this session
    let session_id_str = session_id.to_string();
    let tool_names: Vec<String> = session_tool_associations::table
        .filter(session_tool_associations::session_id.eq(&session_id_str))
        .select(session_tool_associations::tool_name)
        .load::<String>(&mut *conn)
        .map_err(|e| format!("Failed to get tools for session: {}", e))?;

    if tool_names.is_empty() {
        debug!("No tools associated with session {}", session_id);
        return Ok(vec![]);
    }

    // Build path to work/{bot_name}.gbai/{bot_name}.gbdialog directory
    let home_dir = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let gb_dir = format!("{}/gb", home_dir);
    let work_path = Path::new(&gb_dir).join("work").join(format!("{}.gbai/{}.gbdialog", bot_name, bot_name));

    info!("Loading {} tools for session {} from {:?}", tool_names.len(), session_id, work_path);

    let mut tools = Vec::new();

    for tool_name in &tool_names {
        // Find the .mcp.json file for this tool
        let mcp_path = work_path.join(format!("{}.mcp.json", tool_name));

        if !mcp_path.exists() {
            warn!("Tool JSON file not found: {:?}", mcp_path);
            continue;
        }

        // Read and parse the .mcp.json file
        let mcp_content = std::fs::read_to_string(&mcp_path)
            .map_err(|e| format!("Failed to read tool file {:?}: {}", mcp_path, e))?;

        let mcp_json: Value = serde_json::from_str(&mcp_content)
            .map_err(|e| format!("Failed to parse tool JSON from {:?}: {}", mcp_path, e))?;

        // Extract tool information and format for OpenAI
        if let Some(tool) = format_tool_for_openai(&mcp_json, tool_name) {
            tools.push(tool);
        }
    }

    info!("Loaded {} tools for session {}", tools.len(), session_id);
    Ok(tools)
}

/// Formats a tool definition from .mcp.json format to OpenAI tool format
fn format_tool_for_openai(mcp_json: &Value, tool_name: &str) -> Option<Value> {
    let _name = mcp_json.get("name")?.as_str()?;
    let description = mcp_json.get("description")?.as_str()?;
    let input_schema = mcp_json.get("input_schema")?;

    let parameters = input_schema.get("properties")?.as_object()?;
    let required = input_schema.get("required")?.as_array()?;

    let mut openai_params = serde_json::Map::new();

    for (param_name, param_info) in parameters {
        let param_obj = param_info.as_object()?;
        let param_desc = param_obj.get("description")?.as_str().unwrap_or("");
        let param_type = param_obj.get("type")?.as_str().unwrap_or("string");

        openai_params.insert(
            param_name.clone(),
            json!({
                "type": param_type,
                "description": param_desc
            })
        );
    }

    Some(json!({
        "type": "function",
        "function": {
            "name": tool_name,
            "description": description,
            "parameters": {
                "type": "object",
                "properties": openai_params,
                "required": required
            }
        }
    }))
}
