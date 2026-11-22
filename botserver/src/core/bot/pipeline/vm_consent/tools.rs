//! The native tools' side of the gate: each tool's arguments become a
//! [`VmAction`], a denial becomes the tool result the model reads, and the
//! result closes the action's step. The person sees the step's outcome; the
//! texts of a gated tool reach the model only, and its answer is what the
//! person reads, in their language.

use serde_json::Value;

use super::{ask, report, VmAction, VmActionKind, VmConsent, VmDecision, VmOutcome, VmReceipt};
use crate::core::bot::pipeline::native_tools::{peek_capture, send, tell_model, ModelOnly};
use crate::core::bot::pipeline::types::SessionIdentity;

fn strings(value: Option<&Value>) -> Vec<String> {
    match value {
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|v| v.as_str().or_else(|| v.get("name").and_then(Value::as_str)))
            .map(str::to_string)
            .collect(),
        Some(Value::String(one)) if !one.trim().is_empty() => vec![one.trim().to_string()],
        _ => Vec::new(),
    }
}

fn text(parsed: &Value, keys: &[&str]) -> Option<String> {
    keys.iter()
        .find_map(|key| parsed.get(*key).and_then(Value::as_str))
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// A `vm_exec` program (or its automatic corrected version).
#[must_use]
pub fn program_action(language: &str, code: &str, packages: &[String], background: bool, repair: bool) -> VmAction {
    VmAction::native(VmActionKind::Command {
        command: code.to_string(),
        cwd: None,
        language: Some(language.to_string()),
        packages: packages.to_vec(),
        background,
        repair,
    })
}

/// Asks; on a denial tells the model (as the tool result, never shown to the
/// person, whose step already reads "denied") and returns `None`.
pub async fn gate_or_explain(identity: &SessionIdentity<'_>, action: &VmAction) -> Option<VmConsent> {
    match ask(identity, action).await {
        VmDecision::Allowed(ticket) => Some(ticket),
        VmDecision::Denied(denial) => {
            tell_model(identity.session_id, &denial.model_text(action));
            None
        }
    }
}

/// The corrected program after a syntax error is a different program: it
/// needs its own decision (a one-time approval of the first one does not
/// cover it; a conversation grant or "Permitir tudo" does).
pub async fn run_repaired(
    identity: &SessionIdentity<'_>,
    language: &str,
    fixed: &str,
    timeout_ms: u64,
    packages: &[String],
) -> Option<(bool, String, String)> {
    let action = program_action(language, fixed, packages, false, true);
    let ticket = gate_or_explain(identity, &action).await?;
    let receipt = ticket.receipt();
    let run = crate::core::bot::agent_vm_hook::exec_in_user_vm_with_timeout(
        ticket,
        identity.user_id,
        identity.session_id,
        language,
        fixed,
        timeout_ms,
        packages,
    )
    .await;
    report(identity, &receipt, &VmOutcome::from_run(run.as_ref(), None)).await;
    run
}

/// `terminal_run`: the command and its working directory.
pub async fn gate_terminal(identity: &SessionIdentity<'_>, args: &str) -> Option<VmConsent> {
    let parsed = crate::core::bot::pipeline::native_args::parse_args(args).unwrap_or(Value::Null);
    let Some(command) = text(&parsed, &["command"]) else {
        tell_model(identity.session_id, "terminal_run requires a `command`.");
        return None;
    };
    let action = VmAction::native(VmActionKind::Command {
        command,
        cwd: text(&parsed, &["cwd", "workdir"]),
        language: None,
        packages: Vec::new(),
        background: false,
        repair: false,
    });
    gate_or_explain(identity, &action).await
}

/// `drive_fetch`: copies a Drive file into the conversation folder.
pub async fn gate_drive_fetch(identity: &SessionIdentity<'_>, args: &str) -> Option<VmConsent> {
    let parsed = crate::core::bot::pipeline::native_args::parse_args(args).unwrap_or(Value::Null);
    let Some(reference) = text(&parsed, &["path", "name"]) else {
        tell_model(identity.session_id, "drive_fetch requires the Drive file's `path`.");
        return None;
    };
    let name = reference.rsplit('/').next().unwrap_or(&reference).to_string();
    let action = VmAction::native(VmActionKind::FileWrite {
        path: format!("data/{name}"),
        from: Some(format!("Drive: {reference}")),
    });
    gate_or_explain(identity, &action).await
}

/// `drive_save` and `save_to_external_drive` (they leave the VM).
#[must_use]
pub fn outside_action(name: &str, args: &str) -> Option<VmAction> {
    let parsed = crate::core::bot::pipeline::native_args::parse_args(args).unwrap_or(Value::Null);
    let mut files = strings(parsed.get("files"));
    if files.is_empty() {
        files = strings(parsed.get("file").or_else(|| parsed.get("name")).or_else(|| parsed.get("path")));
    }
    if files.is_empty() {
        files.push("output/".to_string());
    }
    let kind = match name {
        "drive_save" => VmActionKind::DriveSave { files, folder: text(&parsed, &["folder"]) },
        "save_to_external_drive" => VmActionKind::ExternalSave {
            files,
            provider: text(&parsed, &["provider"]).unwrap_or_else(|| "external".to_string()),
        },
        _ => return None,
    };
    Some(VmAction::native(kind))
}

/// The turn's automatic copy of new `output/` files to Drive.
pub async fn allow_turn_publish(identity: &SessionIdentity<'_>, names: Vec<String>) -> Option<VmReceipt> {
    let action = VmAction::native(VmActionKind::DriveSave { files: names, folder: None });
    match ask(identity, &action).await {
        VmDecision::Allowed(ticket) => Some(ticket.receipt()),
        VmDecision::Denied(_) => None,
    }
}

/// Where the tool's own chat text starts, to read it back for its step.
#[must_use]
pub fn capture_mark(identity: &SessionIdentity<'_>) -> usize {
    peek_capture(identity.session_id).len()
}

/// Closes a tool's step with the chat text the tool sent after `mark`, and
/// the program's exit code. Without an exit code the command never ran to
/// the end, so the step failed.
pub async fn report_captured(
    identity: &SessionIdentity<'_>,
    receipt: &VmReceipt,
    mark: usize,
    files: Vec<String>,
    exit_code: Option<i32>,
) {
    let output = peek_capture(identity.session_id).get(mark..).unwrap_or("").to_string();
    let ok = exit_code == Some(0);
    let outcome = VmOutcome { ok, exit_code, output, files, reason: None };
    report(identity, receipt, &outcome).await;
}

/// Closes a `vm_exec` step with the run's result and the files now in
/// `output/`.
pub async fn report_run(
    identity: &SessionIdentity<'_>,
    receipt: &VmReceipt,
    result: &Result<Option<(bool, String, String)>, (u16, String)>,
    exit_code: Option<i32>,
) {
    let mut outcome = VmOutcome::from_run(
        result.as_ref().ok().and_then(Option::as_ref),
        result.as_ref().err().map(|(_, reason)| reason.as_str()),
    )
    .with_exit_code(exit_code);
    if outcome.ok {
        if let Some(listing) =
            crate::core::bot::agent_vm_hook::list_session_folder(identity.user_id, identity.session_id, "output").await
        {
            outcome.files = listing.into_iter().map(|(name, _)| format!("output/{name}")).collect();
        }
    }
    report(identity, receipt, &outcome).await;
}

/// `drive_fetch`, `drive_save` and `save_to_external_drive` behind the gate.
/// Returns false for any other tool.
#[cfg(feature = "drive")]
pub async fn run_drive_tool(identity: &SessionIdentity<'_>, name: &str, args: &str) -> bool {
    if name == "drive_fetch" {
        if !identity.agent_vm_enabled {
            send(identity, crate::core::bot::pipeline::agent_turn::DISABLED_MESSAGE).await;
            return true;
        }
        if let Some(ticket) = gate_drive_fetch(identity, args).await {
            let receipt = ticket.receipt();
            let scope = ModelOnly::begin(identity.session_id);
            let fetched = crate::core::bot::pipeline::native_drive::run_drive_fetch(identity, args, ticket).await;
            drop(scope);
            report(identity, &receipt, &fetched.into()).await;
        }
        return true;
    }
    if !identity.agent_vm_enabled {
        return false;
    }
    let Some(action) = outside_action(name, args) else {
        return false;
    };
    if let Some(ticket) = gate_or_explain(identity, &action).await {
        let receipt = ticket.receipt();
        let scope = ModelOnly::begin(identity.session_id);
        // `outside_action` only knows `drive_save` and `save_to_external_drive`.
        let saved = if name == "drive_save" {
            crate::core::bot::pipeline::native_drive::run_drive_save(identity, args).await
        } else {
            crate::core::bot::pipeline::native_external_save::run(identity, args).await
        };
        drop(scope);
        report(identity, &receipt, &saved.into()).await;
    }
    true
}
