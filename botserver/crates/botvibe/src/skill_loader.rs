use crate::skills::VibeSkill;
use uuid::Uuid;

pub const MAX_STACKED_SKILLS: usize = 5;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedSkillMarkdown {
    pub name: String,
    pub description: String,
    pub triggers: Vec<String>,
    pub body: String,
}

pub fn parse_skill_markdown(content: &str) -> Option<ParsedSkillMarkdown> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        return None;
    }
    let rest = trimmed.trim_start_matches("---");
    let end = rest.find("---")?;
    let frontmatter = &rest[..end];
    let body = rest[end + 3..].trim().to_string();
    if body.is_empty() {
        return None;
    }
    let mut name = None;
    let mut description = String::new();
    let mut triggers = Vec::new();
    let mut in_triggers = false;
    for line in frontmatter.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if in_triggers {
            if let Some(item) = line.strip_prefix('-') {
                let t = item.trim().to_string();
                if !t.is_empty() {
                    triggers.push(t);
                }
                continue;
            }
            in_triggers = false;
        }
        if let Some(v) = line.strip_prefix("name:") {
            name = Some(v.trim().to_string());
        } else if let Some(v) = line.strip_prefix("description:") {
            description = v.trim().to_string();
        } else if line == "triggers:" {
            in_triggers = true;
        }
    }
    let name = name.filter(|n| !n.is_empty())?;
    Some(ParsedSkillMarkdown {
        name,
        description,
        triggers,
        body,
    })
}

pub struct BootstrapSkill {
    pub name: &'static str,
    pub description: &'static str,
    pub content: &'static str,
    pub triggers: &'static [&'static str],
}

pub const BOOTSTRAP_SKILLS: &[BootstrapSkill] = &[
    BootstrapSkill {
        name: "security-review",
        description: "Review code changes for security issues before release",
        content: "Audit the diff for secrets, injection, missing tenant scoping, unsafe SQL, and unchecked user input. Report findings per file with severity.",
        triggers: &["security review", "audit security", "revisao de seguranca", "security audit"],
    },
    BootstrapSkill {
        name: "lint-test-loop",
        description: "Run lint and test loop until green",
        content: "Run cargo check and cargo clippy in the background, fix all warnings, then run the crate tests. Loop until zero warnings and zero failures.",
        triggers: &["lint and test", "lint+test", "make it green", "zero warnings", "clippy"],
    },
    BootstrapSkill {
        name: "release-checklist",
        description: "Checklist for shipping a release",
        content: "Verify migrations, run the test suite, update the changelog, tag the version, and push to all remotes. Confirm CI status before announcing.",
        triggers: &["release checklist", "release", "ship it", "deploy release"],
    },
    BootstrapSkill {
        name: "domain-deploy",
        description: "Bind a custom domain to a bot and deploy",
        content: "Resolve the bot from the requested hostname, register the domain mapping in bot_domains, update Caddy, and verify the DNS record resolves.",
        triggers: &["domain deploy", "bind domain", "domain binding", "vincular dominio"],
    },
    BootstrapSkill {
        name: "autotask-authoring",
        description: "Author an AutoTask BASIC script from a plain-language request",
        content: "Convert the user request into a plan of BASIC keywords (GET, SAVE, TALK, SEND MAIL), generate the script, register it as a tool, and test it in chat.",
        triggers: &["autotask", "auto task", "autor task", "write a script", "script authoring"],
    },
    BootstrapSkill {
        name: "issue-triage",
        description: "Triage and close GitHub issues",
        content: "List open issues, group them by severity, fix the blockers first with tests, verify with cargo check, then close each issue referencing the fix commit.",
        triggers: &["triage issues", "close issues", "issue triage", "gh issues"],
    },
];

pub fn bootstrap_skill_definitions() -> Vec<(String, String, String, Vec<String>)> {
    BOOTSTRAP_SKILLS
        .iter()
        .map(|s| {
            (
                s.name.to_string(),
                s.description.to_string(),
                s.content.to_string(),
                s.triggers.iter().map(|t| t.to_string()).collect(),
            )
        })
        .collect()
}

pub fn skill_from_markdown(content: &str) -> Option<VibeSkill> {
    let parsed = parse_skill_markdown(content)?;
    Some(VibeSkill {
        skill_id: Uuid::new_v4(),
        name: parsed.name,
        description: parsed.description,
        content: parsed.body,
        triggers: parsed.triggers,
        enabled: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_full_frontmatter() {
        let md = "---\nname: post-deploy\ndescription: Run post deploy checks\ntriggers:\n  - post de deploy\n  - pos deploy\n---\nRun the checks and ping the chat.";
        let parsed = parse_skill_markdown(md).unwrap();
        assert_eq!(parsed.name, "post-deploy");
        assert_eq!(parsed.description, "Run post deploy checks");
        assert_eq!(parsed.triggers, vec!["post de deploy", "pos deploy"]);
        assert_eq!(parsed.body, "Run the checks and ping the chat.");
    }

    #[test]
    fn parses_minimal_frontmatter() {
        let md = "---\nname: fix-it\n---\nFix everything.";
        let parsed = parse_skill_markdown(md).unwrap();
        assert_eq!(parsed.name, "fix-it");
        assert!(parsed.description.is_empty());
        assert!(parsed.triggers.is_empty());
        assert_eq!(parsed.body, "Fix everything.");
    }

    #[test]
    fn rejects_missing_frontmatter() {
        assert!(parse_skill_markdown("just a body").is_none());
    }

    #[test]
    fn rejects_missing_name() {
        let md = "---\ndescription: no name\n---\nBody";
        assert!(parse_skill_markdown(md).is_none());
    }

    #[test]
    fn rejects_empty_body() {
        let md = "---\nname: x\n---\n   ";
        assert!(parse_skill_markdown(md).is_none());
    }

    #[test]
    fn bootstrap_definitions_cover_all_six() {
        let defs = bootstrap_skill_definitions();
        assert_eq!(defs.len(), 6);
        let names: Vec<&str> = defs.iter().map(|d| d.0.as_str()).collect();
        for expected in ["security-review", "lint-test-loop", "release-checklist", "domain-deploy", "autotask-authoring", "issue-triage"] {
            assert!(names.contains(&expected), "missing {expected}");
        }
        for (name, _, content, triggers) in &defs {
            assert!(!name.is_empty());
            assert!(!content.is_empty());
            assert!(!triggers.is_empty(), "bootstrap skill {name} needs triggers");
        }
    }

    #[test]
    fn skill_from_markdown_builds_vibeskill() {
        let md = "---\nname: post-deploy\ndescription: Checks\ntriggers:\n  - deploy\n---\nBody here";
        let skill = skill_from_markdown(md).unwrap();
        assert_eq!(skill.name, "post-deploy");
        assert_eq!(skill.content, "Body here");
        assert_eq!(skill.triggers, vec!["deploy"]);
        assert!(skill.enabled);
    }

    #[test]
    fn max_stacked_is_five() {
        assert_eq!(MAX_STACKED_SKILLS, 5);
    }
}
