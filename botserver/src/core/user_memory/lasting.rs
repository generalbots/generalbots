//! Which fact a chat message asks the assistant to keep: a "remember …"
//! request or a lasting preference about the person, also after a greeting or
//! as one sentence of a longer message ("Oi! Tudo bem? Prefiro respostas
//! curtas."). A question is never kept.

use super::directives::extract_remember_directive;

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
    "de agora em diante ",
    "a partir de agora, ",
    "a partir de agora ",
    "daqui para frente, ",
    "daqui pra frente, ",
    "i prefer ",
    "my name is ",
    "call me ",
    "i work at ",
    "i work in ",
    "i work for ",
    "always answer ",
    "always reply ",
    "from now on, ",
    "from now on ",
];

/// Greetings and courtesies that may open a message or one of its sentences.
const LEAD_INS: &[&str] = &[
    "por favor",
    "please",
    "olá",
    "ola",
    "oi",
    "ei",
    "hey",
    "hello",
    "hi",
    "ok",
    "bom dia",
    "boa tarde",
    "boa noite",
    "e aí",
    "aliás",
    "só pra avisar",
    "só para avisar",
    "só pra você saber",
    "só para você saber",
    "by the way",
    "just so you know",
];

/// Endings that make a statement look like a question without asking one
/// ("Prefiro respostas curtas, ok?").
const TAG_QUESTIONS: &[&str] = &[
    ", ok", ", tá", ", ta", ", certo", ", beleza", ", combinado", ", pode ser", ", tudo bem", ", okay",
    ", alright", ", right",
];

/// A preference sentence longer than this is a task, not a preference.
const MAX_PREFERENCE_CHARS: usize = 240;
/// Only short, single-paragraph messages are searched sentence by sentence.
const MAX_MESSAGE_CHARS: usize = 600;

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
        let after_lead_in = LEAD_INS.iter().find_map(|word| {
            let head = rest.get(..word.len())?;
            let tail = rest.get(word.len()..)?;
            (head.to_lowercase() == *word && tail.starts_with([',', ' ', '!']))
                .then(|| tail.trim_start_matches([',', '!', ' ']))
        });
        match after_lead_in {
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

fn remember_request(text: &str) -> Option<String> {
    extract_remember_directive(text).or_else(|| after_prefix(text, MORE_REMEMBER_PREFIXES))
}

fn lasting_preference(sentence: &str) -> Option<String> {
    let sentence = sentence.trim().trim_end_matches(['.', '!', ';']).trim();
    let lower = sentence.to_lowercase();
    (sentence.chars().count() <= MAX_PREFERENCE_CHARS
        && PREFERENCE_PREFIXES.iter().any(|prefix| lower.starts_with(*prefix)))
    .then(|| sentence.to_string())
}

/// The sentences of a message with the mark that ends each one. A mark only
/// ends a sentence before a space or the end, so "versão 2.0" stays whole.
pub(super) fn sentences(text: &str) -> Vec<(&str, Option<char>)> {
    let mut found = Vec::new();
    let mut start = 0;
    let mut chars = text.char_indices().peekable();
    while let Some((index, mark)) = chars.next() {
        let ends = matches!(mark, '.' | '!' | '?' | ';')
            && chars.peek().is_none_or(|(_, next)| next.is_whitespace());
        if ends {
            let sentence = text[start..index].trim();
            if !sentence.is_empty() {
                found.push((sentence, Some(mark)));
            }
            start = index + mark.len_utf8();
        }
    }
    let last = text[start..].trim();
    if !last.is_empty() {
        found.push((last, None));
    }
    found
}

/// The sentence as a statement: itself, or without a tag question such as
/// ", ok?"; None for a real question.
fn statement(sentence: &str, mark: Option<char>) -> Option<&str> {
    if mark != Some('?') {
        return Some(sentence);
    }
    let lower = sentence.to_lowercase();
    let tag = TAG_QUESTIONS.iter().find(|tag| lower.ends_with(**tag))?;
    sentence.get(..sentence.len().checked_sub(tag.len())?).map(str::trim_end)
}

/// The fact a message asks to keep, if any. Questions never are.
pub fn fact_to_remember(text: &str) -> Option<String> {
    let text = strip_lead_in(text).trim();
    if !text.ends_with('?') {
        if let Some(fact) = remember_request(text) {
            return Some(fact);
        }
    }
    if text.contains('\n') || text.chars().count() > MAX_MESSAGE_CHARS {
        return None;
    }
    sentences(text).into_iter().find_map(|(sentence, mark)| {
        let sentence = statement(strip_lead_in(sentence), mark)?;
        remember_request(sentence).or_else(|| lasting_preference(sentence))
    })
}
