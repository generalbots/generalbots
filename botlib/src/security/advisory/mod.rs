//! Canonical security advisories: one record source for the read-only API,
//! `gb audit`, CSAF, OSV and offline bundles (#2712, #2723).

pub mod store;
pub mod types;

pub use store::{is_version_affected, AdvisoryError, AdvisoryStore};
pub use types::{
    Advisory, AdvisoryBundle, AdvisoryQuery, Cvss, Remediation, Revision, Severity, VersionRange,
    ADVISORY_SCHEMA_URL, ADVISORY_SCHEMA_VERSION,
};
