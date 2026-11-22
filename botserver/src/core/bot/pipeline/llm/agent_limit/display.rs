//! The agent's own limit message, cleaned for the card.
//!
//! The words stay the agent's; only what must not reach the screen goes:
//! terminal escapes and line breaks, query strings of links, anything shaped
//! like a credential, file paths, plain-http and IP addresses.

/// Longest message the card shows, in characters.
const MAX_CHARS: usize = 400;

/// Prefixes of well-known credential formats (compared in lower case).
const SECRET_PREFIXES: &[&str] = &["sk-", "sk_", "ghp_", "gho_", "github_pat_", "xox", "eyj", "akia", "aiza"];
/// `name=value` pairs whose name says the value is secret.
const SECRET_NAMES: &[&str] = &["token", "key", "secret", "password", "passwd", "auth", "session", "cookie"];

/// Characters a word may end with that are not part of it.
fn is_trailing(c: char) -> bool {
    matches!(c, ')' | ']' | '}' | ',' | '.' | ';' | ':' | '!' | '?' | '"' | '\'')
}

/// Removes ANSI escape sequences and every other control character.
fn plain(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            for next in chars.by_ref() {
                if next.is_ascii_alphabetic() {
                    break;
                }
            }
        } else if c.is_control() {
            out.push(' ');
        } else {
            out.push(c);
        }
    }
    out
}

fn looks_secret(core: &str) -> bool {
    let lower = core.to_ascii_lowercase();
    if lower.starts_with("https://") {
        return false;
    }
    if SECRET_PREFIXES.iter().any(|p| lower.starts_with(p) && core.len() >= p.len() + 8) {
        return true;
    }
    if let Some((name, value)) = lower.split_once('=') {
        if !value.is_empty() && SECRET_NAMES.iter().any(|n| name.contains(n)) {
            return true;
        }
    }
    let opaque = core.len() >= 24
        && core.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.' | '/' | '+' | '='));
    let mixed = core.chars().any(|c| c.is_ascii_digit())
        && core.chars().any(|c| c.is_ascii_lowercase())
        && core.chars().any(|c| c.is_ascii_uppercase());
    opaque && (mixed || core.len() >= 32)
}

fn looks_internal(core: &str) -> bool {
    let host = core.split(':').next().unwrap_or(core);
    let ipv4 = host.split('.').count() == 4
        && host.split('.').all(|p| (1..=3).contains(&p.len()) && p.chars().all(|c| c.is_ascii_digit()));
    let path = (core.starts_with('/') && core[1..].contains('/'))
        || core.starts_with("~/")
        || core.starts_with("./")
        || core.starts_with("../")
        || core.contains('\\');
    ipv4 || path || core.to_ascii_lowercase().starts_with("http://") || core.contains("localhost")
}

/// One word as the card may show it, or `None` to drop it. Links keep their
/// address without query or fragment.
fn word(raw: &str) -> Option<String> {
    let Some(start) = raw.find(|c: char| c.is_alphanumeric() || matches!(c, '/' | '~' | '.' | '\\')) else {
        return Some(raw.to_string());
    };
    let end = raw
        .char_indices()
        .rev()
        .find(|(_, c)| !is_trailing(*c))
        .map(|(i, c)| i + c.len_utf8())
        .filter(|end| *end > start);
    let Some(end) = end else {
        return Some(raw.to_string());
    };
    let (lead, core, trail) = (&raw[..start], &raw[start..end], &raw[end..]);
    let core = if core.to_ascii_lowercase().starts_with("https://") {
        core.split(['?', '#']).next().unwrap_or(core)
    } else {
        core
    };
    if looks_secret(core) || looks_internal(core) {
        return None;
    }
    Some(format!("{lead}{core}{trail}"))
}

/// The message for the card: one line of plain text, at most
/// [`MAX_CHARS`] characters, with nothing secret or internal in it.
pub(super) fn message(detail: &str) -> String {
    let mut out = String::new();
    let mut count = 0;
    for shown in plain(detail).split_whitespace().filter_map(word) {
        let len = shown.chars().count();
        if count + len + 1 > MAX_CHARS {
            out.push('\u{2026}');
            break;
        }
        if !out.is_empty() {
            out.push(' ');
            count += 1;
        }
        out.push_str(&shown);
        count += len;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::message;

    #[test]
    fn the_codex_message_is_shown_word_for_word() {
        let codex = "You've hit your usage limit. Upgrade to Pro (https://chatgpt.com/explore/pro), visit https://chatgpt.com/settings/usage to purchase more credits or try again at 10:15 PM.";
        assert_eq!(message(codex), codex);
    }

    #[test]
    fn whitespace_and_escapes_are_flattened() {
        assert_eq!(message("\u{1b}[31mYou've hit\u{1b}[0m   your\n limit \u{2219} resets 3pm"), "You've hit your limit \u{2219} resets 3pm");
    }

    #[test]
    fn credentials_paths_and_addresses_are_dropped() {
        let raw = "Limit reached for sk-proj-abcdefghijklmnop token=abc123 key \
                   eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIx at /home/codex/.codex/auth.json \
                   via http://10.0.0.5:8080 and 192.168.1.20:443, see https://example.com/usage?session=XYZ#top.";
        assert_eq!(message(raw), "Limit reached for key at via and see https://example.com/usage.");
        assert_eq!(message("Opaque Ab3dEf9hIj2kLm5nOp8qRs1tUv here"), "Opaque here");
    }

    #[test]
    fn ordinary_words_survive() {
        let raw = "Run /upgrade \u{2014} gpt-5.1-codex-max resets in 2 hours (UTC).";
        assert_eq!(message(raw), raw);
    }

    #[test]
    fn long_messages_are_cut_at_a_word() {
        let long = "word ".repeat(200);
        let shown = message(&long);
        assert!(shown.chars().count() <= 401);
        assert!(shown.ends_with("word\u{2026}"));
    }
}
