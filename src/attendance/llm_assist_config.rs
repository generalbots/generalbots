use super::llm_assist_types::LlmAssistConfig;
use log::info;
use std::path::PathBuf;
use uuid::Uuid;

impl LlmAssistConfig {
    pub fn from_config(bot_id: Uuid, work_path: &str) -> Self {
        let config_path = PathBuf::from(work_path)
            .join(format!("{}.gbai", bot_id))
            .join("config.csv");

        let alt_path = PathBuf::from(work_path).join("config.csv");

        let path = if config_path.exists() {
            config_path
        } else if alt_path.exists() {
            alt_path
        } else {
            return Self::default();
        };

        let mut config = Self::default();

        if let Ok(content) = std::fs::read_to_string(&path) {
            for line in content.lines() {
                let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();

                if parts.len() < 2 {
                    continue;
                }

                let key = parts[0].to_lowercase();
                let value = parts[1];

                match key.as_str() {
                    "attendant-llm-tips" => {
                        config.tips_enabled = value.to_lowercase() == "true";
                    }
                    "attendant-polish-message" => {
                        config.polish_enabled = value.to_lowercase() == "true";
                    }
                    "attendant-smart-replies" => {
                        config.smart_replies_enabled = value.to_lowercase() == "true";
                    }
                    "attendant-auto-summary" => {
                        config.auto_summary_enabled = value.to_lowercase() == "true";
                    }
                    "attendant-sentiment-analysis" => {
                        config.sentiment_enabled = value.to_lowercase() == "true";
                    }
                    "bot-description" | "bot_description" => {
                        config.bot_description = Some(value.to_string());
                    }
                    "bot-system-prompt" | "system-prompt" => {
                        config.bot_system_prompt = Some(value.to_string());
                    }
                    _ => {}
                }
            }
        }

        info!(
            "LLM Assist config loaded: tips={}, polish={}, replies={}, summary={}, sentiment={}",
            config.tips_enabled,
            config.polish_enabled,
            config.smart_replies_enabled,
            config.auto_summary_enabled,
            config.sentiment_enabled
        );

        config
    }

    pub fn any_enabled(&self) -> bool {
        self.tips_enabled
            || self.polish_enabled
            || self.smart_replies_enabled
            || self.auto_summary_enabled
            || self.sentiment_enabled
    }
}

pub fn get_bot_system_prompt(bot_id: Uuid, work_path: &str) -> String {
    let config = LlmAssistConfig::from_config(bot_id, work_path);
    if let Some(prompt) = config.bot_system_prompt {
        return prompt;
    }

    let start_bas_path = PathBuf::from(work_path)
        .join(format!("{}.gbai", bot_id))
        .join(format!("{}.gbdialog", bot_id))
        .join("start.bas");

    if let Ok(content) = std::fs::read_to_string(&start_bas_path) {
        let mut description_lines = Vec::new();
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("REM ") || trimmed.starts_with("' ") {
                let comment = trimmed.trim_start_matches("REM ").trim_start_matches("' ");
                description_lines.push(comment);
            } else if !trimmed.is_empty() {
                break;
            }
        }
        if !description_lines.is_empty() {
            return description_lines.join(" ");
        }
    }

    "You are a professional customer service assistant. Be helpful, empathetic, and solution-oriented. Maintain a friendly but professional tone.".to_string()
}
