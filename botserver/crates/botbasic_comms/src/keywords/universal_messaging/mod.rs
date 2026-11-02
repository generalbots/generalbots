// Channel adapters accessed via botserver layer
// use botinstagram::InstagramAdapter;
// use botmsteams::TeamsAdapter;
// use botwhatsapp::WhatsAppAdapter;

// Stub adapter types - real implementations require botserver-layer registration
// These allow the module to compile without the adapter crate dependencies.
// Actual channel adapters are registered at the botserver layer with full AppState access.

pub(crate) use log::trace;
pub(crate) use rhai::{Dynamic, Engine};

struct WhatsAppAdapter;

struct InstagramAdapter;

struct TeamsAdapter;

use botcore::shared::MessageType;
use botcore::shared::UserSession;
use botcore::shared::state::AppState;
use serde_json::json;
use std::sync::Arc;
use reqwest::Client;

mod send_message;
mod instagram_adapter;
mod register_universal_messaging;
mod send_file_to_recipient;
mod send_web_message;

pub use register_universal_messaging::*;
use send_file_to_recipient::*;
use send_web_message::*;
