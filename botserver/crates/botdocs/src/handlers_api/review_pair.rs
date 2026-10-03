//! HTTP surface for pair-of-LLM document review (issue #1458).

use axum::extract::State;
use axum::http::StatusCode;
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::review::{
    MergedFinding, Review, ReviewError, ReviewOutcome, ReviewerRole, build_prompt,
    build_rewrite_prompt, merge_reviews, parse_review,
};
use crate::state::DocState;

/// Request body for `POST /api/docs/ai/review-pair`.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct ReviewPairRequest {
    /// The document to review. Empty is rejected rather than reviewed.
    #[serde(default)]
    pub text: String,
    /// Optional extra requirement handed to both reviewers.
    #[serde(default)]
    pub instruction: String,
    /// Apply the merged findings and return the rewritten document too.
    #[serde(default)]
    pub rewrite: bool,
}

/// Response for `POST /api/docs/ai/review-pair`.
///
/// Machine-readable on purpose: a caller renders the list, it does not parse
/// prose out of a chat reply.
#[derive(Debug, Clone, Serialize)]
pub struct ReviewPairResponse {
    /// Findings from both reviewers, most severe first.
    pub findings: Vec<MergedFinding>,
    /// How many findings both reviewers raised.
    pub agreed: usize,
    /// The rewrite, present only when `rewrite` was requested and the pass ran.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revised_text: Option<String>,
    /// One line per reviewer, for display.
    pub verdicts: Vec<ReviewerVerdict>,
}

/// One reviewer's headline, attributed.
#[derive(Debug, Clone, Serialize)]
pub struct ReviewerVerdict {
    pub role: ReviewerRole,
    pub verdict: String,
    /// Number of findings this reviewer raised on its own.
    pub findings: usize,
}

/// Cap on the document characters sent to a reviewer.
///
/// A 400-page manual pasted into a review prompt will be truncated by the
/// provider anyway, at a cost the tenant pays for; refusing is clearer than a
/// silently partial review.
pub const MAX_DOCUMENT_CHARS: usize = 120_000;

/// `POST /api/docs/ai/review-pair` — review a document with two LLMs at once.
pub async fn handle_ai_review_pair(
    State(_state): State<Arc<DocState>>,
    Json(body): Json<ReviewPairRequest>,
) -> Result<Json<ReviewPairResponse>, (StatusCode, Json<serde_json::Value>)> {
    let text = body.text.trim();
    if text.is_empty() {
        return Err(bad_request("text is required"));
    }
    if text.chars().count() > MAX_DOCUMENT_CHARS {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            Json(serde_json::json!({
                "error": format!("document exceeds {MAX_DOCUMENT_CHARS} characters")
            })),
        ));
    }

    let reviews = run_reviewers(text, &body.instruction).await?;
    let mut outcome = merge_reviews(&reviews);

    if body.rewrite && !outcome.findings.is_empty() {
        match rewrite(text, &outcome.findings).await {
            Ok(revised) => outcome.revised_text = revised,
            Err(message) => {
                // The review itself succeeded; a failed rewrite must not discard
                // the findings the caller can still act on.
                tracing_warn(&message);
            }
        }
    }

    let verdicts = reviews
        .iter()
        .map(|review| ReviewerVerdict {
            role: review.role,
            verdict: review.verdict.clone(),
            findings: review.findings.len(),
        })
        .collect();

    Ok(Json(ReviewPairResponse {
        findings: outcome.findings,
        agreed: outcome.agreed,
        revised_text: if outcome.revised_text.is_empty() {
            None
        } else {
            Some(outcome.revised_text)
        },
        verdicts,
    }))
}

/// Runs both reviewers concurrently.
///
/// Concurrency is the point: a reviewer must not see the other's verdict, and
/// running them one after another would tempt a later pass to echo an earlier one.
async fn run_reviewers(
    text: &str,
    instruction: &str,
) -> Result<Vec<Review>, (StatusCode, Json<serde_json::Value>)> {
    let mut set = tokio::task::JoinSet::new();
    for role in ReviewerRole::ALL {
        let document = text.to_string();
        let instruction = instruction.to_string();
        set.spawn(async move {
            let model = model_for(role);
            let prompt = build_prompt(role, &document, &instruction);
            let raw = complete(&model, role.system_prompt(), &prompt).await;
            (role, raw)
        });
    }

    let mut reviews = Vec::new();
    let mut failures: Vec<String> = Vec::new();
    while let Some(joined) = set.join_next().await {
        match joined {
            Ok((role, Ok(raw))) => reviews.push(parse_review(role, &raw)),
            Ok((role, Err(message))) => failures.push(format!("{}: {message}", role.as_str())),
            Err(err) => failures.push(format!("reviewer task failed: {err}")),
        }
    }

    if reviews.is_empty() {
        let message = if failures.is_empty() {
            "no reviewer ran".to_string()
        } else {
            failures.join("; ")
        };
        let status = if message.contains("LLM_URL") {
            StatusCode::SERVICE_UNAVAILABLE
        } else {
            StatusCode::BAD_GATEWAY
        };
        return Err((
            status,
            Json(serde_json::json!({ "error": message })),
        ));
    }

    reviews.sort_by_key(|review| review.role);
    Ok(reviews)
}

/// Applies the merged findings to produce a revised document.
async fn rewrite(text: &str, findings: &[MergedFinding]) -> Result<String, String> {
    let model = std::env::var("LLM_MODEL").unwrap_or_else(|_| "default".to_string());
    let prompt = build_rewrite_prompt(text, findings);
    match complete(&model, "You are a precise editor who applies exactly the corrections asked for and invents nothing.", &prompt).await {
        Ok(revised) if !revised.trim().is_empty() => Ok(revised),
        Ok(_) => Err("rewrite pass returned nothing".to_string()),
        Err(message) => Err(message),
    }
}

/// Per-role model override.
///
/// `DOCS_REVIEW_MODEL_EDITOR` and `DOCS_REVIEW_MODEL_CRITIC` let a deployment put
/// two genuinely different models behind the roles. Without them both fall back
/// to `LLM_MODEL`, which still gives role separation but not model separation —
/// the pairing is weaker, and worth knowing.
fn model_for(role: ReviewerRole) -> String {
    let variable = match role {
        ReviewerRole::Editor => "DOCS_REVIEW_MODEL_EDITOR",
        ReviewerRole::Critic => "DOCS_REVIEW_MODEL_CRITIC",
    };
    std::env::var(variable)
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| {
            std::env::var("LLM_MODEL")
                .ok()
                .filter(|value| !value.trim().is_empty())
        })
        .unwrap_or_else(|| "default".to_string())
}

/// True when the two roles are backed by different models.
#[must_use]
pub fn roles_use_distinct_models() -> bool {
    let mut models: Vec<String> = ReviewerRole::ALL.iter().map(|role| model_for(*role)).collect();
    models.sort();
    let count = models.len();
    models.dedup();
    count != models.len()
}

/// One chat-completions call.
async fn complete(model: &str, system: &str, user: &str) -> Result<String, String> {
    let url = std::env::var("LLM_URL").unwrap_or_default();
    let key = std::env::var("LLM_KEY").unwrap_or_default();
    if url.trim().is_empty() {
        return Err("LLM_URL is not set".to_string());
    }
    let body = serde_json::json!({
        "model": model,
        "messages": [
            { "role": "system", "content": system },
            { "role": "user", "content": user }
        ],
        "temperature": 0.2,
    });

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .map_err(|e| format!("could not build an HTTP client: {e}"))?;

    let mut request = client.post(&url).json(&body);
    if !key.trim().is_empty() {
        request = request.header("Authorization", format!("Bearer {key}"));
    }
    let response = request.send().await.map_err(|e| format!("request failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("provider returned HTTP {}", response.status()));
    }
    let parsed: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("response was not JSON: {e}"))?;
    parsed["choices"][0]["message"]["content"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "response carried no content".to_string())
}

fn bad_request(message: &str) -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::BAD_REQUEST, Json(serde_json::json!({ "error": message })))
}

fn tracing_warn(message: &str) {
    log::warn!("[docs] pair review rewrite pass failed: {message}");
}

/// Convenience wrapper for callers that want the outcome rather than HTTP.
pub async fn review_document(
    text: &str,
    instruction: &str,
) -> Result<ReviewOutcome, ReviewError> {
    if std::env::var("LLM_URL").unwrap_or_default().trim().is_empty() {
        return Err(ReviewError::NotConfigured);
    }
    let mut reviews = Vec::new();
    for role in ReviewerRole::ALL {
        let model = model_for(role);
        let prompt = build_prompt(role, text, instruction);
        match complete(&model, role.system_prompt(), &prompt).await {
            Ok(raw) => reviews.push(parse_review(role, &raw)),
            Err(message) => {
                log::warn!("[docs] reviewer {} failed: {message}", role.as_str());
            }
        }
    }
    if reviews.is_empty() {
        return Err(ReviewError::NoReviewers(
            "every reviewer call failed".to_string(),
        ));
    }
    Ok(merge_reviews(&reviews))
}

#[cfg(test)]
#[path = "review_pair_tests.rs"]
mod tests;
