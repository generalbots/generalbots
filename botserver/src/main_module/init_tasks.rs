use std::sync::Arc;

pub fn init_task_scheduler(_app_state: &Arc<botcore::shared::state::AppState>) {
    #[cfg(feature = "tasks")]
    {
        let tasks_state = Arc::new(crate::tasks::TasksState {
            pool: app_state.conn.clone(),
            run_command: Arc::new(|_cmd: &str, _args: &[&str]| -> Result<String, String> {
                Ok("stub".to_string())
            }),
            call_llm: Arc::new(|_sys: &str, _prompt: &str| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<String, String>> + Send>> {
                Box::pin(async { Ok("stub".to_string()) })
            }),
            get_config: Arc::new(|_key: &str| -> Result<String, String> {
                Ok("stub".to_string())
            }),
            cache_get: Arc::new(|_key: String| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<String>, String>> + Send>> {
                Box::pin(async { Ok(None) })
            }),
            cache_set: Arc::new(|_key: String, _value: String, _ttl: Option<u64>| -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>> + Send>> {
                Box::pin(async { Ok(()) })
            }),
        });
        let task_scheduler = Arc::new(crate::tasks::scheduler::TaskScheduler::new(tasks_state));
        task_scheduler.start();
    }
}
