pub fn rule_score(severity: &str) -> i32 {
    match severity {
        "critical" => 40,
        "high" => 25,
        "medium" => 15,
        "low" => 5,
        _ => 10,
    }
}

pub fn classify(score: i32, blocked: bool) -> (String, String) {
    if blocked || score >= 80 {
        ("critical".into(), "block".into())
    } else if score >= 60 {
        ("high".into(), "review".into())
    } else if score >= 30 {
        ("medium".into(), "flag".into())
    } else {
        ("low".into(), "allow".into())
    }
}

#[cfg(test)]
mod scoring_tests {
    use super::*;

    #[test]
    fn each_severity_has_its_weight() {
        assert_eq!(rule_score("critical"), 40);
        assert_eq!(rule_score("high"), 25);
        assert_eq!(rule_score("medium"), 15);
        assert_eq!(rule_score("low"), 5);
        // An unknown severity must not be free: it scores the default.
        assert_eq!(rule_score("invented"), 10);
        assert_eq!(rule_score(""), 10);
    }

    #[test]
    fn scores_escalate_through_the_four_levels() {
        assert_eq!(classify(0, false), ("low".into(), "allow".into()));
        assert_eq!(classify(29, false), ("low".into(), "allow".into()));
        assert_eq!(classify(30, false), ("medium".into(), "flag".into()));
        assert_eq!(classify(59, false), ("medium".into(), "flag".into()));
        assert_eq!(classify(60, false), ("high".into(), "review".into()));
        assert_eq!(classify(79, false), ("high".into(), "review".into()));
        assert_eq!(classify(80, false), ("critical".into(), "block".into()));
        assert_eq!(classify(100, false), ("critical".into(), "block".into()));
    }

    #[test]
    fn a_blocked_decision_is_always_critical() {
        // Even a low score is critical when something else forced the block.
        assert_eq!(classify(0, true), ("critical".into(), "block".into()));
        assert_eq!(classify(5, true), ("critical".into(), "block".into()));
    }
}
