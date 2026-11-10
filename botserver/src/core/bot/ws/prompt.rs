//! Prompt loading and response-language assembly (#2095).
//!
//! Split out of `ws/message.rs` to stay inside the 450-line budget. The
//! response-language contract is appended here — the single assembly point —
//! so every channel and every custom bot inherits it.

use botcore::shared::utils::current_org_id;

use crate::core::bot::ws::handler::verify_path_within_workdir;
use log::{debug, info, warn};

/// #2095 — the response-language contract every chat prompt carries.
///
/// It lives in the prompt assembly path (not only in the shipped templates) so
/// a custom bot that authored its own `PROMPT.md` still answers in the language
/// the user wrote. The wording matches `bottemplates/README.md` and the shipped
/// `PROMPT-*.md` files, and the block is skipped when the authored prompt
/// already states it (no duplicated instructions).
pub const RESPONSE_LANGUAGE_INSTRUCTION: &str = "Always answer in the user's language. If the user writes in Portuguese, answer in Portuguese; if Japanese, answer in Japanese. Keep code, commands, identifiers and file paths unchanged.";

pub fn load_system_prompt(bot_name: &str) -> String {
    load_system_prompt_for_channel(bot_name, "web")
}

/// Loads the bot's authored prompt and appends the response-language contract.
pub fn load_system_prompt_for_channel(bot_name: &str, channel: &str) -> String {
    with_response_language_contract(load_bot_prompt_for_channel(bot_name, channel))
}

/// Appends [`RESPONSE_LANGUAGE_INSTRUCTION`] unless the prompt already has it.
#[must_use]
pub fn with_response_language_contract(prompt: String) -> String {
    if prompt.contains(RESPONSE_LANGUAGE_INSTRUCTION) {
        return prompt;
    }
    format!("{prompt}\n\n## Language\n\n{RESPONSE_LANGUAGE_INSTRUCTION}")
}

/// Reads the authored prompt for `channel`; the contract is added by
/// [`load_system_prompt_for_channel`] so every early return gets it.
fn load_bot_prompt_for_channel(bot_name: &str, channel: &str) -> String {
    let work_dir = botcore::shared::utils::get_work_path();
    let org_id = current_org_id();
    let nil_uuid = uuid::Uuid::nil();

    // Helper to read a prompt by channel suffix, then fall back to base PROMPT.md
    let read_prompt = |gbot_dir: &str| -> Option<String> {
        let channel_file = format!("PROMPT-{}.md", channel.to_uppercase());
        let channel_file_lower = format!("PROMPT-{}.md", channel.to_lowercase());
        let base = |f: &str| -> Option<String> { std::fs::read_to_string(f).ok() };
        let channel_prompt = base(&format!("{}{}", gbot_dir, channel_file))
            .or_else(|| base(&format!("{}{}", gbot_dir, channel_file_lower)));
        let base_prompt = base(&format!("{}PROMPT.md", gbot_dir))
            .or_else(|| base(&format!("{}prompt.md", gbot_dir)))
            .or_else(|| base(&format!("{}PROMPT.txt", gbot_dir)))
            .or_else(|| base(&format!("{}prompt.txt", gbot_dir)));
        match (channel_prompt, base_prompt) {
            (Some(c), Some(b)) => Some(format!("{}\n\n{}", c, b)),
            (Some(c), None) => Some(c),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        }
    };

    // Primary path: {bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbot/
    let primary_rel = format!("{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbot/");
    if verify_path_within_workdir(&primary_rel) {
        let gbot_dir = format!("{work_dir}/{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbot/");
        if let Some(p) = read_prompt(&gbot_dir) {
            return p;
        }
    }

    // Fallback: {org_id}.gborg/{bot_name}.gbai/{bot_name}.gbot/
    let fallback_rel = format!(
        "{org_id}.gborg/{bot_name}.gbai/{bot_name}.gbot/",
        org_id = org_id
    );
    if org_id != nil_uuid && verify_path_within_workdir(&fallback_rel) {
        let gbot_dir = format!(
            "{work_dir}/{org_id}.gborg/{bot_name}.gbai/{bot_name}.gbot/",
            org_id = org_id
        );
        if let Some(p) = read_prompt(&gbot_dir) {
            return p;
        }
    }

    // The materialized org directory is `<org-id>.gborg`, which on a desktop
    // install is the all-zero id: neither path above matches, so the bot used
    // to answer with the generic fallback and the agent never received its
    // instructions. Resolve the prompt through the shared work-tree resolver.
    let channel_file = format!("PROMPT-{}.md", channel.to_uppercase());
    let candidates = [
        channel_file.clone(),
        format!("PROMPT-{}.md", channel.to_lowercase()),
        "PROMPT.md".to_string(),
        "prompt.md".to_string(),
        "PROMPT.txt".to_string(),
        "prompt.txt".to_string(),
    ];
    let channel_prompt = crate::core::bot::pipeline::tool_paths::resolve_bot_file_in(
        &work_dir,
        bot_name,
        "gbot",
        &channel_file,
    )
    .and_then(|path| std::fs::read_to_string(path).ok());
    if let Some(p) = channel_prompt {
        let base = candidates.iter().skip(2).find_map(|name| {
            crate::core::bot::pipeline::tool_paths::resolve_bot_file_in(
                &work_dir, bot_name, "gbot", name,
            )
            .and_then(|path| std::fs::read_to_string(path).ok())
        });
        return match base {
            Some(b) => format!("{p}\n\n{b}"),
            None => p,
        };
    }
    let base_prompt = candidates.iter().skip(2).find_map(|name| {
        crate::core::bot::pipeline::tool_paths::resolve_bot_file_in(
            &work_dir, bot_name, "gbot", name,
        )
        .and_then(|path| std::fs::read_to_string(path).ok())
    });
    if let Some(p) = base_prompt {
        return p;
    }

    let now = chrono::Utc::now().format("%B %d, %Y").to_string();
    format!("Today is {now}.\n\nYou are a helpful assistant. Be concise. Respond in plain text.")
}

pub fn load_bot_styles_css(bot_name: &str) -> String {
    let work_dir = botcore::shared::utils::get_work_path();
    let org_id = current_org_id();
    let nil_uuid = uuid::Uuid::nil();

    // Helper: read CSS from a gbot directory
    let read_css = |gbot_dir: &str| -> String {
        let global_css_path = format!("{}global.css", gbot_dir);
        let mut combined = match std::fs::read_to_string(&global_css_path) {
            Ok(c) => {
                info!(
                    "global.css loaded from {} ({} bytes)",
                    global_css_path,
                    c.len()
                );
                c
            }
            Err(_) => String::new(),
        };

        let css_path = format!("{}styles.css", gbot_dir);
        let local_css = match std::fs::read_to_string(&css_path) {
            Ok(c) => {
                info!("styles.css loaded from {} ({} bytes)", css_path, c.len());
                c
            }
            Err(e1) => {
                let alt_path = format!("{}style.css", gbot_dir);
                match std::fs::read_to_string(&alt_path) {
                    Ok(c) => {
                        info!("style.css loaded from {} ({} bytes)", alt_path, c.len());
                        c
                    }
                    Err(e2) => {
                        // A bot without a custom stylesheet is the normal case, so an
                        // absent file is not a warning; only a real IO failure
                        // (permissions, broken symlink) is worth reporting.
                        if e2.kind() == std::io::ErrorKind::NotFound {
                            debug!(
                                "No styles.css/style.css for this bot at {gbot_dir}; using platform styles"
                            );
                        } else {
                            warn!(
                                "Failed to read styles.css at {} or {}: {}, {}",
                                css_path, alt_path, e1, e2
                            );
                        }
                        String::new()
                    }
                }
            }
        };

        if !local_css.is_empty() {
            if !combined.is_empty() {
                combined.push('\n');
            }
            combined.push_str(&local_css);
        }
        combined
    };

    // Primary: {bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbot/
    let primary_rel = format!("{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbot/");
    if verify_path_within_workdir(&primary_rel) {
        let gbot_dir = format!("{work_dir}/{bot_name}.gborg/{bot_name}.gbai/{bot_name}.gbot/");
        let css = read_css(&gbot_dir);
        if !css.is_empty() {
            return css;
        }
    }

    // Fallback: {org_id}.gborg path (nil UUID)
    let fallback_rel = format!(
        "{org_id}.gborg/{bot_name}.gbai/{bot_name}.gbot/",
        org_id = org_id
    );
    if org_id != nil_uuid && verify_path_within_workdir(&fallback_rel) {
        let gbot_dir = format!(
            "{work_dir}/{org_id}.gborg/{bot_name}.gbai/{bot_name}.gbot/",
            org_id = org_id
        );
        let css = read_css(&gbot_dir);
        if !css.is_empty() {
            return css;
        }
    }

    String::new()
}

#[cfg(test)]
mod response_language_tests {
    use super::{
        load_system_prompt_for_channel, with_response_language_contract,
        RESPONSE_LANGUAGE_INSTRUCTION,
    };

    /// A bot with no authored prompt (the custom-bot case the issue calls out)
    /// must still be told to answer in the user's language.
    #[test]
    fn fallback_prompt_carries_the_language_contract() {
        let prompt = load_system_prompt_for_channel("bot-without-prompt-2095", "web");
        assert!(prompt.contains("You are a helpful assistant"));
        assert!(prompt.contains(RESPONSE_LANGUAGE_INSTRUCTION));
        assert!(prompt.contains("## Language"));
    }

    /// Every channel resolves through the same wrapper, so the contract is not
    /// a web-only behaviour.
    #[test]
    fn every_channel_gets_the_contract() {
        for channel in ["web", "whatsapp", "telegram", "msteams"] {
            let prompt = load_system_prompt_for_channel("bot-without-prompt-2095", channel);
            assert!(
                prompt.contains(RESPONSE_LANGUAGE_INSTRUCTION),
                "channel {channel}"
            );
        }
    }

    /// A prompt that already states the rule (the shipped templates) keeps its
    /// own wording and is not given a second, conflicting copy.
    #[test]
    fn authored_contract_is_not_duplicated() {
        let authored = format!("## Language\n\n{RESPONSE_LANGUAGE_INSTRUCTION}");
        let prompt = with_response_language_contract(authored.clone());
        assert_eq!(prompt, authored);
        assert_eq!(prompt.matches(RESPONSE_LANGUAGE_INSTRUCTION).count(), 1);
    }

    #[test]
    fn plain_prompt_gets_one_contract_appended() {
        let prompt = with_response_language_contract("You are a bot.".to_string());
        assert!(prompt.starts_with("You are a bot.\n\n## Language\n\n"));
        assert_eq!(prompt.matches(RESPONSE_LANGUAGE_INSTRUCTION).count(), 1);
    }
}
