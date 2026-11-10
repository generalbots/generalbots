use super::*;

impl IncidentSeverity {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
            Self::Informational => "informational",
        }
    }

    pub fn from_label(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "critical" => Some(Self::Critical),
            "high" => Some(Self::High),
            "medium" => Some(Self::Medium),
            "low" => Some(Self::Low),
            "informational" | "info" => Some(Self::Informational),
            _ => None,
        }
    }

    pub fn response_time_minutes(&self) -> i64 {
        match self {
            Self::Critical => 15,
            Self::High => 60,
            Self::Medium => 240,
            Self::Low => 1440,
            Self::Informational => 10080,
        }
    }
}

#[cfg(test)]
mod incident_severity_tests {
    use super::*;

    #[test]
    fn every_severity_round_trips_through_its_label() {
        for severity in [
            IncidentSeverity::Critical,
            IncidentSeverity::High,
            IncidentSeverity::Medium,
            IncidentSeverity::Low,
            IncidentSeverity::Informational,
        ] {
            let label = severity.as_str();
            assert_eq!(
                IncidentSeverity::from_label(label),
                Some(severity.clone()),
                "{label} must round-trip"
            );
        }
    }

    #[test]
    fn labels_are_case_insensitive_and_accept_info() {
        assert_eq!(
            IncidentSeverity::from_label("CRITICAL"),
            Some(IncidentSeverity::Critical)
        );
        assert_eq!(
            IncidentSeverity::from_label(" High "),
            Some(IncidentSeverity::High)
        );
        assert_eq!(
            IncidentSeverity::from_label("info"),
            Some(IncidentSeverity::Informational)
        );
        assert_eq!(IncidentSeverity::from_label("urgent"), None);
        assert_eq!(IncidentSeverity::from_label(""), None);
    }

    #[test]
    fn response_times_grow_as_severity_falls() {
        let ladder = [
            IncidentSeverity::Critical,
            IncidentSeverity::High,
            IncidentSeverity::Medium,
            IncidentSeverity::Low,
            IncidentSeverity::Informational,
        ];
        let mut previous = 0;
        for severity in ladder {
            let minutes = severity.response_time_minutes();
            assert!(
                minutes > previous,
                "{severity:?} must allow more time than the level above"
            );
            previous = minutes;
        }
        assert_eq!(IncidentSeverity::Critical.response_time_minutes(), 15);
        assert_eq!(IncidentSeverity::Informational.response_time_minutes(), 10080);
    }
}
