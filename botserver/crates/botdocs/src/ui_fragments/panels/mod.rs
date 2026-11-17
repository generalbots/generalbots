use super::{empty_fragment, html_escape};
use crate::auth::DocUser;
use crate::state::DocState;
use crate::types_core::{
    DocumentComment, DocumentStyle, Endnote, Footnote, OutlineItem, TableOfContents, TocEntry,
    TrackChange,
};
use axum::extract::State;
use axum::{response::Html, Json};
use botuifragments::ui_fragments::{panel_shell, panel_shell_with_body_style};
use std::sync::Arc;

mod ai;
mod notes_styles;
mod review;
mod toc_outline;

pub use ai::handle_ai_panel;
pub use notes_styles::{handle_endnotes_panel, handle_footnotes_panel, handle_styles_panel};
pub use review::{handle_comments_panel, handle_track_changes_panel};
pub use toc_outline::{handle_outline_panel, handle_toc_panel};

/// Loads the stored document for a panel. Drive-opened ids (`drive-…`) and
/// the unsaved `current` session are not stored yet, so they stay empty.
pub(super) async fn panel_document(
    state: &Arc<DocState>,
    user_id: &str,
    payload: &serde_json::Value,
) -> Option<crate::types::Document> {
    let id = payload
        .get("id")
        .or_else(|| payload.get("doc_id"))
        .and_then(|value| value.as_str())?;
    if id.is_empty() || id == "current" || id.starts_with("drive-") {
        return None;
    }
    crate::storage::load_document_from_drive(state, user_id, id)
        .await
        .ok()
        .flatten()
}

/// Extracts H1–H6 headings from editor HTML (attributes allowed). The stored
/// outline API only matches exact `<hN>` tags and cannot see unsaved edits.
pub(super) fn heading_items(content: &str) -> Vec<OutlineItem> {
    let mut items = Vec::new();
    for level in 1..=6u32 {
        let open = format!("<h{level}");
        let close = format!("</h{level}>");
        let mut position = 0usize;
        while let Some(start) = content[position..].find(&open) {
            let absolute = position + start;
            let Some(tag_end) = content[absolute..].find('>') else {
                break;
            };
            let text_start = absolute + tag_end + 1;
            let Some(text_end) = content[text_start..].find(&close) else {
                break;
            };
            let text = plain_text(&content[text_start..text_start + text_end]);
            if !text.is_empty() {
                items.push(OutlineItem {
                    id: uuid::Uuid::new_v4().to_string(),
                    text,
                    level,
                    position: absolute,
                    length: text_end,
                    style_name: format!("Heading {level}"),
                });
            }
            position = text_start + text_end + close.len();
        }
    }
    items.sort_by_key(|item| item.position);
    items
}

fn plain_text(html: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut inside_tag = false;
    for character in html.chars() {
        match character {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            other if !inside_tag => output.push(other),
            _ => {}
        }
    }
    output.trim().to_string()
}
