pub mod api_call;
pub mod attachments;
pub mod channel_entry;
pub mod exec;
pub mod history;
pub mod kb;
pub mod llm;
pub mod mentions;
pub mod native_tools;
pub mod sink;
pub mod start_bas;
pub mod tool_exec;
pub mod types;
pub mod enriched_llm;
pub mod vibe_route;
#[cfg(all(feature = "agent-vm", feature = "drive"))]
pub mod native_drive;
#[cfg(feature = "drive")]
pub mod drive_context;
#[cfg(feature = "research")]
pub mod web_context;

pub use channel_entry::run_pipeline_for_channel;
pub use exec::{process_message_internal, PipelineFn};
pub use kb::inject_kb;
pub use llm::stream_llm_response;
pub use sink::{ChannelSink, MpscChannelSink};
pub use start_bas::run_start_bas;
pub use tool_exec::{run_llm_tool_call, run_tool_exec};
pub use types::{LlmTarget, PipelineError, PipelineResult, SessionIdentity};
