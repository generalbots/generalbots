//! Persisted wizard progress and the context sent to the static page.

use serde::{Deserialize, Serialize};

use super::paths::{log_path, marker_path, progress_path, stack_path, workspace_path};
use super::paths::{is_desktop_mode, is_first_run};
use super::types::{ComponentSpec, WizardContext, COMPONENTS};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub(crate) struct ProgressMarker {
    #[serde(default)]
    pub(crate) completed: Vec<String>,
    #[serde(default)]
    pub(crate) step: u8,
}

fn read_progress() -> ProgressMarker {
    let raw = match std::fs::read_to_string(progress_path()) {
        Ok(raw) => raw,
        Err(_) => return ProgressMarker::default(),
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

pub(crate) fn write_progress(marker: &ProgressMarker) -> std::io::Result<()> {
    let stack = stack_path();
    std::fs::create_dir_all(&stack)?;
    let raw = serde_json::to_string_pretty(marker)
        .map_err(|e| std::io::Error::other(format!("serialize progress: {e}")))?;
    std::fs::write(progress_path(), raw)
}

fn components() -> Vec<ComponentSpec> {
    COMPONENTS
        .iter()
        .map(|(id, label, optional)| ComponentSpec {
            id: (*id).to_string(),
            label: (*label).to_string(),
            optional: *optional,
        })
        .collect()
}

pub(crate) fn context() -> WizardContext {
    let progress = read_progress();
    let resume_step = if progress.step > 0 && !marker_path().exists() {
        Some(progress.step)
    } else {
        None
    };
    WizardContext {
        desktop_mode: is_desktop_mode(),
        first_run: is_first_run(),
        desktop_local: cfg!(feature = "desktop-local"),
        stack_path: stack_path().to_string_lossy().to_string(),
        workspace_path: workspace_path().to_string_lossy().to_string(),
        log_path: log_path().to_string_lossy().to_string(),
        platform: std::env::consts::OS.to_string(),
        api_base: "http://127.0.0.1:8080".to_string(),
        redirect_uri: "http://127.0.0.1:8080/api/integrations/oauth/{provider}/callback".to_string(),
        resume_step,
        completed_components: progress.completed,
        components: components(),
    }
}
