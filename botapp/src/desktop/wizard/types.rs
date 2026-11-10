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
    /// When true, agent-related setup (WSL2/Incus VM work on Windows) is
    /// skipped so Chat/Drive-only installs can proceed without WSL2.
    #[serde(default)]
    pub skip_all_agentic_work: bool,
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

/// Network mode chosen on the wizard Network step.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkDraft {
    /// `"p2p"` or `"offline"` (default). Persisted as GB_NETWORK_MODE.
    #[serde(default)]
    pub mode: String,
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
    #[serde(default)]
    pub network: NetworkDraft,
    /// Persisted choice from the install step; allows re-runs to remember that
    /// agent-related setup was intentionally skipped.
    #[serde(default)]
    pub skip_all_agentic_work: bool,
}

#[cfg(test)]
mod tests {
    use super::{
        ComponentSpec, FinishRequest, ModelRequest, NetworkDraft, ProgressEvent, ProvisionRequest,
        WorkspaceDraft,
    };

    #[test]
    fn an_empty_model_request_has_empty_fields() {
        let request: ModelRequest = serde_json::from_str("{}").expect("defaults apply");
        assert!(request.mode.is_empty());
        assert!(request.provider.is_empty());
        assert!(request.endpoint.is_empty());
        assert!(request.model.is_empty());
        assert!(request.api_key.is_empty());
    }

    #[test]
    fn a_minimal_provision_request_fills_every_section() {
        let request: ProvisionRequest = serde_json::from_str("{}").expect("defaults apply");
        assert!(request.model.mode.is_empty());
        assert!(request.workspace.path.is_empty());
        assert!(request.services.is_empty());
        assert!(request.display_name.is_empty());
        // Agent work is opted out of, never silently opted in.
        assert!(!request.skip_all_agentic_work);
    }

    #[test]
    fn the_network_mode_defaults_to_an_empty_choice() {
        let draft: NetworkDraft = serde_json::from_str("{}").expect("defaults apply");
        assert!(draft.mode.is_empty());
        let chosen: NetworkDraft = serde_json::from_str(r#"{"mode":"p2p"}"#).expect("valid");
        assert_eq!(chosen.mode, "p2p");
    }

    #[test]
    fn a_finish_request_carries_the_administrator_and_network_choice() {
        let request: FinishRequest = serde_json::from_str(
            r#"{"admin":{"email":"a@b.c","password":"s"},"network":{"mode":"p2p"}}"#,
        )
        .expect("valid payload");
        assert_eq!(request.admin.email, "a@b.c");
        assert_eq!(request.network.mode, "p2p");
        assert!(request.workspace.size.is_empty());
        assert!(request.services.is_empty());
    }

    #[test]
    fn a_workspace_draft_keeps_its_size() {
        let draft: WorkspaceDraft =
            serde_json::from_str(r#"{"path":"/data/ws","size":"50 GB"}"#).expect("valid");
        assert_eq!(draft.path, "/data/ws");
        assert_eq!(draft.size, "50 GB");
    }

    #[test]
    fn a_progress_event_round_trips_for_the_polling_client() {
        let event = ProgressEvent {
            id: "vault".to_string(),
            state: "ready".to_string(),
            message: "Local vault is ready".to_string(),
            elapsed_ms: 1500,
        };
        let raw = serde_json::to_string(&event).expect("serialize");
        let back: ProgressEvent = serde_json::from_str(&raw).expect("deserialize");
        assert_eq!(back.id, "vault");
        assert_eq!(back.state, "ready");
        assert_eq!(back.elapsed_ms, 1500);
    }

    #[test]
    fn a_component_spec_serializes_the_keys_the_page_reads() {
        let spec = ComponentSpec {
            id: "llm".to_string(),
            label: "Local model runtime".to_string(),
            optional: true,
        };
        let value = serde_json::to_value(&spec).expect("serialize");
        assert_eq!(value["id"], serde_json::json!("llm"));
        assert_eq!(value["optional"], serde_json::json!(true));
        assert_eq!(value["label"], serde_json::json!("Local model runtime"));
    }
}
