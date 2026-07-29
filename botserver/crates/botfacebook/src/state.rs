use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type SendMessageFn =
    Arc<dyn Fn(&str, &str, &str) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync>;

pub type ProcessMessageFn =
    Arc<dyn Fn(&str, &str, &str, &str, &str) -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync>;

pub type FindBotFn =
    Arc<dyn Fn(&str) -> Pin<Box<dyn Future<Output = Option<String>> + Send>> + Send + Sync>;

pub type SecretsProvider =
    Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;

pub type DefaultBotFn =
    Arc<dyn Fn() -> (uuid::Uuid, String) + Send + Sync>;

pub struct FacebookState {
    pub secrets: SecretsProvider,
    pub send_message: SendMessageFn,
    pub process_message: ProcessMessageFn,
    pub find_bot: FindBotFn,
    pub get_default_bot: DefaultBotFn,
}
