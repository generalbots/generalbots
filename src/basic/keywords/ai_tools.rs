use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{debug, trace};
use rhai::{Dynamic, Engine, EvalAltResult, Map, Position};
use std::sync::Arc;
use std::time::Duration;

pub fn register_ai_tools_keywords(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    register_translate_keyword(state.clone(), user.clone(), engine);
    register_ocr_keyword(state.clone(), user.clone(), engine);
    register_sentiment_keyword(state.clone(), user.clone(), engine);
    register_classify_keyword(state, user, engine);
}

fn register_translate_keyword(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["TRANSLATE", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let text = context.eval_expression_tree(&inputs[0])?.to_string();
                let target_lang = context.eval_expression_tree(&inputs[1])?.to_string();
                trace!("TRANSLATE to {}", target_lang);
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().expect("failed to create runtime");
                    let result = rt.block_on(async { translate_text(&text, &target_lang).await });
                    let _ = tx.send(result);
                });
                match rx.recv_timeout(Duration::from_secs(60)) {
                    Ok(Ok(result)) => Ok(Dynamic::from(result)),
                    Ok(Err(e)) => Err(Box::new(EvalAltResult::ErrorRuntime(
                        format!("TRANSLATE failed: {}", e).into(),
                        Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(EvalAltResult::ErrorRuntime(
                        "TRANSLATE timed out".into(),
                        Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid syntax registration");

    debug!("Registered TRANSLATE keyword");
}

fn register_ocr_keyword(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(["OCR", "$expr$"], false, move |context, inputs| {
            let image_path = context.eval_expression_tree(&inputs[0])?.to_string();
            trace!("OCR {}", image_path);
            let (tx, rx) = std::sync::mpsc::channel();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("failed to create runtime");
                let result = rt.block_on(async { perform_ocr(&image_path).await });
                let _ = tx.send(result);
            });
            match rx.recv_timeout(Duration::from_secs(60)) {
                Ok(Ok(result)) => Ok(Dynamic::from(result)),
                Ok(Err(e)) => Err(Box::new(EvalAltResult::ErrorRuntime(
                    format!("OCR failed: {}", e).into(),
                    Position::NONE,
                ))),
                Err(_) => Err(Box::new(EvalAltResult::ErrorRuntime(
                    "OCR timed out".into(),
                    Position::NONE,
                ))),
            }
        })
        .expect("valid syntax registration");

    debug!("Registered OCR keyword");
}

fn register_sentiment_keyword(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(["SENTIMENT", "$expr$"], false, move |context, inputs| {
            let text = context.eval_expression_tree(&inputs[0])?.to_string();
            trace!("SENTIMENT analysis");
            let (tx, rx) = std::sync::mpsc::channel();
            let text_clone = text.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().expect("failed to create runtime");
                let result = rt.block_on(async { analyze_sentiment(&text_clone).await });
                let _ = tx.send(result);
            });
            match rx.recv_timeout(Duration::from_secs(30)) {
                Ok(Ok(result)) => Ok(result),
                Ok(Err(e)) => Err(Box::new(EvalAltResult::ErrorRuntime(
                    format!("SENTIMENT failed: {}", e).into(),
                    Position::NONE,
                ))),
                Err(_) => Ok(analyze_sentiment_quick(&text)),
            }
        })
        .expect("valid syntax registration");

    engine.register_fn("SENTIMENT_QUICK", |text: &str| -> Dynamic {
        analyze_sentiment_quick(text)
    });

    debug!("Registered SENTIMENT keyword");
}

fn register_classify_keyword(_state: Arc<AppState>, _user: UserSession, engine: &mut Engine) {
    engine
        .register_custom_syntax(
            ["CLASSIFY", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let text = context.eval_expression_tree(&inputs[0])?.to_string();
                let categories = context.eval_expression_tree(&inputs[1])?;
                trace!("CLASSIFY into categories");
                let cat_list: Vec<String> = if categories.is_array() {
                    categories
                        .into_array()
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(|c| c.into_string().ok())
                        .collect()
                } else {
                    categories
                        .into_string()
                        .unwrap_or_default()
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .collect()
                };
                let (tx, rx) = std::sync::mpsc::channel();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().expect("failed to create runtime");
                    let result = rt.block_on(async { classify_text(&text, &cat_list).await });
                    let _ = tx.send(result);
                });
                match rx.recv_timeout(Duration::from_secs(30)) {
                    Ok(Ok(result)) => Ok(result),
                    Ok(Err(e)) => Err(Box::new(EvalAltResult::ErrorRuntime(
                        format!("CLASSIFY failed: {}", e).into(),
                        Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(EvalAltResult::ErrorRuntime(
                        "CLASSIFY timed out".into(),
                        Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid syntax registration");

    debug!("Registered CLASSIFY keyword");
}

async fn translate_text(
    text: &str,
    target_lang: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let llm_url = std::env::var("LLM_URL").unwrap_or_else(|_| "http://localhost:8081".to_string());
    let prompt = format!(
        "Translate to {}. Return ONLY the translation:\n\n{}",
        target_lang, text
    );
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/v1/chat/completions", llm_url))
        .json(&serde_json::json!({
            "model": "default",
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.1
        }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    if let Some(content) = response["choices"][0]["message"]["content"].as_str() {
        return Ok(content.trim().to_string());
    }
    Ok(text.to_string())
}

async fn perform_ocr(image_path: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let botmodels_url =
        std::env::var("BOTMODELS_URL").unwrap_or_else(|_| "http://localhost:8001".to_string());
    let client = reqwest::Client::new();
    let image_data = if image_path.starts_with("http") {
        client.get(image_path).send().await?.bytes().await?.to_vec()
    } else {
        std::fs::read(image_path)?
    };
    let response = client
        .post(format!("{}/ocr", botmodels_url))
        .header("Content-Type", "application/octet-stream")
        .body(image_data)
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    if let Some(text) = response["text"].as_str() {
        return Ok(text.to_string());
    }
    Ok(String::new())
}

async fn analyze_sentiment(
    text: &str,
) -> Result<Dynamic, Box<dyn std::error::Error + Send + Sync>> {
    let llm_url = std::env::var("LLM_URL").unwrap_or_else(|_| "http://localhost:8081".to_string());
    let prompt = format!(
        r#"Analyze sentiment. Return JSON only:
{{"sentiment":"positive/negative/neutral","score":-100 to 100,"urgent":true/false}}

Text: {}"#,
        text
    );
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/v1/chat/completions", llm_url))
        .json(&serde_json::json!({
            "model": "default",
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.1
        }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    if let Some(content) = response["choices"][0]["message"]["content"].as_str() {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(content) {
            let mut result = Map::new();
            result.insert(
                "sentiment".into(),
                Dynamic::from(
                    parsed["sentiment"]
                        .as_str()
                        .unwrap_or("neutral")
                        .to_string(),
                ),
            );
            result.insert(
                "score".into(),
                Dynamic::from(parsed["score"].as_i64().unwrap_or(0)),
            );
            result.insert(
                "urgent".into(),
                Dynamic::from(parsed["urgent"].as_bool().unwrap_or(false)),
            );
            return Ok(Dynamic::from(result));
        }
    }
    Ok(analyze_sentiment_quick(text))
}

fn analyze_sentiment_quick(text: &str) -> Dynamic {
    let text_lower = text.to_lowercase();
    let positive = [
        "good",
        "great",
        "excellent",
        "amazing",
        "wonderful",
        "love",
        "happy",
        "thank",
        "thanks",
        "awesome",
        "perfect",
        "best",
    ];
    let negative = [
        "bad",
        "terrible",
        "awful",
        "horrible",
        "hate",
        "angry",
        "frustrated",
        "disappointed",
        "worst",
        "broken",
        "fail",
        "problem",
    ];
    let urgent = [
        "urgent",
        "asap",
        "immediately",
        "emergency",
        "critical",
        "help",
    ];
    let pos_count = positive.iter().filter(|w| text_lower.contains(*w)).count();
    let neg_count = negative.iter().filter(|w| text_lower.contains(*w)).count();
    let is_urgent = urgent.iter().any(|w| text_lower.contains(*w));
    let sentiment = match pos_count.cmp(&neg_count) {
        std::cmp::Ordering::Greater => "positive",
        std::cmp::Ordering::Less => "negative",
        std::cmp::Ordering::Equal => "neutral",
    };
    let score = ((pos_count as i64 - neg_count as i64) * 100)
        / (pos_count as i64 + neg_count as i64 + 1).max(1);
    let mut result = Map::new();
    result.insert("sentiment".into(), Dynamic::from(sentiment.to_string()));
    result.insert("score".into(), Dynamic::from(score));
    result.insert("urgent".into(), Dynamic::from(is_urgent));
    Dynamic::from(result)
}

async fn classify_text(
    text: &str,
    categories: &[String],
) -> Result<Dynamic, Box<dyn std::error::Error + Send + Sync>> {
    let llm_url = std::env::var("LLM_URL").unwrap_or_else(|_| "http://localhost:8081".to_string());
    let cats = categories.join(", ");
    let prompt = format!(
        r#"Classify into one of: {}
Return JSON: {{"category":"chosen_category","confidence":0.0-1.0}}

Text: {}"#,
        cats, text
    );
    let client = reqwest::Client::new();
    let response = client
        .post(format!("{}/v1/chat/completions", llm_url))
        .json(&serde_json::json!({
            "model": "default",
            "messages": [{"role": "user", "content": prompt}],
            "temperature": 0.1
        }))
        .send()
        .await?
        .json::<serde_json::Value>()
        .await?;
    if let Some(content) = response["choices"][0]["message"]["content"].as_str() {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(content) {
            let mut result = Map::new();
            result.insert(
                "category".into(),
                Dynamic::from(
                    parsed["category"]
                        .as_str()
                        .unwrap_or_else(|| {
                            categories.first().map(|s| s.as_str()).unwrap_or("unknown")
                        })
                        .to_string(),
                ),
            );
            result.insert(
                "confidence".into(),
                Dynamic::from(parsed["confidence"].as_f64().unwrap_or(0.5)),
            );
            return Ok(Dynamic::from(result));
        }
    }
    let mut result = Map::new();
    result.insert(
        "category".into(),
        Dynamic::from(
            categories
                .first()
                .map(|s| s.as_str())
                .unwrap_or("unknown")
                .to_string(),
        ),
    );
    result.insert("confidence".into(), Dynamic::from(0.0_f64));
    Ok(Dynamic::from(result))
}
