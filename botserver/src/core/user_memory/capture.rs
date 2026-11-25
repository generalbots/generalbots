//! One chat turn and the person's memory: what the turn asked for (remember,
//! forget, a lasting preference), what was done, and the note that tells the
//! model what it remembers and what happened, so it never claims a save that
//! did not happen nor denies a memory it has.

use uuid::Uuid;

use botcore::shared::utils::DbPool;

use super::directives::extract_forget_directive;
use super::forget::{forget_acknowledgement, forget_memories_with_outcome};
use super::lasting::{fact_to_remember, strip_lead_in};
use super::store::{is_enabled, render_for_prompt, store_memory};

/// What a turn did to the person's memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemoryTurn {
    /// Nothing to remember or forget.
    Untouched,
    /// The fact was stored.
    Saved(String),
    /// The person asked to remember something while memory is off.
    Off,
    /// Storing the fact failed.
    Failed,
    /// A forget request removed this many entries.
    Forgot(usize),
    /// A forget request could not be applied.
    ForgetFailed,
}

/// Applies the turn's memory request. Facts are kept for the person, not for
/// one bot; a forget request works even with memory off.
pub fn capture_from_message(pool: &DbPool, user_id: Uuid, bot_id: Option<Uuid>, text: &str) -> MemoryTurn {
    let text = strip_lead_in(text);
    if let Some(needle) = extract_forget_directive(text) {
        return match forget_memories_with_outcome(pool, user_id, bot_id, &needle) {
            Ok(deleted) => {
                botlog::safe_info!("memory: turn outcome=forgot count={deleted}");
                MemoryTurn::Forgot(deleted)
            }
            Err(error) => {
                botlog::safe_warn!("memory: turn outcome=forget-failed error={error}");
                MemoryTurn::ForgetFailed
            }
        };
    }
    let Some(fact) = fact_to_remember(text) else {
        botlog::safe_info!("memory: turn outcome=untouched");
        return MemoryTurn::Untouched;
    };
    if !is_enabled(pool, user_id) {
        botlog::safe_info!("memory: turn outcome=off");
        return MemoryTurn::Off;
    }
    match store_memory(pool, user_id, None, &fact) {
        Ok((key, existed)) => {
            let outcome = if existed { "updated" } else { "saved" };
            botlog::safe_info!("memory: turn outcome={outcome} key={key}");
            MemoryTurn::Saved(fact)
        }
        Err(error) => {
            botlog::safe_warn!("memory: turn outcome=failed error={error}");
            MemoryTurn::Failed
        }
    }
}

/// The direct answer to a forget request, in the person's language; None for
/// every other turn, which the model answers.
pub fn forget_reply(pool: &DbPool, user_id: Uuid, turn: &MemoryTurn) -> Option<String> {
    let (key, count) = match turn {
        MemoryTurn::Forgot(count) => (forget_acknowledgement(*count), *count),
        MemoryTurn::ForgetFailed => ("error-memory-not-updated", 0),
        _ => return None,
    };
    let locale = botcore::i18n::locale_for_user_pool(pool, user_id);
    Some(botcore::i18n::t_args(&locale, key, &[("count", &count.to_string())]))
}

/// The system note of every turn: what the assistant remembers about the
/// person and what this turn did.
pub fn turn_note(pool: &DbPool, user_id: Uuid, bot_id: Option<Uuid>, turn: &MemoryTurn) -> String {
    let enabled = is_enabled(pool, user_id);
    let stored = if enabled { render_for_prompt(pool, user_id, bot_id) } else { String::new() };
    compose_note(enabled, &stored, turn)
}

pub(super) fn compose_note(enabled: bool, stored: &str, turn: &MemoryTurn) -> String {
    if !enabled {
        return "MEMORY: switched off by this person in Settings › Memory, so nothing from earlier \
                conversations is available and nothing is saved. If they ask you to remember something, \
                tell them memory is off and can be turned on in Settings › Memory; never say it was saved."
            .to_string();
    }
    let mut note = String::from(
        "MEMORY (kept across conversations): you remember the facts and preferences this person asked \
         you to keep; they can view, edit or delete them in Settings › Memory. Use them whenever they are \
         relevant, including when asked what you know about the person. Never say you cannot remember \
         between conversations, and never look for them through a command or a settings tool: this list \
         is all there is.\n",
    );
    if stored.is_empty() {
        note.push_str(
            "Nothing is stored yet. A message such as \"lembre que …\" or \"remember that …\" is saved automatically.\n",
        );
    } else {
        note.push_str("Stored for this person:\n");
        note.push_str(stored);
    }
    match turn {
        MemoryTurn::Saved(fact) => note.push_str(&format!(
            "THIS TURN: saved to memory: \"{fact}\". Confirm it in one short sentence in the person's \
             language (for example \"Vou lembrar disso.\") and answer anything else they asked."
        )),
        MemoryTurn::Failed => note.push_str(
            "THIS TURN: the person asked you to remember something but saving it failed. Say it could not \
             be saved this time and suggest trying again; never say it was saved.",
        ),
        MemoryTurn::Untouched => note.push_str(
            "THIS TURN: nothing was saved to memory. Never say you noted, saved or will remember anything \
             (no \"Anotado\", \"Vou lembrar\", \"Guardei\"); follow what they ask within this conversation, \
             and if they want it kept for later, say they can write \"lembre que …\".",
        ),
        _ => {}
    }
    note.trim_end().to_string()
}
