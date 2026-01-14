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

        remember_keyword(state.clone(), user.clone(), &mut engine);
        book_keyword(state.clone(), user.clone(), &mut engine);
        send_mail_keyword(state.clone(), user.clone(), &mut engine);
        save_from_unstructured_keyword(state.clone(), user.clone(), &mut engine);
        create_task_keyword(state.clone(), user.clone(), &mut engine);
        add_member_keyword(state.clone(), user.clone(), &mut engine);

        register_bot_keywords(&state, &user, &mut engine);

        register_model_routing_keywords(state.clone(), user.clone(), &mut engine);

        keywords::universal_messaging::register_universal_messaging(
            state.clone(),
            user.clone(),
            &mut engine,
        );

        register_multimodal_keywords(state.clone(), user.clone(), &mut engine);

        register_string_functions(state.clone(), user.clone(), &mut engine);

        switch_keyword(&state, user.clone(), &mut engine);

        register_http_operations(state.clone(), user.clone(), &mut engine);

        register_data_operations(state.clone(), user.clone(), &mut engine);

        register_file_operations(state.clone(), user.clone(), &mut engine);

        webhook_keyword(&state, user.clone(), &mut engine);

        register_social_media_keywords(state.clone(), user.clone(), &mut engine);

        register_send_template_keywords(state.clone(), user.clone(), &mut engine);

        on_form_submit_keyword(state.clone(), user.clone(), &mut engine);

        register_lead_scoring_keywords(state.clone(), user.clone(), &mut engine);

        register_transfer_to_human_keyword(state.clone(), user.clone(), &mut engine);

        register_ai_tools_keywords(state.clone(), user.clone(), &mut engine);

        register_web_data_keywords(state.clone(), user.clone(), &mut engine);

        register_sms_keywords(state.clone(), user.clone(), &mut engine);

        register_core_functions(state.clone(), user, &mut engine);

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
    pub fn run(&mut self, ast: &rhai::AST) -> Result<Dynamic, Box<EvalAltResult>> {
        self.engine.eval_ast_with_scope(&mut self.scope, ast)
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
        ];

        let _identifier_re = Regex::new(r"([a-zA-Z_][a-zA-Z0-9_]*)").expect("valid regex");

        for line in script.lines() {
            let trimmed = line.trim();

            if trimmed.starts_with("REM") || trimmed.starts_with('\'') || trimmed.starts_with("//")
            {
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
