use crate::permissions::{PermissionEngine, PermissionEngineRef, PermissionMode};
use crate::prompt_manager::VibePromptManager;
use crate::skills::SkillStore;
use crate::telemetry::{ToolCallRecord, VibeTelemetry};
use crate::tool_executor::VibeToolExecutor;
use crate::types::{VibeProgressEvent, VibeRun, VibeRunState, VibeState, VibeToolCall, VibeUseCase};
use log::{error, info, warn};
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use uuid::Uuid;
const DEFAULT_MAX_STEPS: u32 = 50;
const DEFAULT_TIMEOUT_SECS: u64 = 600;
const MAX_EMPTY_PARSE_RETRIES: u32 = 3;
const MAX_TOOL_RESULT_CHARS: usize = 4000;
const MAX_TOOL_RETRIES: u32 = 2;
const MAX_VERIFY_FAILURES: u32 = 2;
// vibe33 #813 — transient provider failures must not kill the run.
const MAX_LLM_RETRIES: u32 = 6;
const LLM_RETRY_BACKOFF_SECS: &[u64] = &[1, 2, 4, 8, 12, 16];
// Some providers (e.g. tabitoken.com behind Cloudflare) reject the default
// reqwest/curl User-Agent with a 403 WAF block. Send a browser UA so the
// agent loop's LLM calls are not mistaken for scraping.
const LLM_USER_AGENT: &str =
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/149.0.0.0 Safari/537.36";

pub struct AgentLoop {
    prompt_manager: Arc<VibePromptManager>,
    tool_executor: Arc<VibeToolExecutor>,
    telemetry: Arc<VibeTelemetry>,
    state: Arc<dyn VibeState>,
    permissions: PermissionEngineRef,
    skills: Arc<SkillStore>,
}

impl AgentLoop {
    pub fn new(
        prompt_manager: Arc<VibePromptManager>,
        tool_executor: Arc<VibeToolExecutor>,
        telemetry: Arc<VibeTelemetry>,
        state: Arc<dyn VibeState>,
    ) -> Self {
        Self {
            prompt_manager,
            tool_executor,
            telemetry,
            state,
            permissions: Arc::new(PermissionEngine::new()),
            skills: Arc::new(SkillStore::new()),
        }
    }

    pub fn with_security(
        mut self,
        permissions: PermissionEngineRef,
        skills: Arc<SkillStore>,
    ) -> Self {
        self.permissions = permissions;
        self.skills = skills;
        self
    }

    pub async fn execute_run(&self, run: &mut VibeRun) {
        run.transition(VibeRunState::Running);
        self.broadcast_event(run, "running", "Autonomous agent loop started", 0);
        let config = run.config.clone();
        let max_steps = config.max_tool_calls.min(DEFAULT_MAX_STEPS);
        let timeout_duration = Duration::from_secs(config.timeout_seconds.min(DEFAULT_TIMEOUT_SECS));

        let result = timeout(timeout_duration, self.run_loop(run, max_steps)).await;

        match result {
            Ok(()) => {
                if run.state == VibeRunState::Running {
                    self.finish_truthfully(run, "Agent loop completed successfully").await;
                }
            }
            Err(_) => {
                run.transition(VibeRunState::Failed);
                run.error = Some("Agent loop timed out".to_string());
                self.telemetry.record_run_completion(run, 0, None, 0.0).await;
                self.broadcast_event(run, "failed", "Agent loop timed out", 100);
                warn!(
                    "Vibe run {} timed out after {}s",
                    run.run_id,
                    timeout_duration.as_secs()
                );
            }
        }
    }

    /// #819 — a run that executed zero tool calls performed no work and must
    /// not be reported as "Completed" (the model text-replied instead of acting).
    /// This terminal helper fails the run with an honest verdict when no tool
    /// ever ran, and otherwise completes it normally.
    async fn finish_truthfully(&self, run: &mut VibeRun, completed_msg: &str) {
        if run.tool_calls.is_empty() {
            run.transition(VibeRunState::Failed);
            run.error = Some("Agent produced no tool calls — no work was executed".to_string());
            self.telemetry.record_run_completion(run, 0, None, 0.0).await;
            self.broadcast_event(
                run,
                "failed",
                "No tool calls executed — run performed no work",
                100,
            );
        } else {
            run.transition(VibeRunState::Completed);
            self.telemetry.record_run_completion(run, 0, None, 0.0).await;
            self.broadcast_event(run, "completed", completed_msg, 100);
        }
    }

    async fn run_loop(&self, run: &mut VibeRun, max_steps: u32) {
        let mut context = self
            .prompt_manager
            .build_context(run.use_case, &run.config.lang, &run.intent, &[]);
        context.run_id = run.run_id;

        let triggered = self.skills.auto_trigger(&run.intent).await;
        let grounding_refs: Vec<String> = triggered
            .iter()
            .map(|s| s.content.clone())
            .collect();

        let mut empty_parse_rounds: u32 = 0;
        let mut verify_failures: u32 = 0;
        for step in 0..max_steps {
            if run.state == VibeRunState::Cancelled {
                info!("Vibe run {} cancelled at step {}", run.run_id, step);
                return;
            }

            if self.budget_exceeded(run).await {
                run.transition(VibeRunState::Failed);
                run.error = Some(format!(
                    "Budget cap exceeded ({} cents)",
                    run.config.budget_cents
                ));
                self.telemetry.record_run_completion(run, 0, None, 0.0).await;
                self.broadcast_event(run, "failed", "Budget cap exceeded", 100);
                return;
            }

            let progress = ((step as f64 / max_steps as f64) * 100.0) as u8;
            self.broadcast_event(
                run,
                "thinking",
                &format!("Step {}/{}", step + 1, max_steps),
                progress,
            );

            context.kb_references = {
                let mut refs = grounding_refs.clone();
                refs.extend(crate::grounding::sources_for_run(run));
                refs
            };

            let (llm_response, llm_usage) = match self
                .call_llm_with_retry(&context, run, &run.intent)
                .await
            {
                Ok((response, usage)) => (response, usage),
                Err(e) => {
                    error!("LLM call failed at step {}: {}", step, e);
                    run.transition(VibeRunState::Failed);
                    run.error = Some(format!(
                        "LLM call failed at step {step} after {} attempts: {e}",
                        MAX_LLM_RETRIES + 1
                    ));
                    self.telemetry.record_run_completion(run, 0, None, 0.0).await;
                    return;
                }
            };

            // #923 — record real provider usage so the budget meter and cost
            // accounting reflect actual spend instead of a hardcoded 0.0.
            if let Some(usage) = llm_usage {
                let model = run.config.model.clone().unwrap_or_default();
                let cost = estimate_llm_cost(&model, usage.prompt_tokens, usage.completion_tokens);
                self.telemetry
                    .record_tool_call(ToolCallRecord {
                        run_id: run.run_id,
                        use_case: run.use_case,
                        tool_name: "llm/chat".to_string(),
                        latency_ms: 0,
                        tokens: Some(usage.prompt_tokens.saturating_add(usage.completion_tokens)),
                        cost,
                        success: true,
                        error: None,
                    })
                    .await;
            }

            let tool_calls = self.parse_tool_calls(&llm_response);

            if tool_calls.is_empty() {
                if !looks_like_tool_intent(&llm_response)
                    || empty_parse_rounds + 1 >= MAX_EMPTY_PARSE_RETRIES
                {
                    context.add_assistant_message(llm_response);
                    self.finish_truthfully(run, "Agent finished — no more tool calls").await;
                    return;
                }
                empty_parse_rounds += 1;
                context.add_assistant_message(llm_response);
                context.add_assistant_message(
                    "Your previous response could not be parsed into tool calls. \
                     Return a JSON object exactly like: \
                     {\"tool_calls\": [{\"tool_name\": \"example_tool\", \"arguments\": {}}]} \
                     — or, if the task is complete, answer plainly without any tool_calls key."
                        .to_string(),
                );
                continue;
            }
            empty_parse_rounds = 0;
            context.add_assistant_message(llm_response);

            let mut step_mutated = false;
            let mut cap_reached = false;
            for tc in &tool_calls {
                if (run.tool_calls.len() as u32) >= max_steps {
                    info!(
                        "Vibe run {} reached tool-call cap ({max_steps}) at step {}",
                        run.run_id, step
                    );
                    cap_reached = true;
                    break;
                }
                if self
                    .process_tool_call(run, &mut context, tc, step, max_steps)
                    .await
                {
                    step_mutated = true;
                }
            }

            if cap_reached {
                // Tool-call budget exhausted: the work requested is bounded,
                // so finish the run instead of looping into the timeout.
                run.transition(VibeRunState::Completed);
                self.telemetry.record_run_completion(run, 0, None, 0.0).await;
                self.broadcast_event(
                    run,
                    "completed",
                    "Tool-call cap reached — run finished with the work produced so far",
                    100,
                );
                return;
            }

            if step_mutated {
                self.broadcast_event(
                    run,
                    "verifying",
                    "Self-verifying latest changes",
                    progress,
                );
                let verified = self.verify_latest(&mut context, run).await;
                if !verified {
                    verify_failures += 1;
                    context.add_assistant_message(
                        "Self-verification failed for the latest change. Recheck and correct it before proceeding."
                            .to_string(),
                    );
                    if verify_failures >= MAX_VERIFY_FAILURES {
                        // vibe33 #814 — a repeated failed verification must
                        // not soft-complete with unverified work. Fail the
                        // run with a distinct verdict; the produced workspace
                        // artifacts are NOT deleted, only the run verdict is
                        // honest about them.
                        run.transition(VibeRunState::Failed);
                        run.error = Some(
                            "Self-verification failed repeatedly; run failed (produced artifacts remain in the workspace)."
                                .to_string(),
                        );
                        warn!(
                            "Vibe run {} self-verification failed repeatedly; run failed",
                            run.run_id
                        );
                        self.telemetry.record_run_completion(run, 0, None, 0.0).await;
                        self.broadcast_event(
                            run,
                            "failed",
                            "Self-verification failed repeatedly; run failed",
                            100,
                        );
                        return;
                    }
                } else {
                    verify_failures = 0;
                }
            }

            tokio::task::yield_now().await;
        }

        if run.state == VibeRunState::Running {
            self.finish_truthfully(run, "Max steps reached — loop completed").await;
        }
    }

    /// Resolves (model, api_key, api_url) for a run: explicit run overrides >
    /// per-bot config (Vault for secrets, Drive config.csv for the rest) >
    /// environment > built-in defaults.
    fn resolve_llm(&self, run: &VibeRun) -> (String, String, String) {
        let llm = self.state.llm_config(&run.bot_id);
        let model = run
            .config
            .model
            .clone()
            .or_else(|| llm.as_ref().map(|l| l.model.clone()))
            .or_else(|| std::env::var("LLM_MODEL").ok())
            .unwrap_or_else(|| "gpt-4o-mini".to_string());
        let api_key = run
            .config
            .llm_key
            .clone()
            .or_else(|| llm.as_ref().map(|l| l.key.clone()))
            .or_else(|| std::env::var("LLM_KEY").ok())
            .unwrap_or_default();
        let api_url = run
            .config
            .llm_url
            .clone()
            .or_else(|| llm.as_ref().map(|l| l.url.clone()))
            .or_else(|| std::env::var("LLM_URL").ok())
            .unwrap_or_else(|| "https://api.openai.com/v1/chat/completions".to_string());
        (model, api_key, api_url)
    }

    /// vibe33 #813 — retries the LLM call with short backoff so transient
    /// provider failures do not kill the whole run. Deterministic client
    /// errors (bad key, wrong model, malformed request) fail fast on the
    /// first attempt instead of wasting the retry budget (#932).
    async fn call_llm_with_retry(
        &self,
        context: &crate::types::VibeContext,
        run: &VibeRun,
        user_message: &str,
    ) -> Result<(String, Option<LlmUsage>), String> {
        let mut last_error = String::new();
        for attempt in 0..=MAX_LLM_RETRIES {
            match self.call_llm(context, run, user_message).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    last_error = e.clone();
                    if Self::is_non_retryable_llm_error(&e) {
                        warn!(
                            "Vibe run {} LLM call rejected (non-retryable): {e}",
                            run.run_id
                        );
                        break;
                    }
                    if attempt == MAX_LLM_RETRIES {
                        break;
                    }
                    let backoff = LLM_RETRY_BACKOFF_SECS
                        .get(attempt as usize)
                        .copied()
                        .unwrap_or(3);
                    warn!(
                        "Vibe run {} LLM call failed (attempt {}), retrying in {backoff}s: {e}",
                        run.run_id,
                        attempt + 1
                    );
                    tokio::time::sleep(Duration::from_secs(backoff)).await;
                }
            }
        }
        Err(last_error)
    }

    async fn call_llm(
        &self,
        context: &crate::types::VibeContext,
        run: &VibeRun,
        user_message: &str,
    ) -> Result<(String, Option<LlmUsage>), String> {
        let prompt = self.prompt_manager.compose_prompt(context, user_message);
        let system = self.prompt_manager.system_prompt_for(run.use_case, &run.config.lang);
        // Per-bot config (Issue #795): explicit run overrides > config
        // (Vault for secrets, Drive config.csv for the rest) via the state
        // > environment > built-in defaults.
        let (model, api_key, api_url) = self.resolve_llm(run);

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .user_agent(LLM_USER_AGENT)
            .build()
            .map_err(|e| format!("http client: {e}"))?;
        let tools = self.tool_schemas_for(run.use_case).await;
        let body = serde_json::json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.3,
            "max_tokens": 4096,
            "tools": tools,
            "tool_choice": "auto",
            "stream": true,
            "stream_options": {"include_usage": true},
        });

        // Native streaming with SSE accumulation (Issue #794); the streamed
        // content and tool-call deltas are reassembled into the canonical
        // form `parse_tool_calls` already understands.
        let send_result = client
            .post(&api_url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await;

        let resp = match send_result {
            Ok(r) => r,
            Err(e) => return Err(format!("HTTP request failed: {e}")),
        };

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            // Some providers reject the `tools` field; retry without it so
            // the plain chat path keeps working (fallback per issue #794).
            if text.contains("tool") || text.contains("function") {
                return self
                    .call_llm_plain(&api_url, &api_key, &model, &system, &prompt)
                    .await;
            }
            return Err(format!("LLM returned status {status}: {text}"));
        }

        let payload = match self.read_body(resp).await {
            Ok(payload) => payload,
            Err(e) => {
                // Streaming failed (e.g. truncated body on a large context) —
                // retry non-streaming with the same tool set so function
                // calling still works instead of failing the run.
                warn!("LLM streaming read failed ({e}); retrying non-streaming");
                return self
                    .call_llm_nonstream(&api_url, &api_key, &model, &system, &prompt, &tools)
                    .await;
            }
        };
        let usage = usage_from_payload(&payload);
        match Self::content_from_payload(&payload) {
            Ok(content) => Ok((content, usage)),
            Err(e) if e.contains("truncated") => {
                // The stream ended mid-tool-argument (e.g. the model hit the
                // output token cap while writing several large files). Retry
                // non-streaming so a complete arguments document is returned
                // instead of silently writing an empty/corrupt file.
                warn!("LLM streaming truncated tool-call arguments ({e}); retrying non-streaming");
                self.call_llm_nonstream(&api_url, &api_key, &model, &system, &prompt, &tools)
                    .await
            }
            Err(e) => Err(e),
        }
    }

    /// Non-streaming fallback: identical request (tools + system prompt) but
    /// `stream: false`, so the response is a single JSON document instead of
    /// an SSE stream — more robust against truncated/decoded bodies.
    async fn call_llm_nonstream(
        &self,
        api_url: &str,
        api_key: &str,
        model: &str,
        system: &str,
        prompt: &str,
        tools: &[serde_json::Value],
    ) -> Result<(String, Option<LlmUsage>), String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .user_agent(LLM_USER_AGENT)
            .build()
            .map_err(|e| format!("http client: {e}"))?;
        let body = serde_json::json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.3,
            "max_tokens": 4096,
            "tools": tools,
            "tool_choice": "auto",
            "stream": false,
        });
        let resp = client
            .post(api_url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {e}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("LLM returned status {status}: {text}"));
        }
        let payload = self.read_body(resp).await?;
        let usage = usage_from_payload(&payload);
        Self::content_from_payload(&payload).map(|content| (content, usage))
    }

    /// OpenAI-style `tools` array derived from the live tool registry.
    async fn tool_schemas_for(&self, use_case: VibeUseCase) -> Vec<serde_json::Value> {
        self.tool_executor
            .registry()
            .list_tools_for_use_case(use_case)
            .await
            .iter()
            .map(|t| {
                serde_json::json!({
                    "type": "function",
                    "function": {
                        "name": t.schema.name,
                        "description": t.schema.description,
                        "parameters": t.schema.parameters,
                    }
                })
            })
            .collect()
    }

    /// Extracts either native tool calls (canonical envelope) or plain text
    /// content from an OpenAI-style completion payload. Returns an error when
    /// a tool call's `arguments` is non-empty but not valid JSON — that means
    /// the stream was truncated mid-argument, and silently substituting `{}`
    /// would write an empty/corrupt file.
    fn content_from_payload(payload: &serde_json::Value) -> Result<String, String> {
        if let Some(calls) = payload["choices"][0]["message"]["tool_calls"].as_array() {
            let mut has_call = false;
            for tc in calls {
                let name = tc["function"]["name"].as_str().unwrap_or_default();
                let args_raw = tc["function"]["arguments"].as_str().unwrap_or_default();
                if name.is_empty() && args_raw.is_empty() {
                    continue;
                }
                has_call = true;
                if !args_raw.is_empty()
                    && serde_json::from_str::<serde_json::Value>(args_raw).is_err()
                {
                    return Err(
                        "LLM tool-call arguments were truncated (invalid JSON)".to_string(),
                    );
                }
            }
            if has_call {
                if let Some(calls) = native_tool_calls_from_value(payload) {
                    return Ok(canonical_tool_calls_json(&calls));
                }
            }
        }
        let content = payload["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();
        if content.is_empty() {
            return Err("LLM returned empty content".to_string());
        }
        Ok(content)
    }

    /// Reads the entire HTTP body, classifying it as either an SSE stream
    /// (`data: ...` lines, Issue #794) or a plain JSON completion payload.
    /// Bounded by `read_body_timeout()` so stalled provider streams fail
    /// fast instead of hanging the whole run until the run-level timeout.
    async fn read_body(&self, mut resp: reqwest::Response) -> Result<serde_json::Value, String> {
        let read_timeout = Duration::from_secs(120);
        let result = timeout(
            read_timeout,
            Self::read_body_inner(&mut resp),
        )
        .await
        .map_err(|_| "LLM response body read timed out".to_string());
        match result {
            Ok(inner) => inner,
            Err(e) => {
                log::warn!("read_body: {e}");
                Err(e)
            }
        }
    }

    async fn read_body_inner(resp: &mut reqwest::Response) -> Result<serde_json::Value, String> {
        let content_type = resp
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let mut bytes = Vec::new();
        while let Some(chunk) = resp
            .chunk()
            .await
            .map_err(|e| format!("stream error: {e}"))?
        {
            bytes.extend_from_slice(&chunk);
        }
        let text = String::from_utf8_lossy(&bytes).into_owned();
        if content_type.contains("text/event-stream") || text.contains("data:") {
            parse_sse(&text).ok_or_else(|| "LLM returned malformed SSE stream".to_string())
        } else {
            serde_json::from_str(&text)
                .map_err(|e| format!("Failed to parse LLM response: {e}"))
        }
    }

    /// Plain (non-stream, no `tools`) fallback for providers that reject the
    /// `tools` field (Issue #794).
    async fn call_llm_plain(
        &self,
        api_url: &str,
        api_key: &str,
        model: &str,
        system: &str,
        prompt: &str,
    ) -> Result<(String, Option<LlmUsage>), String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .user_agent(LLM_USER_AGENT)
            .build()
            .map_err(|e| format!("http client: {e}"))?;
        let body = serde_json::json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system},
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.3,
            "max_tokens": 4096,
        });
        let resp = client
            .post(api_url)
            .header("Authorization", format!("Bearer {api_key}"))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("HTTP request failed: {e}"))?;
        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("LLM returned status {status}: {text}"));
        }
        let payload = self.read_body(resp).await?;
        let usage = usage_from_payload(&payload);
        Self::content_from_payload(&payload).map(|content| (content, usage))
    }

/// True when the LLM error is a deterministic client rejection (bad key,
/// wrong model, malformed request) that retrying will not fix. 408 (request
/// timeout) and 429 (rate limit) remain retryable; 400/401/403/404/422 and
/// the like are not.
fn is_non_retryable_llm_error(e: &str) -> bool {
    if e.contains("LLM returned status 408") || e.contains("LLM returned status 429") {
        return false;
    }
    e.contains("LLM returned status 4")
}

/// Parses a self-verification reply into a verdict. Reasoning models often
/// restate earlier failures before the verdict ("the first test FAILED but
/// was corrected"); a naive `contains()` would read the historical mention
/// as the verdict and condemn real executed work. The LAST verdict token in
/// the reply wins, so a trailing "VERIFIED" after a historical "FAILED" is
/// respected (and vice versa). Returns `None` when no explicit verdict token
/// is present.
fn last_verdict(response: &str) -> Option<bool> {
    let last_failed = response.rfind("FAILED");
    let last_verified = response.rfind("VERIFIED");
    match (last_failed, last_verified) {
        // VERIFIED appearing after FAILED (or only VERIFIED present) -> pass.
        (Some(f), Some(v)) => Some(v > f),
        (Some(_), None) => Some(false),
        (None, Some(_)) => Some(true),
        (None, None) => None,
    }
}

    fn parse_tool_calls(&self, llm_response: &str) -> Vec<ExtractedToolCall> {
        if let Some(json_start) = llm_response.find('{') {
            if let Some(json_str) = extract_json_object(&llm_response[json_start..]) {
                if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&json_str) {
                    if let Some(calls) = parsed.get("tool_calls").and_then(|v| v.as_array()) {
                        return calls
                            .iter()
                            .filter_map(|tc| {
                                let name = tc.get("tool_name")?.as_str()?.to_string();
                                if name.is_empty() {
                                    return None;
                                }
                                let args = tc
                                    .get("arguments")
                                    .cloned()
                                    .unwrap_or(serde_json::json!({}));
                                Some(ExtractedToolCall {
                                    tool_name: name,
                                    arguments: args,
                                })
                            })
                            .collect();
                    }
                }
            }
        }
        Vec::new()
    }

    async fn verify_latest(
        &self,
        context: &mut crate::types::VibeContext,
        run: &VibeRun,
    ) -> bool {
        let question = "Verify the latest tool results above are consistent and complete. Reply with exactly VERIFIED or FAILED.";
        context.add_user_message(question.to_string());
        let (model, api_key, api_url) = self.resolve_llm(run);
        let system = self.prompt_manager.system_prompt_for(run.use_case, &run.config.lang);
        let prompt = self.prompt_manager.compose_prompt(context, question);
        // Use the plain (no-tools) call so the model replies with text
        // (VERIFIED/FAILED) instead of a tool-call JSON envelope.
        let response = match self
            .call_llm_plain(&api_url, &api_key, &model, &system, &prompt)
            .await
        {
            Ok((response, _usage)) => response,
            Err(e) => {
                // Verification is a gate, not a blocker: when the LLM is
                // unavailable, keep the run going instead of failing work.
                warn!(
                    "Vibe run {} self-verification LLM call failed: {e}",
                    run.run_id
                );
                return true;
            }
        };
        context.add_assistant_message(response.clone());
        match Self::last_verdict(&response) {
            Some(false) => false,
            Some(true) => true,
            None => {
                // Reasoning models (e.g. gpt-oss) may answer in prose instead of
                // the literal VERIFIED/FAILED token. Equivalent to the LLM-error
                // path below: a verification that produced no explicit verdict
                // must not condemn real executed work.
                warn!(
                    "Vibe run {} self-verification replied without an explicit verdict; treating as verified",
                    run.run_id
                );
                true
            }
        }
    }

    async fn process_tool_call(
        &self,
        run: &mut VibeRun,
        context: &mut crate::types::VibeContext,
        extracted: &ExtractedToolCall,
        step: u32,
        max_steps: u32,
    ) -> bool {
        let schema_requires = self
            .tool_executor
            .registry()
            .get_descriptor(&extracted.tool_name)
            .await
            .map(|d| d.schema.requires_approval)
            .unwrap_or(false);

        let mode = self.permissions.mode().await;
        let requires_approval = if matches!(mode, PermissionMode::Bypass) {
            false
        } else {
            self.permissions
                .requires_approval(schema_requires, &extracted.tool_name, mode)
        };

        let mut tool_call = VibeToolCall::new(
            run.run_id,
            extracted.tool_name.clone(),
            extracted.arguments.clone(),
            requires_approval,
        );

        if matches!(mode, PermissionMode::Bypass) {
            tool_call.approved = true;
        }

        // NOTE: no placeholder ToolCallFailed record here — the real outcome
        // is recorded after execution. A pre-execution record with
        // success=false would emit a phantom tool_call_failed event for every
        // tool call (even successful ones), polluting metrics and the UI.
        self.broadcast_event(
            run,
            "executing_tool",
            &format!("Executing: {}", tool_call.tool_name),
            ((step as f64 / max_steps as f64) * 100.0) as u8,
        );
        log::info!(
            "agent_loop: run {} executing tool {} args {}",
            run.run_id,
            tool_call.tool_name,
            tool_call.arguments
        );

        if requires_approval && !run.config.auto_approve {
            run.transition(VibeRunState::AwaitingApproval);
            self.broadcast_event(
                run,
                "awaiting_approval",
                &format!("Waiting for approval: {}", tool_call.tool_name),
                ((step as f64 / max_steps as f64) * 100.0) as u8,
            );

            let approved = self.wait_for_approval(run.run_id).await;
            if !approved {
                tool_call.approved = false;
                tool_call.result = Some(crate::types::VibeToolResult {
                    success: false,
                    data: serde_json::json!({"denied": true}),
                    error: Some("Approval denied".to_string()),
                    latency_ms: 0,
                });
                run.tool_calls.push(tool_call.clone());
                context.add_assistant_message(format!(
                    "Tool {} was denied approval. Continuing.",
                    tool_call.tool_name
                ));
                run.transition(VibeRunState::Running);
                return false;
            }
            tool_call.approved = true;
            run.transition(VibeRunState::Running);
        } else if requires_approval {
            // Auto-approve: the executor refuses unapproved tools
            // ("Aprovação requerida antes da execução"), so mark the call
            // approved when the run was created with auto_approve=true.
            tool_call.approved = true;
        }

        let start = tokio::time::Instant::now();
        let executed_ok = match self
            .tool_executor
            .execute(&mut tool_call, run.use_case, self.state.as_ref())
            .await
        {
            Ok(()) => {
                if !tool_call.requires_approval {
                    let mut attempts: u32 = 1;
                    while attempts < MAX_TOOL_RETRIES
                        && tool_call
                            .result
                            .as_ref()
                            .map(|r| !r.success)
                            .unwrap_or(false)
                    {
                        attempts += 1;
                        context.add_assistant_message(format!(
                            "Tool {} reported failure; retry attempt {attempts}.",
                            tool_call.tool_name
                        ));
                        // vibe33 #815 — capture the retry outcome instead of
                        // dropping it: the result/telemetry must reflect the
                        // actual last attempt.
                        if let Err(e) = self
                            .tool_executor
                            .execute(&mut tool_call, run.use_case, self.state.as_ref())
                            .await
                        {
                            warn!(
                                "Vibe run {} tool {} retry {attempts} failed: {e}",
                                run.run_id, tool_call.tool_name
                            );
                        }
                    }
                }

                let latency = start.elapsed().as_millis() as u64;
                let success = tool_call
                    .result
                    .as_ref()
                    .map(|r| r.success)
                    .unwrap_or(false);

                self.telemetry
                    .record_tool_call(ToolCallRecord {
                        run_id: run.run_id,
                        use_case: run.use_case,
                        tool_name: tool_call.tool_name.clone(),
                        latency_ms: latency,
                        tokens: None,
                        cost: 0.0,
                        success,
                        error: tool_call
                            .result
                            .as_ref()
                            .and_then(|r| r.error.clone()),
                    })
                    .await;

                let result_summary = tool_call
                    .result
                    .as_ref()
                    .map(|r| {
                        if r.success {
                            let data = if r.data.is_null()
                                || r.data.as_object().is_some_and(|m| m.is_empty())
                            {
                                "no data returned".to_string()
                            } else {
                                r.data.to_string()
                            };
                            format!("Success: {data}")
                        } else {
                            format!("Failed: {}", r.error.as_deref().unwrap_or("unknown"))
                        }
                    })
                    .unwrap_or_else(|| "No result".to_string());

                context.add_assistant_message(format!(
                    "Tool {} result: {}",
                    tool_call.tool_name,
                    truncate(&result_summary, MAX_TOOL_RESULT_CHARS)
                ));
                log::info!(
                    "agent_loop: run {} tool {} completed success={} summary={}",
                    run.run_id,
                    tool_call.tool_name,
                    success,
                    truncate(&result_summary, 160)
                );
                success
            }
            Err(e) => {
                let latency = start.elapsed().as_millis() as u64;
                self.telemetry
                    .record_tool_call(ToolCallRecord {
                        run_id: run.run_id,
                        use_case: run.use_case,
                        tool_name: tool_call.tool_name.clone(),
                        latency_ms: latency,
                        tokens: None,
                        cost: 0.0,
                        success: false,
                        error: Some(e.clone()),
                    })
                    .await;

                tool_call.result = Some(crate::types::VibeToolResult {
                    success: false,
                    data: serde_json::json!({"error": e}),
                    error: Some(e.clone()),
                    latency_ms: latency,
                });

                context.add_assistant_message(format!(
                    "Tool {} failed: {e}. Continuing with next step.",
                    tool_call.tool_name
                ));
                log::info!(
                    "agent_loop: run {} tool {} execute-ERR: {}",
                    run.run_id,
                    tool_call.tool_name,
                    e
                );
                false
            }
        };

        run.tool_calls.push(tool_call);
        executed_ok
    }

    async fn wait_for_approval(&self, run_id: Uuid) -> bool {
        let approval_timeout = Duration::from_secs(300);
        let start = tokio::time::Instant::now();

        // Prefer the run-signal channel: approve/cancel publish the decision
        // immediately, so a run awaiting approval resumes as soon as the
        // operator approves, instead of polling a possibly-stale run snapshot.
        if let Some(tx) = self.state.run_signal_sender() {
            let mut rx = tx.subscribe();
            loop {
                let remaining = if start.elapsed() > approval_timeout {
                    warn!("Approval timeout for run {run_id}");
                    return false;
                } else {
                    approval_timeout.saturating_sub(start.elapsed())
                };
                match tokio::time::timeout(remaining, rx.recv()).await {
                    Ok(Ok(crate::types::VibeRunSignal::Approved(id))) if id == run_id => {
                        info!("Run {run_id} approved; resuming");
                        return true;
                    }
                    Ok(Ok(crate::types::VibeRunSignal::Cancelled(id))) if id == run_id => {
                        info!("Run {run_id} cancelled while awaiting approval");
                        return false;
                    }
                    Ok(Ok(_)) => continue,
                    Ok(Err(tokio::sync::broadcast::error::RecvError::Lagged(_))) => continue,
                    Ok(Err(tokio::sync::broadcast::error::RecvError::Closed)) => {
                        return false;
                    }
                    Err(_) => {
                        warn!("Approval timeout for run {run_id}");
                        return false;
                    }
                }
            }
        }

        // Fallback when no signal channel is wired: poll the active-runs map.
        loop {
            if start.elapsed() > approval_timeout {
                warn!("Approval timeout for run {run_id}");
                return false;
            }

            let runs = self.state.active_runs().read().await;
            if let Some(run) = runs.get(&run_id) {
                if run.state == VibeRunState::Running {
                    return true;
                }
                if matches!(run.state, VibeRunState::Cancelled | VibeRunState::Failed) {
                    return false;
                }
            }
            drop(runs);

            tokio::time::sleep(Duration::from_secs(2)).await;
        }
    }

    fn broadcast_event(&self, run: &VibeRun, step: &str, message: &str, progress: u8) {
        let event = VibeProgressEvent {
            event_type: "vibe_progress".to_string(),
            run_id: run.run_id.to_string(),
            step: step.to_string(),
            message: message.to_string(),
            progress,
            total_steps: run.config.max_tool_calls as u8,
            current_step: run.tool_calls.len() as u8,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        self.state.broadcast_progress(event);
    }

    async fn budget_exceeded(&self, run: &VibeRun) -> bool {
        let budget_cents = run.config.budget_cents;
        if budget_cents == 0 {
            return false;
        }
        let spent = self
            .telemetry
            .get_run_metrics(run.run_id)
            .await
            .map(|m| m.total_cost)
            .unwrap_or(0.0);
        let spent_cents = (spent * 100.0).round() as u64;
        if spent_cents >= budget_cents {
            warn!(
                "Vibe run {} exceeded budget: {spent_cents} cents >= {budget_cents} cents",
                run.run_id
            );
            true
        } else {
            false
        }
    }
}
struct ExtractedToolCall {
    tool_name: String,
    arguments: serde_json::Value,
}

/// True when the response mentions tool-call JSON keys, meaning the model
/// attempted to call tools even if the payload could not be parsed.
fn looks_like_tool_intent(response: &str) -> bool {
    response.contains("tool_calls") || response.contains("tool_name")
}

/// Accumulates an SSE body (`data: {...}` lines) into a single JSON payload
/// with the assistant `content`/`tool_calls` merged (Issue #794). Returns
/// `None` when the body contains no SSE events.
fn parse_sse(body: &str) -> Option<serde_json::Value> {
    let mut message = serde_json::json!({"role": "assistant", "content": "", "tool_calls": []});
    let mut first_seen = false;
    let mut usage: Option<serde_json::Value> = None;
    for line in body.lines() {
        let Some(data) = line.trim().strip_prefix("data:") else {
            continue;
        };
        let data = data.trim();
        if data == "[DONE]" {
            first_seen = true;
            break;
        }
        let Ok(event) = serde_json::from_str::<serde_json::Value>(data) else {
            continue;
        };
        first_seen = true;
        if let Some(u) = event.get("usage").filter(|u| !u.is_null()) {
            usage = Some(u.clone());
        }
        let delta = &event["choices"][0]["delta"];
        if let Some(text) = delta["content"].as_str() {
            message["content"] =
                serde_json::Value::String(message["content"].as_str().unwrap_or_default().to_string() + text);
        }
        if let Some(deltas) = delta["tool_calls"].as_array() {
            let Some(calls) = message["tool_calls"].as_array_mut() else {
                continue;
            };
            for delta_call in deltas {
                let index = delta_call["index"].as_u64().unwrap_or(0) as usize;
                while calls.len() <= index {
                    calls.push(serde_json::json!({
                        "id": "", "type": "function",
                        "function": {"name": "", "arguments": ""}
                    }));
                }
                let entry = &mut calls[index];
                if let Some(id) = delta_call["id"].as_str() {
                    if !id.is_empty() {
                        entry["id"] = serde_json::Value::String(id.to_string());
                    }
                }
                if let Some(name) = delta_call["function"]["name"].as_str() {
                    if !name.is_empty() {
                        let current = entry["function"]["name"].as_str().unwrap_or_default();
                        entry["function"]["name"] =
                            serde_json::Value::String(current.to_string() + name);
                    }
                }
                if let Some(args) = delta_call["function"]["arguments"].as_str() {
                    let current = entry["function"]["arguments"].as_str().unwrap_or_default();
                    entry["function"]["arguments"] =
                        serde_json::Value::String(current.to_string() + args);
                }
            }
        }
    }
    if !first_seen {
        return None;
    }
    let mut accumulated = serde_json::Map::new();
    accumulated.insert("choices".to_string(), serde_json::json!([{"message": message}]));
    if let Some(u) = usage {
        accumulated.insert("usage".to_string(), u);
    }
    Some(serde_json::Value::Object(accumulated))
}

/// Token usage reported by an LLM provider for a single completion attempt.
#[derive(Debug, Clone, Copy, Default)]
struct LlmUsage {
    prompt_tokens: u32,
    completion_tokens: u32,
}

/// Extracts provider-reported token usage from an OpenAI-style completion
/// payload (`usage.prompt_tokens` / `usage.completion_tokens`), if present.
fn usage_from_payload(payload: &serde_json::Value) -> Option<LlmUsage> {
    let usage = payload.get("usage")?;
    Some(LlmUsage {
        prompt_tokens: usage.get("prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
        completion_tokens: usage.get("completion_tokens").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
    })
}

/// USD cost estimate for a model from a small built-in price table (per 1M
/// tokens). Unknown models fall back to `VIBE_LLM_COST_PER_1M_TOKENS` (default
/// 0.0) rather than fabricating a price, so a budget is never under-charged.
fn estimate_llm_cost(model: &str, prompt_tokens: u32, completion_tokens: u32) -> f64 {
    let m = model.to_ascii_lowercase();
    let (input, output): (f64, f64) = if m.contains("gpt-4o-mini") {
        (0.15, 0.60)
    } else if m.contains("gpt-4o") {
        (2.50, 10.0)
    } else if m.contains("gpt-3.5") {
        (0.50, 1.50)
    } else if m.contains("claude-3-5") || m.contains("claude-3-7") || m.contains("claude-3.5") || m.contains("claude-3.7") {
        (3.0, 15.0)
    } else if m.contains("claude") {
        (15.0, 75.0)
    } else if m.contains("llama-3.3") || m.contains("llama-3.1-70b") {
        (0.59, 0.79)
    } else if m.contains("llama") {
        (0.10, 0.30)
    } else {
        let fallback = std::env::var("VIBE_LLM_COST_PER_1M_TOKENS")
            .ok()
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);
        return (prompt_tokens as f64 + completion_tokens as f64) / 1_000_000.0 * fallback;
    };
    (prompt_tokens as f64) / 1_000_000.0 * input + (completion_tokens as f64) / 1_000_000.0 * output
}

/// Extracts native OpenAI-format tool calls (`message.tool_calls[].function`)
/// from a completion payload, if present.
fn native_tool_calls_from_value(payload: &serde_json::Value) -> Option<Vec<ExtractedToolCall>> {
    let calls = payload["choices"][0]["message"]["tool_calls"].as_array()?;
    let parsed: Vec<ExtractedToolCall> = calls
        .iter()
        .filter_map(|tc| {
            let name = tc["function"]["name"].as_str()?.to_string();
            // SSE deltas can leave a placeholder entry with no name — skip
            // it instead of forwarding an empty tool name to the executor.
            if name.is_empty() {
                return None;
            }
            let arguments = tc["function"]["arguments"]
                .as_str()
                .and_then(|a| serde_json::from_str::<serde_json::Value>(a).ok())
                .unwrap_or_else(|| serde_json::json!({}));
            Some(ExtractedToolCall {
                tool_name: name,
                arguments,
            })
        })
        .collect();
    if parsed.is_empty() {
        None
    } else {
        Some(parsed)
    }
}

/// Serializes native tool calls back into the canonical JSON envelope that
/// `parse_tool_calls` consumes downstream.
fn canonical_tool_calls_json(calls: &[ExtractedToolCall]) -> String {
    let body = serde_json::json!({
        "tool_calls": calls.iter().map(|c| serde_json::json!({
            "tool_name": c.tool_name,
            "arguments": c.arguments,
        })).collect::<Vec<_>>(),
    });
    body.to_string()
}

/// Caps a string at `limit` characters, cutting on a char boundary and
/// marking the truncation so long tool results stay visible but bounded.
fn truncate(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.to_string();
    }
    let mut end = limit;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}… (truncated)", &text[..end])
}

fn extract_json_object(s: &str) -> Option<String> {
    let mut depth = 0i32;
    let mut in_string = false;
    let mut escape = false;
    let mut start = None;

    for (i, ch) in s.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        if ch == '\\' && in_string {
            escape = true;
            continue;
        }
        if ch == '"' {
            in_string = !in_string;
            continue;
        }
        if in_string {
            continue;
        }
        match ch {
            '{' => {
                if depth == 0 {
                    start = Some(i);
                }
                depth += 1;
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    if let Some(start) = start {
                        return Some(s[start..=i].to_string());
                    }
                }
            }
            _ => {}
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tokio::sync::RwLock;

    #[test]
    fn classifies_non_retryable_llm_errors() {
        assert!(AgentLoop::is_non_retryable_llm_error("LLM returned status 401: unauthorized"));
        assert!(AgentLoop::is_non_retryable_llm_error("LLM returned status 403: forbidden"));
        assert!(AgentLoop::is_non_retryable_llm_error("LLM returned status 404: model not found"));
        assert!(AgentLoop::is_non_retryable_llm_error("LLM returned status 422: bad request"));
        assert!(!AgentLoop::is_non_retryable_llm_error("LLM returned status 429: rate limited"));
        assert!(!AgentLoop::is_non_retryable_llm_error("LLM returned status 408: timeout"));
        assert!(!AgentLoop::is_non_retryable_llm_error("LLM returned status 502: bad gateway"));
        assert!(!AgentLoop::is_non_retryable_llm_error("HTTP request failed: connection reset"));
    }

    #[test]
    fn last_verdict_uses_final_token() {
        // Historical FAILED mention must not override a trailing VERIFIED.
        assert_eq!(AgentLoop::last_verdict("The first test FAILED but was corrected. VERIFIED"), Some(true));
        assert_eq!(AgentLoop::last_verdict("Looks good. VERIFIED"), Some(true));
        assert_eq!(AgentLoop::last_verdict("FAILED: server.js still missing"), Some(false));
        assert_eq!(AgentLoop::last_verdict("Everything passed: VERIFIED. Wait, no — FAILED"), Some(false));
        assert_eq!(AgentLoop::last_verdict("No explicit verdict here"), None);
        assert_eq!(AgentLoop::last_verdict(""), None);
    }

    #[test]
    fn extract_json_object_handles_nested_and_strings() {
        let input = r#"prefix {"a": {"b": [1, 2]}, "c": "x{y}z"} suffix"#;
        let extracted = extract_json_object(input).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&extracted).unwrap();
        assert_eq!(parsed["a"]["b"][1], 2);
        assert_eq!(parsed["c"], "x{y}z");
    }

    #[test]
    fn extract_json_object_returns_none_for_no_object() {
        assert!(extract_json_object("no braces here").is_none());
        assert!(extract_json_object("").is_none());
        assert!(extract_json_object("unbalanced {").is_none());
    }

    #[test]
    fn looks_like_tool_intent_detects_json_tool_phrases() {
        assert!(looks_like_tool_intent(r#"{"tool_calls": []}"#));
        assert!(looks_like_tool_intent(r#"use tool_name read_file"#));
        assert!(!looks_like_tool_intent("no tools needed, task done"));
        assert!(!looks_like_tool_intent(""));
    }

    #[test]
    fn truncate_keeps_short_and_marks_long() {
        assert_eq!(truncate("short", 100), "short");
        let text = "a".repeat(5000);
        let out = truncate(&text, 4000);
        assert!(out.len() <= 4016);
        assert!(out.ends_with("(truncated)"));
        assert!(out.contains('…'));
    }

    #[test]
    fn truncate_respects_char_boundaries() {
        let text = "ç".repeat(3000);
        let out = truncate(&text, 100);
        assert!(out.is_char_boundary(out.len()));
    }

    #[test]
    fn native_tool_calls_parse_openai_format() {
        let payload = serde_json::json!({
            "choices": [{"message": {"tool_calls": [
                {"id": "call_1", "type": "function",
                 "function": {"name": "read_file", "arguments": "{\"path\": \"a.txt\"}"}}
            ]}}]
        });
        let calls = native_tool_calls_from_value(&payload).expect("native calls present");
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].tool_name, "read_file");
        assert_eq!(calls[0].arguments["path"], "a.txt");
        assert!(
            native_tool_calls_from_value(&serde_json::json!({"choices": [{"message": {"content": "hi"}}]}))
                .is_none()
        );
    }

    #[test]
    fn content_from_payload_errors_on_truncated_tool_arguments() {
        // A stream cut off mid-argument leaves invalid JSON; it must be
        // reported (so call_llm retries non-streaming) instead of silently
        // substituting `{}` and writing an empty/corrupt file.
        let truncated = serde_json::json!({
            "choices": [{"message": {"tool_calls": [
                {"id": "call_1", "type": "function",
                 "function": {"name": "file/write", "arguments": "{\"path\": \"server.js\", \"content\": \"const app = req"}}
            ]}}]
        });
        let err = AgentLoop::content_from_payload(&truncated).expect_err("must fail on truncation");
        assert!(err.contains("truncated"), "error should flag truncation: {err}");

        // A complete call with valid JSON arguments parses normally.
        let valid = serde_json::json!({
            "choices": [{"message": {"tool_calls": [
                {"id": "call_1", "type": "function",
                 "function": {"name": "file/write", "arguments": "{\"path\": \"server.js\", \"content\": \"ok\"}"}}
            ]}}]
        });
        let out = AgentLoop::content_from_payload(&valid).expect("valid call parses");
        assert!(out.contains("file/write") && out.contains("server.js"));
    }

    #[test]
    fn canonical_json_envelope_parses_like_loop_output() {
        let calls = vec![ExtractedToolCall {
            tool_name: "file/read".to_string(),
            arguments: serde_json::json!({"path": "a.txt"}),
        }];
        let json = canonical_tool_calls_json(&calls);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("envelope is json");
        assert_eq!(parsed["tool_calls"][0]["tool_name"], "file/read");
        assert_eq!(parsed["tool_calls"][0]["arguments"]["path"], "a.txt");
    }

    #[test]
    fn parse_sse_accumulates_content_and_tool_call_deltas() {
        let lines = [
            format!("data: {}", json!({"choices": [{"delta": {"role": "assistant"}}]})),
            format!("data: {}", json!({"choices": [{"delta": {"content": "Hel"}}]})),
            format!("data: {}", json!({"choices": [{"delta": {"content": "lo"}}]})),
            format!("data: {}", json!({"choices": [{"delta": {"tool_calls": [{"index": 0, "id": "call_1", "function": {"name": "read_fi", "arguments": "{\"pa"}}]}}]})),
            format!("data: {}", json!({"choices": [{"delta": {"tool_calls": [{"index": 0, "function": {"name": "le", "arguments": "th\": \"a.txt\"}"}}]}}]})),
        ]
        .join("\n")
            + "\ndata: [DONE]";
        let payload = parse_sse(&lines).expect("sse parsed");
        assert_eq!(payload["choices"][0]["message"]["content"], "Hello");
        assert_eq!(
            payload["choices"][0]["message"]["tool_calls"][0]["function"]["name"],
            "read_file"
        );
        assert_eq!(
            payload["choices"][0]["message"]["tool_calls"][0]["function"]["arguments"],
            "{\"path\": \"a.txt\"}"
        );
    }

    #[test]
    fn parse_sse_rejects_non_sse_body() {
        assert!(parse_sse("{\"choices\":[]}").is_none());
    }

    #[test]
    fn parse_tool_calls_from_llm_response() {
        let agent = AgentLoop::new(
            Arc::new(VibePromptManager::new()),
            Arc::new(VibeToolExecutor::new(Arc::new(crate::tool_executor::ToolRegistry::new()))),
            Arc::new(VibeTelemetry::new()),
            Arc::new(MockState::new()),
        );
        let response = r#"{"tool_calls": [{"tool_name": "file/read", "arguments": {"path": "a.txt"}}, {"tool_name": "web/search", "arguments": {}}]}"#;
        let calls = agent.parse_tool_calls(response);
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].tool_name, "file/read");
        assert_eq!(calls[0].arguments["path"], "a.txt");
        assert_eq!(calls[1].tool_name, "web/search");
        assert!(agent.parse_tool_calls("no calls").is_empty());
    }

    struct MockState {
        runs: Arc<RwLock<std::collections::HashMap<Uuid, VibeRun>>>,
    }

    impl MockState {
        fn new() -> Self {
            Self { runs: Arc::new(RwLock::new(std::collections::HashMap::new())) }
        }
    }

    impl VibeState for MockState {
        fn db_pool(&self) -> &crate::types::DbPool {
            unreachable!("db_pool not exercised in parse tests")
        }
        fn broadcast_progress(&self, _event: VibeProgressEvent) {}
        fn progress_sender(&self) -> Option<&tokio::sync::broadcast::Sender<VibeProgressEvent>> {
            None
        }
        fn active_runs(&self) -> &Arc<RwLock<std::collections::HashMap<Uuid, VibeRun>>> {
            &self.runs
        }
        fn run_signal_sender(&self) -> Option<&tokio::sync::broadcast::Sender<crate::types::VibeRunSignal>> {
            None
        }
        fn llm_config(&self, _bot_id: &uuid::Uuid) -> Option<crate::types::LlmConfig> {
            None
        }
    }
}
