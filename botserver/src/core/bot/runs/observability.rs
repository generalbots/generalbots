//! Observability for chat runs (#1679).
//!
//! Spans are OTel-shaped (`gen_ai.*` attributes) and exported as OTLP/HTTP
//! JSON when `OTEL_EXPORTER_OTLP_ENDPOINT` is configured. The dependency-free
//! exporter keeps the offline desktop build intact; a collector receives the
//! same payload an SDK would send. Spans are also appended to `agent_spans`,
//! the existing usage table, so the admin view can aggregate them without a
//! parallel store.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use serde::Serialize;
use sha2::{Digest, Sha256};
use diesel::prelude::*;

#[derive(Debug, Clone)]
struct OpenSpan {
    span_id: String,
    name: String,
    kind: &'static str,
    started_unix_nano: u128,
    attributes: Vec<(String, String)>,
}

#[derive(Debug, Clone, Serialize)]
struct FinishedSpan {
    trace_id: String,
    span_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_span_id: Option<String>,
    name: String,
    kind: i32,
    kind_name: String,
    start_unix_nano: u128,
    end_unix_nano: u128,
    attributes: Vec<(String, String)>,
}

#[derive(Debug, Default)]
struct RunTrace {
    trace_id: String,
    spans: Vec<OpenSpan>,
    finished: Vec<FinishedSpan>,
}

fn traces() -> &'static Mutex<HashMap<String, RunTrace>> {
    static TRACES: OnceLock<Mutex<HashMap<String, RunTrace>>> = OnceLock::new();
    TRACES.get_or_init(|| Mutex::new(HashMap::new()))
}

fn lock_traces() -> std::sync::MutexGuard<'static, HashMap<String, RunTrace>> {
    match traces().lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

fn hex_id(seed: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    hasher
        .finalize()
        .iter()
        .take(8)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn trace_hex(seed: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(seed.as_bytes());
    hasher
        .finalize()
        .iter()
        .take(16)
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn unix_nanos() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0)
}

pub fn start_run(thread_id: &str, run_id: &str, model: Option<&str>) {
    let trace_id = trace_hex(&format!("{thread_id}|{run_id}"));
    let mut guard = lock_traces();
    let entry = guard.entry(run_id.to_string()).or_default();
    entry.trace_id = trace_id;
    entry.spans.push(OpenSpan {
        span_id: hex_id(&format!("{run_id}|run")),
        name: "chat.run".to_string(),
        kind: "run",
        started_unix_nano: unix_nanos(),
        attributes: model
            .map(|model| vec![("gen_ai.request.model".to_string(), model.to_string())])
            .unwrap_or_default(),
    });
}

pub fn start_span(run_id: &str, kind: &'static str, name: &str, attributes: Vec<(String, String)>) {
    let mut guard = lock_traces();
    let entry = guard.entry(run_id.to_string()).or_default();
    let key = format!("{run_id}|{kind}|{name}|{}", entry.spans.len());
    entry.spans.push(OpenSpan {
        span_id: hex_id(&key),
        name: name.to_string(),
        kind,
        started_unix_nano: unix_nanos(),
        attributes,
    });
}

pub fn finish_span(run_id: &str, kind: &'static str, name: &str, extra: Vec<(String, String)>) {
    let mut guard = lock_traces();
    let Some(entry) = guard.get_mut(run_id) else {
        return;
    };
    let trace_id = entry.trace_id.clone();
    let Some(index) = entry
        .spans
        .iter()
        .rposition(|span| span.kind == kind && span.name == name)
    else {
        return;
    };
    let mut span = entry.spans.remove(index);
    span.attributes.extend(extra);
    entry.finished.push(FinishedSpan {
        trace_id,
        span_id: span.span_id,
        parent_span_id: Some(hex_id(&format!("{run_id}|run"))),
        name: span.name,
        kind: 1,
        kind_name: span.kind.to_string(),
        start_unix_nano: span.started_unix_nano,
        end_unix_nano: unix_nanos(),
        attributes: span.attributes,
    });
}

/// Ends the run span, exports the trace and removes it from memory.
pub fn finish_run(run_id: &str, state: &str, pool: Option<&botlib::db_pool::DbPool>) {
    let mut guard = lock_traces();
    let Some(mut entry) = guard.remove(run_id) else {
        return;
    };
    let trace_id = entry.trace_id.clone();
    let mut spans = std::mem::take(&mut entry.finished);
    if let Some(run_span) = entry.spans.pop() {
        spans.push(FinishedSpan {
            trace_id,
            span_id: run_span.span_id,
            parent_span_id: None,
            name: run_span.name,
            kind: 1,
            kind_name: run_span.kind.to_string(),
            start_unix_nano: run_span.started_unix_nano,
            end_unix_nano: unix_nanos(),
            attributes: {
                let mut attributes = run_span.attributes;
                attributes.push(("gb.run.state".to_string(), state.to_string()));
                attributes
            },
        });
    }
    drop(guard);

    if let Some(pool) = pool {
        persist_spans(pool, run_id, &spans);
    }
    export_otlp(&spans);
}

fn persist_spans(pool: &botlib::db_pool::DbPool, run_id: &str, spans: &[FinishedSpan]) {
    let Ok(run_uuid) = uuid::Uuid::parse_str(run_id) else {
        return;
    };
    let mut conn = match pool.get() {
        Ok(conn) => conn,
        Err(_) => return,
    };
    for span in spans {
        let tokens_in = attribute(&span.attributes, "gen_ai.usage.input_tokens")
            .and_then(|value| value.parse::<i32>().ok());
        let tokens_out = attribute(&span.attributes, "gen_ai.usage.output_tokens")
            .and_then(|value| value.parse::<i32>().ok());
        let duration_ms = i64::try_from((span.end_unix_nano - span.start_unix_nano) / 1_000_000)
            .unwrap_or(0);
        let _ = diesel::sql_query(
            "INSERT INTO agent_spans (id, run_id, kind, name, tokens_in, tokens_out, status, started_at, finished_at) \
             VALUES (gen_random_uuid(), $1, $2, $3, $4, $5, 'ok', NOW() - ($6 || ' milliseconds')::interval, NOW())",
        )
        .bind::<diesel::sql_types::Uuid, _>(run_uuid)
        .bind::<diesel::sql_types::Text, _>(&span.kind_name)
        .bind::<diesel::sql_types::Text, _>(&span.name)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Int4>, _>(tokens_in)
        .bind::<diesel::sql_types::Nullable<diesel::sql_types::Int4>, _>(tokens_out)
        .bind::<diesel::sql_types::BigInt, _>(duration_ms)
        .execute(&mut conn);
    }
}

fn attribute<'a>(attributes: &'a [(String, String)], key: &str) -> Option<&'a str> {
    attributes
        .iter()
        .find(|(name, _)| name == key)
        .map(|(_, value)| value.as_str())
}

fn export_otlp(spans: &[FinishedSpan]) {
    let Ok(endpoint) = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") else {
        return;
    };
    let endpoint = endpoint.trim().trim_end_matches('/').to_string();
    if endpoint.is_empty() || spans.is_empty() {
        return;
    }
    let url = if endpoint.ends_with("/v1/traces") {
        endpoint
    } else {
        format!("{endpoint}/v1/traces")
    };
    let payload = serde_json::json!({
        "resourceSpans": [{
            "resource": { "attributes": [
                { "key": "service.name", "value": { "stringValue": "general-bots" } }
            ]},
            "scopeSpans": [{
                "scope": { "name": "gb.chat", "version": "1" },
                "spans": spans.iter().map(|span| {
                    serde_json::json!({
                        "traceId": span.trace_id,
                        "spanId": span.span_id,
                        "name": span.name,
                        "kind": span.kind,
                        "startTimeUnixNano": span.start_unix_nano.to_string(),
                        "endTimeUnixNano": span.end_unix_nano.to_string(),
                        "attributes": span.attributes.iter().map(|(key, value)| {
                            serde_json::json!({
                                "key": key,
                                "value": { "stringValue": value },
                            })
                        }).collect::<Vec<_>>(),
                    })
                }).collect::<Vec<_>>(),
            }],
        }],
    });
    tokio::spawn(async move {
        let client = match reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
        {
            Ok(client) => client,
            Err(error) => {
                log::warn!("OTLP client unavailable: {error}");
                return;
            }
        };
        if let Err(error) = client.post(&url).json(&payload).send().await {
            log::warn!("OTLP export failed: {error}");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::{attribute, hex_id};

    #[test]
    fn ids_are_stable_and_bounded() {
        assert_eq!(hex_id("a"), hex_id("a"));
        assert_eq!(hex_id("a").len(), 16);
    }

    #[test]
    fn attribute_lookup_is_exact() {
        let attributes = vec![("gen_ai.request.model".to_string(), "m".to_string())];
        assert_eq!(attribute(&attributes, "gen_ai.request.model"), Some("m"));
        assert_eq!(attribute(&attributes, "missing"), None);
    }
}
