//! Canonical advisory record shared by the advisory API, the `gb audit` CLI,
//! the CSAF feed and the OSV feed. There is exactly one source of truth: a
//! record here is transformed into the feed documents, never re-authored.

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

/// Version of the canonical advisory schema. Bump only with a migration path
/// for the offline bundles shipped to air-gapped deployments (#2720).
pub const ADVISORY_SCHEMA_VERSION: &str = "1.0";
pub const ADVISORY_SCHEMA_URL: &str = "https://generalbots.org/security/schema/advisory-1.0.json";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Info,
}

impl Severity {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::Info => "info",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "critical" => Some(Self::Critical),
            "high" => Some(Self::High),
            "medium" | "moderate" => Some(Self::Medium),
            "low" => Some(Self::Low),
            "info" | "informational" => Some(Self::Info),
            _ => None,
        }
    }
}

/// A single affected version interval. `introduced` is inclusive, `fixed` is
/// exclusive; an absent bound means "all earlier" / "all later" respectively.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionRange {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub introduced: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed: Option<String>,
    /// The exact range expression authored in the advisory front matter,
    /// e.g. `">= 6.8.0, < 6.8.12"`. Kept verbatim so the renderer never has
    /// to re-invent a range string for CSAF/OSV.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Remediation {
    /// First release that is known not to be affected.
    pub recommended_version: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Cvss {
    /// `3.1` or `4.0` — the score/vector pair must use the same version.
    pub version: String,
    pub vector: String,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Revision {
    pub date: NaiveDate,
    pub summary: String,
}

/// The canonical record. CSAF and OSV generators consume this struct; they do
/// not keep their own copy of the fields (#2714, #2718).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Advisory {
    pub id: String,
    pub title: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    pub severity: Severity,
    pub product: String,
    pub component: String,
    pub published: NaiveDate,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated: Option<NaiveDate>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_versions: Vec<VersionRange>,
    pub remediation: Remediation,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cvss: Vec<Cvss>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub cwe: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub references: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revisions: Vec<Revision>,
}

impl Advisory {
    /// True when `version` falls in one of this advisory's affected ranges for
    /// the given product/component. Product matching is case-insensitive.
    pub fn is_affected(&self, product: &str, component: &str, version: &str) -> bool {
        if !self.product.eq_ignore_ascii_case(product)
            || !self.component.eq_ignore_ascii_case(component)
        {
            return false;
        }
        self.affected_versions
            .iter()
            .any(|range| crate::security::advisory::store::is_version_affected(range, version))
    }
}

/// Filters accepted by `GET /api/advisories` and by the CLI. `version` means
/// "only advisories that affect this version of the product".
#[derive(Debug, Clone, Default)]
pub struct AdvisoryQuery {
    pub product: Option<String>,
    pub component: Option<String>,
    pub version: Option<String>,
    pub severity: Option<Severity>,
    pub from: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}

impl AdvisoryQuery {
    pub fn matches(&self, advisory: &Advisory) -> bool {
        if let Some(product) = &self.product {
            if !advisory.product.eq_ignore_ascii_case(product) {
                return false;
            }
        }
        if let Some(component) = &self.component {
            if !advisory.component.eq_ignore_ascii_case(component) {
                return false;
            }
        }
        if let Some(severity) = self.severity {
            if advisory.severity != severity {
                return false;
            }
        }
        if let Some(from) = self.from {
            if advisory.published < from {
                return false;
            }
        }
        if let Some(to) = self.to {
            if advisory.published > to {
                return false;
            }
        }
        if let Some(version) = &self.version {
            let product = self.product.as_deref().unwrap_or(advisory.product.as_str());
            let component = self
                .component
                .as_deref()
                .unwrap_or(advisory.component.as_str());
            if !advisory.is_affected(product, component, version) {
                return false;
            }
        }
        true
    }
}

/// A bundle is the on-disk / on-wire envelope. The very same file backs the
/// static CSAF/OSV feeds and the air-gapped offline bundle.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdvisoryBundle {
    pub schema: String,
    pub advisories: Vec<Advisory>,
}
