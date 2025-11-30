use crate::basic::keywords::add_suggestion::clear_suggestions_keyword;
use crate::basic::keywords::set_user::set_user_keyword;
use crate::basic::keywords::string_functions::register_string_functions;
use crate::basic::keywords::switch_case::switch_keyword;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::info;
use rhai::{Dynamic, Engine, EvalAltResult};
use std::sync::Arc;
pub mod compiler;
pub mod keywords;
use self::keywords::add_member::add_member_keyword;
use self::keywords::add_suggestion::add_suggestion_keyword;
use self::keywords::book::book_keyword;
use self::keywords::bot_memory::{get_bot_memory_keyword, set_bot_memory_keyword};
use self::keywords::clear_kb::register_clear_kb_keyword;
use self::keywords::clear_tools::clear_tools_keyword;
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
use self::keywords::multimodal::register_multimodal_keywords;
use self::keywords::remember::remember_keyword;
use self::keywords::save_from_unstructured::save_from_unstructured_keyword;
use self::keywords::send_mail::send_mail_keyword;
use self::keywords::switch_case::preprocess_switch;
use self::keywords::use_kb::register_use_kb_keyword;
use self::keywords::use_tool::use_tool_keyword;
use self::keywords::use_website::{clear_websites_keyword, use_website_keyword};
use self::keywords::webhook::webhook_keyword;

use self::keywords::llm_keyword::llm_keyword;
use self::keywords::on::on_keyword;
use self::keywords::print::print_keyword;
use self::keywords::set::set_keyword;
use self::keywords::set_context::set_context_keyword;

use self::keywords::wait::wait_keyword;
#[derive(Debug)]
pub struct ScriptService {
    pub engine: Engine,
}
impl ScriptService {
    #[must_use]
    pub fn new(state: Arc<AppState>, user: UserSession) -> Self {
        let mut engine = Engine::new();
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

        ScriptService { engine }
    }
    fn preprocess_basic_script(&self, script: &str) -> String {
        // First, preprocess SWITCH/CASE blocks
        let script = preprocess_switch(script);

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
                "DELETE_HTTP",
                "SET_HEADER",
                "CLEAR_HEADERS",
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
                "GROUP_BY",
                // File Operations
                "READ",
                "WRITE",
                "DELETE_FILE",
                "COPY",
                "MOVE",
                "LIST",
                "COMPRESS",
                "EXTRACT",
                "UPLOAD",
                "DOWNLOAD",
                "GENERATE_PDF",
                "MERGE_PDF",
                // Webhook
                "WEBHOOK",
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
    pub fn run(&self, ast: &rhai::AST) -> Result<Dynamic, Box<EvalAltResult>> {
        self.engine.eval_ast(ast)
    }
}
