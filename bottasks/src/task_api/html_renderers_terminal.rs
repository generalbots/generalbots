use crate::types::TaskManifest;

pub fn render_terminal(manifest: &TaskManifest) -> String {
    let mut html = String::new();

    html.push_str("<div class=\"task-terminal\">");

    html.push_str("<div class=\"terminal-header\">");
    html.push_str("<span class=\"terminal-dots\">");
    html.push_str("<span class=\"dot dot-red\"></span>");
    html.push_str("<span class=\"dot dot-yellow\"></span>");
    html.push_str("<span class=\"dot dot-green\"></span>");
    html.push_str("</span>");
    html.push_str("<span class=\"terminal-title\">");
    html.push_str(&html_escape(&manifest.title));
    html.push_str("</span>");
    html.push_str("<span class=\"terminal-status ");
    html.push_str(&manifest.status);
    html.push_str("\">");
    html.push_str(&html_escape(&manifest.status));
    html.push_str("</span>");
    html.push_str("</div>");

    html.push_str("<div class=\"terminal-body\">");

    if manifest.terminal_output.is_empty() {
        html.push_str("<div class=\"terminal-line\">No output yet</div>");
    } else {
        for line in &manifest.terminal_output {
            let line_class = match line.line_type.as_deref() {
                Some("error") => "terminal-error",
                Some("warning") => "terminal-warning",
                Some("info") => "terminal-info",
                _ => "terminal-output",
            };

            html.push_str("<div class=\"terminal-line ");
            html.push_str(line_class);
            html.push_str("\">");

            if let Some(ref lt) = line.line_type {
                html.push_str("<span class=\"line-prefix\">[");
                html.push_str(&html_escape(lt));
                html.push_str("]</span> ");
            }

            html.push_str(&html_escape(&line.text));
            html.push_str("</div>");
        }
    }

    if manifest.status == "running" {
        html.push_str("<div class=\"terminal-cursor\">▊</div>");
    }

    html.push_str("</div>");

    html.push_str("<div class=\"terminal-footer\">");
    html.push_str("<span class=\"terminal-stats\">");
    html.push_str(&manifest.terminal_output.len().to_string());
    html.push_str(" lines | ");
    html.push_str(&manifest.processing_stats.total_tokens.to_string());
    html.push_str(" tokens | $");
    html.push_str(&format!("{:.4}", manifest.processing_stats.total_cost));
    html.push_str("</span>");
    html.push_str("<span class=\"terminal-progress\">");
    html.push_str(&manifest.completed_steps.to_string());
    html.push_str("/");
    html.push_str(&manifest.total_steps.to_string());
    html.push_str(" steps</span>");
    html.push_str("</div>");

    html.push_str("</div>");

    html
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
