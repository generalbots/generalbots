use chrono::{DateTime, Utc};

pub fn get_default_files() -> Vec<(String, String, DateTime<Utc>)> {
    vec![
        (
            "welcome".to_string(),
            "Welcome Dialog".to_string(),
            Utc::now(),
        ),
        ("faq".to_string(), "FAQ Bot".to_string(), Utc::now()),
        (
            "support".to_string(),
            "Customer Support".to_string(),
            Utc::now(),
        ),
    ]
}

pub fn get_default_dialog_content() -> String {
    "' Welcome Dialog\n\
     ' Created with Dialog Designer\n\
     \n\
     SUB Main()\n\
     TALK \"Hello! How can I help you today?\"\n\
     \n\
     answer = HEAR\n\
     \n\
     IF answer LIKE \"*help*\" THEN\n\
     TALK \"I'm here to assist you.\"\n\
     ELSE IF answer LIKE \"*bye*\" THEN\n\
     TALK \"Goodbye!\"\n\
     ELSE\n\
     TALK \"I understand: \" + answer\n\
     END IF\n\
     END SUB\n"
        .to_string()
}

pub struct DialogNode {
    pub id: String,
    pub node_type: String,
    pub content: String,
    pub x: i32,
    pub y: i32,
}

pub fn parse_basic_to_nodes(content: &str) -> Vec<DialogNode> {
    let mut nodes = Vec::new();
    let mut y_pos = 100;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('\'') {
            continue;
        }

        let upper = trimmed.to_uppercase();
        let node_type = if upper.starts_with("TALK ") {
            "talk"
        } else if upper.starts_with("HEAR") {
            "hear"
        } else if upper.starts_with("IF ") {
            "if"
        } else if upper.starts_with("FOR ") {
            "for"
        } else if upper.starts_with("SET ") || upper.contains(" = ") {
            "set"
        } else if upper.starts_with("CALL ") {
            "call"
        } else if upper.starts_with("SUB ") {
            "sub"
        } else {
            continue;
        };

        nodes.push(DialogNode {
            id: format!("node-{}", i),
            node_type: node_type.to_string(),
            content: trimmed.to_string(),
            x: 400,
            y: y_pos,
        });

        y_pos += 80;
    }

    nodes
}

pub fn format_node_html(node: &DialogNode) -> String {
    let mut html = String::new();
    html.push_str("<div class=\"canvas-node node-");
    html.push_str(&node.node_type);
    html.push_str("\" id=\"");
    html.push_str(&html_escape(&node.id));
    html.push_str("\" style=\"left: ");
    html.push_str(&node.x.to_string());
    html.push_str("px; top: ");
    html.push_str(&node.y.to_string());
    html.push_str("px;\" draggable=\"true\">");
    html.push_str("<div class=\"node-header\">");
    html.push_str("<span class=\"node-type\">");
    html.push_str(&node.node_type.to_uppercase());
    html.push_str("</span>");
    html.push_str("</div>");
    html.push_str("<div class=\"node-content\">");
    html.push_str(&html_escape(&node.content));
    html.push_str("</div>");
    html.push_str("<div class=\"node-ports\">");
    html.push_str("<div class=\"port port-in\"></div>");
    html.push_str("<div class=\"port port-out\"></div>");
    html.push_str("</div>");
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
