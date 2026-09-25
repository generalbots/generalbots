//! `agent_loop::core` — split per #1443 (AGENTS.md 450-line rule).

use super::*;

pub(crate) const DEFAULT_TIMEOUT_SECS: u64 = 600;

pub(crate) const MAX_TOOL_RESULT_CHARS: usize = 4000;

pub(crate) const MAX_TOOL_RETRIES: u32 = 2;

impl AgentLoop {
    pub fn with_security(
        mut self,
        permissions: PermissionEngineRef,
        skills: Arc<SkillStore>,
    ) -> Self {
        self.permissions = permissions;
        self.skills = skills;
        self
    }

    pub(crate) fn requested_title(intent: &str) -> Option<String> {
        let lower = intent.to_ascii_lowercase();
        let marker = ["title to ", "title as "]
            .iter()
            .find_map(|candidate| lower.rfind(candidate).map(|index| (index, *candidate)))?;
        let raw = intent[marker.0 + marker.1.len()..].trim();
        let title = raw
            .trim_end_matches(['.', '!', '?'])
            .trim()
            .trim_matches(['\'', '"', '`'])
            .trim();
        (!title.is_empty()).then(|| title.to_string())
    }
}

#[cfg(target_os = "windows")]
pub(crate) fn preferred_source_path(entries: &[String], intent: &str) -> Option<String> {
    let intent = intent.to_ascii_lowercase();
    entries
        .iter()
        .filter(|path| !path.ends_with('/'))
        .filter_map(|path| {
            let lower = path.to_ascii_lowercase();
            let extension = std::path::Path::new(path)
                .extension()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            let mut score: u16 = match extension.as_str() {
                "html" | "htm" => 160,
                "css" | "scss" | "sass" => 150,
                "js" | "jsx" | "ts" | "tsx" | "vue" | "svelte" => 120,
                "py" | "rs" | "go" | "java" => 80,
                _ => 0,
            };
            if score == 0 {
                return None;
            }
            if intent.contains(&lower) {
                score += 1000;
            }
            let stem = std::path::Path::new(path)
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or_default()
                .to_ascii_lowercase();
            if matches!(stem.as_str(), "index" | "app" | "main") {
                score += 100;
            }
            if lower.contains("style")
                && ["color", "colour", "theme", "background", "font", "layout"]
                    .iter()
                    .any(|word| intent.contains(word))
            {
                score += 120;
            }
            if lower.contains("test") || lower.contains("spec") {
                score = score.saturating_sub(100);
            }
            Some((score, path.clone()))
        })
        .max_by(|(left_score, left_path), (right_score, right_path)| {
            left_score
                .cmp(right_score)
                .then_with(|| right_path.cmp(left_path))
        })
        .map(|(_, path)| path)
}
