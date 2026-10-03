//! Document review by a pair of independent LLMs (issue #1458).
//!
//! A single model asked to "improve this document" grades its own work: it
//! rewrites first and judges second, so a confident-sounding rewrite gets a
//! confident-sounding approval. Two models with *different* roles, run
//! concurrently on the same input and never shown each other's output, break
//! that loop — the agreement between them is itself a signal, which is why the
//! merged findings carry an `agreement` count rather than just a severity.
//!
//! The shape is deliberately machine-readable. A caller gets a JSON list of
//! findings, each attributed to the reviewer that raised it, plus the rewritten
//! text — not a chatty report the caller has to parse.
//!
//! Roles:
//!
//! * **Editor** — structure, clarity, tone, and the mechanical defects.
//! * **Critic** — claims, gaps, and what a hostile reader would object to.
//!
//! Neither sees the other's output, and neither is asked to rewrite: rewriting is
//! a third pass driven by the merged findings, so a reviewer cannot smuggle its
//! own edit past the other's scrutiny.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// How serious a finding is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Worth fixing, would not embarrass the author.
    Minor,
    /// Costs the reader confidence or comprehension.
    Major,
    /// A factual, legal or safety problem. Never silently ignored.
    Critical,
}

impl Severity {
    fn rank(self) -> u8 {
        match self {
            Self::Minor => 0,
            Self::Major => 1,
            Self::Critical => 2,
        }
    }

    /// Parses a model's severity word, defaulting to `Minor` for anything it
    /// invents: an unrecognised label must not silently become `Critical`.
    #[must_use]
    pub fn parse(value: &str) -> Self {
        match value.trim().to_lowercase().as_str() {
            "critical" | "blocker" | "high" => Self::Critical,
            "major" | "medium" | "important" => Self::Major,
            _ => Self::Minor,
        }
    }
}

/// The two reviewing roles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewerRole {
    Editor,
    Critic,
}

impl ReviewerRole {
    pub const ALL: [Self; 2] = [Self::Editor, Self::Critic];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Editor => "editor",
            Self::Critic => "critic",
        }
    }

    /// System prompt for the role.
    ///
    /// The Critic is told to argue against the document and the Editor to argue
    /// for its readability, so their findings do not overlap by construction.
    #[must_use]
    pub fn system_prompt(self) -> &'static str {
        match self {
            Self::Editor => "You are a meticulous editor. Report problems with structure, \
                clarity, tone, consistency and mechanical correctness. Do not rewrite the \
                document and do not evaluate whether its claims are true; another reviewer \
                does that. Reply with JSON only.",
            Self::Critic => "You are an adversarial reviewer. Look for unsupported claims, \
                logical gaps, missing caveats, and anything a hostile or expert reader would \
                object to. Do not rewrite the document and do not comment on style; another \
                reviewer does that. Reply with JSON only.",
        }
    }
}

/// One reviewer's objection to the document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    /// Short slug for the kind of problem, e.g. `clarity`, `unsupported-claim`.
    pub category: String,
    /// One sentence, addressed to the author.
    pub summary: String,
    /// The quoted span the finding is about, when the reviewer gave one.
    pub excerpt: String,
}

/// What one reviewer returned.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Review {
    pub role: ReviewerRole,
    pub findings: Vec<Finding>,
    /// A one-line verdict for the UI.
    pub verdict: String,
}

/// The wire shape a reviewer is asked to produce.
#[derive(Debug, Clone, Deserialize)]
struct ReviewerPayload {
    #[serde(default)]
    findings: Vec<RawFinding>,
    #[serde(default)]
    verdict: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RawFinding {
    #[serde(default)]
    severity: String,
    #[serde(default)]
    category: String,
    #[serde(default)]
    summary: String,
    #[serde(default)]
    excerpt: String,
}

/// A finding after both reviewers have spoken, with its provenance.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MergedFinding {
    pub severity: Severity,
    pub category: String,
    pub summary: String,
    /// How many reviewers raised this. Two agreeing is a much stronger signal
    /// than one, which is the entire reason for running a pair.
    pub agreement: usize,
    /// Which roles raised it.
    pub reviewers: Vec<ReviewerRole>,
}

/// The merged result of a pair review.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReviewOutcome {
    /// Findings from both reviewers, most severe first.
    pub findings: Vec<MergedFinding>,
    /// Number of findings both reviewers raised.
    pub agreed: usize,
    /// The document after applying the findings. Empty when no rewrite pass ran.
    pub revised_text: String,
}

/// What went wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewError {
    /// Neither reviewer produced usable output.
    NoReviewers(String),
    /// The LLM is not configured at all.
    NotConfigured,
    /// A provider call failed.
    Provider(String),
}

impl std::fmt::Display for ReviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoReviewers(message) => write!(f, "no reviewer produced a usable review: {message}"),
            Self::NotConfigured => write!(f, "no LLM is configured"),
            Self::Provider(message) => write!(f, "review provider failed: {message}"),
        }
    }
}

/// Normalises a summary so two reviewers describing the same problem the same way
/// still merge, while genuinely different complaints stay separate.
///
/// Punctuation is dropped, not merely ignored: two reviewers will write
/// "unclear." and "unclear" for the same finding, and a trailing full stop is a
/// difference in typing, not in substance.
#[must_use]
pub fn dedup_key(summary: &str) -> String {
    summary
        .split_whitespace()
        .map(|word| {
            word.chars()
                .filter(|character| character.is_alphanumeric())
                .collect::<String>()
        })
        .filter(|word| word.len() > 3)
        .collect::<Vec<String>>()
        .join(" ")
        .to_lowercase()
}

/// Parses a reviewer's reply, tolerating a JSON fence around it.
///
/// A model asked for "JSON only" still wraps the object in a ```json fence about
/// half the time; failing the whole review over presentation would make the
/// feature useless.
#[must_use]
pub fn parse_review(role: ReviewerRole, raw: &str) -> Review {
    let text = strip_fence(raw);
    let parsed: ReviewerPayload = serde_json::from_str(&text).unwrap_or(ReviewerPayload {
        findings: Vec::new(),
        verdict: String::new(),
    });

    let findings = parsed
        .findings
        .into_iter()
        // A finding with no summary cannot be acted on or de-duplicated.
        .filter(|finding| !finding.summary.trim().is_empty())
        .map(|finding| Finding {
            severity: Severity::parse(&finding.severity),
            category: if finding.category.trim().is_empty() {
                "general".to_string()
            } else {
                finding.category.trim().to_lowercase()
            },
            summary: finding.summary.trim().to_string(),
            excerpt: finding.excerpt.trim().to_string(),
        })
        .collect();

    Review { role, findings, verdict: parsed.verdict.trim().to_string() }
}

/// Removes a Markdown code fence, if the whole payload is wrapped in one.
#[must_use]
pub fn strip_fence(raw: &str) -> String {
    let trimmed = raw.trim();
    let Some(after_open) = trimmed.strip_prefix("```") else {
        return trimmed.to_string();
    };
    // Drop the language hint on the opening fence, e.g. ```json
    let body = after_open
        .strip_prefix("json")
        .or_else(|| after_open.strip_prefix("JSON"))
        .unwrap_or(after_open)
        .trim_start_matches(['\n', '\r']);
    body.strip_suffix("```").unwrap_or(body).trim().to_string()
}

/// Merges two reviews into one prioritised list.
///
/// Agreement is the ranking signal within a severity band: a problem both
/// reviewers found outranks one only a single reviewer noticed, because it is the
/// finding the pair design exists to surface.
#[must_use]
pub fn merge_reviews(reviews: &[Review]) -> ReviewOutcome {
    let mut by_key: BTreeMap<String, MergedFinding> = BTreeMap::new();

    for review in reviews {
        for finding in &review.findings {
            let key = format!("{}:{}", finding.category, dedup_key(&finding.summary));
            match by_key.get_mut(&key) {
                None => {
                    by_key.insert(
                        key,
                        MergedFinding {
                            // The more severe of two reports wins, so a reviewer
                            // who rates something Critical is not diluted by one
                            // who calls it Minor.
                            severity: finding.severity,
                            category: finding.category.clone(),
                            summary: finding.summary.clone(),
                            agreement: 1,
                            reviewers: vec![review.role],
                        },
                    );
                }
                Some(existing) => {
                    existing.severity = existing.severity.max(finding.severity);
                    existing.agreement += 1;
                    if !existing.reviewers.contains(&review.role) {
                        existing.reviewers.push(review.role);
                    }
                }
            }
        }
    }

    let mut findings: Vec<MergedFinding> = by_key.into_values().collect();
    findings.sort_by(|left, right| {
        right
            .severity
            .rank()
            .cmp(&left.severity.rank())
            .then(right.agreement.cmp(&left.agreement))
            .then(left.summary.cmp(&right.summary))
    });

    let agreed = findings.iter().filter(|f| f.agreement > 1).count();
    ReviewOutcome { findings, agreed, revised_text: String::new() }
}

/// Builds the user prompt handed to one reviewer.
#[must_use]
pub fn build_prompt(role: ReviewerRole, document: &str, instruction: &str) -> String {
    let extra = if instruction.trim().is_empty() {
        String::new()
    } else {
        format!("\n\nAdditional requirement: {}\n", instruction.trim())
    };
    format!(
        "You are reviewing as the {}. Review the document below and reply with JSON only, \
         shaped as {{\"verdict\": \"<one line>\", \"findings\": [{{\"severity\": \
         \"minor|major|critical\", \"category\": \"<slug>\", \"summary\": \
         \"<one sentence>\", \"excerpt\": \"<quoted span>\"}}]}}. \
         Report at most ten findings, most serious first.{extra}\n\n\
         --- DOCUMENT ---\n{document}",
        role.as_str()
    )
}

/// Builds the rewrite instruction from the merged findings.
///
/// The rewriter sees the findings, not the reviewers' prose, so a single model
/// can apply both roles' work without being asked to reconcile two voices.
#[must_use]
pub fn build_rewrite_prompt(document: &str, findings: &[MergedFinding]) -> String {
    let mut list = String::new();
    for finding in findings {
        list.push_str(&format!(
            "- [{}] ({}): {}\n",
            finding.severity_rank(),
            finding.category,
            finding.summary
        ));
    }
    format!(
        "Rewrite the document below, applying every listed correction. Keep the author's \
         meaning, structure and voice; change only what the corrections require. Where a \
         correction asks for a fact you cannot verify, insert the marker \
         [VERIFY: what is needed] rather than inventing it. Reply with the rewritten \
         document and nothing else.\n\n--- CORRECTIONS ---\n{list}\n--- DOCUMENT ---\n{document}"
    )
}

impl MergedFinding {
    /// Severity as the wire word, for a prompt or a log line.
    #[must_use]
    pub fn severity_rank(&self) -> &'static str {
        match self.severity {
            Severity::Minor => "minor",
            Severity::Major => "major",
            Severity::Critical => "critical",
        }
    }
}

#[cfg(test)]
#[path = "review_tests.rs"]
mod tests;
