//! The person's view of every web search and page read, on every engine.
//!
//! A search ("Pesquisou na web: <consulta>") or a read ("Leu a página:
//! <site>") is a step of the answer in the `web` phase, whoever ran it: the
//! native `web_search` / `web_fetch` tools, the Claude API's own search, or a
//! subscription agent's search and fetch tools. Each step has its own id, so
//! five reads are five rows. The query is shown only in the person's own step;
//! logs carry its length and a short hash, never the text.

use std::time::Instant;

use serde_json::json;
use sha2::{Digest, Sha256};

use super::super::event_sink::{self, RunContext};
use super::super::sink::ChannelSink;
use super::super::types::SessionIdentity;
use super::web_text::site_of;

pub(crate) const SEARCH: &str = "web_search";
pub(crate) const FETCH: &str = "web_fetch";
const PHASE: &str = "web";
/// Longest query or address shown in a step.
const DETAIL_CHARS: usize = 160;
/// Result sites listed under one search step.
const SITES_SHOWN: usize = 8;

/// In this edition web search and page reads are always allowed: they are
/// steps of the answer, never permission cards (owner's decision).
pub(crate) const ALWAYS_ON: bool = cfg!(feature = "gbgov");

/// The step name for a web kind sent by an engine (`web_search`,
/// `web_fetch`), or `None` for anything else.
pub(crate) fn kind_of(kind: &str) -> Option<&'static str> {
    match kind {
        SEARCH => Some(SEARCH),
        FETCH => Some(FETCH),
        _ => None,
    }
}

/// What the step shows: the query (redacted, cut) or the site of an address.
pub(crate) fn detail_for(tool: &str, text: &str) -> String {
    let text = text.trim();
    if tool == FETCH {
        if let Some(site) = site_of(text) {
            return site;
        }
    }
    let cut: String = text.chars().take(600).collect();
    botsecurity_crypto::secrets::redact_sensitive_data(&cut)
        .chars()
        .take(DETAIL_CHARS)
        .collect()
}

/// A query as logs may carry it: its length and the first bytes of its hash.
pub(crate) fn query_fingerprint(query: &str) -> String {
    let digest = Sha256::digest(query.as_bytes());
    format!("query_chars={} query_hash={}", query.chars().count(), hex::encode(&digest[..4]))
}

/// The run a step belongs to: the conversation's chat run.
pub(crate) fn run_for(identity: &SessionIdentity<'_>) -> RunContext {
    RunContext::new(&identity.session_id.to_string(), "chat")
        .with_state(identity.state.clone())
        .with_scope(identity.bot_uuid, identity.user_id, None)
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| u64::try_from(d.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or(0)
}

fn new_step_id(tool: &str) -> String {
    format!("{PHASE}:{tool}:{}", uuid::Uuid::new_v4().simple())
}

/// One running web step, finished with [`WebStep::end`].
pub(crate) struct WebStep {
    run: RunContext,
    id: String,
    tool: &'static str,
    detail: String,
    started: Instant,
}

impl WebStep {
    /// Starts a step on `identity`'s chat run; `None` when the channel
    /// carries no run frames.
    pub(crate) async fn begin(identity: &SessionIdentity<'_>, tool: &'static str, text: &str) -> Option<Self> {
        Self::begin_on(identity.sink, run_for(identity), tool, text).await
    }

    pub(crate) async fn begin_on(sink: &dyn ChannelSink, run: RunContext, tool: &'static str, text: &str) -> Option<Self> {
        crate::core::bot::pipeline::turn_progress::touch(&run.thread_id);
        if !event_sink::can_emit(sink) {
            return None;
        }
        let step = Self { run, id: new_step_id(tool), tool, detail: detail_for(tool, text), started: Instant::now() };
        let extra = json!({ "phase": PHASE, "step": tool, "detail": step.detail, "started_at": now_ms() });
        event_sink::step_started_with(sink, &step.run, &step.id, tool, None, extra).await;
        Some(step)
    }

    /// Finishes the step; `sites` are the result sites of a search.
    pub(crate) async fn end(self, sink: &dyn ChannelSink, ok: bool, sites: &[String]) {
        let elapsed = u64::try_from(self.started.elapsed().as_millis()).unwrap_or(u64::MAX);
        let shown: Vec<&String> = sites.iter().take(SITES_SHOWN).collect();
        let extra = json!({
            "phase": PHASE, "step": self.tool, "detail": self.detail,
            "sites": shown, "elapsed_ms": elapsed, "finished_at": now_ms(),
        });
        let status = if ok { "ok" } else { "error" };
        event_sink::step_finished_with(sink, &self.run, &self.id, status, Some(elapsed), None, extra).await;
    }
}

/// A step an engine reports once it already happened (a Codex search, the
/// Claude API's search): started and finished at once.
pub(crate) async fn noted_on(sink: &dyn ChannelSink, run: RunContext, tool: &'static str, text: &str, ok: bool, sites: &[String]) {
    if let Some(step) = WebStep::begin_on(sink, run, tool, text).await {
        step.end(sink, ok, sites).await;
    }
}

pub(crate) async fn noted(identity: &SessionIdentity<'_>, tool: &'static str, text: &str, ok: bool) {
    noted_on(identity.sink, run_for(identity), tool, text, ok, &[]).await;
}

/// A step marker a model client put in its stream
/// (`botllm::claude::web_search::WEB_STEP_MARKER`): the step name, its text, the result sites
/// and whether it worked.
pub(crate) fn parse_marker(chunk: &str) -> Option<(&'static str, String, Vec<String>, bool)> {
    let value: serde_json::Value = serde_json::from_str(chunk.trim()).ok()?;
    let step = value.get(botllm::claude::web_search::WEB_STEP_MARKER)?;
    let tool = kind_of(step.get("tool")?.as_str()?)?;
    let text = step.get("text").and_then(serde_json::Value::as_str).unwrap_or_else(|| { log::debug!("using default for step text"); Default::default() }).to_string();
    let sites = step
        .get("urls")
        .and_then(serde_json::Value::as_array)
        .map(|urls| urls.iter().filter_map(serde_json::Value::as_str).filter_map(site_of).collect::<Vec<_>>())
        .unwrap_or_else(|| { log::debug!("using default for step sites"); Default::default() });
    let mut unique: Vec<String> = Vec::new();
    for site in sites {
        if !unique.contains(&site) {
            unique.push(site);
        }
    }
    let ok = step.get("ok").and_then(serde_json::Value::as_bool).unwrap_or(true);
    Some((tool, text, unique, ok))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_read_shows_its_site_and_a_search_its_query() {
        assert_eq!(detail_for(FETCH, "https://www.example.org/a?cpf=1"), "example.org");
        assert_eq!(detail_for(SEARCH, "  preço do café hoje "), "preço do café hoje");
        assert_eq!(detail_for(SEARCH, &"a".repeat(400)).chars().count(), DETAIL_CHARS);
    }

    #[test]
    fn only_web_kinds_are_web_steps() {
        assert_eq!(kind_of("web_search"), Some(SEARCH));
        assert_eq!(kind_of("web_fetch"), Some(FETCH));
        assert_eq!(kind_of("mcp_tool"), None);
    }

    #[test]
    fn logs_carry_a_fingerprint_not_the_query() {
        let line = query_fingerprint("meu cpf 123");
        assert!(line.starts_with("query_chars=11 query_hash="));
        assert!(!line.contains("cpf"));
        assert_eq!(line, query_fingerprint("meu cpf 123"));
    }

    #[test]
    fn a_marker_gives_the_query_and_unique_sites() {
        let chunk = json!({ botllm::claude::web_search::WEB_STEP_MARKER: {
            "tool": "web_search", "text": "rust 2026",
            "urls": ["https://www.example.org/a", "https://example.org/b", "https://news.example.net/c"],
        }})
        .to_string();
        let (tool, text, sites, ok) = parse_marker(&chunk).unwrap_or((FETCH, String::new(), Vec::new(), false));
        assert_eq!((tool, text.as_str(), ok), (SEARCH, "rust 2026", true));
        assert_eq!(sites, vec!["example.org".to_string(), "news.example.net".to_string()]);
        assert!(parse_marker("plain text").is_none());
        assert!(parse_marker(r#"{"__web_step__": {"tool": "vm_exec"}}"#).is_none());
    }
}
