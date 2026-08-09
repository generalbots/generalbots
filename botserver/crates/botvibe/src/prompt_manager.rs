use crate::types::{VibeContext, VibeUseCase};
use std::collections::HashMap;

pub const FALLBACK_LANG: &str = "en";

/// One fully-expanded set of prompt fragments for a (use_case, lang) pair.
#[derive(Debug, Clone)]
pub struct LoadedTemplate {
    pub version: String,
    pub lang: String,
    pub system_prompt: String,
    pub tool_instructions: String,
    pub output_format: String,
}

impl LoadedTemplate {
    pub fn build_system_prompt(&self) -> String {
        format!(
            "{}\n\n{}\n\n{}\n{}",
            self.system_prompt,
            self.tool_instructions,
            output_label(&self.lang),
            self.output_format
        )
    }
}

/// Canonical language key used for lookup; the request may carry e.g. "pt-br"
/// or "pt_BR", both normalized to "pt-BR".
pub fn normalize_lang(lang: &str) -> String {
    let trimmed = lang.trim();
    if trimmed.is_empty() {
        return FALLBACK_LANG.to_string();
    }
    if trimmed.eq_ignore_ascii_case("pt") || trimmed.eq_ignore_ascii_case("pt-br")
        || trimmed.eq_ignore_ascii_case("pt_br") || trimmed.eq_ignore_ascii_case("ptbr")
        || trimmed.eq_ignore_ascii_case("por")
    {
        "pt-BR".to_string()
    } else {
        FALLBACK_LANG.to_string()
    }
}

fn output_label(lang: &str) -> String {
    if lang.starts_with("pt") {
        "Formato de saída esperado:".to_string()
    } else {
        "Expected output format:".to_string()
    }
}

/// Embedded, versioned prompt assets. Loaded once at construction with
/// `include_str!` so there is no runtime filesystem dependency. Returns the
/// (use_case, template) mapping in a flat list.
pub fn load_builtin_templates() -> Vec<(VibeUseCase, LoadedTemplate)> {
    vec![
        (
            VibeUseCase::SoftwareDevelopment,
            LoadedTemplate {
                version: "v2".into(),
                lang: "en".into(),
                system_prompt: body(include_str!("../prompts/software_development/en/system.md")),
                tool_instructions: body(include_str!("../prompts/software_development/en/tools.md")),
                output_format: body(include_str!("../prompts/software_development/en/output.md")),
            },
        ),
        (
            VibeUseCase::SoftwareDevelopment,
            LoadedTemplate {
                version: "v2".into(),
                lang: "pt-BR".into(),
                system_prompt: body(include_str!("../prompts/software_development/pt-BR/system.md")),
                tool_instructions: body(include_str!("../prompts/software_development/pt-BR/tools.md")),
                output_format: body(include_str!("../prompts/software_development/pt-BR/output.md")),
            },
        ),
        (
            VibeUseCase::CustomerSupport,
            LoadedTemplate {
                version: "v2".into(),
                lang: "en".into(),
                system_prompt: body(include_str!("../prompts/customer_support/en/system.md")),
                tool_instructions: body(include_str!("../prompts/customer_support/en/tools.md")),
                output_format: body(include_str!("../prompts/customer_support/en/output.md")),
            },
        ),
        (
            VibeUseCase::CustomerSupport,
            LoadedTemplate {
                version: "v2".into(),
                lang: "pt-BR".into(),
                system_prompt: body(include_str!("../prompts/customer_support/pt-BR/system.md")),
                tool_instructions: body(include_str!("../prompts/customer_support/pt-BR/tools.md")),
                output_format: body(include_str!("../prompts/customer_support/pt-BR/output.md")),
            },
        ),
        (
            VibeUseCase::FinancialAnalysis,
            LoadedTemplate {
                version: "v2".into(),
                lang: "en".into(),
                system_prompt: body(include_str!("../prompts/financial_analysis/en/system.md")),
                tool_instructions: body(include_str!("../prompts/financial_analysis/en/tools.md")),
                output_format: body(include_str!("../prompts/financial_analysis/en/output.md")),
            },
        ),
        (
            VibeUseCase::FinancialAnalysis,
            LoadedTemplate {
                version: "v2".into(),
                lang: "pt-BR".into(),
                system_prompt: body(include_str!("../prompts/financial_analysis/pt-BR/system.md")),
                tool_instructions: body(include_str!("../prompts/financial_analysis/pt-BR/tools.md")),
                output_format: body(include_str!("../prompts/financial_analysis/pt-BR/output.md")),
            },
        ),
    ]
}

/// Strip the YAML frontmatter block (`---\n...\n---\n`) from an asset file.
fn body(asset: &str) -> String {
    asset.split_once("---\n").and_then(|(_, rest)| rest.split_once("---\n")).map(|(_, b)| b.trim().to_string()).unwrap_or_else(|| asset.trim().to_string())
}

pub struct VibePromptManager {
    templates: HashMap<(VibeUseCase, String), LoadedTemplate>,
}

impl VibePromptManager {
    pub fn new() -> Self {
        let templates: HashMap<(VibeUseCase, String), LoadedTemplate> = load_builtin_templates()
            .into_iter()
            .map(|(use_case, tpl)| ((use_case, tpl.lang.clone()), tpl))
            .collect();
        Self { templates }
    }

    fn template(&self, use_case: VibeUseCase, lang: &str) -> Option<&LoadedTemplate> {
        let lang = normalize_lang(lang);
        if let Some(t) = self.templates.get(&(use_case, lang.clone())) {
            return Some(t);
        }
        self.templates.get(&(use_case, FALLBACK_LANG.to_string()))
    }

    pub fn build_context(
        &self,
        use_case: VibeUseCase,
        lang: &str,
        user_message: &str,
        history: &[crate::types::ContextMessage],
    ) -> VibeContext {
        let mut ctx = VibeContext::new(uuid::Uuid::nil());
        if let Some(template) = self.template(use_case, lang) {
            ctx.system_prompt = template.build_system_prompt();
        }
        ctx.conversation_history = history.to_vec();
        ctx.add_user_message(user_message.to_string());
        ctx
    }

    pub fn system_prompt_for(&self, use_case: VibeUseCase, lang: &str) -> String {
        // `template` always matches: every use_case ships an entry for the
        // requested lang or FALLBACK_LANG, so this never returns an empty or
        // inline prompt.
        self.template(use_case, lang)
            .map(|t| t.build_system_prompt())
            .unwrap_or_default()
    }

    pub fn compose_prompt(&self, context: &VibeContext, user_message: &str) -> String {
        let mut parts = Vec::new();
        parts.push(format!("System: {}", context.system_prompt));

        for msg in &context.conversation_history {
            parts.push(format!("{}: {}", msg.role, msg.content));
        }

        if !context.kb_references.is_empty() {
            parts.push(render_grounding(&context.kb_references));
        }

        parts.push(format!("User: {user_message}"));
        parts.join("\n\n")
    }
}

const MAX_GROUNDING_REFS: usize = 20;
const MAX_GROUNDING_REF_CHARS: usize = 1500;

/// Renders the grounding section: deduplicated, numbered and length-capped
/// references followed by explicit instructions so the model grounds factual
/// claims in the provided sources instead of inventing context.
fn render_grounding(references: &[String]) -> String {
    let mut seen = std::collections::HashSet::new();
    let mut lines = Vec::new();
    for reference in references {
        if lines.len() >= MAX_GROUNDING_REFS {
            break;
        }
        let trimmed = reference.trim();
        if trimmed.is_empty() || !seen.insert(trimmed.to_string()) {
            continue;
        }
        lines.push(format!("[{}] {}", lines.len() + 1, truncate_chars(trimmed, MAX_GROUNDING_REF_CHARS)));
    }
    if lines.is_empty() {
        return String::new();
    }
    let mut section = String::from("Grounding context (answer factual questions using only these sources):\n");
    section.push_str(&lines.join("\n"));
    section.push_str("\nIf the grounding context does not contain the answer, say so explicitly; never invent sources.");
    section
}

/// Cuts text on a character boundary, marking any truncation.
fn truncate_chars(text: &str, limit: usize) -> String {
    if text.chars().count() <= limit {
        text.to_string()
    } else {
        let mut end = limit;
        while !text.is_char_boundary(end) {
            end -= 1;
        }
        format!("{}…", &text[..end])
    }
}

impl Default for VibePromptManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_prompt_for_all_use_cases_both_langs() {
        let pm = VibePromptManager::new();
        for use_case in [
            VibeUseCase::SoftwareDevelopment,
            VibeUseCase::CustomerSupport,
            VibeUseCase::FinancialAnalysis,
        ] {
            let en = pm.system_prompt_for(use_case, "en");
            assert!(en.contains("Expected output format"), "en prompt for {use_case}: {en}");
            assert!(en.contains("tool_name"));
            let pt = pm.system_prompt_for(use_case, "pt-BR");
            assert!(pt.contains("Formato de saída esperado"), "pt prompt for {use_case}");
            assert!(pt.contains("tool_name"));
        }
    }

    #[test]
    fn lang_normalization_and_fallback() {
        let pm = VibePromptManager::new();
        // Unsupported lang falls back to English.
        let fr = pm.system_prompt_for(VibeUseCase::CustomerSupport, "fr");
        assert!(fr.contains("Expected output format"));
        // Case/dash variants of Portuguese resolve to pt-BR assets.
        let pt = pm.system_prompt_for(VibeUseCase::CustomerSupport, "pt_br");
        assert!(pt.contains("Formato de saída esperado"));
        let _ = normalize_lang("en-US");
    }

    #[test]
    fn build_context_sets_prompt_and_history() {
        let pm = VibePromptManager::new();
        let history = vec![
            crate::types::ContextMessage { role: "assistant".into(), content: "hi".into(), timestamp: chrono::Utc::now() },
        ];
        let ctx = pm.build_context(VibeUseCase::CustomerSupport, "pt-BR", "need help", &history);
        assert!(ctx.system_prompt.contains("atendimento"));
        assert!(ctx.system_prompt.contains("Formato de saída esperado"));
        assert_eq!(ctx.conversation_history.len(), 2);
        assert_eq!(ctx.conversation_history.last().unwrap().role, "user");
        assert_eq!(ctx.conversation_history.last().unwrap().content, "need help");
        assert_eq!(ctx.run_id, uuid::Uuid::nil());
    }

    #[test]
    fn compose_prompt_includes_context_and_user() {
        let pm = VibePromptManager::new();
        let mut ctx = pm.build_context(VibeUseCase::SoftwareDevelopment, "en", "first msg", &[]);
        ctx.kb_references.push("kb-doc-1".to_string());
        let composed = pm.compose_prompt(&ctx, "second msg");
        assert!(composed.contains("System:"));
        assert!(composed.contains("first msg"));
        assert!(composed.contains("second msg"));
        assert!(composed.contains("kb-doc-1"));
        assert!(composed.contains("User: second msg"));
    }

    #[test]
    fn grounding_renders_numbered_deduplicated_refs() {
        let section = render_grounding(&[
            "doc-A: first snippet".to_string(),
            "doc-A: first snippet".to_string(),
            "doc-B: second snippet".to_string(),
        ]);
        assert!(section.contains("[1] doc-A: first snippet"));
        assert!(section.contains("[2] doc-B: second snippet"));
        assert!(!section.contains("[3]"));
        assert!(section.contains("never invent sources"));
        assert_eq!(section.matches("doc-A: first snippet").count(), 1, "duplicates removed");
    }

    #[test]
    fn grounding_caps_count_and_length() {
        let many: Vec<String> = (0..30).map(|i| format!("ref-{i}: data")).collect();
        let section = render_grounding(&many);
        assert!(section.matches('[').count() <= MAX_GROUNDING_REFS + 1, "numbered refs capped");

        let long = vec![format!("snippet {}", "x".repeat(3000))];
        let section = render_grounding(&long);
        assert!(section.contains('…'), "oversized reference is truncated");
        let body = section.lines().find(|l| l.starts_with("[1]")).unwrap_or_default();
        assert!(body.chars().count() <= MAX_GROUNDING_REF_CHARS + 8);
    }

    #[test]
    fn grounding_omits_blank_and_returns_empty_section_safely() {
        assert_eq!(render_grounding(&[]), "");
        assert_eq!(render_grounding(&["   ".to_string(), "".to_string()]), "");
    }
}