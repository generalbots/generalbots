use crate::permissions::{PermissionEngine, PermissionEngineRef, PermissionMode};
use crate::prompt_manager::VibePromptManager;
use crate::skills::SkillStore;
use crate::telemetry::{ToolCallRecord, VibeTelemetry};
use crate::tool_executor::VibeToolExecutor;
use crate::types::{VibeProgressEvent, VibeRun, VibeRunState, VibeState, VibeToolCall};
use log::{error, info, warn};
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use uuid::Uuid;
const DEFAULT_MAX_STEPS: u32 = 50;
const DEFAULT_TIMEOUT_SECS: u64 = 300;

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
                    run.transition(VibeRunState::Completed);
                    self.telemetry.record_run_completion(run, 0, None, 0.0).await;
                    self.broadcast_event(run, "completed", "Agent loop completed successfully", 100);
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

    async fn run_loop(&self, run: &mut VibeRun, max_steps: u32) {
        let mut context = self.prompt_manager.build_context(run.use_case, &run.intent, &[]);
        context.run_id = run.run_id;

        let triggered = self.skills.auto_trigger(&run.intent).await;
        if !triggered.is_empty() {
            context.kb_references = triggered
                .iter()
                .map(|s| s.content.clone())
                .collect();
        }

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

            let llm_response = match self.call_llm(&context, run).await {
                Ok(response) => response,
                Err(e) => {
                    error!("LLM call failed at step {}: {}", step, e);
                    run.transition(VibeRunState::Failed);
                    run.error = Some(format!("LLM call failed at step {step}: {e}"));
                    self.telemetry.record_run_completion(run, 0, None, 0.0).await;
                    return;
                }
            };

            let tool_calls = self.parse_tool_calls(&llm_response);

            if tool_calls.is_empty() {
                context.add_assistant_message(llm_response);
                run.transition(VibeRunState::Completed);
                self.telemetry.record_run_completion(run, 0, None, 0.0).await;
                self.broadcast_event(run, "completed", "Agent finished — no more tool calls", 100);
                return;
            }

            context.add_assistant_message(format!(
                "Proposed {} tool call(s): {}",
                tool_calls.len(),
                tool_calls
                    .iter()
                    .map(|tc| tc.tool_name.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));

            for tc in &tool_calls {
                self.process_tool_call(run, &mut context, tc, step, max_steps)
                    .await;
            }

            tokio::task::yield_now().await;
        }

        if run.state == VibeRunState::Running {
            run.transition(VibeRunState::Completed);
            self.telemetry.record_run_completion(run, 0, None, 0.0).await;
            self.broadcast_event(run, "completed", "Max steps reached — loop completed", 100);
        }
    }

    async fn call_llm(
        &self,
        context: &crate::types::VibeContext,
        run: &VibeRun,
    ) -> Result<String, String> {
        let prompt = self.prompt_manager.compose_prompt(context, &run.intent);
        let system = self.prompt_manager.system_prompt_for(run.use_case);
        let model = run.config.model.clone().unwrap_or_else(|| {
            std::env::var("LLM_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string())
        });
        let api_key = std::env::var("LLM_KEY").unwrap_or_default();
        let api_url = std::env::var("LLM_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1/chat/completions".to_string());

        let client = reqwest::Client::new();
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
            .post(&api_url)
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

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| format!("Failed to parse LLM response: {e}"))?;

        let content = json["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("")
            .to_string();

        if content.is_empty() {
            return Err("LLM returned empty content".to_string());
        }

        Ok(content)
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

    async fn process_tool_call(
        &self,
        run: &mut VibeRun,
        context: &mut crate::types::VibeContext,
        extracted: &ExtractedToolCall,
        step: u32,
        max_steps: u32,
    ) {
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

        self.telemetry
            .record_tool_call(ToolCallRecord {
                run_id: run.run_id,
                use_case: run.use_case,
                tool_name: tool_call.tool_name.clone(),
                latency_ms: 0,
                tokens: None,
                cost: 0.0,
                success: false,
                error: None,
            })
            .await;

        self.broadcast_event(
            run,
            "executing_tool",
            &format!("Executing: {}", tool_call.tool_name),
            ((step as f64 / max_steps as f64) * 100.0) as u8,
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
                return;
            }
            tool_call.approved = true;
            run.transition(VibeRunState::Running);
        }

        let start = tokio::time::Instant::now();
        match self
            .tool_executor
            .execute(&mut tool_call, run.use_case, self.state.as_ref())
            .await
        {
            Ok(()) => {
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
                            format!("Success: {}", r.data)
                        } else {
                            format!("Failed: {}", r.error.as_deref().unwrap_or("unknown"))
                        }
                    })
                    .unwrap_or_else(|| "No result".to_string());

                context.add_assistant_message(format!(
                    "Tool {} result: {result_summary}",
                    tool_call.tool_name
                ));
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
            }
        }

        run.tool_calls.push(tool_call);
    }

    async fn wait_for_approval(&self, run_id: Uuid) -> bool {
        let approval_timeout = Duration::from_secs(300);
        let start = tokio::time::Instant::now();

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
