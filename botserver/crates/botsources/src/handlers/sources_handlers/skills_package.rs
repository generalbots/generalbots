//! SKILL.md package parsing and hashing (#1681). Split from the handlers so
//! the parser stays unit-testable and the handler file stays within the size
//! limit.

use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct SkillPackage {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub instructions: String,
    pub hash: String,
}

fn slug_is_valid(value: &str) -> bool {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) if first.is_ascii_lowercase() || first.is_ascii_digit() => {}
        _ => return false,
    }
    value.len() <= 64
        && value
            .chars()
            .all(|character| character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '-' | '_' | '.'))
}

/// Parses the front-matter and body of one `SKILL.md` document. The parser is
/// deliberately small and forgiving: unknown front-matter keys are ignored,
/// and the body is everything after the closing `---`.
pub fn parse_skill_package(markdown: &str, fallback_name: Option<&str>) -> Result<SkillPackage, String> {
    let normalized = markdown.replace("\r\n", "\n");
    let mut lines = normalized.lines();
    let first = lines.next().unwrap_or_default().trim();
    if first != "---" {
        return Err("SKILL.md must start with a YAML front-matter block (---)".to_string());
    }
    let mut name = String::new();
    let mut description = String::new();
    let mut version = String::new();
    let mut body_started = false;
    let mut body = String::new();
    for line in lines {
        if !body_started {
            if line.trim() == "---" {
                body_started = true;
                continue;
            }
            if let Some((key, value)) = line.split_once(':') {
                let value = value.trim().trim_matches('"').trim_matches('\'').to_string();
                match key.trim() {
                    "name" => name = value,
                    "description" => description = value,
                    "version" => version = value,
                    _ => {}
                }
            }
        } else {
            body.push_str(line);
            body.push('\n');
        }
    }
    if !body_started {
        return Err("SKILL.md front-matter is not closed with ---".to_string());
    }
    if name.is_empty() {
        name = fallback_name.unwrap_or("").to_string();
    }
    let id = name.to_lowercase().replace(' ', "-");
    if !slug_is_valid(&id) {
        return Err("skill name must be a lowercase slug (a-z, 0-9, -, _, .)".to_string());
    }
    if description.is_empty() {
        description = format!("Imported skill {id}");
    }
    if version.is_empty() {
        version = "unversioned".to_string();
    }
    let instructions = body.trim().to_string();
    if instructions.is_empty() {
        return Err("skill instructions are empty".to_string());
    }
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    let hash = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    Ok(SkillPackage {
        id,
        name: if name.is_empty() {
            "Unnamed skill".to_string()
        } else {
            name
        },
        description,
        version,
        instructions,
        hash,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_skill_package;

    const PACKAGE: &str = "---\nname: webapp-testing\nversion: \"1.2.0\"\ndescription: Playwright toolkit\nauthor: example\n---\n\nUse the toolkit to drive a browser.\n";

    #[test]
    fn parses_front_matter_and_hashes_content() {
        let package = parse_skill_package(PACKAGE, None).expect("package parses");
        assert_eq!(package.id, "webapp-testing");
        assert_eq!(package.version, "1.2.0");
        assert_eq!(package.description, "Playwright toolkit");
        assert!(package.instructions.contains("drive a browser"));
        assert_eq!(package.hash.len(), 64);
        let again = parse_skill_package(PACKAGE, None).expect("idempotent");
        assert_eq!(package.hash, again.hash);
    }

    #[test]
    fn rejects_missing_front_matter_and_bad_names() {
        assert!(parse_skill_package("no front matter", None).is_err());
        // Human names normalize to a slug; unusable names are rejected.
        let normalized = parse_skill_package("---\nname: Bad Name\n---\nx", None).expect("normalizes");
        assert_eq!(normalized.id, "bad-name");
        assert!(parse_skill_package("---\nname: \"!!!\"\n---\nx", None).is_err());
        assert!(parse_skill_package("---\nname: ok\n---\n", None).is_err());
    }
}
