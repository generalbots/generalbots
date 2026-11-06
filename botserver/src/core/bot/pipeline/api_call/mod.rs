//! Declarative `{"__api_call__": …}` handling: extraction, argument recovery
//! and command execution with an optional composed answer.

mod args;
mod handle;
mod humanize;
mod parse;

pub(super) use handle::handle_api_call;
