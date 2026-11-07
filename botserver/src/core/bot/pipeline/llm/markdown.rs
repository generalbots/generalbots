//! Markdown-to-HTML conversion applied to the final answer.

/// Converts the model's `**bold**` and `- ` bullets into the HTML the chat
/// renderer expects. A stray `**` leaves the text untouched.
pub(super) fn convert_markdown(content: &str) -> String {
    let mut converted = content
        .split("**")
        .enumerate()
        .map(|(i, part)| {
            if i % 2 == 1 {
                format!("<strong>{}</strong>", part)
            } else {
                part.to_string()
            }
        })
        .collect::<Vec<_>>()
        .concat();
    let lines: Vec<&str> = converted.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim_start();
        if trimmed.starts_with("- ") {
            out.push("<ul>".to_string());
            while i < lines.len() && lines[i].trim_start().starts_with("- ") {
                let li_content = lines[i]
                    .trim_start()
                    .strip_prefix("- ")
                    .unwrap_or(lines[i].trim_start());
                out.push(format!("<li>{}</li>", li_content));
                i += 1;
            }
            out.push("</ul>".to_string());
        } else {
            out.push(lines[i].to_string());
            i += 1;
        }
    }
    converted = out.join("\n");
    converted
}
