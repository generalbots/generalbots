//! A compact, model-only record of what a turn received or did.
//!
//! Tool results (a page read, a command run) and attachment contents reach
//! the model only in the turn they happen; the stored conversation keeps the
//! texts alone. Replayed later, an answer that quoted a page or an attached
//! picture looked unfounded to the model, which then took it back ("I never
//! received an image"). This module keeps a one-line record per event, stores
//! it in the metadata of the turn's answer (never shown to the person) and
//! appends it to the replayed message, together with a line per attachment
//! the person's own message carried. The contents themselves are not
//! replayed: the record is enough to keep earlier answers consistent.

use std::collections::HashMap;
use std::sync::{Mutex, MutexGuard, OnceLock};

use uuid::Uuid;

use super::llm::message_meta::{self, AttachmentInfo};

/// Told to the model every turn: an earlier record is a fact, whatever this
/// turn's own capabilities are.
pub(crate) const RECORDS_ARE_FACTS: &str = "Lines such as [Anexo: …], [Página lida: …] and \
     [Comando executado: …] in earlier messages record what that turn really received, read or \
     ran. Never deny or retract an earlier answer because this turn cannot do the same; if a \
     detail is needed again, ask the person to attach the file or write the address again.";

/// Records one turn may keep.
const MAX_LINES: usize = 12;
/// Characters one record line may take.
const MAX_LINE_CHARS: usize = 240;
/// Characters of a command shown in its record.
const MAX_COMMAND_CHARS: usize = 120;
/// The stored role of an assistant message.
const ASSISTANT_ROLE: i32 = 2;

static RECORDS: OnceLock<Mutex<HashMap<Uuid, Vec<String>>>> = OnceLock::new();

fn records() -> MutexGuard<'static, HashMap<Uuid, Vec<String>>> {
    let map = RECORDS.get_or_init(|| Mutex::new(HashMap::new()));
    match map.lock() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    }
}

/// Forgets what an earlier, unfinished turn left behind.
pub(crate) fn begin(session_id: Uuid) {
    records().remove(&session_id);
}

/// Adds one record line to this turn.
pub(crate) fn note(session_id: Uuid, line: &str) {
    let line = clip(line.trim(), MAX_LINE_CHARS);
    if line.is_empty() {
        return;
    }
    let mut map = records();
    let lines = map.entry(session_id).or_default();
    if lines.len() < MAX_LINES && !lines.contains(&line) {
        lines.push(line);
    }
}

/// The record of a page this turn read (or failed to read).
pub(crate) fn note_page(session_id: Uuid, site: &str, title: Option<&str>, read: bool) {
    let line = match (read, title.filter(|t| !t.trim().is_empty())) {
        (true, Some(title)) => format!("[Página lida: {site} — {}]", title.trim()),
        (true, None) => format!("[Página lida: {site}]"),
        (false, _) => format!("[Página não lida: {site}]"),
    };
    note(session_id, &line);
}

/// The record of a command or program this turn ran in the workspace.
pub(crate) fn note_command(session_id: Uuid, command: &str, exit_code: Option<i32>) {
    note(session_id, &command_line(command, exit_code));
}

fn command_line(command: &str, exit_code: Option<i32>) -> String {
    let mut lines = command.lines().map(str::trim).filter(|line| !line.is_empty());
    let head = lines.next().unwrap_or("");
    let more = if lines.next().is_some() { " …" } else { "" };
    let outcome = match exit_code {
        Some(code) => format!("código {code}"),
        None => "sem código de saída".to_string(),
    };
    format!(
        "[Comando executado: {}{more} → {outcome}]",
        clip(head, MAX_COMMAND_CHARS)
    )
}

/// `content` with this turn's records in its metadata, when an assistant
/// message is stored and the turn has records; `None` leaves it unchanged.
pub(crate) fn attach(session_id: Uuid, role: i32, content: &str) -> Option<String> {
    if role != ASSISTANT_ROLE {
        return None;
    }
    let lines = records().remove(&session_id)?;
    if lines.is_empty() {
        return None;
    }
    let (visible, meta) = message_meta::decode(content);
    let mut meta = meta.unwrap_or_default();
    if !meta.model_note.is_empty() {
        meta.model_note.push('\n');
    }
    meta.model_note.push_str(&lines.join("\n"));
    Some(message_meta::encode(visible, &meta))
}

/// A stored message as the model reads it again: the visible text followed
/// by the records of what reached or happened in that turn.
pub(crate) fn replay(role: &str, stored: &str) -> String {
    let (visible, meta) = message_meta::decode(stored);
    let Some(meta) = meta else {
        return visible.to_string();
    };
    let mut lines: Vec<String> = Vec::new();
    if role == "user" {
        lines.extend(meta.attachments.iter().map(attachment_line));
    }
    lines.extend(
        meta.model_note
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(str::to_string),
    );
    if lines.is_empty() {
        return visible.to_string();
    }
    format!("{visible}\n\n{}", lines.join("\n"))
}

fn attachment_line(attachment: &AttachmentInfo) -> String {
    let mime = attachment.mime_type.to_ascii_lowercase();
    let kind = if mime.starts_with("image/") {
        "imagem"
    } else if mime == "application/pdf" {
        "PDF"
    } else if mime.starts_with("text/") {
        "texto"
    } else {
        "arquivo"
    };
    format!(
        "[Anexo: {} ({kind}) — recebido inteiro e lido naquela rodada; o histórico só não repete o conteúdo]",
        clip(&attachment.name, MAX_COMMAND_CHARS)
    )
}

fn clip(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        return text.to_string();
    }
    let mut cut: String = text.chars().take(limit.saturating_sub(1)).collect();
    cut.push('…');
    cut
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::bot::pipeline::llm::message_meta::MessageMeta;

    #[test]
    fn an_answer_keeps_the_records_of_its_turn() {
        let session = Uuid::new_v4();
        begin(session);
        note_page(session, "example.com", Some("Example Domain"), true);
        note_command(session, "uname -a", Some(0));
        let stored = attach(session, ASSISTANT_ROLE, "O título é Example Domain.")
            .expect("records attached");
        let (visible, meta) = message_meta::decode(&stored);
        assert_eq!(visible, "O título é Example Domain.");
        let note = meta.expect("meta").model_note;
        assert!(note.contains("[Página lida: example.com — Example Domain]"));
        assert!(note.contains("[Comando executado: uname -a → código 0]"));
        assert!(attach(session, ASSISTANT_ROLE, "next").is_none());
    }

    #[test]
    fn the_person_message_is_never_changed() {
        let session = Uuid::new_v4();
        note(session, "[Página lida: example.com]");
        assert!(attach(session, 1, "leia example.com").is_none());
        begin(session);
        assert!(attach(session, ASSISTANT_ROLE, "x").is_none());
    }

    #[test]
    fn reasoning_survives_and_stays_out_of_the_replay() {
        let session = Uuid::new_v4();
        let answer = message_meta::encode(
            "Pronto.",
            &MessageMeta {
                reasoning: "pensei".to_string(),
                ..Default::default()
            },
        );
        note_command(session, "python3 main.py\nprint(1)", None);
        let stored = attach(session, ASSISTANT_ROLE, &answer).expect("records attached");
        let (_, meta) = message_meta::decode(&stored);
        assert_eq!(meta.expect("meta").reasoning, "pensei");
        let replayed = replay("assistant", &stored);
        assert!(replayed.starts_with("Pronto.\n\n[Comando executado: python3 main.py … → sem código"));
        assert!(!replayed.contains("pensei"));
    }

    #[test]
    fn an_attachment_is_replayed_as_a_short_record() {
        let stored = message_meta::encode(
            "Descreva o cartaz",
            &MessageMeta {
                attachments: vec![AttachmentInfo {
                    name: "Cartaz de Teste.png".to_string(),
                    mime_type: "image/png".to_string(),
                    size: 1024,
                    drive_path: "inbox/Cartaz de Teste.png".to_string(),
                }],
                ..Default::default()
            },
        );
        let replayed = replay("user", &stored);
        assert!(replayed.starts_with("Descreva o cartaz\n\n[Anexo: Cartaz de Teste.png (imagem)"));
        assert_eq!(replay("user", "texto simples"), "texto simples");
    }

    #[test]
    fn a_turn_keeps_a_bounded_number_of_records() {
        let session = Uuid::new_v4();
        for index in 0..(MAX_LINES + 5) {
            note(session, &format!("[Página lida: site{index}.example]"));
        }
        let stored = attach(session, ASSISTANT_ROLE, "ok").expect("records attached");
        let (_, meta) = message_meta::decode(&stored);
        assert_eq!(meta.expect("meta").model_note.lines().count(), MAX_LINES);
    }
}
