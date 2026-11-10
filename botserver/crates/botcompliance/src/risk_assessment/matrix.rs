use super::*;

impl RiskMatrix {
    pub fn calculate_risk_level(&self, likelihood: &Likelihood, impact: &Impact) -> RiskLevel {
        self.matrix
            .get(&(likelihood.clone(), impact.clone()))
            .cloned()
            .unwrap_or(RiskLevel::Medium)
    }
}

impl Default for RiskMatrix {
    fn default() -> Self {
        let mut matrix = HashMap::new();

        matrix.insert((Likelihood::Rare, Impact::Negligible), RiskLevel::Low);
        matrix.insert((Likelihood::Rare, Impact::Minor), RiskLevel::Low);
        matrix.insert((Likelihood::Rare, Impact::Moderate), RiskLevel::Low);
        matrix.insert((Likelihood::Rare, Impact::Major), RiskLevel::Medium);
        matrix.insert((Likelihood::Rare, Impact::Catastrophic), RiskLevel::High);

        matrix.insert((Likelihood::Unlikely, Impact::Negligible), RiskLevel::Low);
        matrix.insert((Likelihood::Unlikely, Impact::Minor), RiskLevel::Low);
        matrix.insert((Likelihood::Unlikely, Impact::Moderate), RiskLevel::Medium);
        matrix.insert((Likelihood::Unlikely, Impact::Major), RiskLevel::High);
        matrix.insert(
            (Likelihood::Unlikely, Impact::Catastrophic),
            RiskLevel::High,
        );

        matrix.insert((Likelihood::Possible, Impact::Negligible), RiskLevel::Low);
        matrix.insert((Likelihood::Possible, Impact::Minor), RiskLevel::Medium);
        matrix.insert((Likelihood::Possible, Impact::Moderate), RiskLevel::Medium);
        matrix.insert((Likelihood::Possible, Impact::Major), RiskLevel::High);
        matrix.insert(
            (Likelihood::Possible, Impact::Catastrophic),
            RiskLevel::Critical,
        );

        matrix.insert((Likelihood::Likely, Impact::Negligible), RiskLevel::Medium);
        matrix.insert((Likelihood::Likely, Impact::Minor), RiskLevel::Medium);
        matrix.insert((Likelihood::Likely, Impact::Moderate), RiskLevel::High);
        matrix.insert((Likelihood::Likely, Impact::Major), RiskLevel::Critical);
        matrix.insert(
            (Likelihood::Likely, Impact::Catastrophic),
            RiskLevel::Critical,
        );

        matrix.insert(
            (Likelihood::AlmostCertain, Impact::Negligible),
            RiskLevel::Medium,
        );
        matrix.insert(
            (Likelihood::AlmostCertain, Impact::Minor),
            RiskLevel::High,
        );
        matrix.insert(
            (Likelihood::AlmostCertain, Impact::Moderate),
            RiskLevel::High,
        );
        matrix.insert(
            (Likelihood::AlmostCertain, Impact::Major),
            RiskLevel::Critical,
        );
        matrix.insert(
            (Likelihood::AlmostCertain, Impact::Catastrophic),
            RiskLevel::Critical,
        );

        Self { matrix }
    }
}

#[cfg(test)]
mod risk_matrix_tests {
    use super::*;

    fn level(likelihood: Likelihood, impact: Impact) -> RiskLevel {
        RiskMatrix::default().calculate_risk_level(&likelihood, &impact)
    }

    #[test]
    fn the_matrix_covers_every_likelihood_and_impact_pair() {
        let likelihoods = [
            Likelihood::Rare,
            Likelihood::Unlikely,
            Likelihood::Possible,
            Likelihood::Likely,
            Likelihood::AlmostCertain,
        ];
        let impacts = [
            Impact::Negligible,
            Impact::Minor,
            Impact::Moderate,
            Impact::Major,
            Impact::Catastrophic,
        ];
        let matrix = RiskMatrix::default();
        for likelihood in &likelihoods {
            for impact in &impacts {
                assert!(
                    matrix.matrix.contains_key(&(likelihood.clone(), impact.clone())),
                    "{likelihood:?}/{impact:?} must be defined"
                );
            }
        }
    }

    #[test]
    fn the_corners_are_low_and_critical() {
        assert_eq!(level(Likelihood::Rare, Impact::Negligible), RiskLevel::Low);
        assert_eq!(
            level(Likelihood::AlmostCertain, Impact::Catastrophic),
            RiskLevel::Critical
        );
    }

    #[test]
    fn risk_never_decreases_as_likelihood_rises() {
        let impacts = [
            Impact::Negligible,
            Impact::Minor,
            Impact::Moderate,
            Impact::Major,
            Impact::Catastrophic,
        ];
        let ladder = [
            Likelihood::Rare,
            Likelihood::Unlikely,
            Likelihood::Possible,
            Likelihood::Likely,
            Likelihood::AlmostCertain,
        ];
        for impact in impacts {
            let mut previous = RiskLevel::Low;
            for likelihood in &ladder {
                let current = level(likelihood.clone(), impact.clone());
                assert!(
                    current >= previous,
                    "{likelihood:?}/{impact:?} must not be lower than the row below"
                );
                previous = current;
            }
        }
    }

    #[test]
    fn risk_never_decreases_as_impact_rises() {
        let ladder = [
            Impact::Negligible,
            Impact::Minor,
            Impact::Moderate,
            Impact::Major,
            Impact::Catastrophic,
        ];
        let likelihoods = [
            Likelihood::Rare,
            Likelihood::Unlikely,
            Likelihood::Possible,
            Likelihood::Likely,
            Likelihood::AlmostCertain,
        ];
        for likelihood in likelihoods {
            let mut previous = RiskLevel::Low;
            for impact in &ladder {
                let current = level(likelihood.clone(), impact.clone());
                assert!(
                    current >= previous,
                    "{likelihood:?}/{impact:?} must not be lower than the column to its left"
                );
                previous = current;
            }
        }
    }

    #[test]
    fn a_known_pair_returns_its_documented_level() {
        // ISO 31000 style: possible + major is high.
        assert_eq!(level(Likelihood::Possible, Impact::Major), RiskLevel::High);
        // likely + moderate is high.
        assert_eq!(level(Likelihood::Likely, Impact::Moderate), RiskLevel::High);
    }
}
