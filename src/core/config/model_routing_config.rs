use diesel::prelude::*;
use log::{debug, warn};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::core::shared::utils::DbPool;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum RoutingStrategy {
    #[default]
    Default,

    TaskBased,

    RoundRobin,

    Latency,

    Cost,

    Custom,
}

impl From<&str> for RoutingStrategy {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "task-based" | "taskbased" | "task" => Self::TaskBased,
            "round-robin" | "roundrobin" | "robin" => Self::RoundRobin,
            "latency" | "fast" | "speed" => Self::Latency,
            "cost" | "cheap" | "economy" => Self::Cost,
            "custom" => Self::Custom,
            _ => Self::Default,
        }
    }
}

impl std::fmt::Display for RoutingStrategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Default => write!(f, "default"),
            Self::TaskBased => write!(f, "task-based"),
            Self::RoundRobin => write!(f, "round-robin"),
            Self::Latency => write!(f, "latency"),
            Self::Cost => write!(f, "cost"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ModelRoutingConfig {
    pub routing_strategy: RoutingStrategy,

    pub default_model: String,

    pub fast_model: Option<String>,

    pub quality_model: Option<String>,

    pub code_model: Option<String>,

    pub fallback_enabled: bool,

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

    pub fn get_model_for_task(&self, task_type: TaskType) -> &str {
        match self.routing_strategy {
            RoutingStrategy::Default => &self.default_model,
            RoutingStrategy::TaskBased => match task_type {
                TaskType::Simple => self.fast_model.as_deref().unwrap_or(&self.default_model),
                TaskType::Complex => self.quality_model.as_deref().unwrap_or(&self.default_model),
                TaskType::Code => self.code_model.as_deref().unwrap_or(&self.default_model),
                TaskType::Default => &self.default_model,
            },
            RoutingStrategy::Latency
            | RoutingStrategy::Cost
            | RoutingStrategy::RoundRobin
            | RoutingStrategy::Custom => self.fast_model.as_deref().unwrap_or(&self.default_model),
        }
    }

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskType {
    Simple,

    Complex,

    Code,

    Default,
}
