use botcore::shared::state::AppState;
use diesel::prelude::*;
use log::warn;
use rhai::{Dynamic, Engine, EvalAltResult, Scope};
pub use botcore::shared::UserSession;

pub use botbasic_compiler as compiler;
pub mod keywords;
#[path = "keywords/set_answer_mode.rs"]
mod set_answer_mode;

#[derive(QueryableByName)]
struct ParamConfigRow {
    #[diesel(sql_type = diesel::sql_types::Text)]
    config_key: String,
    #[diesel(sql_type = diesel::sql_types::Text)]
    config_value: String,
}

#[derive(Debug)]
pub struct ScriptService {
    pub engine: Engine,
    pub scope: Scope<'static>,
}

impl ScriptService {
    #[must_use]
    pub fn new(state: Arc<AppState>, user: LocalUserSession) -> Self {
        let mut engine = Engine::new();
        let scope = Scope::new();
        engine.set_allow_anonymous_fn(true);
        engine.set_allow_looping(true);

        let state_for_local = state.clone();
        let runtime: Arc<dyn BasicRuntime> = Arc::new(AppStateBasicRuntime(state));
        let bt_user = botbasic_types::UserSession::from(user.clone());

        botbasic_core::register_core_keywords(runtime.clone(), bt_user.clone(), &mut engine);
        botbasic_data::register_data_keywords(runtime.clone(), bt_user.clone(), &mut engine);
        botbasic_comms::register_comms_keywords(&runtime, bt_user.clone(), &mut engine);
        botbasic_ai::register_ai_keywords(runtime.clone(), bt_user.clone(), &mut engine);
        botbasic_system::register_system_keywords(runtime, bt_user, &mut engine);

        // Register local (botserver-only) keywords
        set_answer_mode::register_set_answer_mode_keyword(state_for_local, user, &mut engine);

        Self { engine, scope }
    }

    pub fn inject_config_variables(&mut self, config_vars: HashMap<String, String>) {
        for (key, value) in config_vars {
            let var_name = if key.starts_with("param-") {
                key.strip_prefix("param-").unwrap_or(&key).to_lowercase()
            } else {
                key.to_lowercase()
            };

            if let Ok(int_val) = value.parse::<i64>() {
                self.scope.push(&var_name, int_val);
            } else if let Ok(float_val) = value.parse::<f64>() {
                self.scope.push(&var_name, float_val);
            } else if value.eq_ignore_ascii_case("true") {
                self.scope.push(&var_name, true);
            } else if value.eq_ignore_ascii_case("false") {
                self.scope.push(&var_name, false);
            } else {
                self.scope.push(&var_name, value);
            }
        }
    }

    pub fn load_bot_config_params(&mut self, state: &AppState, bot_id: uuid::Uuid) {
        if let Ok(mut conn) = state.conn.get() {
            let result = diesel::sql_query(
                "SELECT config_key, config_value FROM bot_configuration WHERE bot_id = $1 AND config_key LIKE 'param-%'"
            )
            .bind::<diesel::sql_types::Uuid, _>(bot_id)
            .load::<ParamConfigRow>(&mut conn);

            if let Ok(params) = result {
                let config_vars: HashMap<String, String> = params
                    .into_iter()
                    .map(|row| (row.config_key, row.config_value))
                    .collect();
                self.inject_config_variables(config_vars);
            }
        }
    }

    pub fn run(&mut self, ast_content: &str) -> Result<Dynamic, Box<EvalAltResult>> {
        std::fs::write("/opt/gbo/logs/run_debug.txt", format!("run() called, ast_content len={}\n", ast_content.len())).ok();
        let preprocessed = compiler::syntax_transforms::convert_multiword_keywords(ast_content);
        let ast = match self.engine.compile(&preprocessed) {
            Ok(ast) => ast,
            Err(e) => {
                std::fs::write("/opt/gbo/logs/compile_error.txt", format!("compile error: {}\n", e)).ok();
                log::error!("[BASIC_EXEC] Failed to compile AST: {}", e);
                return Err(Box::new(e.into()));
            }
        };
        std::fs::write("/opt/gbo/logs/eval_start.txt", format!("eval_ast_with_scope starting\n")).ok();
        let result = self.engine.eval_ast_with_scope(&mut self.scope, &ast);
        std::fs::write("/opt/gbo/logs/eval_result.txt", format!("eval result: {:?}\n", result)).ok();
        result
    }

    pub async fn execute_script(
        state: Arc<AppState>,
        user: LocalUserSession,
        ast_content: &str,
    ) -> Result<String, String> {
        std::fs::write("/tmp/execute_script_debug.txt", format!(
            "ast_content len={}\nfirst 200: {}\n", ast_content.len(), &ast_content.chars().take(200).collect::<String>()
        )).ok();
        let mut script_service = Self::new(state.clone(), user.clone());
        script_service.load_bot_config_params(&state, user.bot_id);

        let result = script_service.run(ast_content);
        std::fs::write("/tmp/run_result.txt", format!("run result: {:?}\n", result)).ok();
        match result {
            Ok(result) => {
                let s = result.to_string();
                std::fs::write("/tmp/talk_debug.txt", format!("TALK debug: result='{}' len={}\n", s, s.len())).ok();
                Ok(s)
            }
            Err(e) => {
                let err_str = format!("Script error: {}", e);
                std::fs::write("/tmp/talk_debug.txt", format!("TALK ERROR: {}\n", err_str)).ok();
                Err(err_str)
            }
        }
    }

    pub fn set_variable(&mut self, name: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        self.scope.set_or_push(name, Dynamic::from(value.to_string()));
        Ok(())
    }
}

#[cfg(test)]
pub mod tests;

use std::collections::HashMap;
use std::sync::Arc;
use botbasic_types::BasicRuntime;
use botcore::shared::UserSession as LocalUserSession;
use uuid::Uuid;

#[derive(Debug)]
pub struct AppStateBasicRuntime(pub Arc<botcore::shared::state::AppState>);

impl BasicRuntime for AppStateBasicRuntime {
    fn db_pool(&self) -> &botlib::db_pool::DbPool {
        &self.0.conn
    }

    fn cache_client(&self) -> Option<Arc<redis::Client>> {
        #[cfg(feature = "cache")]
        {
            self.0.cache.clone()
        }
        #[cfg(not(feature = "cache"))]
        {
            None
        }
    }

    fn bucket_name(&self) -> &str {
        &self.0.bucket_name
    }

    fn hear_channels(&self) -> &std::sync::Mutex<HashMap<Uuid, std::sync::mpsc::SyncSender<String>>> {
        &self.0.hear_channels
    }

    fn bot_database_manager(&self) -> Arc<dyn botlib::traits::BotDatabaseService> {
        Arc::clone(&self.0.bot_database_manager)
    }

    fn web_adapter(&self) -> Arc<dyn botlib::traits::ChannelAdapter> {
        Arc::clone(&self.0.web_adapter)
    }

    fn drive_repository(&self) -> Option<Arc<dyn botlib::traits::DriveRepository>> {
        self.0.drive.clone()
    }

    fn config_value(&self, key: &str) -> Option<String> {
        self.0.config.as_ref()?.get(key)
    }

    fn session_manager(&self) -> Arc<tokio::sync::Mutex<dyn botlib::traits::SessionManagerService>> {
        Arc::clone(&self.0.session_manager)
    }

    fn update_session_user(&self, session_id: Uuid, user_id: Uuid) -> Result<(), String> {
        let sm = Arc::clone(&self.0.session_manager);
        let (tx, rx) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build();
            if let Ok(rt) = rt {
                let result = rt.block_on(async {
                    let mut guard = sm.lock().await;
                    guard.update_user_id(session_id, user_id)
                });
                let _ = tx.send(result);
            }
        });
        rx.recv().unwrap_or(Err("Channel error".to_string()))
    }

    fn send_message(&self, response: &botlib::models::BotResponse) -> Result<(), String> {
        let sid = response.session_id.clone();
        let resp = response.clone();
        let channels = self.0.response_channels.clone();
        tokio::task::block_in_place(move || {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async move {
                let guard = channels.lock().await;
                if let Some(tx) = guard.get(&sid).cloned() {
                    let _ = tx.send(resp).await;
                } else {
                    warn!("send_message: no channel for session {}", sid);
                }
            });
        });
        Ok(())
    }

    fn execute_script(&self, user: botbasic_types::UserSession, script: &str) -> Result<String, String> {
        let lib_user = botcore::shared::UserSession {
            id: user.id,
            user_id: user.user_id,
            bot_id: user.bot_id,
            title: user.title.clone(),
            context_data: user.context_data.clone(),
            current_tool: user.current_tool.clone(),
            created_at: user.created_at,
            updated_at: user.updated_at,
        };
        let mut service = crate::basic::ScriptService::new(Arc::clone(&self.0), lib_user);
        service.load_bot_config_params(&self.0, user.bot_id);

        // Inject trigger context variables from UserSession.context_data
        if let Some(obj) = user.context_data.as_object() {
            for key in &["trigger_record_id", "trigger_old_status", "trigger_new_status"] {
                if let Some(val) = obj.get(*key).and_then(|v| v.as_str()) {
                    let rhai_key = key.to_uppercase();
                    let _ = service.set_variable(&rhai_key, val);
                }
            }
        }

        let result = service.run(script);
        match result {
            Ok(dynamic) => Ok(dynamic.to_string()),
            Err(e) => Err(format!("Script execution error: {}", e)),
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl botlib::traits::ScriptRunner for crate::basic::ScriptService {
    fn run_script(
        &self,
        script: &str,
        _session_id: uuid::Uuid,
        _bot_id: &str,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>> {
        log::info!("ScriptRunner::run_script called: {}", script);
        let script = script.to_string();
        Box::pin(async move { Ok(script) })
    }

    fn get_suggestions(
        &self,
        _session_id: &uuid::Uuid,
        _bot_id: &str,
    ) -> Result<Vec<botlib::models::Suggestion>, String> {
        Ok(Vec::new())
    }
}
