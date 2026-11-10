//! Contract tests for the `BasicRuntime` defaults (#2064).
//!
//! Every BASIC keyword crate implements `BasicRuntime`, and the trait supplies
//! fallbacks for the capabilities a thin runtime (desktop, tests, a bot without
//! an LLM) does not have. Those defaults are the difference between a clear
//! error and a panic, so they are pinned here.

use crate::runtime::BasicRuntime;
use crate::types::UserSession;
use std::any::Any;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use uuid::Uuid;

/// A pool that is never connected to: `build_unchecked` skips the handshake, so
/// the stub can satisfy `db_pool()` without a database.
fn detached_pool() -> &'static botlib::db_pool::DbPool {
    static POOL: OnceLock<botlib::db_pool::DbPool> = OnceLock::new();
    POOL.get_or_init(|| {
        let manager = diesel::r2d2::ConnectionManager::<diesel::PgConnection>::new(
            "postgres://test:test@127.0.0.1:1/test",
        );
        diesel::r2d2::Pool::builder()
            .build_unchecked(manager)
    })
}

#[derive(Debug)]
struct StubRuntime;

impl BasicRuntime for StubRuntime {
    fn db_pool(&self) -> &botlib::db_pool::DbPool {
        detached_pool()
    }

    fn cache_client(&self) -> Option<Arc<redis::Client>> {
        None
    }

    fn bucket_name(&self) -> &str {
        "stub.gbai"
    }

    fn hear_channels(&self) -> &Mutex<HashMap<Uuid, std::sync::mpsc::SyncSender<String>>> {
        static CHANNELS: OnceLock<Mutex<HashMap<Uuid, std::sync::mpsc::SyncSender<String>>>> =
            OnceLock::new();
        CHANNELS.get_or_init(|| Mutex::new(HashMap::new()))
    }

    fn bot_database_manager(&self) -> Arc<dyn botlib::traits::BotDatabaseService> {
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
        Arc::new(StubDb)
    }

    fn web_adapter(&self) -> Arc<dyn botlib::traits::ChannelAdapter> {
        #[derive(Debug)]
        struct StubAdapter;
        impl botlib::traits::ChannelAdapter for StubAdapter {
            fn channel_type(&self) -> &str {
                "stub"
            }
        }
        Arc::new(StubAdapter)
    }

    fn drive_repository(&self) -> Option<Arc<dyn botlib::traits::DriveRepository>> {
        None
    }

    fn config_value(&self, _key: &str) -> Option<String> {
        None
    }

    fn session_manager(&self) -> Arc<tokio::sync::Mutex<dyn botlib::traits::SessionManagerService>> {
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
                Ok(Uuid::nil())
            }
            fn create_session(
                &mut self,
                _user_id: Uuid,
                _bot_id: Uuid,
                _title: &str,
            ) -> Result<botlib::models::UserSession, String> {
                Err("stub".to_string())
            }
            fn get_or_create_session_by_id(
                &mut self,
                _session_id: Uuid,
                _user_id: Uuid,
                _bot_id: Uuid,
                _session_title: &str,
            ) -> Result<botlib::models::UserSession, String> {
                Err("stub".to_string())
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
            fn get_first_user_message(
                &mut self,
                _session_id: Uuid,
            ) -> Result<Option<String>, String> {
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
        Arc::new(tokio::sync::Mutex::new(StubSession))
    }

    fn update_session_user(&self, _session_id: Uuid, _user_id: Uuid) -> Result<(), String> {
        Ok(())
    }

    fn send_message(&self, _response: &botlib::models::BotResponse) -> Result<(), String> {
        Ok(())
    }

    fn execute_script(&self, _user: UserSession, _script: &str) -> Result<String, String> {
        Ok(String::new())
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

/// A runtime without an LLM must answer with a diagnosable error, never panic
/// and never fabricate output.
#[test]
fn llm_generate_defaults_to_a_configuration_error() {
    let runtime = StubRuntime;
    let error = runtime
        .llm_generate("hello", "any-model", "no-key")
        .expect_err("the default implementation has no provider");
    assert_eq!(error, "LLM provider not configured");
}

/// `default_bot_id` is empty until a runtime resolves one: the keyword layer
/// treats the empty string as "no bot scope", so it must not be a UUID.
#[test]
fn default_bot_id_is_empty() {
    let runtime = StubRuntime;
    assert_eq!(runtime.default_bot_id(), "");
    assert!(runtime.default_bot_id().is_empty());
}

/// The capabilities the stub leaves out are reported as absent, not as empty
/// objects, so callers can tell "not configured" from "configured but empty".
#[test]
fn optional_capabilities_default_to_none() {
    let runtime = StubRuntime;
    assert!(runtime.cache_client().is_none());
    assert!(runtime.drive_repository().is_none());
    assert!(runtime.config_value("llm-model").is_none());
    assert_eq!(runtime.bucket_name(), "stub.gbai");
}

/// `as_any` must hand back the concrete runtime so the host can downcast it.
#[test]
fn as_any_returns_the_concrete_runtime() {
    let runtime = StubRuntime;
    assert!(runtime.as_any().downcast_ref::<StubRuntime>().is_some());
}
