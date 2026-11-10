//! Answer-mode side effects: session state, data/chart answers and the SQL
//! and HTML helpers they use.
//!
//! #1730 — the module outgrew the 450-line limit, so it is split by
//! responsibility into sibling modules. Every public item is re-exported here
//! so `super::answer_mode_ops::*` (and therefore `answer_mode::*`) keeps
//! resolving the same paths.

mod chart_answer;
mod data_answer;
mod html_render;
mod llm_text;
mod response_builder;
mod session_state;
mod sql_exec;

pub use chart_answer::generate_chart_response;
pub use data_answer::generate_data_response;
pub use html_render::{format_chart_html, format_results_as_html_table};
pub use session_state::{get_answer_mode, store_answer_mode};
pub use sql_exec::execute_sql_query;
