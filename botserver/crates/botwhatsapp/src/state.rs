use std::{future::Future, pin::Pin, sync::Arc};

use crate::DbPool;
use uuid::Uuid;

pub type SendMessageFn = Arc<
    dyn Fn(&str, &str, &str) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync,
>;

pub type GetDefaultBotFn = Arc<dyn Fn(&mut diesel::PgConnection) -> (Uuid, String) + Send + Sync>;

pub type FindBotFn = Arc<dyn Fn(&str) -> (Uuid, String) + Send + Sync>;

pub type GetConfigFn = Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;

pub type SecretsProvider = Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;

pub type TranscribeAudioFn = Arc<
    dyn Fn(&[u8]) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send>> + Send + Sync,
>;

pub type ProcessMessageFn = Arc<
    dyn Fn(String, String, String, String) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync,
>;

pub type UserLookupFn = Arc<
    dyn Fn(&str) -> Pin<Box<dyn Future<Output = Result<Option<String>, String>> + Send>> + Send + Sync,
>;

pub type UserCreateFn = Arc<
    dyn Fn(&str, &str, &str, Option<&str>) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send>> + Send + Sync,
>;

pub struct WhatsAppState {
    pub pool: DbPool,
    pub send_message: SendMessageFn,
    pub get_default_bot: GetDefaultBotFn,
    pub find_bot: FindBotFn,
    pub get_config: GetConfigFn,
    pub secrets: SecretsProvider,
    pub transcribe_audio: TranscribeAudioFn,
    pub process_message: ProcessMessageFn,
    pub user_lookup: UserLookupFn,
    pub user_create: UserCreateFn,
}
