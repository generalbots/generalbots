//! Loading, filtering and version matching for the canonical advisory store.

use super::types::{Advisory, AdvisoryBundle, AdvisoryQuery, ADVISORY_SCHEMA_VERSION};
use std::{cmp::Ordering, collections::HashSet, fs, path::Path};

#[derive(Debug, thiserror::Error)]
pub enum AdvisoryError {
    #[error("cannot read advisory bundle: {0}")]
    Io(#[from] std::io::Error),
    #[error("cannot parse advisory bundle: {0}")]
    Json(#[from] serde_json::Error),
    #[error("unsupported advisory bundle schema: {0}")]
    UnsupportedSchema(String),
    #[error("duplicate advisory id: {0}")]
    DuplicateId(String),
}

/// Read-only, immutable set of canonical advisories. Built once and shared by
/// the API handlers; rebuilt by the CLI for each offline bundle.
#[derive(Debug, Clone)]
pub struct AdvisoryStore {
    advisories: Vec<Advisory>,
}

impl AdvisoryStore {
    /// Build a store from records already deserialized (e.g. an API response).
    pub fn new(advisories: Vec<Advisory>) -> Result<Self, AdvisoryError> {
        let mut seen = HashSet::new();
        for advisory in &advisories {
            if !seen.insert(advisory.id.as_str()) {
                return Err(AdvisoryError::DuplicateId(advisory.id.clone()));
            }
        }
        Ok(Self { advisories })
    }

    pub fn from_bundle(bundle: AdvisoryBundle) -> Result<Self, AdvisoryError> {
        if !bundle
            .schema
            .contains(&format!("advisory-{ADVISORY_SCHEMA_VERSION}"))
        {
            return Err(AdvisoryError::UnsupportedSchema(bundle.schema));
        }
        Self::new(bundle.advisories)
    }

    /// Load a bundle from disk. Used by `--offline` and by the API when
    /// `GB_ADVISORY_BUNDLE` points at a mirrored bundle.
    pub fn from_path(path: &Path) -> Result<Self, AdvisoryError> {
        let raw = fs::read_to_string(path)?;
        let bundle: AdvisoryBundle = serde_json::from_str(&raw)?;
        Self::from_bundle(bundle)
    }

    /// The bundle compiled into the binary: a safe fallback when no mirror is
    /// configured and the network is unavailable.
    pub fn embedded() -> Result<Self, AdvisoryError> {
        let bundle: AdvisoryBundle = serde_json::from_str(include_str!("data/advisories.json"))?;
        Self::from_bundle(bundle)
    }

    pub fn all(&self) -> &[Advisory] {
        &self.advisories
    }

    pub fn query(&self, filter: &AdvisoryQuery) -> Vec<&Advisory> {
        self.advisories
            .iter()
            .filter(|advisory| filter.matches(advisory))
            .collect()
    }

    pub fn get(&self, id: &str) -> Option<&Advisory> {
        self.advisories.iter().find(|advisory| advisory.id == id)
    }
}

/// Compare dotted numeric versions (`2.3`, `2.3.1`, `2.3.1-rc1`). A prerelease
/// suffix is ignored for range bounds, which are always stable releases.
fn compare_versions(left: &str, right: &str) -> Ordering {
    let left_parts = numeric_parts(left);
    let right_parts = numeric_parts(right);
    let width = left_parts.len().max(right_parts.len());
    for index in 0..width {
        let lhs = left_parts.get(index).copied().unwrap_or(0);
        let rhs = right_parts.get(index).copied().unwrap_or(0);
        match lhs.cmp(&rhs) {
            Ordering::Equal => continue,
            other => return other,
        }
    }
    Ordering::Equal
}

fn numeric_parts(version: &str) -> Vec<u64> {
    let core = version.split(['-', '+']).next().unwrap_or(version);
    core.split('.')
        .map(|part| match part.parse::<u64>() {
            Ok(value) => value,
            Err(_) => 0,
        })
        .collect()
}

/// Evaluate an authored range expression such as `">= 6.8.0, < 6.8.12"`.
fn authored_range_matches(expression: &str, version: &str) -> bool {
    for clause in expression.split(',') {
        let clause = clause.trim();
        let (operator, bound) = if let Some(value) = clause.strip_prefix(">=") {
            (">=", value.trim())
        } else if let Some(value) = clause.strip_prefix("<=") {
            ("<=", value.trim())
        } else if let Some(value) = clause.strip_prefix('>') {
            (">", value.trim())
        } else if let Some(value) = clause.strip_prefix('<') {
            ("<", value.trim())
        } else {
            ("=", clause)
        };
        if bound.is_empty() {
            continue;
        }
        let order = compare_versions(version, bound);
        let satisfied = match operator {
            ">=" => order != Ordering::Less,
            ">" => order == Ordering::Greater,
            "<=" => order != Ordering::Greater,
            "<" => order == Ordering::Less,
            _ => order == Ordering::Equal,
        };
        if !satisfied {
            return false;
        }
    }
    true
}

pub fn is_version_affected(range: &super::types::VersionRange, version: &str) -> bool {
    let introduced = match &range.introduced {
        Some(bound) => compare_versions(version, bound) != Ordering::Less,
        None => true,
    };
    let fixed = match &range.fixed {
        Some(bound) => compare_versions(version, bound) == Ordering::Less,
        None => true,
    };
    if !introduced || !fixed {
        return false;
    }
    match &range.range {
        Some(expression) => authored_range_matches(expression, version),
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::advisory::types::Severity;
    use chrono::NaiveDate;

    fn sample() -> Advisory {
        Advisory {
            id: "GBSA-2026-0001".to_string(),
            title: "Test advisory".to_string(),
            summary: None,
            severity: Severity::High,
            product: "botserver".to_string(),
            component: "botserver".to_string(),
            published: NaiveDate::from_ymd_opt(2026, 11, 20).unwrap(),
            updated: None,
            affected_versions: vec![super::super::types::VersionRange {
                introduced: Some("1.0.0".to_string()),
                fixed: Some("2.3.1".to_string()),
                range: Some(">= 1.0.0, < 2.3.1".to_string()),
            }],
            remediation: super::super::types::Remediation {
                recommended_version: "2.3.1".to_string(),
                text: "Upgrade to 2.3.1 or later".to_string(),
            },
            cvss: Vec::new(),
            cwe: Vec::new(),
            references: Vec::new(),
            revisions: Vec::new(),
        }
    }

    #[test]
    fn embedded_bundle_is_valid() {
        let store = AdvisoryStore::embedded().unwrap();
        assert!(store.all().len() >= 2);
        assert!(store.get("GBSA-2026-0001").is_some());
    }

    #[test]
    fn embedded_bundle_matches_canonical_schema() {
        use serde_json::Value;
        let schema: Value =
            serde_json::from_str(include_str!("data/advisory.schema.json")).unwrap();
        let required = schema["required"].as_array().unwrap();
        let bundle: Value = serde_json::from_str(include_str!("data/advisories.json")).unwrap();
        for advisory in bundle["advisories"].as_array().unwrap() {
            for field in required {
                let name = field.as_str().unwrap();
                assert!(
                    advisory.get(name).is_some(),
                    "missing required field {name}"
                );
            }
        }
    }

    #[test]
    fn version_ranges_are_inclusive_exclusive() {
        let advisory = sample();
        assert!(advisory.is_affected("botserver", "botserver", "2.3.0"));
        assert!(advisory.is_affected("botserver", "botserver", "1.0.0"));
        assert!(!advisory.is_affected("botserver", "botserver", "2.3.1"));
        assert!(!advisory.is_affected("botui", "botserver", "2.3.0"));
    }

    #[test]
    fn authored_range_expression_is_honoured() {
        let range = super::super::types::VersionRange {
            introduced: Some("6.8.0".to_string()),
            fixed: Some("6.8.12".to_string()),
            range: Some(">= 6.8.0, < 6.8.12".to_string()),
        };
        assert!(is_version_affected(&range, "6.8.11"));
        assert!(!is_version_affected(&range, "6.8.12"));
    }

    #[test]
    fn severity_and_date_filters() {
        let advisory = sample();
        let store = AdvisoryStore::new(vec![advisory]).unwrap();
        let mut filter = AdvisoryQuery {
            severity: Some(Severity::High),
            ..AdvisoryQuery::default()
        };
        assert_eq!(store.query(&filter).len(), 1);
        filter.severity = Some(Severity::Low);
        assert!(store.query(&filter).is_empty());
    }
}
