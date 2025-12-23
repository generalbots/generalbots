use crate::basic::keywords::set_schedule::execute_set_schedule;
use crate::basic::keywords::table_definition::process_table_definitions;
use crate::basic::keywords::webhook::execute_webhook_registration;
use crate::shared::models::TriggerKind;
use crate::shared::state::AppState;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use log::{trace, warn};

pub mod goto_transform;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::Arc;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamDeclaration {
    pub name: String,
    pub param_type: String,
    pub example: Option<String>,
    pub description: String,
    pub required: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: Vec<ParamDeclaration>,
    pub source_file: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPTool {
    pub name: String,
    pub description: String,
    pub input_schema: MCPInputSchema,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPInputSchema {
    #[serde(rename = "type")]
    pub schema_type: String,
    pub properties: HashMap<String, MCPProperty>,
    pub required: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MCPProperty {
    #[serde(rename = "type")]
    pub prop_type: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAITool {
    #[serde(rename = "type")]
    pub tool_type: String,
    pub function: OpenAIFunction,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIFunction {
    pub name: String,
    pub description: String,
    pub parameters: OpenAIParameters,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIParameters {
    #[serde(rename = "type")]
    pub param_type: String,
    pub properties: HashMap<String, OpenAIProperty>,
    pub required: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIProperty {
    #[serde(rename = "type")]
    pub prop_type: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub example: Option<String>,
}
#[derive(Debug)]
pub struct BasicCompiler {
    state: Arc<AppState>,
    bot_id: uuid::Uuid,
    previous_schedules: HashSet<String>,
}
impl BasicCompiler {
    #[must_use]
    pub fn new(state: Arc<AppState>, bot_id: uuid::Uuid) -> Self {
        Self {
            state,
            bot_id,
            previous_schedules: HashSet::new(),
        }
    }
    pub fn compile_file(
        &mut self,
        source_path: &str,
        output_dir: &str,
    ) -> Result<CompilationResult, Box<dyn Error + Send + Sync>> {
        let source_content = fs::read_to_string(source_path)
            .map_err(|e| format!("Failed to read source file: {e}"))?;


        if let Err(e) =
            process_table_definitions(Arc::clone(&self.state), self.bot_id, &source_content)
        {
            log::warn!("Failed to process TABLE definitions: {}", e);
        }

        let tool_def = self.parse_tool_definition(&source_content, source_path)?;
        let file_name = Path::new(source_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Invalid file name")?;
        let ast_path = format!("{output_dir}/{file_name}.ast");
        let ast_content = self.preprocess_basic(&source_content, source_path, self.bot_id)?;
        fs::write(&ast_path, &ast_content).map_err(|e| format!("Failed to write AST file: {e}"))?;
        let (mcp_json, tool_json) = if tool_def.parameters.is_empty() {
            (None, None)
        } else {
            let mcp = self.generate_mcp_tool(&tool_def)?;
            let openai = self.generate_openai_tool(&tool_def)?;
            let mcp_path = format!("{output_dir}/{file_name}.mcp.json");
            let tool_path = format!("{output_dir}/{file_name}.tool.json");
            let mcp_json_str = serde_json::to_string_pretty(&mcp)?;
            fs::write(&mcp_path, mcp_json_str)
                .map_err(|e| format!("Failed to write MCP JSON: {e}"))?;
            let tool_json_str = serde_json::to_string_pretty(&openai)?;
            fs::write(&tool_path, tool_json_str)
                .map_err(|e| format!("Failed to write tool JSON: {e}"))?;
            (Some(mcp), Some(openai))
        };
        Ok(CompilationResult {
            mcp_tool: mcp_json,
            _openai_tool: tool_json,
        })
    }
    pub fn parse_tool_definition(
        &self,
        source: &str,
        source_path: &str,
    ) -> Result<ToolDefinition, Box<dyn Error + Send + Sync>> {
        let mut params = Vec::new();
        let mut description = String::new();
        let lines: Vec<&str> = source.lines().collect();
        let mut i = 0;
        while i < lines.len() {
            let line = lines[i].trim();
            if line.starts_with("PARAM ") {
                if let Some(param) = self.parse_param_line(line)? {
                    params.push(param);
                }
            }
            if line.starts_with("DESCRIPTION ") {
                let desc_start = line.find('"').unwrap_or(0);
                let desc_end = line.rfind('"').unwrap_or(line.len());
                if desc_start < desc_end {
                    description = line[desc_start + 1..desc_end].to_string();
                }
            }
            i += 1;
        }
        let tool_name = Path::new(source_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        Ok(ToolDefinition {
            name: tool_name,
            description,
            parameters: params,
            source_file: source_path.to_string(),
        })
    }
    fn parse_param_line(
        &self,
        line: &str,
    ) -> Result<Option<ParamDeclaration>, Box<dyn Error + Send + Sync>> {
        let line = line.trim();
        if !line.starts_with("PARAM ") {
            return Ok(None);
        }
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 4 {
            warn!("Invalid PARAM line: {line}");
            return Ok(None);
        }
        let name = parts[1].to_string();
        let as_index = parts.iter().position(|&p| p == "AS");
        let param_type = if let Some(idx) = as_index {
            if idx + 1 < parts.len() {
                parts[idx + 1].to_lowercase()
            } else {
                "string".to_string()
            }
        } else {
            "string".to_string()
        };
        let example = if let Some(like_pos) = line.find("LIKE") {
            let rest = &line[like_pos + 4..].trim();
            if let Some(start) = rest.find('"') {
                if let Some(end) = rest[start + 1..].find('"') {
                    Some(rest[start + 1..start + 1 + end].to_string())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };
        let description = if let Some(desc_pos) = line.find("DESCRIPTION") {
            let rest = &line[desc_pos + 11..].trim();
            if let Some(start) = rest.find('"') {
                if let Some(end) = rest[start + 1..].rfind('"') {
                    rest[start + 1..start + 1 + end].to_string()
                } else {
                    "".to_string()
                }
            } else {
                "".to_string()
            }
        } else {
            "".to_string()
        };
        Ok(Some(ParamDeclaration {
            name,
            param_type: self.normalize_type(&param_type),
            example,
            description,
            required: true,
        }))
    }
    fn normalize_type(&self, basic_type: &str) -> String {
        match basic_type.to_lowercase().as_str() {
            "string" | "text" => "string".to_string(),
            "integer" | "int" | "number" => "integer".to_string(),
            "float" | "double" | "decimal" => "number".to_string(),
            "boolean" | "bool" => "boolean".to_string(),
            "date" | "datetime" => "string".to_string(),
            "array" | "list" => "array".to_string(),
            "object" | "map" => "object".to_string(),
            _ => "string".to_string(),
        }
    }
    fn generate_mcp_tool(
        &self,
        tool_def: &ToolDefinition,
    ) -> Result<MCPTool, Box<dyn Error + Send + Sync>> {
        let mut properties = HashMap::new();
        let mut required = Vec::new();
        for param in &tool_def.parameters {
            properties.insert(
                param.name.clone(),
                MCPProperty {
                    prop_type: param.param_type.clone(),
                    description: param.description.clone(),
                    example: param.example.clone(),
                },
            );
            if param.required {
                required.push(param.name.clone());
            }
        }
        Ok(MCPTool {
            name: tool_def.name.clone(),
            description: tool_def.description.clone(),
            input_schema: MCPInputSchema {
                schema_type: "object".to_string(),
                properties,
                required,
            },
        })
    }
    fn generate_openai_tool(
        &self,
        tool_def: &ToolDefinition,
    ) -> Result<OpenAITool, Box<dyn Error + Send + Sync>> {
        let mut properties = HashMap::new();
        let mut required = Vec::new();
        for param in &tool_def.parameters {
            properties.insert(
                param.name.clone(),
                OpenAIProperty {
                    prop_type: param.param_type.clone(),
                    description: param.description.clone(),
                    example: param.example.clone(),
                },
            );
            if param.required {
                required.push(param.name.clone());
            }
        }
        Ok(OpenAITool {
            tool_type: "function".to_string(),
            function: OpenAIFunction {
                name: tool_def.name.clone(),
                description: tool_def.description.clone(),
                parameters: OpenAIParameters {
                    param_type: "object".to_string(),
                    properties,
                    required,
                },
            },
        })
    }
    fn preprocess_basic(
        &mut self,
        source: &str,
        source_path: &str,
        bot_id: uuid::Uuid,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let bot_uuid = bot_id;
        let mut result = String::new();



        let source = if goto_transform::has_goto_constructs(source) {
            trace!("GOTO constructs detected, transforming to state machine");
            goto_transform::transform_goto(source)
        } else {
            source.to_string()
        };
        let source = source.as_str();
        let mut has_schedule = false;
        let mut _has_webhook = false;
        let script_name = Path::new(source_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();
        {
            let mut conn = self
                .state
                .conn
                .get()
                .map_err(|e| format!("Failed to get database connection: {e}"))?;
            use crate::shared::models::system_automations::dsl::*;
            diesel::delete(
                system_automations
                    .filter(bot_id.eq(bot_uuid))
                    .filter(kind.eq(TriggerKind::Scheduled as i32))
                    .filter(param.eq(&script_name)),
            )
            .execute(&mut conn)
            .ok();
        }
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with("'")
                || trimmed.starts_with("//")
                || trimmed.starts_with("REM")
            {
                continue;
            }




            let normalized = trimmed
                .replace("FOR EACH", "FOR_EACH")
                .replace("EXIT FOR", "EXIT_FOR")
                .replace("GROUP BY", "GROUP_BY");
            if normalized.starts_with("SET SCHEDULE") || trimmed.starts_with("SET SCHEDULE") {
                has_schedule = true;
                let parts: Vec<&str> = normalized.split('"').collect();
                if parts.len() >= 3 {
                    let cron = parts[1];
                    let mut conn = self
                        .state
                        .conn
                        .get()
                        .map_err(|e| format!("Failed to get database connection: {e}"))?;
                    if let Err(e) = execute_set_schedule(&mut conn, cron, &script_name, bot_id) {
                        log::error!(
                            "Failed to schedule SET SCHEDULE during preprocessing: {}",
                            e
                        );
                    }
                } else {
                    log::warn!("Malformed SET SCHEDULE line ignored: {}", trimmed);
                }
                continue;
            }

            if normalized.starts_with("WEBHOOK") {
                _has_webhook = true;
                let parts: Vec<&str> = normalized.split('"').collect();
                if parts.len() >= 2 {
                    let endpoint = parts[1];
                    let mut conn = self
                        .state
                        .conn
                        .get()
                        .map_err(|e| format!("Failed to get database connection: {}", e))?;
                    if let Err(e) =
                        execute_webhook_registration(&mut conn, endpoint, &script_name, bot_id)
                    {
                        log::error!("Failed to register WEBHOOK during preprocessing: {}", e);
                    } else {
                        log::info!(
                            "Registered webhook endpoint {} for script {} during preprocessing",
                            endpoint,
                            script_name
                        );
                    }
                } else {
                    log::warn!("Malformed WEBHOOK line ignored: {}", normalized);
                }
                continue;
            }

            if trimmed.starts_with("USE WEBSITE") {
                let parts: Vec<&str> = normalized.split('"').collect();
                if parts.len() >= 2 {
                    let url = parts[1];
                    let mut conn = self
                        .state
                        .conn
                        .get()
                        .map_err(|e| format!("Failed to get database connection: {}", e))?;
                    if let Err(e) =
                        crate::basic::keywords::use_website::execute_use_website_preprocessing(
                            &mut conn, url, bot_id,
                        )
                    {
                        log::error!("Failed to register USE_WEBSITE during preprocessing: {}", e);
                    } else {
                        log::info!(
                            "Registered website {} for crawling during preprocessing",
                            url
                        );
                    }
                } else {
                    log::warn!("Malformed USE_WEBSITE line ignored: {}", normalized);
                }
                continue;
            }
            if normalized.starts_with("PARAM ") || normalized.starts_with("DESCRIPTION ") {
                continue;
            }
            result.push_str(&normalized);
            result.push('\n');
        }
        if self.previous_schedules.contains(&script_name) && !has_schedule {
            let mut conn = self
                .state
                .conn
                .get()
                .map_err(|e| format!("Failed to get database connection: {}", e))?;
            use crate::shared::models::system_automations::dsl::*;
            diesel::delete(
                system_automations
                    .filter(bot_id.eq(bot_uuid))
                    .filter(kind.eq(TriggerKind::Scheduled as i32))
                    .filter(param.eq(&script_name)),
            )
            .execute(&mut conn)
            .map_err(|e| log::error!("Failed to remove schedule for {}: {}", script_name, e))
            .ok();
        }
        if has_schedule {
            self.previous_schedules.insert(script_name);
        } else {
            self.previous_schedules.remove(&script_name);
        }
        Ok(result)
    }
}
#[derive(Debug)]
pub struct CompilationResult {
    pub mcp_tool: Option<MCPTool>,
    pub _openai_tool: Option<OpenAITool>,
}
