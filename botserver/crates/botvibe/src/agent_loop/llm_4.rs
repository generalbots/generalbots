//! `agent_loop::llm_4` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

impl AgentLoop {
    pub(crate) async fn read_body_inner(resp: &mut reqwest::Response) -> Result<serde_json::Value, String> {
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
            serde_json::from_str(&text).map_err(|e| format!("Failed to parse LLM response: {e}"))
        }
    }

    /// Plain (non-stream, no `tools`) fallback for providers that reject the
    /// `tools` field (Issue #794).
    pub(crate) async fn call_llm_plain(
        &self,
        api_url: &str,
        api_key: &str,
        model: &str,
        system: &str,
        prompt: &str,
    ) -> Result<(String, Option<LlmUsage>), String> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(LLM_REQUEST_TIMEOUT_SECS))
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

    pub(crate) fn is_local_windows_llm(api_url: &str) -> bool {
        if !cfg!(windows) {
            return false;
        }
        let is_loopback = reqwest::Url::parse(api_url)
            .ok()
            .and_then(|url| url.host_str().map(str::to_string))
            .is_some_and(|host| matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1"));
        is_loopback
    }

    pub(crate) fn local_forced_tool(api_url: &str, run: &VibeRun) -> Option<&'static str> {
        if !Self::is_local_windows_llm(api_url) {
            return None;
        }

        let intent = run.intent.to_ascii_lowercase();
        if intent.contains("publish") || intent.contains("deploy") {
            return if run
                .tool_calls
                .iter()
                .any(|call| call.tool_name == "publish/project")
            {
                None
            } else {
                Some("publish/project")
            };
        }
        if intent.contains("title") {
            return if run
                .tool_calls
                .iter()
                .any(|call| call.tool_name == "file/set-title")
            {
                None
            } else {
                Some("file/set-title")
            };
        }
        let is_file_edit = [
            "change",
            "edit",
            "modify",
            "rename",
            "update",
            "make",
            "set ",
            "title",
            "color",
            "colour",
            "theme",
            "background",
        ]
        .iter()
        .any(|word| intent.contains(word));
        if is_file_edit {
            for tool in ["file/list", "file/read", "file/replace"] {
                if !run.tool_calls.iter().any(|call| call.tool_name == tool) {
                    return Some(tool);
                }
            }
        }
        None
    }

    /// True when the LLM error is a deterministic client rejection (bad key,
    /// wrong model, malformed request) that retrying will not fix. 408 (request
    /// timeout) and 429 (rate limit) remain retryable; 400/401/403/404/422 and
    /// the like are not.
    pub(crate) fn is_non_retryable_llm_error(e: &str) -> bool {
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
    pub(crate) fn last_verdict(response: &str) -> Option<bool> {
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

    pub(crate) fn parse_tool_calls(&self, llm_response: &str) -> Vec<ExtractedToolCall> {
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
                                    tool_name: restore_tool_name_from_llm(&name),
                                    arguments: args,
                                })
                            })
                            .collect();
                    }
                }
            }
        }
        // #1276 — text-embedded tool calls: some providers (NVIDIA nemotron
        // observed on dev) ignore `tool_choice:"required"` and emit the call
        // inside `content` as `[[ {"name": "...", "parameters": {...}} ]]`
        // or a bare array, with no native `tool_calls`. Accept these
        // variants so the run still executes the requested work.
        if let Some(calls) = Self::parse_embedded_tool_calls(llm_response) {
            return calls;
        }
        Vec::new()
    }

    /// Parse tool calls embedded in content text: a bracketed call block
    /// (`[[ {...} ]]` — nemotron style), a bare JSON array of calls, or a
    /// single JSON object carrying a name-ish key plus arguments/parameters.
    /// Returns None when no recognizable call is present so the caller can
    /// treat the message as plain prose.
    pub(crate) fn parse_embedded_tool_calls(response: &str) -> Option<Vec<ExtractedToolCall>> {
        if let Some(arr_text) = extract_json_array(response) {
            if let Ok(serde_json::Value::Array(items)) =
                serde_json::from_str::<serde_json::Value>(&arr_text)
            {
                let mut calls = Vec::new();
                for item in items {
                    // `[[ {...} ]]` nests the call one level deep.
                    match item {
                        serde_json::Value::Array(nested) => {
                            for v in nested {
                                if let Some(c) = Self::call_from_value(&v) {
                                    calls.push(c);
                                }
                            }
                        }
                        v => {
                            if let Some(c) = Self::call_from_value(&v) {
                                calls.push(c);
                            }
                        }
                    }
                }
                if !calls.is_empty() {
                    return Some(calls);
                }
            }
        }
        if let Some(obj_text) = extract_json_object(response) {
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&obj_text) {
                if let Some(c) = Self::call_from_value(&v) {
                    return Some(vec![c]);
                }
            }
        }
        None
    }

    pub(crate) async fn verify_latest(&self, context: &mut crate::types::VibeContext, run: &VibeRun) -> bool {
        let question = "Verify the latest tool results above are consistent and complete. Reply with exactly VERIFIED or FAILED.";
        context.add_user_message(question.to_string());
        let (model, api_key, api_url) = self.resolve_llm(run);
        let system = self
            .prompt_manager
            .system_prompt_for(run.use_case, &run.config.lang);
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
}
