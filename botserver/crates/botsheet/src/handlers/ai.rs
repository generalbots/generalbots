use crate::state::SheetState;
use crate::types::{SheetAiRequest, SheetAiResponse};
use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;
use tokio::sync::Mutex;
use tokio::sync::Semaphore;

const MAX_BATCH_SIZE: usize = 100;
const MAX_CONCURRENT_LLM: usize = 10;

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

static AI_CACHE: LazyLock<Mutex<HashMap<String, String>>> = LazyLock::new(|| Mutex::new(HashMap::new()));

static LLM_SEMAPHORE: LazyLock<Semaphore> = LazyLock::new(|| Semaphore::new(MAX_CONCURRENT_LLM));

fn compute_cache_key(prompt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(prompt.as_bytes());
    let hash = hasher.finalize();
    format!("ai_prompt:{}", hex::encode(hash))
}

pub async fn handle_sheet_ai(
    State(_state): State<Arc<SheetState>>,
    Json(req): Json<SheetAiRequest>,
) -> impl IntoResponse {
    let command = req.command.to_lowercase();
    let response = if command.contains("sum") {
        "I can help you sum values. Select a range and use the SUM formula, or I've added a SUM formula below your selection."
    } else if command.contains("average") || command.contains("avg") {
        "I can calculate averages. Select a range and use the AVERAGE formula."
    } else if command.contains("chart") {
        "To create a chart, select your data range first, then choose the chart type from the Chart menu."
    } else if command.contains("sort") {
        "I can sort your data. Select the range you want to sort, then specify ascending or descending order."
    } else if command.contains("format")
        || command.contains("currency")
        || command.contains("percent")
    {
        "I've applied the formatting to your selected cells."
    } else if command.contains("bold") || command.contains("italic") {
        "I've applied the text formatting to your selected cells."
    } else if command.contains("filter") {
        "I've enabled filtering on your data. Use the dropdown arrows in the header row to filter."
    } else if command.contains("freeze") {
        "I've frozen the specified rows/columns so they stay visible when scrolling."
    } else if command.contains("merge") {
        "I've merged the selected cells into one."
    } else if command.contains("clear") {
        "I've cleared the content from the selected cells."
    } else if command.contains("help") {
        "I can help you with:\n- Sum/Average columns\n- Format as currency or percent\n- Bold/Italic formatting\n- Sort data\n- Create charts\n- Filter data\n- Freeze panes\n- Merge cells"
    } else {
        "I understand you want help with your spreadsheet. Try commands like 'sum column B', 'format as currency', 'sort ascending', or 'create a chart'."
    };
    Json(SheetAiResponse { response: response.to_string(), action: None, data: None })
}

pub async fn handle_evaluate_ai_prompt(
    State(_state): State<Arc<SheetState>>,
    Json(req): Json<EvaluateAiPromptRequest>,
) -> Result<Json<EvaluateAiPromptResponse>, (StatusCode, String)> {
    let cache_key = compute_cache_key(&req.prompt);
    {
        let cache = AI_CACHE.lock().await;
        if let Some(cached) = cache.get(&cache_key) {
            return Ok(Json(EvaluateAiPromptResponse { result: cached.clone(), cached: true }));
        }
    }
    let _permit = LLM_SEMAPHORE.acquire().await.map_err(|e| {
        (StatusCode::TOO_MANY_REQUESTS, format!("Rate limited: {}", e))
    })?;
    let result = call_llm(&req.prompt).await.map_err(|e| {
        (StatusCode::SERVICE_UNAVAILABLE, format!("LLM call failed: {}", e))
    })?;
    {
        let mut cache = AI_CACHE.lock().await;
        cache.insert(cache_key, result.clone());
    }
    Ok(Json(EvaluateAiPromptResponse { result, cached: false }))
}

pub async fn handle_batch_evaluate_ai(
    State(_state): State<Arc<SheetState>>,
    Json(req): Json<BatchEvaluateRequest>,
) -> Result<Json<BatchEvaluateResponse>, (StatusCode, String)> {
    if req.prompts.len() > MAX_BATCH_SIZE {
        return Err((
            StatusCode::BAD_REQUEST,
            format!("Batch size {} exceeds maximum of {}", req.prompts.len(), MAX_BATCH_SIZE),
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
            results.push(BatchItemResult { index, result: cached, cached: true, error: None });
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
                results.push(BatchItemResult { index, result: value, cached: false, error: None });
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
    Ok(Json(BatchEvaluateResponse { results, cached_count, fresh_count }))
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
    let text = resp.text().await.map_err(|e| format!("Failed to read response: {}", e))?;
    Ok(text)
}
