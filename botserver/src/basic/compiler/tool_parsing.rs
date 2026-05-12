use super::types::*;
use std::collections::HashMap;
use std::error::Error;
use log::warn;

impl super::BasicCompiler {
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
        let tool_name = std::path::Path::new(source_path)
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

    pub(crate) fn parse_param_line(
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

        let enum_values = if let Some(enum_pos) = line.find("ENUM") {
            let rest = &line[enum_pos + 4..].trim();
            if let Some(start) = rest.find('[') {
                if let Some(end) = rest[start..].find(']') {
                    let array_content = &rest[start + 1..start + end];
                    let values: Vec<String> = array_content
                        .split(',')
                        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_string())
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

    pub(crate) fn normalize_type(basic_type: &str) -> String {
        match basic_type.to_lowercase().as_str() {
            "integer" | "int" | "number" => "integer".to_string(),
            "float" | "double" | "decimal" => "number".to_string(),
            "boolean" | "bool" => "boolean".to_string(),
            "array" | "list" => "array".to_string(),
            "object" | "map" => "object".to_string(),
            _ => "string".to_string(),
        }
    }

    pub(crate) fn generate_enum_suggestions(
        &self,
        source: &str,
        tool_def: &ToolDefinition,
    ) -> Result<String, Box<dyn Error + Send + Sync>> {
        let mut result = String::new();
        let mut suggestion_lines = Vec::new();

        for param in &tool_def.parameters {
            if let Some(ref enum_values) = param.enum_values {
                for enum_value in enum_values {
                    let suggestion_cmd = format!(
                        "ADD SUGGESTION TEXT \"{}\" AS \"{}\"",
                        enum_value, enum_value
                    );
                    suggestion_lines.push(suggestion_cmd);
                }
            }
        }

        let lines: Vec<&str> = source.lines().collect();
        let mut inserted = false;

        for line in lines.iter() {
            result.push_str(line);
            result.push('\n');

            if !inserted && line.trim().starts_with("DESCRIPTION ") {
                for suggestion in &suggestion_lines {
                    result.push_str(suggestion);
                    result.push('\n');
                }
                inserted = true;
            }
        }

        if !inserted && !suggestion_lines.is_empty() {
            for suggestion in &suggestion_lines {
                result.push_str(suggestion);
                result.push('\n');
            }
        }

        Ok(result)
    }

    pub(crate) fn generate_mcp_tool(
        tool_def: &ToolDefinition,
    ) -> Result<MCPTool, Box<dyn Error + Send + Sync>> {
        let mut properties = HashMap::new();
        let mut required = Vec::new();
        for param in &tool_def.parameters {
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

    pub(crate) fn generate_openai_tool(
        tool_def: &ToolDefinition,
    ) -> Result<OpenAITool, Box<dyn Error + Send + Sync>> {
        let mut properties = HashMap::new();
        let mut required = Vec::new();
        for param in &tool_def.parameters {
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
}
