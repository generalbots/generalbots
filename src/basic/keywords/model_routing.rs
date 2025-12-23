use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use diesel::prelude::*;
use log::{info, trace};
use rhai::{Dynamic, Engine};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Model routing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub name: String,
    pub url: String,
    pub model_path: String,
    pub api_key: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

/// Routing strategy for automatic model selection
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RoutingStrategy {
    Manual,       // User explicitly specifies model
    Auto,         // Route based on query analysis
    LoadBalanced, // Distribute across available models
    Fallback,     // Try models in order until success
}

impl Default for RoutingStrategy {
    fn default() -> Self {
        RoutingStrategy::Manual
    }
}

/// Model router for managing multiple LLM configurations
#[derive(Debug, Clone)]
pub struct ModelRouter {
    pub models: HashMap<String, ModelConfig>,
    pub default_model: String,
    pub routing_strategy: RoutingStrategy,
}

impl ModelRouter {
    pub fn new() -> Self {
        Self {
            models: HashMap::new(),
            default_model: "default".to_string(),
            routing_strategy: RoutingStrategy::Manual,
        }
    }

    /// Load models from config.csv llm-models property
    /// Format: llm-models,default;fast;quality;code
    pub fn from_config(config_models: &str, bot_id: Uuid, state: &AppState) -> Self {
        let mut router = Self::new();

        let model_names: Vec<&str> = config_models.split(';').collect();

        for name in model_names {
            let name = name.trim();
            if name.is_empty() {
                continue;
            }

            // Try to load model config from bot configuration
            // Looking for: llm-model-{name}, llm-url-{name}, llm-key-{name}
            if let Ok(mut conn) = state.conn.get() {
                let model_config = load_model_config(&mut conn, bot_id, name);
                if let Some(config) = model_config {
                    router.models.insert(name.to_string(), config);
                }
            }
        }

        // Set first model as default if available
        if let Some(first_name) = config_models.split(';').next() {
            router.default_model = first_name.trim().to_string();
        }

        router
    }

    /// Get model configuration by name
    pub fn get_model(&self, name: &str) -> Option<&ModelConfig> {
        self.models.get(name)
    }

    /// Get the default model configuration
    pub fn get_default(&self) -> Option<&ModelConfig> {
        self.models.get(&self.default_model)
    }

    /// Route a query to the optimal model based on strategy
    pub fn route_query(&self, query: &str) -> &str {
        match self.routing_strategy {
            RoutingStrategy::Auto => self.auto_route(query),
            RoutingStrategy::LoadBalanced => self.load_balanced_route(),
            RoutingStrategy::Fallback => &self.default_model,
            RoutingStrategy::Manual => &self.default_model,
        }
    }

    /// Automatic routing based on query analysis
    fn auto_route(&self, query: &str) -> &str {
        let query_lower = query.to_lowercase();

        // Code-related queries
        if query_lower.contains("code")
            || query_lower.contains("program")
            || query_lower.contains("function")
            || query_lower.contains("debug")
            || query_lower.contains("error")
            || query_lower.contains("syntax")
        {
            if self.models.contains_key("code") {
                return "code";
            }
        }

        // Complex reasoning queries
        if query_lower.contains("analyze")
            || query_lower.contains("explain")
            || query_lower.contains("compare")
            || query_lower.contains("evaluate")
            || query.len() > 500
        {
            if self.models.contains_key("quality") {
                return "quality";
            }
        }

        // Simple/fast queries
        if query.len() < 100
            || query_lower.contains("what is")
            || query_lower.contains("define")
            || query_lower.contains("hello")
        {
            if self.models.contains_key("fast") {
                return "fast";
            }
        }

        &self.default_model
    }

    /// Simple round-robin load balancing
    fn load_balanced_route(&self) -> &str {
        // For simplicity, return default - a full implementation would track usage
        &self.default_model
    }
}

/// Load model configuration from database
fn load_model_config(
    conn: &mut diesel::PgConnection,
    bot_id: Uuid,
    model_name: &str,
) -> Option<ModelConfig> {
    #[derive(QueryableByName)]
    struct ConfigRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        config_key: String,
        #[diesel(sql_type = diesel::sql_types::Text)]
        config_value: String,
    }

    // Query for model-specific config
    let suffix = if model_name == "default" {
        "".to_string()
    } else {
        format!("-{}", model_name)
    };

    let model_key = format!("llm-model{}", suffix);
    let url_key = format!("llm-url{}", suffix);
    let key_key = format!("llm-key{}", suffix);

    let configs: Vec<ConfigRow> = diesel::sql_query(
        "SELECT config_key, config_value FROM bot_configuration \
         WHERE bot_id = $1 AND config_key IN ($2, $3, $4)",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .bind::<diesel::sql_types::Text, _>(&model_key)
    .bind::<diesel::sql_types::Text, _>(&url_key)
    .bind::<diesel::sql_types::Text, _>(&key_key)
    .load(conn)
    .ok()?;

    let mut model_path = String::new();
    let mut url = String::new();
    let mut api_key = None;

    for config in configs {
        if config.config_key == model_key {
            model_path = config.config_value;
        } else if config.config_key == url_key {
            url = config.config_value;
        } else if config.config_key == key_key && config.config_value != "none" {
            api_key = Some(config.config_value);
        }
    }

    if model_path.is_empty() && url.is_empty() {
        return None;
    }

    Some(ModelConfig {
        name: model_name.to_string(),
        url,
        model_path,
        api_key,
        max_tokens: None,
        temperature: None,
    })
}

/// Registers model routing keywords
pub fn register_model_routing_keywords(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) {
    use_model_keyword(state.clone(), user.clone(), engine);
    set_model_routing_keyword(state.clone(), user.clone(), engine);
    get_current_model_keyword(state.clone(), user.clone(), engine);
    list_models_keyword(state.clone(), user.clone(), engine);
}

/// USE MODEL "model_name"
/// Switch the LLM model for the current conversation
pub fn use_model_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["USE", "MODEL", "$expr$"],
            false,
            move |context, inputs| {
                let model_name = context
                    .eval_expression_tree(&inputs[0])?
                    .to_string()
                    .trim_matches('"')
                    .to_string();

                trace!("USE MODEL '{}' for session: {}", model_name, user_clone.id);

                let state_for_task = Arc::clone(&state_clone);
                let session_id = user_clone.id;
                let model_name_clone = model_name.clone();

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                    let result = rt.block_on(async {
                        set_session_model(&state_for_task, session_id, &model_name_clone).await
                    });
                    let _ = tx.send(result);
                });

                match rx.recv_timeout(std::time::Duration::from_secs(10)) {
                    Ok(Ok(msg)) => Ok(Dynamic::from(msg)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        e.into(),
                        rhai::Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "USE MODEL timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("Failed to register USE MODEL syntax");
}

/// SET MODEL ROUTING "strategy"
/// Set the model routing strategy: "manual", "auto", "load-balanced", "fallback"
pub fn set_model_routing_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(
            &["SET", "MODEL", "ROUTING", "$expr$"],
            false,
            move |context, inputs| {
                let strategy_str = context
                    .eval_expression_tree(&inputs[0])?
                    .to_string()
                    .trim_matches('"')
                    .to_lowercase();

                let strategy = match strategy_str.as_str() {
                    "auto" => RoutingStrategy::Auto,
                    "load-balanced" | "loadbalanced" => RoutingStrategy::LoadBalanced,
                    "fallback" => RoutingStrategy::Fallback,
                    _ => RoutingStrategy::Manual,
                };

                trace!(
                    "SET MODEL ROUTING {:?} for session: {}",
                    strategy,
                    user_clone.id
                );

                let state_for_task = Arc::clone(&state_clone);
                let session_id = user_clone.id;

                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().expect("Failed to create runtime");
                    let result = rt.block_on(async {
                        set_session_routing_strategy(&state_for_task, session_id, strategy).await
                    });
                    let _ = tx.send(result);
                });

                match rx.recv_timeout(std::time::Duration::from_secs(10)) {
                    Ok(Ok(msg)) => Ok(Dynamic::from(msg)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        e.into(),
                        rhai::Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "SET MODEL ROUTING timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("Failed to register SET MODEL ROUTING syntax");
}

/// GET CURRENT MODEL()
/// Returns the name of the currently active model for this session
pub fn get_current_model_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine.register_fn("GET CURRENT MODEL", move || -> String {
        let state = Arc::clone(&state_clone);

        if let Ok(mut conn) = state.conn.get() {
            get_session_model_sync(&mut conn, user_clone.id)
                .unwrap_or_else(|_| "default".to_string())
        } else {
            "default".to_string()
        }
    });
}

/// LIST MODELS()
/// Returns an array of available model names
pub fn list_models_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine.register_fn("LIST MODELS", move || -> rhai::Array {
        let state = Arc::clone(&state_clone);

        if let Ok(mut conn) = state.conn.get() {
            list_available_models_sync(&mut conn, user_clone.bot_id)
                .unwrap_or_default()
                .into_iter()
                .map(Dynamic::from)
                .collect()
        } else {
            rhai::Array::new()
        }
    });
}

// Database Operations

/// Set the model for a session
async fn set_session_model(
    state: &AppState,
    session_id: Uuid,
    model_name: &str,
) -> Result<String, String> {
    let mut conn = state
        .conn
        .get()
        .map_err(|e| format!("Failed to acquire database connection: {}", e))?;

    let now = chrono::Utc::now();

    // Update or insert session model preference
    diesel::sql_query(
        "INSERT INTO session_preferences (session_id, preference_key, preference_value, updated_at) \
         VALUES ($1, 'current_model', $2, $3) \
         ON CONFLICT (session_id, preference_key) DO UPDATE SET \
         preference_value = EXCLUDED.preference_value, \
         updated_at = EXCLUDED.updated_at",
    )
    .bind::<diesel::sql_types::Uuid, _>(session_id)
    .bind::<diesel::sql_types::Text, _>(model_name)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(|e| format!("Failed to set session model: {}", e))?;

    info!("Session {} now using model: {}", session_id, model_name);

    Ok(format!("Now using model: {}", model_name))
}

/// Set the routing strategy for a session
async fn set_session_routing_strategy(
    state: &AppState,
    session_id: Uuid,
    strategy: RoutingStrategy,
) -> Result<String, String> {
    let mut conn = state
        .conn
        .get()
        .map_err(|e| format!("Failed to acquire database connection: {}", e))?;

    let now = chrono::Utc::now();
    let strategy_str = match strategy {
        RoutingStrategy::Manual => "manual",
        RoutingStrategy::Auto => "auto",
        RoutingStrategy::LoadBalanced => "load-balanced",
        RoutingStrategy::Fallback => "fallback",
    };

    diesel::sql_query(
        "INSERT INTO session_preferences (session_id, preference_key, preference_value, updated_at) \
         VALUES ($1, 'model_routing', $2, $3) \
         ON CONFLICT (session_id, preference_key) DO UPDATE SET \
         preference_value = EXCLUDED.preference_value, \
         updated_at = EXCLUDED.updated_at",
    )
    .bind::<diesel::sql_types::Uuid, _>(session_id)
    .bind::<diesel::sql_types::Text, _>(strategy_str)
    .bind::<diesel::sql_types::Timestamptz, _>(now)
    .execute(&mut conn)
    .map_err(|e| format!("Failed to set routing strategy: {}", e))?;

    info!(
        "Session {} routing strategy set to: {}",
        session_id, strategy_str
    );

    Ok(format!("Model routing set to: {}", strategy_str))
}

/// Get the current model for a session (sync version)
fn get_session_model_sync(
    conn: &mut diesel::PgConnection,
    session_id: Uuid,
) -> Result<String, String> {
    #[derive(QueryableByName)]
    struct PrefValue {
        #[diesel(sql_type = diesel::sql_types::Text)]
        preference_value: String,
    }

    let result: Option<PrefValue> = diesel::sql_query(
        "SELECT preference_value FROM session_preferences \
         WHERE session_id = $1 AND preference_key = 'current_model' LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(session_id)
    .get_result(conn)
    .optional()
    .map_err(|e| format!("Failed to get session model: {}", e))?;

    Ok(result
        .map(|r| r.preference_value)
        .unwrap_or_else(|| "default".to_string()))
}

/// List available models for a bot (sync version)
fn list_available_models_sync(
    conn: &mut diesel::PgConnection,
    bot_id: Uuid,
) -> Result<Vec<String>, String> {
    #[derive(QueryableByName)]
    struct ConfigRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        config_value: String,
    }

    // Get llm-models config
    let result: Option<ConfigRow> = diesel::sql_query(
        "SELECT config_value FROM bot_configuration \
         WHERE bot_id = $1 AND config_key = 'llm-models' LIMIT 1",
    )
    .bind::<diesel::sql_types::Uuid, _>(bot_id)
    .get_result(conn)
    .optional()
    .map_err(|e| format!("Failed to list models: {}", e))?;

    if let Some(config) = result {
        Ok(config
            .config_value
            .split(';')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect())
    } else {
        // Return default model if no models configured
        Ok(vec!["default".to_string()])
    }
}

/// Get the current model name for a session (public helper)
pub fn get_session_model(state: &AppState, session_id: Uuid) -> String {
    if let Ok(mut conn) = state.conn.get() {
        get_session_model_sync(&mut conn, session_id).unwrap_or_else(|_| "default".to_string())
    } else {
        "default".to_string()
    }
}

/// Get the routing strategy for a session
pub fn get_session_routing_strategy(state: &AppState, session_id: Uuid) -> RoutingStrategy {
    if let Ok(mut conn) = state.conn.get() {
        #[derive(QueryableByName)]
        struct PrefValue {
            #[diesel(sql_type = diesel::sql_types::Text)]
            preference_value: String,
        }

        let result: Option<PrefValue> = diesel::sql_query(
            "SELECT preference_value FROM session_preferences \
             WHERE session_id = $1 AND preference_key = 'model_routing' LIMIT 1",
        )
        .bind::<diesel::sql_types::Uuid, _>(session_id)
        .get_result(&mut conn)
        .optional()
        .ok()
        .flatten();

        if let Some(pref) = result {
            match pref.preference_value.as_str() {
                "auto" => RoutingStrategy::Auto,
                "load-balanced" => RoutingStrategy::LoadBalanced,
                "fallback" => RoutingStrategy::Fallback,
                _ => RoutingStrategy::Manual,
            }
        } else {
            RoutingStrategy::Manual
        }
    } else {
        RoutingStrategy::Manual
    }
}

// Tests
