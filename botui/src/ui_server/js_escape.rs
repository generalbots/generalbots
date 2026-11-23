//! Safe interpolation of server-side values into inline `<script>` blocks.
//!
//! `format!("window.X = \"{}\"", value)` breaks out of the string literal when
//! the value comes from an environment variable or a query parameter: a quote,
//! a backslash, `<`/`>`/`&` (which can form `</script>`), or U+2028/2029 can
//! inject script into the page. Every injected value must go through
//! [`js_string`], or — preferably — through [`js_window_script`], which also
//! emits the assignment for the caller.

/// Escape `value` so it is inert inside a double-quoted JavaScript string.
///
/// Quotes, backslashes and control characters use JS escapes. `<`, `>` and
/// `&` are escaped to `\uXXXX` as well (even though they are printable ASCII),
/// so no substring can close the HTML `<script>` element or open an entity.
/// Non-ASCII characters (including U+2028/2029) are emitted as `\uXXXX`
/// (surrogate pairs for code points above the BMP).
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
            '<' => out.push_str("\\u003c"),
            '>' => out.push_str("\\u003e"),
            '&' => out.push_str("\\u0026"),
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

/// Build one inline `<script>` block assigning escaped strings to
/// `window.<name>`. This is the single helper the cloud, suite and login
/// servers use for their config injection — callers must never build the
/// assignment with `format!` themselves.
///
/// `name` must be a hardcoded JavaScript identifier (all call sites pass
/// literals); only `value` may come from an environment variable.
pub fn js_window_script(assignments: &[(&str, &str)]) -> String {
    let mut out = String::from("<script>");
    for (name, value) in assignments {
        out.push_str(&format!("window.{name} = \"{}\";", js_string(value)));
    }
    out.push_str("</script>");
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
    use super::{html_attr, js_string, js_window_script};

    #[test]
    fn escapes_script_breakout_payloads() {
        let result = js_string("a\";alert(1);//");
        assert!(result.contains("\\\""));
        // The quote is escaped to `\"`; the raw prefix must not survive.
        assert!(!result.contains("a\";alert"));
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
    fn escapes_angle_brackets_and_ampersands() {
        assert_eq!(js_string("</script>"), "\\u003c/script\\u003e");
        assert_eq!(js_string("a&b"), "a\\u0026b");
        assert!(!js_string("</script><img src=x>").contains('<'));
    }

    #[test]
    fn keeps_ordinary_urls_readable() {
        assert_eq!(js_string("http://localhost:5000"), "http://localhost:5000");
    }

    #[test]
    fn window_script_keeps_hostile_values_inert() {
        // Values arriving from env vars: a quote, a closing script tag and
        // U+2028/2029. None may terminate the string or the script element.
        for hostile in [
            "\";alert(1);//",
            "</script><img src=x onerror=alert(1)>",
            "a\u{2028}b\u{2029}c",
            "&<>\\\"",
        ] {
            let script = js_window_script(&[("GB_TEST", hostile)]);
            assert!(script.starts_with("<script>window.GB_TEST = \""), "{script}");
            assert!(script.ends_with("\";</script>"), "{script}");
            assert_eq!(script.matches("</script>").count(), 1, "{script}");
            assert!(!script.contains("<img"), "{script}");
            assert!(!script.contains('\u{2028}'), "{script}");
            assert!(!script.contains('\u{2029}'), "{script}");
        }
    }

    #[test]
    fn window_script_assigns_every_value_in_one_block() {
        assert_eq!(
            js_window_script(&[("A", "one"), ("B", "two")]),
            "<script>window.A = \"one\";window.B = \"two\";</script>"
        );
    }

    #[test]
    fn html_attributes_cannot_break_out() {
        assert_eq!(
            html_attr("/x\"><script>/"),
            "/x&quot;&gt;&lt;script&gt;/"
        );
    }
}
