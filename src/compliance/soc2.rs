use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum TrustServiceCategory {
    Security,
    Availability,
    ProcessingIntegrity,
    Confidentiality,
    Privacy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Soc2Control {
    pub id: String,
    pub category: TrustServiceCategory,
    pub name: String,
    pub description: String,
    pub criteria: Vec<String>,
    pub implementation_status: ImplementationStatus,
    pub evidence_types: Vec<EvidenceType>,
    pub test_frequency: TestFrequency,
    pub last_tested: Option<DateTime<Utc>>,
    pub next_test_due: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ImplementationStatus {
    NotImplemented,
    PartiallyImplemented,
    FullyImplemented,
    NotApplicable,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum EvidenceType {
    Configuration,
    Logs,
    Policy,
    Procedure,
    Screenshot,
    Report,
    Certificate,
    Interview,
    Observation,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TestFrequency {
    Continuous,
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annually,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlTestResult {
    pub control_id: String,
    pub tested_at: DateTime<Utc>,
    pub tested_by: String,
    pub result: TestResult,
    pub findings: Vec<Finding>,
    pub evidence_collected: Vec<Evidence>,
    pub remediation_required: bool,
    pub remediation_deadline: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TestResult {
    Pass,
    PassWithException,
    Fail,
    NotTested,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub id: Uuid,
    pub severity: FindingSeverity,
    pub title: String,
    pub description: String,
    pub impact: String,
    pub recommendation: String,
    pub status: FindingStatus,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FindingSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FindingStatus {
    Open,
    InProgress,
    Resolved,
    Accepted,
    Deferred,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Evidence {
    pub id: Uuid,
    pub evidence_type: EvidenceType,
    pub title: String,
    pub description: String,
    pub collected_at: DateTime<Utc>,
    pub collected_by: String,
    pub storage_path: String,
    pub hash: String,
    pub retention_until: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopLog {
    pub id: Uuid,
    pub procedure_id: String,
    pub procedure_name: String,
    pub executed_by: String,
    pub executed_at: DateTime<Utc>,
    pub steps_completed: Vec<SopStep>,
    pub result: SopResult,
    pub notes: Option<String>,
    pub duration_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SopStep {
    pub step_number: u32,
    pub description: String,
    pub completed: bool,
    pub completed_at: Option<DateTime<Utc>>,
    pub evidence: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum SopResult {
    Success,
    PartialSuccess,
    Failed,
    Aborted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentRecord {
    pub id: Uuid,
    pub incident_type: IncidentType,
    pub severity: FindingSeverity,
    pub title: String,
    pub description: String,
    pub detected_at: DateTime<Utc>,
    pub detected_by: String,
    pub affected_systems: Vec<String>,
    pub affected_users_count: Option<u64>,
    pub status: IncidentStatus,
    pub timeline: Vec<IncidentTimelineEntry>,
    pub root_cause: Option<String>,
    pub remediation_actions: Vec<String>,
    pub lessons_learned: Option<String>,
    pub resolved_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IncidentType {
    SecurityBreach,
    DataLeak,
    ServiceOutage,
    UnauthorizedAccess,
    MalwareDetection,
    PolicyViolation,
    ConfigurationError,
    Other,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IncidentStatus {
    Detected,
    Investigating,
    Containing,
    Eradicating,
    Recovering,
    Resolved,
    PostIncidentReview,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncidentTimelineEntry {
    pub timestamp: DateTime<Utc>,
    pub action: String,
    pub performed_by: String,
    pub notes: Option<String>,
}

pub struct Soc2ComplianceService {
    controls: Arc<RwLock<HashMap<String, Soc2Control>>>,
    test_results: Arc<RwLock<Vec<ControlTestResult>>>,
    sop_logs: Arc<RwLock<Vec<SopLog>>>,
    incidents: Arc<RwLock<Vec<IncidentRecord>>>,
    evidence_store: Arc<RwLock<Vec<Evidence>>>,
}

impl Soc2ComplianceService {
    pub fn new() -> Self {
        let service = Self {
            controls: Arc::new(RwLock::new(HashMap::new())),
            test_results: Arc::new(RwLock::new(Vec::new())),
            sop_logs: Arc::new(RwLock::new(Vec::new())),
            incidents: Arc::new(RwLock::new(Vec::new())),
            evidence_store: Arc::new(RwLock::new(Vec::new())),
        };

        tokio::spawn({
            let controls = service.controls.clone();
            async move {
                let mut guard = controls.write().await;
                for control in Self::get_default_controls() {
                    guard.insert(control.id.clone(), control);
                }
            }
        });

        service
    }

    fn get_default_controls() -> Vec<Soc2Control> {
        vec![
            Soc2Control {
                id: "CC1.1".to_string(),
                category: TrustServiceCategory::Security,
                name: "Control Environment".to_string(),
                description: "The entity demonstrates a commitment to integrity and ethical values".to_string(),
                criteria: vec![
                    "Code of conduct established".to_string(),
                    "Ethics training completed".to_string(),
                    "Whistleblower policy in place".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Policy, EvidenceType::Certificate],
                test_frequency: TestFrequency::Annually,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "CC2.1".to_string(),
                category: TrustServiceCategory::Security,
                name: "Communication and Information".to_string(),
                description: "The entity obtains or generates relevant information to support internal control".to_string(),
                criteria: vec![
                    "Security policies communicated".to_string(),
                    "Security awareness training".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Policy, EvidenceType::Certificate],
                test_frequency: TestFrequency::Quarterly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "CC3.1".to_string(),
                category: TrustServiceCategory::Security,
                name: "Risk Assessment".to_string(),
                description: "The entity specifies objectives with sufficient clarity to identify risks".to_string(),
                criteria: vec![
                    "Risk assessment performed".to_string(),
                    "Risk register maintained".to_string(),
                    "Risk treatment plans defined".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Report, EvidenceType::Policy],
                test_frequency: TestFrequency::Annually,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "CC4.1".to_string(),
                category: TrustServiceCategory::Security,
                name: "Monitoring Activities".to_string(),
                description: "The entity selects and develops monitoring activities".to_string(),
                criteria: vec![
                    "Continuous monitoring implemented".to_string(),
                    "Security metrics defined".to_string(),
                    "Alert thresholds configured".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Configuration, EvidenceType::Logs],
                test_frequency: TestFrequency::Monthly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "CC5.1".to_string(),
                category: TrustServiceCategory::Security,
                name: "Control Activities".to_string(),
                description: "The entity selects and develops control activities".to_string(),
                criteria: vec![
                    "Access controls implemented".to_string(),
                    "Change management process".to_string(),
                    "Segregation of duties".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Configuration, EvidenceType::Procedure],
                test_frequency: TestFrequency::Quarterly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "CC6.1".to_string(),
                category: TrustServiceCategory::Security,
                name: "Logical and Physical Access Controls".to_string(),
                description: "The entity implements logical access security measures".to_string(),
                criteria: vec![
                    "Authentication mechanisms".to_string(),
                    "MFA for privileged access".to_string(),
                    "Role-based access control".to_string(),
                    "Access reviews performed".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Configuration, EvidenceType::Logs, EvidenceType::Report],
                test_frequency: TestFrequency::Monthly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "CC6.2".to_string(),
                category: TrustServiceCategory::Security,
                name: "User Registration and Authorization".to_string(),
                description: "Prior to issuing system credentials, the entity registers authorized users".to_string(),
                criteria: vec![
                    "User provisioning process".to_string(),
                    "Access request workflow".to_string(),
                    "Manager approval required".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Procedure, EvidenceType::Logs],
                test_frequency: TestFrequency::Monthly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "CC6.3".to_string(),
                category: TrustServiceCategory::Security,
                name: "User Deprovisioning".to_string(),
                description: "The entity removes access when no longer required".to_string(),
                criteria: vec![
                    "Termination checklist".to_string(),
                    "Automated deprovisioning".to_string(),
                    "Access removal within 24 hours".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Procedure, EvidenceType::Logs],
                test_frequency: TestFrequency::Monthly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "CC6.6".to_string(),
                category: TrustServiceCategory::Security,
                name: "System Boundary Protection".to_string(),
                description: "The entity implements measures to protect system boundaries".to_string(),
                criteria: vec![
                    "Firewall configuration".to_string(),
                    "Network segmentation".to_string(),
                    "Intrusion detection".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Configuration, EvidenceType::Logs],
                test_frequency: TestFrequency::Monthly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "CC6.7".to_string(),
                category: TrustServiceCategory::Security,
                name: "Encryption".to_string(),
                description: "The entity restricts transmission and movement of data to authorized parties".to_string(),
                criteria: vec![
                    "TLS 1.3 for transit".to_string(),
                    "AES-256 for data at rest".to_string(),
                    "Key management procedures".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Configuration, EvidenceType::Certificate],
                test_frequency: TestFrequency::Quarterly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "CC7.1".to_string(),
                category: TrustServiceCategory::Security,
                name: "System Operations".to_string(),
                description: "The entity detects and monitors security events".to_string(),
                criteria: vec![
                    "Security monitoring configured".to_string(),
                    "Log aggregation".to_string(),
                    "Alerting mechanisms".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Configuration, EvidenceType::Logs],
                test_frequency: TestFrequency::Continuous,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "CC7.2".to_string(),
                category: TrustServiceCategory::Security,
                name: "Incident Management".to_string(),
                description: "The entity monitors system components for anomalies".to_string(),
                criteria: vec![
                    "Incident response plan".to_string(),
                    "Incident classification".to_string(),
                    "Escalation procedures".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Policy, EvidenceType::Procedure],
                test_frequency: TestFrequency::Quarterly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "CC7.3".to_string(),
                category: TrustServiceCategory::Security,
                name: "Incident Response".to_string(),
                description: "The entity evaluates security events and responds appropriately".to_string(),
                criteria: vec![
                    "Response procedures documented".to_string(),
                    "Communication protocols".to_string(),
                    "Post-incident review".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Procedure, EvidenceType::Report],
                test_frequency: TestFrequency::Quarterly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "CC7.4".to_string(),
                category: TrustServiceCategory::Security,
                name: "Incident Recovery".to_string(),
                description: "The entity responds to identified security incidents".to_string(),
                criteria: vec![
                    "Recovery procedures".to_string(),
                    "Business continuity plan".to_string(),
                    "Disaster recovery testing".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Procedure, EvidenceType::Report],
                test_frequency: TestFrequency::Annually,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "CC8.1".to_string(),
                category: TrustServiceCategory::Security,
                name: "Change Management".to_string(),
                description: "The entity authorizes, designs, develops, configures, and tests changes".to_string(),
                criteria: vec![
                    "Change request process".to_string(),
                    "Change approval workflow".to_string(),
                    "Testing requirements".to_string(),
                    "Rollback procedures".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Procedure, EvidenceType::Logs],
                test_frequency: TestFrequency::Monthly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "CC9.1".to_string(),
                category: TrustServiceCategory::Security,
                name: "Risk Mitigation".to_string(),
                description: "The entity identifies and mitigates risks from business disruptions".to_string(),
                criteria: vec![
                    "Business impact analysis".to_string(),
                    "Vendor risk management".to_string(),
                    "Insurance coverage".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Report, EvidenceType::Policy],
                test_frequency: TestFrequency::Annually,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "A1.1".to_string(),
                category: TrustServiceCategory::Availability,
                name: "System Availability".to_string(),
                description: "The entity maintains system availability commitments".to_string(),
                criteria: vec![
                    "SLA defined".to_string(),
                    "Uptime monitoring".to_string(),
                    "Capacity planning".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Report, EvidenceType::Configuration],
                test_frequency: TestFrequency::Monthly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "A1.2".to_string(),
                category: TrustServiceCategory::Availability,
                name: "Backup and Recovery".to_string(),
                description: "The entity implements backup and recovery procedures".to_string(),
                criteria: vec![
                    "Backup schedule defined".to_string(),
                    "Backup verification".to_string(),
                    "Recovery testing".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Configuration, EvidenceType::Logs],
                test_frequency: TestFrequency::Monthly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "PI1.1".to_string(),
                category: TrustServiceCategory::ProcessingIntegrity,
                name: "Processing Integrity".to_string(),
                description: "The entity implements procedures to achieve processing objectives".to_string(),
                criteria: vec![
                    "Input validation".to_string(),
                    "Processing controls".to_string(),
                    "Output verification".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Configuration, EvidenceType::Logs],
                test_frequency: TestFrequency::Monthly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "C1.1".to_string(),
                category: TrustServiceCategory::Confidentiality,
                name: "Confidential Information Protection".to_string(),
                description: "The entity protects confidential information".to_string(),
                criteria: vec![
                    "Data classification".to_string(),
                    "Access restrictions".to_string(),
                    "Encryption requirements".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Policy, EvidenceType::Configuration],
                test_frequency: TestFrequency::Quarterly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "C1.2".to_string(),
                category: TrustServiceCategory::Confidentiality,
                name: "Confidential Information Disposal".to_string(),
                description: "The entity disposes of confidential information".to_string(),
                criteria: vec![
                    "Retention policy".to_string(),
                    "Secure deletion".to_string(),
                    "Certificate of destruction".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Policy, EvidenceType::Certificate],
                test_frequency: TestFrequency::Annually,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "P1.1".to_string(),
                category: TrustServiceCategory::Privacy,
                name: "Privacy Notice".to_string(),
                description: "The entity provides notice about privacy practices".to_string(),
                criteria: vec![
                    "Privacy policy published".to_string(),
                    "Data collection disclosed".to_string(),
                    "Third-party sharing disclosed".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Policy, EvidenceType::Screenshot],
                test_frequency: TestFrequency::Annually,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "P2.1".to_string(),
                category: TrustServiceCategory::Privacy,
                name: "Choice and Consent".to_string(),
                description: "The entity obtains consent for data collection".to_string(),
                criteria: vec![
                    "Consent mechanism".to_string(),
                    "Opt-out capability".to_string(),
                    "Consent records".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Configuration, EvidenceType::Logs],
                test_frequency: TestFrequency::Quarterly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "P3.1".to_string(),
                category: TrustServiceCategory::Privacy,
                name: "Data Collection".to_string(),
                description: "The entity collects personal information for specified purposes".to_string(),
                criteria: vec![
                    "Purpose limitation".to_string(),
                    "Data minimization".to_string(),
                    "Collection methods documented".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Policy, EvidenceType::Configuration],
                test_frequency: TestFrequency::Quarterly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "P4.1".to_string(),
                category: TrustServiceCategory::Privacy,
                name: "Data Use and Retention".to_string(),
                description: "The entity limits data use and retention to stated purposes".to_string(),
                criteria: vec![
                    "Use limitation".to_string(),
                    "Retention schedule".to_string(),
                    "Automated deletion".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Policy, EvidenceType::Configuration],
                test_frequency: TestFrequency::Quarterly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "P5.1".to_string(),
                category: TrustServiceCategory::Privacy,
                name: "Data Subject Access".to_string(),
                description: "The entity provides data subjects access to their data".to_string(),
                criteria: vec![
                    "Access request process".to_string(),
                    "Data export capability".to_string(),
                    "Response within timeframe".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Procedure, EvidenceType::Configuration],
                test_frequency: TestFrequency::Quarterly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "P6.1".to_string(),
                category: TrustServiceCategory::Privacy,
                name: "Third-Party Disclosure".to_string(),
                description: "The entity discloses personal information to third parties with consent".to_string(),
                criteria: vec![
                    "Third-party agreements".to_string(),
                    "Disclosure tracking".to_string(),
                    "Privacy impact assessment".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Policy, EvidenceType::Report],
                test_frequency: TestFrequency::Quarterly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "P7.1".to_string(),
                category: TrustServiceCategory::Privacy,
                name: "Data Quality".to_string(),
                description: "The entity maintains accurate personal information".to_string(),
                criteria: vec![
                    "Data quality controls".to_string(),
                    "Update mechanisms".to_string(),
                    "Correction procedures".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Procedure, EvidenceType::Configuration],
                test_frequency: TestFrequency::Quarterly,
                last_tested: None,
                next_test_due: None,
            },
            Soc2Control {
                id: "P8.1".to_string(),
                category: TrustServiceCategory::Privacy,
                name: "Privacy Complaints".to_string(),
                description: "The entity addresses privacy-related complaints".to_string(),
                criteria: vec![
                    "Complaint process".to_string(),
                    "Investigation procedures".to_string(),
                    "Resolution tracking".to_string(),
                ],
                implementation_status: ImplementationStatus::FullyImplemented,
                evidence_types: vec![EvidenceType::Procedure, EvidenceType::Logs],
                test_frequency: TestFrequency::Quarterly,
                last_tested: None,
                next_test_due: None,
            },
        ]
    }

    pub async fn get_control(&self, control_id: &str) -> Option<Soc2Control> {
        let controls = self.controls.read().await;
        controls.get(control_id).cloned()
    }

    pub async fn get_all_controls(&self) -> Vec<Soc2Control> {
        let controls = self.controls.read().await;
        controls.values().cloned().collect()
    }

    pub async fn get_controls_by_category(&self, category: TrustServiceCategory) -> Vec<Soc2Control> {
        let controls = self.controls.read().await;
        controls
            .values()
            .filter(|c| c.category == category)
            .cloned()
            .collect()
    }

    pub async fn record_test_result(&self, result: ControlTestResult) {
        let mut test_results = self.test_results.write().await;
        test_results.push(result.clone());

        let mut controls = self.controls.write().await;
        if let Some(control) = controls.get_mut(&result.control_id) {
            control.last_tested = Some(result.tested_at);
            control.next_test_due = Some(self.calculate_next_test_due(control.test_frequency));
        }
    }

    pub async fn get_test_results(&self, control_id: &str) -> Vec<ControlTestResult> {
        let test_results = self.test_results.read().await;
        test_results
            .iter()
            .filter(|r| r.control_id == control_id)
            .cloned()
            .collect()
    }

    pub async fn log_sop_execution(&self, sop_log: SopLog) {
        let mut logs = self.sop_logs.write().await;
        logs.push(sop_log);
    }

    pub async fn get_sop_logs(&self, procedure_id: Option<&str>, limit: usize) -> Vec<SopLog> {
        let logs = self.sop_logs.read().await;
        let filtered: Vec<SopLog> = match procedure_id {
            Some(id) => logs.iter().filter(|l| l.procedure_id == id).cloned().collect(),
            None => logs.iter().cloned().collect(),
        };
        filtered.into_iter().rev().take(limit).collect()
    }

    pub async fn record_incident(&self, incident: IncidentRecord) {
        let mut incidents = self.incidents.write().await;
        incidents.push(incident);
    }

    pub async fn update_incident(&self, incident_id: Uuid, status: IncidentStatus, action: &str, performed_by: &str) {
        let mut incidents = self.incidents.write().await;
        if let Some(incident) = incidents.iter_mut().find(|i| i.id == incident_id) {
            incident.status = status;
            incident.timeline.push(IncidentTimelineEntry {
                timestamp: Utc::now(),
                action: action.to_string(),
                performed_by: performed_by.to_string(),
                notes: None,
            });
            if status == IncidentStatus::Closed || status == IncidentStatus::Resolved {
                incident.resolved_at = Some(Utc::now());
            }
        }
    }

    pub async fn get_open_incidents(&self) -> Vec<IncidentRecord> {
        let incidents = self.incidents.read().await;
        incidents
            .iter()
            .filter(|i| !matches!(i.status, IncidentStatus::Closed | IncidentStatus::Resolved))
            .cloned()
            .collect()
    }

    pub async fn get_incidents_by_severity(&self, severity: FindingSeverity) -> Vec<IncidentRecord> {
        let incidents = self.incidents.read().await;
        incidents
            .iter()
            .filter(|i| i.severity == severity)
            .cloned()
            .collect()
    }

    pub async fn store_evidence(&self, evidence: Evidence) {
        let mut store = self.evidence_store.write().await;
        store.push(evidence);
    }

    pub async fn get_evidence_for_control(&self, control_id: &str) -> Vec<Evidence> {
        let test_results = self.test_results.read().await;
        let evidence_ids: Vec<Uuid> = test_results
            .iter()
            .filter(|r| r.control_id == control_id)
            .flat_map(|r| r.evidence_collected.iter().map(|e| e.id))
            .collect();

        let store = self.evidence_store.read().await;
        store
            .iter()
            .filter(|e| evidence_ids.contains(&e.id))
            .cloned()
            .collect()
    }

    pub async fn generate_compliance_report(&self) -> Soc2ComplianceReport {
        let controls = self.controls.read().await;
        let test_results = self.test_results.read().await;
        let incidents = self.incidents.read().await;

        let total_controls = controls.len();
        let implemented = controls
            .values()
            .filter(|c| c.implementation_status == ImplementationStatus::FullyImplemented)
            .count();
        let partially_implemented = controls
            .values()
            .filter(|c| c.implementation_status == ImplementationStatus::PartiallyImplemented)
            .count();

        let recent_tests: Vec<ControlTestResult> = test_results
            .iter()
            .rev()
            .take(50)
            .cloned()
            .collect();

        let passed_tests = recent_tests.iter().filter(|r| r.result == TestResult::Pass).count();
        let failed_tests = recent_tests.iter().filter(|r| r.result == TestResult::Fail).count();

        let open_findings: Vec<Finding> = recent_tests
            .iter()
            .flat_map(|r| r.findings.iter())
            .filter(|f| f.status == FindingStatus::Open || f.status == FindingStatus::InProgress)
            .cloned()
            .collect();

        let open_incidents = incidents
            .iter()
            .filter(|i| !matches!(i.status, IncidentStatus::Closed | IncidentStatus::Resolved))
            .count();

        let compliance_score = if total_controls > 0 {
            ((implemented as f64 + (partially_implemented as f64 * 0.5)) / total_controls as f64) * 100.0
        } else {
            0.0
        };

        Soc2ComplianceReport {
            generated_at: Utc::now(),
            report_period_start: Utc::now() - chrono::Duration::days(365),
            report_period_end: Utc::now(),
            total_controls,
            implemented_controls: implemented,
            partially_implemented_controls: partially_implemented,
            not_implemented_controls: total_controls - implemented - partially_implemented,
            compliance_score,
            tests_performed: recent_tests.len(),
            tests_passed: passed_tests,
            tests_failed: failed_tests,
            open_findings: open_findings.len(),
            critical_findings: open_findings.iter().filter(|f| f.severity == FindingSeverity::Critical).count(),
            high_findings: open_findings.iter().filter(|f| f.severity == FindingSeverity::High).count(),
            open_incidents,
            controls_by_category: self.count_by_category(&controls),
        }
    }

    fn count_by_category(&self, controls: &HashMap<String, Soc2Control>) -> HashMap<TrustServiceCategory, CategoryStats> {
        let mut stats: HashMap<TrustServiceCategory, CategoryStats> = HashMap::new();

        for control in controls.values() {
            let entry = stats.entry(control.category).or_insert(CategoryStats {
                total: 0,
                implemented: 0,
                partially_implemented: 0,
                not_implemented: 0,
            });
            entry.total += 1;
            match control.implementation_status {
                ImplementationStatus::FullyImplemented => entry.implemented += 1,
                ImplementationStatus::PartiallyImplemented => entry.partially_implemented += 1,
                ImplementationStatus::NotImplemented => entry.not_implemented += 1,
                ImplementationStatus::NotApplicable => {}
            }
        }

        stats
    }

    fn calculate_next_test_due(&self, frequency: TestFrequency) -> DateTime<Utc> {
        let now = Utc::now();
        match frequency {
            TestFrequency::Continuous => now,
            TestFrequency::Daily => now + chrono::Duration::days(1),
            TestFrequency::Weekly => now + chrono::Duration::weeks(1),
            TestFrequency::Monthly => now + chrono::Duration::days(30),
            TestFrequency::Quarterly => now + chrono::Duration::days(90),
            TestFrequency::Annually => now + chrono::Duration::days(365),
        }
    }

    pub async fn get_overdue_tests(&self) -> Vec<Soc2Control> {
        let controls = self.controls.read().await;
        let now = Utc::now();
        controls
            .values()
            .filter(|c| c.next_test_due.map(|d| d < now).unwrap_or(false))
            .cloned()
            .collect()
    }
}

impl Default for Soc2ComplianceService {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Soc2ComplianceReport {
    pub generated_at: DateTime<Utc>,
    pub report_period_start: DateTime<Utc>,
    pub report_period_end: DateTime<Utc>,
    pub total_controls: usize,
    pub implemented_controls: usize,
    pub partially_implemented_controls: usize,
    pub not_implemented_controls: usize,
    pub compliance_score: f64,
    pub tests_performed: usize,
    pub tests_passed: usize,
    pub tests_failed: usize,
    pub open_findings: usize,
    pub critical_findings: usize,
    pub high_findings: usize,
    pub open_incidents: usize,
    pub controls_by_category: HashMap<TrustServiceCategory, CategoryStats>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CategoryStats {
    pub total: usize,
    pub implemented: usize,
    pub partially_implemented: usize,
    pub not_implemented: usize,
}

pub fn create_sop_log(
    procedure_id: &str,
    procedure_name: &str,
    executed_by: &str,
    steps: Vec<SopStep>,
    result: SopResult,
    duration_seconds: u64,
) -> SopLog {
    SopLog {
        id: Uuid::new_v4(),
        procedure_id: procedure_id.to_string(),
        procedure_name: procedure_name.to_string(),
        executed_by: executed_by.to_string(),
        executed_at: Utc::now(),
        steps_completed: steps,
        result,
        notes: None,
        duration_seconds,
    }
}

pub fn create_incident(
    incident_type: IncidentType,
    severity: FindingSeverity,
    title: &str,
    description: &str,
    detected_by: &str,
    affected_systems: Vec<String>,
) -> IncidentRecord {
    IncidentRecord {
        id: Uuid::new_v4(),
        incident_type,
        severity,
        title: title.to_string(),
        description: description.to_string(),
        detected_at: Utc::now(),
        detected_by: detected_by.to_string(),
        affected_systems,
        affected_users_count: None,
        status: IncidentStatus::Detected,
        timeline: vec![IncidentTimelineEntry {
            timestamp: Utc::now(),
            action: "Incident detected".to_string(),
            performed_by: detected_by.to_string(),
            notes: None,
        }],
        root_cause: None,
        remediation_actions: vec![],
        lessons_learned: None,
        resolved_at: None,
    }
}
