use crate::types::TaskManifest;

pub fn render_cards(manifest: &TaskManifest) -> String {
    let mut html = String::new();

    html.push_str("<div class=\"task-cards\">");

    html.push_str(&render_status_card(manifest));
    html.push_str(&render_progress_card(manifest));
    html.push_str(&render_stats_card(manifest));

    for section in &manifest.sections {
        html.push_str(&render_section_card(&section.title, &section.items));
    }

    for item in &manifest.items {
        html.push_str(&render_item_card(item));
    }

    html.push_str("</div>");

    html
}

fn render_status_card(manifest: &TaskManifest) -> String {
    let status_class = match manifest.status.as_str() {
        "completed" => "status-completed",
        "running" => "status-running",
        "failed" => "status-failed",
        "cancelled" => "status-cancelled",
        _ => "status-pending",
    };

    let mut html = String::new();
    html.push_str("<div class=\"task-card status-card ");
    html.push_str(status_class);
    html.push_str("\">");
    html.push_str("<div class=\"card-header\">");
    html.push_str("<span class=\"card-icon\">📋</span>");
    html.push_str("<span class=\"card-title\">");
    html.push_str(&html_escape(&manifest.title));
    html.push_str("</span>");
    html.push_str("</div>");
    html.push_str("<div class=\"card-body\">");
    html.push_str("<div class=\"status-badge ");
    html.push_str(status_class);
    html.push_str("\">");
    html.push_str(&html_escape(&manifest.status));
    html.push_str("</div>");
    html.push_str("<div class=\"current-status\">");
    html.push_str(&html_escape(&manifest.current_status.title));
    html.push_str("</div>");
    if let Some(ref detail) = manifest.current_status.detail {
        html.push_str("<div class=\"status-detail\">");
        html.push_str(&html_escape(detail));
        html.push_str("</div>");
    }
    html.push_str("</div>");
    html.push_str("</div>");
    html
}

fn render_progress_card(manifest: &TaskManifest) -> String {
    let progress = if manifest.total_steps > 0 {
        (manifest.completed_steps as f64 / manifest.total_steps as f64 * 100.0) as u32
    } else {
        0
    };

    let mut html = String::new();
    html.push_str("<div class=\"task-card progress-card\">");
    html.push_str("<div class=\"card-header\">");
    html.push_str("<span class=\"card-icon\">📊</span>");
    html.push_str("<span class=\"card-title\">Progress</span>");
    html.push_str("</div>");
    html.push_str("<div class=\"card-body\">");
    html.push_str("<div class=\"progress-bar\"><div class=\"progress-fill\" style=\"width: ");
    html.push_str(&progress.to_string());
    html.push_str("%\"></div></div>");
    html.push_str("<div class=\"progress-text\">");
    html.push_str(&manifest.completed_steps.to_string());
    html.push_str(" / ");
    html.push_str(&manifest.total_steps.to_string());
    html.push_str(" steps (");
    html.push_str(&progress.to_string());
    html.push_str("%)</div>");
    html.push_str("</div>");
    html.push_str("</div>");

    html
}

fn render_stats_card(manifest: &TaskManifest) -> String {
    let mut html = String::new();
    html.push_str("<div class=\"task-card stats-card\">");
    html.push_str("<div class=\"card-header\">");
    html.push_str("<span class=\"card-icon\">⚡</span>");
    html.push_str("<span class=\"card-title\">Processing Stats</span>");
    html.push_str("</div>");
    html.push_str("<div class=\"card-body\">");
    html.push_str("<div class=\"stat-row\"><span>Tokens</span><span>");
    html.push_str(&manifest.processing_stats.total_tokens.to_string());
    html.push_str("</span></div>");
    html.push_str("<div class=\"stat-row\"><span>Cost</span><span>$");
    html.push_str(&format!("{:.4}", manifest.processing_stats.total_cost));
    html.push_str("</span></div>");
    html.push_str("</div>");
    html.push_str("</div>");

    html
}

fn render_section_card(title: &str, items: &[String]) -> String {    let mut html = String::new();
    html.push_str("<div class=\"task-card section-card\">");
    html.push_str("<div class=\"card-header\">");
    html.push_str("<span class=\"card-icon\">📂</span>");
    html.push_str("<span class=\"card-title\">");
    html.push_str(&html_escape(title));
    html.push_str("</span>");
    html.push_str("</div>");
    html.push_str("<div class=\"card-body\"><ul>");
    for item in items {
        html.push_str("<li>");
        html.push_str(&html_escape(item));
        html.push_str("</li>");
    }
    html.push_str("</ul></div>");
    html.push_str("</div>");

    html
}

fn render_item_card(item: &crate::types::ManifestItem) -> String {
    let mut html = String::new();
    html.push_str("<div class=\"task-card item-card\">");
    html.push_str("<div class=\"card-header\">");
    html.push_str("<span class=\"card-icon\">📌</span>");
    html.push_str("<span class=\"card-title\">");
    html.push_str(&html_escape(&item.name));
    html.push_str("</span>");
    html.push_str("</div>");
    html.push_str("<div class=\"card-body\">");

    for step in &item.steps {
        let step_class = match step.status.as_str() {
            "completed" => "step-completed",
            "running" => "step-running",
            "failed" => "step-failed",
            _ => "step-pending",
        };

        html.push_str("<div class=\"step ");
        html.push_str(step_class);
        html.push_str("\">");
        html.push_str("<span class=\"step-icon\">");
        html.push_str(match step.status.as_str() {
            "completed" => "✅",
            "running" => "🔄",
            "failed" => "❌",
            _ => "⏳",
        });
        html.push_str("</span>");
        html.push_str("<span class=\"step-name\">");
        html.push_str(&html_escape(&step.name));
        html.push_str("</span>");
        if let Some(ref detail) = step.detail {
            html.push_str("<span class=\"step-detail\">");
            html.push_str(&html_escape(detail));
            html.push_str("</span>");
        }
        html.push_str("</div>");
    }

    html.push_str("</div></div>");

    html
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
