//! Unit tests for the tracing serialisers, id generation, sampling decisions
//! and the W3C/B3 context propagation contract.
//!
//! Everything here is pure: no exporter is enabled by default, so `end()` and
//! `record_span()` only touch in-memory maps.

use super::*;
use serde_json::json;

fn span_with(kind: SpanKind, status: SpanStatus) -> Span {
    Span {
        trace_id: "0af7651916cd43dd8448eb211c80319c".to_string(),
        span_id: "b7ad6b7169203331".to_string(),
        parent_span_id: None,
        operation_name: "op".to_string(),
        service_name: "botserver".to_string(),
        kind,
        start_time: Utc::now(),
        end_time: None,
        duration_us: Some(1200),
        status,
        status_message: None,
        attributes: HashMap::new(),
        events: Vec::new(),
        links: Vec::new(),
        resource: ResourceAttributes::default(),
    }
}

fn resource_with_host(host: Option<&str>) -> ResourceAttributes {
    ResourceAttributes {
        host_name: host.map(|h| h.to_string()),
        ..ResourceAttributes::default()
    }
}

#[test]
fn attr_to_otlp_value_uses_the_exact_otlp_keys() {
    let cases: Vec<(AttributeValue, serde_json::Value)> = vec![
        (AttributeValue::String("a".into()), json!({"stringValue": "a"})),
        // OTLP encodes int64 as a decimal *string*, never as a JSON number.
        (AttributeValue::Int(-7), json!({"intValue": "-7"})),
        (AttributeValue::Float(1.5), json!({"doubleValue": 1.5})),
        (AttributeValue::Bool(true), json!({"boolValue": true})),
        (
            AttributeValue::StringArray(vec!["a".into(), "b".into()]),
            json!({"arrayValue": {"values": [{"stringValue": "a"}, {"stringValue": "b"}]}}),
        ),
        (
            AttributeValue::IntArray(vec![1, 2]),
            json!({"arrayValue": {"values": [{"intValue": "1"}, {"intValue": "2"}]}}),
        ),
    ];

    for (value, expected) in cases {
        assert_eq!(attr_to_otlp_value(&value), expected);
    }
}

#[test]
fn empty_arrays_still_serialise_an_array_value() {
    assert_eq!(
        attr_to_otlp_value(&AttributeValue::StringArray(Vec::new())),
        json!({"arrayValue": {"values": []}})
    );
    assert_eq!(
        attr_to_otlp_value(&AttributeValue::IntArray(Vec::new())),
        json!({"arrayValue": {"values": []}})
    );
}

#[test]
fn jaeger_tags_always_carry_the_three_standard_tags() {
    let span = span_with(SpanKind::Server, SpanStatus::Ok);
    let tags = jaeger_tags(&span);
    assert_eq!(tags.len(), 3);
    assert_eq!(tags[0], json!({"key": "span.kind", "type": "string", "value": "server"}));
    assert_eq!(tags[1], json!({"key": "service.name", "type": "string", "value": "botserver"}));
    assert_eq!(tags[2], json!({"key": "status", "type": "string", "value": "Ok"}));
}

#[test]
fn jaeger_tags_add_message_and_host_only_when_set() {
    let mut span = span_with(SpanKind::Client, SpanStatus::Error);
    span.status_message = Some("boom".into());
    span.resource = resource_with_host(Some("node-1"));

    let tags = jaeger_tags(&span);
    let keys: Vec<&str> = tags.iter().filter_map(|t| t["key"].as_str()).collect();
    assert_eq!(
        keys,
        vec!["span.kind", "service.name", "status", "status.message", "host.name"]
    );
    assert_eq!(tags[3]["value"], json!("boom"));
    assert_eq!(tags[4]["value"], json!("node-1"));
}

#[test]
fn jaeger_tags_flatten_arrays_and_keep_attribute_types() {
    let mut span = span_with(SpanKind::Producer, SpanStatus::Unset);
    span.attributes.insert("strings".into(), AttributeValue::StringArray(vec!["a".into(), "b".into()]));
    span.attributes.insert("ints".into(), AttributeValue::IntArray(vec![3, 4]));
    span.attributes.insert("flag".into(), AttributeValue::Bool(false));
    span.attributes.insert("ratio".into(), AttributeValue::Float(0.5));
    span.attributes.insert("count".into(), AttributeValue::Int(9));

    let tags = jaeger_tags(&span);
    let find = |key: &str| {
        tags.iter()
            .find(|t| t["key"] == json!(key))
            .cloned()
            .expect("tag present")
    };
    assert_eq!(find("strings"), json!({"key": "strings", "type": "string", "value": "a,b"}));
    assert_eq!(find("ints"), json!({"key": "ints", "type": "int64", "value": "3,4"}));
    assert_eq!(find("flag"), json!({"key": "flag", "type": "bool", "value": false}));
    assert_eq!(find("ratio"), json!({"key": "ratio", "type": "float64", "value": 0.5}));
    assert_eq!(find("count"), json!({"key": "count", "type": "int64", "value": 9}));
}

#[test]
fn an_attribute_colliding_with_span_kind_is_emitted_after_the_standard_tag() {
    let mut span = span_with(SpanKind::Server, SpanStatus::Ok);
    span.attributes.insert("span.kind".into(), AttributeValue::String("custom".into()));

    let tags = jaeger_tags(&span);
    assert_eq!(tags[0]["value"], json!("server"));
    let last = tags.last().expect("attributes appended");
    assert_eq!(last["key"], json!("span.kind"));
    assert_eq!(last["value"], json!("custom"));
}

#[test]
fn zipkin_tags_are_a_map_with_the_same_contract() {
    let mut span = span_with(SpanKind::Consumer, SpanStatus::Ok);
    let tags = zipkin_tags(&span);
    assert_eq!(tags.get("span.kind"), Some(&json!("consumer")));
    assert_eq!(tags.get("status"), Some(&json!("Ok")));
    assert_eq!(tags.len(), 2, "no status.message / host.name when unset");

    span.status_message = Some("slow".into());
    span.resource = resource_with_host(Some("node-2"));
    span.attributes.insert("ints".into(), AttributeValue::IntArray(vec![5, 6]));
    span.attributes.insert("count".into(), AttributeValue::Int(5));

    let tags = zipkin_tags(&span);
    assert_eq!(tags.get("status.message"), Some(&json!("slow")));
    assert_eq!(tags.get("host.name"), Some(&json!("node-2")));
    assert_eq!(tags.get("ints"), Some(&json!("5,6")));
    assert_eq!(tags.get("count"), Some(&json!(5)));
}

#[test]
fn generated_ids_have_the_w3c_shape_and_differ() {
    let trace_a = generate_trace_id();
    let trace_b = generate_trace_id();
    assert_eq!(trace_a.len(), 32);
    assert!(trace_a.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    assert_ne!(trace_a, trace_b);

    let span_a = generate_span_id();
    let span_b = generate_span_id();
    assert_eq!(span_a.len(), 16);
    assert!(span_a.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    assert_ne!(span_a, span_b);
}

#[test]
fn sampling_config_default_is_probabilistic_at_one_in_ten() {
    let config = SamplingConfig::default();
    assert_eq!(config.strategy, SamplingStrategy::Probabilistic);
    assert!((config.rate - 0.1).abs() < f32::EPSILON);
    assert_eq!(config.max_traces_per_second, 100);
    assert!(config.service_overrides.is_empty());
    assert!(config.operation_overrides.is_empty());
}

#[test]
fn exporter_config_default_is_opt_in() {
    let config = ExporterConfig::default();
    assert!(!config.enabled);
    assert_eq!(config.exporter_type, ExporterType::None);
    assert!(config.endpoint.is_empty());
    assert_eq!(config.batch_size, 512);
}

#[test]
fn resource_attributes_default_identifies_the_service() {
    let resource = ResourceAttributes::default();
    assert_eq!(resource.service_name, "botserver");
    assert_eq!(resource.service_version, env!("CARGO_PKG_VERSION"));
    assert!(!resource.service_instance_id.is_empty());
    assert_eq!(resource.os_type.as_deref(), Some(std::env::consts::OS));
}

#[tokio::test]
async fn sampling_with_rate_zero_never_and_one_always() {
    let service = DistributedTracingService::new();
    let trace_id = "0af7651916cd43dd8448eb211c80319c";
    assert!(!service.should_sample_with_rate(0.0, trace_id));
    assert!(service.should_sample_with_rate(1.0, trace_id));
}

#[tokio::test]
async fn sampling_the_same_trace_id_is_deterministic() {
    let service = DistributedTracingService::new();
    let trace_id = "0af7651916cd43dd8448eb211c80319c";
    let first = service.should_sample_with_rate(0.5, trace_id);
    for _ in 0..5 {
        assert_eq!(service.should_sample_with_rate(0.5, trace_id), first);
    }
}

#[tokio::test]
async fn always_and_never_ignore_the_rate() {
    let service = DistributedTracingService::new();
    let trace_id = "0af7651916cd43dd8448eb211c80319c";

    {
        let mut config = service.sampling_config.write().await;
        config.strategy = SamplingStrategy::Always;
        config.rate = 0.0;
    }
    assert!(service.should_sample(trace_id, "op").await);

    {
        let mut config = service.sampling_config.write().await;
        config.strategy = SamplingStrategy::Never;
        config.rate = 1.0;
    }
    assert!(!service.should_sample(trace_id, "op").await);
}

#[tokio::test]
async fn an_operation_override_wins_over_the_strategy() {
    let service = DistributedTracingService::new();
    let trace_id = "0af7651916cd43dd8448eb211c80319c";

    {
        let mut config = service.sampling_config.write().await;
        config.strategy = SamplingStrategy::Never;
        config.operation_overrides.insert("health_check".to_string(), 1.0);
    }
    assert!(service.should_sample(trace_id, "health_check").await);
    assert!(!service.should_sample(trace_id, "other_op").await);
}

#[tokio::test]
async fn probabilistic_and_rate_limiting_share_the_rate_decision() {
    let service = DistributedTracingService::new();
    let trace_id = "0af7651916cd43dd8448eb211c80319c";

    for strategy in [SamplingStrategy::Probabilistic, SamplingStrategy::RateLimiting] {
        let mut config = service.sampling_config.write().await;
        config.strategy = strategy;
        config.rate = 0.0;
        drop(config);
        assert!(!service.should_sample(trace_id, "op").await);
    }
}

#[test]
fn w3c_traceparent_round_trips() {
    let context = TraceContext::new();
    let header = context.to_w3c_traceparent();
    assert_eq!(header.len(), 2 + 1 + 32 + 1 + 16 + 1 + 2);
    assert!(header.starts_with("00-"));

    let parsed = TraceContext::from_w3c_traceparent(&header).expect("parsed");
    assert_eq!(parsed.trace_id, context.trace_id);
    assert_eq!(parsed.span_id, context.span_id);
    assert_eq!(parsed.trace_flags, 1);
    assert_eq!(parsed.parent_span_id, None);
}

#[test]
fn a_malformed_traceparent_is_rejected_not_guessed() {
    for header in ["", "00", "00-abc-def", "00-abc-def-zz", "a-b-c-d-e"] {
        assert!(
            TraceContext::from_w3c_traceparent(header).is_none(),
            "header={header:?}"
        );
    }
}

#[test]
fn b3_headers_carry_the_parent_only_for_child_contexts() {
    let root = TraceContext::new();
    let headers = root.to_b3_headers();
    assert_eq!(headers.get("X-B3-TraceId"), Some(&root.trace_id));
    assert_eq!(headers.get("X-B3-SpanId"), Some(&root.span_id));
    assert_eq!(headers.get("X-B3-Sampled"), Some(&"1".to_string()));
    assert!(!headers.contains_key("X-B3-ParentSpanId"));

    let child = root.child();
    let headers = child.to_b3_headers();
    assert_eq!(headers.get("X-B3-ParentSpanId"), Some(&root.span_id));
    assert_eq!(child.trace_id, root.trace_id);
    assert_ne!(child.span_id, root.span_id);
}

#[tokio::test]
async fn an_active_span_records_attributes_events_and_errors() {
    let service = DistributedTracingService::new();
    let mut active = service.span("op").start(&service);

    active.set_attribute("http.method", "GET");
    active.set_attribute("http.status", 200i64);
    active.add_event("cache_hit");
    active.set_status(SpanStatus::Ok, None);
    let context = active.context().clone();
    active.end().await;

    let trace = service.get_trace(&context.trace_id).await.expect("recorded");
    assert_eq!(trace.span_count, 1);
    assert_eq!(trace.root_span_id.as_deref(), Some(context.span_id.as_str()));
    let span = &trace.spans[0];
    assert_eq!(span.operation_name, "op");
    assert_eq!(span.status, SpanStatus::Ok);
    assert!(span.duration_us.is_some());
    assert_eq!(span.events.len(), 1);
    assert_eq!(span.events[0].name, "cache_hit");
    assert_eq!(span.attributes.len(), 2);
}

#[tokio::test]
async fn record_error_marks_the_span_and_attaches_an_exception_event() {
    let service = DistributedTracingService::new();
    let mut active = service.span("failing").start(&service);
    active.record_error("kaboom");
    let context = active.context().clone();
    active.end().await;

    let trace = service.get_trace(&context.trace_id).await.expect("recorded");
    assert!(trace.has_errors);
    let span = &trace.spans[0];
    assert_eq!(span.status, SpanStatus::Error);
    assert_eq!(span.status_message.as_deref(), Some("kaboom"));
    assert_eq!(span.events[0].name, "exception");
}

#[tokio::test]
async fn an_unknown_trace_id_returns_none() {
    let service = DistributedTracingService::new();
    assert!(service.get_trace("does-not-exist").await.is_none());
}
