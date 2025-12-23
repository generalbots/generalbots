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

        ScriptService { engine, scope }
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

        let script = preprocess_switch(script);


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


        let _identifier_re = Regex::new(r"([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();

        for line in script.lines() {
            let trimmed = line.trim();


            if trimmed.starts_with("REM") || trimmed.starts_with("'") || trimmed.starts_with("//") {
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
