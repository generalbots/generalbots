//! Attached images and PDFs for models that can see them.
//!
//! The chat pipeline puts the files of the current turn on a message as
//! `"media": [{"type": "image"|"pdf", "media_type": "...", "data": "<base64>"}]`.
//! Each client turns that into its provider's own content parts, or drops it:
//! a `media` key must never reach a provider as is.

use serde_json::{json, Value};

/// Hosts whose chat models accept images in the request. Anthropic is served
/// by its own client and always sees images; Gemini goes through the Vertex
/// client.
pub fn sees_images(url: &str) -> bool {
    let host = url::Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(str::to_ascii_lowercase))
        .unwrap_or_default();
    let on = |d: &str| host == d || host.ends_with(&format!(".{d}"));
    on("anthropic.com") || on("openai.com") || on("x.ai") || on("googleapis.com")
}

/// Every media item of the conversation, in order.
pub fn collect(messages: &Value) -> Vec<Value> {
    messages
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|m| m.get("media").and_then(Value::as_array))
                .flatten()
                .cloned()
                .collect()
        })
        .unwrap_or_default()
}

fn strip(messages: &Value) -> Value {
    match messages.as_array() {
        Some(arr) => Value::Array(
            arr.iter()
                .map(|m| {
                    let mut m = m.clone();
                    if let Some(obj) = m.as_object_mut() {
                        obj.remove("media");
                    }
                    m
                })
                .collect(),
        ),
        None => messages.clone(),
    }
}

/// OpenAI-shaped request: images become `image_url` parts of the last user
/// message (PDFs are left out: the text already read from them is in the
/// prompt). With `vision` false the media is only removed.
pub fn openai_messages(messages: &Value, vision: bool) -> Value {
    let media = collect(messages);
    let mut out = strip(messages);
    if !vision || media.is_empty() {
        return out;
    }
    let images: Vec<Value> = media
        .iter()
        .filter(|m| m.get("type").and_then(Value::as_str) == Some("image"))
        .map(|m| {
            let mt = m.get("media_type").and_then(Value::as_str).unwrap_or("image/png");
            let data = m.get("data").and_then(Value::as_str).unwrap_or("");
            json!({ "type": "image_url", "image_url": { "url": format!("data:{mt};base64,{data}") } })
        })
        .collect();
    if images.is_empty() {
        return out;
    }
    if let Some(last_user) = out
        .as_array_mut()
        .and_then(|arr| arr.iter_mut().rev().find(|m| m.get("role").and_then(Value::as_str) == Some("user")))
    {
        let text = last_user.get("content").and_then(Value::as_str).unwrap_or("").to_string();
        let mut parts = vec![json!({ "type": "text", "text": text })];
        parts.extend(images);
        last_user["content"] = Value::Array(parts);
    }
    out
}

/// Anthropic content blocks for the media of the conversation: images and
/// PDFs (`document`), followed by the text of the user turn.
pub fn anthropic_blocks(media: &[Value], text: &str) -> Value {
    let mut blocks: Vec<Value> = media
        .iter()
        .filter_map(|m| {
            let kind = m.get("type").and_then(Value::as_str)?;
            let mt = m.get("media_type").and_then(Value::as_str)?;
            let data = m.get("data").and_then(Value::as_str)?;
            let block_type = match kind {
                "image" => "image",
                "pdf" => "document",
                _ => return None,
            };
            Some(json!({ "type": block_type, "source": { "type": "base64", "media_type": mt, "data": data } }))
        })
        .collect();
    blocks.push(json!({ "type": "text", "text": text }));
    Value::Array(blocks)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn convo() -> Value {
        json!([
            {"role": "system", "content": "note", "media": [
                {"type": "image", "media_type": "image/png", "data": "AAA"},
                {"type": "pdf", "media_type": "application/pdf", "data": "BBB"}
            ]},
            {"role": "user", "content": "what is this?"}
        ])
    }

    #[test]
    fn media_never_reaches_a_text_only_host() {
        let out = openai_messages(&convo(), false);
        assert!(out[0].get("media").is_none());
        assert_eq!(out[1]["content"], "what is this?");
    }

    #[test]
    fn images_become_parts_of_the_user_turn() {
        let out = openai_messages(&convo(), true);
        assert!(out[0].get("media").is_none());
        let parts = out[1]["content"].as_array().unwrap();
        assert_eq!(parts[0]["text"], "what is this?");
        assert_eq!(parts[1]["image_url"]["url"], "data:image/png;base64,AAA");
        assert_eq!(parts.len(), 2, "PDFs are not sent to OpenAI-shaped hosts");
    }

    #[test]
    fn anthropic_gets_images_and_documents() {
        let blocks = anthropic_blocks(&collect(&convo()), "hi");
        assert_eq!(blocks[0]["type"], "image");
        assert_eq!(blocks[1]["type"], "document");
        assert_eq!(blocks[2]["text"], "hi");
    }

    #[test]
    fn vision_hosts() {
        assert!(sees_images("https://api.anthropic.com"));
        assert!(sees_images("https://api.openai.com/v1/chat/completions"));
        assert!(sees_images("https://generativelanguage.googleapis.com"));
        assert!(!sees_images("http://localhost:8081/v1/chat/completions"));
        assert!(!sees_images("https://api.kilo.ai/api/gateway/chat/completions"));
        assert!(!sees_images("https://openai.com.evil.example/v1"));
    }
}
