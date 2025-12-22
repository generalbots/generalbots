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

/// Helper struct for loading param config from database
#[derive(QueryableByName)]
struct ParamConfigRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    config_key: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    config_value: String,
}
use self::keywords::add_bot::register_bot_keywords;
use self::keywords::add_member::add_member_keyword;
use self::keywords::add_suggestion::add_suggestion_keyword;
use self::keywords::ai_tools::register_ai_tools_keywords;
use self::keywords::book::book_keyword;
use self::keywords::bot_memory::{get_bot_memory_keyword, set_bot_memory_keyword};
use self::keywords::clear_kb::register_clear_kb_keyword;
use self::keywords::clear_tools::clear_tools_keyword;
use self::keywords::core_functions::register_core_functions;
use self::keywords::create_draft::create_draft_keyword;
use self::keywords::create_site::create_site_keyword;
use self::keywords::create_task::create_task_keyword;
use self::keywords::data_operations::register_data_operations;
use self::keywords::file_operations::register_file_operations;
use self::keywords::find::find_keyword;
use self::keywords::first::first_keyword;
use self::keywords::for_next::for_keyword;
use self::keywords::format::format_keyword;
use self::keywords::get::get_keyword;
use self::keywords::hear_talk::{hear_keyword, talk_keyword};
use self::keywords::http_operations::register_http_operations;
use self::keywords::last::last_keyword;
use self::keywords::lead_scoring::register_lead_scoring_keywords;
use self::keywords::model_routing::register_model_routing_keywords;
use self::keywords::multimodal::register_multimodal_keywords;
use self::keywords::on_form_submit::on_form_submit_keyword;
use self::keywords::remember::remember_keyword;
use self::keywords::save_from_unstructured::save_from_unstructured_keyword;
use self::keywords::send_mail::send_mail_keyword;
use self::keywords::send_template::register_send_template_keywords;
use self::keywords::sms::register_sms_keywords;
use self::keywords::social_media::register_social_media_keywords;
use self::keywords::switch_case::preprocess_switch;
use self::keywords::transfer_to_human::register_transfer_to_human_keyword;
use self::keywords::use_kb::register_use_kb_keyword;
use self::keywords::use_tool::use_tool_keyword;
use self::keywords::use_website::{clear_websites_keyword, use_website_keyword};
use self::keywords::web_data::register_web_data_keywords;
use self::keywords::webhook::webhook_keyword;

use self::keywords::llm_keyword::llm_keyword;
use self::keywords::on::on_keyword;
use self::keywords::on_change::on_change_keyword;
use self::keywords::on_email::on_email_keyword;
use self::keywords::print::print_keyword;
use self::keywords::set::set_keyword;
use self::keywords::set_context::set_context_keyword;

use self::keywords::wait::wait_keyword;
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

        // Core keywords
        create_draft_keyword(&state, user.clone(), &mut engine);
        set_bot_memory_keyword(state.clone(), user.clone(), &mut engine);
        get_bot_memory_keyword(state.clone(), user.clone(), &mut engine);
        create_site_keyword(&state, user.clone(), &mut engine);
        find_keyword(&state, user.clone(), &mut engine);
        for_keyword(&state, user.clone(), &mut engine);
        let _ = register_use_kb_keyword(&mut engine, state.clone(), Arc::new(user.clone()));
        let _ = register_clear_kb_keyword(&mut engine, state.clone(), Arc::new(user.clone()));
        first_keyword(&mut engine);
        last_keyword(&mut engine);
        format_keyword(&mut engine);
        llm_keyword(state.clone(), user.clone(), &mut engine);
        get_keyword(state.clone(), user.clone(), &mut engine);
        set_keyword(&state, user.clone(), &mut engine);
        wait_keyword(&state, user.clone(), &mut engine);
        print_keyword(&state, user.clone(), &mut engine);
        on_keyword(&state, user.clone(), &mut engine);
        on_email_keyword(&state, user.clone(), &mut engine);
        on_change_keyword(&state, user.clone(), &mut engine);
        hear_keyword(state.clone(), user.clone(), &mut engine);
        talk_keyword(state.clone(), user.clone(), &mut engine);
        set_context_keyword(state.clone(), user.clone(), &mut engine);
        set_user_keyword(state.clone(), user.clone(), &mut engine);
        clear_suggestions_keyword(state.clone(), user.clone(), &mut engine);

        use_tool_keyword(state.clone(), user.clone(), &mut engine);
        clear_tools_keyword(state.clone(), user.clone(), &mut engine);

        use_website_keyword(state.clone(), user.clone(), &mut engine);
        clear_websites_keyword(state.clone(), user.clone(), &mut engine);
        add_suggestion_keyword(state.clone(), user.clone(), &mut engine);

        // Register the 6 new power keywords
        remember_keyword(state.clone(), user.clone(), &mut engine);
        book_keyword(state.clone(), user.clone(), &mut engine);
        send_mail_keyword(state.clone(), user.clone(), &mut engine);
        save_from_unstructured_keyword(state.clone(), user.clone(), &mut engine);
        create_task_keyword(state.clone(), user.clone(), &mut engine);
        add_member_keyword(state.clone(), user.clone(), &mut engine);

        // Register dynamic bot management keywords (ADD BOT, REMOVE BOT)
        register_bot_keywords(&state, &user, &mut engine);

        // Register model routing keywords (USE MODEL, SET MODEL ROUTING, etc.)
        register_model_routing_keywords(state.clone(), user.clone(), &mut engine);

        // Register universal messaging keywords
        keywords::universal_messaging::register_universal_messaging(
            state.clone(),
            user.clone(),
            &mut engine,
        );

        // Register multimodal keywords (IMAGE, VIDEO, AUDIO, SEE)
        // These connect to botmodels for image/video/audio generation and vision/captioning
        register_multimodal_keywords(state.clone(), user.clone(), &mut engine);

        // Register string functions (INSTR, IS_NUMERIC, LEN, LEFT, RIGHT, MID, etc.)
        register_string_functions(state.clone(), user.clone(), &mut engine);

        // Register SWITCH/CASE helper functions
        switch_keyword(&state, user.clone(), &mut engine);

        // ========================================================================
        // NEW KEYWORDS for office.gbai - Compete with n8n
        // ========================================================================

        // HTTP Operations: POST, PUT, PATCH, DELETE_HTTP, SET_HEADER, GRAPHQL, SOAP
        register_http_operations(state.clone(), user.clone(), &mut engine);

        // Data Operations: SAVE, INSERT, UPDATE, DELETE, MERGE, FILL, MAP, FILTER,
        // AGGREGATE, JOIN, PIVOT, GROUP_BY
        register_data_operations(state.clone(), user.clone(), &mut engine);

        // File Operations: READ, WRITE, DELETE_FILE, COPY, MOVE, LIST,
        // COMPRESS, EXTRACT, UPLOAD, DOWNLOAD, GENERATE_PDF, MERGE_PDF
        register_file_operations(state.clone(), user.clone(), &mut engine);

        // Webhook keyword for event-driven automation
        webhook_keyword(&state, user.clone(), &mut engine);

        // ========================================================================
        // NEW KEYWORDS: Social Media, Marketing, CRM
        // ========================================================================

        // Social Media: POST TO (Instagram, Facebook, LinkedIn, Twitter)
        // GET METRICS, scheduled posting
        register_social_media_keywords(state.clone(), user.clone(), &mut engine);

        // SEND TEMPLATE: Multi-channel templated messaging (email, WhatsApp, SMS)
        register_send_template_keywords(state.clone(), user.clone(), &mut engine);

        // ON FORM SUBMIT: Webhook-based form handling for landing pages
        on_form_submit_keyword(state.clone(), user.clone(), &mut engine);

        // Lead Scoring: SCORE LEAD, GET LEAD SCORE, QUALIFY LEAD, AI SCORE LEAD
        register_lead_scoring_keywords(state.clone(), user.clone(), &mut engine);

        // ========================================================================
        // CRM & HUMAN HANDOFF
        // ========================================================================

        // TRANSFER TO HUMAN: Bot-to-human handoff for hybrid support workflows
        // Supports transfer by name/alias, department, priority, and context
        register_transfer_to_human_keyword(state.clone(), user.clone(), &mut engine);

        // ========================================================================
        // AI-POWERED TOOLS: TRANSLATE, OCR, SENTIMENT, CLASSIFY
        // ========================================================================
        register_ai_tools_keywords(state.clone(), user.clone(), &mut engine);

        // ========================================================================
        // WEB DATA: RSS, SCRAPE, SCRAPE_ALL, SCRAPE_TABLE, SCRAPE_LINKS, SCRAPE_IMAGES
        // ========================================================================
        register_web_data_keywords(state.clone(), user.clone(), &mut engine);

        // ========================================================================
        // SMS: SEND_SMS phone, message - Send SMS via Twilio, AWS SNS, Vonage, etc.
        // ========================================================================
        register_sms_keywords(state.clone(), user.clone(), &mut engine);

        // ========================================================================
        // CORE BASIC FUNCTIONS: Math, Date/Time, Validation, Arrays, Error Handling
        // ========================================================================

        // Math: ABS, ROUND, INT, MAX, MIN, MOD, RANDOM, SGN, SQR, LOG, EXP, SIN, COS, TAN
        // Date/Time: NOW, TODAY, YEAR, MONTH, DAY, HOUR, MINUTE, SECOND, DATEADD, DATEDIFF
        // Validation: VAL, STR, ISNULL, ISEMPTY, ISDATE, TYPEOF
        // Arrays: ARRAY, UBOUND, SORT, UNIQUE, CONTAINS, PUSH, POP, REVERSE, SLICE, BATCH, CHUNK
        // Error Handling: THROW, ERROR, IS_ERROR, ASSERT
        register_core_functions(state.clone(), user, &mut engine);

        ScriptService { engine, scope }
    }

    /// Inject param-* configuration variables from config.csv into the script scope
    /// Variables are made available without the "param-" prefix and normalized to lowercase
    pub fn inject_config_variables(&mut self, config_vars: HashMap<String, String>) {
        for (key, value) in config_vars {
            // Remove "param-" prefix if present and normalize to lowercase
            let var_name = if key.starts_with("param-") {
                key.strip_prefix("param-").unwrap_or(&key).to_lowercase()
            } else {
                key.to_lowercase()
            };

            // Try to parse as number, otherwise use as string
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

    /// Load and inject param-* variables from bot configuration
    pub fn load_bot_config_params(&mut self, state: &AppState, bot_id: uuid::Uuid) {
        if let Ok(mut conn) = state.conn.get() {
            // Query all config entries for this bot that start with "param-"
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
        // First, preprocess SWITCH/CASE blocks
        let script = preprocess_switch(script);

        // Make variables case-insensitive by normalizing to lowercase
        let script = Self::normalize_variables_to_lowercase(&script);

        let mut result = String::new();
        let mut for_stack: Vec<usize> = Vec::new();
        let mut current_indent = 0;
        for line in script.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with("'") {
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
                    if (current_indent - 4) != expected_indent {
                        panic!("NEXT without matching FOR EACH");
                    }
                    current_indent = current_indent - 4;
                    result.push_str(&" ".repeat(current_indent));
                    result.push_str("}\n");
                    result.push_str(&" ".repeat(current_indent));
                    result.push_str(trimmed);
                    result.push(';');
                    result.push('\n');
                    continue;
                }
                panic!("NEXT without matching FOR EACH");
            }
            if trimmed == "EXIT FOR" {
                result.push_str(&" ".repeat(current_indent));
                result.push_str(trimmed);
                result.push('\n');
                continue;
            }
            result.push_str(&" ".repeat(current_indent));
            let basic_commands = [
                // Core commands
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
                // HTTP Operations
                "POST",
                "PUT",
                "PATCH",
                "DELETE",
                "SET HEADER",
                "CLEAR HEADERS",
                "GRAPHQL",
                "SOAP",
                // Data Operations
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
                // File Operations
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
                // Webhook
                "WEBHOOK",
                // Social Media
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
                // Template & Messaging
                "SEND MAIL",
                "SEND TEMPLATE",
                "CREATE TEMPLATE",
                "GET TEMPLATE",
                // Error Handling
                "ON ERROR RESUME NEXT",
                "ON ERROR GOTO",
                "CLEAR ERROR",
                "ERROR MESSAGE",
                // Form Handling
                "ON FORM SUBMIT",
                // Lead Scoring
                "SCORE LEAD",
                "GET LEAD SCORE",
                "QUALIFY LEAD",
                "UPDATE LEAD SCORE",
                "AI SCORE LEAD",
                // Math Functions
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
                // Date/Time Functions
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
                // Validation Functions
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
                // Array Functions
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
                // Error Handling
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
            if is_basic_command || !for_stack.is_empty() || is_control_flow {
                result.push_str(trimmed);
                result.push(';');
            } else {
                result.push_str(trimmed);
                if !trimmed.ends_with(';') && !trimmed.ends_with('{') && !trimmed.ends_with('}') {
                    result.push(';');
                }
            }
            result.push('\n');
        }
        if !for_stack.is_empty() {
            panic!("Unclosed FOR EACH loop");
        }
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
    pub fn run(&mut self, ast: &rhai::AST) -> Result<Dynamic, Box<EvalAltResult>> {
        self.engine.eval_ast_with_scope(&mut self.scope, ast)
    }

    /// Normalize variable names to lowercase for case-insensitive BASIC semantics
    /// This transforms variable assignments and references to use lowercase names
    /// while preserving string literals, keywords, and comments
    fn normalize_variables_to_lowercase(script: &str) -> String {
        use regex::Regex;

        let mut result = String::new();

        // Keywords that should remain uppercase (BASIC commands)
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
        ];

        // Regex to match identifiers (variable names)
        let _identifier_re = Regex::new(r"([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();

        for line in script.lines() {
            let trimmed = line.trim();

            // Skip comments entirely
            if trimmed.starts_with("REM") || trimmed.starts_with("'") || trimmed.starts_with("//") {
                result.push_str(line);
                result.push('\n');
                continue;
            }

            // Process line character by character to handle strings properly
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
                        // Handle escape sequences
                        if let Some(&next) = chars.peek() {
                            processed_line.push(next);
                            chars.next();
                        }
                    }
                } else if c == '"' || c == '\'' {
                    // Flush current word before string
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
                    // Flush current word
                    if !current_word.is_empty() {
                        processed_line.push_str(&Self::normalize_word(&current_word, &keywords));
                        current_word.clear();
                    }
                    processed_line.push(c);
                }
            }

            // Flush any remaining word
            if !current_word.is_empty() {
                processed_line.push_str(&Self::normalize_word(&current_word, &keywords));
            }

            result.push_str(&processed_line);
            result.push('\n');
        }

        result
    }

    /// Normalize a single word - convert to lowercase if it's a variable (not a keyword)
    fn normalize_word(word: &str, keywords: &[&str]) -> String {
        let upper = word.to_uppercase();

        // Check if it's a keyword (case-insensitive)
        if keywords.contains(&upper.as_str()) {
            // Return the keyword in uppercase for consistency
            upper
        } else if word
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
        {
            // It's a number, keep as-is
            word.to_string()
        } else {
            // It's a variable - normalize to lowercase
            word.to_lowercase()
        }
    }
}
