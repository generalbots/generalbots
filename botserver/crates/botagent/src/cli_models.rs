//! The models each subscription CLI accepts for `--model`.
//!
//! A live list is read from the CLI inside the user's VM when the CLI can
//! print one; otherwise (VM stopped, CLI signed out) a curated list is used.
//! Every list starts with [`DEFAULT_ID`], which runs the CLI without
//! `--model` so it picks its own default.
//!
//! Where each list comes from:
//! - Claude Code: no command prints a model list. `claude --help` (2.1.329)
//!   says `--model` takes "an alias for the latest model (e.g. 'fable',
//!   'opus', or 'sonnet') or a model's full name"; the CLI's alias table
//!   holds `sonnet`, `opus`, `haiku` and `fable`. Curated from those aliases.
//! - Codex: `codex debug models` (codex-cli 0.170.0) prints the model
//!   catalog as JSON; the models with `visibility: "list"` are the ones its
//!   own picker shows. Read live; the curated copy is that catalog as of
//!   2026-11-21.
//! - Antigravity: `agy models` (1.5.13) lists the models once signed in.
//!   Read live; signed out the CLI prints no list, so only the default is
//!   offered.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;

use crate::cli_turn::cli_model;
use crate::vm;
use crate::AgentService;

/// The entry that runs the CLI with its own default model.
pub const DEFAULT_ID: &str = "default";

/// How long a live list is reused before the CLI is asked again.
const CACHE_TTL: Duration = Duration::from_secs(15 * 60);

/// Upper bound for one list command inside the VM.
const LIST_TIMEOUT: Duration = Duration::from_secs(45);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Cli {
    ClaudeCode,
    Codex,
    Antigravity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CliModel {
    pub id: String,
    pub label: String,
}

impl CliModel {
    fn new(id: &str, label: &str) -> Self {
        Self { id: id.to_string(), label: label.to_string() }
    }
}

fn default_entry() -> CliModel {
    CliModel::new(DEFAULT_ID, "CLI default")
}

/// The list used when no live list can be read.
pub fn curated(cli: Cli) -> Vec<CliModel> {
    let rest: &[(&str, &str)] = match cli {
        Cli::ClaudeCode => &[
            ("opus", "Opus (latest)"),
            ("sonnet", "Sonnet (latest)"),
            ("haiku", "Haiku (latest)"),
            ("fable", "Fable (latest)"),
        ],
        Cli::Codex => &[
            ("gpt-6.2-sol", "GPT-6.2-Sol"),
            ("gpt-6.2-astra", "GPT-6.2-Astra"),
            ("gpt-6.1-terra", "GPT-6.1 Terra"),
            ("gpt-6.1-sol", "GPT-6.1-Sol"),
            ("gpt-6-astra", "GPT-6-Astra"),
            ("gpt-6-sol", "GPT-6-Sol"),
            ("gpt-6-luna", "GPT-6-Luna"),
            ("gpt-5.6-sol", "GPT-5.6-Sol"),
            ("gpt-5.6-terra", "GPT-5.6-Terra"),
            ("gpt-5.6-luna", "GPT-5.6-Luna"),
        ],
        Cli::Antigravity => &[],
    };
    std::iter::once(default_entry())
        .chain(rest.iter().map(|(id, label)| CliModel::new(id, label)))
        .collect()
}

/// A model id the CLI turn passes through unchanged (see `cli_model`).
fn usable_id(id: &str) -> bool {
    !id.is_empty() && id.len() <= 80 && cli_model(id) == id
}

fn with_default(models: Vec<CliModel>) -> Vec<CliModel> {
    let mut out = vec![default_entry()];
    for model in models {
        if model.id != DEFAULT_ID && !out.iter().any(|m| m.id == model.id) {
            out.push(model);
        }
    }
    out
}

/// Parses `codex debug models`: the listed (not hidden) models in the
/// catalog's priority order.
pub fn parse_codex_catalog(json: &str) -> Vec<CliModel> {
    let Ok(root) = serde_json::from_str::<Value>(json) else {
        return Vec::new();
    };
    let mut rows: Vec<(i64, CliModel)> = root
        .get("models")
        .and_then(Value::as_array)
        .map(|models| {
            models
                .iter()
                .filter(|m| m.get("visibility").and_then(Value::as_str).unwrap_or("list") == "list")
                .filter_map(|m| {
                    let id = m.get("slug").and_then(Value::as_str)?.trim();
                    if !usable_id(id) {
                        return None;
                    }
                    let label = m
                        .get("display_name")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|l| !l.is_empty())
                        .unwrap_or(id);
                    let priority = m.get("priority").and_then(Value::as_i64).unwrap_or(i64::MAX);
                    Some((priority, CliModel::new(id, label)))
                })
                .collect()
        })
        .unwrap_or_else(|| {
            tracing::debug!("CLI returned no model list; starting from an empty set");
            Vec::new()
        });
    rows.sort_by_key(|(priority, _)| *priority);
    rows.into_iter().map(|(_, model)| model).collect()
}

/// Parses `agy models`: one model per line, its id first, an optional
/// description after it. Lines before an "Available models" header, status
/// lines and table headers are skipped.
pub fn parse_agy_models(text: &str) -> Vec<CliModel> {
    let lines: Vec<&str> = text.lines().collect();
    let start = lines
        .iter()
        .rposition(|l| l.to_ascii_lowercase().contains("available models"))
        .map_or(0, |i| i + 1);
    let mut out = Vec::new();
    for line in &lines[start..] {
        let line = line.trim().trim_start_matches(['-', '*', '\u{2022}', '>']).trim();
        let lower = line.to_ascii_lowercase();
        if line.is_empty() || lower.starts_with("error") || lower.ends_with("...") {
            continue;
        }
        let mut parts = line.splitn(2, char::is_whitespace);
        let id = parts.next().unwrap_or_else(|| {
            tracing::debug!("CLI model line '{line}' has no id; skipping it");
            ""
        }).trim_end_matches([':', ',']);
        if !usable_id(id) || !id.chars().any(|c| c.is_ascii_lowercase()) {
            continue;
        }
        let label = parts
            .next()
            .map(|rest| rest.trim().trim_matches(['-', ':', '|', '(', ')', '\u{2013}', ' ']).trim())
            .filter(|rest| !rest.is_empty())
            .unwrap_or(id);
        if !out.iter().any(|m: &CliModel| m.id == id) {
            out.push(CliModel::new(id, label));
        }
    }
    out
}

type Cache = Mutex<HashMap<(String, Cli), (Instant, Vec<CliModel>)>>;

fn cache() -> &'static Cache {
    static CACHE: OnceLock<Cache> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// The last live list read for this VM, while it is fresh.
pub fn cached(vm_name: &str, cli: Cli) -> Option<Vec<CliModel>> {
    let guard = cache().lock().ok()?;
    let (at, models) = guard.get(&(vm_name.to_string(), cli))?;
    (at.elapsed() < CACHE_TTL).then(|| models.clone())
}

fn remember(vm_name: &str, cli: Cli, models: &[CliModel]) {
    if let Ok(mut guard) = cache().lock() {
        guard.insert((vm_name.to_string(), cli), (Instant::now(), models.to_vec()));
    }
}

fn list_script(cli: Cli) -> Option<String> {
    let (user, home, command) = match cli {
        Cli::ClaudeCode => return None,
        Cli::Codex => ("codex", "/home/codex", "codex debug models"),
        Cli::Antigravity => ("antigravity", "/home/antigravity", "agy models"),
    };
    Some(format!(
        "timeout 40 su {user} -s /bin/sh -c 'cd {home} && HOME={home} {command}' </dev/null 2>/dev/null"
    ))
}

/// Asks the CLI inside the VM for its models. `None` when it cannot print a
/// list (no such command, VM unreachable, signed out).
pub async fn live(state: &AgentService, vm_name: &str, cli: Cli) -> Option<Vec<CliModel>> {
    let script = list_script(cli)?;
    let outcome = vm::exec_in_vm_streaming(
        state,
        vm_name,
        &["sh".into(), "-c".into(), script],
        Some(LIST_TIMEOUT),
        None,
        None,
    )
    .await
    .map_err(|(_, error)| tracing::debug!("model list for {cli:?} unavailable: {error}"))
    .ok()?;
    let models = match cli {
        Cli::Codex => parse_codex_catalog(&outcome.stdout),
        Cli::Antigravity => parse_agy_models(&outcome.stdout),
        Cli::ClaudeCode => Vec::new(),
    };
    if models.is_empty() {
        return None;
    }
    let models = with_default(models);
    remember(vm_name, cli, &models);
    Some(models)
}

/// The best list without touching the VM: a fresh live list, else curated.
pub fn known(vm_name: Option<&str>, cli: Cli) -> Vec<CliModel> {
    vm_name.and_then(|vm| cached(vm, cli)).unwrap_or_else(|| curated(cli))
}

/// The list to offer: a fresh cached list, a new live list, else curated.
pub async fn resolve(state: &AgentService, vm_name: Option<&str>, cli: Cli) -> Vec<CliModel> {
    let Some(vm_name) = vm_name else {
        return curated(cli);
    };
    if let Some(models) = cached(vm_name, cli) {
        return models;
    }
    live(state, vm_name, cli).await.unwrap_or_else(|| curated(cli))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_curated_list_starts_with_the_cli_default() {
        for cli in [Cli::ClaudeCode, Cli::Codex, Cli::Antigravity] {
            let list = curated(cli);
            assert_eq!(list[0].id, DEFAULT_ID);
            assert!(list[1..].iter().all(|m| usable_id(&m.id)), "{cli:?}");
            assert_eq!(cli_model(DEFAULT_ID), "", "the default runs without --model");
        }
        assert!(curated(Cli::ClaudeCode).iter().any(|m| m.id == "opus"));
    }

    #[test]
    fn the_codex_catalog_keeps_listed_models_in_priority_order() {
        let json = r#"{"models":[
            {"slug":"b-model","display_name":"B","visibility":"list","priority":2},
            {"slug":"hidden","display_name":"H","visibility":"hide","priority":0},
            {"slug":"a-model","display_name":"","visibility":"list","priority":1},
            {"slug":"bad;id","display_name":"X","visibility":"list","priority":3}
        ]}"#;
        let list = parse_codex_catalog(json);
        assert_eq!(list, vec![CliModel::new("a-model", "a-model"), CliModel::new("b-model", "B")]);
        assert!(parse_codex_catalog("not json").is_empty());
    }

    #[test]
    fn the_agy_list_skips_status_lines_and_headers() {
        let text = "Fetching available models...\nAvailable models:\n  NAME   DESCRIPTION\n  - gemini-3.1-pro  Gemini 3.1 Pro\n  * claude-sonnet-5-5 (Claude Sonnet)\n  gemini-3.1-pro dup\n";
        let list = parse_agy_models(text);
        assert_eq!(
            list,
            vec![
                CliModel::new("gemini-3.1-pro", "Gemini 3.1 Pro"),
                CliModel::new("claude-sonnet-5-5", "Claude Sonnet"),
            ]
        );
        let signed_out = "Fetching available models...\nError: Please sign in to view available models.";
        assert!(parse_agy_models(signed_out).is_empty());
    }

    #[test]
    fn a_live_list_gets_the_default_entry_once() {
        let list = with_default(vec![CliModel::new("x-1", "X"), CliModel::new(DEFAULT_ID, "dup")]);
        assert_eq!(list.iter().map(|m| m.id.as_str()).collect::<Vec<_>>(), vec![DEFAULT_ID, "x-1"]);
    }

    #[test]
    fn claude_has_no_list_command() {
        assert!(list_script(Cli::ClaudeCode).is_none());
        let script = list_script(Cli::Codex).unwrap_or_default();
        assert!(script.contains("su codex") && !script.contains("$("));
    }
}
