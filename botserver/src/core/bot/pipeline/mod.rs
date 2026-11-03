pub mod api_call;
pub mod attachments;
pub mod attachment_text;
pub mod channel_entry;
pub mod exec;
pub mod history;
pub mod kb;
pub mod llm;
pub mod mentions;
pub mod native_args;
pub mod native_tools;
pub mod sink;
pub mod start_bas;
pub mod turn_context;
pub mod start_gate;
pub mod agent_vm_context;
pub mod surface_tools;
pub mod system_prompt;
pub mod tool_paths;
pub mod tool_exec;
pub mod turn_hooks;
pub mod types;
pub mod enriched_llm;
pub mod event_sink;
pub mod vibe_route;
#[cfg(all(feature = "agent-vm", feature = "drive"))]
pub mod native_drive;
#[cfg(feature = "drive")]
pub mod drive_context;
#[cfg(any(feature = "research", feature = "web-search"))]
pub mod web_context;

pub use channel_entry::run_pipeline_for_channel;
pub use exec::{process_message_internal, PipelineFn};
pub use kb::inject_kb;
pub use llm::stream_llm_response;
pub use sink::{ChannelSink, MpscChannelSink};
pub use start_bas::run_start_bas;
pub use tool_exec::{run_llm_tool_call, run_tool_exec};
pub use types::{LlmTarget, PipelineError, PipelineResult, SessionIdentity};
