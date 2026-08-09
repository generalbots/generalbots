mod types;

pub use types::*;

use crate::types::{AutoTaskState, ConfigOps, DbPool, LlmProviderOps, TaskProgressEvent, UserSession};
use chrono::Utc;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Float8, Text, Uuid as DieselUuid};
use log::{info, trace, warn};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

pub struct IntentClassifier {
    pool: DbPool,
    config_ops: Arc<dyn ConfigOps>,
    llm_ops: Arc<dyn LlmProviderOps>,
    state: Arc<dyn AutoTaskState>,
}

impl IntentClassifier {
    pub fn new(
        pool: DbPool,
        config_ops: Arc<dyn ConfigOps>,
        llm_ops: Arc<dyn LlmProviderOps>,
        state: Arc<dyn AutoTaskState>,
    ) -> Self {
        Self { pool, config_ops, llm_ops, state }
    }

    pub fn state(&self) -> &Arc<dyn AutoTaskState> {
        &self.state
    }

    pub fn config_ops(&self) -> &Arc<dyn ConfigOps> {
        &self.config_ops
    }

    pub fn llm_ops(&self) -> &Arc<dyn LlmProviderOps> {
        &self.llm_ops
    }

    pub async fn classify(
        &self, intent: &str, session: &UserSession,
    ) -> Result<ClassifiedIntent, Box<dyn std::error::Error + Send + Sync>> {
        info!("Classifying intent for session {}: {}", session.id, &intent[..intent.len().min(100)]);
        let classification = self.classify_with_llm(intent, session.bot_id).await?;
        self.store_classification(&classification, session)?;
        Ok(classification)
    }

    /// Classify for the AutoTask API (#755) — no session required, tries the
    /// heuristic classifier first (offline-safe), falls back to LLM.
    pub async fn classify_api(
        &self, intent: &str, bot_id: Uuid,
    ) -> Result<ClassifiedIntent, Box<dyn std::error::Error + Send + Sync>> {
        info!("API classify intent for bot {bot_id}: {}", &intent[..intent.len().min(100)]);
        let heuristic = Self::classify_heuristic(intent)?;
        if !heuristic.requires_clarification {
            Ok(heuristic)
        } else {
            self.classify_with_llm(intent, bot_id).await
        }
    }

    pub async fn classify_and_process(
        &self, intent: &str, session: &UserSession,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        self.classify_and_process_with_task_id(intent, session, None).await
    }

    pub async fn classify_and_process_with_task_id(
        &self, intent: &str, session: &UserSession, task_id: Option<String>,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        let classification = self.classify(intent, session).await?;
        self.process_classified_intent_with_task_id(&classification, session, task_id).await
    }

    pub async fn process_classified_intent(
        &self, classification: &ClassifiedIntent, session: &UserSession,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        self.process_classified_intent_with_task_id(classification, session, None).await
    }

    pub async fn process_classified_intent_with_task_id(
        &self, classification: &ClassifiedIntent, session: &UserSession, task_id: Option<String>,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("Processing {} intent: {}", classification.intent_type, &classification.original_text[..classification.original_text.len().min(50)]);
        match classification.intent_type {
            IntentType::AppCreate => self.handle_app_create(classification, session, task_id).await,
            IntentType::Todo => self.handle_todo(classification, session),
            IntentType::Monitor => self.handle_monitor(classification, session),
            IntentType::Action => self.handle_action_placeholder(classification, session),
            IntentType::Schedule => self.handle_schedule(classification, session),
            IntentType::Goal => self.handle_goal(classification, session),
            IntentType::Tool => self.handle_tool(classification, session),
            IntentType::Unknown => Self::handle_unknown(classification),
        }
    }

    async fn classify_with_llm(
        &self, intent: &str, bot_id: Uuid,
    ) -> Result<ClassifiedIntent, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            r#"Classify this user request into one of these intent types:

USER REQUEST: "{intent}"

INTENT TYPES:
- APP_CREATE: Create a full application, utility, calculator, tool, system, etc.
- TODO: Simple task or reminder
- MONITOR: Watch for changes and alert
- ACTION: Execute something immediately
- SCHEDULE: Create recurring automation
- GOAL: Long-term objective to achieve
- TOOL: Create a voice/chat command

YOLO MODE: NEVER ask for clarification. Always make a decision and proceed.

Respond with JSON only:
{{
"intent_type": "APP_CREATE|TODO|MONITOR|ACTION|SCHEDULE|GOAL|TOOL|UNKNOWN",
"confidence": 0.0-1.0,
"subject": "main subject or null",
"action": "main action verb or null",
"domain": "industry/domain or null",
"time_spec": {{"type": "ONCE|DAILY|WEEKLY|MONTHLY", "time": "9:00", "day": "monday"}} or null,
"condition": "trigger condition or null",
"recipient": "notification recipient or null",
"features": ["feature1", "feature2"],
"tables": ["table1", "table2"],
"trigger_phrases": ["phrase1", "phrase2"],
"target_value": "metric target or null",
"suggested_name": "short name for the resource",
"requires_clarification": false,
"clarification_question": null,
"alternatives": []
}}"#
        );

        info!("Starting LLM call for classification, prompt_len={} chars", prompt.len());
        let start = std::time::Instant::now();
        let response = self.call_llm(&prompt, bot_id).await?;
        let elapsed = start.elapsed();
        info!("LLM classification completed in {:?}, response_len={} chars", elapsed, response.len());
        trace!("LLM classification response: {}", response.chars().take(500).collect::<String>());
        Self::parse_classification_response(&response, intent)
    }

    fn parse_classification_response(
        response: &str, original_intent: &str,
    ) -> Result<ClassifiedIntent, Box<dyn std::error::Error + Send + Sync>> {
        #[derive(Deserialize)]
        struct LlmResponse {
            intent_type: String, confidence: f64, subject: Option<String>,
            action: Option<String>, domain: Option<String>,
            time_spec: Option<TimeSpecResponse>, condition: Option<String>,
            recipient: Option<String>, features: Option<Vec<String>>,
            tables: Option<Vec<String>>, trigger_phrases: Option<Vec<String>>,
            target_value: Option<String>, suggested_name: Option<String>,
            requires_clarification: Option<bool>, clarification_question: Option<String>,
            alternatives: Option<Vec<AlternativeResponse>>,
        }
        #[derive(Deserialize)]
        struct TimeSpecResponse {
            #[serde(rename = "type")] schedule_type: Option<String>,
            time: Option<String>, day: Option<String>,
            interval: Option<String>, cron_expression: Option<String>,
        }
        #[derive(Deserialize)]
        struct AlternativeResponse {
            #[serde(rename = "type")] intent_type: String,
            confidence: f64, reason: String,
        }

        let cleaned = response.trim()
            .trim_start_matches("```json").trim_start_matches("```JSON")
            .trim_start_matches("```").trim_end_matches("```").trim();

        match serde_json::from_str::<LlmResponse>(cleaned) {
            Ok(resp) => {
                let time_spec = resp.time_spec.map(|ts| TimeSpec {
                    schedule_type: match ts.schedule_type.as_deref() {
                        Some("DAILY") => ScheduleType::Daily, Some("WEEKLY") => ScheduleType::Weekly,
                        Some("MONTHLY") => ScheduleType::Monthly, Some("INTERVAL") => ScheduleType::Interval,
                        Some("CRON") => ScheduleType::Cron, _ => ScheduleType::Once,
                    },
                    time: ts.time, day: ts.day, interval: ts.interval, cron_expression: ts.cron_expression,
                });
                let alternatives = resp.alternatives.unwrap_or_default().into_iter().map(|a| AlternativeClassification {
                    intent_type: IntentType::from(a.intent_type.as_str()),
                    confidence: a.confidence, reason: a.reason,
                }).collect();
                Ok(ClassifiedIntent {
                    id: Uuid::new_v4().to_string(), original_text: original_intent.to_string(),
                    intent_type: IntentType::from(resp.intent_type.as_str()),
                    confidence: resp.confidence,
                    entities: ClassifiedEntities {
                        subject: resp.subject, action: resp.action, domain: resp.domain,
                        time_spec, condition: resp.condition, recipient: resp.recipient,
                        features: resp.features.unwrap_or_default(), tables: resp.tables.unwrap_or_default(),
                        trigger_phrases: resp.trigger_phrases.unwrap_or_default(),
                        target_value: resp.target_value,
                    },
                    suggested_name: resp.suggested_name,
                    requires_clarification: resp.requires_clarification.unwrap_or(false),
                    clarification_question: resp.clarification_question,
                    alternative_types: alternatives, classified_at: Utc::now(),
                })
            }
            Err(e) => {
                warn!("Failed to parse LLM response, using heuristic: {e}");
                Self::classify_heuristic(original_intent)
            }
        }
    }

    /// Keyword/PT-phrase classifier. Works fully offline (no LLM) and always
    /// produces a concrete `suggested_name` for later resource creation.
    /// Public so callers without LLM plumbing (e.g. Vibe tools) can classify.
    pub fn classify_heuristic(intent: &str) -> Result<ClassifiedIntent, Box<dyn std::error::Error + Send + Sync>> {
        let lower = intent.to_lowercase();
        let (intent_type, confidence) =
            if has_any(&lower, &[
                "todo dia", "todos os dias", "toda hora", "a cada dia", "todo mês", "todo mes",
                "diariamente", "semanalmente", "mensalmente", "em horas", "a cada hora",
                "every day", "daily", "weekly", "monthly", "every hour",
            ]) || (lower.contains("a cada") && (lower.contains(" hora") || lower.contains(" dia") || lower.contains(" semana") || lower.contains(" mes")))
            { (IntentType::Schedule, 0.85) }
            else if has_phrase(&lower, & ["quando eu disser", "eu disser", "comando de voz", "comando por voz",
                "when i say", "create command", "voice command", "eu falo"])
            { (IntentType::Tool, 0.80) }
            else if has_phrase(&lower, &["criar", "fazer", "construir", "gerar", "desenvolver",
                "crie um", "crie meu", "aquilo", "calcular", "calculadora", "sistema de", "portal de",
                "aplicativo", "página", "landing", "site", "blog", "loja", "dashboard", "painel",
                "create app", "build app", "crm", "inventory", "booking", "website", "form", "store"])
            { (IntentType::AppCreate, 0.75) }
            else if has_phrase(&lower, &["me lembre", "lembrou", "lembrar", "to lembre", "remind", "call me",
                "me avise quando", "amanhã", "amanha"])
            { (IntentType::Todo, 0.70) }
            else if has_phrase(&lower, &["me avise se", "me avise quando", "alerta quando", "avisar quando",
                "notifique se", "monitor", "ficar de olho", "alert when", "notify if"])
            { (IntentType::Monitor, 0.70) }
            else if has_phrase(&lower, &["envie um", "envie o", "enviar email", "excluir", "apagar",
                "exportar", "export", "delete all", "send email"])
            { (IntentType::Action, 0.65) }
            else if has_phrase(&lower, &["aumentar", "melhorar", "alcançar", "atingir", "meta de",
                "increase", "improve", "achieve"])
            { (IntentType::Goal, 0.60) }
            else { (IntentType::Unknown, 0.30) };

        let friendly = suggest_name(intent);
        Ok(ClassifiedIntent {
            id: Uuid::new_v4().to_string(), original_text: intent.to_string(),
            intent_type, confidence, entities: ClassifiedEntities::default(),
            suggested_name: if friendly.is_empty() { None } else { Some(friendly) },
            requires_clarification: intent_type == IntentType::Unknown,
            clarification_question: if intent_type == IntentType::Unknown {
                Some("Could you please clarify what you'd like me to do?".to_string())
            } else { None },
            alternative_types: Vec::new(), classified_at: Utc::now(),
        })
    }

    async fn handle_app_create(
        &self,
        classification: &ClassifiedIntent, session: &UserSession, task_id: Option<String>,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("Handling APP_CREATE intent — orchestrator integration required from caller");
        let _ = (classification, session, task_id);
        Ok(IntentResult {
            success: true, intent_type: IntentType::AppCreate,
            message: "App creation request received — use Orchestrator for full pipeline".to_string(),
            created_resources: Vec::new(), app_url: None, task_id: None,
            schedule_id: None, tool_triggers: Vec::new(),
            next_steps: vec!["Run through Orchestrator pipeline".to_string()], error: None,
        })
    }

    fn handle_todo(
        &self, classification: &ClassifiedIntent, session: &UserSession,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("Handling TODO intent");
        let task_id = Uuid::new_v4();
        let title = classification.suggested_name.clone().unwrap_or_else(|| classification.original_text.clone());
        let mut conn = self.pool.get()?;
        let branch_id = {
            #[derive(diesel::QueryableByName)]
            #[diesel(check_for_backend(diesel::pg::Pg))]
            struct BotBranchRow {
                #[diesel(sql_type = DieselUuid)]
                branch_id: Uuid,
            }
            sql_query("SELECT branch_id FROM bots WHERE id = $1 LIMIT 1")
                .bind::<DieselUuid, _>(session.bot_id)
                .get_result::<BotBranchRow>(&mut conn)
                .map(|row| row.branch_id)
                .unwrap_or_else(|_| Uuid::nil())
        };
        sql_query("INSERT INTO tasks (id, title, description, status, priority, branch_id, created_at) VALUES ($1, $2, $3, 'pending', 'normal', $4, NOW())")
            .bind::<DieselUuid, _>(task_id).bind::<Text, _>(&title).bind::<Text, _>(&classification.original_text)
            .bind::<DieselUuid, _>(branch_id)
            .execute(&mut conn)?;
        Ok(IntentResult {
            success: true, intent_type: IntentType::Todo,
            message: format!("Task saved: {title}"),
            created_resources: vec![CreatedResource { resource_type: "task".to_string(), name: title, path: None }],
            app_url: None, task_id: Some(task_id.to_string()), schedule_id: None,
            tool_triggers: Vec::new(), next_steps: vec!["View tasks in your task list".to_string()], error: None,
        })
    }

    fn handle_monitor(
        &self, classification: &ClassifiedIntent, session: &UserSession,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("Handling MONITOR intent");
        let subject = classification.entities.subject.clone().unwrap_or_else(|| "data".to_string());
        let condition = classification.entities.condition.clone().unwrap_or_else(|| "changes".to_string());
        let handler_name = format!("monitor_{}.bas", subject.to_lowercase().replace(' ', "_"));
        let basic_code = format!(
            "' Monitor: {subject}\n' Condition: {condition}\n' Created: {}\n\nON CHANGE \"{subject}\"\ncurrent_value = GET \"{subject}\"\nIF {condition} THEN\n  TALK \"Alert: {subject} has changed\"\nEND IF\nEND ON\n",
            Utc::now().format("%Y-%m-%d %H:%M")
        );
        let event_path = format!("events/{handler_name}");
        self.save_basic_file(session.bot_id, &event_path, &basic_code)?;
        Ok(IntentResult {
            success: true, intent_type: IntentType::Monitor,
            message: format!("Monitor created for: {subject}"),
            created_resources: vec![CreatedResource { resource_type: "event".to_string(), name: handler_name, path: Some(event_path) }],
            app_url: None, task_id: None, schedule_id: None, tool_triggers: Vec::new(),
            next_steps: vec![format!("You'll be notified when {subject} {condition}")], error: None,
        })
    }

    fn handle_action_placeholder(
        &self, classification: &ClassifiedIntent, _session: &UserSession,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("Handling ACTION intent (placeholder — IntentCompiler integration needed)");
        Ok(IntentResult {
            success: true, intent_type: IntentType::Action,
            message: format!("Action noted: {}", classification.original_text),
            created_resources: Vec::new(), app_url: None, task_id: None,
            schedule_id: None, tool_triggers: Vec::new(),
            next_steps: vec!["Use IntentCompiler for execution planning".to_string()], error: None,
        })
    }

    fn handle_schedule(
        &self, classification: &ClassifiedIntent, session: &UserSession,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("Handling SCHEDULE intent");
        let schedule_name = classification.suggested_name.clone().unwrap_or_else(|| "scheduled-task".to_string()).to_lowercase().replace(' ', "-");
        let time_spec = classification.entities.time_spec.as_ref().map(|ts| {
            format!("{} at {}", match ts.schedule_type {
                ScheduleType::Daily => "Every day", ScheduleType::Weekly => "Every week",
                ScheduleType::Monthly => "Every month", _ => "Once",
            }, ts.time.as_deref().unwrap_or("9:00 AM"))
        }).unwrap_or_else(|| "Every day at 9:00 AM".to_string());
        let scheduler_file = format!("{schedule_name}.bas");
        let basic_code = format!(
            "' Scheduler: {schedule_name}\n' Schedule: {time_spec}\n' Created: {}\n\nSET SCHEDULE \"{time_spec}\"\nTALK \"Running scheduled task: {schedule_name}\"\nEND SCHEDULE\n",
            Utc::now().format("%Y-%m-%d %H:%M")
        );
        let scheduler_path = format!("schedulers/{scheduler_file}");
        self.save_basic_file(session.bot_id, &scheduler_path, &basic_code)?;
        let schedule_id = Uuid::new_v4();
        Ok(IntentResult {
            success: true, intent_type: IntentType::Schedule,
            message: format!("Scheduler created: {scheduler_file}\nSchedule: {time_spec}"),
            created_resources: vec![CreatedResource { resource_type: "scheduler".to_string(), name: scheduler_file, path: Some(scheduler_path) }],
            app_url: None, task_id: None, schedule_id: Some(schedule_id.to_string()), tool_triggers: Vec::new(),
            next_steps: vec![format!("The task will run {time_spec}")], error: None,
        })
    }

    fn handle_goal(
        &self, classification: &ClassifiedIntent, session: &UserSession,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("Handling GOAL intent");
        let goal_name = classification.suggested_name.clone().unwrap_or_else(|| "goal".to_string());
        let target = classification.entities.target_value.clone().unwrap_or_else(|| "unspecified".to_string());
        let basic_code = format!(
            "' Goal: {goal_name}\n' Target: {target}\n' Created: {}\n\nSET GOAL \"{goal_name}\"\nTARGET = \"{target}\"\ncurrent = GET_METRIC \"{goal_name}\"\nTALK \"Goal progress: \" + current + \" / \" + TARGET\nEND GOAL\n",
            Utc::now().format("%Y-%m-%d %H:%M")
        );
        let goal_file = format!("{}.bas", goal_name.to_lowercase().replace(' ', "-"));
        let goal_path = format!("goals/{goal_file}");
        self.save_basic_file(session.bot_id, &goal_path, &basic_code)?;
        Ok(IntentResult {
            success: true, intent_type: IntentType::Goal,
            message: format!("Goal created: {goal_name}\nTarget: {target}"),
            created_resources: vec![CreatedResource { resource_type: "goal".to_string(), name: goal_name, path: Some(goal_path) }],
            app_url: None, task_id: None, schedule_id: None, tool_triggers: Vec::new(),
            next_steps: vec!["The system will work toward this goal autonomously".to_string()], error: None,
        })
    }

    fn handle_tool(
        &self, classification: &ClassifiedIntent, session: &UserSession,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("Handling TOOL intent");
        let tool_name = classification.suggested_name.clone().unwrap_or_else(|| "custom-command".to_string()).to_lowercase().replace(' ', "-");
        let triggers = if classification.entities.trigger_phrases.is_empty() {
            vec![tool_name.clone()]
        } else { classification.entities.trigger_phrases.clone() };
        let triggers_str = triggers.iter().map(|t| format!("\"{}\"", t)).collect::<Vec<_>>().join(", ");
        let tool_file = format!("{tool_name}.bas");
        let basic_code = format!(
            "' Tool: {tool_name}\n' Triggers: {triggers_str}\n' Created: {}\n\nTRIGGER {triggers_str}\nTALK \"Running command: {tool_name}\"\nEND TRIGGER\n",
            Utc::now().format("%Y-%m-%d %H:%M")
        );
        let tool_path = format!("tools/{tool_file}");
        self.save_basic_file(session.bot_id, &tool_path, &basic_code)?;
        Ok(IntentResult {
            success: true, intent_type: IntentType::Tool,
            message: format!("Command created: {tool_file}\nTriggers: {}", triggers.join(", ")),
            created_resources: vec![CreatedResource { resource_type: "tool".to_string(), name: tool_file, path: Some(tool_path) }],
            app_url: None, task_id: None, schedule_id: None, tool_triggers: triggers,
            next_steps: vec!["Say any of the trigger phrases to use the command".to_string()], error: None,
        })
    }

    fn handle_unknown(classification: &ClassifiedIntent) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("Handling UNKNOWN intent");
        let suggestions = if classification.alternative_types.is_empty() {
            "- Create an app\n- Add a task\n- Set up monitoring\n- Schedule automation".to_string()
        } else {
            classification.alternative_types.iter().map(|a| format!("- {}: {}", a.intent_type, a.reason)).collect::<Vec<_>>().join("\n")
        };
        Ok(IntentResult {
            success: false, intent_type: IntentType::Unknown,
            message: format!("I'm not sure what you'd like me to do. Could you clarify?\n\nPossible interpretations:\n{suggestions}"),
            created_resources: Vec::new(), app_url: None, task_id: None,
            schedule_id: None, tool_triggers: Vec::new(),
            next_steps: vec!["Provide more details about what you want".to_string()], error: None,
        })
    }

    async fn call_llm(&self, _prompt: &str, _bot_id: Uuid) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(feature = "llm")]
        {
            let model = self.config_ops.get_config(&_bot_id, "llm-model", None)
                .unwrap_or_else(|_| self.config_ops.get_config(&Uuid::nil(), "llm-model", None)
                .unwrap_or_else(|_| "gpt-4".to_string()));
            let key = self.config_ops.get_config(&_bot_id, "llm-key", None)
                .unwrap_or_else(|_| self.config_ops.get_config(&Uuid::nil(), "llm-key", None)
                .unwrap_or_default());
            let (tx, mut rx) = tokio::sync::mpsc::channel(100);
            let llm_config = serde_json::json!({"temperature": 0.3, "max_tokens": 1000});
            self.llm_ops.generate_stream(_prompt, &llm_config, tx, &model, &key, None).await?;
            let mut response = String::new();
            while let Some(chunk) = rx.recv().await { response.push_str(&chunk); }
            Ok(response)
        }
        #[cfg(not(feature = "llm"))]
        { warn!("LLM feature not enabled, using heuristic classification"); Ok("{}".to_string()) }
    }

    fn save_basic_file(&self, bot_id: Uuid, path: &str, content: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.state.broadcast_task_progress(
            TaskProgressEvent::new(&bot_id.to_string(), "save_basic_file", "Persisting BASIC to Drive")
                .with_details(path.to_string()),
        );
        let mut conn = self.pool.get().map_err(|e| format!("db pool: {e}"))?;
        #[derive(diesel::QueryableByName)]
        struct BotNameRow {
            #[diesel(sql_type = Text)]
            name: String,
        }
        let bot = sql_query("SELECT name FROM bots WHERE id = $1")
            .bind::<DieselUuid, _>(bot_id)
            .get_result::<BotNameRow>(&mut conn)
            .optional()
            .map_err(|e| format!("resolve bot: {e}"))?;
        let Some(bot) = bot else {
            return Err(format!("Bot not found for id {bot_id}").into());
        };
        let Some(ops) = self.state.file_ops() else {
            return Err("Drive ops not available — cannot persist generated BASIC".into());
        };
        let bucket = format!("{}.gbai", bot.name);
        let key = format!("{}.gbdialog/{}", bot.name, path.trim_start_matches('/'));
        ops.put_object(&bucket, &key, content.as_bytes().to_vec(), "text/plain")?;
        info!("Saved BASIC file to Drive: {bucket}/{key}");
        Ok(())
    }

    fn store_classification(&self, classification: &ClassifiedIntent, session: &UserSession) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.pool.get().ok();
        if let Some(ref mut conn) = conn {
            sql_query("INSERT INTO intent_classifications (id, bot_id, session_id, original_text, intent_type, confidence, entities, created_at) VALUES ($1, $2, $3, $4, $5, $6, $7, NOW()) ON CONFLICT DO NOTHING")
                .bind::<DieselUuid, _>(Uuid::parse_str(&classification.id).unwrap_or(Uuid::nil()))
                .bind::<DieselUuid, _>(session.bot_id)
                .bind::<DieselUuid, _>(session.id)
                .bind::<Text, _>(&classification.original_text)
                .bind::<Text, _>(&classification.intent_type.to_string())
                .bind::<Float8, _>(classification.confidence)
                .bind::<Text, _>(serde_json::to_string(&classification.entities).unwrap_or_default())
                .execute(conn).ok();
        }
        Ok(())
    }
}

/// Contains any of the substrings (case already normalized by caller).
fn has_any(lower: &str, phrases: &[&str]) -> bool {
    phrases.iter().any(|p| lower.contains(p))
}

/// Contains the full phrase as a word boundary (phrase + whitespace or EOS).
/// Multi-word phrases fall back to substring matching (e.g. "quando eu disser").
fn has_phrase(lower: &str, phrases: &[&str]) -> bool {
    phrases.iter().any(|p| {
        if p.contains(' ') {
            lower.contains(p)
        } else {
            lower.split_whitespace().any(|w| w == *p)
        }
    })
}

/// Deterministic resource-name suggestion without any LLM; slugs the first
/// meaningful words of the intent (PT or PT/EN mixture), e.g.
/// "fala todo dia que o comercial vendeu" -> "comercial_vendeu_diario".
fn suggest_name(intent: &str) -> String {
    let mut words: Vec<String> = Vec::new();
    for word in intent.to_lowercase().split_whitespace() {
        if word.len() < 3 || word.chars().any(|c| !c.is_alphanumeric()) {
            continue;
        }
        let kept = word;
        words.push(kept.to_string());
    }
    let name = words.iter().take(3).cloned().collect::<Vec<_>>().join("_");
    if name.is_empty() {
        return String::new();
    }
    let trimmed: String = name.chars().filter(|c| c.is_alphanumeric() || *c == '_').collect();
    let mut wrapped: Vec<char> = Vec::new();
    for c in trimmed.chars() {
        if wrapped.is_empty() {
            wrapped.push(c.to_ascii_lowercase());
        } else {
            wrapped.push(c);
        }
    }
    wrapped.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::types::*;

    #[test]
    fn test_intent_type_from_str() {
        assert_eq!(IntentType::from("APP_CREATE"), IntentType::AppCreate);
        assert_eq!(IntentType::from("app"), IntentType::AppCreate);
        assert_eq!(IntentType::from("TODO"), IntentType::Todo);
        assert_eq!(IntentType::from("unknown_value"), IntentType::Unknown);
    }

    #[test]
    fn test_intent_type_display() {
        assert_eq!(IntentType::AppCreate.to_string(), "APP_CREATE");
        assert_eq!(IntentType::Todo.to_string(), "TODO");
    }
}
