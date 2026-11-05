//! Pooled group budgets and usage aggregation (#1697).
//!
//! A group budget pools a single milli-USD allowance across its members. The
//! HTTP layer stores the allowance and the per-member spend rows; this module
//! owns the arithmetic and the report aggregation so it can be tested without
//! a database or a server.

/// Allowance vs. spend for one group.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BudgetSummary {
    pub budget_milli: u64,
    pub spent_milli: u64,
}

impl BudgetSummary {
    #[must_use]
    pub fn remaining_milli(&self) -> u64 {
        self.budget_milli.saturating_sub(self.spent_milli)
    }

    #[must_use]
    pub fn over_budget(&self) -> bool {
        self.spent_milli > self.budget_milli
    }

    /// Whole percentage used, capped at 100 for display.
    #[must_use]
    pub fn percent_used(&self) -> u8 {
        if self.budget_milli == 0 {
            return 0;
        }
        let percent = self.spent_milli.saturating_mul(100) / self.budget_milli;
        u8::try_from(percent.min(100)).unwrap_or(100)
    }
}

/// One attributed spend row (a run, a browser task, a VM-minute...).
#[derive(Debug, Clone, Copy)]
pub struct SpendEntry<'a> {
    pub user_id: &'a str,
    pub amount_milli: u64,
}

/// Per-member part of the pooled report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberSpend {
    pub user_id: String,
    pub amount_milli: u64,
    pub entries: u64,
}

/// The pooled report: total spend plus the per-member breakdown.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PooledReport {
    pub total_milli: u64,
    pub members: Vec<MemberSpend>,
}

/// Aggregates spend rows per member, highest spend first (ties by id so the
/// report is deterministic).
#[must_use]
pub fn aggregate<'a, I>(entries: I) -> PooledReport
where
    I: IntoIterator<Item = SpendEntry<'a>>,
{
    let mut members: Vec<MemberSpend> = Vec::new();
    let mut total = 0u64;
    for entry in entries {
        total = total.saturating_add(entry.amount_milli);
        match members.iter_mut().find(|member| member.user_id == entry.user_id) {
            Some(member) => {
                member.amount_milli = member.amount_milli.saturating_add(entry.amount_milli);
                member.entries += 1;
            }
            None => members.push(MemberSpend {
                user_id: entry.user_id.to_string(),
                amount_milli: entry.amount_milli,
                entries: 1,
            }),
        }
    }
    members.sort_by(|left, right| {
        right
            .amount_milli
            .cmp(&left.amount_milli)
            .then_with(|| left.user_id.cmp(&right.user_id))
    });
    PooledReport {
        total_milli: total,
        members,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry<'a>(user: &'a str, amount: u64) -> SpendEntry<'a> {
        SpendEntry {
            user_id: user,
            amount_milli: amount,
        }
    }

    #[test]
    fn summary_reports_remaining_percent_and_overflow() {
        let summary = BudgetSummary {
            budget_milli: 10_000,
            spent_milli: 4_500,
        };
        assert_eq!(summary.remaining_milli(), 5_500);
        assert_eq!(summary.percent_used(), 45);
        assert!(!summary.over_budget());

        let over = BudgetSummary {
            budget_milli: 1_000,
            spent_milli: 1_500,
        };
        assert_eq!(over.remaining_milli(), 0);
        assert_eq!(over.percent_used(), 100);
        assert!(over.over_budget());
    }

    #[test]
    fn zero_budget_never_divides_by_zero() {
        let summary = BudgetSummary {
            budget_milli: 0,
            spent_milli: 700,
        };
        assert_eq!(summary.percent_used(), 0);
        assert_eq!(summary.remaining_milli(), 0);
        assert!(summary.over_budget());
    }

    #[test]
    fn empty_ledger_is_an_empty_report() {
        let report = aggregate(Vec::<SpendEntry>::new());
        assert_eq!(report.total_milli, 0);
        assert!(report.members.is_empty());
    }

    #[test]
    fn rows_combine_per_member_and_sort_by_spend() {
        let report = aggregate(vec![
            entry("alice", 100),
            entry("bob", 400),
            entry("alice", 250),
            entry("carol", 50),
        ]);
        assert_eq!(report.total_milli, 800);
        assert_eq!(
            report.members,
            vec![
                MemberSpend { user_id: "bob".into(), amount_milli: 400, entries: 1 },
                MemberSpend { user_id: "alice".into(), amount_milli: 350, entries: 2 },
                MemberSpend { user_id: "carol".into(), amount_milli: 50, entries: 1 },
            ]
        );
    }

    #[test]
    fn report_total_matches_budget_summary() {
        let report = aggregate(vec![entry("alice", 120), entry("bob", 80)]);
        let summary = BudgetSummary {
            budget_milli: 500,
            spent_milli: report.total_milli,
        };
        assert_eq!(summary.remaining_milli(), 300);
        assert_eq!(summary.percent_used(), 40);
    }
}
