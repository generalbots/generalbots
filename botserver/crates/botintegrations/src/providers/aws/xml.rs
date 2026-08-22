//! Minimal XML helpers for AWS query-protocol responses (#950).
//!
//! AWS STS/EC2/CloudWatch/S3 return small XML documents. These helpers
//! extract element text by ancestor-path suffix and collect repeated child
//! blocks (for example `<Contents>` entries) without building a DOM.
//!
//! Path matching is suffix based: a request for `["Contents", "Key"]` matches
//! `...<ListBucketResult><Contents><Key>` regardless of the document root,
//! which keeps call sites short. Parsing is bounded - nesting depth, matched
//! counts and block counts are capped, and malformed input yields empty
//! results instead of errors.

use std::collections::BTreeMap;

use quick_xml::events::Event;
use quick_xml::Reader;

/// Maximum element nesting accepted while scanning a document.
const MAX_DEPTH: usize = 32;
/// Default cap for leaf-text collection; AWS responses stay far below this.
const MAX_MATCHES: usize = 500;

fn reader_from(xml: &str) -> Reader<&[u8]> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);
    reader
}

fn tag_name(raw: &[u8]) -> String {
    String::from_utf8_lossy(raw).to_string()
}

fn stack_matches(stack: &[String], path: &[&str]) -> bool {
    if path.is_empty() || stack.len() < path.len() {
        return false;
    }
    stack[stack.len() - path.len()..]
        .iter()
        .zip(path)
        .all(|(seen, wanted)| seen == wanted)
}

/// Text content of the first element whose ancestor chain ends with `path`
/// (e.g. `["Error", "Code"]`). Returns `None` when absent or empty.
pub(crate) fn first_text_by_path(xml: &str, path: &[&str]) -> Option<String> {
    let mut reader = reader_from(xml);
    let mut stack: Vec<String> = Vec::new();
    loop {
        match reader.read_event() {
            Ok(Event::Start(start)) => {
                if stack.len() >= MAX_DEPTH {
                    return None;
                }
                stack.push(tag_name(start.name().into_inner()));
                if stack_matches(&stack, path) {
                    return capture_immediate_text(&mut reader);
                }
            }
            Ok(Event::End(_)) => {
                stack.pop();
            }
            Ok(Event::Eof) | Err(_) => return None,
            _ => {}
        }
    }
}

/// Text content of every element whose ancestor chain ends with `path`, in
/// document order, capped at [`MAX_MATCHES`] entries.
pub(crate) fn collect_leaf_texts(xml: &str, path: &[&str], max_matches: usize) -> Vec<String> {
    let limit = max_matches.min(MAX_MATCHES);
    let mut reader = reader_from(xml);
    let mut stack: Vec<String> = Vec::new();
    let mut matches: Vec<String> = Vec::new();
    loop {
        if matches.len() >= limit {
            return matches;
        }
        match reader.read_event() {
            Ok(Event::Start(start)) => {
                if stack.len() >= MAX_DEPTH {
                    return matches;
                }
                stack.push(tag_name(start.name().into_inner()));
                if stack_matches(&stack, path) {
                    match capture_immediate_text(&mut reader) {
                        Some(text) => matches.push(text),
                        None => matches.push(String::new()),
                    }
                }
            }
            Ok(Event::End(_)) => {
                stack.pop();
            }
            Ok(Event::Eof) | Err(_) => return matches,
            _ => {}
        }
    }
}

fn capture_immediate_text(reader: &mut Reader<&[u8]>) -> Option<String> {
    loop {
        match reader.read_event() {
            Ok(Event::Text(text)) => {
                let decoded = text.unescape().ok()?;
                return Some(decoded.to_string()).filter(|value| !value.is_empty());
            }
            Ok(Event::End(_)) | Ok(Event::Eof) | Err(_) => return None,
            _ => {}
        }
    }
}

/// Collects up to `max_blocks` `<child_name>` elements that appear directly
/// inside `<parent_name>` elements into ordered maps of direct child name to
/// concatenated text. Nested markup inside a child is skipped rather than
/// interpreted; documents with unmatched tags terminate the scan cleanly.
pub(crate) fn collect_child_blocks(
    xml: &str,
    parent_name: &str,
    child_name: &str,
    max_blocks: usize,
) -> Vec<BTreeMap<String, String>> {
    let mut reader = reader_from(xml);
    let mut blocks: Vec<BTreeMap<String, String>> = Vec::new();
    let mut depth = 0usize;
    let mut parent_depth = 0usize;
    let mut inside_parent = false;
    loop {
        if blocks.len() >= max_blocks {
            return blocks;
        }
        match reader.read_event() {
            Ok(Event::Start(start)) => {
                depth += 1;
                if depth > MAX_DEPTH {
                    return blocks;
                }
                let name = tag_name(start.name().into_inner());
                if name == parent_name {
                    inside_parent = true;
                    parent_depth = depth;
                } else if inside_parent && name == child_name {
                    match read_block_children(&mut reader, &mut depth) {
                        Some(block) => blocks.push(block),
                        None => return blocks,
                    }
                }
            }
            Ok(Event::End(_)) => {
                if depth == 0 {
                    return blocks;
                }
                depth -= 1;
                if inside_parent && depth < parent_depth {
                    inside_parent = false;
                }
            }
            Ok(Event::Eof) | Err(_) => return blocks,
            _ => {}
        }
    }
}

/// Reads the children of an already-entered block until its closing tag.
/// `depth` tracks nesting so the caller's stack stays consistent afterwards.
fn read_block_children(
    reader: &mut Reader<&[u8]>,
    depth: &mut usize,
) -> Option<BTreeMap<String, String>> {
    let mut children: BTreeMap<String, String> = BTreeMap::new();
    let mut current_child: Option<String> = None;
    let mut buffer = String::new();
    let mut skipping_nested = 0usize;
    loop {
        match reader.read_event() {
            Ok(Event::Start(start)) => {
                *depth += 1;
                if *depth > MAX_DEPTH {
                    return None;
                }
                if skipping_nested > 0 {
                    skipping_nested += 1;
                } else if current_child.is_some() {
                    skipping_nested = 1;
                } else {
                    current_child = Some(tag_name(start.name().into_inner()));
                    buffer.clear();
                }
            }
            Ok(Event::Empty(_)) => {}
            Ok(Event::Text(text)) => {
                if skipping_nested == 0 && current_child.is_some() {
                    match text.unescape() {
                        Ok(decoded) => buffer.push_str(&decoded),
                        Err(_) => return None,
                    }
                }
            }
            Ok(Event::End(_)) => {
                if *depth == 0 {
                    return None;
                }
                *depth -= 1;
                if skipping_nested > 0 {
                    skipping_nested -= 1;
                } else if let Some(child) = current_child.take() {
                    children.insert(child, std::mem::take(&mut buffer));
                } else {
                    return Some(children);
                }
            }
            Ok(Event::Eof) | Err(_) => return None,
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_first_matching_path_suffix() {
        let document = "<Response><GetCallerIdentityResult>\
            <Arn>arn:aws:iam::123:user/bot</Arn><Account>123</Account>\
            </GetCallerIdentityResult></Response>";
        assert_eq!(
            first_text_by_path(document, &["GetCallerIdentityResult", "Arn"]).as_deref(),
            Some("arn:aws:iam::123:user/bot")
        );
        assert_eq!(
            first_text_by_path(document, &["Arn"]).as_deref(),
            Some("arn:aws:iam::123:user/bot")
        );
        assert_eq!(first_text_by_path(document, &["Missing", "Arn"]), None);
        assert_eq!(first_text_by_path("<K></K>", &["K"]), None);
    }

    #[test]
    fn collects_leaf_texts_in_document_order() {
        let document = "<R><List><Timestamps><member>t1</member><member>t2</member></Timestamps>\
            <Values><member>1</member><member>2</member></Values></List></R>";
        assert_eq!(
            collect_leaf_texts(document, &["Timestamps", "member"], 100),
            vec!["t1".to_string(), "t2".to_string()]
        );
        assert_eq!(
            collect_leaf_texts(document, &["Values", "member"], 1),
            vec!["1".to_string()]
        );
        assert!(collect_leaf_texts(document, &["Absent", "member"], 10).is_empty());
    }

    #[test]
    fn collects_repeated_child_blocks_with_children() {
        let document = "<ListBucketResult><Contents><Key>a.txt</Key><Size>3</Size></Contents>\
            <Contents><Key>b/c.bin</Key><Size>9</Size></Contents></ListBucketResult>";
        let blocks = collect_child_blocks(document, "ListBucketResult", "Contents", 100);
        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].get("Key").map(String::as_str), Some("a.txt"));
        assert_eq!(blocks[1].get("Size").map(String::as_str), Some("9"));
        let capped = collect_child_blocks(document, "ListBucketResult", "Contents", 1);
        assert_eq!(capped.len(), 1);
    }

    #[test]
    fn skips_nested_markup_inside_children_and_survives_malformed_input() {
        let nested = "<R><Item><Name>x<Nested>junk</Nested>y</Name></Item></R>";
        let blocks = collect_child_blocks(nested, "R", "Item", 10);
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].get("Name").map(String::as_str), Some("xy"));

        assert!(collect_child_blocks("<Broken><Item>", "Broken", "Item", 10).is_empty());
        assert_eq!(first_text_by_path("<Broken", &["A"]), None);
    }

    #[test]
    fn unescapes_entity_references_in_text() {
        let document = "<K>v&lt;1&amp;2</K>";
        assert_eq!(
            first_text_by_path(document, &["K"]).as_deref(),
            Some("v<1&2")
        );
    }
}
