//! One chat turn and the person's memory: what the turn asked for (remember,
//! forget, a lasting preference), what was done, and the note that tells the
//! model what it remembers and what happened, so it never claims a save that
//! did not happen nor denies a memory it has.

use uuid::Uuid;

use botcore::shared::utils::DbPool;

use super::directives::{extract_forget_directive, extract_remember_directive};
use super::forget::{forget_acknowledgement, forget_memories_with_outcome};
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

/// Further ways of asking to remember, beside the directive prefixes.
const MORE_REMEMBER_PREFIXES: &[&str] = &[
    "quero que você se lembre de que ",
    "quero que você se lembre que ",
    "quero que você lembre que ",
    "quero que voce lembre que ",
    "você pode lembrar que ",
    "voce pode lembrar que ",
    "pode lembrar que ",
    "lembre-se sempre de que ",
    "lembre-se sempre que ",
    "lembre sempre que ",
    "não se esqueça de que ",
    "não se esqueça que ",
    "não esqueça que ",
    "nao esqueca que ",
    "guarde na memória que ",
    "guarde na memoria que ",
    "salve na memória que ",
    "salve na memoria que ",
    "memorize que ",
    "memorize: ",
    "grave que ",
    "keep in mind that ",
    "don't forget that ",
    "do not forget that ",
    "save to memory that ",
    "save to memory: ",
    "memorize that ",
];

/// Openings of a lasting preference or fact about the person, stated without
/// "remember". Kept short and first-person so a task is not taken for one.
const PREFERENCE_PREFIXES: &[&str] = &[
    "prefiro ",
    "eu prefiro ",
    "meu nome é ",
    "me chamo ",
    "meu setor é ",
    "eu trabalho na ",
    "eu trabalho no ",
    "trabalho na ",
    "trabalho no ",
    "sempre responda ",
    "responda sempre ",
    "de agora em diante, ",
    "a partir de agora, ",
    "i prefer ",
    "my name is ",
    "call me ",
    "i work at ",
    "i work in ",
    "i work for ",
    "always answer ",
    "always reply ",
    "from now on, ",
];

const COURTESIES: &[&str] = &["por favor", "please", "olá", "ola", "oi", "ei", "hey", "hi", "ok"];
const MAX_PREFERENCE_CHARS: usize = 240;

/// The message without leading tags such as "[teste]" and greetings such as
/// "Por favor, ", so "Por favor, lembre que …" reads as a remember request.
pub fn strip_lead_in(text: &str) -> &str {
    let mut rest = text.trim_start();
    loop {
        if let Some(inner) = rest.strip_prefix('[') {
            if let Some(end) = inner.find(']').filter(|end| *end <= 24) {
                rest = inner[end + 1..].trim_start();
                continue;
            }
        }
        let after_courtesy = COURTESIES.iter().find_map(|word| {
            let head = rest.get(..word.len())?;
            let tail = rest.get(word.len()..)?;
            (head.to_lowercase() == *word && tail.starts_with([',', ' ', '!']))
                .then(|| tail.trim_start_matches([',', '!', ' ']))
        });
        match after_courtesy {
            Some(tail) if !tail.is_empty() => rest = tail,
            _ => return rest,
        }
    }
}

fn after_prefix(text: &str, prefixes: &[&str]) -> Option<String> {
    let lower = text.to_lowercase();
    let prefix = prefixes.iter().find(|prefix| lower.starts_with(**prefix))?;
    let fact: String = text.chars().skip(prefix.chars().count()).collect();
    let fact = fact.trim().trim_end_matches(['.', '!', ';']).trim().to_string();
    (fact.chars().count() >= 3 && fact.chars().count() <= 500).then_some(fact)
}

/// The fact a message asks to keep, if any. Questions never are.
pub fn fact_to_remember(text: &str) -> Option<String> {
    let text = strip_lead_in(text).trim();
    if text.ends_with('?') {
        return None;
    }
    if let Some(fact) = extract_remember_directive(text).or_else(|| after_prefix(text, MORE_REMEMBER_PREFIXES)) {
        return Some(fact);
    }
    let single_sentence = !text.contains('\n') && text.chars().count() <= MAX_PREFERENCE_CHARS;
    let lower = text.to_lowercase();
    (single_sentence && PREFERENCE_PREFIXES.iter().any(|prefix| lower.starts_with(*prefix)))
        .then(|| text.trim_end_matches(['.', '!', ';']).trim().to_string())
}

/// Applies the turn's memory request. Facts are kept for the person, not for
/// one bot; a forget request works even with memory off.
pub fn capture_from_message(pool: &DbPool, user_id: Uuid, bot_id: Option<Uuid>, text: &str) -> MemoryTurn {
    let text = strip_lead_in(text);
    if let Some(needle) = extract_forget_directive(text) {
        return match forget_memories_with_outcome(pool, user_id, bot_id, &needle) {
            Ok(deleted) => MemoryTurn::Forgot(deleted),
            Err(error) => {
                botlog::safe_warn!("memory: could not forget entry: {error}");
                MemoryTurn::ForgetFailed
            }
        };
    }
    let Some(fact) = fact_to_remember(text) else {
        return MemoryTurn::Untouched;
    };
    if !is_enabled(pool, user_id) {
        return MemoryTurn::Off;
    }
    match store_memory(pool, user_id, None, &fact) {
        Ok(()) => {
            botlog::safe_info!("memory: stored an entry for user {user_id}");
            MemoryTurn::Saved(fact)
        }
        Err(error) => {
            botlog::safe_warn!("memory: could not store entry: {error}");
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
        _ => {}
    }
    note.trim_end().to_string()
}
