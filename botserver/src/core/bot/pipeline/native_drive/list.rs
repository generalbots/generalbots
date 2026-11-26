//! `drive_list` (#2164): lists files in the bot's `.gbdrive` so the model can
//! answer "open my last document about X" without the user naming a file.
//!
//! The model reads the listing in English (with the Docs link hint); the
//! person reads it, and the history keeps it, in their own language.

use botcore::i18n::{t, t_args, Locale};

use super::super::native_tools::{persist, send_split};
use super::super::types::SessionIdentity;
use super::{drive_link, human_size, parse, sanitize_name};
use crate::core::bot::drive_bridge::{self, DriveListItem};

const DRIVE_LIST_DEFAULT_LIMIT: usize = 20;
const DRIVE_LIST_MAX_LIMIT: usize = 100;
const DRIVE_LIST_SEARCH_SCAN_LIMIT: usize = 200;
const MODEL_OPEN_LABEL: &str = "Open in Drive";
const MODEL_DOCS_HINT: &str =
    "To open a document in the editor, use its Docs link (`app://docs?file=<key>&bucket=<bucket>`).";

/// Deep link that opens a Drive file in the Docs editor (DOC-05 retrieval path).
fn docs_href(bucket: &str, key: &str) -> String {
    let path = urlencoding::encode(key);
    let bucket = urlencoding::encode(bucket);
    format!("app://docs?file={path}&bucket={bucket}")
}

fn docs_link(name: &str, bucket: &str, key: &str) -> String {
    format!("[{name}]({})", docs_href(bucket, key))
}

fn drive_href(bucket: &str, key: &str) -> String {
    let path = urlencoding::encode(key);
    let bucket = urlencoding::encode(bucket);
    format!("app://drive?path={path}&scope=bot&bucket={bucket}")
}

fn is_document_file(name: &str) -> bool {
    matches!(
        name.rsplit('.')
            .next()
            .unwrap_or("")
            .to_ascii_lowercase()
            .as_str(),
        "doc" | "docx" | "odt" | "rtf" | "txt" | "md" | "markdown" | "pdf" | "html" | "htm"
    )
}

fn matches_drive_query(item: &DriveListItem, query: &str) -> bool {
    let needle = query.trim().to_ascii_lowercase();
    if needle.is_empty() {
        return true;
    }
    let hay = format!(
        "{} {}",
        item.relative.to_ascii_lowercase(),
        item.relative
            .rsplit('/')
            .next()
            .unwrap_or(&item.relative)
            .to_ascii_lowercase()
    );
    needle
        .split_whitespace()
        .filter(|t| !t.is_empty())
        .all(|token| hay.contains(token))
}

/// One listing line; `open_label` names the Drive link of a document.
fn entry(item: &DriveListItem, bucket: &str, open_label: &str) -> String {
    let name = sanitize_name(&item.relative);
    if is_document_file(&item.relative) {
        format!(
            "- {} · [{open_label}]({}) ({})",
            docs_link(&name, bucket, &item.key),
            drive_href(bucket, &item.key),
            human_size(item.size)
        )
    } else {
        format!("- {} ({})", drive_link(&name, bucket, &item.key), human_size(item.size))
    }
}

/// The empty answer: (for the model, for the person).
fn empty_texts(query: Option<&str>, locale: &Locale) -> (String, String) {
    match query {
        Some(q) => (
            format!("No files matching \"{q}\" were found in the bot Drive (`.gbdrive`)."),
            t_args(locale, "chat-drive-list-empty-query", &[("query", q)]),
        ),
        None => (
            "No files found in the bot Drive (`.gbdrive`).".to_string(),
            t(locale, "chat-drive-list-empty"),
        ),
    }
}

/// The listing: (for the model, for the person).
fn listing_texts(
    items: &[DriveListItem],
    bucket: &str,
    query: Option<&str>,
    limit: usize,
    locale: &Locale,
) -> (String, String) {
    let max = limit.to_string();
    let (model_heading, person_heading) = match query {
        Some(q) => (
            format!("Bot Drive files matching \"{q}\" (newest first, up to {limit}):"),
            t_args(locale, "chat-drive-list-heading-query", &[("query", q), ("limit", &max)]),
        ),
        None => (
            format!("Bot Drive files (newest first, up to {limit}):"),
            t_args(locale, "chat-drive-list-heading", &[("limit", &max)]),
        ),
    };
    let open_label = t(locale, "chat-drive-open");
    let mut model = vec![model_heading];
    let mut person = vec![person_heading];
    for item in items {
        model.push(entry(item, bucket, MODEL_OPEN_LABEL));
        person.push(entry(item, bucket, &open_label));
    }
    if query.is_some() {
        model.push(String::new());
        model.push(MODEL_DOCS_HINT.to_string());
    }
    (model.join("\n"), person.join("\n"))
}

pub(super) async fn run_drive_list(identity: &SessionIdentity<'_>, args: &str) {
    let parsed = parse(args);
    let limit = parsed
        .get("limit")
        .and_then(|v| v.as_u64())
        .map(|n| n as usize)
        .unwrap_or(DRIVE_LIST_DEFAULT_LIMIT)
        .clamp(1, DRIVE_LIST_MAX_LIMIT);
    let query = parsed
        .get("query")
        .or_else(|| parsed.get("q"))
        .and_then(|v| v.as_str())
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string);

    let SessionIdentity {
        state,
        session_id,
        user_id,
        bot_uuid,
        ..
    } = *identity;
    let locale = botcore::i18n::locale_for_user_pool(&state.conn, user_id);
    let Some(scope) = drive_bridge::scope(state, &bot_uuid, user_id) else {
        let person = t(&locale, "chat-drive-unavailable");
        send_split(identity, "Drive is not available for this workspace.", &person).await;
        return;
    };
    let scan_limit = if query.is_some() {
        DRIVE_LIST_SEARCH_SCAN_LIMIT
    } else {
        limit
    };
    let Some(items) = drive_bridge::list_bot_drive_objects(state, &scope, scan_limit).await else {
        let person = t(&locale, "chat-drive-list-failed");
        send_split(identity, "Could not list files in your bot Drive.", &person).await;
        return;
    };
    let filtered: Vec<_> = match query.as_deref() {
        Some(q) => items
            .into_iter()
            .filter(|item| matches_drive_query(item, q))
            .take(limit)
            .collect(),
        None => items.into_iter().take(limit).collect(),
    };
    let (for_model, for_person) = if filtered.is_empty() {
        empty_texts(query.as_deref(), &locale)
    } else {
        listing_texts(&filtered, &scope.bucket, query.as_deref(), limit, &locale)
    };
    send_split(identity, &for_model, &for_person).await;
    persist(state, session_id, user_id, &for_person).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn item(relative: &str) -> DriveListItem {
        DriveListItem {
            key: format!("conversations/{relative}"),
            relative: relative.to_string(),
            size: 2048,
        }
    }

    #[test]
    fn the_person_reads_the_listing_in_their_language_and_the_model_in_english() {
        botcore::i18n::ensure_initialized();
        let pt = Locale::new("pt-BR").unwrap_or_default();
        let items = vec![item("relatorio.docx"), item("dados.csv")];
        let (model, person) = listing_texts(&items, "bot.gbdrive", Some("relatorio"), 5, &pt);
        assert!(model.starts_with("Bot Drive files matching \"relatorio\" (newest first, up to 5):"));
        assert!(model.contains("[Open in Drive](app://drive?"));
        assert!(model.contains("app://docs?file=<key>&bucket=<bucket>"));
        assert!(person.starts_with("Arquivos do seu Drive com “relatorio” (mais recentes primeiro, até 5):"));
        assert!(person.contains("[Abrir no Drive](app://drive?"));
        assert!(!person.contains("app://docs?file=<key>"));
        assert_eq!(model.lines().filter(|l| l.starts_with("- ")).count(), 2);
        assert_eq!(person.lines().filter(|l| l.starts_with("- ")).count(), 2);
    }

    #[test]
    fn an_empty_listing_is_localised_for_the_person() {
        botcore::i18n::ensure_initialized();
        let pt = Locale::new("pt-BR").unwrap_or_default();
        let (model, person) = empty_texts(None, &pt);
        assert_eq!(model, "No files found in the bot Drive (`.gbdrive`).");
        assert_eq!(person, "Nenhum arquivo encontrado no seu Drive.");
        let (_, person) = empty_texts(Some("notas"), &pt);
        assert_eq!(person, "Nenhum arquivo com “notas” foi encontrado no seu Drive.");
    }

    #[test]
    fn a_query_matches_every_word_of_the_path() {
        assert!(matches_drive_query(&item("2026/relatorio final.docx"), "relatorio 2026"));
        assert!(!matches_drive_query(&item("dados.csv"), "relatorio"));
        assert!(matches_drive_query(&item("dados.csv"), "  "));
    }
}
