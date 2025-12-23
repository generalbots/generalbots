//! Model Routing Configuration
//! Parameters: model-routing-strategy, model-default, model-fast, model-quality, model-code, model-fallback-enabled, model-fallback-order
//!
//! Config.csv properties:
//! ```csv
//! model-routing-strategy,default
//! model-default,gpt-4o
//! model-fast,gpt-4o-mini
//! model-quality,gpt-4o
//! model-code,gpt-4o
//! model-fallback-enabled,true
//! model-fallback-order,gpt-4o,gpt-4o-mini,gpt-3.5-turbo
//! ```

use diesel::prelude::*;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::shared::utils::DbPool;

/// Routing strategy for model selection
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum RoutingStrategy {
    /// Always use the default model
    #[default]
    Default,
    /// Select model based on task complexity
    TaskBased,
    /// Round-robin across available models
    RoundRobin,
    /// Use fastest model for the task
    Latency,
    /// Use cheapest model that meets requirements
    Cost,
    /// Custom routing logic
    Custom,
}

impl From<&str> for RoutingStrategy {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "task-based" | "taskbased" | "task" => RoutingStrategy::TaskBased,
            "round-robin" | "roundrobin" | "robin" => RoutingStrategy::RoundRobin,
            "latency" | "fast" | "speed" => RoutingStrategy::Latency,
            "cost" | "cheap" | "economy" => RoutingStrategy::Cost,
            "custom" => RoutingStrategy::Custom,
            _ => RoutingStrategy::Default,
        }
    }
}

impl std::fmt::Display for RoutingStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoutingStrategy::Default => write!(f, "default"),
            RoutingStrategy::TaskBased => write!(f, "task-based"),
            RoutingStrategy::RoundRobin => write!(f, "round-robin"),
            RoutingStrategy::Latency => write!(f, "latency"),
            RoutingStrategy::Cost => write!(f, "cost"),
            RoutingStrategy::Custom => write!(f, "custom"),
        }
    }
}

/// Configuration for Model Routing
///
/// Model routing allows bots to intelligently select the appropriate LLM
/// based on task requirements, cost constraints, or custom logic.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelRoutingConfig {
    /// Strategy for selecting models
    pub routing_strategy: RoutingStrategy,
    /// Default model to use when no specific model is requested
    pub default_model: String,
    /// Model optimized for fast responses (simple tasks)
    pub fast_model: Option<String>,
    /// Model optimized for quality responses (complex tasks)
    pub quality_model: Option<String>,
    /// Model optimized for code generation tasks
    pub code_model: Option<String>,
    /// Whether fallback to alternative models is enabled
    pub fallback_enabled: bool,
    /// Ordered list of models to try if primary model fails
    pub fallback_order: Vec<String>,
}

impl Default for ModelRoutingConfig {
    fn default() -> Self {
        Self {
            routing_strategy: RoutingStrategy::Default,
            default_model: "gpt-4o".to_string(),
            fast_model: Some("gpt-4o-mini".to_string()),
            quality_model: Some("gpt-4o".to_string()),
            code_model: Some("gpt-4o".to_string()),
            fallback_enabled: true,
            fallback_order: vec![
                "gpt-4o".to_string(),
                "gpt-4o-mini".to_string(),
                "gpt-3.5-turbo".to_string(),
            ],
        }
    }
}

impl ModelRoutingConfig {
    /// Load Model Routing configuration from bot_configuration table
    ///
    /// Reads the following parameters:
    /// - `model-routing-strategy`: Routing strategy (default: "default")
    /// - `model-default`: Default model name (default: "gpt-4o")
    /// - `model-fast`: Fast/lightweight model (default: "gpt-4o-mini")
    /// - `model-quality`: High-quality model (default: "gpt-4o")
    /// - `model-code`: Code generation model (default: "gpt-4o")
    /// - `model-fallback-enabled`: Enable fallback (default: true)
    /// - `model-fallback-order`: Comma-separated fallback models
    /// Reads parameters: `model-routing-strategy`, `model-default`, `model-fast`, `model-quality`, `model-code`, `model-fallback-enabled`, `model-fallback-order`
    pub fn from_bot_config(pool: &DbPool, target_bot_id: &Uuid) -> Self {
        let mut config = Self::default();

        let mut conn = match pool.get() {
            Ok(c) => c,
            Err(e) => {
                warn!(
                    "Failed to get database connection for Model Routing config: {}",
                    e
                );
                return config;
            }
        };

        #[derive(QueryableByName)]
        struct ConfigRow {
            #[diesel(sql_type = diesel::sql_types::Text)]
            config_key: String,
            #[diesel(sql_type = diesel::sql_types::Text)]
            config_value: String,
        }

        let configs: Vec<ConfigRow> = diesel::sql_query(
            "SELECT config_key, config_value FROM bot_configuration \
             WHERE bot_id = $1 AND config_key LIKE 'model-%'",
        )
        .bind::<diesel::sql_types::Uuid, _>(target_bot_id)
        .load(&mut conn)
        .unwrap_or_default();

        for row in configs {
            match row.config_key.as_str() {
                "model-routing-strategy" => {
                    config.routing_strategy = RoutingStrategy::from(row.config_value.as_str());
                    debug!("Model routing strategy: {}", config.routing_strategy);
                }
                "model-default" => {
                    if !row.config_value.is_empty() {
                        config.default_model = row.config_value;
                        debug!("Default model: {}", config.default_model);
                    }
                }
                "model-fast" => {
                    config.fast_model = if row.config_value.is_empty() {
                        None
                    } else {
                        Some(row.config_value)
                    };
                    debug!("Fast model: {:?}", config.fast_model);
                }
                "model-quality" => {
                    config.quality_model = if row.config_value.is_empty() {
                        None
                    } else {
                        Some(row.config_value)
                    };
                    debug!("Quality model: {:?}", config.quality_model);
                }
                "model-code" => {
                    config.code_model = if row.config_value.is_empty() {
                        None
                    } else {
                        Some(row.config_value)
                    };
                    debug!("Code model: {:?}", config.code_model);
                }
                "model-fallback-enabled" => {
                    config.fallback_enabled = row.config_value.to_lowercase() == "true";
                    debug!("Model fallback enabled: {}", config.fallback_enabled);
                }
                "model-fallback-order" => {
                    let models: Vec<String> = row
                        .config_value
                        .split(',')
                        .map(|s| s.trim().to_string())
                        .filter(|s| !s.is_empty())
                        .collect();
                    if !models.is_empty() {
                        config.fallback_order = models;
                    }
                    debug!("Model fallback order: {:?}", config.fallback_order);
                }
                _ => {}
            }
        }

        config
    }

    /// Get the appropriate model for a given task type
    pub fn get_model_for_task(&self, task_type: TaskType) -> &str {
        match self.routing_strategy {
            RoutingStrategy::Default => &self.default_model,
            RoutingStrategy::TaskBased => match task_type {
                TaskType::Simple => self.fast_model.as_deref().unwrap_or(&self.default_model),
                TaskType::Complex => self.quality_model.as_deref().unwrap_or(&self.default_model),
                TaskType::Code => self.code_model.as_deref().unwrap_or(&self.default_model),
                TaskType::Default => &self.default_model,
            },
            RoutingStrategy::Latency => self.fast_model.as_deref().unwrap_or(&self.default_model),
            RoutingStrategy::Cost => self.fast_model.as_deref().unwrap_or(&self.default_model),
            _ => &self.default_model,
        }
    }

    /// Get the next fallback model after the given model
    pub fn get_fallback_model(&self, current_model: &str) -> Option<&str> {
        if !self.fallback_enabled {
            return None;
        }

        let current_idx = self
            .fallback_order
            .iter()
            .position(|m| m == current_model)?;

        self.fallback_order.get(current_idx + 1).map(|s| s.as_str())
    }

    /// Get all available models in preference order
    pub fn get_all_models(&self) -> Vec<&str> {
        let mut models = vec![self.default_model.as_str()];

        if let Some(ref fast) = self.fast_model {
            if !models.contains(&fast.as_str()) {
                models.push(fast.as_str());
            }
        }

        if let Some(ref quality) = self.quality_model {
            if !models.contains(&quality.as_str()) {
                models.push(quality.as_str());
            }
        }

        if let Some(ref code) = self.code_model {
            if !models.contains(&code.as_str()) {
                models.push(code.as_str());
            }
        }

        for model in &self.fallback_order {
            if !models.contains(&model.as_str()) {
                models.push(model.as_str());
            }
        }

        models
    }
}

/// Task type for model selection
#[derive(Clone, Debug, PartialEq)]
pub enum TaskType {
    /// Simple conversational tasks
    Simple,
    /// Complex reasoning tasks
    Complex,
    /// Code generation/analysis tasks
    Code,
    /// Default/unknown task type
    Default,
}
