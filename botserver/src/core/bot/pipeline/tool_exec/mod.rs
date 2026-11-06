//! Tool execution entry points.
//!
//! `run_tool_exec` answers a `TOOL_EXEC` (message type 6) by running the tool
//! script directly; `run_llm_tool_call` executes a tool the model requested in
//! a `__tool_call__` block. Both share the script runner in `script`; the
//! reply dialects and extraction live in `parse`.
//!
//! Split out of a single 751-line file: the public surface (`run_tool_exec`,
//! `run_llm_tool_call`, the placeholder and greeting gates) is unchanged.

mod direct;
mod is_placeholder_value;
mod llm_call;
mod parse;
mod script;

pub use direct::run_tool_exec;
pub(crate) use is_placeholder_value::bot_llm_credentials;
pub use is_placeholder_value::{is_generic_greeting, is_placeholder_value};
pub use llm_call::run_llm_tool_call;
