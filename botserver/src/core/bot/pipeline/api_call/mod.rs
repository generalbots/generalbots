//! Declarative `{"__api_call__": …}` handling: extraction, argument recovery
//! and command execution with an optional composed answer.

mod args;
mod consent_class;
mod handle;
mod humanize;
mod parse;

#[cfg(test)]
mod consent_class_tests;

pub(super) use handle::handle_api_call;
// #2583 — the caller must be able to scrub an unhandled envelope from the
// reply so raw JSON never reaches the user.
pub(crate) use parse::strip_api_call_block;
