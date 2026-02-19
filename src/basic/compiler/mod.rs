#[cfg(feature = "tasks")]
use crate::basic::keywords::set_schedule::execute_set_schedule;
use crate::basic::keywords::table_definition::process_table_definitions;
use crate::basic::keywords::webhook::execute_webhook_registration;
use crate::core::shared::models::TriggerKind;
use crate::core::shared::state::AppState;
use diesel::QueryableByName;
// use diesel::sql_types::Text; // Removed unused import
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use log::{trace, warn};
use regex::Regex;

pub mod goto_transform;
pub mod blocks;
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
    pub original_type: String,
    pub example: Option<String>,
    pub description: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
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
    #[serde(rename = "enum", skip_serializing_if = "Option::is_none")]
    pub enum_values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub format: Option<String>,
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

        // Generate ADD SUGGESTION commands for enum parameters
        let source_with_suggestions = self.generate_enum_suggestions(&source_content, &tool_def)?;

        let ast_path = format!("{output_dir}/{file_name}.ast");
        let ast_content = self.preprocess_basic(&source_with_suggestions, source_path, self.bot_id)?;
        fs::write(&ast_path, &ast_content).map_err(|e| format!("Failed to write AST file: {e}"))?;
        let (mcp_json, tool_json) = if tool_def.parameters.is_empty() {
            (None, None)
        } else {
            let mcp = Self::generate_mcp_tool(&tool_def)?;
            let openai = Self::generate_openai_tool(&tool_def)?;
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
            openai_tool: tool_json,
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
                if let Some(param) = Self::parse_param_line(line)? {
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
        let example = line.find("LIKE").and_then(|like_pos| {
            let rest = &line[like_pos + 4..].trim();
            rest.find('"').and_then(|start| {
                rest[start + 1..]
                    .find('"')
                    .map(|end| rest[start + 1..start + 1 + end].to_string())
            })
        });

        // Parse ENUM array directly from PARAM statement
        // Syntax: PARAM name AS TYPE ENUM ["value1", "value2", ...]
        let enum_values = if let Some(enum_pos) = line.find("ENUM") {
            let rest = &line[enum_pos + 4..].trim();
            if let Some(start) = rest.find('[') {
                if let Some(end) = rest[start..].find(']') {
                    let array_content = &rest[start + 1..start + end];
                    // Parse the array elements
                    let values: Vec<String> = array_content
                        .split(',')
                        .map(|s| {
                            s.trim()
                                .trim_matches('"')
                                .trim_matches('\'')
                                .to_string()
                        })
                        .filter(|s| !s.is_empty())
                        .collect();
                    Some(values)
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
            param_type: Self::normalize_type(&param_type),
            original_type: param_type.to_lowercase(),
            example,
            description,
            required: true,
            enum_values,
        }))
    }
    fn normalize_type(basic_type: &str) -> String {
        match basic_type.to_lowercase().as_str() {
            "integer" | "int" | "number" => "integer".to_string(),
            "float" | "double" | "decimal" => "number".to_string(),
            "boolean" | "bool" => "boolean".to_string(),
            "array" | "list" => "array".to_string(),
            "object" | "map" => "object".to_string(),
            // "string", "text", "date", "datetime", and any other type default to string
            _ => "string".to_string(),
        }
    }

    /// Generate ADD SUGGESTION commands for parameters with enum values
    fn generate_enum_suggestions(
        &self,
        source: &str,
        tool_def: &ToolDefinition,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let mut result = String::new();
        let mut suggestion_lines = Vec::new();

        // Generate ADD SUGGESTION TEXT commands for each parameter with enum values
        // These will send the enum value as a text message when clicked
        for param in &tool_def.parameters {
            if let Some(ref enum_values) = param.enum_values {
                // For each enum value, create a suggestion button
                for enum_value in enum_values {
                    // Use the enum value as both the text to send and the button label
                    let suggestion_cmd = format!(
                        "ADD SUGGESTION TEXT \"{}\" AS \"{}\"",
                        enum_value, enum_value
                    );
                    suggestion_lines.push(suggestion_cmd);
                }
            }
        }

        // Insert suggestions after the DESCRIPTION line (or at end if no DESCRIPTION)
        let lines: Vec<&str> = source.lines().collect();
        let mut inserted = false;

        for line in lines.iter() {
            result.push_str(line);
            result.push('\n');

            // Insert suggestions after DESCRIPTION line
            if !inserted && line.trim().starts_with("DESCRIPTION ") {
                // Insert suggestions after this line
                for suggestion in &suggestion_lines {
                    result.push_str(suggestion);
                    result.push('\n');
                }
                inserted = true;
            }
        }

        // If we didn't find a DESCRIPTION line, insert at the end
        if !inserted && !suggestion_lines.is_empty() {
            for suggestion in &suggestion_lines {
                result.push_str(suggestion);
                result.push('\n');
            }
        }

        Ok(result)
    }

    fn generate_mcp_tool(
        tool_def: &ToolDefinition,
    ) -> Result<MCPTool, Box<dyn Error + Send + Sync>> {
        let mut properties = HashMap::new();
        let mut required = Vec::new();
        for param in &tool_def.parameters {
            // Add format="date" for DATE type parameters to indicate ISO 8601 format
            let format = if param.original_type == "date" {
                Some("date".to_string())
            } else {
                None
            };

            properties.insert(
                param.name.clone(),
                MCPProperty {
                    prop_type: param.param_type.clone(),
                    description: param.description.clone(),
                    example: param.example.clone(),
                    format,
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
        tool_def: &ToolDefinition,
    ) -> Result<OpenAITool, Box<dyn Error + Send + Sync>> {
        let mut properties = HashMap::new();
        let mut required = Vec::new();
        for param in &tool_def.parameters {
            // Add format="date" for DATE type parameters to indicate ISO 8601 format
            let format = if param.original_type == "date" {
                Some("date".to_string())
            } else {
                None
            };

            properties.insert(
                param.name.clone(),
                OpenAIProperty {
                    prop_type: param.param_type.clone(),
                    description: param.description.clone(),
                    example: param.example.clone(),
                    enum_values: param.enum_values.clone(),
                    format,
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
            use crate::core::shared::models::system_automations::dsl::*;
            diesel::delete(
                system_automations
                    .filter(bot_id.eq(&bot_uuid))
                    .filter(kind.eq(TriggerKind::Scheduled as i32))
                    .filter(param.eq(&script_name)),
            )
            .execute(&mut conn)
            .ok();
        }

        let website_regex = Regex::new(r#"(?i)USE\s+WEBSITE\s+"([^"]+)"(?:\s+REFRESH\s+"([^"]+)")?"#)
            .unwrap_or_else(|_| Regex::new(r"").unwrap());

        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty()
                || trimmed.starts_with('\'')
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
                    #[cfg(feature = "tasks")]
                    {
                        #[allow(unused_variables, unused_mut)]
                        let cron = parts[1];
                        #[allow(unused_variables, unused_mut)]
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
                    }
                    #[cfg(not(feature = "tasks"))]
                    log::warn!("SET SCHEDULE requires 'tasks' feature - ignoring");
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

            if trimmed.to_uppercase().starts_with("USE WEBSITE") {
                if let Some(caps) = website_regex.captures(&normalized) {
                    if let Some(url_match) = caps.get(1) {
                        let url = url_match.as_str();
                        let refresh = caps.get(2).map(|m| m.as_str()).unwrap_or("1m");
                        let mut conn = self
                            .state
                            .conn
                            .get()
                            .map_err(|e| format!("Failed to get database connection: {}", e))?;
                        if let Err(e) =
                            crate::basic::keywords::use_website::execute_use_website_preprocessing_with_refresh(
                                &mut conn, url, bot_id, refresh,
                            )
                        {
                            log::error!("Failed to register USE_WEBSITE during preprocessing: {}", e);
                        } else {
                            log::info!(
                                "Registered website {} for crawling during preprocessing (refresh: {})",
                                url, refresh
                            );
                        }
                        
                        result.push_str(&format!("USE_WEBSITE(\"{}\", \"{}\");\n", url, refresh));
                        continue;
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
            use crate::core::shared::models::system_automations::dsl::*;
            diesel::delete(
                system_automations
                    .filter(bot_id.eq(&bot_uuid))
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

        // Convert SAVE statements with field lists to map-based SAVE
        let result = match self.convert_save_statements(&result, bot_id) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("SAVE conversion failed: {}, using original code", e);
                result
            }
        };
        // Convert BEGIN TALK and BEGIN MAIL blocks to Rhai code
        let result = crate::basic::compiler::blocks::convert_begin_blocks(&result);
        // Convert IF ... THEN / END IF to if ... { }
        let result = crate::basic::ScriptService::convert_if_then_syntax(&result);
        // Convert SELECT ... CASE / END SELECT to match expressions
        let result = crate::basic::ScriptService::convert_select_case_syntax(&result);
        // Convert BASIC keywords to lowercase (but preserve variable casing)
        let result = crate::basic::ScriptService::convert_keywords_to_lowercase(&result);

        Ok(result)
    }

    /// Convert SAVE statements with field lists to map-based SAVE
    /// SAVE "table", field1, field2, ... -> let __data__ = #{field1: value1, ...}; SAVE "table", __data__
    fn convert_save_statements(
        &self,
        source: &str,
        bot_id: uuid::Uuid,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let mut result = String::new();
        let mut save_counter = 0;

        for line in source.lines() {
            let trimmed = line.trim();

            // Check if this is a SAVE statement with field list
            if trimmed.to_uppercase().starts_with("SAVE ") {
                if let Some(converted) = self.convert_save_line(line, bot_id, &mut save_counter)? {
                    result.push_str(&converted);
                    result.push('\n');
                    continue;
                }
            }

            result.push_str(line);
            result.push('\n');
        }

        Ok(result)
    }

    /// Convert a single SAVE statement line if it has a field list
    fn convert_save_line(
        &self,
        line: &str,
        bot_id: uuid::Uuid,
        save_counter: &mut usize,
    ) -> Result<Option<String>, Box<dyn Error + Send + Sync>> {
        let trimmed = line.trim();

        // Parse SAVE statement
        // Format: SAVE "table", value1, value2, ...
        let upper = trimmed.to_uppercase();
        if !upper.starts_with("SAVE ") {
            return Ok(None);
        }

        // Extract the content after "SAVE"
        let content = &trimmed[4..].trim();

        // Parse table name and values
        let parts = self.parse_save_statement(content)?;

        // If only 2 parts (table + data map), leave as-is (structured SAVE)
        if parts.len() <= 2 {
            return Ok(None);
        }

        // This is a field list SAVE - convert to map-based SAVE
        let table_name = &parts[0];

        // Strip quotes from table name if present
        let table_name = table_name.trim_matches('"');

        // Debug log to see what we're querying
        log::info!("[SAVE] Converting SAVE for table: '{}' (original: '{}')", table_name, &parts[0]);

        // Get column names from TABLE definition (preserves order from .bas file)
        let column_names = self.get_table_columns_for_save(table_name, bot_id)?;

        // Build the map by matching variable names to column names (case-insensitive)
        let values: Vec<&String> = parts.iter().skip(1).collect();
        let mut map_pairs = Vec::new();

        log::info!("[SAVE] Matching {} variables to {} columns", values.len(), column_names.len());

        for value_var in values.iter() {
            // Find the column that matches this variable (case-insensitive)
            let value_lower = value_var.to_lowercase();

            if let Some(column_name) = column_names.iter().find(|col| col.to_lowercase() == value_lower) {
                map_pairs.push(format!("{}: {}", column_name, value_var));
            } else {
                log::warn!("[SAVE] No matching column for variable '{}'", value_var);
            }
        }

        let map_expr = format!("#{{{}}}", map_pairs.join(", "));
        let data_var = format!("__save_data_{}__", save_counter);
        *save_counter += 1;

        // Generate: let __save_data_N__ = #{...}; SAVE "table", __save_data_N__
        let converted = format!("let {} = {}; SAVE {}, {}", data_var, map_expr, table_name, data_var);

        Ok(Some(converted))
    }

    /// Parse SAVE statement into parts
    fn parse_save_statement(&self, content: &str) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        // Simple parsing - split by comma, but respect quoted strings
        let mut parts = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut chars = content.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '"' if chars.peek() == Some(&'"') => {
                    // Escaped quote
                    current.push('"');
                    chars.next();
                }
                '"' => {
                    in_quotes = !in_quotes;
                    current.push('"');
                }
                ',' if !in_quotes => {
                    parts.push(current.trim().to_string());
                    current = String::new();
                }
                _ => {
                    current.push(c);
                }
            }
        }

        if !current.trim().is_empty() {
            parts.push(current.trim().to_string());
        }

        Ok(parts)
    }

    /// Get column names for a table from TABLE definition (preserves field order)
    fn get_table_columns_for_save(
        &self,
        table_name: &str,
        bot_id: uuid::Uuid,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        // Try to parse TABLE definition from the bot's .bas files to get correct field order
        if let Ok(columns) = self.get_columns_from_table_definition(table_name, bot_id) {
            if !columns.is_empty() {
                log::info!("Using TABLE definition for '{}': {} columns", table_name, columns.len());
                return Ok(columns);
            }
        }

        // Fallback to database schema query (may have different order)
        self.get_columns_from_database_schema(table_name, bot_id)
    }

    /// Parse TABLE definition from .bas files to get field order
    fn get_columns_from_table_definition(
        &self,
        table_name: &str,
        bot_id: uuid::Uuid,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        // use std::path::Path;

        // Find the tables.bas file in the bot's data directory
        let bot_name = self.get_bot_name_by_id(bot_id)?;
        let tables_path = format!("/opt/gbo/data/{}.gbai/{}.gbdialog/tables.bas", bot_name, bot_name);

        let tables_content = fs::read_to_string(&tables_path)?;
        let columns = self.parse_table_definition_for_fields(&tables_content, table_name)?;

        Ok(columns)
    }

    /// Parse TABLE definition and extract field names in order
    fn parse_table_definition_for_fields(
        &self,
        content: &str,
        table_name: &str,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        let mut columns = Vec::new();
        let mut in_target_table = false;

        for line in content.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("TABLE ") && trimmed.contains(table_name) {
                in_target_table = true;
                continue;
            }

            if in_target_table {
                if trimmed.starts_with("END TABLE") {
                    break;
                }

                if trimmed.starts_with("FIELD ") {
                    // Parse: FIELD fieldName AS TYPE
                    let parts: Vec<&str> = trimmed.split_whitespace().collect();
                    if parts.len() >= 2 {
                        let field_name = parts[1].to_string();
                        columns.push(field_name);
                    }
                }
            }
        }

        Ok(columns)
    }

    /// Get bot name by bot_id
    fn get_bot_name_by_id(&self, bot_id: uuid::Uuid) -> Result<String, Box<dyn Error + Send + Sync>> {
        use crate::core::shared::models::schema::bots::dsl::*;
        use diesel::QueryDsl;

        let mut conn = self.state.conn.get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let bot_name: String = bots
            .filter(id.eq(&bot_id))
            .select(name)
            .first(&mut conn)
            .map_err(|e| format!("Failed to get bot name: {}", e))?;

        Ok(bot_name)
    }

    /// Get column names from database schema (fallback, order may differ)
    fn get_columns_from_database_schema(
        &self,
        table_name: &str,
        bot_id: uuid::Uuid,
    ) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
        use diesel::sql_query;
        use diesel::sql_types::Text;
        use diesel::RunQueryDsl;

        #[derive(QueryableByName)]
        struct ColumnRow {
            #[diesel(sql_type = Text)]
            column_name: String,
        }

        // First, try to get columns from the main database's information_schema
        // This works because tables are created in the bot's database which shares the schema
        let mut conn = self.state.conn.get()
            .map_err(|e| format!("Failed to get DB connection: {}", e))?;

        let query = format!(
            "SELECT column_name FROM information_schema.columns \
             WHERE table_name = '{}' AND table_schema = 'public' \
             ORDER BY ordinal_position",
            table_name
        );

        let columns: Vec<String> = match sql_query(&query).load(&mut conn) {
            Ok(cols) => {
                if cols.is_empty() {
                    log::warn!("Found 0 columns for table '{}' in main database, trying bot database", table_name);
                    // Try bot's database as fallback when main DB returns empty
                    let bot_pool = self.state.bot_database_manager.get_bot_pool(bot_id);
                    if let Ok(pool) = bot_pool {
                        let mut bot_conn = pool.get()
                            .map_err(|e| format!("Bot DB error: {}", e))?;

                        let bot_query = format!(
                            "SELECT column_name FROM information_schema.columns \
                             WHERE table_name = '{}' AND table_schema = 'public' \
                             ORDER BY ordinal_position",
                            table_name
                        );

                        match sql_query(&bot_query).load(&mut *bot_conn) {
                            Ok(bot_cols) => {
                                log::info!("Found {} columns for table '{}' in bot database", bot_cols.len(), table_name);
                                bot_cols.into_iter()
                                    .map(|c: ColumnRow| c.column_name)
                                    .collect()
                            }
                            Err(e) => {
                                log::error!("Failed to get columns from bot DB for '{}': {}", table_name, e);
                                Vec::new()
                            }
                        }
                    } else {
                        log::error!("No bot database available for bot_id: {}", bot_id);
                        Vec::new()
                    }
                } else {
                    log::info!("Found {} columns for table '{}' in main database", cols.len(), table_name);
                    cols.into_iter()
                        .map(|c: ColumnRow| c.column_name)
                        .collect()
                }
            }
            Err(e) => {
                log::warn!("Failed to get columns for table '{}' from main DB: {}", table_name, e);

                // Try bot's database as fallback
                let bot_pool = self.state.bot_database_manager.get_bot_pool(bot_id);
                if let Ok(pool) = bot_pool {
                    let mut bot_conn = pool.get()
                        .map_err(|e| format!("Bot DB error: {}", e))?;

                    let bot_query = format!(
                        "SELECT column_name FROM information_schema.columns \
                         WHERE table_name = '{}' AND table_schema = 'public' \
                         ORDER BY ordinal_position",
                        table_name
                    );

                    match sql_query(&bot_query).load(&mut *bot_conn) {
                        Ok(cols) => {
                            log::info!("Found {} columns for table '{}' in bot database", cols.len(), table_name);
                            cols.into_iter()
                                .filter(|c: &ColumnRow| c.column_name != "id")
                                .map(|c: ColumnRow| c.column_name)
                                .collect()
                        }
                        Err(e) => {
                            log::error!("Failed to get columns from bot DB for '{}': {}", table_name, e);
                            Vec::new()
                        }
                    }
                } else {
                    log::error!("No bot database available for bot_id: {}", bot_id);
                    Vec::new()
                }
            }
        };

        Ok(columns)
    }
}
#[derive(Debug)]
pub struct CompilationResult {
    pub mcp_tool: Option<MCPTool>,
    pub openai_tool: Option<OpenAITool>,
}
