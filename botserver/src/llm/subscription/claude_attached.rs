//! The files the person attached to a Claude Code turn, sent inside the
//! stream-json user message (pictures shrunk like the API path does, PDFs
//! as documents, text files the message does not already quote) so the
//! model sees them without opening anything in the workspace. A `Read` of
//! one of these very copies is the person's own request (their message
//! carries the file), so it needs no card; anything else still asks.

use std::sync::Arc;

use base64::Engine;
use uuid::Uuid;

use botagent::claude::bridge::Attachment;
use botagent::AgentService;

use super::antigravity_bridge::{
    extension, image_mime, is_text, relative_path, MAX_IMAGE_BYTES, MAX_IMAGE_SOURCE, MAX_TEXT_BYTES,
};

/// Largest PDF sent as a document: its base64 form stays under the API's
/// 32 MB request cap.
const MAX_PDF_BYTES: usize = 20 * 1024 * 1024;

/// The workspace paths of the files staged for this turn.
pub(super) fn staged_paths(session_id: Uuid) -> Vec<String> {
    #[cfg(feature = "drive")]
    {
        crate::core::bot::pipeline::native_drive::attached::staged_for(session_id)
            .into_iter()
            .map(|file| file.vm_path)
            .collect()
    }
    #[cfg(not(feature = "drive"))]
    {
        std::hint::black_box(session_id);
        Vec::new()
    }
}

/// Whether `path` is exactly one of this turn's staged copies.
pub(super) fn is_own_attachment(path: &str, staged: &[String]) -> bool {
    !path.split('/').any(|part| part == "..") && staged.iter().any(|copy| copy == path)
}

/// What one staged file becomes in the message, by its name and kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Image(&'static str),
    Pdf,
    Text,
}

fn kind_of(rel: &str, image: bool, quoted: bool) -> Option<Kind> {
    if image {
        return image_mime(rel).map(Kind::Image);
    }
    if extension(rel) == "pdf" {
        return Some(Kind::Pdf);
    }
    (!quoted && is_text(rel)).then_some(Kind::Text)
}

/// The turn's attachments, read back from the workspace copies.
pub(super) async fn attachments(
    service: &Arc<AgentService>,
    vm_name: &str,
    session_id: Uuid,
    quoted: bool,
) -> Vec<Attachment> {
    let conversation = session_id.to_string();
    let mut out = Vec::new();
    #[cfg(feature = "drive")]
    for file in crate::core::bot::pipeline::native_drive::attached::staged_for(session_id) {
        let Some(rel) = relative_path(&file.vm_path, &conversation) else {
            continue;
        };
        let Some(kind) = kind_of(&rel, file.image, quoted) else {
            continue;
        };
        let bytes = match botagent::transfer::read_file(service, vm_name, &conversation, &rel).await {
            Ok(bytes) => bytes,
            Err((_, error)) => {
                botlog::safe_warn!("claude turn: attachment not read back: {error}");
                continue;
            }
        };
        let size = bytes.len();
        let attachment = match kind {
            Kind::Image(mime) if size <= MAX_IMAGE_SOURCE => tokio::task::spawn_blocking(move || {
                crate::core::bot::pipeline::turn_hooks::vision_image::prepare(&bytes, mime)
            })
            .await
            .ok()
            .flatten()
            .filter(|(bytes, _)| bytes.len() <= MAX_IMAGE_BYTES)
            .map(|(bytes, out_type)| Attachment::Image {
                media_type: out_type.to_string(),
                data: base64::engine::general_purpose::STANDARD.encode(&bytes),
            }),
            Kind::Pdf if size <= MAX_PDF_BYTES => {
                Some(Attachment::Pdf { data: base64::engine::general_purpose::STANDARD.encode(&bytes) })
            }
            Kind::Text if size <= MAX_TEXT_BYTES => Some(Attachment::Text {
                path: file.vm_path.clone(),
                text: String::from_utf8_lossy(&bytes).into_owned(),
            }),
            _ => None,
        };
        match attachment {
            Some(attachment) => out.push(attachment),
            None => botlog::safe_warn!("claude turn: attachment of {size} bytes not sent with the message"),
        }
    }
    #[cfg(not(feature = "drive"))]
    std::hint::black_box((service, vm_name, &conversation, quoted));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_staged_copy_becomes_the_block_its_kind_needs() {
        assert_eq!(kind_of("data/Cartaz de Teste.png", true, false), Some(Kind::Image("image/png")));
        assert_eq!(kind_of("data/relatorio.PDF", false, true), Some(Kind::Pdf));
        assert_eq!(kind_of("data/notas.txt", false, false), Some(Kind::Text));
        assert_eq!(kind_of("data/notas.txt", false, true), None);
        assert_eq!(kind_of("data/planilha.xlsx", false, false), None);
    }

    #[test]
    fn only_the_exact_staged_copy_is_the_persons_own_file() {
        let staged = vec!["/opt/vibe/conversations/c1/data/Cartaz de Teste.png".to_string()];
        assert!(is_own_attachment("/opt/vibe/conversations/c1/data/Cartaz de Teste.png", &staged));
        assert!(!is_own_attachment("/opt/vibe/conversations/c1/data/other.png", &staged));
        assert!(!is_own_attachment("/opt/vibe/conversations/c1/data/../../c2/data/Cartaz de Teste.png", &staged));
        assert!(!is_own_attachment("/home/claude/.claude/.credentials.json", &staged));
    }
}
