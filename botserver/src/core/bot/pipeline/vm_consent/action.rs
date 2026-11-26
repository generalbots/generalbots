//! What an agent asks to do, as the card shows it, the audit keeps it and
//! the grant classes count it.

use serde_json::{json, Value};

/// Who asks: the chat model's native tools or one of the agent CLIs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VmAgent {
    Native,
    ClaudeCode,
    CodexCli,
    Antigravity,
}

impl VmAgent {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Native => "chat",
            Self::ClaudeCode => "claude-code",
            Self::CodexCli => "codex-cli",
            Self::Antigravity => "antigravity",
        }
    }
}

/// One action. Text fields hold the exact command, path or URL: the card
/// shows them in full; logs never carry them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VmActionKind {
    /// A command or program. `language` is `shell`, `python` or `node` for a
    /// program; `packages` are installed from the internet first; `repair`
    /// marks the automatic corrected version of a failed program.
    Command {
        command: String,
        cwd: Option<String>,
        language: Option<String>,
        packages: Vec<String>,
        background: bool,
        repair: bool,
    },
    /// Read a file in the VM. Its own class, so allowing reads in a
    /// conversation never allows commands.
    FileRead { path: String },
    /// Create or change a file in the VM (`from`: where its content comes from).
    FileWrite { path: String, from: Option<String> },
    FileDelete { path: String },
    /// Open a URL (browser or fetch) from the VM.
    OpenUrl { url: String },
    /// Install packages from the network.
    NetworkInstall { packages: Vec<String>, manager: Option<String> },
    /// Copy VM files into the person's Drive (leaves the VM).
    DriveSave { files: Vec<String>, folder: Option<String> },
    /// Send VM files to an external cloud drive (leaves the VM).
    ExternalSave { files: Vec<String>, provider: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VmAction {
    pub agent: VmAgent,
    pub kind: VmActionKind,
}

/// Paths of the agent CLIs' own credentials: an action that mentions one is
/// asked every time, never covered by a stored grant or "Permitir tudo".
/// Detail key of the redacted copy (`botconsent::enforce::AUDIT_KEY`).
pub(crate) const AUDIT_KEY: &str = "audit";

const PROTECTED_MARKERS: &[&str] = &[".credentials", "/.claude", ".claude/", ".codex/auth.json", ".gemini", "oauth_creds"];

pub(crate) fn sha256_hex(text: &str) -> String {
    use sha2::{Digest, Sha256};
    hex::encode(Sha256::digest(text.as_bytes()))
}

fn clip(text: &str, max: usize) -> String {
    let mut out: String = text.chars().take(max).collect();
    if text.chars().count() > max {
        out.push('…');
    }
    out
}

impl VmAction {
    #[must_use]
    pub fn native(kind: VmActionKind) -> Self {
        Self { agent: VmAgent::Native, kind }
    }

    /// The `app_permissions.action_class` this action is granted under.
    #[must_use]
    pub fn class(&self) -> &'static str {
        match self.kind {
            VmActionKind::Command { .. } => "vm_command",
            VmActionKind::FileRead { .. } => "vm_file_read",
            VmActionKind::FileWrite { .. } => "vm_file_write",
            VmActionKind::FileDelete { .. } => "vm_file_delete",
            VmActionKind::OpenUrl { .. } => "vm_browser",
            VmActionKind::NetworkInstall { .. } => "vm_network",
            VmActionKind::DriveSave { .. } => "vm_drive_save",
            VmActionKind::ExternalSave { .. } => "vm_external_save",
        }
    }

    /// Short name of the kind, used by the card, the steps and the logs.
    #[must_use]
    pub fn kind_key(&self) -> &'static str {
        match self.kind {
            VmActionKind::Command { .. } => "command",
            VmActionKind::FileRead { .. } => "file_read",
            VmActionKind::FileWrite { .. } => "file_write",
            VmActionKind::FileDelete { .. } => "file_delete",
            VmActionKind::OpenUrl { .. } => "browser",
            VmActionKind::NetworkInstall { .. } => "network",
            VmActionKind::DriveSave { .. } => "drive_save",
            VmActionKind::ExternalSave { .. } => "external_save",
        }
    }

    #[must_use]
    pub fn outside_vm(&self) -> bool {
        matches!(self.kind, VmActionKind::DriveSave { .. } | VmActionKind::ExternalSave { .. })
    }

    /// The exact thing that will happen: command, path, URL or file list.
    #[must_use]
    pub fn text(&self) -> String {
        match &self.kind {
            VmActionKind::Command { command, .. } => command.clone(),
            VmActionKind::FileRead { path }
            | VmActionKind::FileWrite { path, .. }
            | VmActionKind::FileDelete { path } => path.clone(),
            VmActionKind::OpenUrl { url } => url.clone(),
            VmActionKind::NetworkInstall { packages, .. } => packages.join(", "),
            VmActionKind::DriveSave { files, .. } | VmActionKind::ExternalSave { files, .. } => {
                files.join(", ")
            }
        }
    }

    #[must_use]
    pub fn preview(&self, max: usize) -> String {
        clip(self.text().lines().next().unwrap_or(""), max)
    }

    #[must_use]
    pub fn protected(&self) -> bool {
        let text = self.text();
        let cwd = match &self.kind {
            VmActionKind::Command { cwd: Some(cwd), .. } => cwd.as_str(),
            _ => "",
        };
        PROTECTED_MARKERS.iter().any(|m| text.contains(m) || cwd.contains(m))
    }

    /// Decisions the card offers. Outside-VM and protected actions never
    /// offer "Permitir tudo"; protected ones are asked every time.
    #[must_use]
    pub fn offers(&self) -> Vec<&'static str> {
        if self.protected() {
            return vec!["allow_once", "deny"];
        }
        if self.outside_vm() {
            return vec!["allow_once", "allow_conversation", "deny"];
        }
        vec!["allow_once", "allow_conversation", "allow_all", "deny"]
    }

    /// Canonical form hashed for "this exact action".
    fn fields(&self) -> Value {
        match &self.kind {
            VmActionKind::Command { command, cwd, language, packages, background, repair } => json!({
                "command": command, "cwd": cwd, "language": language,
                "packages": packages, "background": background, "repair": repair,
            }),
            VmActionKind::FileWrite { path, from } => json!({ "path": path, "from": from }),
            VmActionKind::FileRead { path } | VmActionKind::FileDelete { path } => json!({ "path": path }),
            VmActionKind::OpenUrl { url } => json!({ "url": url }),
            VmActionKind::NetworkInstall { packages, manager } => {
                json!({ "packages": packages, "manager": manager })
            }
            VmActionKind::DriveSave { files, folder } => json!({ "files": files, "folder": folder }),
            VmActionKind::ExternalSave { files, provider } => {
                json!({ "files": files, "provider": provider })
            }
        }
    }

    /// SHA-256 of agent, kind and fields: identifies this exact action.
    #[must_use]
    pub fn hash(&self) -> String {
        let canonical = json!({ "agent": self.agent.as_str(), "kind": self.kind_key(), "fields": self.fields() });
        sha256_hex(&canonical.to_string())
    }

    /// What the step rows show for this action.
    #[must_use]
    pub fn step_view(&self) -> Value {
        let mut view = json!({
            "agent": self.agent.as_str(),
            "kind": self.kind_key(),
            "text": clip(&self.text(), 600),
            "outside_vm": self.outside_vm(),
        });
        if let (Some(map), Value::Object(fields)) = (view.as_object_mut(), self.fields()) {
            for key in ["cwd", "language", "packages", "background", "repair", "path", "from", "folder", "provider"] {
                if let Some(value) = fields.get(key) {
                    map.insert(key.to_string(), value.clone());
                }
            }
        }
        view
    }

    /// The card detail: the exact action, the offered decisions and the
    /// redacted copy (`audit`) that `consent_audit` keeps instead.
    #[must_use]
    pub fn card_detail(&self) -> Value {
        let mut detail = self.step_view();
        if let Some(map) = detail.as_object_mut() {
            map.insert("vm".to_string(), Value::Bool(true));
            map.insert("text".to_string(), Value::String(self.text()));
            map.insert("protected".to_string(), Value::Bool(self.protected()));
            map.insert("offers".to_string(), json!(self.offers()));
            map.insert("action_hash".to_string(), Value::String(self.hash()));
            map.insert(AUDIT_KEY.to_string(), self.audit_detail());
        }
        detail
    }

    /// Redacted, truncated copy for the audit trail, with the full hash.
    #[must_use]
    pub fn audit_detail(&self) -> Value {
        let redact = botsecurity_crypto::secrets::redact_sensitive_data;
        json!({
            "agent": self.agent.as_str(),
            "kind": self.kind_key(),
            "text": redact(&clip(&self.text(), 200)),
            "outside_vm": self.outside_vm(),
            "argument_hash": self.hash(),
        })
    }
}
