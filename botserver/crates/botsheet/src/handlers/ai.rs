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

/// Maps a lowercased command to an honest reply.
///
/// This endpoint does not mutate the workbook, so no branch may claim that an
/// action was performed. Each branch explains the manual control instead.
fn ai_reply_for(command: &str) -> &'static str {
    if command.contains("sum") {
        "This chat endpoint cannot edit the sheet yet. Select the cell where you want the total, then use the SUM formula (for example =SUM(B2:B10)) or AutoSum in the toolbar."
    } else if command.contains("average") || command.contains("avg") {
        "This chat endpoint cannot edit the sheet yet. Select a range and use the AVERAGE formula (for example =AVERAGE(B2:B10)) from the formula menu."
    } else if command.contains("chart") {
        "This chat endpoint cannot insert charts yet. Select your data range, then choose the chart type from the Insert \u{2192} Chart menu."
    } else if command.contains("sort") {
        "This chat endpoint cannot sort data yet. Select the range, then use Data \u{2192} Sort and choose ascending or descending order."
    } else if command.contains("format")
        || command.contains("currency")
        || command.contains("percent")
    {
        "This chat endpoint cannot change cell formatting yet. Select the cells, then use the number-format control in the toolbar to apply currency or percent."
    } else if command.contains("bold") || command.contains("italic") {
        "This chat endpoint cannot change text formatting yet. Select the cells, then use the bold or italic button in the toolbar."
    } else if command.contains("filter") {
        "This chat endpoint cannot enable filters yet. Select the header row, then use Data \u{2192} Filter; dropdown arrows will appear in the header cells."
    } else if command.contains("freeze") {
        "This chat endpoint cannot freeze panes yet. Select the row or column to keep visible, then use View \u{2192} Freeze panes."
    } else if command.contains("merge") {
        "This chat endpoint cannot merge cells yet. Select the cells to combine, then use Format \u{2192} Merge cells."
    } else if command.contains("clear") {
        "This chat endpoint cannot clear cells yet. Select the cells and press Delete or Backspace."
    } else if command.contains("help") {
        "This assistant only explains how to use the editor; it cannot modify the workbook. The editor can help you:\n- Sum/Average columns with formulas or AutoSum\n- Format as currency or percent from the toolbar\n- Bold/Italic formatting from the toolbar\n- Sort data from Data \u{2192} Sort\n- Create charts from Insert \u{2192} Chart\n- Filter data from Data \u{2192} Filter\n- Freeze panes from View \u{2192} Freeze panes\n- Merge cells from Format \u{2192} Merge cells"
    } else {
        "This assistant only explains how to use the editor; it cannot modify the workbook. Try 'sum column B', 'format as currency', 'sort ascending' or 'create a chart' to get directions for those controls."
    }
}

pub async fn handle_sheet_ai(
    State(_state): State<Arc<SheetState>>,
    Json(req): Json<SheetAiRequest>,
) -> impl IntoResponse {
    let command = req.command.to_lowercase();
    let response = ai_reply_for(&command);
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

#[cfg(test)]
mod tests {
    use super::*;

    const ACTION_COMMANDS: &[&str] = &[
        "sum",
        "average",
        "create chart",
        "sort ascending",
        "format as currency",
        "percent",
        "bold",
        "italic",
        "filter",
        "freeze rows",
        "merge cells",
        "clear cells",
    ];

    fn assert_no_false_confirmation(command: &str, reply: &str) {
        assert!(
            !reply.contains(concat!("I", "'ve")) && !reply.contains(concat!("I", " have")),
            "false confirmation for {command}: {reply}"
        );
    }

    #[test]
    fn mutation_commands_explain_controls_without_claiming_success() {
        for command in ACTION_COMMANDS {
            let reply = ai_reply_for(command);
            assert_no_false_confirmation(command, reply);
            assert!(
                reply.contains("This chat endpoint cannot"),
                "missing honest limitation for {command}: {reply}"
            );
        }
    }

    #[test]
    fn action_commands_point_to_manual_controls() {
        assert!(ai_reply_for("sum").contains("SUM formula"));
        assert!(ai_reply_for("format as currency").contains("number-format"));
        assert!(ai_reply_for("bold").contains("toolbar"));
        assert!(ai_reply_for("filter").contains("Data"));
        assert!(ai_reply_for("help").contains("cannot modify the workbook"));
    }

    #[test]
    fn unknown_command_is_instruction_only() {
        let reply = ai_reply_for("make it prettier");
        assert!(!reply.contains(concat!("I", "'ve")) && !reply.contains(concat!("I", " have")));
        assert!(reply.contains("cannot modify the workbook"));
    }
}
