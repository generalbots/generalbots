use crate::shared::utils::create_tls_client;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesConfig {
    pub url: String,

    pub token: String,

    pub org: String,

    pub bucket: String,

    pub batch_size: usize,

    pub flush_interval_ms: u64,

    pub verify_tls: bool,
}

impl Default for TimeSeriesConfig {
    fn default() -> Self {
        Self {
            url: "http://localhost:8086".to_string(),
            token: String::new(),
            org: "system".to_string(),
            bucket: "metrics".to_string(),
            batch_size: 1000,
            flush_interval_ms: 1000,
            verify_tls: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricPoint {
    pub measurement: String,

    pub tags: HashMap<String, String>,

    pub fields: HashMap<String, FieldValue>,

    pub timestamp: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FieldValue {
    Float(f64),
    Integer(i64),
    UnsignedInteger(u64),
    String(String),
    Boolean(bool),
}

impl MetricPoint {
    pub fn new(measurement: impl Into<String>) -> Self {
        Self {
            measurement: measurement.into(),
            tags: HashMap::new(),
            fields: HashMap::new(),
            timestamp: None,
        }
    }

    pub fn tag(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.tags.insert(key.into(), value.into());
        self
    }

    pub fn field_f64(mut self, key: impl Into<String>, value: f64) -> Self {
        self.fields.insert(key.into(), FieldValue::Float(value));
        self
    }

    pub fn field_i64(mut self, key: impl Into<String>, value: i64) -> Self {
        self.fields.insert(key.into(), FieldValue::Integer(value));
        self
    }

    pub fn field_u64(mut self, key: impl Into<String>, value: u64) -> Self {
        self.fields
            .insert(key.into(), FieldValue::UnsignedInteger(value));
        self
    }

    pub fn field_str(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.fields
            .insert(key.into(), FieldValue::String(value.into()));
        self
    }

    pub fn field_bool(mut self, key: impl Into<String>, value: bool) -> Self {
        self.fields.insert(key.into(), FieldValue::Boolean(value));
        self
    }

    pub fn at(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    pub fn to_line_protocol(&self) -> String {
        let mut line = self.measurement.clone();

        let mut sorted_tags: Vec<_> = self.tags.iter().collect();
        sorted_tags.sort_by_key(|(k, _)| *k);
        for (key, value) in sorted_tags {
            line.push(',');
            line.push_str(&escape_tag_key(key));
            line.push('=');
            line.push_str(&escape_tag_value(value));
        }

        line.push(' ');
        let mut sorted_fields: Vec<_> = self.fields.iter().collect();
        sorted_fields.sort_by_key(|(k, _)| *k);
        let fields_str: Vec<String> = sorted_fields
            .iter()
            .map(|(key, value)| {
                let escaped_key = escape_field_key(key);
                match value {
                    FieldValue::Float(v) => format!("{}={}", escaped_key, v),
                    FieldValue::Integer(v) => format!("{}={}i", escaped_key, v),
                    FieldValue::UnsignedInteger(v) => format!("{}={}u", escaped_key, v),
                    FieldValue::String(v) => {
                        format!("{}=\"{}\"", escaped_key, escape_string_value(v))
                    }
                    FieldValue::Boolean(v) => format!("{}={}", escaped_key, v),
                }
            })
            .collect();
        line.push_str(&fields_str.join(","));

        if let Some(ts) = self.timestamp {
            line.push(' ');
            line.push_str(&ts.timestamp_nanos_opt().unwrap_or(0).to_string());
        }

        line
    }
}

fn escape_tag_key(s: &str) -> String {
    s.replace(',', "\\,")
        .replace('=', "\\=")
        .replace(' ', "\\ ")
}

fn escape_tag_value(s: &str) -> String {
    s.replace(',', "\\,")
        .replace('=', "\\=")
        .replace(' ', "\\ ")
}

fn escape_field_key(s: &str) -> String {
    s.replace(',', "\\,")
        .replace('=', "\\=")
        .replace(' ', "\\ ")
}

fn escape_string_value(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
}

pub struct TimeSeriesClient {
    config: TimeSeriesConfig,
    http_client: reqwest::Client,
    write_buffer: Arc<RwLock<Vec<MetricPoint>>>,
    write_sender: mpsc::Sender<MetricPoint>,
}

impl TimeSeriesClient {
    pub async fn new(config: TimeSeriesConfig) -> Result<Self, TimeSeriesError> {
        let http_client = create_tls_client(Some(30));

        let write_buffer = Arc::new(RwLock::new(Vec::with_capacity(config.batch_size)));
        let (write_sender, write_receiver) = mpsc::channel::<MetricPoint>(10000);

        let client = Self {
            config: config.clone(),
            http_client: http_client.clone(),
            write_buffer: write_buffer.clone(),
            write_sender,
        };

        let buffer_clone = write_buffer.clone();
        let config_clone = config.clone();
        tokio::spawn(async move {
            Self::background_writer(write_receiver, buffer_clone, http_client, config_clone).await;
        });

        Ok(client)
    }

    async fn background_writer(
        mut receiver: mpsc::Receiver<MetricPoint>,
        buffer: Arc<RwLock<Vec<MetricPoint>>>,
        http_client: reqwest::Client,
        config: TimeSeriesConfig,
    ) {
        let mut interval =
            tokio::time::interval(std::time::Duration::from_millis(config.flush_interval_ms));

        loop {
            tokio::select! {
                Some(point) = receiver.recv() => {
                    let mut buf = buffer.write().await;
                    buf.push(point);

                    if buf.len() >= config.batch_size {
                        let points: Vec<MetricPoint> = buf.drain(..).collect();
                        drop(buf);
                        if let Err(e) = Self::flush_points(&http_client, &config, &points).await {
                            log::error!("Failed to flush metrics: {}", e);
                        }
                    }
                }
                _ = interval.tick() => {
                    let mut buf = buffer.write().await;
                    if !buf.is_empty() {
                        let points: Vec<MetricPoint> = buf.drain(..).collect();
                        drop(buf);
                        if let Err(e) = Self::flush_points(&http_client, &config, &points).await {
                            log::error!("Failed to flush metrics: {}", e);
                        }
                    }
                }
            }
        }
    }

    async fn flush_points(
        http_client: &reqwest::Client,
        config: &TimeSeriesConfig,
        points: &[MetricPoint],
    ) -> Result<(), TimeSeriesError> {
        if points.is_empty() {
            return Ok(());
        }

        let line_protocol: String = points
            .iter()
            .map(|p| p.to_line_protocol())
            .collect::<Vec<_>>()
            .join("\n");

        let url = format!(
            "{}/api/v2/write?org={}&bucket={}&precision=ns",
            config.url, config.org, config.bucket
        );

        let response = http_client
            .post(&url)
            .header("Authorization", format!("Token {}", config.token))
            .header("Content-Type", "text/plain; charset=utf-8")
            .body(line_protocol)
            .send()
            .await
            .map_err(|e| TimeSeriesError::WriteError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TimeSeriesError::WriteError(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        log::debug!("Flushed {} metric points to InfluxDB", points.len());
        Ok(())
    }

    pub async fn write_point(&self, point: MetricPoint) -> Result<(), TimeSeriesError> {
        self.write_sender
            .send(point)
            .await
            .map_err(|e| TimeSeriesError::WriteError(e.to_string()))
    }

    pub async fn write_points(&self, points: Vec<MetricPoint>) -> Result<(), TimeSeriesError> {
        for point in points {
            self.write_point(point).await?;
        }
        Ok(())
    }

    pub async fn query(&self, flux_query: &str) -> Result<QueryResult, TimeSeriesError> {
        let url = format!("{}/api/v2/query?org={}", self.config.url, self.config.org);

        let response = self
            .http_client
            .post(&url)
            .header("Authorization", format!("Token {}", self.config.token))
            .header("Accept", "application/csv")
            .header("Content-Type", "application/vnd.flux")
            .body(flux_query.to_string())
            .send()
            .await
            .map_err(|e| TimeSeriesError::QueryError(e.to_string()))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(TimeSeriesError::QueryError(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        let csv_data = response
            .text()
            .await
            .map_err(|e| TimeSeriesError::QueryError(e.to_string()))?;

        Self::parse_csv_result(&csv_data)
    }

    fn parse_csv_result(csv_data: &str) -> Result<QueryResult, TimeSeriesError> {
        let mut result = QueryResult {
            columns: Vec::new(),
            rows: Vec::new(),
        };

        let mut lines = csv_data.lines().peekable();

        while let Some(line) = lines.peek() {
            if line.starts_with('#') || line.is_empty() {
                lines.next();
            } else {
                break;
            }
        }

        if let Some(header_line) = lines.next() {
            result.columns = header_line
                .split(',')
                .map(|s| s.trim().to_string())
                .collect();
        }

        for line in lines {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let values: Vec<serde_json::Value> = line
                .split(',')
                .map(|s| {
                    let trimmed = s.trim();

                    if let Ok(n) = trimmed.parse::<i64>() {
                        serde_json::Value::Number(n.into())
                    } else if let Ok(n) = trimmed.parse::<f64>() {
                        serde_json::json!(n)
                    } else if trimmed == "true" {
                        serde_json::Value::Bool(true)
                    } else if trimmed == "false" {
                        serde_json::Value::Bool(false)
                    } else {
                        serde_json::Value::String(trimmed.to_string())
                    }
                })
                .collect();

            result.rows.push(values);
        }

        Ok(result)
    }

    pub async fn query_range(
        &self,
        measurement: &str,
        start: &str,
        stop: Option<&str>,
        aggregation_window: Option<&str>,
    ) -> Result<QueryResult, TimeSeriesError> {
        let stop_clause = stop.map_or("now()".to_string(), |s| format!("\"{}\"", s));
        let window = aggregation_window.unwrap_or("1m");

        let flux = format!(
            r#"from(bucket: "{}")
  |> range(start: {}, stop: {})
  |> filter(fn: (r) => r._measurement == "{}")
  |> aggregateWindow(every: {}, fn: mean, createEmpty: false)
  |> yield(name: "mean")"#,
            self.config.bucket, start, stop_clause, measurement, window
        );

        self.query(&flux).await
    }

    pub async fn query_last(&self, measurement: &str) -> Result<QueryResult, TimeSeriesError> {
        let flux = format!(
            r#"from(bucket: "{}")
  |> range(start: -1h)
  |> filter(fn: (r) => r._measurement == "{}")
  |> last()"#,
            self.config.bucket, measurement
        );

        self.query(&flux).await
    }

    pub async fn query_stats(
        &self,
        measurement: &str,
        start: &str,
    ) -> Result<QueryResult, TimeSeriesError> {
        let flux = format!(
            r#"from(bucket: "{}")
  |> range(start: {})
  |> filter(fn: (r) => r._measurement == "{}")
  |> group()
  |> reduce(
      identity: {{count: 0.0, sum: 0.0, min: 0.0, max: 0.0}},
      fn: (r, accumulator) => ({{
          count: accumulator.count + 1.0,
          sum: accumulator.sum + r._value,
          min: if accumulator.count == 0.0 then r._value else if r._value < accumulator.min then r._value else accumulator.min,
          max: if accumulator.count == 0.0 then r._value else if r._value > accumulator.max then r._value else accumulator.max
      }})
  )"#,
            self.config.bucket, start, measurement
        );

        self.query(&flux).await
    }

    pub async fn health_check(&self) -> Result<bool, TimeSeriesError> {
        let url = format!("{}/health", self.config.url);

        let response = self
            .http_client
            .get(&url)
            .send()
            .await
            .map_err(|e| TimeSeriesError::ConnectionError(e.to_string()))?;

        Ok(response.status().is_success())
    }
}

pub struct Metrics;

impl Metrics {
    pub fn message(bot_id: &str, channel: &str, direction: &str) -> MetricPoint {
        MetricPoint::new("messages")
            .tag("bot_id", bot_id)
            .tag("channel", channel)
            .tag("direction", direction)
            .field_i64("count", 1)
            .at(Utc::now())
    }

    pub fn response_time(bot_id: &str, duration_ms: f64) -> MetricPoint {
        MetricPoint::new("response_time")
            .tag("bot_id", bot_id)
            .field_f64("duration_ms", duration_ms)
            .at(Utc::now())
    }

    pub fn llm_tokens(
        bot_id: &str,
        model: &str,
        prompt_tokens: i64,
        completion_tokens: i64,
    ) -> MetricPoint {
        MetricPoint::new("llm_tokens")
            .tag("bot_id", bot_id)
            .tag("model", model)
            .field_i64("prompt_tokens", prompt_tokens)
            .field_i64("completion_tokens", completion_tokens)
            .field_i64("total_tokens", prompt_tokens + completion_tokens)
            .at(Utc::now())
    }

    pub fn active_sessions(bot_id: &str, count: i64) -> MetricPoint {
        MetricPoint::new("active_sessions")
            .tag("bot_id", bot_id)
            .field_i64("count", count)
            .at(Utc::now())
    }

    pub fn error(bot_id: &str, error_type: &str, message: &str) -> MetricPoint {
        MetricPoint::new("errors")
            .tag("bot_id", bot_id)
            .tag("error_type", error_type)
            .field_i64("count", 1)
            .field_str("message", message)
            .at(Utc::now())
    }

    pub fn storage_usage(bot_id: &str, bytes_used: u64, file_count: u64) -> MetricPoint {
        MetricPoint::new("storage_usage")
            .tag("bot_id", bot_id)
            .field_u64("bytes_used", bytes_used)
            .field_u64("file_count", file_count)
            .at(Utc::now())
    }

    pub fn api_request(
        endpoint: &str,
        method: &str,
        status_code: i64,
        duration_ms: f64,
    ) -> MetricPoint {
        MetricPoint::new("api_requests")
            .tag("endpoint", endpoint)
            .tag("method", method)
            .field_i64("status_code", status_code)
            .field_f64("duration_ms", duration_ms)
            .field_i64("count", 1)
            .at(Utc::now())
    }

    pub fn system(cpu_percent: f64, memory_percent: f64, disk_percent: f64) -> MetricPoint {
        MetricPoint::new("system_metrics")
            .field_f64("cpu_percent", cpu_percent)
            .field_f64("memory_percent", memory_percent)
            .field_f64("disk_percent", disk_percent)
            .at(Utc::now())
    }
}

#[derive(Debug, Clone)]
pub enum TimeSeriesError {
    ConnectionError(String),
    WriteError(String),
    QueryError(String),
    ConfigError(String),
}

impl std::fmt::Display for TimeSeriesError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeSeriesError::ConnectionError(msg) => write!(f, "Connection error: {}", msg),
            TimeSeriesError::WriteError(msg) => write!(f, "Write error: {}", msg),
            TimeSeriesError::QueryError(msg) => write!(f, "Query error: {}", msg),
            TimeSeriesError::ConfigError(msg) => write!(f, "Config error: {}", msg),
        }
    }
}

impl std::error::Error for TimeSeriesError {}
