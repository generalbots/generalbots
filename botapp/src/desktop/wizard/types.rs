//! Data shared by the wizard commands, the static page and provisioning.

use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::Mutex;

/// Components provisioned by the wizard, in dependency order.
pub const COMPONENTS: &[(&str, &str, bool)] = &[
    ("vault", "Local vault", false),
    ("tables", "Database", false),
    ("cache", "Cache", false),
    ("drive", "Object store", false),
    ("vector_db", "Vector store", false),
    ("directory", "Identity provider", false),
    ("llm", "Local model runtime", true),
    ("botmodels", "AI media service", true),
    ("botserver", "Assistant service", false),
];

#[derive(Default)]
pub struct WizardState {
    /// Pid of the `botserver --provision-stack` child, used only to cancel it.
    pub pid: Mutex<Option<u32>>,
    pub cancelled: AtomicBool,
    pub provisioning: AtomicBool,
    /// Last known state of every component, for clients that poll instead of
    /// listening to `wizard://progress` (events can be missed if the webview
    /// subscribes late).
    pub progress: Mutex<Vec<ProgressEvent>>,
    pub failed: Mutex<Option<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionStatus {
    pub running: bool,
    pub ready: bool,
    pub failed: Option<String>,
    pub components: Vec<ProgressEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSpec {
    pub id: String,
    pub label: String,
    pub optional: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardContext {
    pub desktop_mode: bool,
    pub first_run: bool,
    pub desktop_local: bool,
    pub stack_path: String,
    pub workspace_path: String,
    pub log_path: String,
    pub platform: String,
    pub api_base: String,
    pub redirect_uri: String,
    pub resume_step: Option<u8>,
    pub completed_components: Vec<String>,
    pub components: Vec<ComponentSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressEvent {
    pub id: String,
    pub state: String,
    pub message: String,
    pub elapsed_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRequest {
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceClient {
    #[serde(default)]
    pub client_id: String,
    #[serde(default)]
    pub client_secret: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelDraft {
    #[serde(default)]
    pub mode: String,
    #[serde(default)]
    pub provider: String,
    #[serde(default)]
    pub endpoint: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub api_key: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceDraft {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProvisionRequest {
    #[serde(default)]
    pub model: ModelDraft,
    #[serde(default)]
    pub workspace: WorkspaceDraft,
    #[serde(default)]
    pub services: std::collections::HashMap<String, ServiceClient>,
    #[serde(default)]
    pub display_name: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AdminDraft {
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub password: String,
    #[serde(default)]
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FinishRequest {
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub model: ModelDraft,
    #[serde(default)]
    pub workspace: WorkspaceDraft,
    #[serde(default)]
    pub services: std::collections::HashMap<String, ServiceClient>,
    #[serde(default)]
    pub admin: AdminDraft,
}
