use super::*;

impl ScanStats {
    pub fn add_issue(&mut self, severity: &IssueSeverity) {
        match severity {
            IssueSeverity::Critical => self.critical += 1,
            IssueSeverity::High => self.high += 1,
            IssueSeverity::Medium => self.medium += 1,
            IssueSeverity::Low => self.low += 1,
            IssueSeverity::Info => self.info += 1,
        }
        self.total += 1;
    }

    pub fn merge(&mut self, other: &Self) {
        self.critical += other.critical;
        self.high += other.high;
        self.medium += other.medium;
        self.low += other.low;
        self.info += other.info;
        self.total += other.total;
    }
}

#[cfg(test)]
mod scan_stats_tests {
    use super::*;

    fn empty() -> ScanStats {
        ScanStats {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
            info: 0,
            total: 0,
        }
    }

    #[test]
    fn each_severity_increments_its_own_counter() {
        let mut stats = empty();
        stats.add_issue(&IssueSeverity::Critical);
        stats.add_issue(&IssueSeverity::High);
        stats.add_issue(&IssueSeverity::Medium);
        stats.add_issue(&IssueSeverity::Low);
        stats.add_issue(&IssueSeverity::Info);

        assert_eq!(stats.critical, 1);
        assert_eq!(stats.high, 1);
        assert_eq!(stats.medium, 1);
        assert_eq!(stats.low, 1);
        assert_eq!(stats.info, 1);
        assert_eq!(stats.total, 5);
    }

    #[test]
    fn repeated_severities_accumulate() {
        let mut stats = empty();
        for _ in 0..3 {
            stats.add_issue(&IssueSeverity::High);
        }
        assert_eq!(stats.high, 3);
        assert_eq!(stats.total, 3);
    }

    #[test]
    fn merging_adds_every_counter_and_the_total() {
        let mut left = empty();
        left.add_issue(&IssueSeverity::Critical);
        left.add_issue(&IssueSeverity::Low);

        let mut right = empty();
        right.add_issue(&IssueSeverity::Critical);
        right.add_issue(&IssueSeverity::Info);
        right.add_issue(&IssueSeverity::Info);

        left.merge(&right);

        assert_eq!(left.critical, 2);
        assert_eq!(left.low, 1);
        assert_eq!(left.info, 2);
        assert_eq!(left.total, 5);
    }

    #[test]
    fn merging_an_empty_stats_changes_nothing() {
        let mut stats = empty();
        stats.add_issue(&IssueSeverity::Medium);
        let (critical, high, medium, low, info, total) = (
            stats.critical,
            stats.high,
            stats.medium,
            stats.low,
            stats.info,
            stats.total,
        );
        stats.merge(&empty());
        assert_eq!(
            (stats.critical, stats.high, stats.medium, stats.low, stats.info, stats.total),
            (critical, high, medium, low, info, total)
        );
    }

    #[test]
    fn the_total_always_equals_the_sum_of_the_counters() {
        let mut stats = empty();
        for severity in [
            IssueSeverity::Critical,
            IssueSeverity::High,
            IssueSeverity::Medium,
            IssueSeverity::Low,
            IssueSeverity::Info,
            IssueSeverity::Critical,
        ] {
            stats.add_issue(&severity);
        }
        assert_eq!(
            stats.total,
            stats.critical + stats.high + stats.medium + stats.low + stats.info
        );
    }
}
