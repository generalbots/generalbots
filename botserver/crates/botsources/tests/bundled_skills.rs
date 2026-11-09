//! #2063 — the bundled skill pack contract.
//!
//! Every packaged skill lives at `botskills/<category>/<slug>/SKILL.md` with
//! YAML front-matter (`name`, `description`, `version`). These tests guard the
//! pack itself and the prompt block that advertises it:
//!
//! * at least 100 skills ship (the issue's own tables sum 96 unique names, and
//!   five pre-existing skills already covered a listed concern);
//! * every package has a usable manifest and a body;
//! * the prompt block carries `name: description` only and stays proportional
//!   to the number of skills, never to their body size.

use std::fs;
use std::path::{Path, PathBuf};

use botsources::handlers::sources_handlers::render_skill_lines;

/// Repository root, from this crate's manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("..")
}

/// Every `SKILL.md` under `botskills/`, sorted for deterministic assertions.
fn skill_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect(&repo_root().join("botskills"), &mut files);
    files.sort();
    files
}

fn collect(directory: &Path, files: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, files);
        } else if path.file_name().and_then(|name| name.to_str()) == Some("SKILL.md") {
            files.push(path);
        }
    }
}

/// Reads one front-matter value, including `key: |` block scalars.
fn front_matter(document: &str, key: &str) -> Option<String> {
    let mut lines = document.lines();
    if lines.next()?.trim() != "---" {
        return None;
    }
    let prefix = format!("{key}:");
    let mut block = false;
    let mut value = String::new();
    for line in lines {
        if line.trim() == "---" {
            break;
        }
        if block {
            if line.starts_with(' ') || line.starts_with('\t') {
                if !value.is_empty() {
                    value.push(' ');
                }
                value.push_str(line.trim());
                continue;
            }
            block = false;
        }
        if let Some(rest) = line.strip_prefix(&prefix) {
            let rest = rest.trim();
            if matches!(rest, "|" | ">" | "|-" | ">-") {
                block = true;
            } else {
                value = rest.trim_matches('"').trim_matches('\'').to_string();
            }
        }
    }
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

/// Splits the body (everything after the closing `---`).
fn body_of(document: &str) -> &str {
    let mut seen_open = false;
    let mut offset = 0;
    for line in document.split_inclusive('\n') {
        if line.trim() == "---" {
            if !seen_open {
                seen_open = true;
            } else {
                return &document[offset + line.len()..];
            }
        }
        offset += line.len();
    }
    ""
}

#[test]
fn the_pack_ships_at_least_one_hundred_skills() {
    let files = skill_files();
    assert!(
        files.len() >= 100,
        "expected at least 100 bundled skills, found {}",
        files.len()
    );
    // The five original categories must survive a pack regeneration.
    for category in [
        "automation",
        "documentation",
        "engineering",
        "frontend",
        "integration",
        "operations",
        "research",
        "testing",
    ] {
        let prefix = repo_root().join("botskills").join(category);
        assert!(
            prefix.is_dir(),
            "the {category}/ category disappeared from botskills/"
        );
    }
    for category in [
        "office", "design", "data", "content", "quality", "business", "media",
    ] {
        assert!(
            repo_root().join("botskills").join(category).is_dir(),
            "the {category}/ pack category is missing"
        );
    }
}

#[test]
fn every_package_has_a_usable_manifest_and_body() {
    let mut problems = Vec::new();
    for file in skill_files() {
        let document = fs::read_to_string(&file).unwrap_or_default();
        let relative = file
            .strip_prefix(repo_root())
            .unwrap_or(&file)
            .display()
            .to_string();
        let name = front_matter(&document, "name");
        let description = front_matter(&document, "description");
        let version = front_matter(&document, "version");
        if name.as_deref().map(str::trim).unwrap_or_default().is_empty() {
            problems.push(format!("{relative}: missing name"));
        }
        match description.as_deref() {
            Some(description) if description.trim().len() >= 20 => {}
            _ => problems.push(format!("{relative}: description missing or too short")),
        }
        if version.is_none() {
            problems.push(format!("{relative}: missing version"));
        }
        if body_of(&document).trim().len() < 100 {
            problems.push(format!("{relative}: body is empty or too short"));
        }
    }
    assert!(problems.is_empty(), "invalid skill packages:\n{}", problems.join("\n"));

    // Slugs are unique: two packages with the same name would shadow each
    // other in `bot_skills` (keyed on name).
    let mut seen = std::collections::HashMap::new();
    for file in skill_files() {
        let document = fs::read_to_string(&file).unwrap_or_default();
        let Some(name) = front_matter(&document, "name") else {
            continue;
        };
        let slug = name.trim().to_lowercase();
        let previous = seen.insert(slug.clone(), file.clone());
        assert!(
            previous.is_none(),
            "duplicate skill name {slug}: {previous:?} and {file:?}"
        );
    }
}

#[test]
fn the_prompt_lists_name_and_description_without_bodies() {
    let pairs: Vec<(String, Option<String>)> = skill_files()
        .iter()
        .filter_map(|file| {
            let document = fs::read_to_string(file).ok()?;
            Some((front_matter(&document, "name")?, front_matter(&document, "description")))
        })
        .collect();
    let block = render_skill_lines(pairs.iter().cloned());
    assert!(block.contains("skills.find"), "the block must point at skills.find");
    for (name, description) in &pairs {
        assert!(
            block.contains(&format!("- {name}: ")),
            "{name} is not advertised in the prompt block"
        );
        if let Some(description) = description {
            let head: String = description.trim().chars().take(40).collect();
            assert!(
                block.contains(&head),
                "{name}'s description is not advertised"
            );
        }
        // A body marker would mean the package content leaked into the prompt.
        assert!(!block.contains("## Workflow"), "package bodies must not be inlined");
        assert!(
            !block.contains("## Quality checks"),
            "package bodies must not be inlined"
        );
    }
    // One short line per skill: ~350 bytes each including the header/footer.
    let budget = 700 + pairs.len() * 400;
    assert!(
        block.len() <= budget,
        "prompt block is {} bytes for {} skills (budget {budget})",
        block.len(),
        pairs.len()
    );
}

#[test]
fn the_block_scales_with_skill_count_not_body_size() {
    let one = render_skill_lines(
        vec![("alpha".to_string(), Some("A short description.".to_string()))].into_iter(),
    );
    let many: Vec<(String, Option<String>)> = (0..150)
        .map(|index| {
            (
                format!("skill-{index}"),
                Some("A short description of this method.".to_string()),
            )
        })
        .collect();
    let block = render_skill_lines(many.into_iter());
    // 150 lines of a fixed shape: the size tracks the number of skills and
    // nothing else, so a 101st install adds one line, not a body.
    assert_eq!(block.matches("- skill-").count(), 150);
    assert!(block.len() < 150 * 120, "block grew to {} bytes", block.len());
    assert!(block.len() > one.len());
    assert_eq!(one.matches("- alpha: ").count(), 1);
}
