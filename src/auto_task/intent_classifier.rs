use crate::auto_task::app_generator::AppGenerator;
use crate::auto_task::intent_compiler::IntentCompiler;
use crate::core::config::ConfigManager;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::{Text, Uuid as DieselUuid};
use log::{error, info, trace, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum IntentType {
    AppCreate,
    Todo,
    Monitor,
    Action,
    Schedule,
    Goal,
    Tool,
    Unknown,
}

impl std::fmt::Display for IntentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AppCreate => write!(f, "APP_CREATE"),
            Self::Todo => write!(f, "TODO"),
            Self::Monitor => write!(f, "MONITOR"),
            Self::Action => write!(f, "ACTION"),
            Self::Schedule => write!(f, "SCHEDULE"),
            Self::Goal => write!(f, "GOAL"),
            Self::Tool => write!(f, "TOOL"),
            Self::Unknown => write!(f, "UNKNOWN"),
        }
    }
}

impl From<&str> for IntentType {
    fn from(s: &str) -> Self {
        match s.to_uppercase().as_str() {
            "APP_CREATE" | "APP" | "APPLICATION" | "CREATE_APP" => Self::AppCreate,
            "TODO" | "TASK" | "REMINDER" => Self::Todo,
            "MONITOR" | "WATCH" | "ALERT" | "ON_CHANGE" => Self::Monitor,
            "ACTION" | "EXECUTE" | "DO" | "RUN" => Self::Action,
            "SCHEDULE" | "SCHEDULED" | "DAILY" | "WEEKLY" | "MONTHLY" | "CRON" => Self::Schedule,
            "GOAL" | "OBJECTIVE" | "TARGET" | "ACHIEVE" => Self::Goal,
            "TOOL" | "COMMAND" | "TRIGGER" | "WHEN_I_SAY" => Self::Tool,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassifiedIntent {
    pub id: String,
    pub original_text: String,
    pub intent_type: IntentType,
    pub confidence: f64,
    pub entities: ClassifiedEntities,
    pub suggested_name: Option<String>,
    pub requires_clarification: bool,
    pub clarification_question: Option<String>,
    pub alternative_types: Vec<AlternativeClassification>,
    pub classified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClassifiedEntities {
    pub subject: Option<String>,
    pub action: Option<String>,
    pub domain: Option<String>,
    pub time_spec: Option<TimeSpec>,
    pub condition: Option<String>,
    pub recipient: Option<String>,
    pub features: Vec<String>,
    pub tables: Vec<String>,
    pub trigger_phrases: Vec<String>,
    pub target_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSpec {
    pub schedule_type: ScheduleType,
    pub time: Option<String>,
    pub day: Option<String>,
    pub interval: Option<String>,
    pub cron_expression: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduleType {
    Once,
    Daily,
    Weekly,
    Monthly,
    Interval,
    Cron,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeClassification {
    pub intent_type: IntentType,
    pub confidence: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntentResult {
    pub success: bool,
    pub intent_type: IntentType,
    pub message: String,
    pub created_resources: Vec<CreatedResource>,
    pub app_url: Option<String>,
    pub task_id: Option<String>,
    pub schedule_id: Option<String>,
    pub tool_triggers: Vec<String>,
    pub next_steps: Vec<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreatedResource {
    pub resource_type: String,
    pub name: String,
    pub path: Option<String>,
}

pub struct IntentClassifier {
    state: Arc<AppState>,
    intent_compiler: IntentCompiler,
}

impl IntentClassifier {
    pub fn new(state: Arc<AppState>) -> Self {
        Self {
            state: state.clone(),
            intent_compiler: IntentCompiler::new(state),
        }
    }

    /// Classify an intent and determine which handler should process it
    pub async fn classify(
        &self,
        intent: &str,
        session: &UserSession,
    ) -> Result<ClassifiedIntent, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Classifying intent for session {}: {}",
            session.id,
            &intent[..intent.len().min(100)]
        );

        // Use LLM to classify the intent
        let classification = self.classify_with_llm(intent, session.bot_id).await?;

        // Store classification for analytics
        self.store_classification(&classification, session)?;

        Ok(classification)
    }

    /// Classify and then process the intent through the appropriate handler
    pub async fn classify_and_process(
        &self,
        intent: &str,
        session: &UserSession,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        let classification = self.classify(intent, session).await?;

        // If clarification is needed, return early
        if classification.requires_clarification {
            return Ok(IntentResult {
                success: false,
                intent_type: classification.intent_type,
                message: classification
                    .clarification_question
                    .unwrap_or_else(|| "Could you please provide more details?".to_string()),
                created_resources: Vec::new(),
                app_url: None,
                task_id: None,
                schedule_id: None,
                tool_triggers: Vec::new(),
                next_steps: vec!["Provide more information".to_string()],
                error: None,
            });
        }

        // Route to appropriate handler
        self.process_classified_intent(&classification, session)
            .await
    }

    /// Process a classified intent through the appropriate handler
    pub async fn process_classified_intent(
        &self,
        classification: &ClassifiedIntent,
        session: &UserSession,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        info!(
            "Processing {} intent: {}",
            classification.intent_type,
            &classification.original_text[..classification.original_text.len().min(50)]
        );

        match classification.intent_type {
            IntentType::AppCreate => self.handle_app_create(classification, session).await,
            IntentType::Todo => self.handle_todo(classification, session),
            IntentType::Monitor => self.handle_monitor(classification, session),
            IntentType::Action => self.handle_action(classification, session).await,
            IntentType::Schedule => self.handle_schedule(classification, session),
            IntentType::Goal => self.handle_goal(classification, session),
            IntentType::Tool => self.handle_tool(classification, session),
            IntentType::Unknown => Self::handle_unknown(classification),
        }
    }

    /// Use LLM to classify the intent
    async fn classify_with_llm(
        &self,
        intent: &str,
        bot_id: Uuid,
    ) -> Result<ClassifiedIntent, Box<dyn std::error::Error + Send + Sync>> {
        let prompt = format!(
            r#"Classify this user request into one of these intent types:

USER REQUEST: "{intent}"

INTENT TYPES:
- APP_CREATE: Create a full application (CRM, inventory, booking system, etc.)
  Keywords: "create app", "build system", "make application", "CRM", "management system"

- TODO: Simple task or reminder
  Keywords: "call", "remind me", "don't forget", "tomorrow", "later"

- MONITOR: Watch for changes and alert
  Keywords: "alert when", "notify if", "watch", "monitor", "track changes"

- ACTION: Execute something immediately
  Keywords: "send email", "delete", "update all", "export", "do now"

- SCHEDULE: Create recurring automation
  Keywords: "every day", "daily at", "weekly", "monthly", "at 9am"

- GOAL: Long-term objective to achieve
  Keywords: "increase", "improve", "achieve", "reach target", "grow by"

- TOOL: Create a voice/chat command
  Keywords: "when I say", "create command", "shortcut for", "trigger"

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
    "alternatives": [
        {{"type": "OTHER_TYPE", "confidence": 0.3, "reason": "could also be..."}}
    ]
}}"#
        );

        let response = self.call_llm(&prompt, bot_id).await?;
        Self::parse_classification_response(&response, intent)
    }

    fn parse_classification_response(
        response: &str,
        original_intent: &str,
    ) -> Result<ClassifiedIntent, Box<dyn std::error::Error + Send + Sync>> {
        #[derive(Deserialize)]
        struct LlmResponse {
            intent_type: String,
            confidence: f64,
            subject: Option<String>,
            action: Option<String>,
            domain: Option<String>,
            time_spec: Option<TimeSpecResponse>,
            condition: Option<String>,
            recipient: Option<String>,
            features: Option<Vec<String>>,
            tables: Option<Vec<String>>,
            trigger_phrases: Option<Vec<String>>,
            target_value: Option<String>,
            suggested_name: Option<String>,
            requires_clarification: Option<bool>,
            clarification_question: Option<String>,
            alternatives: Option<Vec<AlternativeResponse>>,
        }

        #[derive(Deserialize)]
        struct TimeSpecResponse {
            #[serde(rename = "type")]
            schedule_type: Option<String>,
            time: Option<String>,
            day: Option<String>,
            interval: Option<String>,
            cron_expression: Option<String>,
        }

        #[derive(Deserialize)]
        struct AlternativeResponse {
            #[serde(rename = "type")]
            intent_type: String,
            confidence: f64,
            reason: String,
        }

        // Try to parse, fall back to heuristic classification
        let parsed: Result<LlmResponse, _> = serde_json::from_str(response);

        match parsed {
            Ok(resp) => {
                let intent_type = IntentType::from(resp.intent_type.as_str());

                let time_spec = resp.time_spec.map(|ts| TimeSpec {
                    schedule_type: match ts.schedule_type.as_deref() {
                        Some("DAILY") => ScheduleType::Daily,
                        Some("WEEKLY") => ScheduleType::Weekly,
                        Some("MONTHLY") => ScheduleType::Monthly,
                        Some("INTERVAL") => ScheduleType::Interval,
                        Some("CRON") => ScheduleType::Cron,
                        _ => ScheduleType::Once,
                    },
                    time: ts.time,
                    day: ts.day,
                    interval: ts.interval,
                    cron_expression: ts.cron_expression,
                });

                let alternatives = resp
                    .alternatives
                    .unwrap_or_default()
                    .into_iter()
                    .map(|a| AlternativeClassification {
                        intent_type: IntentType::from(a.intent_type.as_str()),
                        confidence: a.confidence,
                        reason: a.reason,
                    })
                    .collect();

                Ok(ClassifiedIntent {
                    id: Uuid::new_v4().to_string(),
                    original_text: original_intent.to_string(),
                    intent_type,
                    confidence: resp.confidence,
                    entities: ClassifiedEntities {
                        subject: resp.subject,
                        action: resp.action,
                        domain: resp.domain,
                        time_spec,
                        condition: resp.condition,
                        recipient: resp.recipient,
                        features: resp.features.unwrap_or_default(),
                        tables: resp.tables.unwrap_or_default(),
                        trigger_phrases: resp.trigger_phrases.unwrap_or_default(),
                        target_value: resp.target_value,
                    },
                    suggested_name: resp.suggested_name,
                    requires_clarification: resp.requires_clarification.unwrap_or(false),
                    clarification_question: resp.clarification_question,
                    alternative_types: alternatives,
                    classified_at: Utc::now(),
                })
            }
            Err(e) => {
                warn!("Failed to parse LLM response, using heuristic: {e}");
                Self::classify_heuristic(original_intent)
            }
        }
    }

    fn classify_heuristic(
        intent: &str,
    ) -> Result<ClassifiedIntent, Box<dyn std::error::Error + Send + Sync>> {
        let lower = intent.to_lowercase();

        let (intent_type, confidence) = if lower.contains("create app")
            || lower.contains("build app")
            || lower.contains("make app")
            || lower.contains("crm")
            || lower.contains("management system")
            || lower.contains("inventory")
            || lower.contains("booking")
        {
            (IntentType::AppCreate, 0.75)
        } else if lower.contains("remind")
            || lower.contains("call ")
            || lower.contains("tomorrow")
            || lower.contains("don't forget")
        {
            (IntentType::Todo, 0.70)
        } else if lower.contains("alert when")
            || lower.contains("notify if")
            || lower.contains("watch for")
            || lower.contains("monitor")
        {
            (IntentType::Monitor, 0.70)
        } else if lower.contains("send email")
            || lower.contains("delete all")
            || lower.contains("update all")
            || lower.contains("export")
        {
            (IntentType::Action, 0.65)
        } else if lower.contains("every day")
            || lower.contains("daily")
            || lower.contains("weekly")
            || lower.contains("at 9")
            || lower.contains("at 8")
        {
            (IntentType::Schedule, 0.70)
        } else if lower.contains("increase")
            || lower.contains("improve")
            || lower.contains("achieve")
            || lower.contains("grow by")
        {
            (IntentType::Goal, 0.60)
        } else if lower.contains("when i say")
            || lower.contains("create command")
            || lower.contains("shortcut")
        {
            (IntentType::Tool, 0.70)
        } else {
            (IntentType::Unknown, 0.30)
        };

        Ok(ClassifiedIntent {
            id: Uuid::new_v4().to_string(),
            original_text: intent.to_string(),
            intent_type,
            confidence,
            entities: ClassifiedEntities::default(),
            suggested_name: None,
            requires_clarification: intent_type == IntentType::Unknown,
            clarification_question: if intent_type == IntentType::Unknown {
                Some("Could you please clarify what you'd like me to do?".to_string())
            } else {
                None
            },
            alternative_types: Vec::new(),
            classified_at: Utc::now(),
        })
    }

    async fn handle_app_create(
        &self,
        classification: &ClassifiedIntent,
        session: &UserSession,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("Handling APP_CREATE intent");

        let app_generator = AppGenerator::new(self.state.clone());

        match app_generator
            .generate_app(&classification.original_text, session)
            .await
        {
            Ok(app) => {
                let mut resources = Vec::new();

                // Track created tables
                for table in &app.tables {
                    resources.push(CreatedResource {
                        resource_type: "table".to_string(),
                        name: table.name.clone(),
                        path: Some("tables.bas".to_string()),
                    });
                }

                for page in &app.pages {
                    resources.push(CreatedResource {
                        resource_type: "page".to_string(),
                        name: page.filename.clone(),
                        path: Some(page.filename.clone()),
                    });
                }

                for tool in &app.tools {
                    resources.push(CreatedResource {
                        resource_type: "tool".to_string(),
                        name: tool.filename.clone(),
                        path: Some(tool.filename.clone()),
                    });
                }

                let app_url = format!("/apps/{}", app.name.to_lowercase().replace(' ', "-"));

                Ok(IntentResult {
                    success: true,
                    intent_type: IntentType::AppCreate,
                    message: format!(
                        "Done:\n{}\nApp available at {}",
                        resources
                            .iter()
                            .filter(|r| r.resource_type == "table")
                            .map(|r| format!("{} table created", r.name))
                            .collect::<Vec<_>>()
                            .join("\n"),
                        app_url
                    ),
                    created_resources: resources,
                    app_url: Some(app_url),
                    task_id: None,
                    schedule_id: None,
                    tool_triggers: Vec::new(),
                    next_steps: vec![
                        "Open the app to start using it".to_string(),
                        "Use Designer to customize the app".to_string(),
                    ],
                    error: None,
                })
            }
            Err(e) => {
                error!("Failed to generate app: {e}");
                Ok(IntentResult {
                    success: false,
                    intent_type: IntentType::AppCreate,
                    message: "Failed to create the application".to_string(),
                    created_resources: Vec::new(),
                    app_url: None,
                    task_id: None,
                    schedule_id: None,
                    tool_triggers: Vec::new(),
                    next_steps: vec!["Try again with more details".to_string()],
                    error: Some(e.to_string()),
                })
            }
        }
    }

    fn handle_todo(
        &self,
        classification: &ClassifiedIntent,
        session: &UserSession,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("Handling TODO intent");

        let task_id = Uuid::new_v4();
        let title = classification
            .suggested_name
            .clone()
            .unwrap_or_else(|| classification.original_text.clone());

        let mut conn = self.state.conn.get()?;

        // Insert into tasks table
        sql_query(
            "INSERT INTO tasks (id, bot_id, title, description, status, priority, created_at)
             VALUES ($1, $2, $3, $4, 'pending', 'normal', NOW())",
        )
        .bind::<DieselUuid, _>(task_id)
        .bind::<DieselUuid, _>(session.bot_id)
        .bind::<Text, _>(&title)
        .bind::<Text, _>(&classification.original_text)
        .execute(&mut conn)?;

        Ok(IntentResult {
            success: true,
            intent_type: IntentType::Todo,
            message: format!("Task saved: {title}"),
            created_resources: vec![CreatedResource {
                resource_type: "task".to_string(),
                name: title,
                path: None,
            }],
            app_url: None,
            task_id: Some(task_id.to_string()),
            schedule_id: None,
            tool_triggers: Vec::new(),
            next_steps: vec!["View tasks in your task list".to_string()],
            error: None,
        })
    }

    fn handle_monitor(
        &self,
        classification: &ClassifiedIntent,
        session: &UserSession,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("Handling MONITOR intent");

        let subject = classification
            .entities
            .subject
            .clone()
            .unwrap_or_else(|| "data".to_string());
        let condition = classification
            .entities
            .condition
            .clone()
            .unwrap_or_else(|| "changes".to_string());

        // Generate ON CHANGE handler BASIC code
        let handler_name = format!("monitor_{}.bas", subject.to_lowercase().replace(' ', "_"));

        let basic_code = format!(
            r#"' Monitor: {subject}
' Condition: {condition}
' Created: {}

ON CHANGE "{subject}"
    current_value = GET "{subject}"
    IF {condition} THEN
        TALK "Alert: {subject} has changed"
        ' Add notification logic here
    END IF
END ON
"#,
            Utc::now().format("%Y-%m-%d %H:%M")
        );

        // Save to .gbdialog/events/
        let event_path = format!(".gbdialog/events/{handler_name}");
        self.save_basic_file(session.bot_id, &event_path, &basic_code)?;

        Ok(IntentResult {
            success: true,
            intent_type: IntentType::Monitor,
            message: format!("Monitor created for: {subject}"),
            created_resources: vec![CreatedResource {
                resource_type: "event".to_string(),
                name: handler_name,
                path: Some(event_path),
            }],
            app_url: None,
            task_id: None,
            schedule_id: None,
            tool_triggers: Vec::new(),
            next_steps: vec![format!("You'll be notified when {subject} {condition}")],
            error: None,
        })
    }

    async fn handle_action(
        &self,
        classification: &ClassifiedIntent,
        session: &UserSession,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("Handling ACTION intent");

        // Compile the intent into an execution plan
        let compiled = self
            .intent_compiler
            .compile(&classification.original_text, session)
            .await?;

        // For immediate actions, we'd execute the plan
        // For safety, high-risk actions require approval
        if compiled.risk_assessment.requires_human_review {
            return Ok(IntentResult {
                success: false,
                intent_type: IntentType::Action,
                message: format!(
                    "This action requires approval:\n{}",
                    compiled.risk_assessment.review_reason.unwrap_or_default()
                ),
                created_resources: Vec::new(),
                app_url: None,
                task_id: Some(compiled.id),
                schedule_id: None,
                tool_triggers: Vec::new(),
                next_steps: vec!["Approve the action to proceed".to_string()],
                error: None,
            });
        }

        // Execute low-risk actions immediately
        // In production, this would run the BASIC program
        Ok(IntentResult {
            success: true,
            intent_type: IntentType::Action,
            message: format!(
                "Executing: {}\nSteps: {}",
                compiled.plan.name,
                compiled.plan.steps.len()
            ),
            created_resources: Vec::new(),
            app_url: None,
            task_id: Some(compiled.id),
            schedule_id: None,
            tool_triggers: Vec::new(),
            next_steps: vec!["Action is being executed".to_string()],
            error: None,
        })
    }

    fn handle_schedule(
        &self,
        classification: &ClassifiedIntent,
        session: &UserSession,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("Handling SCHEDULE intent");

        let schedule_name = classification
            .suggested_name
            .clone()
            .unwrap_or_else(|| "scheduled-task".to_string())
            .to_lowercase()
            .replace(' ', "-");

        let time_spec = classification
            .entities
            .time_spec
            .as_ref()
            .map(|ts| {
                format!(
                    "{} at {}",
                    match ts.schedule_type {
                        ScheduleType::Daily => "Every day",
                        ScheduleType::Weekly => "Every week",
                        ScheduleType::Monthly => "Every month",
                        _ => "Once",
                    },
                    ts.time.as_deref().unwrap_or("9:00 AM")
                )
            })
            .unwrap_or_else(|| "Every day at 9:00 AM".to_string());

        // Generate scheduler BASIC code
        let scheduler_file = format!("{schedule_name}.bas");
        let basic_code = format!(
            r#"' Scheduler: {schedule_name}
' Schedule: {time_spec}
' Created: {}

SET SCHEDULE "{time_spec}"
    ' Task logic from: {}
    TALK "Running scheduled task: {schedule_name}"
    ' Add your automation logic here
END SCHEDULE
"#,
            Utc::now().format("%Y-%m-%d %H:%M"),
            classification.original_text
        );

        // Save to .gbdialog/schedulers/
        let scheduler_path = format!(".gbdialog/schedulers/{scheduler_file}");
        self.save_basic_file(session.bot_id, &scheduler_path, &basic_code)?;

        let schedule_id = Uuid::new_v4();

        Ok(IntentResult {
            success: true,
            intent_type: IntentType::Schedule,
            message: format!("Scheduler created: {scheduler_file}\nSchedule: {time_spec}"),
            created_resources: vec![CreatedResource {
                resource_type: "scheduler".to_string(),
                name: scheduler_file,
                path: Some(scheduler_path),
            }],
            app_url: None,
            task_id: None,
            schedule_id: Some(schedule_id.to_string()),
            tool_triggers: Vec::new(),
            next_steps: vec![format!("The task will run {time_spec}")],
            error: None,
        })
    }

    fn handle_goal(
        &self,
        classification: &ClassifiedIntent,
        session: &UserSession,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("Handling GOAL intent");

        let goal_name = classification
            .suggested_name
            .clone()
            .unwrap_or_else(|| "goal".to_string());
        let target = classification
            .entities
            .target_value
            .clone()
            .unwrap_or_else(|| "unspecified".to_string());

        let _goal_id = Uuid::new_v4();

        // Goals are more complex - they create a monitoring + action loop
        let basic_code = format!(
            r#"' Goal: {goal_name}
' Target: {target}
' Created: {}

' This goal runs as an autonomous loop
SET GOAL "{goal_name}"
    TARGET = "{target}"

    ' Check current metrics
    current = GET_METRIC "{goal_name}"

    ' LLM analyzes progress and suggests actions
    analysis = LLM "Analyze progress toward {target}. Current: " + current

    ' Execute suggested improvements
    IF analysis.has_action THEN
        EXECUTE analysis.action
    END IF

    ' Report progress
    TALK "Goal progress: " + current + " / " + TARGET
END GOAL
"#,
            Utc::now().format("%Y-%m-%d %H:%M")
        );

        // Save to .gbdialog/goals/
        let goal_file = format!("{}.bas", goal_name.to_lowercase().replace(' ', "-"));
        let goal_path = format!(".gbdialog/goals/{goal_file}");
        self.save_basic_file(session.bot_id, &goal_path, &basic_code)?;

        Ok(IntentResult {
            success: true,
            intent_type: IntentType::Goal,
            message: format!("Goal created: {goal_name}\nTarget: {target}"),
            created_resources: vec![CreatedResource {
                resource_type: "goal".to_string(),
                name: goal_name,
                path: Some(goal_path),
            }],
            app_url: None,
            task_id: None,
            schedule_id: None,
            tool_triggers: Vec::new(),
            next_steps: vec![
                "The system will work toward this goal autonomously".to_string(),
                "Check progress in the Goals dashboard".to_string(),
            ],
            error: None,
        })
    }

    fn handle_tool(
        &self,
        classification: &ClassifiedIntent,
        session: &UserSession,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("Handling TOOL intent");

        let tool_name = classification
            .suggested_name
            .clone()
            .unwrap_or_else(|| "custom-command".to_string())
            .to_lowercase()
            .replace(' ', "-");

        let triggers = if classification.entities.trigger_phrases.is_empty() {
            vec![tool_name.clone()]
        } else {
            classification.entities.trigger_phrases.clone()
        };

        let triggers_str = triggers
            .iter()
            .map(|t| format!("\"{}\"", t))
            .collect::<Vec<_>>()
            .join(", ");

        // Generate tool BASIC code
        let tool_file = format!("{tool_name}.bas");
        let basic_code = format!(
            r#"' Tool: {tool_name}
' Triggers: {triggers_str}
' Created: {}

TRIGGER {triggers_str}
    ' Command logic from: {}
    TALK "Running command: {tool_name}"
    ' Add your command logic here
END TRIGGER
"#,
            Utc::now().format("%Y-%m-%d %H:%M"),
            classification.original_text
        );

        // Save to .gbdialog/tools/
        let tool_path = format!(".gbdialog/tools/{tool_file}");
        self.save_basic_file(session.bot_id, &tool_path, &basic_code)?;

        Ok(IntentResult {
            success: true,
            intent_type: IntentType::Tool,
            message: format!(
                "Command created: {tool_file}\nTriggers: {}",
                triggers.join(", ")
            ),
            created_resources: vec![CreatedResource {
                resource_type: "tool".to_string(),
                name: tool_file,
                path: Some(tool_path),
            }],
            app_url: None,
            task_id: None,
            schedule_id: None,
            tool_triggers: triggers,
            next_steps: vec!["Say any of the trigger phrases to use the command".to_string()],
            error: None,
        })
    }

    fn handle_unknown(
        classification: &ClassifiedIntent,
    ) -> Result<IntentResult, Box<dyn std::error::Error + Send + Sync>> {
        info!("Handling UNKNOWN intent - requesting clarification");

        let suggestions = if classification.alternative_types.is_empty() {
            "- Create an app\n- Add a task\n- Set up monitoring\n- Schedule automation".to_string()
        } else {
            classification
                .alternative_types
                .iter()
                .map(|a| format!("- {}: {}", a.intent_type, a.reason))
                .collect::<Vec<_>>()
                .join("\n")
        };

        Ok(IntentResult {
            success: false,
            intent_type: IntentType::Unknown,
            message: format!(
                "I'm not sure what you'd like me to do. Could you clarify?\n\nPossible interpretations:\n{}",
                suggestions
            ),
            created_resources: Vec::new(),
            app_url: None,
            task_id: None,
            schedule_id: None,
            tool_triggers: Vec::new(),
            next_steps: vec!["Provide more details about what you want".to_string()],
            error: None,
        })
    }

    // =========================================================================
    // HELPER METHODS
    // =========================================================================

    /// Call LLM for classification
    async fn call_llm(
        &self,
        prompt: &str,
        bot_id: Uuid,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        trace!("Calling LLM for intent classification");

        #[cfg(feature = "llm")]
        {
            // Get model and key from bot configuration
            let config_manager = ConfigManager::new(self.state.conn.clone());
            let model = config_manager
                .get_config(&bot_id, "llm-model", None)
                .unwrap_or_else(|_| {
                    config_manager
                        .get_config(&Uuid::nil(), "llm-model", None)
                        .unwrap_or_else(|_| "gpt-4".to_string())
                });
            let key = config_manager
                .get_config(&bot_id, "llm-key", None)
                .unwrap_or_else(|_| {
                    config_manager
                        .get_config(&Uuid::nil(), "llm-key", None)
                        .unwrap_or_default()
                });

            let llm_config = serde_json::json!({
                "temperature": 0.3,
                "max_tokens": 1000
            });
            let response = self
                .state
                .llm_provider
                .generate(prompt, &llm_config, &model, &key)
                .await?;
            return Ok(response);
        }

        #[cfg(not(feature = "llm"))]
        {
            warn!("LLM feature not enabled, using heuristic classification");
            Ok("{}".to_string())
        }
    }

    /// Save a BASIC file to the bot's directory
    fn save_basic_file(
        &self,
        bot_id: Uuid,
        path: &str,
        content: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let site_path = self
            .state
            .config
            .as_ref()
            .map(|c| c.site_path.clone())
            .unwrap_or_else(|| "./botserver-stack/sites".to_string());

        let full_path = format!("{}/{}.gbai/{}", site_path, bot_id, path);

        // Create directory if needed
        if let Some(dir) = std::path::Path::new(&full_path).parent() {
            if !dir.exists() {
                std::fs::create_dir_all(dir)?;
            }
        }

        std::fs::write(&full_path, content)?;
        info!("Saved BASIC file: {full_path}");

        Ok(())
    }

    /// Store classification for analytics
    fn store_classification(
        &self,
        classification: &ClassifiedIntent,
        session: &UserSession,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut conn = self.state.conn.get()?;

        sql_query(
            "INSERT INTO intent_classifications
             (id, bot_id, session_id, original_text, intent_type, confidence, entities, created_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7, NOW())
             ON CONFLICT DO NOTHING",
        )
        .bind::<DieselUuid, _>(Uuid::parse_str(&classification.id)?)
        .bind::<DieselUuid, _>(session.bot_id)
        .bind::<DieselUuid, _>(session.id)
        .bind::<Text, _>(&classification.original_text)
        .bind::<Text, _>(&classification.intent_type.to_string())
        .bind::<diesel::sql_types::Float8, _>(classification.confidence)
        .bind::<Text, _>(serde_json::to_string(&classification.entities)?)
        .execute(&mut conn)
        .ok(); // Ignore errors - analytics shouldn't break the flow

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_intent_type_from_str() {
        assert_eq!(IntentType::from("APP_CREATE"), IntentType::AppCreate);
        assert_eq!(IntentType::from("app"), IntentType::AppCreate);
        assert_eq!(IntentType::from("TODO"), IntentType::Todo);
        assert_eq!(IntentType::from("reminder"), IntentType::Todo);
        assert_eq!(IntentType::from("MONITOR"), IntentType::Monitor);
        assert_eq!(IntentType::from("SCHEDULE"), IntentType::Schedule);
        assert_eq!(IntentType::from("daily"), IntentType::Schedule);
        assert_eq!(IntentType::from("unknown_value"), IntentType::Unknown);
    }

    #[test]
    fn test_intent_type_display() {
        assert_eq!(IntentType::AppCreate.to_string(), "APP_CREATE");
        assert_eq!(IntentType::Todo.to_string(), "TODO");
        assert_eq!(IntentType::Monitor.to_string(), "MONITOR");
    }
}
