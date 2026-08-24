//! #1162 — INVOKE keyword family: call a tenant integration action from
//! BASIC, mirroring the LLM `integrations.invoke` command surface.
//!
//! Forms (function-style, per the vibe_agent precedent — Rhai keys custom
//! syntax by the first token):
//!   invoke(provider, action)                       -> summary string
//!   invoke(provider, action, params_json)          -> full result JSON
//!
//! Transport: POST /api/bots/{bot_id}/integration-actions/invoke on the
//! local botserver with the internal token; credentials stay Vault-side.

use rhai::{Dynamic, Engine};

const DEFAULT_BASE: &str = "http://localhost:8080";

fn base_url() -> String {
    std::env::var("INVOKE_API_URL").unwrap_or_else(|_| DEFAULT_BASE.to_string())
}

fn bot_id() -> String {
    std::env::var("GB_BOT_ID").unwrap_or_default()
}

fn http_invoke(
    provider: String,
    action: String,
    params: serde_json::Value,
) -> Result<serde_json::Value, String> {
    let bid = bot_id();
    if bid.is_empty() {
        return Err("GB_BOT_ID is not configured for this runtime".into());
    }
    let url = format!("{base_url}/api/bots/{bid}/integration-actions/invoke");
    let internal_token = std::env::var("INTERNAL_API_TOKEN").unwrap_or_default();
    let (tx, rx) = std::sync::mpsc::channel();

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread().enable_all().build() {
            Ok(rt) => rt,
            Err(e) => {
                let _ = tx.send(Err(format!("runtime: {e}")));
                return;
            }
        };
        let result = rt.block_on(async move {
            let mut builder = reqwest::Client::new()
                .post(&url)
                .header("Content-Type", "application/json");
            if !internal_token.is_empty() {
                builder = builder.header("X-Internal-Token", internal_token);
            }
            let resp = builder
                .json(&serde_json::json!({
                    "provider": provider,
                    "action": action,
                    "params": params,
                }))
                .send()
                .await
                .map_err(|e| format!("invoke request failed: {e}"))?;
            let status = resp.status();
            let text = resp.text().await.map_err(|e| format!("read: {e}"))?;
            if !status.is_success() {
                return Err(format!("invoke returned {status}: {text}"));
            }
            serde_json::from_str::<serde_json::Value>(&text)
                .map_err(|e| format!("parse: {e}"))
        });
        let _ = tx.send(result);
    });

    rx.recv()
        .map_err(|e| format!("channel error: {e}"))?
}

fn summarize(v: &serde_json::Value) -> String {
    let ok = !v.to_string().contains("\"error\"");
    if ok { "Invoked successfully".to_string() } else { "Invocation reported an error".to_string() }
}

pub fn register_invoke_keywords(engine: &mut Engine) {
    engine.register_fn(
        "invoke",
        |provider: String, action: String| -> Dynamic {
            match http_invoke(provider.clone(), action.clone(), serde_json::json!({})) {
                Ok(v) => Dynamic::from(format!("{} {}.{}: {}", summarize(&v), provider, action, v)),
                Err(e) => Dynamic::from(format!("INVOKE failed: {e}")),
            }
        },
    );

    engine.register_fn(
        "invoke",
        |provider: String, action: String, params: String| -> Dynamic {
            let parsed: serde_json::Value = serde_json::from_str(&params)
                .unwrap_or_else(|_| serde_json::json!({ "raw": params }));
            match http_invoke(provider.clone(), action.clone(), parsed) {
                Ok(v) => Dynamic::from(format!(
                    "{} {}.{}: {}",
                    summarize(&v),
                    provider,
                    action,
                    v
                )),
                Err(e) => Dynamic::from(format!("INVOKE failed: {e}")),
            }
        },
    );
}
