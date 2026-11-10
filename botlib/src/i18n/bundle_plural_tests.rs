//! Plural coverage for the shipped catalogues (#2087).
//!
//! The bundle used to slice a plural block at the first `}`, which is the
//! `{ $count }` placeholder inside the first variant: every plural message
//! rendered empty. These tests walk every plural block in the catalogue and
//! assert a real form is selected for representative counts.

use std::fs;
use std::path::PathBuf;

use super::{I18nBundle, MessageArgs};

/// Extracts every `{ $var -> ... }` plural block from one FTL source.
fn plural_blocks(source: &str) -> Vec<(String, String)> {
    let bytes: Vec<char> = source.chars().collect();
    let mut blocks = Vec::new();
    let mut index = 0usize;

    while index < bytes.len() {
        if bytes[index] == '{' {
            let rest: String = bytes[index..].iter().take(24).collect();
            if let Some(arrow) = rest.find("->") {
                let head = &rest[..arrow];
                if head.contains('$') {
                    if let Some(end) = matching_brace(&bytes, index) {
                        let block: String = bytes[index..=end].iter().collect();
                        blocks.push((head.trim().to_string(), block));
                        index = end + 1;
                        continue;
                    }
                }
            }
        }
        index += 1;
    }

    blocks
}

fn matching_brace(chars: &[char], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (offset, ch) in chars[open..].iter().enumerate() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(open + offset);
                }
            }
            _ => {}
        }
    }
    None
}

fn locale_sources(locale: &str) -> Vec<String> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("locales")
        .join(locale);

    let mut sources = Vec::new();
    let Ok(entries) = fs::read_dir(&dir) else {
        return sources;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("ftl") {
            continue;
        }
        if let Ok(content) = fs::read_to_string(&path) {
            sources.push(content);
        }
    }
    sources
}

fn args_for(value: i64) -> MessageArgs {
    let mut args = MessageArgs::new();
    args.insert("count".to_string(), value.to_string());
    args
}

#[test]
fn every_english_plural_block_renders_for_representative_counts() {
    let mut checked = 0usize;

    for source in locale_sources("en") {
        for (head, block) in plural_blocks(&source) {
            for count in [0i64, 1, 2, 5] {
                let rendered = I18nBundle::interpolate(&block, Some(&args_for(count)));

                assert!(
                    !rendered.trim().is_empty(),
                    "plural block {head} rendered empty for count {count}"
                );
                assert!(
                    !rendered.contains("->"),
                    "plural block {head} kept its selector for count {count}: {rendered}"
                );
                assert!(
                    !rendered.contains("[other]") && !rendered.contains("[one]"),
                    "plural block {head} kept a variant label for count {count}: {rendered}"
                );
            }
            checked += 1;
        }
    }

    assert!(
        checked >= 20,
        "expected the catalogue's plural blocks, found only {checked}"
    );
}

#[test]
fn plural_blocks_substitute_the_count() {
    let template = "{ $count ->\n    [one] One item\n   *[other] { $count } items\n}";

    assert_eq!(
        I18nBundle::interpolate(template, Some(&args_for(7))),
        "7 items"
    );
    assert_eq!(
        I18nBundle::interpolate(template, Some(&args_for(1))),
        "One item"
    );
}

#[test]
fn locales_without_a_one_form_fall_back_to_other() {
    // zh-CN/ja have no grammatical plural: the `*[other]` variant must win for
    // every count instead of leaving the block unrendered.
    let template = "{ $count ->\n    [one] 1 项\n   *[other] { $count } 项\n}";

    for count in [0i64, 1, 2, 5] {
        let rendered = I18nBundle::interpolate(template, Some(&args_for(count)));
        assert!(
            rendered.contains(&count.to_string()),
            "count {count} did not select *[other]: {rendered}"
        );
    }
}

#[test]
fn plural_block_without_zero_form_uses_other() {
    let template = "{ $count ->\n    [one] One\n   *[other] Many\n}";

    assert_eq!(I18nBundle::interpolate(template, Some(&args_for(0))), "Many");
}
