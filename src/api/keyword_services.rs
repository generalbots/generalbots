use crate::shared::state::AppState;
use anyhow::{anyhow, Result};
use axum::{
    extract::{Json, Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use chrono::{Datelike, NaiveDateTime, Timelike};
use num_format::{Locale, ToFormattedString};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// ============================================================================
// Data Structures
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatRequest {
    pub value: String,
    pub pattern: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormatResponse {
    pub formatted: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherRequest {
    pub location: String,
    pub units: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherResponse {
    pub location: String,
    pub temperature: f64,
    pub description: String,
    pub humidity: u32,
    pub wind_speed: f64,
    pub units: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailRequest {
    pub to: Vec<String>,
    pub subject: String,
    pub body: String,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
    pub attachments: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailResponse {
    pub message_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequest {
    pub title: String,
    pub description: Option<String>,
    pub assignee: Option<String>,
    pub due_date: Option<String>,
    pub priority: Option<String>,
    pub labels: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub task_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub kb_name: Option<String>,
    pub limit: Option<usize>,
    pub threshold: Option<f32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub content: String,
    pub source: String,
    pub score: f32,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    pub results: Vec<SearchResult>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRequest {
    pub key: String,
    pub value: Option<serde_json::Value>,
    pub ttl: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryResponse {
    pub key: String,
    pub value: Option<serde_json::Value>,
    pub exists: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDocumentRequest {
    pub content: String,
    pub format: String,
    pub extract_entities: Option<bool>,
    pub extract_keywords: Option<bool>,
    pub summarize: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessDocumentResponse {
    pub text: String,
    pub entities: Option<Vec<Entity>>,
    pub keywords: Option<Vec<String>>,
    pub summary: Option<String>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub text: String,
    pub entity_type: String,
    pub confidence: f32,
}

// ============================================================================
// Service Layer
// ============================================================================

pub struct KeywordService {
    state: Arc<AppState>,
}

impl KeywordService {
    pub fn new(state: Arc<AppState>) -> Self {
        Self { state }
    }

    // ------------------------------------------------------------------------
    // Format Service
    // ------------------------------------------------------------------------

    pub async fn format_value(&self, req: FormatRequest) -> Result<FormatResponse> {
        let formatted = if let Ok(num) = req.value.parse::<f64>() {
            self.format_number(num, &req.pattern)?
        } else if let Ok(dt) = NaiveDateTime::parse_from_str(&req.value, "%Y-%m-%d %H:%M:%S") {
            self.format_date(dt, &req.pattern)?
        } else {
            self.format_text(&req.value, &req.pattern)?
        };

        Ok(FormatResponse { formatted })
    }

    fn format_number(&self, num: f64, pattern: &str) -> Result<String> {
        let formatted = if pattern.starts_with("N") || pattern.starts_with("C") {
            let (prefix, decimals, locale_tag) = self.parse_pattern(pattern);
            let locale = self.get_locale(&locale_tag);
            let symbol = if prefix == "C" {
                self.get_currency_symbol(&locale_tag)
            } else {
                ""
            };

            let int_part = num.trunc() as i64;
            let frac_part = num.fract();

            if decimals == 0 {
                format!("{}{}", symbol, int_part.to_formatted_string(&locale))
            } else {
                let frac_scaled = ((frac_part * 10f64.powi(decimals as i32)).round()) as i64;
                let decimal_sep = match locale_tag.as_str() {
                    "pt" | "fr" | "es" | "it" | "de" => ",",
                    _ => ".",
                };
                format!(
                    "{}{}{}{:0width$}",
                    symbol,
                    int_part.to_formatted_string(&locale),
                    decimal_sep,
                    frac_scaled,
                    width = decimals
                )
            }
        } else {
            match pattern {
                "n" => format!("{:.2}", num),
                "F" => format!("{:.2}", num),
                "f" => format!("{}", num),
                "0%" => format!("{:.0}%", num * 100.0),
                _ => format!("{}", num),
            }
        };

        Ok(formatted)
    }

    fn format_date(&self, dt: NaiveDateTime, pattern: &str) -> Result<String> {
        let formatted = match pattern {
            "dd/MM/yyyy" => format!("{:02}/{:02}/{}", dt.day(), dt.month(), dt.year()),
            "MM/dd/yyyy" => format!("{:02}/{:02}/{}", dt.month(), dt.day(), dt.year()),
            "yyyy-MM-dd" => format!("{}-{:02}-{:02}", dt.year(), dt.month(), dt.day()),
            "HH:mm:ss" => format!("{:02}:{:02}:{:02}", dt.hour(), dt.minute(), dt.second()),
            _ => dt.format(pattern).to_string(),
        };

        Ok(formatted)
    }

    fn format_text(&self, text: &str, pattern: &str) -> Result<String> {
        // Simple placeholder replacement
        Ok(pattern.replace("{}", text))
    }

    fn parse_pattern(&self, pattern: &str) -> (String, usize, String) {
        let prefix = &pattern[0..1];
        let decimals = pattern
            .chars()
            .nth(1)
            .and_then(|c| c.to_digit(10))
            .unwrap_or(2) as usize;
        let locale_tag = if pattern.len() > 2 {
            pattern[2..].to_string()
        } else {
            "en".to_string()
        };
        (prefix.to_string(), decimals, locale_tag)
    }

    fn get_locale(&self, tag: &str) -> Locale {
        match tag {
            "pt" => Locale::pt,
            "fr" => Locale::fr,
            "es" => Locale::es,
            "it" => Locale::it,
            "de" => Locale::de,
            _ => Locale::en,
        }
    }

    fn get_currency_symbol(&self, tag: &str) -> &'static str {
        match tag {
            "pt" | "fr" | "es" | "it" | "de" => "€",
            "uk" => "£",
            _ => "$",
        }
    }

    // ------------------------------------------------------------------------
    // Weather Service
    // ------------------------------------------------------------------------

    pub async fn get_weather(&self, req: WeatherRequest) -> Result<WeatherResponse> {
        // Check for API key
        let api_key = std::env::var("OPENWEATHER_API_KEY")
            .map_err(|_| anyhow!("Weather API key not configured"))?;

        let units = req.units.as_deref().unwrap_or("metric");
        let url = format!(
            "https://api.openweathermap.org/data/2.5/weather?q={}&units={}&appid={}",
            urlencoding::encode(&req.location),
            units,
            api_key
        );

        let client = reqwest::Client::new();
        let response = client.get(&url).send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("Weather API returned error: {}", response.status()));
        }

        let data: serde_json::Value = response.json().await?;

        Ok(WeatherResponse {
            location: req.location,
            temperature: data["main"]["temp"].as_f64().unwrap_or(0.0),
            description: data["weather"][0]["description"]
                .as_str()
                .unwrap_or("Unknown")
                .to_string(),
            humidity: data["main"]["humidity"].as_u64().unwrap_or(0) as u32,
            wind_speed: data["wind"]["speed"].as_f64().unwrap_or(0.0),
            units: units.to_string(),
        })
    }

    // ------------------------------------------------------------------------
    // Email Service
    // ------------------------------------------------------------------------

    pub async fn send_email(&self, req: EmailRequest) -> Result<EmailResponse> {
        use lettre::message::Message;
        use lettre::transport::smtp::authentication::Credentials;
        use lettre::{SmtpTransport, Transport};

        let smtp_host =
            std::env::var("SMTP_HOST").map_err(|_| anyhow!("SMTP_HOST not configured"))?;
        let smtp_user =
            std::env::var("SMTP_USER").map_err(|_| anyhow!("SMTP_USER not configured"))?;
        let smtp_pass =
            std::env::var("SMTP_PASSWORD").map_err(|_| anyhow!("SMTP_PASSWORD not configured"))?;

        let mut email = Message::builder()
            .from(smtp_user.parse()?)
            .subject(&req.subject);

        // Add recipients
        for recipient in &req.to {
            email = email.to(recipient.parse()?);
        }

        // Add CC if present
        if let Some(cc_list) = &req.cc {
            for cc in cc_list {
                email = email.cc(cc.parse()?);
            }
        }

        // Add BCC if present
        if let Some(bcc_list) = &req.bcc {
            for bcc in bcc_list {
                email = email.bcc(bcc.parse()?);
            }
        }

        let email = email.body(req.body)?;

        let creds = Credentials::new(smtp_user, smtp_pass);
        let mailer = SmtpTransport::relay(&smtp_host)?.credentials(creds).build();

        let result = mailer.send(&email)?;

        Ok(EmailResponse {
            message_id: result.message_id().unwrap_or_default().to_string(),
            status: "sent".to_string(),
        })
    }

    // ------------------------------------------------------------------------
    // Task Service
    // ------------------------------------------------------------------------

    pub async fn create_task(&self, req: TaskRequest) -> Result<TaskResponse> {
        use crate::shared::models::schema::tasks;
        use diesel::prelude::*;
        use uuid::Uuid;

        let task_id = Uuid::new_v4();
        let mut conn = self.state.conn.get()?;

        let new_task = (
            tasks::id.eq(&task_id),
            tasks::title.eq(&req.title),
            tasks::description.eq(&req.description),
            tasks::assignee.eq(&req.assignee),
            tasks::priority.eq(&req.priority.as_deref().unwrap_or("normal")),
            tasks::status.eq("open"),
            tasks::created_at.eq(chrono::Utc::now()),
        );

        diesel::insert_into(tasks::table)
            .values(&new_task)
            .execute(&mut conn)?;

        Ok(TaskResponse {
            task_id: task_id.to_string(),
            status: "created".to_string(),
        })
    }

    // ------------------------------------------------------------------------
    // Search Service
    // ------------------------------------------------------------------------

    pub async fn search_kb(&self, req: SearchRequest) -> Result<SearchResponse> {
        #[cfg(feature = "vectordb")]
        {
            use qdrant_client::prelude::*;
            use qdrant_client::qdrant::vectors::VectorsOptions;

            let qdrant_url =
                std::env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6333".to_string());
            let client = QdrantClient::from_url(&qdrant_url).build()?;

            // Generate embedding for query
            let embedding = self.generate_embedding(&req.query).await?;

            let collection_name = req.kb_name.as_deref().unwrap_or("default");
            let limit = req.limit.unwrap_or(10);
            let threshold = req.threshold.unwrap_or(0.7);

            let search_result = client
                .search_points(&SearchPoints {
                    collection_name: collection_name.to_string(),
                    vector: embedding,
                    limit: limit as u64,
                    score_threshold: Some(threshold),
                    with_payload: Some(true.into()),
                    ..Default::default()
                })
                .await?;

            let results: Vec<SearchResult> = search_result
                .result
                .into_iter()
                .map(|point| {
                    let payload = point.payload;
                    SearchResult {
                        content: payload
                            .get("content")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        source: payload
                            .get("source")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        score: point.score,
                        metadata: HashMap::new(),
                    }
                })
                .collect();

            Ok(SearchResponse {
                total: results.len(),
                results,
            })
        }

        #[cfg(not(feature = "vectordb"))]
        {
            // Fallback to simple text search
            Ok(SearchResponse {
                total: 0,
                results: vec![],
            })
        }
    }

    #[cfg(feature = "vectordb")]
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        let api_key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| anyhow!("OpenAI API key not configured"))?;

        let client = reqwest::Client::new();
        let response = client
            .post("https://api.openai.com/v1/embeddings")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&serde_json::json!({
                "model": "text-embedding-ada-002",
                "input": text
            }))
            .send()
            .await?;

        let data: serde_json::Value = response.json().await?;
        let embedding = data["data"][0]["embedding"]
            .as_array()
            .ok_or_else(|| anyhow!("Invalid embedding response"))?
            .iter()
            .map(|v| v.as_f64().unwrap_or(0.0) as f32)
            .collect();

        Ok(embedding)
    }

    // ------------------------------------------------------------------------
    // Memory Service
    // ------------------------------------------------------------------------

    pub async fn get_memory(&self, key: &str) -> Result<MemoryResponse> {
        if let Some(redis_client) = &self.state.redis_client {
            let mut conn = redis_client.get_async_connection().await?;
            use redis::AsyncCommands;

            let value: Option<String> = conn.get(key).await?;
            if let Some(json_str) = value {
                let value: serde_json::Value = serde_json::from_str(&json_str)?;
                Ok(MemoryResponse {
                    key: key.to_string(),
                    value: Some(value),
                    exists: true,
                })
            } else {
                Ok(MemoryResponse {
                    key: key.to_string(),
                    value: None,
                    exists: false,
                })
            }
        } else {
            Err(anyhow!("Redis not configured"))
        }
    }

    pub async fn set_memory(&self, req: MemoryRequest) -> Result<MemoryResponse> {
        if let Some(redis_client) = &self.state.redis_client {
            let mut conn = redis_client.get_async_connection().await?;
            use redis::AsyncCommands;

            if let Some(value) = &req.value {
                let json_str = serde_json::to_string(value)?;
                if let Some(ttl) = req.ttl {
                    let _: () = conn.setex(&req.key, json_str, ttl).await?;
                } else {
                    let _: () = conn.set(&req.key, json_str).await?;
                }

                Ok(MemoryResponse {
                    key: req.key.clone(),
                    value: Some(value.clone()),
                    exists: true,
                })
            } else {
                let _: () = conn.del(&req.key).await?;
                Ok(MemoryResponse {
                    key: req.key,
                    value: None,
                    exists: false,
                })
            }
        } else {
            Err(anyhow!("Redis not configured"))
        }
    }

    // ------------------------------------------------------------------------
    // Document Processing Service
    // ------------------------------------------------------------------------

    pub async fn process_document(
        &self,
        req: ProcessDocumentRequest,
    ) -> Result<ProcessDocumentResponse> {
        let mut response = ProcessDocumentResponse {
            text: String::new(),
            entities: None,
            keywords: None,
            summary: None,
            metadata: HashMap::new(),
        };

        // Extract text based on format
        response.text = match req.format.as_str() {
            "pdf" => self.extract_pdf_text(&req.content).await?,
            "html" => self.extract_html_text(&req.content)?,
            "markdown" => self.process_markdown(&req.content)?,
            _ => req.content.clone(),
        };

        // Extract entities if requested
        if req.extract_entities.unwrap_or(false) {
            response.entities = Some(self.extract_entities(&response.text).await?);
        }

        // Extract keywords if requested
        if req.extract_keywords.unwrap_or(false) {
            response.keywords = Some(self.extract_keywords(&response.text)?);
        }

        // Generate summary if requested
        if req.summarize.unwrap_or(false) {
            response.summary = Some(self.generate_summary(&response.text).await?);
        }

        Ok(response)
    }

    async fn extract_pdf_text(&self, content: &str) -> Result<String> {
        // Base64 decode if needed
        let bytes = base64::decode(content)?;

        // Use pdf-extract crate
        let text = pdf_extract::extract_text_from_mem(&bytes)?;
        Ok(text)
    }

    fn extract_html_text(&self, html: &str) -> Result<String> {
        // Simple HTML tag removal
        let re = regex::Regex::new(r"<[^>]+>")?;
        let text = re.replace_all(html, " ");
        Ok(text.to_string())
    }

    fn process_markdown(&self, markdown: &str) -> Result<String> {
        // For now, just return as-is
        // Could use a markdown parser to extract plain text
        Ok(markdown.to_string())
    }

    async fn extract_entities(&self, text: &str) -> Result<Vec<Entity>> {
        // Simple entity extraction using regex patterns
        let mut entities = Vec::new();

        // Email pattern
        let email_re = regex::Regex::new(r"\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Z|a-z]{2,}\b")?;
        for cap in email_re.captures_iter(text) {
            entities.push(Entity {
                text: cap[0].to_string(),
                entity_type: "email".to_string(),
                confidence: 0.9,
            });
        }

        // Phone pattern
        let phone_re = regex::Regex::new(r"\b\d{3}[-.]?\d{3}[-.]?\d{4}\b")?;
        for cap in phone_re.captures_iter(text) {
            entities.push(Entity {
                text: cap[0].to_string(),
                entity_type: "phone".to_string(),
                confidence: 0.8,
            });
        }

        // URL pattern
        let url_re = regex::Regex::new(r"https?://[^\s]+")?;
        for cap in url_re.captures_iter(text) {
            entities.push(Entity {
                text: cap[0].to_string(),
                entity_type: "url".to_string(),
                confidence: 0.95,
            });
        }

        Ok(entities)
    }

    fn extract_keywords(&self, text: &str) -> Result<Vec<String>> {
        // Simple keyword extraction based on word frequency
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut word_count: HashMap<String, usize> = HashMap::new();

        for word in words {
            let clean_word = word
                .to_lowercase()
                .chars()
                .filter(|c| c.is_alphanumeric())
                .collect::<String>();

            if clean_word.len() > 3 {
                // Skip short words
                *word_count.entry(clean_word).or_insert(0) += 1;
            }
        }

        let mut keywords: Vec<(String, usize)> = word_count.into_iter().collect();
        keywords.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(keywords
            .into_iter()
            .take(10)
            .map(|(word, _)| word)
            .collect())
    }

    async fn generate_summary(&self, text: &str) -> Result<String> {
        // For now, just return first 200 characters
        // In production, would use LLM for summarization
        let summary = if text.len() > 200 {
            format!("{}...", &text[..200])
        } else {
            text.to_string()
        };

        Ok(summary)
    }
}

// ============================================================================
// HTTP Handlers
// ============================================================================

pub async fn format_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FormatRequest>,
) -> impl IntoResponse {
    let service = KeywordService::new(state);
    match service.format_value(req).await {
        Ok(response) => (StatusCode::OK, Json(response)),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(FormatResponse {
                formatted: format!("Error: {}", e),
            }),
        ),
    }
}

pub async fn weather_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<WeatherRequest>,
) -> impl IntoResponse {
    let service = KeywordService::new(state);
    match service.get_weather(req).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            StatusCode::SERVICE_UNAVAILABLE,
            format!("Weather service error: {}", e),
        )),
    }
}

pub async fn email_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<EmailRequest>,
) -> impl IntoResponse {
    let service = KeywordService::new(state);
    match service.send_email(req).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Email service error: {}", e),
        )),
    }
}

pub async fn task_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TaskRequest>,
) -> impl IntoResponse {
    let service = KeywordService::new(state);
    match service.create_task(req).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Task service error: {}", e),
        )),
    }
}

pub async fn search_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SearchRequest>,
) -> impl IntoResponse {
    let service = KeywordService::new(state);
    match service.search_kb(req).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Search service error: {}", e),
        )),
    }
}

pub async fn get_memory_handler(
    State(state): State<Arc<AppState>>,
    Query(params): Query<HashMap<String, String>>,
) -> impl IntoResponse {
    let key = params.get("key").ok_or((
        StatusCode::BAD_REQUEST,
        "Missing 'key' parameter".to_string(),
    ))?;

    let service = KeywordService::new(state);
    match service.get_memory(key).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Memory service error: {}", e),
        )),
    }
}

pub async fn set_memory_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<MemoryRequest>,
) -> impl IntoResponse {
    let service = KeywordService::new(state);
    match service.set_memory(req).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Memory service error: {}", e),
        )),
    }
}

pub async fn process_document_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ProcessDocumentRequest>,
) -> impl IntoResponse {
    let service = KeywordService::new(state);
    match service.process_document(req).await {
        Ok(response) => Ok(Json(response)),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Document processing error: {}", e),
        )),
    }
}

// ============================================================================
// Router Configuration
// ============================================================================

pub fn routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/api/services/format", post(format_handler))
        .route("/api/services/weather", post(weather_handler))
        .route("/api/services/email", post(email_handler))
        .route("/api/services/task", post(task_handler))
        .route("/api/services/search", post(search_handler))
        .route(
            "/api/services/memory",
            get(get_memory_handler).post(set_memory_handler),
        )
        .route("/api/services/document", post(process_document_handler))
}
