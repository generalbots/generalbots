//! HTML <-> Markdown conversion used by the Docs import/export paths.
//!
//! Split out of `utils_convert.rs` to keep both files under the 450-line limit.

use super::utils_format::{html_escape, strip_html};
use crate::re_cache::cached_re;

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
    md = md
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n");
    md = md.replace("<p>", "").replace("</p>", "\n\n");
    md = md.replace("<li>", "- ").replace("</li>", "\n");
    md = md.replace("<ul>", "").replace("</ul>", "\n");
    md = md.replace("<ol>", "").replace("</ol>", "\n");
    md = md
        .replace("<blockquote>", "> ")
        .replace("</blockquote>", "\n");
    md = md.replace("<code>", "`").replace("</code>", "`");
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

        if line.starts_with("- ") || line.starts_with("* ") {
            if !in_list {
                html.push_str("<ul>");
                in_list = true;
            }
            // Format the item text, not the rendered block: slicing the block
            // output used to leave `>- a</p>` inside the `<li>`.
            let item = process_inline_formatting(&line[2..]);
            html.push_str(&format!("<li>{item}</li>"));
        } else {
            if in_list {
                html.push_str("</ul>");
                in_list = false;
            }
            html.push_str(&process_markdown_line(line));
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

    // Ordered longest-first so `### ` never matches the `# ` rule.
    const HEADINGS: [(&str, &str); 6] = [
        ("###### ", "h6"),
        ("##### ", "h5"),
        ("#### ", "h4"),
        ("### ", "h3"),
        ("## ", "h2"),
        ("# ", "h1"),
    ];
    for (prefix, tag) in HEADINGS {
        if let Some(text) = line.strip_prefix(prefix) {
            return format!("<{tag}>{text}</{tag}>");
        }
    }
    if let Some(text) = line.strip_prefix("> ") {
        return format!("<blockquote>{text}</blockquote>");
    }
    if line == "---" || line == "***" || line == "___" {
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

    let bold_re = cached_re!(r"\*\*(.+?)\*\*");
    if let Some(re) = bold_re {
        result = re.replace_all(&result, "<strong>$1</strong>").to_string();
    }

    let italic_re = cached_re!(r"\*(.+?)\*");
    if let Some(re) = italic_re {
        result = re.replace_all(&result, "<em>$1</em>").to_string();
    }

    let code_re = cached_re!(r"`(.+?)`");
    if let Some(re) = code_re {
        result = re.replace_all(&result, "<code>$1</code>").to_string();
    }

    let link_re = cached_re!(r"\[(.+?)\]\((.+?)\)");
    if let Some(re) = link_re {
        result = re
            .replace_all(&result, r#"<a href="$2">$1</a>"#)
            .to_string();
    }

    result
}
