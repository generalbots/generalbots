//! AI assistant and cached prompt evaluation for the sheet editor.
//!
//! #2923 — `/api/sheet/ai` uses the LLM path already shipped in this crate
//! (`call_llm`) and never answers from a keyword table. An unconfigured or
//! failing provider produces an explicit `503 ai_unavailable` envelope.

use crate::state::SheetState;
use crate::types::{SheetAiRequest, SheetAiResponse};
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use tokio::sync::Semaphore;

const MAX_BATCH_SIZE: usize = 100;
const MAX_CONCURRENT_LLM: usize = 10;

/// Typed verbs registered in the app command catalog
/// (`botserver/src/apps/commands/app_commands_extra/table/docs_and_sheet_crud.rs`).
const ALLOWED_ACTIONS: &[&str] = &[
    "sheet.cell.set",
    "sheet.cells.set",
    "sheet.range.set",
    "sheet.formula.set",
    "sheet.format.apply",
    "sheet.merge",
    "sheet.unmerge",
    "sheet.sort",
    "sheet.filter.apply",
    "sheet.chart.create",
    "sheet.pivot.create",
];

#[derive(Debug, Deserialize)]
pub struct EvaluateAiPromptRequest {
    pub prompt: String,
    #[serde(default)]
    pub sheet_id: Option<String>,
    #[serde(default)]
    pub cell_ref: Option<String>,
    #[serde(default)]
    pub worksheet_index: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct EvaluateAiPromptResponse {
    pub result: String,
    pub cached: bool,
}

#[derive(Debug, Deserialize)]
pub struct BatchEvaluateRequest {
    pub prompts: Vec<String>,
    #[serde(default)]
    pub sheet_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct BatchEvaluateResponse {
    pub results: Vec<BatchItemResult>,
    pub cached_count: usize,
    pub fresh_count: usize,
}

#[derive(Debug, Serialize)]
pub struct BatchItemResult {
    pub index: usize,
    pub result: String,
    pub cached: bool,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct LlmPayload {
    #[serde(default)]
    response: String,
    #[serde(default)]
    action: Option<String>,
    #[serde(default)]
    data: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Unavailable {
    NotConfigured,
    Provider,
}

impl Unavailable {
    fn reason(self) -> &'static str {
        match self {
            Self::NotConfigured => "not_configured",
            Self::Provider => "provider_error",
        }
    }
}

static AI_CACHE: LazyLock<Mutex<HashMap<String, String>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

static LLM_SEMAPHORE: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(MAX_CONCURRENT_LLM));

fn compute_cache_key(prompt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prompt.as_bytes());
    let hash = hasher.finalize();
    format!("ai_prompt:{}", hex::encode(hash))
}

fn llm_available() -> bool {
    match std::env::var("BOT_AI_PROMPT_URL") {
        Ok(url) => !url.trim().is_empty(),
        Err(_) => false,
    }
}

fn build_prompt(req: &SheetAiRequest) -> String {
    let selection = serde_json::to_string(&req.selection).unwrap_or_else(|_| "null".to_string());
    let active_cell =
        serde_json::to_string(&req.active_cell).unwrap_or_else(|_| "null".to_string());
    format!(
        "You are an assistant embedded in a spreadsheet editor. Return ONLY a JSON object \
         with a string key \"response\" describing what you did or found. If the user asks for \
         a workbook change, also set \"action\" to exactly one of these typed verbs: {}. Put the \
         verb's arguments in the object \"data\". Never claim an edit you did not request.\n\
         Command: {}\nSheet id: {}\nSelection: {}\nActive cell: {}",
        ALLOWED_ACTIONS.join(", "),
        req.command,
        req.sheet_id.as_deref().unwrap_or(""),
        selection,
        active_cell
    )
}

/// Parses the LLM envelope into the typed sheet response. Unknown verbs are
/// stripped before the answer can be handed to a verb endpoint.
fn parse_llm_answer(content: &str, req: &SheetAiRequest) -> SheetAiResponse {
    let payload: LlmPayload = match serde_json::from_str(content) {
        Ok(payload) => payload,
        Err(_) => {
            return SheetAiResponse {
                response: content.trim().to_string(),
                action: None,
                data: None,
            }
        }
    };
    let mut data = match payload.data {
        Some(value @ serde_json::Value::Object(_)) => Some(value),
        _ => None,
    };
    let action = match payload.action {
        Some(action) if ALLOWED_ACTIONS.contains(&action.as_str()) => Some(action),
        _ => None,
    };
    if let (Some(action), Some(serde_json::Value::Object(fields))) = (&action, &mut data) {
        if let Some(id) = req.sheet_id.as_deref() {
            fields
                .entry("file_id".to_string())
                .or_insert_with(|| serde_json::Value::String(id.to_string()));
        }
        log::debug!("Sheet AI typed action: {action}");
    } else {
        data = None;
    }
    SheetAiResponse {
        response: payload.response,
        action,
        data,
    }
}

fn unavailable_body(reason: Unavailable) -> serde_json::Value {
    serde_json::json!({
        "error": "ai_unavailable",
        "reason": reason.reason()
    })
}

fn unavailable_response(reason: Unavailable) -> Response {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(unavailable_body(reason)),
    )
        .into_response()
}

pub async fn handle_sheet_ai(
    State(state): State<Arc<SheetState>>,
    Json(req): Json<SheetAiRequest>,
) -> Response {
    log::trace!(
        "Sheet AI request (drive configured: {})",
        state.drive.is_some()
    );
    if !llm_available() {
        return unavailable_response(Unavailable::NotConfigured);
    }
    let prompt = build_prompt(&req);
    match call_llm(&prompt).await {
        Ok(content) => Json(parse_llm_answer(&content, &req)).into_response(),
        Err(e) => {
            log::error!("Sheet AI: LLM request failed: {e}");
            unavailable_response(Unavailable::Provider)
        }
    }
}

pub async fn handle_evaluate_ai_prompt(
    State(_state): State<Arc<SheetState>>,
    Json(req): Json<EvaluateAiPromptRequest>,
) -> Result<Json<EvaluateAiPromptResponse>, (StatusCode, String)> {
    let cache_key = compute_cache_key(&req.prompt);
    {
        let cache = AI_CACHE.lock().await;
        if let Some(cached) = cache.get(&cache_key) {
            return Ok(Json(EvaluateAiPromptResponse {
                result: cached.clone(),
                cached: true,
            }));
        }
    }
    let _permit = LLM_SEMAPHORE.acquire().await.map_err(|e| {
        (
            StatusCode::TOO_MANY_REQUESTS,
            format!("Rate limited: {}", e),
        )
    })?;
    let result = call_llm(&req.prompt).await.map_err(|e| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            format!("LLM call failed: {}", e),
        )
    })?;
    {
        let mut cache = AI_CACHE.lock().await;
        cache.insert(cache_key, result.clone());
    }
    Ok(Json(EvaluateAiPromptResponse {
        result,
        cached: false,
    }))
}

pub async fn handle_batch_evaluate_ai(
    State(_state): State<Arc<SheetState>>,
    Json(req): Json<BatchEvaluateRequest>,
) -> Result<Json<BatchEvaluateResponse>, (StatusCode, String)> {
    if req.prompts.len() > MAX_BATCH_SIZE {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Batch size {} exceeds maximum of {}",
                req.prompts.len(),
                MAX_BATCH_SIZE
            ),
        ));
    }
    let mut results = Vec::with_capacity(req.prompts.len().min(MAX_BATCH_SIZE));
    let mut cached_count = 0usize;
    let mut fresh_count = 0usize;

    for (index, prompt) in req.prompts.iter().enumerate() {
        let cache_key = compute_cache_key(prompt);
        let cached_result = {
            let cache = AI_CACHE.lock().await;
            cache.get(&cache_key).cloned()
        };
        if let Some(cached) = cached_result {
            results.push(BatchItemResult {
                index,
                result: cached,
                cached: true,
                error: None,
            });
            cached_count += 1;
            continue;
        }
        let _permit = match LLM_SEMAPHORE.acquire().await {
            Ok(p) => p,
            Err(e) => {
                results.push(BatchItemResult {
                    index,
                    result: String::new(),
                    cached: false,
                    error: Some(format!("Rate limited: {}", e)),
                });
                continue;
            }
        };
        match call_llm(prompt).await {
            Ok(value) => {
                {
                    let mut cache = AI_CACHE.lock().await;
                    cache.insert(cache_key, value.clone());
                }
                results.push(BatchItemResult {
                    index,
                    result: value,
                    cached: false,
                    error: None,
                });
                fresh_count += 1;
            }
            Err(e) => {
                results.push(BatchItemResult {
                    index,
                    result: String::new(),
                    cached: false,
                    error: Some(e),
                });
            }
        }
    }
    Ok(Json(BatchEvaluateResponse {
        results,
        cached_count,
        fresh_count,
    }))
}

async fn call_llm(prompt: &str) -> Result<String, String> {
    perform_llm_request(prompt).await
}

async fn perform_llm_request(prompt: &str) -> Result<String, String> {
    let llm_url = match std::env::var("BOT_AI_PROMPT_URL") {
        Ok(url) if !url.is_empty() => url,
        _ => {
            // Never fabricate an answer: an unconfigured LLM is a hard error
            // the caller surfaces as 503, not a canned "analysis".
            return Err("AI not configured: BOT_AI_PROMPT_URL is not set".to_string());
        }
    };
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {}", e))?;
    let body = serde_json::json!({
        "model": std::env::var("BOT_AI_PROMPT_MODEL").unwrap_or_else(|_| "default".to_string()),
        "messages": [{"role": "user", "content": prompt}]
    });
    let resp = client
        .post(&llm_url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;
    let text = resp
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;
    Ok(text)
}

#[cfg(test)]
#[path = "ai_tests.rs"]
mod ai_tests;
