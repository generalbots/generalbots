//! #2662 — authorization metadata columns of `mcp.csv` (server-wide defaults).
//!
//! These helpers turn the previously discarded `risk_level` /
//! `requires_approval` columns into real, fail-closed values that are folded
//! into every tool's effective requirements by
//! `crate::keywords::mcp_client::tool_authorization`.

use crate::keywords::mcp_client::ToolRiskLevel;

/// Unknown / missing risk metadata fails closed to `High` (admin-only).
pub(super) fn parse_risk_level(value: Option<&str>) -> ToolRiskLevel {
    match value.map(str::to_lowercase).as_deref() {
        Some("safe") => ToolRiskLevel::Safe,
        Some("low") => ToolRiskLevel::Low,
        Some("medium") => ToolRiskLevel::Medium,
        Some("high") => ToolRiskLevel::High,
        Some("critical") => ToolRiskLevel::Critical,
        _ => ToolRiskLevel::High,
    }
}

/// A missing or unparseable approval column fails closed to `true`.
pub(super) fn parse_requires_approval(value: Option<&str>) -> bool {
    match value {
        Some(v) if v.eq_ignore_ascii_case("false") || v.eq_ignore_ascii_case("0") => false,
        // Missing or unrecognised values fail closed to "approval required".
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn risk_level_parsing_fails_closed() {
        assert_eq!(parse_risk_level(Some("critical")), ToolRiskLevel::Critical);
        assert_eq!(parse_risk_level(Some("nonsense")), ToolRiskLevel::High);
        assert_eq!(parse_risk_level(None), ToolRiskLevel::High);
    }

    #[test]
    fn approval_parsing_fails_closed_when_missing() {
        assert!(parse_requires_approval(None));
        assert!(parse_requires_approval(Some("maybe")));
        assert!(!parse_requires_approval(Some("false")));
    }
}
