//! Structured events of one turn, sent to the shell as frames and kept in the
//! answer's stored metadata so a reload shows them again:
//!
//! - `memory_saved {key, value}` / `memory_forgot {count}` / `memory_off`
//!   (meta key `memory_event`);
//! - `agent_enable_request {reason, request}` (meta key `agent_enable_request`);
//! - `files_produced {files: [{name, path, size, mime, drive_path}]}`
//!   (meta key `files_produced`).

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, OnceLock};

use serde_json::{json, Map, Value};
use uuid::Uuid;

use super::types::SessionIdentity;
use crate::core::user_memory::MemoryTurn;

/// Meta key of a memory frame.
pub(crate) const MEMORY_EVENT: &str = "memory_event";
/// Meta key of an Agent switch request.
pub(crate) const AGENT_ENABLE_REQUEST: &str = "agent_enable_request";
/// Meta key of the files a turn produced.
pub(crate) const FILES_PRODUCED: &str = "files_produced";

static EVENTS: OnceLock<Mutex<HashMap<Uuid, Map<String, Value>>>> = OnceLock::new();

fn events() -> MutexGuard<'static, HashMap<Uuid, Map<String, Value>>> {
    let map = EVENTS.get_or_init(|| Mutex::new(HashMap::new()));
    match map.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Keeps `frame` under `key` for the answer this turn stores next.
pub(crate) fn record(session_id: Uuid, key: &str, frame: Value) {
    events().entry(session_id).or_default().insert(key.to_string(), frame);
}

/// The events waiting for this session's answer; empty when none.
pub(crate) fn take(session_id: Uuid) -> Map<String, Value> {
    events().remove(&session_id).unwrap_or_else(|| { log::debug!("using default for session events"); Default::default() })
}

/// Sends `frame` to the shell and keeps it for the stored answer.
pub(crate) async fn emit(identity: &SessionIdentity<'_>, key: &str, frame: Value) {
    record(identity.session_id, key, frame.clone());
    if let Err(error) = identity.sink.send_raw_json(&frame).await {
        botlog::safe_warn!("turn events: could not send {key}: {error}");
    }
}

/// The frame for this turn's memory outcome, or `None` when the turn did not
/// touch memory (or failed, which the answer itself reports).
pub(crate) fn memory_frame(user_id: Uuid, turn: &MemoryTurn) -> Option<Value> {
    match turn {
        MemoryTurn::Saved(fact) => Some(json!({
            "type": "memory_saved",
            "key": botsettings::memory_store::fact_key(user_id, fact),
            "value": fact,
        })),
        MemoryTurn::Forgot(count) => Some(json!({"type": "memory_forgot", "count": count})),
        MemoryTurn::Off => Some(json!({"type": "memory_off"})),
        _ => None,
    }
}

/// The frame asking the person to turn the Agent on for this conversation.
pub(crate) fn agent_enable_frame(reason: &str, request: &str) -> Value {
    json!({"type": "agent_enable_request", "reason": reason, "request": request})
}

/// Short pt-BR reason shown on the Agent card.
pub(crate) const AGENT_REASON: &str =
    "Este pedido precisa da Máquina Virtual para executar comandos ou gerar arquivos.";

/// Whether this agent-off turn asks for work only the VM can do: making a
/// file, running a command or code. Questions about those subjects do not
/// count; the request must ask for the work.
pub(crate) fn needs_vm(text: &str) -> bool {
    let lower = text.to_lowercase();
    let asks = [
        "gere ", "gera ", "gerar ", "crie ", "cria ", "criar ", "faça ", "faz ", "fazer ",
        "monte ", "exporte ", "converta ", "execute ", "executa ", "executar ", "rode ", "roda ",
        "rodar ", "instale ", "compile ", "salve ", "generate ", "create ", "make ", "export ",
        "convert ", "run ", "execute ", "install ", "build ", "save ",
    ];
    let things = [
        "pdf", "docx", "xlsx", "pptx", "planilha", "arquivo", "documento word", "apresentação",
        "gráfico", "imagem png", "csv", "zip", "script", "comando", "código", "programa",
        "spreadsheet", "file", "chart", "command", "code", "program",
    ];
    asks.iter().any(|ask| lower.contains(ask))
        && things.iter().any(|thing| lower.contains(thing))
}

static REQUESTS: OnceLock<Mutex<HashMap<Uuid, String>>> = OnceLock::new();

fn requests() -> MutexGuard<'static, HashMap<Uuid, String>> {
    let map = REQUESTS.get_or_init(|| Mutex::new(HashMap::new()));
    match map.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Keeps the person's message of this turn, for a card that re-runs it.
pub(crate) fn note_request(session_id: Uuid, text: &str) {
    requests().insert(session_id, text.to_string());
}

/// The person's message of this session's current turn ("" when unknown).
pub(crate) fn request_of(session_id: Uuid) -> String {
    requests().get(&session_id).cloned().unwrap_or_else(|| { log::debug!("using default for session request"); Default::default() })
}

/// Merges `extra` into a stored meta object's fields.
pub(crate) fn merge_into(fields: &mut Map<String, Value>, extra: Map<String, Value>) {
    for (key, value) in extra {
        fields.insert(key, value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn memory_outcomes_become_the_agreed_frames() {
        let user = Uuid::new_v4();
        let saved = memory_frame(user, &MemoryTurn::Saved("Prefiro respostas curtas".into())).unwrap_or_default();
        assert_eq!(saved["type"], "memory_saved");
        assert_eq!(saved["value"], "Prefiro respostas curtas");
        assert_eq!(saved["key"], json!(botsettings::memory_store::fact_key(user, "Prefiro respostas curtas")));
        assert_eq!(memory_frame(user, &MemoryTurn::Forgot(2)).unwrap_or_default(), json!({"type": "memory_forgot", "count": 2}));
        assert_eq!(memory_frame(user, &MemoryTurn::Off).unwrap_or_default(), json!({"type": "memory_off"}));
        assert!(memory_frame(user, &MemoryTurn::Untouched).is_none());
        assert!(memory_frame(user, &MemoryTurn::Failed).is_none());
    }

    #[test]
    fn recorded_events_are_taken_once() {
        let session = Uuid::new_v4();
        record(session, MEMORY_EVENT, json!({"type": "memory_off"}));
        record(session, AGENT_ENABLE_REQUEST, agent_enable_frame("r", "Gere um PDF"));
        let taken = take(session);
        assert_eq!(taken.len(), 2);
        assert_eq!(taken[AGENT_ENABLE_REQUEST]["request"], "Gere um PDF");
        assert!(take(session).is_empty());
        note_request(session, "Gere um PDF");
        assert_eq!(request_of(session), "Gere um PDF");
        assert_eq!(request_of(Uuid::new_v4()), "");
    }

    #[test]
    fn file_and_command_requests_need_the_vm_but_questions_do_not() {
        assert!(needs_vm("Gere um PDF com o resumo"));
        assert!(needs_vm("crie uma planilha com os gastos"));
        assert!(needs_vm("Run this code and show the output"));
        assert!(!needs_vm("O que é um arquivo PDF?"));
        assert!(!needs_vm("Resuma este texto em três linhas"));
    }

    #[test]
    fn merging_keeps_existing_fields_and_adds_new_ones() {
        let mut fields = Map::new();
        fields.insert("a".into(), json!(1));
        let mut extra = Map::new();
        extra.insert(FILES_PRODUCED.into(), json!({"type": "files_produced", "files": []}));
        merge_into(&mut fields, extra);
        assert_eq!(fields.len(), 2);
    }
}
