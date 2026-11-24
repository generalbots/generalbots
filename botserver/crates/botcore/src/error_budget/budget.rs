/// Stable rule order shared with the Python scanner's JSON output.
pub const RULE_NAMES: [&str; 5] = ["unwrap", "expect", "panic", "unwrap_or_default", "allow"];

/// Counts of error-handling debt proxies for one scope (a crate or a total).
///
/// Each field is a textual proxy produced by `scripts/check-error-budget.py`:
/// `.unwrap(` calls, `.expect(` calls, `panic!` invocations,
/// `.unwrap_or_default(` calls and `#[allow(...)]` attributes. The struct is
/// intentionally plain data with no parsing or I/O so the budget stays pure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ErrorCounts {
    pub unwrap: u64,
    pub expect: u64,
    pub panic: u64,
    pub unwrap_or_default: u64,
    pub allow: u64,
}

/// Outcome of a budget evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Every rule is at or below its baseline value.
    Allow,
    /// At least one rule grew. `reason` lists each regression and its delta.
    Refuse { reason: String },
}

impl Decision {
    /// Whether the evaluated counts were at or below their baseline.
    pub fn is_allowed(&self) -> bool {
        matches!(self, Decision::Allow)
    }

    /// Regression details when refused, or an empty string when allowed.
    pub fn reason(&self) -> &str {
        match self {
            Decision::Allow => "",
            Decision::Refuse { reason } => reason,
        }
    }
}

/// Shrink-only ratchet over [`ErrorCounts`].
///
/// Counts may stay equal or decrease, never grow. A single rule growing is
/// enough to refuse the change so CI can block newly introduced debt.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Budget;

impl Budget {
    /// Compare `counts` against `baseline` and decide whether to allow them.
    ///
    /// Pure and deterministic: no I/O, no clock, no randomness. When refusing,
    /// the reason names every regressed rule with its current value, the
    /// baseline ceiling and the positive delta.
    pub fn evaluate(counts: ErrorCounts, baseline: ErrorCounts) -> Decision {
        let observed = [
            (RULE_NAMES[0], counts.unwrap, baseline.unwrap),
            (RULE_NAMES[1], counts.expect, baseline.expect),
            (RULE_NAMES[2], counts.panic, baseline.panic),
            (
                RULE_NAMES[3],
                counts.unwrap_or_default,
                baseline.unwrap_or_default,
            ),
            (RULE_NAMES[4], counts.allow, baseline.allow),
        ];

        let mut regressions: Vec<String> = Vec::new();
        for (name, current, ceiling) in observed {
            if current > ceiling {
                regressions.push(format!(
                    "{name}: {current} > baseline {ceiling} (+{})",
                    current - ceiling
                ));
            }
        }

        if regressions.is_empty() {
            Decision::Allow
        } else {
            Decision::Refuse {
                reason: regressions.join("; "),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Budget, Decision, ErrorCounts};

    fn counts(
        unwrap: u64,
        expect: u64,
        panic: u64,
        unwrap_or_default: u64,
        allow: u64,
    ) -> ErrorCounts {
        ErrorCounts {
            unwrap,
            expect,
            panic,
            unwrap_or_default,
            allow,
        }
    }

    #[test]
    fn increase_is_refused_with_a_regression_reason() {
        let baseline = counts(2, 1, 0, 10, 0);
        let current = counts(4, 1, 0, 10, 0);
        let decision = Budget::evaluate(current, baseline);
        assert!(!decision.is_allowed(), "a grown count must not be allowed");
        let reason = decision.reason();
        assert!(
            reason.contains("unwrap"),
            "reason must name the rule: {reason}"
        );
        assert!(
            reason.contains("4 > baseline 2"),
            "reason must show values: {reason}"
        );
        assert!(
            reason.contains("(+2)"),
            "reason must show the delta: {reason}"
        );
    }

    #[test]
    fn multiple_increases_are_all_reported() {
        let baseline = counts(1, 1, 0, 10, 0);
        let current = counts(2, 3, 0, 10, 0);
        let decision = Budget::evaluate(current, baseline);
        assert!(!decision.is_allowed(), "grown counts must not be allowed");
        let reason = decision.reason();
        assert!(reason.contains("unwrap:"), "reason: {reason}");
        assert!(reason.contains("expect:"), "reason: {reason}");
    }

    #[test]
    fn decrease_is_allowed() {
        let baseline = counts(2, 1, 1, 10, 2);
        let current = counts(1, 1, 0, 9, 2);
        assert_eq!(Budget::evaluate(current, baseline), Decision::Allow);
    }

    #[test]
    fn equal_is_allowed() {
        let baseline = counts(2, 1, 0, 10, 3);
        assert_eq!(Budget::evaluate(baseline, baseline), Decision::Allow);
    }

    #[test]
    fn zero_baseline_refuses_any_debt() {
        let current = counts(0, 0, 0, 0, 1);
        let decision = Budget::evaluate(current, ErrorCounts::default());
        assert!(
            !decision.is_allowed(),
            "any new count must be refused at zero baseline"
        );
        assert!(
            decision.reason().contains("allow:"),
            "reason: {}",
            decision.reason()
        );
    }
}
