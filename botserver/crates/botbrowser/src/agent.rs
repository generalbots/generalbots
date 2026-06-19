use std::sync::Arc;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::Mutex;

use crate::BrowserSession;

pub struct LlmConfig {
    pub url: String,
    pub key: String,
    pub model: String,
}

impl LlmConfig {
    pub fn new(url: &str, key: &str, model: &str) -> Self {
        Self {
            url: url.to_string(),
            key: key.to_string(),
            model: if model.is_empty() { "default".to_string() } else { model.to_string() },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AgentAction {
    Navigate { url: String },
    Click { selector: String },
    Fill { selector: String, text: String },
    ExtractText,
    ExtractLinks,
    Screenshot,
    ScrollDown,
    ScrollUp,
    Wait { ms: u64 },
    Done { summary: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStep {
    pub action: AgentAction,
    pub reasoning: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentObservation {
    pub url: String,
    pub title: String,
    pub visible_text: String,
    pub links: Vec<super::LinkInfo>,
    pub error: Option<String>,
}

impl AgentObservation {
    pub fn to_prompt_context(&self) -> String {
        let mut ctx = format!(
            "Current URL: {}\nPage Title: {}\n\nVisible text (first 2000 chars):\n{}\n\n",
            self.url,
            self.title,
            self.visible_text.chars().take(2000).collect::<String>()
        );

        if !self.links.is_empty() {
            ctx.push_str(&format!("Available links ({}):\n", self.links.len()));
            for (i, link) in self.links.iter().take(20).enumerate() {
                ctx.push_str(&format!("  {}. [{}]({})\n", i + 1, link.text, link.href));
            }
        }

        if let Some(ref err) = self.error {
            ctx.push_str(&format!("\nError: {err}\n"));
        }

        ctx
    }
}

async fn call_llm(prompt: &str, config: &LlmConfig) -> Result<String> {
    let client = reqwest::Client::new();
    let response = client
        .post(&config.url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", config.key))
        .json(&serde_json::json!({
            "model": config.model,
            "messages": [
                {"role": "user", "content": prompt}
            ],
            "temperature": 0.3,
            "max_tokens": 2048
        }))
        .send()
        .await
        .context("LLM API request failed")?;

    let body: Value = response.json().await.context("Failed to parse LLM response")?;
    let content = body["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("")
        .to_string();

    Ok(content)
}

pub async fn observe_page(session: &Arc<Mutex<BrowserSession>>) -> Result<AgentObservation> {
    let session = session.lock().await;
    let page = session.page.lock().await;

    let url = page.url().await.ok().flatten().unwrap_or_default();
    let title = page.get_title().await.ok().flatten().unwrap_or_default();

    let text_script = r#"document.body.innerText ? document.body.innerText.substring(0, 5000) : ''"#;

    let visible_text = page
        .evaluate(text_script)
        .await
        .ok()
        .and_then(|r| r.into_value::<String>().ok())
        .unwrap_or_default();

    let links_script = r#"
        JSON.stringify(
            Array.from(document.querySelectorAll('a[href]')).map(a => ({
                text: a.innerText.trim().substring(0, 100),
                href: a.href,
                title: (a.title || '').substring(0, 100)
            })).filter(l => l.href.startsWith('http')).slice(0, 50)
        );
    "#;

    let links = page
        .evaluate(links_script)
        .await
        .ok()
        .and_then(|r| r.into_value::<Vec<super::LinkInfo>>().ok())
        .unwrap_or_default();

    Ok(AgentObservation {
        url,
        title,
        visible_text,
        links,
        error: None,
    })
}

pub async fn decide_next_action(
    observation: &AgentObservation,
    goal: &str,
    last_step: Option<&AgentStep>,
    llm_config: &LlmConfig,
) -> Result<AgentStep> {
    let context = observation.to_prompt_context();

    let last_action = last_step
        .map(|s| format!("Last action: {:?}\nReasoning: {}", s.action, s.reasoning))
        .unwrap_or_default();

    let system_prompt = "You are a web navigation agent. Given a goal and current page state, decide the next action.\n\
        Available actions:\n\
        - Navigate {url}: go to a URL\n\
        - Click {selector}: click an element (use CSS selector like 'button.submit', '#login-btn', 'a[href*=\"contact\"]')\n\
        - Fill {selector, text}: type into an input field\n\
        - ExtractText: get all visible text from page\n\
        - ExtractLinks: get all links from page\n\
        - Screenshot: take a screenshot\n\
        - ScrollDown: scroll down\n\
        - ScrollUp: scroll up\n\
        - Wait {ms}: wait for a duration in milliseconds\n\
        - Done {summary}: goal achieved, provide summary\n\n\
        Return a JSON object with exactly two fields:\n\
        - \"reasoning\": your analysis of what to do next and why\n\
        - \"action\": one of the action objects above\n\
        \n\
        CRITICAL: Return ONLY valid JSON, no other text.";

    let prompt = format!(
        "{system_prompt}\n\n\
        Goal: {goal}\n\n\
        Current page state:\n{context}\n\n\
        {last_action}\n\n\
        What is the next action? Return ONLY JSON."
    );

    let response = call_llm(&prompt, llm_config)
        .await
        .context("LLM failed to decide next action")?;

    let cleaned = response
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```")
        .trim();

    let decision: serde_json::Value =
        serde_json::from_str(cleaned).context("Failed to parse LLM response as JSON")?;

    let reasoning = decision["reasoning"]
        .as_str()
        .unwrap_or("No reasoning provided")
        .to_string();

    let action = parse_action_from_json(&decision["action"])?;

    Ok(AgentStep { action, reasoning })
}

pub async fn run_agent_loop(
    session: &Arc<Mutex<BrowserSession>>,
    goal: &str,
    llm_config: &LlmConfig,
    max_steps: usize,
) -> Result<Vec<AgentStep>> {
    let mut steps: Vec<AgentStep> = Vec::new();

    for step_count in 1..=max_steps {
        let observation = observe_page(session).await?;
        let action = decide_next_action(&observation, goal, steps.last(), llm_config).await?;

        match &action.action {
            AgentAction::Done { summary } => {
                log::info!("Agent finished after {step_count} steps: {summary}");
                steps.push(action);
                return Ok(steps);
            }
            _ => {
                if let Err(e) = execute_action(session, &action.action).await {
                    log::warn!("Agent action failed at step {step_count}: {e}");
                }
                steps.push(action);
            }
        }
    }

    Ok(steps)
}

fn parse_action_from_json(action_val: &Value) -> Result<AgentAction> {
    if let Some(action_type) = action_val.as_str() {
        return match action_type {
            "Done" | "done" => Ok(AgentAction::Done { summary: String::new() }),
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

pub async fn execute_action(
    session: &Arc<Mutex<BrowserSession>>,
    action: &AgentAction,
) -> Result<()> {
    let session = session.lock().await;

    match action {
        AgentAction::Navigate { url } => {
            log::info!("Agent navigating to {url}");
            session.navigate(url).await?;
        }
        AgentAction::Click { selector } => {
            log::info!("Agent clicking {selector}");
            session.click(selector).await?;
        }
        AgentAction::Fill { selector, text } => {
            log::info!("Agent filling {selector} with '{text}'");
            session.fill(selector, text).await?;
        }
        AgentAction::ExtractText => {
            log::info!("Agent extracting text");
        }
        AgentAction::ExtractLinks => {
            log::info!("Agent extracting links");
        }
        AgentAction::Screenshot => {
            log::info!("Agent taking screenshot");
            let _ = session.screenshot().await;
        }
        AgentAction::ScrollDown => {
            log::info!("Agent scrolling down");
            let _ = session
                .execute("window.scrollBy(0, window.innerHeight)")
                .await;
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
        AgentAction::ScrollUp => {
            log::info!("Agent scrolling up");
            let _ = session
                .execute("window.scrollBy(0, -window.innerHeight)")
                .await;
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
        AgentAction::Wait { ms } => {
            log::info!("Agent waiting {ms}ms");
            tokio::time::sleep(Duration::from_millis(*ms)).await;
        }
        AgentAction::Done { .. } => {}
    }

    Ok(())
}

