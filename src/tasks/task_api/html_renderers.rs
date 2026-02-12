//! HTML rendering functions for task UI
use crate::auto_task::TaskManifest;
use crate::core::shared::state::AppState;
use std::sync::Arc;

/// Build HTML for the progress log section from step_results JSON
pub fn build_terminal_html(step_results: &Option<serde_json::Value>, status: &str) -> String {
    let mut html = String::new();
    let mut output_lines: Vec<(String, bool)> = Vec::new();

    if let Some(serde_json::Value::Array(steps)) = step_results {
        for step in steps.iter() {
            let step_status = step.get("status").and_then(|v| v.as_str()).unwrap_or("");
            let is_current = step_status == "running" || step_status == "Running";

            if let Some(serde_json::Value::Array(logs)) = step.get("logs") {
                for log_entry in logs.iter() {
                    if let Some(msg) = log_entry.get("message").and_then(|v| v.as_str()) {
                        if !msg.trim().is_empty() {
                            output_lines.push((msg.to_string(), is_current));
                        }
                    }
                    if let Some(code) = log_entry.get("code").and_then(|v| v.as_str()) {
                        if !code.trim().is_empty() {
                            for line in code.lines().take(20) {
                                output_lines.push((format!("  {}", line), is_current));
                            }
                        }
                    }
                    if let Some(output) = log_entry.get("output").and_then(|v| v.as_str()) {
                        if !output.trim().is_empty() {
                            for line in output.lines().take(10) {
                                output_lines.push((format!("→ {}", line), is_current));
                            }
                        }
                    }
                }
            }
        }
    }

    if output_lines.is_empty() {
        let msg = match status {
            "running" => "Agent working...",
            "pending" => "Waiting to start...",
            "completed" | "done" => "✓ Task completed",
            "failed" | "error" => "✗ Task failed",
            "paused" => "Task paused",
            _ => "Initializing..."
        };
        html.push_str(&format!(r#"<div class="terminal-line">{}</div>"#, msg));
    } else {
        let start = if output_lines.len() > 15 { output_lines.len() - 15 } else { 0 };
        for (line, is_current) in output_lines[start..].iter() {
            let class = if *is_current { "terminal-line current" } else { "terminal-line" };
            let escaped = line.replace('<', "&lt;").replace('>', "&gt;");
            html.push_str(&format!(r#"<div class="{}">{}</div>"#, class, escaped));
        }
    }

    html
}

pub fn build_taskmd_html(state: &Arc<AppState>, task_id: &str, title: &str, runtime: &str, db_manifest: Option<&serde_json::Value>) -> (String, String) {
    log::info!("[TASKMD_HTML] Building TASK.md view for task_id: {}", task_id);

    // First, try to get manifest from in-memory cache (for active/running tasks)
    if let Ok(manifests) = state.task_manifests.read() {
        if let Some(manifest) = manifests.get(task_id) {
            log::info!("[TASKMD_HTML] Found manifest in memory for task: {} with {} sections", manifest.app_name, manifest.sections.len());
            let status_html = build_status_section_html(manifest, title, runtime);
            let progress_html = build_progress_log_html(manifest);
            return (status_html, progress_html);
        }
    }

    // If not in memory, try to load from database (for completed/historical tasks)
    if let Some(manifest_json) = db_manifest {
        log::info!("[TASKMD_HTML] Found manifest in database for task: {}", task_id);
        if let Ok(manifest) = serde_json::from_value::<TaskManifest>(manifest_json.clone()) {
            log::info!("[TASKMD_HTML] Parsed DB manifest for task: {} with {} sections", manifest.app_name, manifest.sections.len());
            let status_html = build_status_section_html(&manifest, title, runtime);
            let progress_html = build_progress_log_html(&manifest);
            return (status_html, progress_html);
        } else {
            // Try parsing as web JSON format (the format we store)
            if let Ok(web_manifest) = super::utils::parse_web_manifest_json(manifest_json) {
                log::info!("[TASKMD_HTML] Parsed web manifest from DB for task: {}", task_id);
                let status_html = build_status_section_from_web_json(&web_manifest, title, runtime);
                let progress_html = build_progress_log_from_web_json(&web_manifest);
                return (status_html, progress_html);
            }
            log::warn!("[TASKMD_HTML] Failed to parse manifest JSON for task: {}", task_id);
        }
    }

    log::info!("[TASKMD_HTML] No manifest found for task: {}", task_id);

    let default_status = format!(r#"
        <div class="status-row">
            <span class="status-title">{}</span>
            <span class="status-time">Runtime: {}</span>
        </div>
    "#, title, runtime);

    (default_status, r#"<div class="progress-empty">No steps executed yet</div>"#.to_string())
}

fn build_status_section_from_web_json(manifest: &serde_json::Value, title: &str, runtime: &str) -> String {
    let mut html = String::new();

    let current_action = manifest
        .get("current_status")
        .and_then(|s| s.get("current_action"))
        .and_then(|a| a.as_str())
        .unwrap_or("Processing...");

    let estimated_seconds = manifest
        .get("estimated_seconds")
        .and_then(|e| e.as_u64())
        .unwrap_or(0);

    let estimated = if estimated_seconds >= 60 {
        format!("{} min", estimated_seconds / 60)
    } else {
        format!("{} sec", estimated_seconds)
    };

    let runtime_display = if runtime == "0s" || runtime == "calculating..." {
        "Not started".to_string()
    } else {
        runtime.to_string()
    };

    html.push_str(&format!(r#"
        <div class="status-row status-main">
            <span class="status-title">{}</span>
            <span class="status-time">Runtime: {} <span class="status-indicator"></span></span>
        </div>
        <div class="status-row status-current">
            <span class="status-dot active"></span>
            <span class="status-text">{}</span>
            <span class="status-time">Estimated: {} <span class="status-gear">⚙</span></span>
        </div>
    "#, title, runtime_display, current_action, estimated));

    html
}

fn build_progress_log_from_web_json(manifest: &serde_json::Value) -> String {
    let mut html = String::new();
    html.push_str(r#"<div class="taskmd-tree">"#);

    let total_steps = manifest
        .get("total_steps")
        .and_then(|t| t.as_u64())
        .unwrap_or(60) as u32;

    let sections = match manifest.get("sections").and_then(|s| s.as_array()) {
        Some(s) => s,
        None => {
            html.push_str("</div>");
            return html;
        }
    };

    for section in sections {
        let section_id = section.get("id").and_then(|i| i.as_str()).unwrap_or("unknown");
        let section_name = section.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown");
        let section_status = section.get("status").and_then(|s| s.as_str()).unwrap_or("Pending");

        // Progress fields are nested inside a "progress" object in the web JSON format
        let progress = section.get("progress");
        let current_step = progress
            .and_then(|p| p.get("current"))
            .and_then(|c| c.as_u64())
            .unwrap_or(0) as u32;
        let global_step_start = progress
            .and_then(|p| p.get("global_start"))
            .and_then(|g| g.as_u64())
            .unwrap_or(0) as u32;

        let section_class = match section_status.to_lowercase().as_str() {
            "completed" => "completed expanded",
            "running" => "running expanded",
            "failed" => "failed",
            "skipped" => "skipped",
            _ => "pending",
        };

        let global_current = global_step_start + current_step;

        html.push_str(&format!(r#"
            <div class="tree-section {} expanded" data-section-id="{}">
                <div class="tree-row tree-level-0" onclick="this.parentElement.classList.toggle('expanded')">
                    <span class="tree-name">{}</span>
                    <span class="tree-step-badge">Step {}/{}</span>
                    <span class="tree-status {}">{}</span>
                    <span class="tree-section-dot {}"></span>
                </div>
                <div class="tree-children">
        "#, section_class, section_id, section_name, global_current, total_steps, section_class, section_status, section_class));

        // Render children
        if let Some(children) = section.get("children").and_then(|c| c.as_array()) {
            for child in children {
                let child_id = child.get("id").and_then(|i| i.as_str()).unwrap_or("unknown");
                let child_name = child.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown");
                let child_status = child.get("status").and_then(|s| s.as_str()).unwrap_or("Pending");

                // Progress fields are nested inside a "progress" object in the web JSON format
                let child_progress = child.get("progress");
                let child_current = child_progress
                    .and_then(|p| p.get("current"))
                    .and_then(|c| c.as_u64())
                    .unwrap_or(0) as u32;
                let child_total = child_progress
                    .and_then(|p| p.get("total"))
                    .and_then(|t| t.as_u64())
                    .unwrap_or(0) as u32;

                let child_class = match child_status.to_lowercase().as_str() {
                    "completed" => "completed expanded",
                    "running" => "running expanded",
                    "failed" => "failed",
                    "skipped" => "skipped",
                    _ => "pending",
                };

                html.push_str(&format!(r#"
                    <div class="tree-child {} expanded" data-child-id="{}">
                        <div class="tree-row tree-level-1" onclick="this.parentElement.classList.toggle('expanded')">
                            <span class="tree-indent"></span>
                            <span class="tree-name">{}</span>
                            <span class="tree-step-badge">Step {}/{}</span>
                            <span class="tree-status {}">{}</span>
                        </div>
                        <div class="tree-items">
                "#, child_class, child_id, child_name, child_current, child_total, child_class, child_status));

                // Render items
                if let Some(items) = child.get("items").and_then(|i| i.as_array()) {
                    for item in items {
                        let item_name = item.get("name").and_then(|n| n.as_str()).unwrap_or("Unknown");
                        let item_status = item.get("status").and_then(|s| s.as_str()).unwrap_or("Pending");
                        let duration = item.get("duration_seconds").and_then(|d| d.as_u64());

                        let item_class = match item_status.to_lowercase().as_str() {
                            "completed" => "completed",
                            "running" => "running",
                            _ => "pending",
                        };

                        let check_mark = if item_status.to_lowercase() == "completed" { "✓" } else { "" };
                        let duration_str = duration
                            .map(|s| if s >= 60 { format!("Duration: {} min", s / 60) } else { format!("Duration: {} sec", s) })
                            .unwrap_or_default();

                        html.push_str(&format!(r#"
                            <div class="tree-item {}">
                                <span class="item-dot {}"></span>
                                <span class="item-name">{}</span>
                                <div class="item-info">
                                    <span class="item-duration">{}</span>
                                    <span class="item-check {}">{}</span>
                                </div>
                            </div>
                        "#, item_class, item_class, item_name, duration_str, item_class, check_mark));
                    }
                }

                html.push_str("</div></div>"); // Close tree-items and tree-child
            }
        }

        html.push_str("</div></div>"); // Close tree-children and tree-section
    }

    html.push_str("</div>"); // Close taskmd-tree
    html
}

fn build_status_section_html(manifest: &TaskManifest, _title: &str, runtime: &str) -> String {
    let mut html = String::new();

    let current_action = manifest.current_status.current_action.as_deref().unwrap_or("Processing...");

    // Format estimated time nicely
    let estimated = if manifest.estimated_seconds >= 60 {
        format!("{} min", manifest.estimated_seconds / 60)
    } else {
        format!("{} sec", manifest.estimated_seconds)
    };

    // Format runtime nicely
    let runtime_display = if runtime == "0s" || runtime == "calculating..." {
        "Not started".to_string()
    } else {
        runtime.to_string()
    };

    html.push_str(&format!(r#"
        <div class="status-row status-current">
            <span class="status-dot active"></span>
            <span class="status-text">{}</span>
            <span class="status-time">Runtime: {} | Est: {}</span>
        </div>
    "#, current_action, runtime_display, estimated));

    if let Some(ref dp) = manifest.current_status.decision_point {
        html.push_str(&format!(r#"
            <div class="status-row status-decision">
                <span class="status-dot pending"></span>
                <span class="status-text">Decision Point Coming (Step {}/{})</span>
                <span class="status-badge">{}</span>
            </div>
        "#, dp.step_current, dp.step_total, dp.message));
    }

    html
}

fn build_progress_log_html(manifest: &TaskManifest) -> String {
    let mut html = String::new();
    html.push_str(r#"<div class="taskmd-tree">"#);

    let total_steps = manifest.total_steps;

    log::info!("[PROGRESS_HTML] Building progress log, {} sections, total_steps={}", manifest.sections.len(), total_steps);

    for section in &manifest.sections {
        log::info!("[PROGRESS_HTML] Section '{}': children={}, items={}, item_groups={}",
            section.name, section.children.len(), section.items.len(), section.item_groups.len());
        let section_class = match section.status {
            crate::auto_task::SectionStatus::Completed => "completed expanded",
            crate::auto_task::SectionStatus::Running => "running expanded",
            crate::auto_task::SectionStatus::Failed => "failed",
            crate::auto_task::SectionStatus::Skipped => "skipped",
            _ => "pending",
        };

        let status_text = match section.status {
            crate::auto_task::SectionStatus::Completed => "Completed",
            crate::auto_task::SectionStatus::Running => "Running",
            crate::auto_task::SectionStatus::Failed => "Failed",
            crate::auto_task::SectionStatus::Skipped => "Skipped",
            _ => "Pending",
        };

        // Use global step count (e.g., "Step 24/60")
        let global_current = section.global_step_start + section.current_step;

        html.push_str(&format!(r#"
            <div class="tree-section {} expanded" data-section-id="{}">
                <div class="tree-row tree-level-0" onclick="this.parentElement.classList.toggle('expanded')">
                    <span class="tree-name">{}</span>
                    <span class="tree-step-badge">Step {}/{}</span>
                    <span class="tree-status {}">{}</span>
                    <span class="tree-section-dot {}"></span>
                </div>
                <div class="tree-children">
        "#, section_class, section.id, section.name, global_current, total_steps, section_class, status_text, section_class));

        for child in &section.children {
            log::info!("[PROGRESS_HTML]   Child '{}': items={}, item_groups={}",
                child.name, child.items.len(), child.item_groups.len());
            let child_class = match child.status {
                crate::auto_task::SectionStatus::Completed => "completed expanded",
                crate::auto_task::SectionStatus::Running => "running expanded",
                crate::auto_task::SectionStatus::Failed => "failed",
                crate::auto_task::SectionStatus::Skipped => "skipped",
                _ => "pending",
            };

            let child_status = match child.status {
                crate::auto_task::SectionStatus::Completed => "Completed",
                crate::auto_task::SectionStatus::Running => "Running",
                crate::auto_task::SectionStatus::Failed => "Failed",
                crate::auto_task::SectionStatus::Skipped => "Skipped",
                _ => "Pending",
            };

            html.push_str(&format!(r#"
                <div class="tree-child {} expanded" data-child-id="{}" onclick="this.classList.toggle('expanded')">
                    <div class="tree-row tree-level-1">
                        <span class="tree-indent"></span>
                        <span class="tree-name">{}</span>
                        <span class="tree-step-badge">Step {}/{}</span>
                        <span class="tree-status {}">{}</span>
                    </div>
                    <div class="tree-items">
            "#, child_class, child.id, child.name, child.current_step, child.total_steps, child_class, child_status));

            // Render item groups first (grouped fields like "email, password_hash, email_verified")
            for group in &child.item_groups {
                let group_class = match group.status {
                    crate::auto_task::ItemStatus::Completed => "completed",
                    crate::auto_task::ItemStatus::Running => "running",
                    _ => "pending",
                };
                let check_mark = if group.status == crate::auto_task::ItemStatus::Completed { "✓" } else { "" };

                let group_duration = group.duration_seconds
                    .map(|s| if s >= 60 { format!("Duration: {} min", s / 60) } else { format!("Duration: {} sec", s) })
                    .unwrap_or_default();

                let group_name = group.display_name();

                html.push_str(&format!(r#"
                    <div class="tree-item {}" data-item-id="{}">
                        <span class="tree-item-dot {}"></span>
                        <span class="tree-item-name">{}</span>
                        <span class="tree-item-duration">{}</span>
                        <span class="tree-item-check {}">{}</span>
                    </div>
                "#, group_class, group.id, group_class, group_name, group_duration, group_class, check_mark));
            }

            // Then individual items
            for item in &child.items {
                let item_class = match item.status {
                    crate::auto_task::ItemStatus::Completed => "completed",
                    crate::auto_task::ItemStatus::Running => "running",
                    _ => "pending",
                };
                let check_mark = if item.status == crate::auto_task::ItemStatus::Completed { "✓" } else { "" };

                let item_duration = item.duration_seconds
                    .map(|s| if s >= 60 { format!("Duration: {} min", s / 60) } else { format!("Duration: {} sec", s) })
                    .unwrap_or_default();

                html.push_str(&format!(r#"
                    <div class="tree-item {}" data-item-id="{}">
                        <span class="tree-item-dot {}"></span>
                        <span class="tree-item-name">{}</span>
                        <span class="tree-item-duration">{}</span>
                        <span class="tree-item-check {}">{}</span>
                    </div>
                "#, item_class, item.id, item_class, item.name, item_duration, item_class, check_mark));
            }

            html.push_str("</div></div>");
        }

        // Render section-level item groups
        for group in &section.item_groups {
            let group_class = match group.status {
                crate::auto_task::ItemStatus::Completed => "completed",
                crate::auto_task::ItemStatus::Running => "running",
                _ => "pending",
            };
            let check_mark = if group.status == crate::auto_task::ItemStatus::Completed { "✓" } else { "" };

            let group_duration = group.duration_seconds
                .map(|s| if s >= 60 { format!("Duration: {} min", s / 60) } else { format!("Duration: {} sec", s) })
                .unwrap_or_default();

            let group_name = group.display_name();

            html.push_str(&format!(r#"
                <div class="tree-item {}" data-item-id="{}">
                    <span class="tree-item-dot {}"></span>
                    <span class="tree-item-name">{}</span>
                    <span class="tree-item-duration">{}</span>
                    <span class="tree-item-check {}">{}</span>
                </div>
            "#, group_class, group.id, group_class, group_name, group_duration, group_class, check_mark));
        }

        // Render section-level items
        for item in &section.items {
            let item_class = match item.status {
                crate::auto_task::ItemStatus::Completed => "completed",
                crate::auto_task::ItemStatus::Running => "running",
                _ => "pending",
            };
            let check_mark = if item.status == crate::auto_task::ItemStatus::Completed { "✓" } else { "" };

            let item_duration = item.duration_seconds
                .map(|s| if s >= 60 { format!("Duration: {} min", s / 60) } else { format!("Duration: {} sec", s) })
                .unwrap_or_default();

            html.push_str(&format!(r#"
                <div class="tree-item {}" data-item-id="{}">
                    <span class="tree-item-dot {}"></span>
                    <span class="tree-item-name">{}</span>
                    <span class="tree-item-duration">{}</span>
                    <span class="tree-item-check {}">{}</span>
                </div>
            "#, item_class, item.id, item_class, item.name, item_duration, item_class, check_mark));
        }

        html.push_str("</div></div>");
    }

    html.push_str("</div>");

    if manifest.sections.is_empty() {
        return r#"<div class="progress-empty">No steps executed yet</div>"#.to_string();
    }

    html
}
