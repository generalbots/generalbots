// ===== CORE KEYWORDS (always available) =====
#[cfg(feature = "chat")]
pub mod add_bot;
#[cfg(feature = "chat")]
pub mod add_member;
#[cfg(feature = "chat")]
pub mod add_suggestion;
pub mod agent_reflection;
#[cfg(feature = "llm")]
pub mod ai_tools;
#[cfg(feature = "automation")]
pub mod api_tool_generator;
pub mod app_server;
pub mod arrays;
pub mod bot_memory;
pub mod clear_tools;
#[cfg(feature = "automation")]
pub mod code_sandbox;
pub mod core_functions;
#[cfg(feature = "people")]
pub mod crm;
pub mod data_operations;
pub mod datetime;
pub mod db_api;

// ===== WORKFLOW ORCHESTRATION MODULES =====
pub mod orchestration;
pub mod events;
pub mod enhanced_memory;
pub mod enhanced_llm;

pub mod errors;
pub mod find;
pub mod first;
#[cfg(feature = "billing")]
pub mod products;
pub mod search;
pub mod for_next;
pub mod format;
pub mod get;
pub mod hear_talk;
pub mod http_operations;
pub mod human_approval;
pub mod last;
#[cfg(feature = "llm")]
pub mod llm_keyword;
#[cfg(feature = "llm")]
pub mod llm_macros;
pub mod math;
#[cfg(feature = "automation")]
pub mod mcp_client;
#[cfg(feature = "automation")]
pub mod mcp_directory;
pub mod messaging;
pub mod on;
#[cfg(feature = "automation")]
pub mod on_form_submit;
pub mod print;
pub mod procedures;
pub mod qrcode;
#[cfg(feature = "security")]
pub mod security_protection;
pub mod set;
pub mod set_context;
pub mod set_user;
pub mod string_functions;
pub mod switch_case;
pub mod table_access;
pub mod table_definition;
pub mod universal_messaging;
pub mod use_tool;
pub mod use_website;
pub mod user_memory;
pub mod validation;
pub mod wait;
pub mod web_data;
#[cfg(feature = "automation")]
pub mod webhook;

// ===== CALENDAR FEATURE KEYWORDS =====
#[cfg(feature = "calendar")]
pub mod book;

// ===== MAIL FEATURE KEYWORDS =====
#[cfg(feature = "mail")]
pub mod create_draft;
#[cfg(feature = "mail")]
pub mod on_email;
#[cfg(feature = "mail")]
pub mod send_mail;
#[cfg(feature = "mail")]
pub mod send_template;

// ===== TASKS FEATURE KEYWORDS =====
#[cfg(feature = "tasks")]
pub mod create_task;
#[cfg(feature = "tasks")]
pub mod set_schedule;

// ===== SOCIAL FEATURE KEYWORDS =====
#[cfg(feature = "social")]

#[cfg(feature = "social")]
pub mod social;
#[cfg(feature = "social")]
pub mod social_media;

// ===== LLM FEATURE KEYWORDS =====
#[cfg(feature = "llm")]
pub mod episodic_memory;
#[cfg(feature = "llm")]
pub mod knowledge_graph;
#[cfg(feature = "llm")]
pub mod model_routing;
#[cfg(feature = "llm")]
pub mod multimodal;
#[cfg(feature = "llm")]
pub mod remember;
#[cfg(feature = "llm")]
pub mod save_from_unstructured;

// ===== VECTORDB FEATURE KEYWORDS =====
#[cfg(feature = "vectordb")]
pub mod clear_kb;
#[cfg(feature = "vectordb")]
pub mod kb_statistics;
#[cfg(feature = "vectordb")]
pub mod use_kb;

// ===== DRIVE FEATURE KEYWORDS =====
#[cfg(feature = "drive")]
pub mod file_operations;
#[cfg(feature = "drive")]
pub mod import_export;

// ===== PEOPLE FEATURE KEYWORDS =====
#[cfg(feature = "people")]
pub mod lead_scoring;

// ===== COMMUNICATIONS FEATURE KEYWORDS =====
#[cfg(any(feature = "whatsapp", feature = "telegram", feature = "mail"))]
pub mod sms;

// ===== CHAT FEATURE KEYWORDS =====
#[cfg(feature = "chat")]
pub mod transfer_to_human;

// ===== AUTOMATION FEATURE KEYWORDS =====
#[cfg(feature = "automation")]
pub mod on_change;
#[cfg(feature = "automation")]
pub mod synchronize;

// ===== MEET FEATURE KEYWORDS =====
#[cfg(feature = "meet")]
pub mod play;

// ===== USE ACCOUNT (needs directory or people) =====
#[cfg(any(feature = "directory", feature = "people", feature = "drive"))]
pub mod use_account;

// ===== MEDIA FEATURE KEYWORDS =====
#[cfg(feature = "video")]
pub mod weather;

// ===== CREATE SITE (needs drive) =====
#[cfg(feature = "drive")]
pub mod create_site;

pub use app_server::configure_app_server_routes;
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
        "CLEAR SUGGESTIONS".to_string(),
        "ADD TOOL".to_string(),
        "CLEAR TOOLS".to_string(),
        "CREATE SITE".to_string(),
        "CREATE TASK".to_string(),
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
