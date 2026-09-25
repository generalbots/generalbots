//! `agent_loop::llm_5` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

impl AgentLoop {
    pub(crate) async fn process_tool_call(
        &self,
        run: &mut VibeRun,
        context: &mut crate::types::VibeContext,
        extracted: &ExtractedToolCall,
        step: u32,
        max_steps: u32,
    ) -> ToolStep {
        // Approval concept removed (#1400): every tool executes directly.
        // The permission mode is no longer consulted for gating; when a tool
        // fails or a real decision is needed, the agent narrates it through
        // the Vibe chat (progress events + final verdict) instead of pausing
        // the run in `awaiting_approval`.
        let mode = self.permissions.mode().await;
        let mut tool_call = VibeToolCall::new(
            run.run_id,
            extracted.tool_name.clone(),
            project_scoped_arguments(run, extracted),
            false,
        );
        tool_call.approved = true;
        let _ = mode;

        // Reject malformed model output before asking the user to approve it.
        // Feed the validation error back into the conversation so the next
        // model step can repair the call instead of waiting on useless input.
        if let Err(validation_error) = self
            .tool_executor
            .registry()
            .validate_arguments(&tool_call.tool_name, &tool_call.arguments)
            .await
        {
            tool_call.result = Some(crate::types::VibeToolResult {
                success: false,
                data: serde_json::Value::Null,
                error: Some(validation_error.clone()),
                latency_ms: 0,
            });
            context.add_assistant_message(format!(
                "Tool {} arguments were invalid: {validation_error}. Correct every required argument and retry.",
                tool_call.tool_name
            ));
            self.telemetry
                .record_tool_call(ToolCallRecord {
                    run_id: run.run_id,
                    use_case: run.use_case,
                    tool_name: tool_call.tool_name.clone(),
                    latency_ms: 0,
                    tokens: None,
                    cost: 0.0,
                    success: false,
                    error: Some(validation_error.clone()),
                    metadata: std::collections::HashMap::new(),
                })
                .await;
            run.tool_calls.push(tool_call);
            self.sync_active_run(run).await;
            self.broadcast_event(
                run,
                "tool_validation_failed",
                &format!("Invalid tool arguments: {validation_error}"),
                ((step as f64 / max_steps as f64) * 100.0) as u8,
            );
            return ToolStep::Skipped;
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

        // (Approval block removed (#1400) — tools always run approved.)

        let start = tokio::time::Instant::now();
        // Cap a single tool call: any handler that awaits long-running work
        // (e.g. a deployment, an incus/lifecycle step) without an internal
        // timeout cannot hold the run away from the outer run-level timeout
        // forever. Previously a wedged tool left the run silently "running"
        // until manually cancelled. On expiry the tool is recorded as failed,
        // a corrective note is fed back, and the loop continues.
        const MAX_TOOL_EXEC_SECS: u64 = 300;
        let use_case = run.use_case;
        let run_id = run.run_id;
        let executed_ok = match timeout(Duration::from_secs(MAX_TOOL_EXEC_SECS), async {
            // #1280 — publish is deploy-role gated downstream. The tool runs
            // server-side without a session, so stamp the initiating user's
            // id into the arguments; the deployment handler enforces RBAC for
            // internal callers through this field (never silently privileged).
            if tool_call.tool_name == "publish/project" {
                if let Some(args) = tool_call.arguments.as_object_mut() {
                    args.insert(
                        "on_behalf_of_user".to_string(),
                        serde_json::Value::String(run.user_id.to_string()),
                    );
                }
            }
            match self
                .tool_executor
                .execute(&mut tool_call, use_case, self.state.as_ref())
                .await
            {
            Ok(()) => {
                {
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
                            .execute(&mut tool_call, use_case, self.state.as_ref())
                            .await
                        {
                            warn!(
                                "Vibe run {} tool {} retry {attempts} failed: {e}",
                                run_id, tool_call.tool_name
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
                        run_id,
                        use_case,
                        tool_name: tool_call.tool_name.clone(),
                        latency_ms: latency,
                        tokens: None,
                        cost: 0.0,
                        success,
                        error: tool_call.result.as_ref().and_then(|r| r.error.clone()),
                        metadata: std::collections::HashMap::new(),
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
                    run_id,
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
                        metadata: std::collections::HashMap::new(),
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
                    run_id,
                    tool_call.tool_name,
                    e
                );
                false
            }
            }
        })
        .await
        {
            Ok(ok) => ok,
            Err(_) => {
                // The tool hit the per-call cap. Record it as a failed call,
                // feed a corrective note back so the model stops retrying the
                // same slow step, and keep the run moving.
                let latency = start.elapsed().as_millis() as u64;
                let message = format!(
                    "Tool {} exceeded {MAX_TOOL_EXEC_SECS}s execution budget and was aborted",
                    tool_call.tool_name
                );
                warn!("Vibe run {run_id} {message}");
                self.telemetry
                    .record_tool_call(ToolCallRecord {
                        run_id,
                        use_case,
                        tool_name: tool_call.tool_name.clone(),
                        latency_ms: latency,
                        tokens: None,
                        cost: 0.0,
                        success: false,
                        error: Some(message.clone()),
                        metadata: std::collections::HashMap::new(),
                    })
                    .await;
                tool_call.result = Some(crate::types::VibeToolResult {
                    success: false,
                    data: serde_json::Value::Null,
                    error: Some(message.clone()),
                    latency_ms: latency,
                });
                context.add_assistant_message(format!(
                    "{message}. Do not repeat this step; continue with a smaller, faster one instead."
                ));
                false
            }
        };

        run.tool_calls.push(tool_call);
        if executed_ok {
            ToolStep::Executed
        } else {
            ToolStep::Skipped
        }
    }

    pub(crate) fn broadcast_event(&self, run: &VibeRun, step: &str, message: &str, progress: u8) {
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
}

/// The browser-selected project is authoritative. LLMs sometimes omit the
/// project argument or repeat a display label; allowing that value through
/// would make an "edit calculator" request write to another workspace.
pub(crate) fn project_scoped_arguments(run: &VibeRun, extracted: &ExtractedToolCall) -> serde_json::Value {
    let Some(project) = run.config.project_name.as_deref() else {
        return extracted.arguments.clone();
    };
    let mut args = extracted.arguments.as_object().cloned().unwrap_or_default();
    let tool = extracted.tool_name.as_str();
    if tool.starts_with("file/")
        || tool.starts_with("shell/")
        || tool.starts_with("git/")
        || tool.starts_with("test/")
        || tool.starts_with("logs/")
    {
        args.insert(
            "project".to_string(),
            serde_json::Value::String(project.to_string()),
        );
    }
    #[cfg(target_os = "windows")]
    if matches!(tool, "file/read" | "file/replace")
        && args
            .get("path")
            .and_then(|value| value.as_str())
            .is_none_or(path_requires_repair)
    {
        let previous_read = run.tool_calls.iter().rev().find_map(|call| {
            (call.tool_name == "file/read"
                && call.result.as_ref().is_some_and(|result| result.success))
            .then(|| call.arguments.get("path").and_then(|value| value.as_str()))
            .flatten()
            .filter(|path| !path_requires_repair(path))
            .map(str::to_string)
        });
        let inferred = previous_read.or_else(|| {
            crate::harness::list_rel(project, "", 0)
                .ok()
                .and_then(|entries| preferred_source_path(&entries, &run.intent))
        });
        if let Some(path) = inferred {
            args.insert("path".to_string(), serde_json::Value::String(path));
        }
    }
    if tool == "publish/project" {
        if let Some(project_id) = run.config.project_id.as_deref() {
            args.insert(
                "project_id".to_string(),
                serde_json::Value::String(project_id.to_string()),
            );
        }
    }
    serde_json::Value::Object(args)
}

/// True when the response mentions tool-call JSON keys, meaning the model
/// attempted to call tools even if the payload could not be parsed.
pub(crate) fn looks_like_tool_intent(response: &str) -> bool {
    response.contains("tool_calls") || response.contains("tool_name")
}
