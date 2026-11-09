//! In-VM trace markers (#2046).
//!
//! A generated program reports what it is doing by printing one line per task
//! to stdout:
//!
//! ```text
//! @@GB_TRACE@@ {"task": "install deps", "detail": "pip", "status": "started"}
//! @@GB_TRACE@@ {"task": "install deps", "status": "ok"}
//! ```
//!
//! `exec_in_conversation_streamed` publishes those lines while the program
//! runs, the pipeline turns each one into a `STEP_STARTED` / `STEP_FINISHED`
//! frame, and the run card renders one row per task with its timing. The same
//! lines are stripped from the rendered output: the table replaces them.
//!
//! A line without JSON is treated as a task name in the `started` state, so a
//! program can emit a bare marker when it has no detail to add.

/// Reserved stdout prefix. Anything else on stdout is program output.
pub const TRACE_PREFIX: &str = "@@GB_TRACE@@";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceMarker {
    pub task: String,
    pub detail: String,
    pub status: TraceStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceStatus {
    Started,
    Ok,
    Error,
}

impl TraceStatus {
    /// Status word stored in the `STEP_FINISHED` frame.
    pub fn as_frame_status(self) -> &'static str {
        match self {
            Self::Started => "running",
            Self::Ok => "ok",
            Self::Error => "error",
        }
    }

    fn from_word(word: &str) -> Self {
        match word.to_ascii_lowercase().as_str() {
            "error" | "failed" | "fail" => Self::Error,
            "ok" | "done" | "finished" | "complete" | "completed" => Self::Ok,
            _ => Self::Started,
        }
    }
}

/// Parses one program output line. `None` when the line is ordinary output.
pub fn parse_trace_line(line: &str) -> Option<TraceMarker> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix(TRACE_PREFIX)?.trim();
    if rest.is_empty() {
        return None;
    }
    if rest.starts_with('{') {
        if let Ok(value) = serde_json::from_str::<serde_json::Value>(rest) {
            let task = value
                .get("task")
                .or_else(|| value.get("name"))
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|task| !task.is_empty());
            if let Some(task) = task {
                let detail = value
                    .get("detail")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .trim()
                    .to_string();
                let status = value
                    .get("status")
                    .and_then(|v| v.as_str())
                    .map(TraceStatus::from_word)
                    .unwrap_or(TraceStatus::Started);
                return Some(TraceMarker {
                    task: task.chars().take(120).collect(),
                    detail: detail.chars().take(200).collect(),
                    status,
                });
            }
        }
    }
    // Bare marker: `@@GB_TRACE@@ installing dependencies`.
    let task: String = rest.chars().take(120).collect();
    Some(TraceMarker {
        task,
        detail: String::new(),
        status: TraceStatus::Started,
    })
}

/// Removes every trace line from captured output, keeping the rest verbatim.
pub fn strip_trace_lines(output: &str) -> String {
    let mut kept: Vec<&str> = Vec::new();
    for line in output.lines() {
        if line.trim_start().starts_with(TRACE_PREFIX) {
            continue;
        }
        kept.push(line);
    }
    let mut text = kept.join("\n");
    if output.ends_with('\n') && !text.is_empty() {
        text.push('\n');
    }
    text
}

/// Stable step id for a task name: lowercase, non-alphanumerics collapsed to
/// `-`. The same task always maps to the same row, so `started` and `ok` land
/// on one table entry.
pub fn step_id_for(task: &str) -> String {
    let mut id = String::with_capacity(task.len());
    let mut last_dash = true;
    for ch in task.chars() {
        if ch.is_ascii_alphanumeric() {
            id.push(ch.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            id.push('-');
            last_dash = true;
        }
    }
    let trimmed = id.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "task".to_string()
    } else {
        trimmed.chars().take(60).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_json_marker_carries_task_detail_and_status() {
        let marker = parse_trace_line(
            r#"@@GB_TRACE@@ {"task": "build deck", "detail": "27 slides", "status": "started"}"#,
        )
        .expect("marker");
        assert_eq!(marker.task, "build deck");
        assert_eq!(marker.detail, "27 slides");
        assert_eq!(marker.status, TraceStatus::Started);

        let done = parse_trace_line(r#"@@GB_TRACE@@ {"task":"build deck","status":"ok"}"#)
            .expect("marker");
        assert_eq!(done.status, TraceStatus::Ok);
        assert_eq!(done.status.as_frame_status(), "ok");
    }

    #[test]
    fn a_bare_marker_is_a_started_task() {
        let marker = parse_trace_line("@@GB_TRACE@@ installing dependencies").expect("marker");
        assert_eq!(marker.task, "installing dependencies");
        assert_eq!(marker.status, TraceStatus::Started);
    }

    #[test]
    fn ordinary_output_is_not_a_marker() {
        assert!(parse_trace_line("total 42").is_none());
        assert!(parse_trace_line("@@GB_TRACE@@").is_none());
        assert!(parse_trace_line("  ").is_none());
        // Malformed JSON still yields a usable task name.
        assert_eq!(
            parse_trace_line(r#"@@GB_TRACE@@ {"task": broken"#).map(|m| m.task),
            Some(r#"{"task": broken"#.to_string())
        );
    }

    #[test]
    fn trace_lines_are_stripped_from_rendered_output() {
        let output = "row 1\n@@GB_TRACE@@ {\"task\":\"x\",\"status\":\"ok\"}\nrow 2\n";
        assert_eq!(strip_trace_lines(output), "row 1\nrow 2\n");
        assert_eq!(strip_trace_lines("only output"), "only output");
        assert_eq!(strip_trace_lines("@@GB_TRACE@@ x"), "");
    }

    #[test]
    fn step_ids_are_stable_and_readable() {
        assert_eq!(step_id_for("Build deck"), "build-deck");
        assert_eq!(step_id_for("install deps!"), "install-deps");
        assert_eq!(step_id_for("  "), "task");
        assert_eq!(step_id_for("Build deck"), step_id_for("build   deck"));
    }
}

/// #2046 — turns a streamed output line into `STEP_STARTED`/`STEP_FINISHED`
/// frames. Non-trace lines are ignored: they still reach the user through the
/// captured output at the end of the run.
pub(super) async fn trace_event(
    progress: Option<&super::super::workspace_progress::WorkspaceProgress<'_>>,
    rows: &mut std::collections::HashMap<String, String>,
    event: botagent::stream::ExecEvent,
) {
    let Some(progress) = progress else { return };
    let botagent::stream::ExecEvent::Stdout { line } = event else {
        return;
    };
    let Some(marker) = parse_trace_line(&line) else {
        return;
    };
    let step_name = step_id_for(&marker.task);
    let step = format!("trace:{step_name}");
    match marker.status {
        TraceStatus::Started => {
            rows.insert(step_name, marker.task.clone());
            progress
                .start(
                    super::super::workspace_progress::PHASE_TOOL,
                    &step,
                    &marker.task,
                    &marker.detail,
                )
                .await;
        }
        status => {
            rows.remove(&step_name);
            progress
                .finish(
                    super::super::workspace_progress::PHASE_TOOL,
                    &step,
                    status.as_frame_status(),
                    &marker.detail,
                )
                .await;
        }
    }
}
