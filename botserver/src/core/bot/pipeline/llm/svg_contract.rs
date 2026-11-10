//! #2108 — the one-fenced-`svg`-block drawing contract, enforced.
//!
//! `system_prompt::canvas_drawing_block` *asks* the model for exactly one
//! fenced ```svg block on a drawing turn. A small model ignores it: it answers
//! with prose plus an inline `<svg>` (or with the markup HTML-escaped), the
//! chat renderer never recognises a block, and the markdown sanitizer escapes
//! the tags — the user reads the markup instead of seeing the drawing.
//!
//! This module is the missing half of that contract. The reply is normalised
//! here, after it is assembled and before `convert_markdown` and the client, so
//! `chat-live-artifact.js` keeps its existing detection untouched: it stays the
//! consumer of the fence, never the repairer of a broken reply.
//!
//! Two invariants:
//! - a fenced block that is *not* a drawing (```js, ```python, a block that
//!   merely mentions `<svg`) is returned byte-for-byte unchanged;
//! - when a drawing is present the reply leaves with exactly one ```svg block
//!   and that block is the tail of the message, which is what the client's
//!   fence parser expects.

/// A `<svg></svg>` stub is not a reply; shorter bodies are not drawings.
const MIN_DRAWING_BYTES: usize = 12;

struct Fence {
    body: String,
    start: usize,
    end: usize,
}

/// Rewrites `reply` so a drawing reaches the client as one ```svg block.
///
/// `drawing_turn` comes from `system_prompt::wants_drawing(user_text)`: on a
/// drawing turn any SVG found is promoted; on an ordinary turn only a drawing
/// that outweighs its surrounding prose is, so an explanatory answer about SVG
/// markup is not turned into an artifact.
pub(super) fn enforce(reply: &str, drawing_turn: bool) -> String {
    if reply.trim().is_empty() {
        return reply.to_string();
    }
    if let Some(fence) = first_fence(reply) {
        if !is_svg_document(&fence.body) {
            return reply.to_string();
        }
        let prose = prose_of(&reply[..fence.start], &reply[fence.end..]);
        return assemble(&prose, &fence.body);
    }
    let Some((start, end, body)) = find_svg_document(reply) else {
        return reply.to_string();
    };
    let prose = prose_of(&reply[..start], &reply[end..]);
    if !drawing_turn && !dominates(&body, &prose) {
        return reply.to_string();
    }
    assemble(&prose, &body)
}

fn assemble(prose: &str, body: &str) -> String {
    let block = format!("```svg\n{}\n```", body.trim());
    if prose.trim().is_empty() {
        block
    } else {
        format!("{}\n\n{}", prose.trim(), block)
    }
}

fn prose_of(before: &str, after: &str) -> String {
    match (before.trim(), after.trim()) {
        ("", "") => String::new(),
        ("", tail) => tail.to_string(),
        (head, "") => head.to_string(),
        (head, tail) => format!("{head}\n\n{tail}"),
    }
}

/// The client's own rule (`dominantSvg`): the drawing wins when it outweighs
/// the prose around it.
fn dominates(svg: &str, prose: &str) -> bool {
    let outside = prose.chars().filter(|c| !c.is_whitespace()).count();
    outside <= svg.chars().count()
}

/* --------------------------------------------------------------- fences */

fn lines_with_offsets(text: &str) -> Vec<(usize, &str)> {
    let mut out = Vec::new();
    let mut offset = 0;
    for raw in text.split('\n') {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        out.push((offset, line));
        offset += raw.len() + 1;
    }
    out
}

/// The first fenced block in the reply, whether or not it is closed.
fn first_fence(text: &str) -> Option<Fence> {
    let lines = lines_with_offsets(text);
    for (index, (offset, line)) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        let Some(marker_char) = trimmed.chars().next() else {
            continue;
        };
        if marker_char != '`' && marker_char != '~' {
            continue;
        }
        let run = trimmed.chars().take_while(|c| *c == marker_char).count();
        if run < 3 {
            continue;
        }
        let marker = &trimmed[..run];
        let body_start = skip_to_next_line(text, *offset + (line.len() - trimmed.len()) + run);
        let mut body_end = text.len();
        let mut end = text.len();
        for (close_offset, close_line) in &lines[index + 1..] {
            if close_line.trim_start().starts_with(marker) {
                body_end = *close_offset;
                end = close_offset + close_line.len();
                break;
            }
        }
        if body_start > body_end {
            return None;
        }
        return Some(Fence {
            body: text[body_start..body_end].trim().to_string(),
            start: *offset,
            end,
        });
    }
    None
}

fn skip_to_next_line(text: &str, from: usize) -> usize {
    match text[from..].find('\n') {
        Some(newline) => from + newline + 1,
        None => text.len(),
    }
}

/// True when the block body is exactly one standalone SVG document: optional
/// XML prolog/doctype, then an `<svg …>` root that closes at the end. This is
/// deliberately strict — it is what separates a drawing from a code block that
/// only talks about `<svg`.
fn is_svg_document(body: &str) -> bool {
    let mut rest = body.trim();
    loop {
        let lower = rest.to_ascii_lowercase();
        if lower.starts_with("<?xml") {
            let Some(end) = rest.find("?>") else {
                return false;
            };
            rest = rest[end + 2..].trim_start();
            continue;
        }
        if lower.starts_with("<!doctype") {
            let Some(end) = rest.find('>') else {
                return false;
            };
            rest = rest[end + 1..].trim_start();
            continue;
        }
        break;
    }
    let trimmed = rest.trim_end();
    let lower = trimmed.to_ascii_lowercase();
    if !lower.starts_with("<svg") || trimmed.len() <= 4 {
        return false;
    }
    if !matches!(trimmed.as_bytes()[4], b' ' | b'\t' | b'\r' | b'\n' | b'>') {
        return false;
    }
    let Some(close) = lower.rfind("</svg") else {
        return false;
    };
    let tail = lower[close + "</svg".len()..].trim();
    tail == ">" && trimmed.chars().count() >= MIN_DRAWING_BYTES
}

/* ------------------------------------------------------------ raw/escaped */

/// Locates an SVG document in free text, raw or HTML-escaped.
fn find_svg_document(text: &str) -> Option<(usize, usize, String)> {
    let lower = text.to_ascii_lowercase();
    if let Some(found) = locate(&lower, text, "<svg", "</svg", false) {
        return Some(found);
    }
    locate(&lower, text, "&lt;svg", "&lt;/svg", true)
}

fn locate(
    lower: &str,
    original: &str,
    open: &str,
    close: &str,
    escaped: bool,
) -> Option<(usize, usize, String)> {
    let start = lower.find(open)?;
    let after_open = start + open.len();
    let after = &lower[after_open..];
    if !after.starts_with(|c: char| c.is_whitespace() || c == '>' || c == '&') {
        return None;
    }
    let close_at = after_open + after.find(close)?;
    // The escaped form has no literal `>`: its terminator is `&gt;`.
    let terminator = if escaped { "&gt;" } else { ">" };
    let end = close_at + lower[close_at..].find(terminator)? + terminator.len();
    // The prolog, when present, travels inside the fence with the root element.
    let start = include_prolog(original, start);
    let raw = &original[start..end];
    let body = if escaped {
        unescape_markup(raw)
    } else {
        raw.to_string()
    };
    if body.chars().count() < MIN_DRAWING_BYTES {
        return None;
    }
    Some((start, end, body))
}

/// Absorbs an XML prolog/doctype that sits immediately before the root element
/// so it travels inside the fence instead of printing in the bubble.
fn include_prolog(text: &str, start: usize) -> usize {
    let before = text[..start].trim_end();
    let lower = before.to_ascii_lowercase();
    if lower.ends_with("?>") {
        if let Some(at) = lower.rfind("<?xml") {
            return at;
        }
    }
    if lower.ends_with('>') {
        if let Some(at) = lower.rfind("<!doctype") {
            return at;
        }
    }
    start
}

fn unescape_markup(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

#[cfg(test)]
mod svg_contract_tests {
    use super::enforce;

    const DRAWING: &str = "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 8 8\"><rect width=\"8\" height=\"8\"/></svg>";

    fn fences(reply: &str) -> usize {
        reply.matches("```svg").count()
    }

    fn expected_block() -> String {
        format!("```svg\n{DRAWING}\n```")
    }

    #[test]
    fn raw_inline_svg_becomes_exactly_one_fenced_block() {
        let out = enforce(DRAWING, true);
        assert_eq!(out, expected_block());
        assert_eq!(fences(&out), 1);
    }

    #[test]
    fn escaped_svg_is_decoded_and_fenced() {
        let escaped = DRAWING
            .replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
            .replace('"', "&quot;");
        let out = enforce(&escaped, true);
        assert_eq!(out, expected_block());
        assert!(!out.contains("&lt;svg"));
    }

    #[test]
    fn an_svg_fence_is_left_alone() {
        let fenced = expected_block();
        assert_eq!(enforce(&fenced, true), fenced);
        assert_eq!(enforce(&fenced, false), fenced);
    }

    #[test]
    fn an_xml_fence_carrying_a_drawing_is_relabelled_svg() {
        let xml = format!("```xml\n{DRAWING}\n```");
        assert_eq!(enforce(&xml, false), expected_block());
    }

    #[test]
    fn prose_around_the_drawing_is_kept_and_the_block_ends_the_reply() {
        let reply = format!("Here is your robot, drawn in SVG:\n{DRAWING}\nEnjoy!");
        let out = enforce(&reply, true);
        assert!(out.starts_with("Here is your robot"));
        assert!(out.contains("Enjoy!"));
        assert!(out.ends_with(&expected_block()));
        assert_eq!(fences(&out), 1);
    }

    #[test]
    fn a_non_svg_code_block_stays_text() {
        let reply = "The root element is documented below.\n\n```js\nconst root = \"<svg>\";\n```";
        assert_eq!(enforce(reply, true), reply);
        let python = format!("```python\nmarkup = \"\"\"{DRAWING}\"\"\"\n```");
        assert_eq!(enforce(&python, true), python);
    }

    #[test]
    fn an_ordinary_turn_keeps_a_drawing_that_is_not_dominant() {
        let reply = format!("{} {DRAWING}", "SVG is an XML vector format. ".repeat(20));
        assert_eq!(enforce(&reply, false), reply);
        // The same reply on a drawing turn is promoted anyway.
        assert_eq!(fences(&enforce(&reply, true)), 1);
    }

    #[test]
    fn a_prolog_travels_inside_the_fence() {
        let reply = format!("<?xml version=\"1.0\"?>\n{DRAWING}");
        let out = enforce(&reply, true);
        assert!(out.starts_with("```svg\n<?xml"));
        assert!(!out.contains("?>\n\n```svg"));
    }
}
