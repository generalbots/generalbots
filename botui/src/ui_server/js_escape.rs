//! Safe interpolation of server-side values into inline `<script>` blocks.
//!
//! `format!("window.X = \"{}\"", value)` breaks out of the string literal when
//! the value comes from an environment variable or a query parameter: a quote,
//! a backslash, `</script>` or U+2028/2029 can inject script into the page.
//! Every injected value must go through [`js_string`] first.

/// Escape `value` so it is inert inside a double-quoted JavaScript string.
///
/// Non-ASCII characters are emitted as `\uXXXX` (surrogate pairs for code
/// points above the BMP), which also removes any chance of a literal
/// `</script>` or line-separator ending the HTML script element.
pub fn js_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 8);
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{08}' => out.push_str("\\b"),
            '\u{0c}' => out.push_str("\\f"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            c if c.is_ascii_graphic() || c == ' ' => out.push(c),
            c => {
                let code = u32::from(c);
                if code <= 0xffff {
                    out.push_str(&format!("\\u{code:04x}"));
                } else {
                    let adjusted = code - 0x1_0000;
                    let high = 0xd800 + (adjusted >> 10);
                    let low = 0xdc00 + (adjusted & 0x3ff);
                    out.push_str(&format!("\\u{high:04x}\\u{low:04x}"));
                }
            }
        }
    }
    out
}

/// Escape `value` for use inside a quoted HTML attribute. Bot names are also
/// validated by `is_valid_bot_name`; this is defence in depth for the `<base>`
/// tag where a quote must never terminate the attribute.
pub fn html_attr(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 8);
    for ch in value.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::js_string;

    #[test]
    fn escapes_script_breakout_payloads() {
        let result = js_string("a\";alert(1);//");
        assert!(result.contains("\\\""));
        assert!(!result.contains("\";alert"));
        let close_tag = js_string("</script><img src=x onerror=alert(1)>");
        assert!(!close_tag.contains("</script>"));
        assert!(close_tag.contains("\\u003c"));
    }

    #[test]
    fn escapes_line_separators_and_backslashes() {
        assert_eq!(js_string("a\\b"), "a\\\\b");
        assert_eq!(js_string("\u{2028}\u{2029}"), "\\u2028\\u2029");
        assert_eq!(js_string("line\nfeed"), "line\\nfeed");
    }

    #[test]
    fn keeps_ordinary_urls_readable() {
        assert_eq!(js_string("http://localhost:5000"), "http://localhost:5000");
    }

    #[test]
    fn html_attributes_cannot_break_out() {
        assert_eq!(
            html_attr("/x\"><script>/"),
            "/x&quot;&gt;&lt;script&gt;/"
        );
    }
}
