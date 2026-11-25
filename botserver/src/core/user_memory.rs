//! Persisted user memory (GB7-16).
//!
//! What the person asked the assistant to keep lives, sealed per person, in
//! `chat_preferences` (`botsettings::memory_store`), which Settings → Memory
//! lists and edits. Every chat turn carries it as a system note, so a new
//! conversation, on any engine, starts with what the person already taught.
//!
//! Split by responsibility: [`store`] reads and renders the entries,
//! [`directives`] parses remember/forget turns, [`lasting`] finds the fact a
//! message asks to keep, [`forget`] applies a forget
//! request and [`capture`] decides what a turn does and what the model is told.

mod capture;
#[cfg(test)]
mod capture_tests;
mod directives;
mod forget;
mod lasting;
#[cfg(test)]
mod lasting_tests;
mod store;

pub use capture::{capture_from_message, forget_reply, turn_note, MemoryTurn};
