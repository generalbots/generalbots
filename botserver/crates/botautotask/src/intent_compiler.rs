use crate::types::{AutoTaskState, ConfigOps, LlmProviderOps, UserSession};
use crate::task_types::{ExecutionMode, TaskPriority};
use crate::intent_classifier::{ClassifiedIntent, IntentType};
use crate::safety_layer::{SafetyLayer, SimulationResult};
use log::{info, warn};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledIntent {
    pub id: String,
    pub intent_type: IntentType,
    pub plan_name: String,
    pub plan_description: String,
    pub steps: Vec<PlanStep>,
    pub alternatives: Vec<Alternative>,
    pub confidence: f64,
    pub risk_level: String,
    pub estimated_duration_minutes: i32,
    pub estimated_cost: f64,
    pub resource_estimate: ResourceEstimate,
    pub basic_program: Option<String>,
    pub requires_approval: bool,
    pub mcp_servers: Vec<String>,
    pub external_apis: Vec<String>,
    pub risks: Vec<Risk>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanStep {
    pub id: String,
    pub order: i32,
    pub name: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub priority: String,
    pub risk_level: String,
    pub estimated_minutes: i32,
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alternative {
    pub id: String,
    pub description: String,
    pub confidence: f64,
    pub pros: Vec<String>,
    pub cons: Vec<String>,
    pub estimated_cost: Option<f64>,
    pub estimated_time_hours: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEstimate {
    pub compute_hours: f64,
    pub storage_gb: f64,
    pub api_calls: i32,
    pub llm_tokens: i32,
    pub estimated_cost_usd: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Risk {
    pub id: String,
    pub category: String,
    pub description: String,
    pub probability: f64,
    pub impact: String,
}

pub struct IntentCompiler {
    state: Arc<dyn AutoTaskState>,
    config_ops: Arc<dyn ConfigOps>,
    llm_ops: Arc<dyn LlmProviderOps>,
    safety: SafetyLayer,
}

impl IntentCompiler {
    pub fn new(
        state: Arc<dyn AutoTaskState>,
        config_ops: Arc<dyn ConfigOps>,
        llm_ops: Arc<dyn LlmProviderOps>,
    ) -> Self {
        let safety = SafetyLayer::new(state.db_pool().clone());
        Self { state, config_ops, llm_ops, safety }
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

    pub async fn compile(
        &self,
        intent: &str,
        execution_mode: Option<ExecutionMode>,
        priority: Option<TaskPriority>,
    ) -> Result<CompiledIntent, Box<dyn std::error::Error + Send + Sync>> {
        info!("Compiling intent: {}", &intent[..intent.len().min(100)]);
        let prompt = self.build_compile_prompt(intent);
        let response = self.call_llm(&prompt).await?;
        self.parse_compile_response(&response, intent, execution_mode, priority)
    }

    pub async fn compile_from_classification(
        &self,
        classification: &ClassifiedIntent,
        execution_mode: Option<ExecutionMode>,
        priority: Option<TaskPriority>,
    ) -> Result<CompiledIntent, Box<dyn std::error::Error + Send + Sync>> {
        let intent_text = &classification.original_text;
        self.compile(intent_text, execution_mode, priority).await
    }

    pub async fn simulate(
        &self,
        compiled: &CompiledIntent,
        session: &UserSession,
    ) -> Result<SimulationResult, Box<dyn std::error::Error + Send + Sync>> {
        self.safety.simulate_execution(&compiled.id, session)
    }

    fn build_compile_prompt(&self, intent: &str) -> String {
        format!(
            r#"You are an intent compiler. Analyze this request and create an execution plan.

USER REQUEST: "{intent}"

Create a detailed plan with steps, resource estimates, and risk assessment.
Respond with JSON only:
{{
  "plan_name": "short name",
  "plan_description": "what will be done",
  "steps": [
    {{
      "name": "step name",
      "description": "what this step does",
      "keywords": ["keyword1"],
      "priority": "high|medium|low",
      "risk_level": "high|medium|low",
      "estimated_minutes": 5,
      "requires_approval": false
    }}
  ],
  "alternatives": [],
  "confidence": 0.85,
  "risk_level": "low",
  "estimated_duration_minutes": 10,
  "estimated_cost": 0.01,
  "resource_estimate": {{
    "compute_hours": 0.1,
    "storage_gb": 0.01,
    "api_calls": 5,
    "llm_tokens": 1000,
    "estimated_cost_usd": 0.01
  }},
  "basic_program": null,
  "requires_approval": false,
  "mcp_servers": [],
  "external_apis": [],
  "risks": []
}}"#
        )
    }

    #[cfg(feature = "llm")]
    async fn call_llm(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(100);
        let model = "gpt-4".to_string();
        let key = "".to_string();
        let config = serde_json::json!({"temperature": 0.3, "max_tokens": 2000});
        self.llm_ops.generate_stream(prompt, &config, tx, &model, &key, None).await?;
        let mut response = String::new();
        while let Some(chunk) = rx.recv().await {
            response.push_str(&chunk);
        }
        Ok(response)
    }

    #[cfg(not(feature = "llm"))]
    async fn call_llm(&self, _prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        warn!("LLM feature not enabled for intent compilation");
        Ok("{}".to_string())
    }

    fn parse_compile_response(
        &self,
        response: &str,
        original_intent: &str,
        execution_mode: Option<ExecutionMode>,
        priority: Option<TaskPriority>,
    ) -> Result<CompiledIntent, Box<dyn std::error::Error + Send + Sync>> {
        let cleaned = response.trim()
            .trim_start_matches("```json").trim_start_matches("```JSON")
            .trim_start_matches("```").trim_end_matches("```").trim();

        #[derive(Deserialize)]
        struct CompileResponse {
            plan_name: String,
            plan_description: String,
            steps: Vec<StepResponse>,
            alternatives: Vec<AltResponse>,
            confidence: f64,
            risk_level: String,
            estimated_duration_minutes: i32,
            estimated_cost: f64,
            resource_estimate: ResourceEstResponse,
            basic_program: Option<String>,
            requires_approval: bool,
            mcp_servers: Vec<String>,
            external_apis: Vec<String>,
            risks: Vec<RiskResponse>,
        }
        #[derive(Deserialize)]
        struct StepResponse {
            name: String,
            description: String,
            keywords: Option<Vec<String>>,
            priority: Option<String>,
            risk_level: Option<String>,
            estimated_minutes: Option<i32>,
            requires_approval: Option<bool>,
        }
        #[derive(Deserialize)]
        struct AltResponse {
            description: String,
            confidence: f64,
            pros: Option<Vec<String>>,
            cons: Option<Vec<String>>,
            estimated_cost: Option<f64>,
            estimated_time_hours: Option<f64>,
        }
        #[derive(Deserialize)]
        struct ResourceEstResponse {
            compute_hours: f64,
            storage_gb: f64,
            api_calls: i32,
            llm_tokens: i32,
            estimated_cost_usd: f64,
        }
        #[derive(Deserialize)]
        struct RiskResponse {
            category: String,
            description: String,
            probability: f64,
            impact: String,
        }

        match serde_json::from_str::<CompileResponse>(cleaned) {
            Ok(resp) => {
                let _ = (execution_mode, priority);
                let steps = resp.steps.into_iter().enumerate().map(|(i, s)| PlanStep {
                    id: Uuid::new_v4().to_string(),
                    order: i as i32 + 1,
                    name: s.name,
                    description: s.description,
                    keywords: s.keywords.unwrap_or_default(),
                    priority: s.priority.unwrap_or_else(|| "medium".to_string()),
                    risk_level: s.risk_level.unwrap_or_else(|| "low".to_string()),
                    estimated_minutes: s.estimated_minutes.unwrap_or(5),
                    requires_approval: s.requires_approval.unwrap_or(false),
                }).collect();
                let alternatives = resp.alternatives.into_iter().map(|a| Alternative {
                    id: Uuid::new_v4().to_string(),
                    description: a.description,
                    confidence: a.confidence,
                    pros: a.pros.unwrap_or_default(),
                    cons: a.cons.unwrap_or_default(),
                    estimated_cost: a.estimated_cost,
                    estimated_time_hours: a.estimated_time_hours,
                }).collect();
                let risks = resp.risks.into_iter().map(|r| Risk {
                    id: Uuid::new_v4().to_string(),
                    category: r.category,
                    description: r.description,
                    probability: r.probability,
                    impact: r.impact,
                }).collect();
                Ok(CompiledIntent {
                    id: Uuid::new_v4().to_string(),
                    intent_type: IntentType::Unknown,
                    plan_name: resp.plan_name,
                    plan_description: resp.plan_description,
                    steps,
                    alternatives,
                    confidence: resp.confidence,
                    risk_level: resp.risk_level,
                    estimated_duration_minutes: resp.estimated_duration_minutes,
                    estimated_cost: resp.estimated_cost,
                    resource_estimate: ResourceEstimate {
                        compute_hours: resp.resource_estimate.compute_hours,
                        storage_gb: resp.resource_estimate.storage_gb,
                        api_calls: resp.resource_estimate.api_calls,
                        llm_tokens: resp.resource_estimate.llm_tokens,
                        estimated_cost_usd: resp.resource_estimate.estimated_cost_usd,
                    },
                    basic_program: resp.basic_program,
                    requires_approval: resp.requires_approval,
                    mcp_servers: resp.mcp_servers,
                    external_apis: resp.external_apis,
                    risks,
                })
            }
            Err(e) => {
                warn!("Failed to parse compile response, creating minimal plan: {e}");
                Ok(CompiledIntent {
                    id: Uuid::new_v4().to_string(),
                    intent_type: IntentType::Unknown,
                    plan_name: original_intent.chars().take(50).collect(),
                    plan_description: original_intent.to_string(),
                    steps: Vec::new(),
                    alternatives: Vec::new(),
                    confidence: 0.5,
                    risk_level: "medium".to_string(),
                    estimated_duration_minutes: 10,
                    estimated_cost: 0.0,
                    resource_estimate: ResourceEstimate {
                        compute_hours: 0.0, storage_gb: 0.0, api_calls: 0,
                        llm_tokens: 0, estimated_cost_usd: 0.0,
                    },
                    basic_program: None,
                    requires_approval: false,
                    mcp_servers: Vec::new(),
                    external_apis: Vec::new(),
                    risks: Vec::new(),
                })
            }
        }
    }
}
