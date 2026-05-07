use std::{future::Future, pin::Pin, sync::Arc};

use crate::DbPool;
use uuid::Uuid;

pub type SendMessageFn = Arc<
    dyn Fn(&str, &str, &str) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync,
>;

pub type GetDefaultBotFn = Arc<dyn Fn(&mut diesel::PgConnection) -> (Uuid, String) + Send + Sync>;

pub type GetConfigFn = Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;

pub type SecretsProvider = Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;

pub type TranscribeAudioFn = Arc<
    dyn Fn(&[u8]) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send>> + Send + Sync,
>;

pub type ProcessMessageFn = Arc<
    dyn Fn(String, String, String) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync,
>;

pub struct WhatsAppState {
    pub pool: DbPool,
    pub send_message: SendMessageFn,
    pub get_default_bot: GetDefaultBotFn,
    pub get_config: GetConfigFn,
    pub secrets: SecretsProvider,
    pub transcribe_audio: TranscribeAudioFn,
    pub process_message: ProcessMessageFn,
}
