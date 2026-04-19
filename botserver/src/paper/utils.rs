use chrono::{DateTime, Utc};
use crate::core::urls::ApiUrls;

pub fn format_document_list_item(id: &str, title: &str, time: &str, is_new: bool) -> String {
    let mut html = String::new();
    let new_class = if is_new { " new-item" } else { "" };

    html.push_str("<div class=\"paper-item");
    html.push_str(new_class);
    html.push_str("\" data-id=\"");
    html.push_str(&html_escape(id));
    html.push_str("\" hx-get=\"");
    html.push_str(&ApiUrls::PAPER_BY_ID.replace(":id", &html_escape(id)));
    html.push_str("\" hx-target=\"#editor-content\" hx-swap=\"innerHTML\">");
    html.push_str("<div class=\"paper-item-icon\">📄</div>");
    html.push_str("<div class=\"paper-item-info\">");
    html.push_str("<span class=\"paper-item-title\">");
    html.push_str(&html_escape(title));
    html.push_str("</span>");
    html.push_str("<span class=\"paper-item-time\">");
    html.push_str(&html_escape(time));
    html.push_str("</span>");
    html.push_str("</div>");
    html.push_str("</div>");

    html
}

pub fn format_document_content(title: &str, content: &str) -> String {
    let mut html = String::new();

    html.push_str("<div class=\"paper-editor\" data-title=\"");
    html.push_str(&html_escape(title));
    html.push_str("\">");
    html.push_str(
        "<div class=\"paper-title\" contenteditable=\"true\" data-placeholder=\"Untitled\">",
    );
    html.push_str(&html_escape(title));
    html.push_str("</div>");
    html.push_str("<div class=\"paper-body\" contenteditable=\"true\">");
    if content.is_empty() {
        html.push_str("<p data-placeholder=\"Start writing...\"></p>");
    } else {
        html.push_str(&markdown_to_html(content));
    }
    html.push_str("</div>");
    html.push_str("</div>");

    html
}

pub fn format_ai_response(content: &str) -> String {
    let mut html = String::new();

    html.push_str("<div class=\"ai-response\">");
    html.push_str("<div class=\"ai-response-header\">");
    html.push_str("<span class=\"ai-icon\"></span>");
    html.push_str("<span>AI Response</span>");
    html.push_str("</div>");
    html.push_str("<div class=\"ai-response-content\">");
    html.push_str(&html_escape(content));
    html.push_str("</div>");
    html.push_str("<div class=\"ai-response-actions\">");
    html.push_str("<button class=\"btn-copy\" onclick=\"copyAiResponse(this)\">Copy</button>");
    html.push_str(
        "<button class=\"btn-insert\" onclick=\"insertAiResponse(this)\">Insert</button>",
    );
    html.push_str(
        "<button class=\"btn-replace\" onclick=\"replaceWithAiResponse(this)\">Replace</button>",
    );
    html.push_str("</div>");
    html.push_str("</div>");

    html
}

pub fn format_error(message: &str) -> String {
    let mut html = String::new();
    html.push_str("<div class=\"error-message\">");
    html.push_str("<span class=\"error-icon\"></span>");
    html.push_str("<span>");
    html.push_str(&html_escape(message));
    html.push_str("</span>");
    html.push_str("</div>");
    html
}

pub fn format_relative_time(time: DateTime<Utc>) -> String {
    let now = Utc::now();
    let duration = now.signed_duration_since(time);

    if duration.num_seconds() < 60 {
        "just now".to_string()
    } else if duration.num_minutes() < 60 {
        format!("{}m ago", duration.num_minutes())
    } else if duration.num_hours() < 24 {
        format!("{}h ago", duration.num_hours())
    } else if duration.num_days() < 7 {
        format!("{}d ago", duration.num_days())
    } else {
        time.format("%b %d").to_string()
    }
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

pub fn markdown_to_html(markdown: &str) -> String {
    let mut html = String::new();
    let mut in_list = false;
    let mut in_code_block = false;

    for line in markdown.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            if in_code_block {
                html.push_str("</code></pre>");
                in_code_block = false;
            } else {
                html.push_str("<pre><code>");
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            html.push_str(&html_escape(line));
            html.push('\n');
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("# ") {
            html.push_str("<h1>");
            html.push_str(&html_escape(rest));
            html.push_str("</h1>");
        } else if let Some(rest) = trimmed.strip_prefix("## ") {
            html.push_str("<h2>");
            html.push_str(&html_escape(rest));
            html.push_str("</h2>");
        } else if let Some(rest) = trimmed.strip_prefix("### ") {
            html.push_str("<h3>");
            html.push_str(&html_escape(rest));
            html.push_str("</h3>");
        } else if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
            if !in_list {
                html.push_str("<ul class=\"todo-list\">");
                in_list = true;
            }
            html.push_str("<li><input type=\"checkbox\"> ");
            html.push_str(&html_escape(rest));
            html.push_str("</li>");
        } else if let Some(rest) = trimmed.strip_prefix("- [x] ") {
            if !in_list {
                html.push_str("<ul class=\"todo-list\">");
                in_list = true;
            }
            html.push_str("<li><input type=\"checkbox\" checked> ");
            html.push_str(&html_escape(rest));
            html.push_str("</li>");
        } else if let Some(rest) = trimmed.strip_prefix("- ") {
            if !in_list {
                html.push_str("<ul>");
                in_list = true;
            }
            html.push_str("<li>");
            html.push_str(&html_escape(rest));
            html.push_str("</li>");
        } else if let Some(rest) = trimmed.strip_prefix("* ") {
            if !in_list {
                html.push_str("<ul>");
                in_list = true;
            }
            html.push_str("<li>");
            html.push_str(&html_escape(rest));
            html.push_str("</li>");
        } else if trimmed
            .chars()
            .next()
            .map(|c| c.is_ascii_digit())
            .unwrap_or(false)
            && trimmed.contains(". ")
        {
            if !in_list {
                html.push_str("<ol>");
                in_list = true;
            }
            if let Some(pos) = trimmed.find(". ") {
                html.push_str("<li>");
                html.push_str(&html_escape(&trimmed[pos + 2..]));
                html.push_str("</li>");
            }
        } else if trimmed.is_empty() {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            html.push_str("<br>");
        } else {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            html.push_str("<p>");
            let formatted = format_inline_markdown(trimmed);
            html.push_str(&formatted);
            html.push_str("</p>");
        }
    }

    if in_list {
        html.push_str("</ul>");
    }
    if in_code_block {
        html.push_str("</code></pre>");
    }

    html
}

fn format_inline_markdown(text: &str) -> String {
    let escaped = html_escape(text);

    let re_bold = escaped.replace("**", "<b>").replace("__", "<b>");

    let re_italic = re_bold.replace(['*', '_'], "<i>");

    let mut result = String::new();
    let mut in_code = false;
    for ch in re_italic.chars() {
        if ch == '`' {
            if in_code {
                result.push_str("</code>");
            } else {
                result.push_str("<code>");
            }
            in_code = !in_code;
        } else {
            result.push(ch);
        }
    }

    result
}

pub fn strip_markdown(markdown: &str) -> String {
    let mut result = String::new();

    for line in markdown.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            continue;
        }

        let content = if let Some(rest) = trimmed.strip_prefix("### ") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("## ") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("# ") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("- [ ] ") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("- [x] ") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("- ") {
            rest
        } else if let Some(rest) = trimmed.strip_prefix("* ") {
            rest
        } else {
            trimmed
        };

        let clean = content
            .replace("**", "")
            .replace("__", "")
            .replace(['*', '_', '`'], "");

        result.push_str(&clean);
        result.push('\n');
    }

    result
}
