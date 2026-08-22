pub mod channel_entry;
pub mod exec;
pub mod kb;
pub mod llm;
pub mod mentions;
pub mod sink;
pub mod start_bas;
pub mod tool_exec;
pub mod types;

pub use channel_entry::run_pipeline_for_channel;
pub use exec::{process_message_internal, PipelineFn};
pub use kb::inject_kb;
pub use llm::stream_llm_response;
pub use sink::{ChannelSink, MpscChannelSink};
pub use start_bas::run_start_bas;
pub use tool_exec::{run_llm_tool_call, run_tool_exec};
pub use types::{PipelineError, PipelineResult};
