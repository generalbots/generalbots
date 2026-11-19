//! Keeps a request to the local engine inside the context it was started
//! with. The engine refuses a prompt longer than its context instead of
//! cutting it, so a conversation that grew, or a document added to it, used
//! to end in an error.

use serde_json::Value;

use crate::openai_client::OpenAIClient;

/// Appended where a text was shortened, so the model knows it is incomplete.
const CUT_NOTE: &str = "\n[... cut here to fit the local model's context]";
/// Tokens the chat template spends around each message.
const PER_MESSAGE_TOKENS: usize = 8;
/// A text is never shortened below this many tokens.
const SHORTEST_KEPT_TOKENS: usize = 256;

/// Tokens a text is assumed to take. The local models read Portuguese at
/// about 3.5 bytes per token; counting 3 keeps the estimate on the safe side.
pub(crate) fn estimate(text: &str) -> usize {
    text.len().div_ceil(3)
}

fn message_tokens(message: &Value) -> usize {
    let content = match message.get("content") {
        Some(Value::String(text)) => estimate(text),
        Some(Value::Null) | None => 0,
        Some(other) => estimate(&other.to_string()),
    };
    content + PER_MESSAGE_TOKENS
}

fn total_tokens(messages: &[Value]) -> usize {
    messages.iter().map(message_tokens).sum()
}

fn role_is(message: &Value, role: &str) -> bool {
    message.get("role").and_then(|r| r.as_str()) == Some(role)
}

/// Tokens the prompt may take in a context of `context` tokens: room is left
/// for the reply and for the tool definitions sent with the request.
pub(crate) fn prompt_budget(context: usize, tools: Option<&Vec<Value>>) -> usize {
    let reply = (context / 8).clamp(512, 2048);
    let tool_tokens: usize = tools.map_or(0, |list| {
        list.iter().map(|tool| estimate(&tool.to_string())).sum()
    });
    context.saturating_sub(reply).saturating_sub(tool_tokens)
}

/// The messages, reduced until their estimate fits `budget`. The oldest
/// turns go first; the system message and the last message always stay. When
/// those alone are too long, the longest of them keeps only its beginning.
pub(crate) fn fit(messages: &Value, budget: usize) -> Value {
    let Some(array) = messages.as_array() else {
        return messages.clone();
    };
    if total_tokens(array) <= budget {
        return messages.clone();
    }
    let mut kept: Vec<Value> = array.clone();

    while total_tokens(&kept) > budget {
        let last = kept.len().saturating_sub(1);
        let oldest = kept
            .iter()
            .enumerate()
            .find(|(index, message)| *index < last && !role_is(message, "system"))
            .map(|(index, _)| index);
        match oldest {
            Some(index) => {
                kept.remove(index);
            }
            None => break,
        }
    }
    // What is left of the conversation must not open with half a turn.
    loop {
        let last = kept.len().saturating_sub(1);
        let first = kept
            .iter()
            .enumerate()
            .find(|(_, message)| !role_is(message, "system"))
            .map(|(index, _)| index);
        match first {
            Some(index) if index < last && !role_is(&kept[index], "user") => {
                kept.remove(index);
            }
            _ => break,
        }
    }

    for _ in 0..kept.len() {
        let total = total_tokens(&kept);
        if total <= budget {
            break;
        }
        let longest = kept
            .iter()
            .enumerate()
            .filter(|(_, message)| message.get("content").is_some_and(Value::is_string))
            .max_by_key(|(_, message)| message_tokens(message))
            .map(|(index, _)| index);
        let Some(index) = longest else { break };
        let Some(text) = kept[index].get("content").and_then(Value::as_str) else {
            break;
        };
        let now = estimate(text);
        let wanted = now
            .saturating_sub(total - budget)
            .saturating_sub(estimate(CUT_NOTE))
            .max(SHORTEST_KEPT_TOKENS);
        if wanted >= now {
            break;
        }
        let shortened = format!("{}{CUT_NOTE}", head(text, wanted * 3));
        kept[index]["content"] = Value::String(shortened);
    }
    Value::Array(kept)
}

/// The first `max_bytes` of `text`, ending on a character boundary.
fn head(text: &str, max_bytes: usize) -> &str {
    if text.len() <= max_bytes {
        return text;
    }
    let mut end = max_bytes;
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

/// `scheme://host:port` of a base URL that may carry a path.
fn origin(base_url: &str) -> &str {
    let after_scheme = base_url.find("://").map_or(0, |at| at + 3);
    match base_url[after_scheme..].find('/') {
        Some(slash) => &base_url[..after_scheme + slash],
        None => base_url,
    }
}

impl OpenAIClient {
    /// The context of one slot of the local engine, as the engine reports it.
    /// `None` when the server on this machine is not that engine or does not
    /// answer: the request then goes out as it is.
    async fn local_context_size(&self) -> Option<usize> {
        let url = format!("{}/props", origin(&self.base_url));
        let reply = self
            .client
            .get(&url)
            .timeout(std::time::Duration::from_secs(2))
            .send()
            .await
            .ok()?;
        if !reply.status().is_success() {
            return None;
        }
        let props: Value = reply.json().await.ok()?;
        props["default_generation_settings"]["n_ctx"]
            .as_u64()
            .and_then(|n| usize::try_from(n).ok())
            .filter(|n| *n >= 1024)
    }

    /// The messages as they can be sent to the local engine: reduced, when
    /// needed, to what its context holds together with the tool definitions
    /// and a reply.
    pub(crate) async fn fit_local_context(
        &self,
        messages: &Value,
        tools: Option<&Vec<Value>>,
    ) -> Value {
        let Some(context) = self.local_context_size().await else {
            return messages.clone();
        };
        let budget = prompt_budget(context, tools);
        let fitted = fit(messages, budget);
        let count = |value: &Value| value.as_array().map_or(0, Vec::len);
        if fitted != *messages {
            log::warn!(
                "Local engine context is {context} tokens: request reduced from {} to {} messages to fit {budget}",
                count(messages),
                count(&fitted),
            );
        }
        fitted
    }
}

#[cfg(test)]
#[path = "local_context_tests.rs"]
mod local_context_tests;
