//! Persisted user memory (GB6-16).
//!
//! Preferences edited in Settings → Memory live in `chat_preferences`; this module
//! is the server-side reader that puts them into the conversation prompt, so a
//! new session starts with what the user already taught the assistant instead
//! of repeating it.
//!
//! Split by responsibility: [`store`] reads and renders the prompt block,
//! [`directives`] parses remember/forget turns and [`forget`] applies them.

mod directives;
mod forget;
mod store;

pub use directives::{extract_forget_directive, extract_remember_directive, memory_key_for};
pub use forget::{capture_from_message, forget_memories};
pub use store::{load_for_user, render_for_prompt, render_rows, UserMemoryRow};
