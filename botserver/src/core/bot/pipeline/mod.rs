pub mod agent_turn;
pub mod agent_vm_context;
pub mod api_call;
pub mod attachment_text;
pub mod attachments;
pub mod channel_entry;
pub mod chat_controls;
#[cfg(feature = "drive")]
pub mod doc_delta;
#[cfg(feature = "drive")]
pub mod drive_context;
pub mod enriched_llm;
pub mod event_sink;
pub mod exec;
pub mod history;
pub mod kb;
pub mod llm;
pub mod mentions;
pub mod native_args;
pub mod connector_hints;
pub mod connector_match;
pub mod native_drive;
pub mod native_external_save;
pub mod native_tools;
#[cfg(all(feature = "agent-vm", feature = "drive"))]
pub mod output_publish;
pub mod sink;
pub mod start_bas;
pub mod start_gate;
pub mod surface_catalog;
pub mod surface_tools;
pub mod system_prompt;
pub mod tool_exec;
pub mod tool_paths;
pub mod turn_context;
pub mod turn_hooks;
pub mod types;
pub mod vibe_route;
#[cfg(any(feature = "research", feature = "web-search"))]
pub mod web_context;
pub mod workspace_progress;

pub use channel_entry::run_pipeline_for_channel;
pub use exec::{process_message_internal, PipelineFn};
pub use kb::inject_kb;
pub use llm::stream_llm_response;
pub use sink::{ChannelSink, MpscChannelSink};
pub use start_bas::run_start_bas;
pub use tool_exec::{run_llm_tool_call, run_tool_exec};
pub use types::{LlmTarget, PipelineError, PipelineResult, SessionIdentity};
