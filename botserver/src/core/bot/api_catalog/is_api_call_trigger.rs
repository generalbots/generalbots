/// Trigger variants the model uses for an api-call envelope. Some emit the
/// JSON key as `__api_call__` (canonical), others as `api_call` or
/// `_api_call_`; spaces around the colon are common too.
const TRIGGERS: [&str; 8] = [
    "\"__api_call__\":",
    "\"__api_call__\" :",
    "\"api_call\":",
    "\"api_call\" :",
    "\"_api_call_\":",
    "\"_api_call_\" :",
    "\"api-call\":",
    "\"api-call\" :",
];

/// Convenience detection for the API-call trigger. Tolerate every variant so
/// the trigger is never missed and never rendered to the user as raw JSON.
pub fn is_api_call_trigger(hay: &str) -> bool {
    TRIGGERS.iter().any(|t| hay.contains(t))
}

/// Position of the first API-call trigger variant in `hay`, if any.
pub fn find_api_call(hay: &str) -> Option<usize> {
    TRIGGERS
        .iter()
        .filter_map(|t| hay.find(t))
        .min()
}

/// Whether the tail of `hay` could be the beginning of an API-call envelope
/// whose trigger arrives in the next streamed chunk. #2583: providers split
/// `{"__api_call__":` across chunks, so a per-chunk check lets the rest of a
/// large payload reach the chat bubble as text. The suffix (not the whole
/// tail) is checked because ordinary prose usually precedes the partial JSON.
pub fn is_api_call_trigger_start(hay: &str) -> bool {
    const MAX_TAIL: usize = 32;
    let tail: String = hay.chars().rev().take(MAX_TAIL).collect::<Vec<_>>().into_iter().rev().collect();
    if tail.is_empty() {
        return false;
    }
    for trigger in TRIGGERS {
        // Both with and without the opening object brace: the provider may
        // split before or after the `{`.
        let patterns = [trigger.to_string(), format!("{{{trigger}")];
        for pattern in patterns {
            for end in 1..=pattern.len() {
                if !pattern.is_char_boundary(end) {
                    continue;
                }
                if tail.ends_with(&pattern[..end]) {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod trigger_tests {
    use super::{is_api_call_trigger, is_api_call_trigger_start};

    #[test]
    fn detects_spaced_and_brace_prefixes() {
        assert!(is_api_call_trigger("{\"__api_call__\" : {\"name\": \"x\"}}"));
        // Split across chunks, with and without preceding prose (#2583).
        assert!(is_api_call_trigger_start("{\"__ap"));
        assert!(is_api_call_trigger_start("I will do that: {\"__api_call__"));
        assert!(is_api_call_trigger_start("\"__api_"));
        assert!(!is_api_call_trigger_start("Here is the answer in prose."));
        assert!(!is_api_call_trigger_start(""));
    }
}
