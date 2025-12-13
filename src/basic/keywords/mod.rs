pub mod a2a_protocol;
pub mod add_bot;
pub mod add_member;
pub mod add_suggestion;
pub mod agent_reflection;
pub mod ai_tools;
pub mod api_tool_generator;
pub mod arrays;
pub mod auto_task;
pub mod autotask_api;
pub mod book;
pub mod bot_memory;
pub mod clear_kb;
pub mod clear_tools;
pub mod code_sandbox;
pub mod core_functions;
pub mod create_draft;
pub mod create_site;
pub mod create_task;
pub mod crm;
pub mod data_operations;
pub mod datetime;
pub mod episodic_memory;
pub mod errors;
pub mod file_operations;
pub mod find;
pub mod first;
pub mod for_next;
pub mod format;
pub mod get;
pub mod hear_talk;
pub mod http_operations;
pub mod human_approval;
pub mod import_export;
pub mod intent_compiler;
pub mod kb_statistics;
pub mod knowledge_graph;
pub mod last;
pub mod lead_scoring;
pub mod llm_keyword;
pub mod llm_macros;
pub mod math;
pub mod mcp_client;
pub mod mcp_directory;
pub mod messaging;
pub mod model_routing;
pub mod multimodal;
pub mod on;
pub mod on_form_submit;
pub mod play;
pub mod print;
pub mod procedures;
pub mod qrcode;
pub mod remember;
pub mod safety_layer;
pub mod save_from_unstructured;
pub mod send_mail;
pub mod send_template;
pub mod set;
pub mod set_context;
pub mod set_schedule;
pub mod set_user;
pub mod sms;
pub mod social;
pub mod social_media;
pub mod string_functions;
pub mod switch_case;
pub mod table_definition;
pub mod transfer_to_human;
pub mod universal_messaging;
pub mod use_kb;
pub mod use_tool;
pub mod use_website;
pub mod user_memory;
pub mod validation;
pub mod wait;
pub mod weather;
pub mod web_data;
pub mod webhook;

// Re-export key types for convenience
pub use auto_task::{AutoTask, AutoTaskStatus, ExecutionMode, TaskPriority};
pub use intent_compiler::{CompiledIntent, ExecutionPlan, IntentCompiler, PlanStep};
pub use mcp_client::{McpClient, McpRequest, McpResponse, McpServer, McpTool};
pub use mcp_directory::{McpDirectoryScanResult, McpDirectoryScanner, McpServerConfig};
pub use safety_layer::{AuditEntry, ConstraintCheckResult, SafetyLayer, SimulationResult};

// Re-export API handlers for route configuration
pub use autotask_api::{
    cancel_task_handler, compile_intent_handler, execute_plan_handler, get_approvals_handler,
    get_decisions_handler, get_stats_handler, list_tasks_handler, pause_task_handler,
    resume_task_handler, simulate_plan_handler, simulate_task_handler, submit_approval_handler,
    submit_decision_handler,
};

/// Configure Auto Task API routes
pub fn configure_autotask_routes() -> axum::Router<std::sync::Arc<crate::shared::state::AppState>> {
    use axum::routing::{get, post};

    axum::Router::new()
        // Intent compilation
        .route("/api/autotask/compile", post(compile_intent_handler))
        // Plan execution
        .route("/api/autotask/execute", post(execute_plan_handler))
        .route(
            "/api/autotask/simulate/:plan_id",
            post(simulate_plan_handler),
        )
        // Task listing and stats
        .route("/api/autotask/list", get(list_tasks_handler))
        .route("/api/autotask/stats", get(get_stats_handler))
        // Task actions
        .route("/api/autotask/:task_id/pause", post(pause_task_handler))
        .route("/api/autotask/:task_id/resume", post(resume_task_handler))
        .route("/api/autotask/:task_id/cancel", post(cancel_task_handler))
        .route(
            "/api/autotask/:task_id/simulate",
            post(simulate_task_handler),
        )
        // Decisions
        .route(
            "/api/autotask/:task_id/decisions",
            get(get_decisions_handler),
        )
        .route(
            "/api/autotask/:task_id/decide",
            post(submit_decision_handler),
        )
        // Approvals
        .route(
            "/api/autotask/:task_id/approvals",
            get(get_approvals_handler),
        )
        .route(
            "/api/autotask/:task_id/approve",
            post(submit_approval_handler),
        )
}

/// List of all available BASIC keywords for the Intent Compiler
pub fn get_all_keywords() -> Vec<String> {
    vec![
        // Bot/Multi-Agent
        "ADD BOT".to_string(),
        "BOT REFLECTION".to_string(),
        "BROADCAST TO BOTS".to_string(),
        "DELEGATE TO BOT".to_string(),
        "TRANSFER CONVERSATION".to_string(),
        // Communication
        "ADD MEMBER".to_string(),
        "CREATE DRAFT".to_string(),
        "SEND MAIL".to_string(),
        "SEND TEMPLATE".to_string(),
        "SMS".to_string(),
        // UI
        "ADD SUGGESTION".to_string(),
        "CLEAR SUGGESTIONS".to_string(),
        // Tools
        "ADD TOOL".to_string(),
        "CLEAR TOOLS".to_string(),
        "CREATE SITE".to_string(),
        "CREATE TASK".to_string(),
        "USE TOOL".to_string(),
        // Data Operations
        "AGGREGATE".to_string(),
        "DELETE".to_string(),
        "FILL".to_string(),
        "FILTER".to_string(),
        "FIND".to_string(),
        "FIRST".to_string(),
        "GROUP BY".to_string(),
        "INSERT".to_string(),
        "JOIN".to_string(),
        "LAST".to_string(),
        "MAP".to_string(),
        "MERGE".to_string(),
        "PIVOT".to_string(),
        "SAVE".to_string(),
        "SAVE FROM UNSTRUCTURED".to_string(),
        "UPDATE".to_string(),
        // Files
        "COMPRESS".to_string(),
        "COPY".to_string(),
        "DELETE FILE".to_string(),
        "DOWNLOAD".to_string(),
        "EXTRACT".to_string(),
        "GENERATE PDF".to_string(),
        "LIST".to_string(),
        "MERGE PDF".to_string(),
        "MOVE".to_string(),
        "READ".to_string(),
        "UPLOAD".to_string(),
        "WRITE".to_string(),
        // HTTP
        "CLEAR HEADERS".to_string(),
        "DELETE HTTP".to_string(),
        "GET".to_string(),
        "GRAPHQL".to_string(),
        "PATCH".to_string(),
        "POST".to_string(),
        "PUT".to_string(),
        "SET HEADER".to_string(),
        "SOAP".to_string(),
        // Control Flow
        "EXIT FOR".to_string(),
        "FOR EACH".to_string(),
        "IF".to_string(),
        "SWITCH".to_string(),
        "WAIT".to_string(),
        "WHILE".to_string(),
        // Variables
        "GET".to_string(),
        "SET".to_string(),
        // Memory
        "GET BOT MEMORY".to_string(),
        "GET USER MEMORY".to_string(),
        "REMEMBER".to_string(),
        "SET BOT MEMORY".to_string(),
        "SET CONTEXT".to_string(),
        "SET USER FACT".to_string(),
        "SET USER MEMORY".to_string(),
        "USER FACTS".to_string(),
        // Knowledge
        "CLEAR KB".to_string(),
        "USE KB".to_string(),
        "USE WEBSITE".to_string(),
        // AI/LLM
        "LLM".to_string(),
        "SET CONTEXT".to_string(),
        "USE MODEL".to_string(),
        // Code Execution
        "RUN BASH".to_string(),
        "RUN JAVASCRIPT".to_string(),
        "RUN PYTHON".to_string(),
        // Dialog
        "HEAR".to_string(),
        "TALK".to_string(),
        // Events
        "ON".to_string(),
        "SET SCHEDULE".to_string(),
        "WEBHOOK".to_string(),
        // Session
        "SET USER".to_string(),
        // Special
        "BOOK".to_string(),
        "WEATHER".to_string(),
        // Debug
        "PRINT".to_string(),
        // String Functions
        "FORMAT".to_string(),
        "INSTR".to_string(),
        "IS NUMERIC".to_string(),
        // Safety & Approval (NEW)
        "REQUIRE APPROVAL".to_string(),
        "SIMULATE IMPACT".to_string(),
        "CHECK CONSTRAINTS".to_string(),
        "AUDIT LOG".to_string(),
        // Auto Task (NEW)
        "PLAN START".to_string(),
        "PLAN END".to_string(),
        "STEP".to_string(),
        "AUTO TASK".to_string(),
        // MCP Integration (NEW)
        "USE MCP".to_string(),
        "MCP LIST TOOLS".to_string(),
        "MCP INVOKE".to_string(),
        // Decision Framework (NEW)
        "OPTION A OR B".to_string(),
        "DECIDE".to_string(),
        "ESCALATE".to_string(),
    ]
}

/// Keyword categories for documentation and UI
pub fn get_keyword_categories() -> std::collections::HashMap<String, Vec<String>> {
    let mut categories = std::collections::HashMap::new();

    categories.insert(
        "Multi-Agent".to_string(),
        vec![
            "ADD BOT".to_string(),
            "BOT REFLECTION".to_string(),
            "BROADCAST TO BOTS".to_string(),
            "DELEGATE TO BOT".to_string(),
            "TRANSFER CONVERSATION".to_string(),
        ],
    );

    categories.insert(
        "Communication".to_string(),
        vec![
            "ADD MEMBER".to_string(),
            "CREATE DRAFT".to_string(),
            "SEND MAIL".to_string(),
            "SEND TEMPLATE".to_string(),
            "SMS".to_string(),
        ],
    );

    categories.insert(
        "Data".to_string(),
        vec![
            "AGGREGATE".to_string(),
            "DELETE".to_string(),
            "FILL".to_string(),
            "FILTER".to_string(),
            "FIND".to_string(),
            "FIRST".to_string(),
            "GROUP BY".to_string(),
            "INSERT".to_string(),
            "JOIN".to_string(),
            "LAST".to_string(),
            "MAP".to_string(),
            "MERGE".to_string(),
            "PIVOT".to_string(),
            "SAVE".to_string(),
            "UPDATE".to_string(),
        ],
    );

    categories.insert(
        "HTTP".to_string(),
        vec![
            "GET".to_string(),
            "POST".to_string(),
            "PUT".to_string(),
            "PATCH".to_string(),
            "DELETE HTTP".to_string(),
            "GRAPHQL".to_string(),
            "SOAP".to_string(),
            "SET HEADER".to_string(),
            "CLEAR HEADERS".to_string(),
        ],
    );

    categories.insert(
        "AI".to_string(),
        vec![
            "LLM".to_string(),
            "SET CONTEXT".to_string(),
            "USE MODEL".to_string(),
        ],
    );

    categories.insert(
        "Code Execution".to_string(),
        vec![
            "RUN PYTHON".to_string(),
            "RUN JAVASCRIPT".to_string(),
            "RUN BASH".to_string(),
        ],
    );

    categories.insert(
        "Safety".to_string(),
        vec![
            "REQUIRE APPROVAL".to_string(),
            "SIMULATE IMPACT".to_string(),
            "CHECK CONSTRAINTS".to_string(),
            "AUDIT LOG".to_string(),
        ],
    );

    categories.insert(
        "MCP".to_string(),
        vec![
            "USE MCP".to_string(),
            "MCP LIST TOOLS".to_string(),
            "MCP INVOKE".to_string(),
        ],
    );

    categories.insert(
        "Auto Task".to_string(),
        vec![
            "PLAN START".to_string(),
            "PLAN END".to_string(),
            "STEP".to_string(),
            "AUTO TASK".to_string(),
            "OPTION A OR B".to_string(),
            "DECIDE".to_string(),
            "ESCALATE".to_string(),
        ],
    );

    categories
}
