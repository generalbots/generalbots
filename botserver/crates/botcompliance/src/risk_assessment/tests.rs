//! Offline tests for the risk assessment service: matrix wiring, threat and
//! vulnerability escalation, residual risk with controls, and the dashboard.

use super::*;

fn service() -> RiskAssessmentService {
    RiskAssessmentService::new()
}

fn assessment_id(service: &mut RiskAssessmentService) -> Uuid {
    service
        .create_assessment(
            "Customer data exposure".to_string(),
            "Unencrypted backups".to_string(),
            RiskCategory::Security,
            "ciso".to_string(),
        )
        .expect("assessment creation must succeed")
        .id
}

fn threat(likelihood: Likelihood) -> Threat {
    Threat {
        id: Uuid::new_v4(),
        name: "Credential stuffing".to_string(),
        description: "Automated login attempts".to_string(),
        threat_actor: "external".to_string(),
        likelihood,
        tactics: vec!["T1110".to_string()],
    }
}

fn vulnerability(severity: RiskLevel) -> Vulnerability {
    Vulnerability {
        id: Uuid::new_v4(),
        name: "Unpatched library".to_string(),
        description: "Known CVE".to_string(),
        severity,
        cve_id: Some("CVE-2024-0001".to_string()),
        discovered_date: Utc::now(),
        patched: false,
    }
}

fn control(effectiveness: Effectiveness) -> Control {
    Control {
        id: Uuid::new_v4(),
        name: "WAF".to_string(),
        description: "Blocks malicious payloads".to_string(),
        control_type: ControlType::Preventive,
        effectiveness,
        implementation_status: ImplementationStatus::Implemented,
        cost: 1000.0,
    }
}

#[test]
fn a_new_service_has_no_assessments() {
    let service = service();
    let dashboard = service.get_risk_dashboard();
    assert_eq!(dashboard.total_risks, 0);
    assert!(dashboard.risks_by_level.is_empty());
    assert_eq!(dashboard.mitigation_plans_pending, 0);
    assert!(service.get_high_risk_assessments().is_empty());
}

#[test]
fn a_new_assessment_starts_at_possible_moderate_medium() {
    let mut service = service();
    let id = assessment_id(&mut service);
    let assessment = service
        .assessments
        .get(&id)
        .expect("the assessment is stored");

    assert_eq!(assessment.likelihood, Likelihood::Possible);
    assert_eq!(assessment.impact, Impact::Moderate);
    assert_eq!(assessment.risk_level, RiskLevel::Medium);
    assert_eq!(assessment.status, RiskStatus::Identified);
    assert!(assessment.assessed_date.is_none());
    assert!(assessment.residual_risk.is_none());
    assert_eq!(assessment.owner, "ciso");
}

#[test]
fn assessing_a_risk_uses_the_matrix_and_marks_it_assessed() {
    let mut service = service();
    let id = assessment_id(&mut service);

    let level = service
        .assess_risk(id, Likelihood::AlmostCertain, Impact::Catastrophic)
        .expect("assessing a known risk must succeed");
    assert_eq!(level, RiskLevel::Critical);

    let assessment = service.assessments.get(&id).expect("stored");
    assert_eq!(assessment.risk_level, RiskLevel::Critical);
    assert_eq!(assessment.status, RiskStatus::Assessed);
    assert!(assessment.assessed_date.is_some());
}

#[test]
fn assessing_an_unknown_risk_fails() {
    let mut service = service();
    let error = service
        .assess_risk(Uuid::new_v4(), Likelihood::Rare, Impact::Minor)
        .expect_err("unknown risks cannot be assessed");
    assert!(error.to_string().contains("Risk assessment not found"));
}

#[test]
fn a_threat_can_raise_likelihood_but_never_lower_it() {
    let mut service = service();
    let id = assessment_id(&mut service);
    service
        .assess_risk(id, Likelihood::Possible, Impact::Moderate)
        .expect("assess");

    service
        .add_threat(id, threat(Likelihood::Likely))
        .expect("threats are accepted");
    let raised = service.assessments.get(&id).expect("stored");
    assert_eq!(raised.likelihood, Likelihood::Likely);
    assert_eq!(raised.risk_level, RiskLevel::High, "likely/moderate is high");

    service
        .add_threat(id, threat(Likelihood::Rare))
        .expect("threats are accepted");
    let unchanged = service.assessments.get(&id).expect("stored");
    assert_eq!(
        unchanged.likelihood,
        Likelihood::Likely,
        "a rarer threat must not lower the likelihood"
    );
    assert_eq!(unchanged.risk_level, RiskLevel::High);
}

#[test]
fn a_critical_vulnerability_raises_the_impact_floor_to_major() {
    let mut service = service();
    let id = assessment_id(&mut service);
    service
        .assess_risk(id, Likelihood::Unlikely, Impact::Negligible)
        .expect("assess");

    service
        .add_vulnerability(id, vulnerability(RiskLevel::Critical))
        .expect("vulnerabilities are accepted");
    let raised = service.assessments.get(&id).expect("stored");
    assert_eq!(raised.impact, Impact::Major);
    assert_eq!(
        raised.risk_level,
        RiskLevel::High,
        "unlikely/major is high, not medium"
    );

    service
        .add_vulnerability(id, vulnerability(RiskLevel::Low))
        .expect("vulnerabilities are accepted");
    let unchanged = service.assessments.get(&id).expect("stored");
    assert_eq!(unchanged.impact, Impact::Major, "impact never falls back");
}

#[test]
fn residual_risk_equals_the_inherent_risk_without_controls() {
    let mut service = service();
    let id = assessment_id(&mut service);
    service
        .assess_risk(id, Likelihood::Likely, Impact::Major)
        .expect("assess");

    // Adding a non-effective control still records the inherent level.
    service
        .add_control(id, control(Effectiveness::Ineffective))
        .expect("controls are accepted");
    let assessment = service.assessments.get(&id).expect("stored");
    assert_eq!(assessment.risk_level, RiskLevel::Critical);
    assert_eq!(assessment.residual_risk, Some(RiskLevel::Critical));
}

#[test]
fn effective_controls_reduce_residual_risk_by_the_published_table() {
    let cases = [
        (Likelihood::AlmostCertain, Impact::Catastrophic, 3usize, RiskLevel::High),
        (Likelihood::AlmostCertain, Impact::Catastrophic, 1usize, RiskLevel::Critical),
        (Likelihood::Likely, Impact::Moderate, 2usize, RiskLevel::Medium),
        (Likelihood::Likely, Impact::Moderate, 1usize, RiskLevel::High),
        (Likelihood::Possible, Impact::Minor, 1usize, RiskLevel::Low),
        (Likelihood::Rare, Impact::Negligible, 1usize, RiskLevel::Low),
    ];

    for (likelihood, impact, effective_controls, expected) in cases {
        let mut service = service();
        let id = assessment_id(&mut service);
        let inherent = service
            .assess_risk(id, likelihood.clone(), impact.clone())
            .expect("assess");

        for _ in 0..effective_controls {
            service
                .add_control(id, control(Effectiveness::Effective))
                .expect("controls are accepted");
        }

        let assessment = service.assessments.get(&id).expect("stored");
        assert_eq!(
            assessment.residual_risk,
            Some(expected.clone()),
            "{likelihood:?}/{impact:?} with {effective_controls} effective controls \
             must leave {expected:?} (inherent {inherent:?})"
        );
        assert!(
            expected <= inherent,
            "residual risk must never exceed the inherent risk"
        );
    }
}

#[test]
fn mitigation_plans_require_a_known_risk_and_start_pending() {
    let mut service = service();
    let error = service
        .create_mitigation_plan(
            Uuid::new_v4(),
            MitigationStrategy::Mitigate,
            Duration::days(30),
            5000.0,
            "security team".to_string(),
        )
        .expect_err("plans need an existing risk");
    assert!(error.to_string().contains("Risk assessment not found"));

    let id = assessment_id(&mut service);
    let plan = service
        .create_mitigation_plan(
            id,
            MitigationStrategy::Mitigate,
            Duration::days(30),
            5000.0,
            "security team".to_string(),
        )
        .expect("known risks accept plans");
    assert_eq!(plan.approval_status, ApprovalStatus::Pending);
    assert_eq!(plan.risk_id, id);
    assert!(plan.actions.is_empty());

    let dashboard = service.get_risk_dashboard();
    assert_eq!(dashboard.mitigation_plans_pending, 1);
}

#[test]
fn the_dashboard_groups_risks_by_level_category_and_status() {
    let mut service = service();
    let critical = assessment_id(&mut service);
    service
        .assess_risk(critical, Likelihood::AlmostCertain, Impact::Catastrophic)
        .expect("assess");

    let operational = service
        .create_assessment(
            "Single region".to_string(),
            "No failover".to_string(),
            RiskCategory::Operational,
            "coo".to_string(),
        )
        .expect("assessment creation must succeed")
        .id;
    service
        .assess_risk(operational, Likelihood::Rare, Impact::Minor)
        .expect("assess");

    let dashboard = service.get_risk_dashboard();
    assert_eq!(dashboard.total_risks, 2);
    assert_eq!(dashboard.risks_by_level.get(&RiskLevel::Critical), Some(&1));
    assert_eq!(dashboard.risks_by_level.get(&RiskLevel::Low), Some(&1));
    assert_eq!(dashboard.risks_by_category.get(&RiskCategory::Security), Some(&1));
    assert_eq!(dashboard.risks_by_category.get(&RiskCategory::Operational), Some(&1));
    assert_eq!(dashboard.risks_by_status.get(&RiskStatus::Assessed), Some(&2));

    let high = service.get_high_risk_assessments();
    assert_eq!(high.len(), 1);
    assert_eq!(high[0].id, critical);
}
