//! #2046 — the in-VM task trace rendered as a task/time table.
//!
//! A generated program reports what it is doing by printing one line per task
//! to stdout:
//!
//! ```text
//! @@GB_TRACE@@ {"task": "install deps", "detail": "npm ci", "status": "started"}
//! @@GB_TRACE@@ {"task": "install deps", "status": "ok"}
//! ```
//!
//! The recorder turns each marker into a [`TraceRow`] (task, start offset,
//! duration, status, detail), keeps the rows after the run ends — including a
//! run cut off by the budget — and renders them either as a progress event for
//! the existing run card or as the `gb.tool_progress` frame the chat already
//! understands. The marker contract is shared with the chat workspace tool
//! (`@@GB_TRACE@@`), so a program needs no extra channel to be visible.
//!
//! A bare marker (`@@GB_TRACE@@ building`) is a task in the `running` state,
//! so a program with nothing to add still produces a row.

use super::budget::WorkspaceBudget;
use crate::types::VibeProgressEvent;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;

/// Reserved stdout prefix. Anything else on stdout is ordinary program output.
pub const TRACE_PREFIX: &str = "@@GB_TRACE@@";

/// Lifecycle of one traced task.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TraceStatus {
    /// Started and not finished yet.
    Running,
    /// Finished cleanly.
    Ok,
    /// Finished with a failure reported by the program.
    Error,
    /// Cut off by the run budget (no finish marker arrived).
    TimedOut,
}

impl TraceStatus {
    /// Status word used by the run card / frame payload.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Ok => "ok",
            Self::Error => "error",
            Self::TimedOut => "timed_out",
        }
    }

    /// True while the task is still in flight.
    pub fn is_open(self) -> bool {
        matches!(self, Self::Running)
    }

    fn from_word(word: &str) -> Self {
        match word.to_ascii_lowercase().as_str() {
            "ok" | "done" | "finished" | "complete" | "completed" | "success" => Self::Ok,
            "error" | "failed" | "fail" => Self::Error,
            "timed_out" | "timeout" => Self::TimedOut,
            _ => Self::Running,
        }
    }
}

/// One program trace marker, parsed from a stdout line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceMarker {
    pub task: String,
    pub detail: String,
    pub status: TraceStatus,
}

/// One row of the task/time table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceRow {
    /// Stable id of the task (`install-deps`), so start/finish share a row.
    pub step: String,
    pub task: String,
    pub detail: String,
    pub status: TraceStatus,
    /// Offset from the run start, in milliseconds.
    pub started_ms: u64,
    /// `None` while the task is in flight.
    pub duration_ms: Option<u64>,
}

/// The whole table for one run, safe to persist and replay.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceTable {
    pub run_id: String,
    pub elapsed_ms: u64,
    pub budget_ms: u64,
    pub remaining_ms: u64,
    pub rows: Vec<TraceRow>,
    pub artifacts: Vec<String>,
    /// Task that was running when the table was taken.
    pub in_flight: Option<String>,
    pub timed_out: bool,
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
                .and_then(|value| value.as_str())
                .map(str::trim)
                .filter(|task| !task.is_empty());
            if let Some(task) = task {
                let detail = value
                    .get("detail")
                    .and_then(|value| value.as_str())
                    .unwrap_or("")
                    .trim();
                let status = value
                    .get("status")
                    .and_then(|value| value.as_str())
                    .map(TraceStatus::from_word)
                    .unwrap_or(TraceStatus::Running);
                return Some(TraceMarker {
                    task: task.chars().take(120).collect(),
                    detail: detail.chars().take(200).collect(),
                    status,
                });
            }
        }
    }
    Some(TraceMarker {
        task: rest.chars().take(120).collect(),
        detail: String::new(),
        status: TraceStatus::Running,
    })
}

/// Stable row id for a task name: lowercase, non-alphanumerics collapsed to
/// `-`. The same task always maps to the same row.
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

/// Collects the trace of one workspace run.
pub struct TraceRecorder {
    run_id: String,
    budget: WorkspaceBudget,
    started: Instant,
    rows: Vec<TraceRow>,
    /// Row index per open step id.
    open: HashMap<String, usize>,
    artifacts: Vec<String>,
    timed_out: bool,
}

impl TraceRecorder {
    pub fn new(run_id: impl Into<String>, budget: WorkspaceBudget) -> Self {
        Self {
            run_id: run_id.into(),
            budget,
            started: Instant::now(),
            rows: Vec::new(),
            open: HashMap::new(),
            artifacts: Vec::new(),
            timed_out: false,
        }
    }

    pub fn budget(&self) -> WorkspaceBudget {
        self.budget
    }

    pub fn run_id(&self) -> &str {
        &self.run_id
    }

    /// Milliseconds since the run started.
    pub fn elapsed_ms(&self) -> u64 {
        self.budget.elapsed_ms(self.started)
    }

    /// Milliseconds left before the budget cuts the run off.
    pub fn remaining_ms(&self) -> u64 {
        self.budget.remaining_ms(self.started)
    }

    /// Starts a task, returning its row id. Restarting a finished task opens a
    /// new row instead of rewriting history.
    pub fn begin(&mut self, task: &str, detail: &str) -> String {
        let step = step_id_for(task);
        let started_ms = self.elapsed_ms();
        let row = TraceRow {
            step: step.clone(),
            task: task.chars().take(120).collect(),
            detail: detail.chars().take(200).collect(),
            status: TraceStatus::Running,
            started_ms,
            duration_ms: None,
        };
        self.rows.push(row);
        let index = self.rows.len() - 1;
        self.open.insert(step.clone(), index);
        step
    }

    /// Closes the open row for `task`, if any. Returns the finished row.
    pub fn finish(&mut self, task: &str, status: TraceStatus, detail: &str) -> Option<TraceRow> {
        let step = step_id_for(task);
        let index = self.open.remove(&step)?;
        let now = self.elapsed_ms();
        let row = self.rows.get_mut(index)?;
        row.status = status;
        if !detail.trim().is_empty() {
            row.detail = detail.chars().take(200).collect();
        }
        row.duration_ms = Some(now.saturating_sub(row.started_ms));
        Some(row.clone())
    }

    /// Records a file or log the program produced, so a failed or timed-out
    /// run still tells the user what survived.
    pub fn artifact(&mut self, path: &str) {
        let path = path.trim();
        if path.is_empty() || self.artifacts.iter().any(|kept| kept == path) {
            return;
        }
        self.artifacts.push(path.chars().take(300).collect());
    }

    pub fn artifacts(&self) -> &[String] {
        &self.artifacts
    }

    /// Applies one program output line. Non-trace lines return `None` and stay
    /// part of the program output.
    pub fn ingest_line(&mut self, line: &str) -> Option<TraceRow> {
        let marker = parse_trace_line(line)?;
        match marker.status {
            TraceStatus::Running => {
                self.begin(&marker.task, &marker.detail);
                self.rows.last().cloned()
            }
            status => self.finish(&marker.task, status, &marker.detail),
        }
    }

    /// Marks the run as cut off by the budget.
    pub fn mark_timed_out(&mut self) {
        self.timed_out = true;
    }

    /// Closes every open row with `status`, so the table never shows a
    /// spinner that can no longer resolve.
    pub fn close_open(&mut self, status: TraceStatus, detail: &str) {
        let open: Vec<String> = self.open.keys().cloned().collect();
        let now = self.elapsed_ms();
        for step in open {
            if let Some(index) = self.open.remove(&step) {
                if let Some(row) = self.rows.get_mut(index) {
                    row.status = status;
                    if !detail.trim().is_empty() {
                        row.detail = detail.chars().take(200).collect();
                    }
                    row.duration_ms = Some(now.saturating_sub(row.started_ms));
                }
            }
        }
    }

    /// Task currently in flight, if any.
    pub fn in_flight(&self) -> Option<String> {
        self.open
            .values()
            .filter_map(|index| self.rows.get(*index))
            .map(|row| row.task.clone())
            .next()
    }

    /// The table as it stands right now — the partial-progress snapshot that
    /// survives a failure or a timeout.
    pub fn table(&self) -> TraceTable {
        TraceTable {
            run_id: self.run_id.clone(),
            elapsed_ms: self.elapsed_ms(),
            budget_ms: self.budget.total_ms,
            remaining_ms: self.remaining_ms(),
            rows: self.rows.clone(),
            artifacts: self.artifacts.clone(),
            in_flight: self.in_flight(),
            timed_out: self.timed_out,
        }
    }

    /// `gb.tool_progress` payload for one row, the frame the chat run card
    /// already renders as a task/time entry.
    pub fn frame_json(&self, row: &TraceRow) -> serde_json::Value {
        serde_json::json!({
            "tool": "workspace_run",
            "phase": row.status.as_str(),
            "step": row.step,
            "label": row.task,
            "detail": row.detail,
            "elapsed_ms": row.duration_ms.unwrap_or_else(|| self.elapsed_ms()),
            "budget_ms": self.budget.total_ms,
            "remaining_ms": self.remaining_ms(),
        })
    }

    /// Progress event for the existing `VibeState::broadcast_progress` sink.
    pub fn progress_event(&self, step: &str, message: &str) -> VibeProgressEvent {
        let finished = self.rows.iter().filter(|row| !row.status.is_open()).count();
        let total = self.rows.len().max(1);
        VibeProgressEvent {
            event_type: "vibe_progress".to_string(),
            run_id: self.run_id.clone(),
            step: step.to_string(),
            message: message.to_string(),
            progress: ((finished * 100) / total).min(100) as u8,
            total_steps: total.min(u8::MAX as usize) as u8,
            current_step: finished.min(u8::MAX as usize) as u8,
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}
