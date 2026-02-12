//! Utility functions for task API
use crate::auto_task::TaskManifest;
use crate::core::shared::state::AppState;
use std::sync::Arc;

/// Extract app URL from step results
pub fn extract_app_url_from_results(step_results: &Option<serde_json::Value>, title: &str) -> Option<String> {
    if let Some(serde_json::Value::Array(steps)) = step_results {
        for step in steps.iter() {
            if let Some(logs) = step.get("logs").and_then(|v| v.as_array()) {
                for log in logs.iter() {
                    if let Some(msg) = log.get("message").and_then(|v| v.as_str()) {
                        if msg.contains("/apps/") {
                            if let Some(start) = msg.find("/apps/") {
                                let rest = &msg[start..];
                                let end = rest.find(|c: char| c.is_whitespace() || c == '"' || c == '\'').unwrap_or(rest.len());
                                let url = rest[..end].to_string();
                                // Add trailing slash if not present
                                if url.ends_with('/') {
                                    return Some(url);
                                } else {
                                    return Some(format!("{}/", url));
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let app_name = title
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_alphanumeric() || *c == '-')
        .collect::<String>();

    if !app_name.is_empty() {
        Some(format!("/apps/{}/", app_name))
    } else {
        None
    }
}

/// Get processed count from manifest
pub fn get_manifest_processed_count(state: &Arc<AppState>, task_id: &str) -> String {
    // First check in-memory manifest
    if let Ok(manifests) = state.task_manifests.read() {
        if let Some(manifest) = manifests.get(task_id) {
            let count = manifest.processing_stats.data_points_processed;
            if count > 0 {
                return count.to_string();
            }
            // Fallback: count completed items from manifest sections
            let completed_items: u64 = manifest.sections.iter()
                .map(|s| {
                    let section_items = s.items.iter().filter(|i| i.status == crate::auto_task::ItemStatus::Completed).count() as u64;
                    let section_groups = s.item_groups.iter().filter(|g| g.status == crate::auto_task::ItemStatus::Completed).count() as u64;
                    let child_items: u64 = s.children.iter().map(|c| {
                        c.items.iter().filter(|i| i.status == crate::auto_task::ItemStatus::Completed).count() as u64 +
                        c.item_groups.iter().filter(|g| g.status == crate::auto_task::ItemStatus::Completed).count() as u64
                    }).sum();
                    section_items + section_groups + child_items
                })
                .sum();
            if completed_items > 0 {
                return completed_items.to_string();
            }
        }
    }
    "-".to_string()
}

/// Get processing speed from manifest
pub fn get_manifest_speed(state: &Arc<AppState>, task_id: &str) -> String {
    if let Ok(manifests) = state.task_manifests.read() {
        if let Some(manifest) = manifests.get(task_id) {
            let speed = manifest.processing_stats.sources_per_min;
            if speed > 0.0 {
                return format!("{:.1}/min", speed);
            }
            // For completed tasks, show "-" instead of "calculating..."
            if manifest.status == crate::auto_task::ManifestStatus::Completed {
                return "-".to_string();
            }
        }
    }
    "-".to_string()
}

/// Get ETA from manifest
pub fn get_manifest_eta(state: &Arc<AppState>, task_id: &str) -> String {
    if let Ok(manifests) = state.task_manifests.read() {
        if let Some(manifest) = manifests.get(task_id) {
            // Check if completed first
            if manifest.status == crate::auto_task::ManifestStatus::Completed {
                return "Done".to_string();
            }
            let eta_secs = manifest.processing_stats.estimated_remaining_seconds;
            if eta_secs > 0 {
                if eta_secs >= 60 {
                    return format!("~{} min", eta_secs / 60);
                } else {
                    return format!("~{} sec", eta_secs);
                }
            }
        }
    }
    "-".to_string()
}

/// Parse the web JSON format that we store in the database
pub fn parse_web_manifest_json(json: &serde_json::Value) -> Result<serde_json::Value, ()> {
    // The web format has sections with status as strings, etc.
    if json.get("sections").is_some() {
        Ok(json.clone())
    } else {
        Err(())
    }
}
