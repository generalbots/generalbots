//! #1547 — the model decides whether a turn needs a live web lookup.
//!
//! There is no keyword list here. A small, cheap completion asks the same
//! model that will answer to classify the turn and, when it decides a lookup
//! is needed, to write the search terms. The decision, the query, the number
//! of sources and the domains are streamed back to the chat as they happen
//! (`reasoning` frames), so the user watches the trip instead of a spinner.
//!
//! A failed decision never blocks the turn: the answer is produced without web
//! context, and the model can still call the `web.search` command itself.

use serde_json::json;
use uuid::Uuid;

use botlib::models::BotResponse;
use botlib::traits::LLMProvider;

use super::sink::ChannelSink;
use super::types::SessionIdentity;

const MAX_RESULTS: usize = 5;
const REGION: &str = "wt-wt";
const DECIDE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(12);
const SEARCH_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(12);

const ROUTER_PROMPT: &str = "You decide, for one user turn, whether an assistant that has a \
live web search tool must use it before answering.\n\
Reply with ONE JSON object and nothing else, no markdown fence:\n\
{\"search\": true|false, \"query\": \"2-6 search keywords\"}\n\
Rules:\n\
- search=true when the answer depends on facts that change (news, prices, releases, versions, \
sports, weather, who currently holds a role, what happened recently), or when the user explicitly \
asks you to search, browse, google, look up or check the web.\n\
- search=false for writing, coding, editing files, running programs, analysing data the user \
provided, translating, calculating, and stable general knowledge that does not change.\n\
- query: the topic only — no \"search for\", no question words, no quotes around the whole query.\n\
- When unsure whether the fact is current, prefer search=true.";

#[derive(Debug, PartialEq, Eq)]
struct Decision {
    search: bool,
    query: String,
}

/// Extracts the router's JSON even if the model wrapped it in prose or a
/// fence. Returns `None` when no usable object is present.
fn parse_decision(raw: &str) -> Option<Decision> {
    let start = raw.find('{')?;
    let end = raw.rfind('}')?;
    if end <= start {
        return None;
    }
    let value: serde_json::Value = serde_json::from_str(&raw[start..=end]).ok()?;
    let search = value.get("search")?.as_bool()?;
    let query = value
        .get("query")
        .and_then(|q| q.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if search && query.is_empty() {
        return None;
    }
    Some(Decision { search, query })
}

/// Streams a one-line progress note into the assistant bubble's thinking
/// section. Channels without raw frames simply ignore it.
async fn status(
    sink: &dyn ChannelSink,
    bot_uuid: Uuid,
    session_id: Uuid,
    user_id: Uuid,
    text: &str,
) {
    // The chat renders a live status line while the trip happens; channels
    // without raw frames fall back to the thinking stream.
    if sink.supports_raw_frames() {
        let frame = json!({ "type": "chat_status", "text": text });
        if sink.send_raw_json(&frame).await.is_ok() {
            return;
        }
    }
    let response = BotResponse {
        bot_id: bot_uuid.to_string(),
        user_id: user_id.to_string(),
        session_id: session_id.to_string(),
        channel: "web".to_string(),
        content: String::new(),
        message_type: botlib::message_types::MessageType::BOT_RESPONSE,
        stream_token: None,
        is_complete: false,
        suggestions: Vec::new(),
        switchers: Vec::new(),
        context_name: None,
        context_length: 0,
        context_max_length: 0,
        reasoning: format!("{text}\n"),
    };
    let _ = sink.send_bot_response(&response).await;
}

async fn decide(
    provider: &std::sync::Arc<dyn LLMProvider>,
    model: &str,
    key: &str,
    user_text: &str,
) -> Option<Decision> {
    let messages = json!([
        { "role": "system", "content": ROUTER_PROMPT },
        { "role": "user", "content": user_text },
    ]);
    let raw = match provider.generate("", &messages, model, key).await {
        Ok(raw) => raw,
        Err(e) => {
            // Some gateways reject the non-streaming shape while serving the
            // same model over the stream (OpenCode Go answers 400 "Model is
            // unavailable" to `stream:false` at times). Retry as a stream so
            // the routing decision still happens.
            log::info!("web context: router generate failed ({e}); retrying as a stream");
            match stream_once(provider, &messages, model, key).await {
                Some(raw) => raw,
                None => return None,
            }
        }
    };
    let decision = parse_decision(&raw);
    if decision.is_none() {
        log::info!(
            "web context: router reply was not usable JSON: {}",
            botlib::text::truncate_with_suffix(raw.trim(), 200, "…")
        );
    }
    decision
}

/// Collects a streaming completion into one string. Used only when the
/// non-streaming call fails.
async fn stream_once(
    provider: &std::sync::Arc<dyn LLMProvider>,
    messages: &serde_json::Value,
    model: &str,
    key: &str,
) -> Option<String> {
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(32);
    let provider = provider.clone();
    let messages = messages.clone();
    let model = model.to_string();
    let key = key.to_string();
    let handle = tokio::spawn(async move {
        let _ = provider
            .generate_stream("", &messages, tx, &model, &key, None)
            .await;
    });
    let mut out = String::new();
    while let Some(chunk) = rx.recv().await {
        out.push_str(&chunk);
    }
    let _ = handle.await;
    (!out.trim().is_empty()).then_some(out)
}

/// Decides (with the model), searches, reports progress and injects the
/// results as the freshest system message.
pub async fn decide_and_inject(
    identity: &SessionIdentity<'_>,
    user_text: &str,
    messages: &mut Vec<serde_json::Value>,
) {
    let SessionIdentity {
        sink,
        state,
        bot_uuid,
        session_id,
        user_id,
        ..
    } = *identity;

    let Some((provider, key, model)) = super::enriched_llm::resolve_bot_llm_provider(state, bot_uuid)
    else {
        log::info!("web context: no LLM provider, skipping the search decision");
        return;
    };

    status(sink, bot_uuid, session_id, user_id, "Checking whether this needs the live web…").await;

    let decision = match tokio::time::timeout(
        DECIDE_TIMEOUT,
        decide(&provider, &model, &key, user_text),
    )
    .await
    {
        Ok(Some(decision)) => decision,
        Ok(None) => {
            log::info!("web context: router returned no usable decision");
            status(sink, bot_uuid, session_id, user_id, "Answering without a web lookup.").await;
            return;
        }
        Err(_) => {
            log::info!("web context: router timed out");
            status(sink, bot_uuid, session_id, user_id, "Answering without a web lookup.").await;
            return;
        }
    };

    if !decision.search {
        status(sink, bot_uuid, session_id, user_id, "Answering from what I already know.").await;
        return;
    }

    status(
        sink,
        bot_uuid,
        session_id,
        user_id,
        &format!("Searching the web for “{}”…", decision.query),
    )
    .await;

    let results = match tokio::time::timeout(
        SEARCH_TIMEOUT,
        botresearch::web_search::search_web(&decision.query, MAX_RESULTS, REGION),
    )
    .await
    {
        Ok(Ok(results)) => results,
        Ok(Err(e)) => {
            log::info!("web context: search failed: {e}");
            status(sink, bot_uuid, session_id, user_id, "The search failed — answering from what I know.").await;
            return;
        }
        Err(_) => {
            log::info!("web context: search timed out");
            status(sink, bot_uuid, session_id, user_id, "The search timed out — answering from what I know.").await;
            return;
        }
    };
    if results.is_empty() {
        status(sink, bot_uuid, session_id, user_id, "No usable sources — answering from what I know.").await;
        return;
    }

    let domains: Vec<&str> = {
        let mut seen: Vec<&str> = Vec::new();
        for result in &results {
            if !seen.contains(&result.source.as_str()) {
                seen.push(result.source.as_str());
            }
        }
        seen
    };
    status(
        sink,
        bot_uuid,
        session_id,
        user_id,
        &format!(
            "Found {} source{} ({}). Reading them and writing the answer…",
            results.len(),
            if results.len() == 1 { "" } else { "s" },
            domains.join(", ")
        ),
    )
    .await;

    let mut block = String::from(
        "Fresh web search results are provided below. They are sufficient for \
         this turn: answer the user NOW from this material. Do not call \
         api.find, web.search or any other command to look for a search tool — \
         the search has already been done for you. Cite the source of every \
         factual or time-sensitive claim as a markdown link [title](url). Never \
         invent a URL or a source. If the results do not answer the question, \
         say what is missing.\n\n",
    );
    for result in &results {
        block.push_str(&format!(
            "--- {} ---\nURL: {}\n{}\n\n",
            result.title, result.url, result.snippet
        ));
    }
    messages.push(json!({ "role": "system", "content": block }));
    log::info!(
        "web context: injected {} results for query '{}'",
        results.len(),
        decision.query
    );
}

#[cfg(test)]
mod tests {
    use super::parse_decision;

    #[test]
    fn reads_a_plain_object() {
        let decision = parse_decision(r#"{"search": true, "query": "computer virus news"}"#)
            .expect("decision");
        assert!(decision.search);
        assert_eq!(decision.query, "computer virus news");
    }

    #[test]
    fn reads_an_object_wrapped_in_prose_or_fences() {
        let raw = "Sure:\n```json\n{\"search\": false, \"query\": \"\"}\n```\n";
        let decision = parse_decision(raw).expect("decision");
        assert!(!decision.search);
        assert!(decision.query.is_empty());
    }

    #[test]
    fn rejects_a_search_without_a_query() {
        assert!(parse_decision(r#"{"search": true, "query": "  "}"#).is_none());
        assert!(parse_decision("no json here").is_none());
    }
}
