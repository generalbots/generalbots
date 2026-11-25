//! Parses an LLM-produced JSON action into an [`AgentAction`] (#2883).
//!
//! Kept separate from the agent loop so the loop stays focused on observation,
//! decision and execution.

use anyhow::{Context, Result};
use serde_json::Value;

use super::AgentAction;

pub(super) fn parse_action_from_json(action_val: &Value) -> Result<AgentAction> {
    if let Some(action_type) = action_val.as_str() {
        return match action_type {
            "Done" | "done" => Ok(AgentAction::Done {
                summary: String::new(),
            }),
            "ExtractText" | "extract_text" => Ok(AgentAction::ExtractText),
            "ExtractLinks" | "extract_links" => Ok(AgentAction::ExtractLinks),
            "Screenshot" | "screenshot" => Ok(AgentAction::Screenshot),
            "ScrollDown" | "scroll_down" => Ok(AgentAction::ScrollDown),
            "ScrollUp" | "scroll_up" => Ok(AgentAction::ScrollUp),
            _ => Err(anyhow::anyhow!("Unknown action type: {action_type}")),
        };
    }

    if let Some(obj) = action_val.as_object() {
        let action_type = obj
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_lowercase();

        return match action_type.as_str() {
            "navigate" => {
                let url = obj
                    .get("url")
                    .and_then(|v| v.as_str())
                    .context("Navigate action requires 'url' field")?
                    .to_string();
                Ok(AgentAction::Navigate { url })
            }
            "click" => {
                let selector = obj
                    .get("selector")
                    .and_then(|v| v.as_str())
                    .context("Click action requires 'selector' field")?
                    .to_string();
                Ok(AgentAction::Click { selector })
            }
            "fill" => {
                let selector = obj
                    .get("selector")
                    .and_then(|v| v.as_str())
                    .context("Fill action requires 'selector' field")?
                    .to_string();
                let text = obj
                    .get("text")
                    .and_then(|v| v.as_str())
                    .context("Fill action requires 'text' field")?
                    .to_string();
                Ok(AgentAction::Fill { selector, text })
            }
            "wait" => {
                let ms = obj.get("ms").and_then(|v| v.as_u64()).unwrap_or(1000);
                Ok(AgentAction::Wait { ms })
            }
            "done" => {
                let summary = obj
                    .get("summary")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Goal achieved")
                    .to_string();
                Ok(AgentAction::Done { summary })
            }
            "extracttext" | "extract_text" => Ok(AgentAction::ExtractText),
            "extractlinks" | "extract_links" => Ok(AgentAction::ExtractLinks),
            "screenshot" => Ok(AgentAction::Screenshot),
            "scrolldown" | "scroll_down" => Ok(AgentAction::ScrollDown),
            "scrollup" | "scroll_up" => Ok(AgentAction::ScrollUp),
            _ => Err(anyhow::anyhow!("Unknown action type: {action_type}")),
        };
    }

    Err(anyhow::anyhow!("Invalid action format: {action_val}"))
}
