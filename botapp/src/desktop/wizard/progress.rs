//! Persisted wizard progress and the context sent to the static page.

use serde::{Deserialize, Serialize};

use super::paths::{is_desktop_mode, is_first_run};
use super::paths::{log_path, marker_path, progress_path, stack_path, workspace_path};
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

/// Resumes at the persisted step only while the stack is still uninitialised:
/// a completed install (marker present) always starts from the beginning.
fn resume_step(step: u8, marker_exists: bool) -> Option<u8> {
    if step > 0 && !marker_exists {
        Some(step)
    } else {
        None
    }
}

pub(crate) fn context() -> WizardContext {
    let progress = read_progress();
    let resume_step = resume_step(progress.step, marker_path().exists());
    WizardContext {
        desktop_mode: is_desktop_mode(),
        first_run: is_first_run(),
        desktop_local: cfg!(feature = "desktop-local"),
        stack_path: stack_path().to_string_lossy().to_string(),
        workspace_path: workspace_path().to_string_lossy().to_string(),
        log_path: log_path().to_string_lossy().to_string(),
        platform: std::env::consts::OS.to_string(),
        api_base: "http://127.0.0.1:8080".to_string(),
        redirect_uri: "http://127.0.0.1:8080/api/integrations/oauth/{provider}/callback"
            .to_string(),
        resume_step,
        completed_components: progress.completed,
        components: components(),
    }
}

#[cfg(test)]
mod tests {
    use super::{components, resume_step};
    use crate::desktop::wizard::COMPONENTS;

    #[test]
    fn a_saved_step_only_resumes_on_an_uninitialised_stack() {
        assert_eq!(resume_step(5, false), Some(5));
        assert_eq!(resume_step(1, false), Some(1));
        // A completed install starts over even if the marker file is stale.
        assert_eq!(resume_step(5, true), None);
        // Step zero means nothing was saved.
        assert_eq!(resume_step(0, false), None);
    }

    #[test]
    fn component_specs_mirror_the_canonical_order() {
        let specs = components();
        assert_eq!(specs.len(), COMPONENTS.len());
        for (spec, (id, label, optional)) in specs.iter().zip(COMPONENTS.iter()) {
            assert_eq!(spec.id, *id);
            assert_eq!(spec.label, *label);
            assert_eq!(spec.optional, *optional);
        }
        // Only the model runtime and the media service are optional.
        let optional: Vec<&str> = specs
            .iter()
            .filter(|spec| spec.optional)
            .map(|spec| spec.id.as_str())
            .collect();
        assert_eq!(optional, vec!["llm", "botmodels"]);
    }
}
