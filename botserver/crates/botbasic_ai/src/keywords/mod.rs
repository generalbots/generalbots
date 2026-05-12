pub mod agent_reflection;
pub mod ai_tools;
pub mod api_tool_generator;
pub mod clear_tools;
pub mod code_sandbox;
pub mod enhanced_llm;
pub mod enhanced_memory;
pub mod episodic_memory;
pub mod events;
pub mod http_operations;
pub mod human_approval;
pub mod knowledge_graph;
pub mod llm_keyword;
pub mod llm_macros;
pub mod mcp_client;
pub mod mcp_directory;
pub mod model_routing;
pub mod multimodal;
pub mod on_form_submit;
pub mod orchestration;
pub mod qrcode;
pub mod remember;
pub mod use_tool;
pub mod use_website;
pub mod web_data;

use botbasic_types::BasicRuntime;
use botbasic_types::UserSession;
use rhai::Engine;
use std::sync::Arc;

/// Register AI keywords that accept Arc<dyn BasicRuntime>.
/// Keywords that require Arc<AppState> directly are registered at the botserver layer.
pub fn register_ai_keywords(
    state: Arc<dyn BasicRuntime>,
    user: UserSession,
    engine: &mut Engine,
) {
    agent_reflection::register_reflection_keywords(state.clone(), user.clone(), engine);
    ai_tools::register_ai_tools_keywords(state.clone(), user.clone(), engine);
    api_tool_generator::register_api_tool_keywords(state.clone(), user.clone(), engine);
    episodic_memory::register_episodic_memory_keywords(engine);
    http_operations::register_http_operations(state.clone(), user.clone(), engine);
    human_approval::register_approval_keywords(engine);
    knowledge_graph::register_knowledge_graph_keywords(engine);
    llm_keyword::llm_keyword(state.clone(), user.clone(), engine);
    model_routing::register_model_routing_keywords(state.clone(), user.clone(), engine);
    on_form_submit::on_form_submit_keyword(state.clone(), user.clone(), engine);
    orchestration::register_orchestrate_workflow(state.clone(), user.clone(), engine);
    qrcode::register_qrcode_keywords(state.clone(), user.clone(), engine);
    use_tool::use_tool_keyword(state.clone(), user.clone(), engine);
    web_data::register_web_data_keywords(state.clone(), user.clone(), engine);

    // Migrated from Arc<AppState> to Arc<dyn BasicRuntime>:
    clear_tools::clear_tools_keyword(state.clone(), user.clone(), engine);
    code_sandbox::register_sandbox_keywords(state.clone(), user.clone(), engine);
    enhanced_memory::register_bot_share_memory(state.clone(), user.clone(), engine);
    enhanced_memory::register_bot_sync_memory(state.clone(), user.clone(), engine);
    events::register_on_event(state.clone(), user.clone(), engine);
    events::register_publish_event(state.clone(), user.clone(), engine);
    events::register_wait_for_event(state.clone(), user.clone(), engine);
    remember::remember_keyword(state.clone(), user.clone(), engine);
    use_website::use_website_keyword(state.clone(), user.clone(), engine);
    use_website::register_use_website_function(state.clone(), user.clone(), engine);
    use_website::clear_websites_keyword(state.clone(), user.clone(), engine);

    // These still require Arc<AppState> — registered at botserver layer:
    // enhanced_llm (SmartLLMRouter needs AppState), llm_macros (direct llm_provider access),
    // mcp_client (struct stores AppState), multimodal (empty stub)
}
