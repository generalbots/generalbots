// Multimodal keyword tests split from `multimodal.rs` to keep the
// implementation file under the AGENTS.md 450-line limit.

#![cfg(test)]

use super::*;
use crate::keywords::multimodal_helpers::RuntimeConfigProvider;
use botmultimodal::{BotModelsConfig, ConfigProvider, ImageGeneratorConfig, VideoGeneratorConfig};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Default)]
struct MapRuntime {
    cfg: HashMap<String, String>,
}

impl BasicRuntime for MapRuntime {
    fn db_pool(&self) -> &botlib::db_pool::DbPool {
        static EMPTY: std::sync::OnceLock<botlib::db_pool::DbPool> =
            std::sync::OnceLock::new();
        EMPTY.get_or_init(|| unreachable!("db_pool used in test without database feature"))
    }
    fn cache_client(&self) -> Option<std::sync::Arc<redis::Client>> {
        None
    }
    fn bucket_name(&self) -> &str {
        "test"
    }
    fn hear_channels(
        &self,
    ) -> &std::sync::Mutex<HashMap<Uuid, std::sync::mpsc::SyncSender<String>>> {
        static CHANNELS: std::sync::OnceLock<
            std::sync::Mutex<HashMap<Uuid, std::sync::mpsc::SyncSender<String>>>,
        > = std::sync::OnceLock::new();
        CHANNELS.get_or_init(|| std::sync::Mutex::new(HashMap::new()))
    }
    fn bot_database_manager(&self) -> std::sync::Arc<dyn botlib::traits::BotDatabaseService> {
        stub_db()
    }
    fn web_adapter(&self) -> std::sync::Arc<dyn botlib::traits::ChannelAdapter> {
        stub_adapter()
    }
    fn drive_repository(
        &self,
    ) -> Option<std::sync::Arc<dyn botlib::traits::DriveRepository>> {
        None
    }
    fn config_value(&self, key: &str) -> Option<String> {
        self.cfg.get(key).cloned()
    }
    fn session_manager(
        &self,
    ) -> std::sync::Arc<tokio::sync::Mutex<dyn botlib::traits::SessionManagerService>>
    {
        stub_session()
    }
    fn update_session_user(
        &self,
        _session_id: Uuid,
        _user_id: Uuid,
    ) -> Result<(), String> {
        Ok(())
    }
    fn send_message(&self, _response: &botlib::models::BotResponse) -> Result<(), String> {
        Ok(())
    }
    fn execute_script(
        &self,
        _user: UserSession,
        _script: &str,
    ) -> Result<String, String> {
        Ok(String::new())
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

fn stub_db() -> std::sync::Arc<dyn botlib::traits::BotDatabaseService> {
    #[derive(Debug)]
    struct StubDb;
    impl botlib::traits::BotDatabaseService for StubDb {
        fn get_bot_pool(&self, _bot_id: Uuid) -> Option<botlib::db_pool::DbPool> {
            None
        }
        fn create_table_in_bot_database(&self, _bot_id: Uuid, _sql: &str) -> Result<(), String> {
            Ok(())
        }
        fn sync_all_bot_databases(&self) -> Result<(), String> {
            Ok(())
        }
    }
    std::sync::Arc::new(StubDb)
}
fn stub_adapter() -> std::sync::Arc<dyn botlib::traits::ChannelAdapter> {
    #[derive(Debug)]
    struct StubAdapter;
    impl botlib::traits::ChannelAdapter for StubAdapter {
        fn channel_type(&self) -> &str {
            "test"
        }
        fn send_message(&self, _to: &str, _message: &str) -> Result<(), String> {
            Err("stub".into())
        }
    }
    std::sync::Arc::new(StubAdapter)
}
fn stub_session(
) -> std::sync::Arc<tokio::sync::Mutex<dyn botlib::traits::SessionManagerService>> {
    #[derive(Debug)]
    struct StubSession;
    impl botlib::traits::SessionManagerService for StubSession {
        fn get_session_by_id(
            &mut self,
            _session_id: Uuid,
        ) -> Result<Option<botlib::models::UserSession>, String> {
            Ok(None)
        }
        fn get_or_create_user_session(
            &mut self,
            _user_id: Uuid,
            _bot_id: Uuid,
            _session_title: &str,
        ) -> Result<Option<botlib::models::UserSession>, String> {
            Ok(None)
        }
        fn get_or_create_anonymous_user(
            &mut self,
            _user_id: Option<Uuid>,
        ) -> Result<Uuid, String> {
            Ok(Uuid::new_v4())
        }
        fn create_session(
            &mut self,
            _user_id: Uuid,
            _bot_id: Uuid,
            _title: &str,
        ) -> Result<botlib::models::UserSession, String> {
            Err("stub".into())
        }
        fn get_or_create_session_by_id(
            &mut self,
            _session_id: Uuid,
            _user_id: Uuid,
            _bot_id: Uuid,
            _session_title: &str,
        ) -> Result<botlib::models::UserSession, String> {
            Err("stub".into())
        }
        fn save_message(
            &mut self,
            _session_id: Uuid,
            _user_id: Uuid,
            _role: i32,
            _content: &str,
            _message_type: i32,
        ) -> Result<(), String> {
            Ok(())
        }
        fn get_conversation_history(
            &mut self,
            _session_id: Uuid,
            _user_id: Uuid,
            _limit: Option<i64>,
        ) -> Result<Vec<(String, String)>, String> {
            Ok(Vec::new())
        }
        fn get_first_user_message(&mut self, _session_id: Uuid) -> Result<Option<String>, String> {
            Ok(None)
        }
        fn get_session_context_data(
            &self,
            _session_id: &Uuid,
            _user_id: &Uuid,
        ) -> Result<String, String> {
            Ok(String::new())
        }
        fn update_session_context(
            &mut self,
            _session_id: &Uuid,
            _user_id: &Uuid,
            _context_data: String,
        ) -> Result<(), String> {
            Ok(())
        }
        fn get_user_sessions(
            &mut self,
            _user_id: Uuid,
        ) -> Result<Vec<botlib::models::UserSession>, String> {
            Ok(Vec::new())
        }
        fn update_user_id(
            &mut self,
            _session_id: Uuid,
            _new_user_id: Uuid,
        ) -> Result<(), String> {
            Ok(())
        }
        fn mark_waiting(&mut self, _session_id: Uuid) {}
        fn active_count(&self) -> usize {
            0
        }
    }
    std::sync::Arc::new(tokio::sync::Mutex::new(StubSession))
}

#[test]
fn provider_uses_runtime_overrides() {
    let cfg = HashMap::from([("botmodels-host".into(), "127.0.0.1".into())]);
    let runtime = MapRuntime { cfg };
    let bot_id = Uuid::new_v4();
    let provider = RuntimeConfigProvider {
        runtime: &runtime,
        bot_id,
    };
    assert_eq!(
        provider.get_config(&bot_id, "botmodels-host", Some("0.0.0.0")),
        Some("127.0.0.1".into())
    );
    assert_eq!(
        provider.get_config(&bot_id, "missing", Some("default")),
        Some("default".into())
    );
}

#[test]
fn botmodels_config_uses_runtime_provider() {
    let cfg = HashMap::from([
        ("botmodels-enabled".into(), "true".into()),
        ("botmodels-host".into(), "10.0.0.1".into()),
        ("botmodels-port".into(), "9999".into()),
    ]);
    let runtime = MapRuntime { cfg };
    let bot_id = Uuid::new_v4();
    let provider = RuntimeConfigProvider {
        runtime: &runtime,
        bot_id,
    };
    let cfg = BotModelsConfig::from_provider(&provider, &bot_id);
    assert!(cfg.enabled);
    assert_eq!(cfg.host, "10.0.0.1");
    assert_eq!(cfg.port, 9999);
}

#[test]
fn image_video_configs_have_defaults() {
    let runtime = MapRuntime::default();
    let bot_id = Uuid::new_v4();
    let provider = RuntimeConfigProvider {
        runtime: &runtime,
        bot_id,
    };
    let img = ImageGeneratorConfig::from_provider(&provider, &bot_id);
    let vid = VideoGeneratorConfig::from_provider(&provider, &bot_id);
    assert!(img.steps >= 1);
    assert!(vid.frames >= 1);
}
