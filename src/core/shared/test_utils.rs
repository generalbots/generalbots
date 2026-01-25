use crate::core::bot::channels::{ChannelAdapter, VoiceAdapter, WebChannelAdapter};
use crate::core::bot_database::BotDatabaseManager;
use crate::core::config::AppConfig;
use crate::core::session::SessionManager;
use crate::core::shared::analytics::MetricsCollector;
use crate::core::shared::state::{AppState, Extensions};
#[cfg(feature = "directory")]
use crate::directory::client::ZitadelConfig;
#[cfg(feature = "directory")]
use crate::directory::AuthService;
#[cfg(feature = "llm")]
use crate::llm::LLMProvider;
use crate::shared::models::BotResponse;
use crate::shared::utils::{get_database_url_sync, DbPool};
#[cfg(feature = "tasks")]
use crate::tasks::TaskEngine;
use async_trait::async_trait;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};

#[cfg(feature = "llm")]
#[derive(Debug)]
pub struct MockLLMProvider {
    pub response: String,
}

#[cfg(feature = "llm")]
impl MockLLMProvider {
    pub fn new() -> Self {
        Self {
            response: "Mock LLM response".to_string(),
        }
    }

    pub fn with_response(response: &str) -> Self {
        Self {
            response: response.to_string(),
        }
    }
}

#[cfg(feature = "llm")]
impl Default for MockLLMProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "llm")]
#[async_trait]
impl LLMProvider for MockLLMProvider {
    async fn generate(
        &self,
        _prompt: &str,
        _config: &Value,
        _model: &str,
        _key: &str,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        Ok(self.response.clone())
    }

    async fn generate_stream(
        &self,
        _prompt: &str,
        _config: &Value,
        tx: mpsc::Sender<String>,
        _model: &str,
        _key: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        tx.send(self.response.clone()).await?;
        Ok(())
    }

    async fn cancel_job(
        &self,
        _session_id: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        Ok(())
    }
}

#[derive(Debug)]
pub struct MockChannelAdapter {
    pub name: String,
    pub messages: Arc<Mutex<Vec<BotResponse>>>,
}

impl MockChannelAdapter {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn get_sent_messages(&self) -> Vec<BotResponse> {
        self.messages.lock().await.clone()
    }
}

#[async_trait]
impl ChannelAdapter for MockChannelAdapter {
    fn name(&self) -> &'static str {
        "Mock"
    }

    fn is_configured(&self) -> bool {
        true
    }

    async fn send_message(
        &self,
        response: BotResponse,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        self.messages.lock().await.push(response);
        Ok(())
    }

    async fn receive_message(
        &self,
        _payload: Value,
    ) -> Result<Option<String>, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Some("mock_message".to_string()))
    }

    async fn get_user_info(
        &self,
        user_id: &str,
    ) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        Ok(serde_json::json!({
            "id": user_id,
            "platform": self.name,
            "name": "Mock User"
        }))
    }
}

#[derive(Debug)]
pub struct TestAppStateBuilder {
    database_url: Option<String>,
    bucket_name: String,
    config: Option<AppConfig>,
}

impl TestAppStateBuilder {
    pub fn new() -> Self {
        Self {
            database_url: None,
            bucket_name: "test-bucket".to_string(),
            config: None,
        }
    }

    pub fn with_database_url(mut self, url: &str) -> Self {
        self.database_url = Some(url.to_string());
        self
    }

    pub fn with_bucket_name(mut self, name: &str) -> Self {
        self.bucket_name = name.to_string();
        self
    }

    pub fn with_config(mut self, config: AppConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn build(self) -> Result<AppState, Box<dyn std::error::Error + Send + Sync>> {
        let database_url = self
            .database_url
            .or_else(|| get_database_url_sync().ok())
            .unwrap_or_else(|| "postgres://test:test@localhost:5432/test".to_string());

        let manager = ConnectionManager::<PgConnection>::new(&database_url);
        let pool = Pool::builder()
            .max_size(1)
            .test_on_check_out(false)
            .connection_timeout(std::time::Duration::from_secs(5))
            .build(manager)?;

        let conn = pool.get()?;
        let session_manager = SessionManager::new(conn, None);

        let (attendant_tx, _) = broadcast::channel(100);

        let (task_progress_tx, _) = broadcast::channel(100);

        let bot_database_manager = Arc::new(BotDatabaseManager::new(pool.clone(), &database_url));

        Ok(AppState {
            #[cfg(feature = "drive")]
            drive: None,
            #[cfg(feature = "drive")]
            s3_client: None,
            #[cfg(feature = "cache")]
            cache: None,
            bucket_name: self.bucket_name,
            config: self.config,
            conn: pool.clone(),
            database_url,
            bot_database_manager,
            session_manager: Arc::new(tokio::sync::Mutex::new(session_manager)),
            metrics_collector: MetricsCollector::new(),
            #[cfg(feature = "tasks")]
            task_scheduler: None,
            #[cfg(feature = "llm")]
            llm_provider: Arc::new(MockLLMProvider::new()),
            #[cfg(feature = "directory")]
            auth_service: Arc::new(tokio::sync::Mutex::new(create_mock_auth_service())),
            channels: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            response_channels: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            web_adapter: Arc::new(WebChannelAdapter::new()),
            voice_adapter: Arc::new(VoiceAdapter::new()),
            #[cfg(any(feature = "research", feature = "llm"))]
            kb_manager: None,
            #[cfg(feature = "tasks")]
            task_engine: Arc::new(TaskEngine::new(pool)),
            extensions: Extensions::new(),
            attendant_broadcast: Some(attendant_tx),
            task_progress_broadcast: Some(task_progress_tx),
            billing_alert_broadcast: None,
            task_manifests: Arc::new(std::sync::RwLock::new(HashMap::new())),
            #[cfg(feature = "project")]
            project_service: Arc::new(tokio::sync::RwLock::new(crate::project::ProjectService::new())),
            #[cfg(feature = "compliance")]
            legal_service: Arc::new(tokio::sync::RwLock::new(crate::legal::LegalService::new())),
            jwt_manager: None,
            auth_provider_registry: None,
            rbac_manager: None,
        })
    }
}

impl Default for TestAppStateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "directory")]
pub fn create_mock_auth_service() -> AuthService {
    let config = ZitadelConfig {
        issuer_url: "http://localhost:8080".to_string(),
        issuer: "http://localhost:8080".to_string(),
        client_id: "mock_client_id".to_string(),
        client_secret: "mock_client_secret".to_string(),
        redirect_uri: "http://localhost:3000/callback".to_string(),
        project_id: "mock_project_id".to_string(),
        api_url: "http://localhost:8080".to_string(),
        service_account_key: None,
    };

    AuthService::new(config).expect("Failed to create mock AuthService")
}

pub fn create_test_db_pool() -> Result<DbPool, Box<dyn std::error::Error + Send + Sync>> {
    let database_url = get_database_url_sync()
        .unwrap_or_else(|_| "postgres://test:test@localhost:5432/test".to_string());
    let manager = ConnectionManager::<PgConnection>::new(&database_url);
    let pool = Pool::builder()
        .max_size(1)
        .connection_timeout(std::time::Duration::from_secs(5))
        .build(manager)?;
    Ok(pool)
}

pub fn create_mock_metrics_collector() -> MetricsCollector {
    MetricsCollector::new()
}
