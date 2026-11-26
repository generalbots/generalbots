//! Size of each message sent to the model, for the turn log.
//!
//! The first answer of a local engine waits for the whole prompt to be read,
//! so the log shows where the bytes are: one `role=bytes` entry per message,
//! then the total and a rough token estimate (about four bytes per token).

/// Bytes per token used for the estimate in the log.
const BYTES_PER_TOKEN: usize = 4;

fn content_bytes(message: &serde_json::Value) -> usize {
    match message.get("content") {
        Some(serde_json::Value::String(text)) => text.len(),
        Some(other) => other.to_string().len(),
        None => 0,
    }
}

/// `system=1200 system=300 user=61 total=1561 est_tokens=390`.
pub fn summarize(messages: &serde_json::Value) -> String {
    let list = messages.as_array().map(Vec::as_slice).unwrap_or_default();
    let mut parts: Vec<String> = Vec::with_capacity(list.len() + 2);
    let mut total = 0;
    for message in list {
        let role = message.get("role").and_then(serde_json::Value::as_str).unwrap_or("?");
        let bytes = content_bytes(message);
        total += bytes;
        parts.push(format!("{role}={bytes}"));
    }
    parts.push(format!("total={total}"));
    parts.push(format!("est_tokens={}", total.div_ceil(BYTES_PER_TOKEN)));
    parts.join(" ")
}

#[cfg(test)]
mod prompt_sizes_tests {
    use super::summarize;

    #[test]
    fn each_message_is_listed_with_its_size() {
        let messages = serde_json::json!([
            {"role": "system", "content": "abcd"},
            {"role": "user", "content": "Qual a capital do Brasil?"},
        ]);
        assert_eq!(summarize(&messages), "system=4 user=25 total=29 est_tokens=8");
    }

    #[test]
    fn missing_or_structured_content_does_not_break_the_line() {
        let messages = serde_json::json!([
            {"role": "assistant"},
            {"role": "user", "content": [{"type": "text", "text": "oi"}]},
        ]);
        let line = summarize(&messages);
        assert!(line.starts_with("assistant=0 user="), "{line}");
        assert_eq!(summarize(&serde_json::json!(null)), "total=0 est_tokens=0");
    }
}
