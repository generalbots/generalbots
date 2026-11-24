//! Append-only access audit with secret redaction (#2452).
//!
//! `AuditLog` exposes no mutation or removal API for recorded events: entries
//! can only be appended. Redaction happens once, at record time, on the
//! free-form detail string, so a token or password can never be persisted by a
//! caller that forgot to scrub it.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{Action, Decision};

/// Replacement inserted wherever a secret value was found.
pub const REDACTED: &str = "[REDACTED]";

/// One immutable audit entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuditEvent {
    pub sequence: u64,
    pub timestamp: DateTime<Utc>,
    pub object_id: String,
    pub actor: String,
    pub action: Action,
    pub decision: Decision,
    pub ip: Option<String>,
    /// Already redacted; never contains a token or password value.
    pub detail: String,
}

/// Input to [`AuditLog::record`]. `now` is supplied by the caller to keep the
/// log deterministic in tests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuditInput {
    pub timestamp: DateTime<Utc>,
    pub object_id: String,
    pub actor: String,
    pub action: Action,
    pub decision: Decision,
    pub ip: Option<String>,
    pub detail: String,
}

/// Append-only list of access events.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuditLog {
    events: Vec<AuditEvent>,
    next_sequence: u64,
}

impl AuditLog {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Append a redacted event and return its monotonic sequence number.
    pub fn record(&mut self, input: AuditInput) -> u64 {
        let sequence = self.next_sequence;
        self.next_sequence = self.next_sequence.saturating_add(1);
        self.events.push(AuditEvent {
            sequence,
            timestamp: input.timestamp,
            object_id: input.object_id,
            actor: input.actor,
            action: input.action,
            decision: input.decision,
            ip: input.ip,
            detail: redact(&input.detail),
        });
        sequence
    }

    #[must_use]
    pub fn events(&self) -> &[AuditEvent] {
        &self.events
    }

    #[must_use]
    pub fn latest(&self) -> Option<&AuditEvent> {
        self.events.last()
    }

    #[must_use]
    pub fn by_object(&self, object_id: &str) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|event| event.object_id == object_id)
            .collect()
    }

    #[must_use]
    pub fn by_actor(&self, actor: &str) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|event| event.actor == actor)
            .collect()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.events.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

/// Redact secret values from a free-form detail string.
///
/// Recognises `key=value`, `key: value`, `Authorization: Bearer <token>` and
/// bare `Bearer <token>` forms. Only values of sensitive keys are replaced;
/// other text (actor names, object ids, timestamps) is preserved so the trail
/// stays useful. No allocation beyond the output string and no panics.
#[must_use]
pub fn redact(input: &str) -> String {
    const SENSITIVE: &[&str] = &[
        "token",
        "password",
        "passwd",
        "pwd",
        "secret",
        "apikey",
        "api_key",
        "authorization",
        "credential",
        "credentials",
        "cookie",
        "session",
        "access_key",
        "private_key",
    ];

    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len() + REDACTED.len());
    let mut i = 0;
    while i < chars.len() {
        if !is_key_char(chars[i]) {
            out.push(chars[i]);
            i += 1;
            continue;
        }

        let start = i;
        while i < chars.len() && is_key_char(chars[i]) {
            i += 1;
        }
        let key: String = chars[start..i].iter().collect();
        let key_lower = key.to_ascii_lowercase();
        let sensitive = SENSITIVE
            .iter()
            .any(|needle| key_lower == *needle || key_lower.ends_with(*needle));

        let mut separator = i;
        while separator < chars.len() && chars[separator].is_whitespace() {
            separator += 1;
        }
        let had_space = separator > i;
        let separator_char = chars.get(separator).copied();

        // Bare "Bearer <token>" — the marker is kept, the token is not.
        if key_lower == "bearer" && had_space && separator < chars.len() {
            out.push_str(&key);
            while i < separator {
                out.push(chars[i]);
                i += 1;
            }
            let end = scan_value(&chars, separator);
            out.push_str(REDACTED);
            i = end;
            continue;
        }

        if matches!(separator_char, Some('=') | Some(':')) {
            out.push_str(&key);
            while i < separator {
                out.push(chars[i]);
                i += 1;
            }
            if let Some(character) = separator_char {
                out.push(character);
            }
            let mut value_start = separator + 1;
            while value_start < chars.len() && chars[value_start].is_whitespace() {
                out.push(chars[value_start]);
                value_start += 1;
            }
            let mut end = scan_value(&chars, value_start);
            // "Authorization: Bearer abc" — consume the bearer token too.
            if sensitive && key_lower == "authorization" {
                if let Some(bearer_end) = bearer_value_end(&chars, value_start, end) {
                    end = bearer_end;
                }
            }
            if sensitive {
                out.push_str(REDACTED);
            } else {
                for character in &chars[value_start..end] {
                    out.push(*character);
                }
            }
            i = end;
            continue;
        }

        out.push_str(&key);
    }

    out
}

fn is_key_char(character: char) -> bool {
    character.is_ascii_alphanumeric() || character == '_' || character == '-'
}

fn is_value_delimiter(character: char) -> bool {
    character.is_whitespace() || matches!(character, ',' | ';' | '&' | '"' | '\'' | ')' | ']' | '}')
}

fn scan_value(chars: &[char], start: usize) -> usize {
    let mut end = start;
    while end < chars.len() && !is_value_delimiter(chars[end]) {
        end += 1;
    }
    end
}

fn bearer_value_end(chars: &[char], start: usize, value_end: usize) -> Option<usize> {
    let value: String = chars[start..value_end].iter().collect();
    if !value.trim().eq_ignore_ascii_case("bearer") {
        return None;
    }
    let mut token_start = value_end;
    while token_start < chars.len() && chars[token_start].is_whitespace() {
        token_start += 1;
    }
    if token_start >= chars.len() {
        None
    } else {
        Some(scan_value(chars, token_start))
    }
}
