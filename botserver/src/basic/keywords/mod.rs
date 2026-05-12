// ===== RE-EXPORTED FROM botbasic_core =====
pub use botbasic_core::keywords::arrays;
pub use botbasic_core::keywords::core_functions;
pub use botbasic_core::keywords::datetime;
pub use botbasic_core::keywords::errors;
pub use botbasic_core::keywords::first;
pub use botbasic_core::keywords::for_next;
pub use botbasic_core::keywords::format;
pub use botbasic_core::keywords::hear_talk;
pub use botbasic_core::keywords::hearing;
pub use botbasic_core::keywords::last;
pub use botbasic_core::keywords::math;
pub use botbasic_core::keywords::print;
pub use botbasic_core::keywords::procedures;
pub use botbasic_core::keywords::set_context;
pub use botbasic_core::keywords::set_user;
pub use botbasic_core::keywords::string_functions;
pub use botbasic_core::keywords::switch_case;
pub use botbasic_core::keywords::validation;
pub use botbasic_core::keywords::wait;

// ===== RE-EXPORTED FROM botbasic_data =====
pub use botbasic_data::keywords::bot_memory;
pub use botbasic_data::keywords::clear_kb;
#[cfg(feature = "people")]
pub use botbasic_data::keywords::crm;
pub use botbasic_data::keywords::data_operations;
pub use botbasic_data::keywords::detect;
pub use botbasic_data::keywords::find;
pub use botbasic_data::keywords::get;
pub use botbasic_data::keywords::import_export;
pub use botbasic_data::keywords::kb_statistics;
#[cfg(feature = "people")]
pub use botbasic_data::keywords::lead_scoring;
#[cfg(all(feature = "billing", feature = "multimodal"))]
pub use botbasic_data::keywords::products;
pub use botbasic_data::keywords::save_from_unstructured;
pub use botbasic_data::keywords::search;
pub use botbasic_data::keywords::set;
pub use botbasic_data::keywords::table_access;
pub use botbasic_data::keywords::table_definition;
pub use botbasic_data::keywords::table_migration;
pub use botbasic_data::keywords::think_kb;
pub use botbasic_data::keywords::use_account;
pub use botbasic_data::keywords::use_kb;
pub use botbasic_data::keywords::user_memory;

// ===== RE-EXPORTED FROM botbasic_comms (Phase 2a) =====
#[cfg(feature = "video")]
pub use botbasic_comms::keywords::weather;
#[cfg(feature = "mail")]
pub use botbasic_comms::keywords::create_draft;
#[cfg(feature = "meet")]
pub use botbasic_comms::keywords::play;
#[cfg(feature = "automation")]
pub use botbasic_comms::keywords::webhook;
#[cfg(feature = "mail")]
pub use botbasic_comms::keywords::on_email;

// ===== RE-EXPORTED FROM botbasic_ai (Phase 2a) =====
#[cfg(feature = "llm")]
pub use botbasic_ai::keywords::remember;
#[cfg(feature = "llm")]
pub use botbasic_ai::keywords::episodic_memory;
#[cfg(feature = "llm")]
pub use botbasic_ai::keywords::knowledge_graph;
pub use botbasic_ai::keywords::human_approval;
pub use botbasic_ai::keywords::qrcode;
pub use botbasic_ai::keywords::web_data;
pub use botbasic_ai::keywords::http_operations;
#[cfg(feature = "automation")]
pub use botbasic_ai::keywords::on_form_submit;
pub use botbasic_ai::keywords::use_tool;
pub use botbasic_ai::keywords::orchestration;
#[cfg(feature = "llm")]
pub use botbasic_ai::keywords::llm_keyword;
#[cfg(feature = "llm")]
pub use botbasic_ai::keywords::ai_tools;
#[cfg(feature = "llm")]
pub use botbasic_ai::keywords::model_routing;
pub use botbasic_ai::keywords::agent_reflection;
#[cfg(feature = "automation")]
pub use botbasic_ai::keywords::api_tool_generator;

// ===== RE-EXPORTED FROM botbasic_system (Phase 2a) =====
#[cfg(feature = "drive")]
pub use botbasic_system::keywords::file_operations;
#[cfg(feature = "tasks")]
pub use botbasic_system::keywords::create_task;
#[cfg(feature = "automation")]
pub use botbasic_system::keywords::on;
#[cfg(feature = "automation")]
pub use botbasic_system::keywords::on_change;
#[cfg(feature = "tasks")]
pub use botbasic_system::keywords::set_schedule;
#[cfg(feature = "automation")]
pub use botbasic_system::keywords::synchronize;

// ===== CORE KEYWORDS (always available) =====
#[cfg(feature = "chat")]
pub mod add_bot;
#[cfg(feature = "chat")]
pub mod add_member;
#[cfg(feature = "chat")]
pub use botbasic_comms::keywords::add_suggestion;
#[cfg(feature = "chat")]
pub use botbasic_comms::keywords::switcher;
pub mod app_server;
pub mod clear_tools;
#[cfg(feature = "automation")]
pub mod code_sandbox;
#[cfg(feature = "people")]
pub mod db_api;
pub mod face_api;

// ===== WORKFLOW ORCHESTRATION MODULES =====
pub mod events;
pub mod enhanced_memory;
pub mod enhanced_llm;

#[cfg(feature = "llm")]
pub mod llm_macros;
#[cfg(feature = "automation")]
pub mod mcp_client;
#[cfg(feature = "automation")]
pub mod mcp_directory;
pub mod messaging;
#[cfg(feature = "security")]
pub mod security_protection;
pub mod universal_messaging;
pub mod use_website;

// ===== CALENDAR FEATURE KEYWORDS =====
#[cfg(feature = "calendar")]
pub mod book;

// ===== MAIL FEATURE KEYWORDS =====
#[cfg(feature = "mail")]
pub mod send_mail;
#[cfg(feature = "mail")]
pub mod send_template;

// ===== TASKS FEATURE KEYWORDS =====

// ===== SOCIAL FEATURE KEYWORDS =====
#[cfg(feature = "social")]
pub mod social;
#[cfg(feature = "social")]
pub mod social_media;

// ===== LLM FEATURE KEYWORDS =====
#[cfg(feature = "multimodal")]
pub mod multimodal;

// ===== VECTORDB FEATURE KEYWORDS =====

// ===== DRIVE FEATURE KEYWORDS =====

// ===== PEOPLE FEATURE KEYWORDS =====

// ===== COMMUNICATIONS FEATURE KEYWORDS =====
#[cfg(any(feature = "whatsapp", feature = "telegram", feature = "mail"))]
pub mod sms;

// ===== CHAT FEATURE KEYWORDS =====
#[cfg(feature = "chat")]
pub mod transfer_to_human;

// ===== AUTOMATION FEATURE KEYWORDS =====

// ===== MEET FEATURE KEYWORDS =====

// ===== MEDIA FEATURE KEYWORDS =====

// ===== CREATE SITE (needs drive) =====
#[cfg(feature = "drive")]
pub mod create_site;

pub use app_server::configure_app_server_routes;
#[cfg(feature = "people")]
pub use db_api::configure_db_routes;
#[cfg(feature = "automation")]
pub use mcp_client::{McpClient, McpRequest, McpResponse, McpServer, McpTool};
#[cfg(feature = "security")]
pub use security_protection::{
    security_get_report, security_hardening_score, security_install_tool, security_run_scan,
    security_service_is_running, security_start_service, security_stop_service,
    security_tool_is_installed, security_tool_status, security_update_definitions,
    SecurityScanResult, SecurityToolResult,
};
#[cfg(feature = "automation")]
pub use mcp_directory::{McpDirectoryScanResult, McpDirectoryScanner, McpServerConfig};
pub use table_access::{
    check_field_write_access, check_table_access, filter_fields_by_role, load_table_access_info,
    AccessType, TableAccessInfo, UserRoles,
};

pub fn get_all_keywords() -> Vec<String> {
    vec![
        "ADD BOT".to_string(),
        "BOT REFLECTION".to_string(),
        "BROADCAST TO BOTS".to_string(),
        "DELEGATE TO BOT".to_string(),
        "TRANSFER CONVERSATION".to_string(),
        "ADD MEMBER".to_string(),
        "CREATE DRAFT".to_string(),
        "SEND MAIL".to_string(),
        "SEND TEMPLATE".to_string(),
        "SMS".to_string(),
        "ADD SUGGESTION".to_string(),
        "ADD_SUGGESTION_TOOL".to_string(),
        "ADD SWITCHER".to_string(),
        "ADD_SWITCHER".to_string(),
        "CLEAR SWITCHERS".to_string(),
        "CLEAR SUGGESTIONS".to_string(),
        "ADD TOOL".to_string(),
        "CLEAR TOOLS".to_string(),
        "CREATE SITE".to_string(),
        "CREATE TASK".to_string(),
        "DETECT".to_string(),
        "USE TOOL".to_string(),
        "AGGREGATE".to_string(),
        "DELETE".to_string(),
        "FILL".to_string(),
        "FILTER".to_string(),
        "FIND".to_string(),
        "FIRST".to_string(),
        "SEARCH".to_string(),
        "SEARCH PRODUCTS".to_string(),
        "PRODUCTS".to_string(),
        "PRODUCT".to_string(),
        "AUTOCOMPLETE".to_string(),
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
        "CLEAR HEADERS".to_string(),
        "DELETE HTTP".to_string(),
        "GET".to_string(),
        "GRAPHQL".to_string(),
        "PATCH".to_string(),
        "POST".to_string(),
        "PUT".to_string(),
        "SET HEADER".to_string(),
        "SOAP".to_string(),
        "EXIT FOR".to_string(),
        "FOR EACH".to_string(),
        "IF".to_string(),
        "SWITCH".to_string(),
        "WAIT".to_string(),
        "WHILE".to_string(),
        "GET".to_string(),
        "SET".to_string(),
        "GET BOT MEMORY".to_string(),
        "GET USER MEMORY".to_string(),
        "REMEMBER".to_string(),
        "SET BOT MEMORY".to_string(),
        "SET CONTEXT".to_string(),
        "SET USER FACT".to_string(),
        "SET USER MEMORY".to_string(),
        "USER FACTS".to_string(),
        "CLEAR KB".to_string(),
        "THINK KB".to_string(),
        "USE KB".to_string(),
        "USE ACCOUNT".to_string(),
        "USE WEBSITE".to_string(),
        "LLM".to_string(),
        "SET CONTEXT".to_string(),
        "USE MODEL".to_string(),
        "RUN BASH".to_string(),
        "RUN JAVASCRIPT".to_string(),
        "RUN PYTHON".to_string(),
        "HEAR".to_string(),
        "TALK".to_string(),
        "ON".to_string(),
        "ON EMAIL".to_string(),
        "ON CHANGE".to_string(),
        "SET SCHEDULE".to_string(),
        "WEBHOOK".to_string(),
        "SET USER".to_string(),
        "BOOK".to_string(),
        "WEATHER".to_string(),
        "PRINT".to_string(),
        "FORMAT".to_string(),
        "INSTR".to_string(),
        "IS NUMERIC".to_string(),
        "REQUIRE APPROVAL".to_string(),
        "SIMULATE IMPACT".to_string(),
        "CHECK CONSTRAINTS".to_string(),
        "AUDIT LOG".to_string(),
        "PLAN START".to_string(),
        "PLAN END".to_string(),
        "STEP".to_string(),
        "AUTO TASK".to_string(),
        "USE MCP".to_string(),
        "MCP LIST TOOLS".to_string(),
        "MCP INVOKE".to_string(),
        "OPTION A OR B".to_string(),
        "DECIDE".to_string(),
        "ESCALATE".to_string(),
        "SECURITY TOOL STATUS".to_string(),
        "SECURITY RUN SCAN".to_string(),
        "SECURITY GET REPORT".to_string(),
        "SECURITY UPDATE DEFINITIONS".to_string(),
        "SECURITY START SERVICE".to_string(),
        "SECURITY STOP SERVICE".to_string(),
        "SECURITY INSTALL TOOL".to_string(),
        "SECURITY HARDENING SCORE".to_string(),
    ]
}

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

    categories.insert(
        "Monitors".to_string(),
        vec![
            "ON EMAIL".to_string(),
            "ON CHANGE".to_string(),
            "SET SCHEDULE".to_string(),
            "WEBHOOK".to_string(),
        ],
    );

    categories.insert(
        "Security Protection".to_string(),
        vec![
            "SECURITY TOOL STATUS".to_string(),
            "SECURITY RUN SCAN".to_string(),
            "SECURITY GET REPORT".to_string(),
            "SECURITY UPDATE DEFINITIONS".to_string(),
            "SECURITY START SERVICE".to_string(),
            "SECURITY STOP SERVICE".to_string(),
            "SECURITY INSTALL TOOL".to_string(),
            "SECURITY HARDENING SCORE".to_string(),
        ],
    );

    categories
}
