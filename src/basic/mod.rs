#[cfg(feature = "chat")]
use crate::basic::keywords::add_suggestion::clear_suggestions_keyword;
use crate::basic::keywords::set_user::set_user_keyword;
use crate::basic::keywords::string_functions::register_string_functions;
use crate::basic::keywords::switch_case::switch_keyword;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use diesel::prelude::*;
use log::info;
use rhai::{Dynamic, Engine, EvalAltResult, Scope};
use std::collections::HashMap;
use std::sync::Arc;
pub mod compiler;
pub mod keywords;

#[derive(QueryableByName)]
struct ParamConfigRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    config_key: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    config_value: String,
}

// ===== CORE KEYWORD IMPORTS (always available) =====
#[cfg(feature = "chat")]
use self::keywords::add_bot::register_bot_keywords;
#[cfg(feature = "chat")]
use self::keywords::add_member::add_member_keyword;
#[cfg(feature = "chat")]
use self::keywords::add_suggestion::add_suggestion_keyword;
#[cfg(feature = "llm")]
use self::keywords::ai_tools::register_ai_tools_keywords;
use self::keywords::bot_memory::{get_bot_memory_keyword, set_bot_memory_keyword};
use self::keywords::clear_tools::clear_tools_keyword;
use self::keywords::core_functions::register_core_functions;
use self::keywords::data_operations::register_data_operations;
use self::keywords::find::find_keyword;
use self::keywords::search::search_keyword;
#[cfg(feature = "billing")]
use self::keywords::products::products_keyword;
use self::keywords::first::first_keyword;
use self::keywords::for_next::for_keyword;
use self::keywords::format::format_keyword;
use self::keywords::get::get_keyword;
use self::keywords::hear_talk::{hear_keyword, talk_keyword};
use self::keywords::http_operations::register_http_operations;
use self::keywords::last::last_keyword;
#[cfg(feature = "automation")]
use self::keywords::on_form_submit::on_form_submit_keyword;
use self::keywords::switch_case::preprocess_switch;
use self::keywords::use_tool::use_tool_keyword;
use self::keywords::use_website::{clear_websites_keyword, register_use_website_function};
use self::keywords::web_data::register_web_data_keywords;
#[cfg(feature = "automation")]
use self::keywords::webhook::webhook_keyword;
#[cfg(feature = "llm")]
use self::keywords::llm_keyword::llm_keyword;
use self::keywords::on::on_keyword;
use self::keywords::print::print_keyword;
use self::keywords::set::set_keyword;
use self::keywords::set_context::set_context_keyword;
use self::keywords::wait::wait_keyword;

// ===== CALENDAR FEATURE IMPORTS =====
#[cfg(feature = "calendar")]
use self::keywords::book::book_keyword;

// ===== MAIL FEATURE IMPORTS =====
#[cfg(feature = "mail")]
use self::keywords::create_draft::create_draft_keyword;
#[cfg(feature = "mail")]
use self::keywords::on_email::on_email_keyword;
#[cfg(feature = "mail")]
use self::keywords::send_mail::send_mail_keyword;
#[cfg(feature = "mail")]
use self::keywords::send_template::register_send_template_keywords;

// ===== TASKS FEATURE IMPORTS =====
#[cfg(feature = "tasks")]
use self::keywords::create_task::create_task_keyword;

// ===== SOCIAL FEATURE IMPORTS =====
#[cfg(feature = "social")]
use self::keywords::social_media::register_social_media_keywords;

// ===== LLM FEATURE IMPORTS =====
#[cfg(feature = "llm")]
use self::keywords::model_routing::register_model_routing_keywords;
#[cfg(feature = "llm")]
use self::keywords::multimodal::register_multimodal_keywords;
#[cfg(feature = "llm")]
use self::keywords::remember::remember_keyword;
#[cfg(feature = "llm")]
use self::keywords::save_from_unstructured::save_from_unstructured_keyword;

// ===== VECTORDB FEATURE IMPORTS =====
#[cfg(feature = "vectordb")]
use self::keywords::clear_kb::register_clear_kb_keyword;
#[cfg(feature = "vectordb")]
use self::keywords::use_kb::register_use_kb_keyword;

// ===== DRIVE FEATURE IMPORTS =====
#[cfg(feature = "drive")]
use self::keywords::file_operations::register_file_operations;
#[cfg(feature = "drive")]
use self::keywords::create_site::create_site_keyword;

// ===== PEOPLE FEATURE IMPORTS =====
#[cfg(feature = "people")]
use self::keywords::lead_scoring::register_lead_scoring_keywords;

// ===== COMMUNICATIONS FEATURE IMPORTS =====
#[cfg(any(feature = "whatsapp", feature = "telegram", feature = "mail"))]
use self::keywords::sms::register_sms_keywords;

// ===== CHAT FEATURE IMPORTS =====
#[cfg(feature = "chat")]
use self::keywords::transfer_to_human::register_transfer_to_human_keyword;

#[derive(Debug)]
pub struct ScriptService {
    pub engine: Engine,
    pub scope: Scope<'static>,
}

impl ScriptService {
    #[must_use]
    pub fn new(state: Arc<AppState>, user: UserSession) -> Self {
        let mut engine = Engine::new();
        let scope = Scope::new();
        engine.set_allow_anonymous_fn(true);
        engine.set_allow_looping(true);

        // ===== CORE KEYWORDS (always available) =====
        set_bot_memory_keyword(state.clone(), user.clone(), &mut engine);
        get_bot_memory_keyword(state.clone(), user.clone(), &mut engine);
        find_keyword(&state, user.clone(), &mut engine);
        search_keyword(&state, user.clone(), &mut engine);
        #[cfg(feature = "billing")]
        products_keyword(&state, user.clone(), &mut engine);
        for_keyword(&state, user.clone(), &mut engine);
        first_keyword(&mut engine);
        last_keyword(&mut engine);
        format_keyword(&mut engine);
        #[cfg(feature = "llm")]
        llm_keyword(state.clone(), user.clone(), &mut engine);
        get_keyword(state.clone(), user.clone(), &mut engine);
        set_keyword(&state, user.clone(), &mut engine);
        wait_keyword(&state, user.clone(), &mut engine);
        print_keyword(&state, user.clone(), &mut engine);
        on_keyword(&state, user.clone(), &mut engine);
        hear_keyword(state.clone(), user.clone(), &mut engine);
        talk_keyword(state.clone(), user.clone(), &mut engine);
        set_context_keyword(state.clone(), user.clone(), &mut engine);
        set_user_keyword(state.clone(), user.clone(), &mut engine);
        #[cfg(feature = "chat")]
        clear_suggestions_keyword(state.clone(), user.clone(), &mut engine);
        use_tool_keyword(state.clone(), user.clone(), &mut engine);
        clear_tools_keyword(state.clone(), user.clone(), &mut engine);
        clear_websites_keyword(state.clone(), user.clone(), &mut engine);
        #[cfg(feature = "chat")]
        add_suggestion_keyword(state.clone(), user.clone(), &mut engine);
        #[cfg(feature = "chat")]
        add_member_keyword(state.clone(), user.clone(), &mut engine);
        #[cfg(feature = "chat")]
        register_bot_keywords(&state, &user, &mut engine);

        // ===== PROCEDURE KEYWORDS (RETURN, etc.) =====
        keywords::procedures::register_procedure_keywords(state.clone(), user.clone(), &mut engine);

        // ===== WORKFLOW ORCHESTRATION KEYWORDS =====
        keywords::orchestration::register_orchestrate_workflow(state.clone(), user.clone(), &mut engine);
        keywords::orchestration::register_step_keyword(state.clone(), user.clone(), &mut engine);
        keywords::events::register_on_event(state.clone(), user.clone(), &mut engine);
        keywords::events::register_publish_event(state.clone(), user.clone(), &mut engine);
        keywords::events::register_wait_for_event(state.clone(), user.clone(), &mut engine);
        keywords::enhanced_memory::register_bot_share_memory(state.clone(), user.clone(), &mut engine);
        keywords::enhanced_memory::register_bot_sync_memory(state.clone(), user.clone(), &mut engine);
        keywords::enhanced_llm::register_enhanced_llm_keyword(state.clone(), user.clone(), &mut engine);

        keywords::universal_messaging::register_universal_messaging(
            state.clone(),
            user.clone(),
            &mut engine,
        );
        register_string_functions(state.clone(), user.clone(), &mut engine);
        switch_keyword(&state, user.clone(), &mut engine);
        register_http_operations(state.clone(), user.clone(), &mut engine);
        register_data_operations(state.clone(), user.clone(), &mut engine);
        #[cfg(feature = "automation")]
        webhook_keyword(&state, user.clone(), &mut engine);
        #[cfg(feature = "automation")]
        on_form_submit_keyword(state.clone(), user.clone(), &mut engine);
        #[cfg(feature = "llm")]
        register_ai_tools_keywords(state.clone(), user.clone(), &mut engine);
        register_web_data_keywords(state.clone(), user.clone(), &mut engine);
        register_core_functions(state.clone(), user.clone(), &mut engine);

        // ===== MAIL FEATURE KEYWORDS =====
        #[cfg(feature = "mail")]
        {
            create_draft_keyword(&state, user.clone(), &mut engine);
            on_email_keyword(&state, user.clone(), &mut engine);
            send_mail_keyword(state.clone(), user.clone(), &mut engine);
            register_send_template_keywords(state.clone(), user.clone(), &mut engine);
        }

        // ===== CALENDAR FEATURE KEYWORDS =====
        #[cfg(feature = "calendar")]
        {
            book_keyword(state.clone(), user.clone(), &mut engine);
        }

        // ===== TASKS FEATURE KEYWORDS =====
        #[cfg(feature = "tasks")]
        {
            create_task_keyword(state.clone(), user.clone(), &mut engine);
        }

        // ===== LLM FEATURE KEYWORDS =====
        #[cfg(feature = "llm")]
        {
            register_model_routing_keywords(state.clone(), user.clone(), &mut engine);
            register_multimodal_keywords(state.clone(), user.clone(), &mut engine);
            remember_keyword(state.clone(), user.clone(), &mut engine);
            save_from_unstructured_keyword(state.clone(), user.clone(), &mut engine);
        }

        // Register USE WEBSITE after all other USE keywords to avoid conflicts
        // USE WEBSITE is now preprocessed to USE_WEBSITE function call
        // Register it as a regular function instead of custom syntax
        register_use_website_function(state.clone(), user.clone(), &mut engine);

        // ===== VECTORDB FEATURE KEYWORDS =====
        #[cfg(feature = "vectordb")]
        {
            let _ = register_use_kb_keyword(&mut engine, state.clone(), Arc::new(user.clone()));
            let _ = register_clear_kb_keyword(&mut engine, state.clone(), Arc::new(user.clone()));
        }

        // ===== DRIVE FEATURE KEYWORDS =====
        #[cfg(feature = "drive")]
        {
            create_site_keyword(&state, user.clone(), &mut engine);
            register_file_operations(state.clone(), user.clone(), &mut engine);
        }

        // ===== SOCIAL FEATURE KEYWORDS =====
        #[cfg(feature = "social")]
        {
            register_social_media_keywords(state.clone(), user.clone(), &mut engine);
        }

        // ===== PEOPLE FEATURE KEYWORDS =====
        #[cfg(feature = "people")]
        {
            register_lead_scoring_keywords(state.clone(), user.clone(), &mut engine);
        }

        // ===== CHAT FEATURE KEYWORDS =====
        #[cfg(feature = "chat")]
        {
            register_transfer_to_human_keyword(state.clone(), user.clone(), &mut engine);
        }

        // ===== COMMUNICATIONS FEATURE KEYWORDS =====
        #[cfg(any(feature = "whatsapp", feature = "telegram", feature = "mail"))]
        {
            register_sms_keywords(state.clone(), user.clone(), &mut engine);
        }

        // Silence unused variable warning when features are disabled
        let _ = user;

        Self { engine, scope }
    }

    pub fn inject_config_variables(&mut self, config_vars: HashMap<String, String>) {
        for (key, value) in config_vars {
            let var_name = if key.starts_with("param-") {
                key.strip_prefix("param-").unwrap_or(&key).to_lowercase()
            } else {
                key.to_lowercase()
            };

            if let Ok(int_val) = value.parse::<i64>() {
                self.scope.push(&var_name, int_val);
            } else if let Ok(float_val) = value.parse::<f64>() {
                self.scope.push(&var_name, float_val);
            } else if value.eq_ignore_ascii_case("true") {
                self.scope.push(&var_name, true);
            } else if value.eq_ignore_ascii_case("false") {
                self.scope.push(&var_name, false);
            } else {
                self.scope.push(&var_name, value);
            }
        }
    }

    pub fn load_bot_config_params(&mut self, state: &AppState, bot_id: uuid::Uuid) {
        if let Ok(mut conn) = state.conn.get() {
            let result = diesel::sql_query(
                "SELECT config_key, config_value FROM bot_configuration WHERE bot_id = $1 AND config_key LIKE 'param-%'"
            )
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .load::<ParamConfigRow>(&mut conn);

            if let Ok(params) = result {
                let config_vars: HashMap<String, String> = params
                    .into_iter()
                    .map(|row| (row.config_key, row.config_value))
                    .collect();
                self.inject_config_variables(config_vars);
            }
        }
    }
    fn preprocess_basic_script(&self, script: &str) -> String {
        let _ = self; // silence unused self warning - kept for API consistency
        let script = preprocess_switch(script);

        // Convert ALL multi-word keywords to underscore versions (e.g., "USE WEBSITE" → "USE_WEBSITE")
        // This avoids Rhai custom syntax conflicts and makes the system more secure
        let script = Self::convert_multiword_keywords(&script);

        let script = Self::normalize_variables_to_lowercase(&script);

        let mut result = String::new();
        let mut for_stack: Vec<usize> = Vec::new();
        let mut current_indent = 0;
        for line in script.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('\'') {
                continue;
            }
            if trimmed.starts_with("FOR EACH") {
                for_stack.push(current_indent);
                result.push_str(&" ".repeat(current_indent));
                result.push_str(trimmed);
                result.push_str("{\n");
                current_indent += 4;
                result.push_str(&" ".repeat(current_indent));
                result.push('\n');
                continue;
            }
            if trimmed.starts_with("NEXT") {
                if let Some(expected_indent) = for_stack.pop() {
                    assert!(
                        (current_indent - 4) == expected_indent,
                        "NEXT without matching FOR EACH"
                    );
                    current_indent -= 4;
                    result.push_str(&" ".repeat(current_indent));
                    result.push_str("}\n");
                    result.push_str(&" ".repeat(current_indent));
                    result.push_str(trimmed);
                    result.push(';');
                    result.push('\n');
                    continue;
                }
                log::error!("NEXT without matching FOR EACH");
                return result;
            }
            if trimmed == "EXIT FOR" {
                result.push_str(&" ".repeat(current_indent));
                result.push_str(trimmed);
                result.push('\n');
                continue;
            }
            result.push_str(&" ".repeat(current_indent));
            let basic_commands = [
                "SET",
                "CREATE",
                "PRINT",
                "FOR",
                "FIND",
                "GET",
                "EXIT",
                "IF",
                "THEN",
                "ELSE",
                "END IF",
                "WHILE",
                "WEND",
                "DO",
                "LOOP",
                "HEAR",
                "TALK",
                "SET CONTEXT",
                "SET USER",
                "GET BOT MEMORY",
                "SET BOT MEMORY",
                "IMAGE",
                "VIDEO",
                "AUDIO",
                "SEE",
                "SEND FILE",
                "SWITCH",
                "CASE",
                "DEFAULT",
                "END SWITCH",
                "USE KB",
                "CLEAR KB",
                "USE TOOL",
                "CLEAR TOOLS",
                "ADD SUGGESTION",
                "CLEAR SUGGESTIONS",
                "INSTR",
                "IS_NUMERIC",
                "IS NUMERIC",
                "POST",
                "PUT",
                "PATCH",
                "DELETE",
                "SET HEADER",
                "CLEAR HEADERS",
                "GRAPHQL",
                "SOAP",
                "SAVE",
                "INSERT",
                "UPDATE",
                "DELETE",
                "MERGE",
                "FILL",
                "MAP",
                "FILTER",
                "AGGREGATE",
                "JOIN",
                "PIVOT",
                "GROUP BY",
                "READ",
                "WRITE",
                "COPY",
                "MOVE",
                "LIST",
                "COMPRESS",
                "EXTRACT",
                "UPLOAD",
                "DOWNLOAD",
                "GENERATE PDF",
                "MERGE PDF",
                "WEBHOOK",
                "POST TO",
                "POST TO INSTAGRAM",
                "POST TO FACEBOOK",
                "POST TO LINKEDIN",
                "POST TO TWITTER",
                "GET INSTAGRAM METRICS",
                "GET FACEBOOK METRICS",
                "GET LINKEDIN METRICS",
                "GET TWITTER METRICS",
                "DELETE POST",
                "SEND MAIL",
                "SEND TEMPLATE",
                "CREATE TEMPLATE",
                "GET TEMPLATE",
                "ON ERROR RESUME NEXT",
                "ON ERROR GOTO",
                "CLEAR ERROR",
                "ERROR MESSAGE",
                "ON FORM SUBMIT",
                "SCORE LEAD",
                "GET LEAD SCORE",
                "QUALIFY LEAD",
                "UPDATE LEAD SCORE",
                "AI SCORE LEAD",
                "ABS",
                "ROUND",
                "INT",
                "FIX",
                "FLOOR",
                "CEIL",
                "MAX",
                "MIN",
                "MOD",
                "RANDOM",
                "RND",
                "SGN",
                "SQR",
                "SQRT",
                "LOG",
                "EXP",
                "POW",
                "SIN",
                "COS",
                "TAN",
                "SUM",
                "AVG",
                "NOW",
                "TODAY",
                "DATE",
                "TIME",
                "YEAR",
                "MONTH",
                "DAY",
                "HOUR",
                "MINUTE",
                "SECOND",
                "WEEKDAY",
                "DATEADD",
                "DATEDIFF",
                "FORMAT_DATE",
                "ISDATE",
                "VAL",
                "STR",
                "CINT",
                "CDBL",
                "CSTR",
                "ISNULL",
                "ISEMPTY",
                "TYPEOF",
                "ISARRAY",
                "ISOBJECT",
                "ISSTRING",
                "ISNUMBER",
                "NVL",
                "IIF",
                "ARRAY",
                "UBOUND",
                "LBOUND",
                "COUNT",
                "SORT",
                "UNIQUE",
                "CONTAINS",
                "INDEX_OF",
                "PUSH",
                "POP",
                "SHIFT",
                "REVERSE",
                "SLICE",
                "SPLIT",
                "CONCAT",
                "FLATTEN",
                "RANGE",
                "THROW",
                "ERROR",
                "IS_ERROR",
                "ASSERT",
                "LOG_ERROR",
                "LOG_WARN",
                "LOG_INFO",
            ];
            let is_basic_command = basic_commands.iter().any(|&cmd| trimmed.starts_with(cmd));
            let is_control_flow = trimmed.starts_with("IF")
                || trimmed.starts_with("ELSE")
                || trimmed.starts_with("END IF");
            result.push_str(trimmed);
            let needs_semicolon = is_basic_command
                || !for_stack.is_empty()
                || is_control_flow
                || (!trimmed.ends_with(';') && !trimmed.ends_with('{') && !trimmed.ends_with('}'));
            if needs_semicolon {
                result.push(';');
            }
            result.push('\n');
        }
        assert!(for_stack.is_empty(), "Unclosed FOR EACH loop");
        result
    }
    pub fn compile(&self, script: &str) -> Result<rhai::AST, Box<EvalAltResult>> {
        let processed_script = self.preprocess_basic_script(script);
        info!("Processed Script:\n{}", processed_script);
        match self.engine.compile(&processed_script) {
            Ok(ast) => Ok(ast),
            Err(parse_error) => Err(Box::new(parse_error.into())),
        }
    }

    /// Compile a tool script (.bas file with PARAM/DESCRIPTION metadata lines)
    /// Filters out tool metadata before compiling
    pub fn compile_tool_script(&self, script: &str) -> Result<rhai::AST, Box<EvalAltResult>> {
        // Filter out PARAM, DESCRIPTION, comment, and empty lines (tool metadata)
        let executable_script: String = script
            .lines()
            .filter(|line| {
                let trimmed = line.trim();
                // Keep lines that are NOT PARAM, DESCRIPTION, comments, or empty
                !(trimmed.starts_with("PARAM ") ||
                  trimmed.starts_with("PARAM\t") ||
                  trimmed.starts_with("DESCRIPTION ") ||
                  trimmed.starts_with("DESCRIPTION\t") ||
                  trimmed.starts_with('\'') || // BASIC comment lines
                  trimmed.is_empty())
            })
            .collect::<Vec<&str>>()
            .join("\n");

        info!("[TOOL] Filtered tool metadata: {} -> {} chars", script.len(), executable_script.len());

        // Apply minimal preprocessing for tools (skip variable normalization to avoid breaking multi-line strings)
        let script = preprocess_switch(&executable_script);
        let script = Self::convert_multiword_keywords(&script);
        // Skip normalize_variables_to_lowercase for tools - it breaks multi-line strings
        // Note: FORMAT is registered as a regular function, so FORMAT(expr, pattern) works directly

        info!("[TOOL] Preprocessed tool script for Rhai compilation");
        // Convert IF ... THEN / END IF to if ... { }
        let script = Self::convert_if_then_syntax(&script);
        // Convert BASIC keywords to lowercase (but preserve variable casing)
        let script = Self::convert_keywords_to_lowercase(&script);
        // Save to file for debugging
        if let Err(e) = std::fs::write("/tmp/tool_preprocessed.bas", &script) {
            log::warn!("Failed to write preprocessed script: {}", e);
        }
        match self.engine.compile(&script) {
            Ok(ast) => Ok(ast),
            Err(parse_error) => Err(Box::new(parse_error.into())),
        }
    }
    pub fn run(&mut self, ast: &rhai::AST) -> Result<Dynamic, Box<EvalAltResult>> {
        self.engine.eval_ast_with_scope(&mut self.scope, ast)
    }

    /// Set a variable in the script scope (for tool parameters)
    pub fn set_variable(&mut self, name: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        use rhai::Dynamic;
        self.scope.set_or_push(name, Dynamic::from(value.to_string()));
        Ok(())
    }

    /// Convert FORMAT(expr, pattern) to FORMAT expr pattern (custom syntax format)
    /// Also handles RANDOM and other functions that need space-separated arguments
    fn convert_format_syntax(script: &str) -> String {
        use regex::Regex;
        let mut result = script.to_string();

        // First, process RANDOM to ensure commas are preserved
        // RANDOM(min, max) stays as RANDOM(min, max) - no conversion needed

        // Convert FORMAT(expr, pattern) → FORMAT expr pattern
        // Need to handle nested functions carefully
        // Match FORMAT( ... ) but don't include inner function parentheses
        // This regex matches FORMAT followed by parentheses containing two comma-separated expressions
        if let Ok(re) = Regex::new(r"(?i)FORMAT\s*\(([^()]+(?:\([^()]*\)[^()]*)*),([^)]+)\)") {
            result = re.replace_all(&result, "FORMAT $1$2").to_string();
        }

        result
    }

    /// Convert BASIC IF ... THEN / END IF syntax to Rhai's if ... { } syntax
    fn convert_if_then_syntax(script: &str) -> String {
        let mut result = String::new();
        let mut if_stack: Vec<bool> = Vec::new(); // Track if we're inside an IF block
        let mut in_with_block = false; // Track if we're inside a WITH block
        let mut line_buffer = String::new();

        log::info!("[TOOL] Converting IF/THEN syntax, input has {} lines", script.lines().count());

        for line in script.lines() {
            let trimmed = line.trim();
            let upper = trimmed.to_uppercase();

            // Skip empty lines and comments
            if trimmed.is_empty() || trimmed.starts_with('\'') || trimmed.starts_with("//") {
                continue;
            }

            // Handle IF ... THEN
            if upper.starts_with("IF ") && upper.contains(" THEN") {
                let then_pos = upper.find(" THEN").unwrap();
                let condition = &trimmed[3..then_pos].trim();
                log::info!("[TOOL] Converting IF statement: condition='{}'", condition);
                result.push_str("if ");
                result.push_str(condition);
                result.push_str(" {\n");
                if_stack.push(true);
                continue;
            }

            // Handle ELSE
            if upper == "ELSE" {
                log::info!("[TOOL] Converting ELSE statement");
                result.push_str("} else {\n");
                continue;
            }

            // Handle END IF
            if upper == "END IF" {
                log::info!("[TOOL] Converting END IF statement");
                if let Some(_) = if_stack.pop() {
                    result.push_str("}\n");
                }
                continue;
            }

            // Handle WITH ... END WITH (BASIC object creation)
            if upper.starts_with("WITH ") {
                let object_name = &trimmed[5..].trim();
                log::info!("[TOOL] Converting WITH statement: object='{}'", object_name);
                // Convert WITH obj → let obj = #{  (start object literal)
                result.push_str("let ");
                result.push_str(object_name);
                result.push_str(" = #{\n");
                in_with_block = true;
                continue;
            }

            if upper == "END WITH" {
                log::info!("[TOOL] Converting END WITH statement");
                result.push_str("};\n");
                in_with_block = false;
                continue;
            }

            // Inside a WITH block - convert property assignments (key = value → key: value)
            if in_with_block {
                // Check if this is a property assignment (identifier = value)
                if trimmed.contains('=') && !trimmed.contains("==") && !trimmed.contains("!=") && !trimmed.contains("+=") && !trimmed.contains("-=") {
                    // Convert assignment to object property syntax
                    let parts: Vec<&str> = trimmed.splitn(2, '=').collect();
                    if parts.len() == 2 {
                        let property_name = parts[0].trim();
                        let property_value = parts[1].trim();
                        // Remove trailing semicolon if present
                        let property_value = property_value.trim_end_matches(';');
                        result.push_str(&format!("    {}: {},\n", property_name, property_value));
                        continue;
                    }
                }
                // Regular line in WITH block - add indentation
                result.push_str("    ");
            }

            // Handle SAVE table, object → INSERT table, object
            // BASIC SAVE uses 2 parameters but Rhai SAVE needs 3
            // INSERT uses 2 parameters which matches the BASIC syntax
            if upper.starts_with("SAVE") && upper.contains(',') {
                log::info!("[TOOL] Processing SAVE line: '{}'", trimmed);
                // Extract table and object name
                let after_save = &trimmed[4..].trim(); // Skip "SAVE"
                let parts: Vec<&str> = after_save.split(',').collect();
                log::info!("[TOOL] SAVE parts: {:?}", parts);
                if parts.len() == 2 {
                    let table = parts[0].trim().trim_matches('"');
                    let object_name = parts[1].trim().trim_end_matches(';');
                    // Convert to INSERT table, object
                    let converted = format!("INSERT \"{}\", {};\n", table, object_name);
                    log::info!("[TOOL] Converted SAVE to INSERT: '{}'", converted);
                    result.push_str(&converted);
                    continue;
                }
            }

            // Handle SEND EMAIL → send_mail (function call style)
            // Syntax: SEND EMAIL to, subject, body → send_mail(to, subject, body, [])
            if upper.starts_with("SEND EMAIL") {
                log::info!("[TOOL] Processing SEND EMAIL line: '{}'", trimmed);
                let after_send = &trimmed[11..].trim(); // Skip "SEND EMAIL " (10 chars + space = 11)
                let parts: Vec<&str> = after_send.split(',').collect();
                log::info!("[TOOL] SEND EMAIL parts: {:?}", parts);
                if parts.len() == 3 {
                    let to = parts[0].trim();
                    let subject = parts[1].trim();
                    let body = parts[2].trim().trim_end_matches(';');
                    // Convert to send_mail(to, subject, body, []) function call
                    let converted = format!("send_mail({}, {}, {}, []);\n", to, subject, body);
                    log::info!("[TOOL] Converted SEND EMAIL to: '{}'", converted);
                    result.push_str(&converted);
                    continue;
                }
            }

            // Regular line - add indentation if inside IF block
            if !if_stack.is_empty() {
                result.push_str("    ");
            }

            // Check if line is a simple statement (not containing THEN or other control flow)
            if !upper.starts_with("IF ") && !upper.starts_with("ELSE") && !upper.starts_with("END IF") {
                // Check if this is a variable assignment (identifier = expression)
                // Pattern: starts with letter/underscore, contains = but not ==, !=, <=, >=, +=, -=
                let is_var_assignment = trimmed.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_')
                    && trimmed.contains('=')
                    && !trimmed.contains("==")
                    && !trimmed.contains("!=")
                    && !trimmed.contains("<=")
                    && !trimmed.contains(">=")
                    && !trimmed.contains("+=")
                    && !trimmed.contains("-=")
                    && !trimmed.contains("*=")
                    && !trimmed.contains("/=");

                if is_var_assignment {
                    // Add 'let' for variable declarations
                    result.push_str("let ");
                }
                result.push_str(trimmed);
                // Add semicolon if line doesn't have one and doesn't end with { or }
                if !trimmed.ends_with(';') && !trimmed.ends_with('{') && !trimmed.ends_with('}') {
                    result.push(';');
                }
                result.push('\n');
            } else {
                result.push_str(trimmed);
                result.push('\n');
            }
        }

        log::info!("[TOOL] IF/THEN conversion complete, output has {} lines", result.lines().count());
        result
    }

    /// Convert BASIC keywords to lowercase without touching variables
    /// This is a simplified version of normalize_variables_to_lowercase for tools
    fn convert_keywords_to_lowercase(script: &str) -> String {
        let keywords = [
            "IF", "THEN", "ELSE", "END IF", "FOR", "NEXT", "WHILE", "WEND",
            "DO", "LOOP", "RETURN", "EXIT", "SELECT", "CASE", "END SELECT",
            "WITH", "END WITH", "AND", "OR", "NOT", "MOD",
            "DIM", "AS", "NEW", "FUNCTION", "SUB", "CALL",
        ];

        let mut result = String::new();
        for line in script.lines() {
            let mut processed_line = line.to_string();
            for keyword in &keywords {
                // Use word boundaries to avoid replacing parts of variable names
                let pattern = format!(r"\b{}\b", regex::escape(keyword));
                if let Ok(re) = regex::Regex::new(&pattern) {
                    processed_line = re.replace_all(&processed_line, keyword.to_lowercase()).to_string();
                }
            }
            result.push_str(&processed_line);
            result.push('\n');
        }
        result
    }

    fn normalize_variables_to_lowercase(script: &str) -> String {
        use regex::Regex;

        let mut result = String::new();

        let keywords = [
            "SET",
            "CREATE",
            "PRINT",
            "FOR",
            "FIND",
            "GET",
            "EXIT",
            "IF",
            "THEN",
            "ELSE",
            "END",
            "WHILE",
            "WEND",
            "DO",
            "LOOP",
            "HEAR",
            "TALK",
            "NEXT",
            "FUNCTION",
            "SUB",
            "CALL",
            "RETURN",
            "DIM",
            "AS",
            "NEW",
            "ARRAY",
            "OBJECT",
            "LET",
            "REM",
            "AND",
            "OR",
            "NOT",
            "TRUE",
            "FALSE",
            "NULL",
            "SWITCH",
            "CASE",
            "DEFAULT",
            "USE",
            "KB",
            "TOOL",
            "CLEAR",
            "ADD",
            "SUGGESTION",
            "SUGGESTIONS",
            "TOOLS",
            "CONTEXT",
            "USER",
            "BOT",
            "MEMORY",
            "IMAGE",
            "VIDEO",
            "AUDIO",
            "SEE",
            "SEND",
            "FILE",
            "POST",
            "PUT",
            "PATCH",
            "DELETE",
            "SAVE",
            "INSERT",
            "UPDATE",
            "MERGE",
            "FILL",
            "MAP",
            "FILTER",
            "AGGREGATE",
            "JOIN",
            "PIVOT",
            "GROUP",
            "BY",
            "READ",
            "WRITE",
            "COPY",
            "MOVE",
            "LIST",
            "COMPRESS",
            "EXTRACT",
            "UPLOAD",
            "DOWNLOAD",
            "GENERATE",
            "PDF",
            "WEBHOOK",
            "TEMPLATE",
            "FORM",
            "SUBMIT",
            "SCORE",
            "LEAD",
            "QUALIFY",
            "AI",
            "ABS",
            "ROUND",
            "INT",
            "FIX",
            "FLOOR",
            "CEIL",
            "MAX",
            "MIN",
            "MOD",
            "RANDOM",
            "RND",
            "SGN",
            "SQR",
            "SQRT",
            "LOG",
            "EXP",
            "POW",
            "SIN",
            "COS",
            "TAN",
            "SUM",
            "AVG",
            "NOW",
            "TODAY",
            "DATE",
            "TIME",
            "YEAR",
            "MONTH",
            "DAY",
            "HOUR",
            "MINUTE",
            "SECOND",
            "WEEKDAY",
            "DATEADD",
            "DATEDIFF",
            "FORMAT",
            "ISDATE",
            "VAL",
            "STR",
            "CINT",
            "CDBL",
            "CSTR",
            "ISNULL",
            "ISEMPTY",
            "TYPEOF",
            "ISARRAY",
            "ISOBJECT",
            "ISSTRING",
            "ISNUMBER",
            "NVL",
            "IIF",
            "UBOUND",
            "LBOUND",
            "COUNT",
            "SORT",
            "UNIQUE",
            "CONTAINS",
            "INDEX",
            "OF",
            "PUSH",
            "POP",
            "SHIFT",
            "REVERSE",
            "SLICE",
            "SPLIT",
            "CONCAT",
            "FLATTEN",
            "RANGE",
            "THROW",
            "ERROR",
            "IS",
            "ASSERT",
            "WARN",
            "INFO",
            "EACH",
            "WITH",
            "TO",
            "STEP",
            "BEGIN",
            "SYSTEM",
            "PROMPT",
            "SCHEDULE",
            "REFRESH",
            "ALLOW",
            "ROLE",
            "ANSWER",
            "MODE",
            "SYNCHRONIZE",
            "TABLE",
            "ON",
            "EMAIL",
            "REPORT",
            "RESET",
            "WAIT",
            "FIRST",
            "LAST",
            "LLM",
            "INSTR",
            "NUMERIC",
            "LEN",
            "LEFT",
            "RIGHT",
            "MID",
            "LOWER",
            "UPPER",
            "TRIM",
            "LTRIM",
            "RTRIM",
            "REPLACE",
            "LIKE",
            "DELEGATE",
            "PRIORITY",
            "BOTS",
            "REMOVE",
            "MEMBER",
            "BOOK",
            "REMEMBER",
            "TASK",
            "SITE",
            "DRAFT",
            "INSTAGRAM",
            "FACEBOOK",
            "LINKEDIN",
            "TWITTER",
            "METRICS",
            "HEADER",
            "HEADERS",
            "GRAPHQL",
            "SOAP",
            "HTTP",
            "DESCRIPTION",
            "PARAM",
            "REQUIRED",
            "WEBSITE",
            "MODEL",
        ];

        let _identifier_re = Regex::new(r"([a-zA-Z_][a-zA-Z0-9_]*)").expect("valid regex");

        for line in script.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("REM") || trimmed.starts_with('\'') || trimmed.starts_with("//")
            {
                continue;
            }

            // Skip lines with custom syntax that should not be lowercased
            // These are registered directly with Rhai in uppercase
            let trimmed_upper = trimmed.to_uppercase();
            if trimmed_upper.contains("ADD_SUGGESTION_TOOL") ||
               trimmed_upper.contains("ADD_SUGGESTION_TEXT") ||
               trimmed_upper.starts_with("ADD_SUGGESTION_") ||
               trimmed_upper.starts_with("ADD_MEMBER") {
                // Keep original line as-is
                result.push_str(line);
                result.push('\n');
                continue;
            }

            let mut processed_line = String::new();
            let mut chars = line.chars().peekable();
            let mut in_string = false;
            let mut string_char = '"';
            let mut current_word = String::new();

            while let Some(c) = chars.next() {
                if in_string {
                    processed_line.push(c);
                    if c == string_char {
                        in_string = false;
                    } else if c == '\\' {
                        if let Some(&next) = chars.peek() {
                            processed_line.push(next);
                            chars.next();
                        }
                    }
                } else if c == '"' || c == '\'' {
                    if !current_word.is_empty() {
                        processed_line.push_str(&Self::normalize_word(&current_word, &keywords));
                        current_word.clear();
                    }
                    in_string = true;
                    string_char = c;
                    processed_line.push(c);
                } else if c.is_alphanumeric() || c == '_' {
                    current_word.push(c);
                } else {
                    if !current_word.is_empty() {
                        processed_line.push_str(&Self::normalize_word(&current_word, &keywords));
                        current_word.clear();
                    }
                    processed_line.push(c);
                }
            }

            if !current_word.is_empty() {
                processed_line.push_str(&Self::normalize_word(&current_word, &keywords));
            }

            result.push_str(&processed_line);
            result.push('\n');
        }

        result
    }

    /// Convert ALL multi-word keywords to underscore versions (function calls)
    /// This avoids Rhai custom syntax conflicts and makes the system more secure
    ///
    /// Examples:
    /// - "USE WEBSITE "url"" → "USE_WEBSITE("url")"
    /// - "USE WEBSITE "url" REFRESH "interval"" → "USE_WEBSITE("url", "interval")"
    /// - "SET BOT MEMORY key AS value" → "SET_BOT_MEMORY(key, value)"
    /// - "CLEAR SUGGESTIONS" → "CLEAR_SUGGESTIONS()"
    fn convert_multiword_keywords(script: &str) -> String {
        use regex::Regex;

        // Known multi-word keywords with their conversion patterns
        // Format: (keyword_pattern, min_params, max_params, param_names)
        let multiword_patterns = vec![
            // USE family
            (r#"USE\s+WEBSITE"#, 1, 2, vec!["url", "refresh"]),
            (r#"USE\s+MODEL"#, 1, 1, vec!["model"]),
            (r#"USE\s+KB"#, 1, 1, vec!["kb_name"]),
            (r#"USE\s+TOOL"#, 1, 1, vec!["tool_path"]),

            // SET family
            (r#"SET\s+BOT\s+MEMORY"#, 2, 2, vec!["key", "value"]),
            (r#"SET\s+CONTEXT"#, 2, 2, vec!["key", "value"]),
            (r#"SET\s+USER"#, 1, 1, vec!["user_id"]),

            // GET family
            (r#"GET\s+BOT\s+MEMORY"#, 1, 1, vec!["key"]),

            // CLEAR family
            (r#"CLEAR\s+SUGGESTIONS"#, 0, 0, vec![]),
            (r#"CLEAR\s+TOOLS"#, 0, 0, vec![]),
            (r#"CLEAR\s+WEBSITES"#, 0, 0, vec![]),

            // ADD family - ADD_SUGGESTION_TOOL must come before ADD\s+SUGGESTION
            (r#"ADD_SUGGESTION_TOOL"#, 2, 2, vec!["tool", "text"]),
            (r#"ADD\s+SUGGESTION\s+TEXT"#, 2, 2, vec!["value", "text"]),
            (r#"ADD\s+SUGGESTION(?!\s*TEXT|\s*TOOL|_TOOL)"#, 2, 2, vec!["context", "text"]),
            (r#"ADD\s+MEMBER"#, 2, 2, vec!["name", "role"]),

            // CREATE family
            (r#"CREATE\s+TASK"#, 1, 1, vec!["task"]),
            (r#"CREATE\s+DRAFT"#, 4, 4, vec!["to", "subject", "body", "attachments"]),
            (r#"CREATE\s+SITE"#, 1, 1, vec!["site"]),

            // ON family
            (r#"ON\s+FORM\s+SUBMIT"#, 1, 1, vec!["form"]),
            (r#"ON\s+EMAIL"#, 1, 1, vec!["filter"]),
            (r#"ON\s+EVENT"#, 1, 1, vec!["event"]),

            // SEND family
            (r#"SEND\s+MAIL"#, 4, 4, vec!["to", "subject", "body", "attachments"]),

            // BOOK (calendar)
            (r#"BOOK"#, 1, 1, vec!["event"]),
        ];

        let mut result = String::new();

        for line in script.lines() {
            let trimmed = line.trim();
            let mut converted = false;

            // Skip lines that already use underscore-style custom syntax
            // These are registered directly with Rhai and should not be converted
            let trimmed_upper = trimmed.to_uppercase();
            if trimmed_upper.contains("ADD_SUGGESTION_TOOL") ||
               trimmed_upper.contains("ADD_SUGGESTION_TEXT") ||
               trimmed_upper.starts_with("ADD_SUGGESTION_") ||
               trimmed_upper.starts_with("ADD_MEMBER") ||
               (trimmed_upper.starts_with("USE_") && trimmed.contains('(')) {
                // Keep original line and add semicolon if needed
                result.push_str(line);
                if !trimmed.ends_with(';') && !trimmed.ends_with('{') && !trimmed.ends_with('}') {
                    result.push(';');
                }
                result.push('\n');
                continue;
            }

            // Try each pattern
            for (pattern, min_params, max_params, _param_names) in &multiword_patterns {
                // Build regex pattern: KEYWORD params...
                // Handle quoted strings and unquoted identifiers
                let regex_str = format!(
                    r#"(?i)^\s*{}\s+(.*?)(?:\s*)$"#,
                    pattern
                );

                if let Ok(re) = Regex::new(&regex_str) {
                    if let Some(caps) = re.captures(trimmed) {
                        if let Some(params_str) = caps.get(1) {
                            let params = Self::parse_parameters(params_str.as_str());
                            let param_count = params.len();

                            // Validate parameter count
                            if param_count >= *min_params && param_count <= *max_params {
                                // Convert keyword to underscores
                                let keyword = pattern.replace(r"\s+", "_");

                                // Build function call
                                let params_str = if params.is_empty() {
                                    String::new()
                                } else {
                                    params.join(", ")
                                };

                                result.push_str(&format!("{}({});", keyword, params_str));
                                result.push('\n');
                                converted = true;
                                break;
                            }
                        }
                    }
                }
            }

            // If not converted, keep original line
            if !converted {
                result.push_str(line);
                result.push('\n');
            }
        }

        result
    }

    /// Parse parameters from a keyword line
    /// Handles quoted strings, AS keyword, and comma-separated values
    fn parse_parameters(params_str: &str) -> Vec<String> {
        let mut params = Vec::new();
        let mut current = String::new();
        let mut in_quotes = false;
        let mut quote_char = '"';
        let mut chars = params_str.chars().peekable();

        while let Some(c) = chars.next() {
            match c {
                '"' | '\'' if !in_quotes => {
                    in_quotes = true;
                    quote_char = c;
                    current.push(c);
                }
                '"' | '\'' if in_quotes && c == quote_char => {
                    in_quotes = false;
                    current.push(c);
                }
                ' ' | '\t' if !in_quotes => {
                    // End of parameter if we have content
                    if !current.is_empty() {
                        params.push(current.trim().to_string());
                        current = String::new();
                    }
                }
                ',' if !in_quotes => {
                    // Comma separator
                    if !current.is_empty() {
                        params.push(current.trim().to_string());
                        current = String::new();
                    }
                }
                _ => {
                    current.push(c);
                }
            }
        }

        // Don't forget the last parameter
        if !current.is_empty() {
            params.push(current.trim().to_string());
        }

        params
    }

    fn normalize_word(word: &str, keywords: &[&str]) -> String {
        let upper = word.to_uppercase();

        if keywords.contains(&upper.as_str()) {
            upper
        } else if word
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            word.to_string()
        } else {
            word.to_lowercase()
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::time::Duration;

    // Test script constants from bottest/fixtures/scripts/mod.rs

    const GREETING_SCRIPT: &str = r#"
' Greeting Flow Script
' Simple greeting and response pattern

REM Initialize greeting
greeting$ = "Hello! Welcome to our service."
TALK greeting$

REM Wait for user response
HEAR userInput$

REM Check for specific keywords
IF INSTR(UCASE$(userInput$), "HELP") > 0 THEN
    TALK "I can help you with: Products, Support, or Billing. What would you like to know?"
ELSEIF INSTR(UCASE$(userInput$), "BYE") > 0 THEN
    TALK "Goodbye! Have a great day!"
    END
ELSE
    TALK "Thank you for your message. How can I assist you today?"
END IF
"#;

    const SIMPLE_ECHO_SCRIPT: &str = r#"
' Simple Echo Script
' Echoes back whatever the user says

TALK "Echo Bot: I will repeat everything you say. Type 'quit' to exit."

echo_loop:
HEAR input$

IF UCASE$(input$) = "QUIT" THEN
    TALK "Goodbye!"
    END
END IF

TALK "You said: " + input$
GOTO echo_loop
"#;

    const VARIABLES_SCRIPT: &str = r#"
' Variables and Expressions Script
' Demonstrates variable types and operations

REM String variables
firstName$ = "John"
lastName$ = "Doe"
fullName$ = firstName$ + " " + lastName$
TALK "Full name: " + fullName$

REM Numeric variables
price = 99.99
quantity = 3
subtotal = price * quantity
tax = subtotal * 0.08
total = subtotal + tax
TALK "Total: $" + STR$(total)
"#;

    fn get_script(name: &str) -> Option<&'static str> {
        match name {
            "greeting" => Some(GREETING_SCRIPT),
            "simple_echo" => Some(SIMPLE_ECHO_SCRIPT),
            "variables" => Some(VARIABLES_SCRIPT),
            _ => None,
        }
    }

    fn available_scripts() -> Vec<&'static str> {
        vec!["greeting", "simple_echo", "variables"]
    }

    fn all_scripts() -> HashMap<&'static str, &'static str> {
        let mut scripts = HashMap::new();
        for name in available_scripts() {
            if let Some(content) = get_script(name) {
                scripts.insert(name, content);
            }
        }
        scripts
    }

    // Runner types from bottest/bot/runner.rs

    #[derive(Debug, Clone)]
    pub struct BotRunnerConfig {
        pub working_dir: std::path::PathBuf,
        pub timeout: Duration,
        pub use_mocks: bool,
        pub env_vars: HashMap<String, String>,
        pub capture_logs: bool,
        log_level: LogLevel,
    }

    impl BotRunnerConfig {
        pub const fn log_level(&self) -> LogLevel {
            self.log_level
        }
    }

    impl Default for BotRunnerConfig {
        fn default() -> Self {
            Self {
                working_dir: std::env::temp_dir().join("bottest"),
                timeout: Duration::from_secs(30),
                use_mocks: true,
                env_vars: HashMap::new(),
                capture_logs: true,
                log_level: LogLevel::Info,
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum LogLevel {
        Trace,
        Debug,
        #[default]
        Info,
        Warn,
        Error,
    }

    #[derive(Debug, Default, Clone)]
    pub struct RunnerMetrics {
        pub total_requests: u64,
        pub successful_requests: u64,
        pub failed_requests: u64,
        pub total_latency_ms: u64,
        pub min_latency_ms: u64,
        pub max_latency_ms: u64,
        pub script_executions: u64,
        pub transfer_to_human_count: u64,
    }

    impl RunnerMetrics {
        pub const fn avg_latency_ms(&self) -> u64 {
            if self.total_requests > 0 {
                self.total_latency_ms / self.total_requests
            } else {
                0
            }
        }

        pub fn success_rate(&self) -> f64 {
            if self.total_requests > 0 {
                (self.successful_requests as f64 / self.total_requests as f64) * 100.0
            } else {
                0.0
            }
        }

        pub const fn min_latency(&self) -> u64 {
            self.min_latency_ms
        }

        pub const fn max_latency(&self) -> u64 {
            self.max_latency_ms
        }

        pub const fn latency_range(&self) -> u64 {
            self.max_latency_ms.saturating_sub(self.min_latency_ms)
        }
    }

    // Tests

    #[test]
    fn test_get_script() {
        assert!(get_script("greeting").is_some());
        assert!(get_script("simple_echo").is_some());
        assert!(get_script("nonexistent").is_none());
    }

    #[test]
    fn test_available_scripts() {
        let scripts = available_scripts();
        assert!(!scripts.is_empty());
        assert!(scripts.contains(&"greeting"));
    }

    #[test]
    fn test_all_scripts() {
        let scripts = all_scripts();
        assert_eq!(scripts.len(), available_scripts().len());
    }

    #[test]
    fn test_greeting_script_content() {
        let script = get_script("greeting").unwrap();
        assert!(script.contains("TALK"));
        assert!(script.contains("HEAR"));
        assert!(script.contains("greeting"));
    }

    #[test]
    fn test_simple_echo_script_content() {
        let script = get_script("simple_echo").unwrap();
        assert!(script.contains("HEAR"));
        assert!(script.contains("TALK"));
        assert!(script.contains("GOTO"));
    }

    #[test]
    fn test_variables_script_content() {
        let script = get_script("variables").unwrap();
        assert!(script.contains("firstName$"));
        assert!(script.contains("price"));
        assert!(script.contains("STR$"));
    }

    #[test]
    fn test_bot_runner_config_default() {
        let config = BotRunnerConfig::default();
        assert_eq!(config.timeout, Duration::from_secs(30));
        assert!(config.use_mocks);
        assert!(config.capture_logs);
    }

    #[test]
    fn test_runner_metrics_avg_latency() {
        let metrics = RunnerMetrics {
            total_requests: 10,
            total_latency_ms: 1000,
            ..RunnerMetrics::default()
        };

        assert_eq!(metrics.avg_latency_ms(), 100);
    }

    #[test]
    fn test_runner_metrics_success_rate() {
        let metrics = RunnerMetrics {
            total_requests: 100,
            successful_requests: 95,
            ..RunnerMetrics::default()
        };

        assert!((metrics.success_rate() - 95.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_runner_metrics_zero_requests() {
        let metrics = RunnerMetrics::default();
        assert_eq!(metrics.avg_latency_ms(), 0);
        assert!(metrics.success_rate().abs() < f64::EPSILON);
    }

    #[test]
    fn test_log_level_default() {
        let level = LogLevel::default();
        assert_eq!(level, LogLevel::Info);
    }

    #[test]
    fn test_runner_config_env_vars() {
        let mut env_vars = HashMap::new();
        env_vars.insert("API_KEY".to_string(), "test123".to_string());
        env_vars.insert("DEBUG".to_string(), "true".to_string());

        let config = BotRunnerConfig {
            env_vars,
            ..BotRunnerConfig::default()
        };

        assert_eq!(config.env_vars.get("API_KEY"), Some(&"test123".to_string()));
        assert_eq!(config.env_vars.get("DEBUG"), Some(&"true".to_string()));
    }

    #[test]
    fn test_runner_config_timeout() {
        let config = BotRunnerConfig {
            timeout: Duration::from_secs(60),
            ..BotRunnerConfig::default()
        };

        assert_eq!(config.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_metrics_tracking() {
        let metrics = RunnerMetrics {
            total_requests: 50,
            successful_requests: 45,
            failed_requests: 5,
            total_latency_ms: 5000,
            min_latency_ms: 10,
            max_latency_ms: 500,
            ..RunnerMetrics::default()
        };

        assert_eq!(metrics.avg_latency_ms(), 100);
        assert!((metrics.success_rate() - 90.0).abs() < f64::EPSILON);
        assert_eq!(
            metrics.total_requests,
            metrics.successful_requests + metrics.failed_requests
        );
        assert_eq!(metrics.min_latency(), 10);
        assert_eq!(metrics.max_latency(), 500);
        assert_eq!(metrics.latency_range(), 490);
    }

    #[test]
    fn test_script_execution_tracking() {
        let metrics = RunnerMetrics {
            script_executions: 25,
            transfer_to_human_count: 3,
            ..RunnerMetrics::default()
        };

        assert_eq!(metrics.script_executions, 25);
        assert_eq!(metrics.transfer_to_human_count, 3);
    }

    #[test]
    fn test_log_level_accessor() {
        let config = BotRunnerConfig::default();
        assert_eq!(config.log_level(), LogLevel::Info);
    }

    #[test]
    fn test_log_levels() {
        assert!(matches!(LogLevel::Trace, LogLevel::Trace));
        assert!(matches!(LogLevel::Debug, LogLevel::Debug));
        assert!(matches!(LogLevel::Info, LogLevel::Info));
        assert!(matches!(LogLevel::Warn, LogLevel::Warn));
        assert!(matches!(LogLevel::Error, LogLevel::Error));
    }

    #[test]
    fn test_script_contains_basic_keywords() {
        for name in available_scripts() {
            if let Some(script) = get_script(name) {
                // All scripts should have some form of output
                let has_output = script.contains("TALK") || script.contains("PRINT");
                assert!(has_output, "Script {} should have output keyword", name);
            }
        }
    }

    #[test]
    fn test_runner_config_working_dir() {
        let config = BotRunnerConfig::default();
        assert!(config.working_dir.to_str().unwrap_or_default().contains("bottest"));
    }
}
