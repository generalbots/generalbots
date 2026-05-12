#[cfg(feature = "tasks")]
use crate::basic::keywords::set_schedule::execute_set_schedule;
use crate::basic::keywords::webhook::execute_webhook_registration;
use crate::basic::AppStateBasicRuntime;
use botbasic_types::BasicRuntime;
use botcore::shared::models::TriggerKind;
use botcore::shared::state::AppState;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use log::trace;
use regex::Regex;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::Arc;

pub mod blocks;
pub mod goto_transform;
pub mod save_conversion;
pub mod syntax_transforms;
pub mod tool_parsing;
pub mod types;

pub use types::*;

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

        let should_process_tables = if source_path.contains("tables.bas") {
            let work_path = botcore::shared::utils::get_work_path();
            let bot_name = Self::get_bot_name_from_state(&self.state, self.bot_id)?;
            let tables_bas_path = format!(
                "{}/{}.gbai/{}.gbdialog/tables.bas",
                work_path, bot_name, bot_name
            );
            let tables_ast_path = tables_bas_path.replace(".bas", ".ast");

            match (
                std::fs::metadata(&tables_bas_path).ok(),
                std::fs::metadata(&tables_ast_path).ok(),
            ) {
                (Some(bas_meta), Some(ast_meta)) => {
                    bas_meta.modified().ok() > ast_meta.modified().ok()
                }
                _ => true,
            }
        } else {
            true
        };

        if should_process_tables {
            if let Err(e) = Self::process_tables_bas(&self.state, self.bot_id) {
                log::warn!("Failed to process tables.bas: {}", e);
            }
            let tool_def = self.parse_tool_definition(&source_content, source_path)?;
            let file_name = Path::new(source_path)
                .file_stem()
                .and_then(|s| s.to_str())
                .ok_or("Invalid file name")?;
            let source_with_suggestions =
                self.generate_enum_suggestions(&source_content, &tool_def)?;
            let ast_path = format!("{output_dir}/{file_name}.ast");
            let ast_content =
                self.preprocess_basic(&source_with_suggestions, source_path, self.bot_id)?;
            fs::write(&ast_path, &ast_content).map_err(|e| format!("Failed to write AST: {e}"))?;
            return Ok(CompilationResult {
                mcp_tool: None,
                openai_tool: None,
            });
        }

        if let Err(e) = {
            let runtime: Arc<dyn BasicRuntime> = Arc::new(AppStateBasicRuntime(Arc::clone(&self.state)));
            crate::basic::keywords::table_definition::process_table_definitions(runtime, self.bot_id, &source_content)
        } {
            log::warn!("Failed to process TABLE definitions: {}", e);
        }

        let tool_def = self.parse_tool_definition(&source_content, source_path)?;
        let file_name = Path::new(source_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or("Invalid file name")?;

        let source_with_suggestions = self.generate_enum_suggestions(&source_content, &tool_def)?;

        let ast_path = format!("{output_dir}/{file_name}.ast");
        let ast_content =
            self.preprocess_basic(&source_with_suggestions, source_path, self.bot_id)?;
        fs::write(&ast_path, &ast_content).map_err(|e| format!("Failed to write AST file: {e}"))?;
        let (mcp_json, tool_json) = if tool_def.parameters.is_empty() {
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

        let source = syntax_transforms::preprocess_llm_keyword(source);
        let mut has_schedule = false;
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
            use botcore::shared::schema::system_automations::dsl::*;
            diesel::delete(
                system_automations
                    .filter(bot_id.eq(&bot_uuid))
                    .filter(kind.eq(TriggerKind::Scheduled as i32))
                    .filter(param.eq(&script_name)),
            )
            .execute(&mut conn)
            .ok();
        }

        let website_regex =
            Regex::new(r#"(?i)USE\s+WEBSITE\s+"([^"]+)"(?:\s+REFRESH\s+"([^"]+)")?"#)?;

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
                .replace("GROUP BY", "GROUP_BY")
                .replace("ADD SUGGESTION TOOL", "ADD_SUGGESTION_TOOL")
                .replace("ADD SUGGESTION TEXT", "ADD_SUGGESTION_TEXT")
                .replace("ADD SUGGESTION", "ADD_SUGGESTION")
                .replace("ADD SWITCHER", "ADD_SWITCHER");
            if normalized.starts_with("SET SCHEDULE") || trimmed.starts_with("SET SCHEDULE") {
                has_schedule = true;
                let parts: Vec<&str> = normalized.split('"').collect();
                if parts.len() >= 3 {
                    #[cfg(feature = "tasks")]
                    {
                        let cron = parts[1];
                        let mut conn = self
                            .state
                            .conn
                            .get()
                            .map_err(|e| format!("Failed to get database connection: {e}"))?;
                        if let Err(e) = execute_set_schedule(&mut conn, cron, &script_name, bot_id)
                        {
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
                        log::trace!(
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
                            log::trace!(
                                "Registered website {} for crawling during preprocessing (refresh: {})",
                                url,
                                refresh
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
            use botcore::shared::schema::system_automations::dsl::*;
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

        let result = match self.convert_save_statements(&result, bot_id) {
            Ok(r) => r,
            Err(e) => {
                log::warn!("SAVE conversion failed: {}, using original code", e);
                result
            }
        };
        let result = blocks::convert_begin_blocks(&result);
        let result = syntax_transforms::convert_multiword_keywords(&result);
        let result = syntax_transforms::convert_while_wend_syntax(&result);
        let result = syntax_transforms::predeclare_variables(&result);
        let result = syntax_transforms::convert_if_then_syntax(&result);
        let result = syntax_transforms::convert_select_case_syntax(&result);
        let result = syntax_transforms::convert_keywords_to_lowercase(&result);

        Ok(result)
    }
}
