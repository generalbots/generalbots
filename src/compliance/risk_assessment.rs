//! Risk Assessment Module
//!
//! Provides comprehensive risk assessment and management capabilities
//! for identifying, evaluating, and mitigating security risks.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Risk category enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskCategory {
    Security,
    Compliance,
    Operational,
    Financial,
    Reputational,
    Technical,
    Legal,
}

/// Risk likelihood levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Likelihood {
    Rare,
    Unlikely,
    Possible,
    Likely,
    AlmostCertain,
}

/// Risk impact levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Impact {
    Negligible,
    Minor,
    Moderate,
    Major,
    Catastrophic,
}

/// Risk level based on likelihood and impact
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Risk status
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RiskStatus {
    Identified,
    Assessed,
    Mitigating,
    Monitoring,
    Accepted,
    Closed,
}

/// Risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub category: RiskCategory,
    pub likelihood: Likelihood,
    pub impact: Impact,
    pub risk_level: RiskLevel,
    pub status: RiskStatus,
    pub identified_date: DateTime<Utc>,
    pub assessed_date: Option<DateTime<Utc>>,
    pub owner: String,
    pub affected_assets: Vec<String>,
    pub vulnerabilities: Vec<Vulnerability>,
    pub threats: Vec<Threat>,
    pub controls: Vec<Control>,
    pub residual_risk: Option<RiskLevel>,
}

/// Vulnerability definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub severity: RiskLevel,
    pub cve_id: Option<String>,
    pub discovered_date: DateTime<Utc>,
    pub patched: bool,
}

/// Threat definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Threat {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub threat_actor: String,
    pub likelihood: Likelihood,
    pub tactics: Vec<String>,
}

/// Control measure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Control {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub control_type: ControlType,
    pub effectiveness: Effectiveness,
    pub implementation_status: ImplementationStatus,
    pub cost: f64,
}

/// Control type
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlType {
    Preventive,
    Detective,
    Corrective,
    Compensating,
}

/// Control effectiveness
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Effectiveness {
    Ineffective,
    PartiallyEffective,
    Effective,
    HighlyEffective,
}

/// Implementation status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ImplementationStatus {
    Planned,
    InProgress,
    Implemented,
    Verified,
}

/// Risk mitigation plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationPlan {
    pub id: Uuid,
    pub risk_id: Uuid,
    pub strategy: MitigationStrategy,
    pub actions: Vec<MitigationAction>,
    pub timeline: Duration,
    pub budget: f64,
    pub responsible_party: String,
    pub approval_status: ApprovalStatus,
}

/// Mitigation strategy
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MitigationStrategy {
    Avoid,
    Transfer,
    Mitigate,
    Accept,
}

/// Mitigation action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitigationAction {
    pub id: Uuid,
    pub description: String,
    pub due_date: DateTime<Utc>,
    pub assigned_to: String,
    pub completed: bool,
}

/// Approval status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApprovalStatus {
    Pending,
    Approved,
    Rejected,
}

/// Risk assessment service
#[derive(Debug, Clone)]
pub struct RiskAssessmentService {
    assessments: HashMap<Uuid, RiskAssessment>,
    mitigation_plans: HashMap<Uuid, MitigationPlan>,
    risk_matrix: RiskMatrix,
}

impl RiskAssessmentService {
    /// Create new risk assessment service
    pub fn new() -> Self {
        Self {
            assessments: HashMap::new(),
            mitigation_plans: HashMap::new(),
            risk_matrix: RiskMatrix::default(),
        }
    }

    /// Create a new risk assessment
    pub fn create_assessment(
        &mut self,
        title: String,
        description: String,
        category: RiskCategory,
        owner: String,
    ) -> Result<RiskAssessment> {
        let assessment = RiskAssessment {
            id: Uuid::new_v4(),
            title,
            description,
            category,
            likelihood: Likelihood::Possible,
            impact: Impact::Moderate,
            risk_level: RiskLevel::Medium,
            status: RiskStatus::Identified,
            identified_date: Utc::now(),
            assessed_date: None,
            owner,
            affected_assets: Vec::new(),
            vulnerabilities: Vec::new(),
            threats: Vec::new(),
            controls: Vec::new(),
            residual_risk: None,
        };

        self.assessments.insert(assessment.id, assessment.clone());
        log::info!("Created risk assessment: {}", assessment.id);

        Ok(assessment)
    }

    /// Assess risk level
    pub fn assess_risk(
        &mut self,
        risk_id: Uuid,
        likelihood: Likelihood,
        impact: Impact,
    ) -> Result<RiskLevel> {
        let assessment = self
            .assessments
            .get_mut(&risk_id)
            .ok_or_else(|| anyhow!("Risk assessment not found"))?;

        assessment.likelihood = likelihood.clone();
        assessment.impact = impact.clone();
        assessment.risk_level = self.risk_matrix.calculate_risk_level(&likelihood, &impact);
        assessment.assessed_date = Some(Utc::now());
        assessment.status = RiskStatus::Assessed;

        log::info!(
            "Assessed risk {}: level = {:?}",
            risk_id,
            assessment.risk_level
        );

        Ok(assessment.risk_level.clone())
    }

    /// Add vulnerability to risk assessment
    pub fn add_vulnerability(&mut self, risk_id: Uuid, vulnerability: Vulnerability) -> Result<()> {
        let assessment = self
            .assessments
            .get_mut(&risk_id)
            .ok_or_else(|| anyhow!("Risk assessment not found"))?;

        assessment.vulnerabilities.push(vulnerability);
        self.recalculate_risk_level(risk_id)?;

        Ok(())
    }

    /// Add threat to risk assessment
    pub fn add_threat(&mut self, risk_id: Uuid, threat: Threat) -> Result<()> {
        let assessment = self
            .assessments
            .get_mut(&risk_id)
            .ok_or_else(|| anyhow!("Risk assessment not found"))?;

        assessment.threats.push(threat);
        self.recalculate_risk_level(risk_id)?;

        Ok(())
    }

    /// Add control to risk assessment
    pub fn add_control(&mut self, risk_id: Uuid, control: Control) -> Result<()> {
        let assessment = self
            .assessments
            .get_mut(&risk_id)
            .ok_or_else(|| anyhow!("Risk assessment not found"))?;

        assessment.controls.push(control);
        self.calculate_residual_risk(risk_id)?;

        Ok(())
    }

    /// Recalculate risk level based on vulnerabilities and threats
    fn recalculate_risk_level(&mut self, risk_id: Uuid) -> Result<()> {
        let assessment = self
            .assessments
            .get_mut(&risk_id)
            .ok_or_else(|| anyhow!("Risk assessment not found"))?;

        // Adjust likelihood based on threats
        if !assessment.threats.is_empty() {
            let max_threat_likelihood = assessment
                .threats
                .iter()
                .map(|t| &t.likelihood)
                .max()
                .cloned()
                .unwrap_or(Likelihood::Possible);

            if max_threat_likelihood > assessment.likelihood {
                assessment.likelihood = max_threat_likelihood;
            }
        }

        // Adjust impact based on vulnerabilities
        if !assessment.vulnerabilities.is_empty() {
            let critical_vulns = assessment
                .vulnerabilities
                .iter()
                .filter(|v| v.severity == RiskLevel::Critical)
                .count();

            if critical_vulns > 0 && assessment.impact < Impact::Major {
                assessment.impact = Impact::Major;
            }
        }

        assessment.risk_level = self
            .risk_matrix
            .calculate_risk_level(&assessment.likelihood, &assessment.impact);

        Ok(())
    }

    /// Calculate residual risk after controls
    fn calculate_residual_risk(&mut self, risk_id: Uuid) -> Result<()> {
        let assessment = self
            .assessments
            .get_mut(&risk_id)
            .ok_or_else(|| anyhow!("Risk assessment not found"))?;

        if assessment.controls.is_empty() {
            assessment.residual_risk = Some(assessment.risk_level.clone());
            return Ok(());
        }

        // Calculate effectiveness of controls
        let effective_controls = assessment
            .controls
            .iter()
            .filter(|c| {
                c.effectiveness == Effectiveness::Effective
                    || c.effectiveness == Effectiveness::HighlyEffective
            })
            .count();

        let residual = match (assessment.risk_level.clone(), effective_controls) {
            (RiskLevel::Critical, n) if n >= 3 => RiskLevel::High,
            (RiskLevel::Critical, n) if n >= 1 => RiskLevel::Critical,
            (RiskLevel::High, n) if n >= 2 => RiskLevel::Medium,
            (RiskLevel::High, n) if n >= 1 => RiskLevel::High,
            (RiskLevel::Medium, n) if n >= 1 => RiskLevel::Low,
            (level, _) => level,
        };

        assessment.residual_risk = Some(residual);

        Ok(())
    }

    /// Create mitigation plan
    pub fn create_mitigation_plan(
        &mut self,
        risk_id: Uuid,
        strategy: MitigationStrategy,
        timeline: Duration,
        budget: f64,
        responsible_party: String,
    ) -> Result<MitigationPlan> {
        if !self.assessments.contains_key(&risk_id) {
            return Err(anyhow!("Risk assessment not found"));
        }

        let plan = MitigationPlan {
            id: Uuid::new_v4(),
            risk_id,
            strategy,
            actions: Vec::new(),
            timeline,
            budget,
            responsible_party,
            approval_status: ApprovalStatus::Pending,
        };

        self.mitigation_plans.insert(plan.id, plan.clone());
        log::info!("Created mitigation plan {} for risk {}", plan.id, risk_id);

        Ok(plan)
    }

    /// Get high-risk assessments
    pub fn get_high_risk_assessments(&self) -> Vec<RiskAssessment> {
        self.assessments
            .values()
            .filter(|a| a.risk_level >= RiskLevel::High)
            .cloned()
            .collect()
    }

    /// Get risk dashboard
    pub fn get_risk_dashboard(&self) -> RiskDashboard {
        let total_risks = self.assessments.len();
        let mut risks_by_level = HashMap::new();
        let mut risks_by_category = HashMap::new();
        let mut risks_by_status = HashMap::new();

        for assessment in self.assessments.values() {
            *risks_by_level
                .entry(assessment.risk_level.clone())
                .or_insert(0) += 1;
            *risks_by_category
                .entry(assessment.category.clone())
                .or_insert(0) += 1;
            *risks_by_status
                .entry(assessment.status.clone())
                .or_insert(0) += 1;
        }

        let mitigation_plans_pending = self
            .mitigation_plans
            .values()
            .filter(|p| p.approval_status == ApprovalStatus::Pending)
            .count();

        RiskDashboard {
            total_risks,
            risks_by_level,
            risks_by_category,
            risks_by_status,
            mitigation_plans_pending,
            last_updated: Utc::now(),
        }
    }
}

/// Risk matrix for calculating risk levels
#[derive(Debug, Clone)]
pub struct RiskMatrix {
    matrix: HashMap<(Likelihood, Impact), RiskLevel>,
}

impl RiskMatrix {
    /// Calculate risk level based on likelihood and impact
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

        // Define risk matrix
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
        matrix.insert((Likelihood::AlmostCertain, Impact::Minor), RiskLevel::High);
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

/// Risk dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskDashboard {
    pub total_risks: usize,
    pub risks_by_level: HashMap<RiskLevel, usize>,
    pub risks_by_category: HashMap<RiskCategory, usize>,
    pub risks_by_status: HashMap<RiskStatus, usize>,
    pub mitigation_plans_pending: usize,
    pub last_updated: DateTime<Utc>,
}

impl Default for RiskAssessmentService {
    fn default() -> Self {
        Self::new()
    }
}
