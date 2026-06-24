pub mod callbacks;
pub mod error;
pub mod manager_types;
pub mod models;
pub mod schema;
pub mod utils;

pub use error::BotCoreBotError;
pub type BotResult<T> = Result<T, BotCoreBotError>;
