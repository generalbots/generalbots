//! Shipped-template fast path for `create-and-execute` (#1459 family).
//!
//! For well-known intents whose implementation already exists as a vetted
//! `bottemplates/bots/...` script, AutoTask persists the template's files
//! verbatim instead of asking the LLM to regenerate them. The template is the
//! product: complete perception (BotModels BLIP/STT), closed taxonomy and
//! filing logic that already compiles — none of which an intent-compiler LLM
//! call can guarantee (the compile prompt emits `basic_program: null`, so a
//! `TALK` stub would be persisted).
//!
//! The matcher is intentionally deterministic (no LLM): the media-classifier
//! family of phrasings resolves to the media-filing template, everything
//! else falls through to the regular classify → compile → persist pipeline.

/// A template matched deterministically from the intent text.
pub struct ShippedTemplate {
    /// Human-readable template name (for logs and the API response).
    pub name: &'static str,
    /// Relative Drive path of the tool script inside the bot's `.gbdialog`.
    pub tool_path: &'static str,
    /// Tool script source, compiled into the binary from `bottemplates/`.
    pub tool_source: &'static str,
    /// Optional MCP manifest shipped alongside the tool.
    pub manifest_path: Option<&'static str>,
    pub manifest_source: Option<&'static str>,
    /// Bot configuration written to `.gbot` — the channel prompts that make the
    /// model call the tool. A tool without its prompt ships behaviour that
    /// never fires: the runtime reads `PROMPT-{CHANNEL}.md` from the work
    /// layout and silently falls back to a generic assistant prompt.
    pub config_files: &'static [(&'static str, &'static str)],
}

const CLASSIFY_MEDIA_TOOL: &str = include_str!(
    "../../../../bottemplates/bots/media-filing/media-filing.gbai/media-filing.gbdialog/classify_media.bas"
);
static CLASSIFY_MEDIA_MANIFEST: std::sync::OnceLock<String> = std::sync::OnceLock::new();

/// The manifest generated from `CLASSIFY_MEDIA_TOOL`'s `DESCRIPTION`/`PARAM`
/// declarations — the `.mcp.json` is a build artifact of the script, so the
/// template directory carries no manifest file at all.
pub fn classify_media_manifest() -> &'static str {
    CLASSIFY_MEDIA_MANIFEST
        .get_or_init(|| {
            botbasic_compiler::BasicCompiler::manifest_json(
                CLASSIFY_MEDIA_TOOL,
                "classify_media.mcp.json",
            )
            .unwrap_or_else(|e| {
                log::error!("classify_media manifest generation failed: {e}");
                r#"{
  "name": "classify_media",
  "input_schema": { "type": "object", "properties": {}, "required": [] }
}"#
                .to_string()
            })
        })
        .as_str()
}
const MEDIA_FILING_PROMPT_TELEGRAM: &str = include_str!(
    "../../../../bottemplates/bots/media-filing/media-filing.gbai/media-filing.gbot/PROMPT-TELEGRAM.md"
);
const MEDIA_FILING_PROMPT_WHATSAPP: &str = include_str!(
    "../../../../bottemplates/bots/media-filing/media-filing.gbai/media-filing.gbot/PROMPT-WHATSAPP.md"
);
/// Channel policies that make the model call `classify_media` on a media marker.
const MEDIA_FILING_CONFIG: &[(&str, &str)] = &[
    ("PROMPT-TELEGRAM.md", MEDIA_FILING_PROMPT_TELEGRAM),
    ("PROMPT-WHATSAPP.md", MEDIA_FILING_PROMPT_WHATSAPP),
];

/// Media-kind words a classification target may use (PT and EN, accented and
/// unaccented forms).
const MEDIA_KINDS: &[&str] = &[
    "media", "mídia", "midia", "image", "imagem", "imagens", "video", "vídeo",
    "vídeos", "videos", "photo", "photos", "foto", "fotos", "audio", "áudio",
];

fn window_has_media(window: &str) -> bool {
    MEDIA_KINDS.iter().any(|k| window.contains(k))
}

/// True when a classify/filing verb is followed (within a short window) by a
/// media-kind word — the shape of "classify media tool", "classificar
/// mídia", "arquivar vídeos por categoria", "media filing tool".
fn media_classification_phrasing(lower: &str) -> bool {
    // Verb-adjacent phrasings: the media word must sit close to the verb
    // (before or after) so unrelated objects ("classify customers") never
    // match even when the sentence mentions media elsewhere.
    for marker in ["classif", "arquiv", "organiz", "file ", "filing"] {
        let mut from = 0;
        while let Some(pos) = lower[from..].find(marker) {
            let start = from + pos;
            let window_start = start.saturating_sub(60);
            let end = (start + marker.len() + 40).min(lower.len());
            if window_has_media(&lower[window_start..end]) {
                return true;
            }
            from = start + marker.len();
        }
    }
    // Noun-first phrasings: "media classification/filing tool".
    lower.contains("media filing")
        || lower.contains("media classif")
        || (lower.contains("mídia") && (lower.contains("classif") || lower.contains("arquiv")))
}

/// Deterministic matcher: `None` means "no shipped template — use the regular
/// LLM pipeline".
pub fn match_shipped_template(intent: &str) -> Option<ShippedTemplate> {
    let lower = intent.to_lowercase();
    if media_classification_phrasing(&lower) {
        return Some(ShippedTemplate {
            name: "media-filing",
            tool_path: "tools/classify_media.bas",
            tool_source: CLASSIFY_MEDIA_TOOL,
            manifest_path: Some("tools/classify_media.mcp.json"),
            manifest_source: Some(classify_media_manifest()),
            config_files: MEDIA_FILING_CONFIG,
        });
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matches_english_media_classification_intents() {
        assert!(match_shipped_template("classify media tool").is_some());
        assert!(match_shipped_template("Create a tool to classify media by category").is_some());
        assert!(match_shipped_template("media filing tool").is_some());
        assert!(match_shipped_template("when I upload a video, classify it").is_some());
    }

    #[test]
    fn matches_portuguese_media_classification_intents() {
        assert!(match_shipped_template("ferramenta para classificar mídia").is_some());
        assert!(match_shipped_template("quero arquivar videos por categoria").is_some());
    }

    #[test]
    fn does_not_match_unrelated_classifications() {
        assert!(match_shipped_template("classify customers by region").is_none());
        assert!(match_shipped_template("classify support tickets").is_none());
        assert!(match_shipped_template("create a report about social media").is_none());
    }

    #[test]
    fn classify_media_template_ships_a_complete_tool() {
        let t = match_shipped_template("classify media tool").expect("template must match");
        assert_eq!(t.name, "media-filing");
        assert_eq!(t.tool_path, "tools/classify_media.bas");
        assert!(t.tool_source.contains("ON ERROR RESUME NEXT"));
        assert!(t.tool_source.contains("DESCRIBE VIDEO"));
        assert!(t.tool_source.contains("category = \"unsorted\""));
        let manifest = t.manifest_source.expect("manifest shipped");
        assert!(manifest.contains("classify_media"));
        assert!(manifest.contains("\"path\""), "manifest declares the path argument");
        assert!(manifest.contains("\"caption\""), "manifest declares the caption argument");
        for marker in ["[image]", "[document]", "[voice]", "[audio]", "[video]"] {
            assert!(manifest.contains(marker), "manifest does not mention {marker}");
        }
    }

    #[test]
    fn classify_media_template_ships_the_channel_prompts() {
        // Without `.gbot` the bot answers with the generic fallback prompt and
        // never calls the tool — the template is incomplete without them.
        let t = match_shipped_template("classify media tool").expect("template must match");
        let names: Vec<&str> = t.config_files.iter().map(|(n, _)| *n).collect();
        assert!(names.contains(&"PROMPT-TELEGRAM.md"));
        assert!(names.contains(&"PROMPT-WHATSAPP.md"));
        for (name, body) in t.config_files {
            assert!(!body.trim().is_empty(), "{name} shipped empty");
            assert!(body.contains("classify_media"), "{name} must name the tool");
        }
    }
}
