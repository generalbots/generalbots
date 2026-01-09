use chrono::{DateTime, Utc};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

fn generate_trace_id() -> String {
    let mut rng = rand::rng();
    let bytes: [u8; 16] = rng.random();
    hex::encode(bytes)
}

fn generate_span_id() -> String {
    let mut rng = rand::rng();
    let bytes: [u8; 8] = rng.random();
    hex::encode(bytes)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceDependency {
    pub parent_service: String,
    pub child_service: String,
    pub call_count: u64,
    pub error_count: u64,
    pub avg_duration_us: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SpanKind {
    Server,
    Client,
    Producer,
    Consumer,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpanStatus {
    Unset,
    Ok,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub trace_flags: u8,
    pub trace_state: HashMap<String, String>,
    pub baggage: HashMap<String, String>,
}

impl TraceContext {
    pub fn new() -> Self {
        Self {
            trace_id: generate_trace_id(),
            span_id: generate_span_id(),
            parent_span_id: None,
            trace_flags: 1,
            trace_state: HashMap::new(),
            baggage: HashMap::new(),
        }
    }

    pub fn child(&self) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: generate_span_id(),
            parent_span_id: Some(self.span_id.clone()),
            trace_flags: self.trace_flags,
            trace_state: self.trace_state.clone(),
            baggage: self.baggage.clone(),
        }
    }

    pub fn to_w3c_traceparent(&self) -> String {
        format!(
            "00-{}-{}-{:02x}",
            self.trace_id, self.span_id, self.trace_flags
        )
    }

    pub fn from_w3c_traceparent(header: &str) -> Option<Self> {
        let parts: Vec<&str> = header.split('-').collect();
        if parts.len() != 4 {
            return None;
        }

        let trace_flags = u8::from_str_radix(parts[3], 16).ok()?;

        Some(Self {
            trace_id: parts[1].to_string(),
            span_id: parts[2].to_string(),
            parent_span_id: None,
            trace_flags,
            trace_state: HashMap::new(),
            baggage: HashMap::new(),
        })
    }

    pub fn to_b3_headers(&self) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("X-B3-TraceId".to_string(), self.trace_id.clone());
        headers.insert("X-B3-SpanId".to_string(), self.span_id.clone());
        if let Some(parent) = &self.parent_span_id {
            headers.insert("X-B3-ParentSpanId".to_string(), parent.clone());
        }
        headers.insert(
            "X-B3-Sampled".to_string(),
            if self.trace_flags & 1 == 1 { "1" } else { "0" }.to_string(),
        );
        headers
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Span {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub operation_name: String,
    pub service_name: String,
    pub kind: SpanKind,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_us: Option<i64>,
    pub status: SpanStatus,
    pub status_message: Option<String>,
    pub attributes: HashMap<String, AttributeValue>,
    pub events: Vec<SpanEvent>,
    pub links: Vec<SpanLink>,
    pub resource: ResourceAttributes,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum AttributeValue {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    StringArray(Vec<String>),
    IntArray(Vec<i64>),
}

impl From<&str> for AttributeValue {
    fn from(s: &str) -> Self {
        AttributeValue::String(s.to_string())
    }
}

impl From<String> for AttributeValue {
    fn from(s: String) -> Self {
        AttributeValue::String(s)
    }
}

impl From<i64> for AttributeValue {
    fn from(i: i64) -> Self {
        AttributeValue::Int(i)
    }
}

impl From<f64> for AttributeValue {
    fn from(f: f64) -> Self {
        AttributeValue::Float(f)
    }
}

impl From<bool> for AttributeValue {
    fn from(b: bool) -> Self {
        AttributeValue::Bool(b)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanEvent {
    pub name: String,
    pub timestamp: DateTime<Utc>,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanLink {
    pub trace_id: String,
    pub span_id: String,
    pub attributes: HashMap<String, AttributeValue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAttributes {
    pub service_name: String,
    pub service_version: String,
    pub service_instance_id: String,
    pub host_name: Option<String>,
    pub host_type: Option<String>,
    pub os_type: Option<String>,
    pub deployment_environment: Option<String>,
    pub custom: HashMap<String, AttributeValue>,
}

impl Default for ResourceAttributes {
    fn default() -> Self {
        Self {
            service_name: "botserver".to_string(),
            service_version: "6.1.0".to_string(),
            service_instance_id: Uuid::new_v4().to_string(),
            host_name: std::env::var("HOSTNAME").ok(),
            host_type: None,
            os_type: Some(std::env::consts::OS.to_string()),
            deployment_environment: std::env::var("DEPLOYMENT_ENV").ok(),
            custom: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub trace_id: String,
    pub spans: Vec<Span>,
    pub root_span_id: Option<String>,
    pub service_count: u32,
    pub span_count: u32,
    pub duration_us: Option<i64>,
    pub has_errors: bool,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceQuery {
    pub service_name: Option<String>,
    pub operation_name: Option<String>,
    pub min_duration_us: Option<i64>,
    pub max_duration_us: Option<i64>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub tags: HashMap<String, String>,
    pub has_errors: Option<bool>,
    pub limit: u32,
    pub offset: u32,
}

impl Default for TraceQuery {
    fn default() -> Self {
        Self {
            service_name: None,
            operation_name: None,
            min_duration_us: None,
            max_duration_us: None,
            start_time: None,
            end_time: None,
            tags: HashMap::new(),
            has_errors: None,
            limit: 100,
            offset: 0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceStatistics {
    pub total_traces: u64,
    pub total_spans: u64,
    pub error_rate: f32,
    pub avg_duration_us: f64,
    pub p50_duration_us: i64,
    pub p90_duration_us: i64,
    pub p95_duration_us: i64,
    pub p99_duration_us: i64,
    pub spans_per_second: f32,
    pub service_breakdown: HashMap<String, ServiceStats>,
    pub operation_breakdown: HashMap<String, OperationStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStats {
    pub span_count: u64,
    pub error_count: u64,
    pub avg_duration_us: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperationStats {
    pub span_count: u64,
    pub error_count: u64,
    pub avg_duration_us: f64,
    pub min_duration_us: i64,
    pub max_duration_us: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SamplingConfig {
    pub strategy: SamplingStrategy,
    pub rate: f32,
    pub max_traces_per_second: u32,
    pub service_overrides: HashMap<String, f32>,
    pub operation_overrides: HashMap<String, f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SamplingStrategy {
    Always,
    Never,
    Probabilistic,
    RateLimiting,
    Adaptive,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            strategy: SamplingStrategy::Probabilistic,
            rate: 0.1,
            max_traces_per_second: 100,
            service_overrides: HashMap::new(),
            operation_overrides: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExporterConfig {
    pub exporter_type: ExporterType,
    pub endpoint: String,
    pub headers: HashMap<String, String>,
    pub batch_size: u32,
    pub flush_interval_ms: u32,
    pub timeout_ms: u32,
    pub compression: bool,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExporterType {
    Otlp,
    Jaeger,
    Zipkin,
    Console,
    None,
}

impl Default for ExporterConfig {
    fn default() -> Self {
        Self {
            exporter_type: ExporterType::Otlp,
            endpoint: "http://localhost:4317".to_string(),
            headers: HashMap::new(),
            batch_size: 512,
            flush_interval_ms: 5000,
            timeout_ms: 10000,
            compression: true,
            enabled: true,
        }
    }
}

pub struct SpanBuilder {
    operation_name: String,
    service_name: String,
    kind: SpanKind,
    parent_context: Option<TraceContext>,
    attributes: HashMap<String, AttributeValue>,
    links: Vec<SpanLink>,
    start_time: Option<DateTime<Utc>>,
}

impl SpanBuilder {
    pub fn new(operation_name: &str) -> Self {
        Self {
            operation_name: operation_name.to_string(),
            service_name: "botserver".to_string(),
            kind: SpanKind::Internal,
            parent_context: None,
            attributes: HashMap::new(),
            links: Vec::new(),
            start_time: None,
        }
    }

    pub fn with_service_name(mut self, name: &str) -> Self {
        self.service_name = name.to_string();
        self
    }

    pub fn with_kind(mut self, kind: SpanKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn with_parent(mut self, context: TraceContext) -> Self {
        self.parent_context = Some(context);
        self
    }

    pub fn with_attribute<V: Into<AttributeValue>>(mut self, key: &str, value: V) -> Self {
        self.attributes.insert(key.to_string(), value.into());
        self
    }

    pub fn with_link(mut self, trace_id: &str, span_id: &str) -> Self {
        self.links.push(SpanLink {
            trace_id: trace_id.to_string(),
            span_id: span_id.to_string(),
            attributes: HashMap::new(),
        });
        self
    }

    pub fn with_start_time(mut self, time: DateTime<Utc>) -> Self {
        self.start_time = Some(time);
        self
    }

    pub fn start(self, tracer: &DistributedTracingService) -> ActiveSpan {
        let context = match self.parent_context {
            Some(parent) => parent.child(),
            None => TraceContext::new(),
        };

        let start_time = self.start_time.unwrap_or_else(Utc::now);

        let span = Span {
            trace_id: context.trace_id.clone(),
            span_id: context.span_id.clone(),
            parent_span_id: context.parent_span_id.clone(),
            operation_name: self.operation_name,
            service_name: self.service_name,
            kind: self.kind,
            start_time,
            end_time: None,
            duration_us: None,
            status: SpanStatus::Unset,
            status_message: None,
            attributes: self.attributes,
            events: Vec::new(),
            links: self.links,
            resource: ResourceAttributes::default(),
        };

        ActiveSpan {
            span,
            context,
            tracer: tracer.clone(),
        }
    }
}

pub struct ActiveSpan {
    span: Span,
    context: TraceContext,
    tracer: DistributedTracingService,
}

impl ActiveSpan {
    pub fn context(&self) -> &TraceContext {
        &self.context
    }

    pub fn set_attribute<V: Into<AttributeValue>>(&mut self, key: &str, value: V) {
        self.span.attributes.insert(key.to_string(), value.into());
    }

    pub fn add_event(&mut self, name: &str) {
        self.span.events.push(SpanEvent {
            name: name.to_string(),
            timestamp: Utc::now(),
            attributes: HashMap::new(),
        });
    }

    pub fn add_event_with_attributes(&mut self, name: &str, attributes: HashMap<String, AttributeValue>) {
        self.span.events.push(SpanEvent {
            name: name.to_string(),
            timestamp: Utc::now(),
            attributes,
        });
    }

    pub fn set_status(&mut self, status: SpanStatus, message: Option<&str>) {
        self.span.status = status;
        self.span.status_message = message.map(|s| s.to_string());
    }

    pub fn record_error(&mut self, error: &str) {
        self.span.status = SpanStatus::Error;
        self.span.status_message = Some(error.to_string());

        let mut attrs = HashMap::new();
        attrs.insert("exception.message".to_string(), AttributeValue::String(error.to_string()));
        attrs.insert("exception.type".to_string(), AttributeValue::String("Error".to_string()));

        self.span.events.push(SpanEvent {
            name: "exception".to_string(),
            timestamp: Utc::now(),
            attributes: attrs,
        });
    }

    pub async fn end(mut self) {
        let end_time = Utc::now();
        self.span.end_time = Some(end_time);
        self.span.duration_us = Some((end_time - self.span.start_time).num_microseconds().unwrap_or(0));

        if self.span.status == SpanStatus::Unset {
            self.span.status = SpanStatus::Ok;
        }

        self.tracer.record_span(self.span).await;
    }
}

#[derive(Clone)]
pub struct DistributedTracingService {
    spans: Arc<RwLock<HashMap<String, Vec<Span>>>>,
    sampling_config: Arc<RwLock<SamplingConfig>>,
    exporter_config: Arc<RwLock<ExporterConfig>>,
    resource: Arc<ResourceAttributes>,
    span_buffer: Arc<RwLock<Vec<Span>>>,
}

impl DistributedTracingService {
    pub fn new() -> Self {
        Self {
            spans: Arc::new(RwLock::new(HashMap::new())),
            sampling_config: Arc::new(RwLock::new(SamplingConfig::default())),
            exporter_config: Arc::new(RwLock::new(ExporterConfig::default())),
            resource: Arc::new(ResourceAttributes::default()),
            span_buffer: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub fn with_resource(mut self, resource: ResourceAttributes) -> Self {
        self.resource = Arc::new(resource);
        self
    }

    pub fn span(&self, operation_name: &str) -> SpanBuilder {
        SpanBuilder::new(operation_name)
            .with_service_name(&self.resource.service_name)
    }

    pub async fn should_sample(&self, trace_id: &str, operation_name: &str) -> bool {
        let config = self.sampling_config.read().await;

        if let Some(rate) = config.operation_overrides.get(operation_name) {
            return self.should_sample_with_rate(*rate, trace_id);
        }

        match config.strategy {
            SamplingStrategy::Always => true,
            SamplingStrategy::Never => false,
            SamplingStrategy::Probabilistic => self.should_sample_with_rate(config.rate, trace_id),
            SamplingStrategy::RateLimiting | SamplingStrategy::Adaptive => {
                self.should_sample_with_rate(config.rate, trace_id)
            }
        }
    }

    fn should_sample_with_rate(&self, rate: f32, trace_id: &str) -> bool {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        trace_id.hash(&mut hasher);
        let hash = hasher.finish();
        let normalized = (hash as f64) / (u64::MAX as f64);
        normalized < (rate as f64)
    }

    pub async fn record_span(&self, mut span: Span) {
        span.resource = (*self.resource).clone();

        let mut spans = self.spans.write().await;
        spans
            .entry(span.trace_id.clone())
            .or_default()
            .push(span.clone());

        let mut buffer = self.span_buffer.write().await;
        buffer.push(span);

        let config = self.exporter_config.read().await;
        if buffer.len() >= config.batch_size as usize {
            let spans_to_export: Vec<Span> = buffer.drain(..).collect();
            drop(buffer);
            drop(config);
            self.export_spans(spans_to_export).await;
        }
    }

    async fn export_spans(&self, spans: Vec<Span>) {
        let config = self.exporter_config.read().await;

        if !config.enabled || spans.is_empty() {
            return;
        }

        match config.exporter_type {
            ExporterType::Console => {
                for span in &spans {
                    log::debug!(
                        "TRACE [{}] {} {} {}us",
                        span.trace_id,
                        span.operation_name,
                        match span.status {
                            SpanStatus::Ok => "OK",
                            SpanStatus::Error => "ERROR",
                            SpanStatus::Unset => "UNSET",
                        },
                        span.duration_us.unwrap_or(0)
                    );
                }
            }
            ExporterType::Otlp => {
                log::debug!("Would export {} spans to OTLP endpoint: {}", spans.len(), config.endpoint);
            }
            ExporterType::Jaeger => {
                log::debug!("Would export {} spans to Jaeger endpoint: {}", spans.len(), config.endpoint);
            }
            ExporterType::Zipkin => {
                log::debug!("Would export {} spans to Zipkin endpoint: {}", spans.len(), config.endpoint);
            }
            ExporterType::None => {}
        }
    }

    pub async fn get_trace(&self, trace_id: &str) -> Option<Trace> {
        let spans = self.spans.read().await;
        let trace_spans = spans.get(trace_id)?;

        if trace_spans.is_empty() {
            return None;
        }

        let root_span = trace_spans.iter().find(|s| s.parent_span_id.is_none());

        let services: std::collections::HashSet<_> = trace_spans.iter().map(|s| &s.service_name).collect();
        let has_errors = trace_spans.iter().any(|s| s.status == SpanStatus::Error);

        let start_time = trace_spans.iter().map(|s| s.start_time).min();
        let end_time = trace_spans.iter().filter_map(|s| s.end_time).max();

        let duration_us = match (start_time, end_time) {
            (Some(start), Some(end)) => Some((end - start).num_microseconds().unwrap_or(0)),
            _ => None,
        };

        Some(Trace {
            trace_id: trace_id.to_string(),
            spans: trace_spans.clone(),
            root_span_id: root_span.map(|s| s.span_id.clone()),
            service_count: services.len() as u32,
            span_count: trace_spans.len() as u32,
            duration_us,
            has_errors,
            start_time,
            end_time,
        })
    }

    pub async fn query_traces(&self, query: TraceQuery) -> Vec<Trace> {
        let spans = self.spans.read().await;
        let mut traces: Vec<Trace> = Vec::new();

        for (trace_id, trace_spans) in spans.iter() {
            let matches = trace_spans.iter().any(|span| {
                let mut matches = true;

                if let Some(ref service) = query.service_name {
                    matches = matches && &span.service_name == service;
                }

                if let Some(ref operation) = query.operation_name {
                    matches = matches && &span.operation_name == operation;
                }

                if let Some(min_duration) = query.min_duration_us {
                    matches = matches && span.duration_us.unwrap_or(0) >= min_duration;
                }

                if let Some(max_duration) = query.max_duration_us {
                    matches = matches && span.duration_us.unwrap_or(0) <= max_duration;
                }

                if let Some(start_time) = query.start_time {
                    matches = matches && span.start_time >= start_time;
                }

                if let Some(end_time) = query.end_time {
                    matches = matches && span.start_time <= end_time;
                }

                if let Some(has_errors) = query.has_errors {
                    if has_errors {
                        matches = matches && span.status == SpanStatus::Error;
                    }
                }

                matches
            });

            if matches {
                if let Some(trace) = self.build_trace(trace_id, trace_spans) {
                    traces.push(trace);
                }
            }
        }

        traces.sort_by(|a, b| b.start_time.cmp(&a.start_time));

        traces
            .into_iter()
            .skip(query.offset as usize)
            .take(query.limit as usize)
            .collect()
    }

    fn build_trace(&self, trace_id: &str, trace_spans: &[Span]) -> Option<Trace> {
        if trace_spans.is_empty() {
            return None;
        }

        let root_span = trace_spans.iter().find(|s| s.parent_span_id.is_none());
        let services: std::collections::HashSet<_> = trace_spans.iter().map(|s| &s.service_name).collect();
        let has_errors = trace_spans.iter().any(|s| s.status == SpanStatus::Error);

        let start_time = trace_spans.iter().map(|s| s.start_time).min();
        let end_time = trace_spans.iter().filter_map(|s| s.end_time).max();

        let duration_us = match (start_time, end_time) {
            (Some(start), Some(end)) => Some((end - start).num_microseconds().unwrap_or(0)),
            _ => None,
        };

        Some(Trace {
            trace_id: trace_id.to_string(),
            spans: trace_spans.to_vec(),
            root_span_id: root_span.map(|s| s.span_id.clone()),
            service_count: services.len() as u32,
            span_count: trace_spans.len() as u32,
            duration_us,
            has_errors,
            start_time,
            end_time,
        })
    }

    pub async fn get_statistics(&self, duration_hours: u32) -> TraceStatistics {
        let spans = self.spans.read().await;
        let cutoff = Utc::now() - chrono::Duration::hours(duration_hours as i64);

        let mut total_spans: u64 = 0;
        let mut total_duration: i64 = 0;
        let mut error_count: u64 = 0;
        let mut durations: Vec<i64> = Vec::new();
        let mut service_stats: HashMap<String, ServiceStats> = HashMap::new();
        let mut operation_stats: HashMap<String, OperationStats> = HashMap::new();

        for trace_spans in spans.values() {
            for span in trace_spans {
                if span.start_time < cutoff {
                    continue;
                }

                total_spans += 1;
                let duration = span.duration_us.unwrap_or(0);
                total_duration += duration;
                durations.push(duration);

                if span.status == SpanStatus::Error {
                    error_count += 1;
                }

                let service_entry = service_stats
                    .entry(span.service_name.clone())
                    .or_insert(ServiceStats {
                        span_count: 0,
                        error_count: 0,
                        avg_duration_us: 0.0,
                    });
                service_entry.span_count += 1;
                if span.status == SpanStatus::Error {
                    service_entry.error_count += 1;
                }

                let op_entry = operation_stats
                    .entry(span.operation_name.clone())
                    .or_insert(OperationStats {
                        span_count: 0,
                        error_count: 0,
                        avg_duration_us: 0.0,
                        min_duration_us: i64::MAX,
                        max_duration_us: 0,
                    });
                op_entry.span_count += 1;
                if span.status == SpanStatus::Error {
                    op_entry.error_count += 1;
                }
                op_entry.min_duration_us = op_entry.min_duration_us.min(duration);
                op_entry.max_duration_us = op_entry.max_duration_us.max(duration);
            }
        }

        durations.sort();

        let p50 = self.percentile(&durations, 50);
        let p90 = self.percentile(&durations, 90);
        let p95 = self.percentile(&durations, 95);
        let p99 = self.percentile(&durations, 99);

        let avg_duration = if total_spans > 0 {
            total_duration as f64 / total_spans as f64
        } else {
            0.0
        };

        let error_rate = if total_spans > 0 {
            error_count as f32 / total_spans as f32
        } else {
            0.0
        };

        let spans_per_second = if duration_hours > 0 {
            total_spans as f32 / (duration_hours as f32 * 3600.0)
        } else {
            0.0
        };

        TraceStatistics {
            total_traces: spans.len() as u64,
            total_spans,
            error_rate,
            avg_duration_us: avg_duration,
            p50_duration_us: p50,
            p90_duration_us: p90,
            p95_duration_us: p95,
            p99_duration_us: p99,
            spans_per_second,
            service_breakdown: service_stats,
            operation_breakdown: operation_stats,
        }
    }

    pub async fn get_service_dependencies(&self) -> Vec<ServiceDependency> {
        let spans = self.spans.read().await;
        let mut dependencies: HashMap<(String, String), ServiceDependency> = HashMap::new();

        for trace_spans in spans.values() {
            let span_map: HashMap<&str, &Span> = trace_spans
                .iter()
                .map(|s| (s.span_id.as_str(), s))
                .collect();

            for span in trace_spans {
                if let Some(parent_id) = &span.parent_span_id {
                    if let Some(parent_span) = span_map.get(parent_id.as_str()) {
                        if parent_span.service_name != span.service_name {
                            let key = (parent_span.service_name.clone(), span.service_name.clone());
                            let dep = dependencies.entry(key).or_insert(ServiceDependency {
                                parent_service: parent_span.service_name.clone(),
                                child_service: span.service_name.clone(),
                                call_count: 0,
                                error_count: 0,
                                avg_duration_us: 0.0,
                            });
                            dep.call_count += 1;
                            if span.status == SpanStatus::Error {
                                dep.error_count += 1;
                            }
                        }
                    }
                }
            }
        }

        dependencies.into_values().collect()
    }

    pub async fn update_sampling_config(&self, config: SamplingConfig) {
        let mut current = self.sampling_config.write().await;
        *current = config;
    }

    pub async fn update_exporter_config(&self, config: ExporterConfig) {
        let mut current = self.exporter_config.write().await;
        *current = config;
    }

    pub async fn flush(&self) {
        let mut buffer = self.span_buffer.write().await;
        let spans_to_export: Vec<Span> = buffer.drain(..).collect();
        drop(buffer);

        if spans_to_export.is_empty() {
            return;
        }

        let exporter_config = self.exporter_config.read().await;
        if exporter_config.enabled {
            for span in spans_to_export {
                tracing::debug!("Exporting span: {} ({})", span.operation_name, span.span_id);
                let _ = span;
            }
        }
    }

    fn percentile(&self, sorted_data: &[i64], p: u8) -> i64 {
        if sorted_data.is_empty() {
            return 0;
        }
        let idx = ((p as f64 / 100.0) * (sorted_data.len() as f64 - 1.0)).round() as usize;
        sorted_data[idx.min(sorted_data.len() - 1)]
    }
}
