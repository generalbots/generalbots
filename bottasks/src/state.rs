use std::{future::Future, pin::Pin, sync::Arc};

use crate::DbPool;

pub type RunCommandFn = Arc<dyn Fn(&str, &[&str]) -> Result<String, String> + Send + Sync>;

pub type CallLlmFn =
    Arc<dyn Fn(&str, &str) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send>> + Send + Sync>;

pub type GetConfigFn = Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;

pub type CacheGetFn =
    Arc<dyn Fn(String) -> Pin<Box<dyn Future<Output = Result<Option<String>, String>> + Send>> + Send + Sync>;

pub type CacheSetFn = Arc<
    dyn Fn(String, String, Option<u64>) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync,
>;

pub struct TasksState {
    pub pool: DbPool,
    pub run_command: RunCommandFn,
    pub call_llm: CallLlmFn,
    pub get_config: GetConfigFn,
    pub cache_get: CacheGetFn,
    pub cache_set: CacheSetFn,
}
