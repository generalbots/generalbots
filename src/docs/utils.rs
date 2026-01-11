use chrono::{DateTime, Duration, Utc};

pub fn format_document_list_item(
    id: &str,
    title: &str,
    updated_at: DateTime<Utc>,
    word_count: usize,
) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "title": title,
        "updated_at": updated_at.to_rfc3339(),
        "updated_relative": format_relative_time(updated_at),
        "word_count": word_count
    })
}

pub fn format_document_content(
    id: &str,
    title: &str,
    content: &str,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
) -> serde_json::Value {
    serde_json::json!({
        "id": id,
        "title": title,
        "content": content,
        "created_at": created_at.to_rfc3339(),
        "updated_at": updated_at.to_rfc3339(),
        "word_count": count_words(content)
    })
}

pub fn format_error(message: &str) -> serde_json::Value {
    serde_json::json!({
        "error": message,
        "success": false
    })
}

pub fn format_relative_time(dt: DateTime<Utc>) -> String {
    let now = Utc::now();
    let diff = now.signed_duration_since(dt);

    if diff < Duration::minutes(1) {
        "just now".to_string()
    } else if diff < Duration::hours(1) {
        let mins = diff.num_minutes();
        format!("{} minute{} ago", mins, if mins == 1 { "" } else { "s" })
    } else if diff < Duration::days(1) {
        let hours = diff.num_hours();
        format!("{} hour{} ago", hours, if hours == 1 { "" } else { "s" })
    } else if diff < Duration::days(7) {
        let days = diff.num_days();
        format!("{} day{} ago", days, if days == 1 { "" } else { "s" })
    } else if diff < Duration::days(30) {
        let weeks = diff.num_weeks();
        format!("{} week{} ago", weeks, if weeks == 1 { "" } else { "s" })
    } else {
        dt.format("%b %d, %Y").to_string()
    }
}

pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#x27;")
}

pub fn strip_html(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }

    result
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
}

pub fn html_to_markdown(html: &str) -> String {
    let mut md = html.to_string();

    md = md.replace("<strong>", "**").replace("</strong>", "**");
    md = md.replace("<b>", "**").replace("</b>", "**");
    md = md.replace("<em>", "*").replace("</em>", "*");
    md = md.replace("<i>", "*").replace("</i>", "*");
    md = md.replace("<u>", "_").replace("</u>", "_");
    md = md.replace("<h1>", "# ").replace("</h1>", "\n");
    md = md.replace("<h2>", "## ").replace("</h2>", "\n");
    md = md.replace("<h3>", "### ").replace("</h3>", "\n");
    md = md.replace("<h4>", "#### ").replace("</h4>", "\n");
    md = md.replace("<h5>", "##### ").replace("</h5>", "\n");
    md = md.replace("<h6>", "###### ").replace("</h6>", "\n");
    md = md.replace("<br>", "\n").replace("<br/>", "\n").replace("<br />", "\n");
    md = md.replace("<p>", "").replace("</p>", "\n\n");
    md = md.replace("<li>", "- ").replace("</li>", "\n");
    md = md.replace("<ul>", "").replace("</ul>", "\n");
    md = md.replace("<ol>", "").replace("</ol>", "\n");
    md = md.replace("<blockquote>", "> ").replace("</blockquote>", "\n");
    md = md.replace("<code>", "`").replace("</code>", "`");
    md = md.replace("<pre>", "```\n").replace("</pre>", "\n```\n");
    md = md.replace("<hr>", "\n---\n").replace("<hr/>", "\n---\n");

    strip_html(&md)
}

pub fn markdown_to_html(md: &str) -> String {
    let mut html = String::new();
    let lines: Vec<&str> = md.lines().collect();
    let mut in_code_block = false;
    let mut in_list = false;

    for line in lines {
        if line.starts_with("```") {
            if in_code_block {
                html.push_str("</pre>");
                in_code_block = false;
            } else {
                html.push_str("<pre>");
                in_code_block = true;
            }
            continue;
        }

        if in_code_block {
            html.push_str(&html_escape(line));
            html.push('\n');
            continue;
        }

        let processed = process_markdown_line(line);

        if line.starts_with("- ") || line.starts_with("* ") {
            if !in_list {
                html.push_str("<ul>");
                in_list = true;
            }
            html.push_str(&format!("<li>{}</li>", &processed[2..]));
        } else {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            html.push_str(&processed);
        }
    }

    if in_list {
        html.push_str("</ul>");
    }
    if in_code_block {
        html.push_str("</pre>");
    }

    html
}

fn process_markdown_line(line: &str) -> String {
    let mut result = line.to_string();

    if line.starts_with("# ") {
        return format!("<h1>{}</h1>", &line[2..]);
    } else if line.starts_with("## ") {
        return format!("<h2>{}</h2>", &line[3..]);
    } else if line.starts_with("### ") {
        return format!("<h3>{}</h3>", &line[4..]);
    } else if line.starts_with("#### ") {
        return format!("<h4>{}</h4>", &line[5..]);
    } else if line.starts_with("##### ") {
        return format!("<h5>{}</h5>", &line[6..]);
    } else if line.starts_with("###### ") {
        return format!("<h6>{}</h6>", &line[7..]);
    } else if line.starts_with("> ") {
        return format!("<blockquote>{}</blockquote>", &line[2..]);
    } else if line == "---" || line == "***" || line == "___" {
        return "<hr>".to_string();
    }

    result = process_inline_formatting(&result);

    if !result.is_empty() && !result.starts_with('<') {
        result = format!("<p>{}</p>", result);
    }

    result
}

fn process_inline_formatting(text: &str) -> String {
    let mut result = text.to_string();

    let bold_re = regex::Regex::new(r"\*\*(.+?)\*\*").ok();
    if let Some(re) = bold_re {
        result = re.replace_all(&result, "<strong>$1</strong>").to_string();
    }

    let italic_re = regex::Regex::new(r"\*(.+?)\*").ok();
    if let Some(re) = italic_re {
        result = re.replace_all(&result, "<em>$1</em>").to_string();
    }

    let code_re = regex::Regex::new(r"`(.+?)`").ok();
    if let Some(re) = code_re {
        result = re.replace_all(&result, "<code>$1</code>").to_string();
    }

    let link_re = regex::Regex::new(r"\[(.+?)\]\((.+?)\)").ok();
    if let Some(re) = link_re {
        result = re.replace_all(&result, r#"<a href="$2">$1</a>"#).to_string();
    }

    result
}

pub fn count_words(text: &str) -> usize {
    let plain_text = strip_html(text);
    plain_text
        .split_whitespace()
        .filter(|s| !s.is_empty())
        .count()
}

pub fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        return text.to_string();
    }

    let truncated: String = text.chars().take(max_chars).collect();
    if let Some(last_space) = truncated.rfind(' ') {
        format!("{}...", &truncated[..last_space])
    } else {
        format!("{}...", truncated)
    }
}

pub fn sanitize_filename(name: &str) -> String {
    name.chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else if c == ' ' {
                '_'
            } else {
                '_'
            }
        })
        .collect::<String>()
        .trim_matches('_')
        .to_string()
}

pub fn generate_document_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

pub fn get_user_docs_path(user_id: &str) -> String {
    format!("users/{}/docs", user_id)
}
