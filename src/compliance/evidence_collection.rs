use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceFramework {
    Soc2TypeI,
    Soc2TypeII,
    Gdpr,
    Hipaa,
    Iso27001,
    PciDss,
    Ccpa,
    Nist,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum TrustServiceCriteria {
    Security,
    Availability,
    ProcessingIntegrity,
    Confidentiality,
    Privacy,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceType {
    Policy,
    Procedure,
    Screenshot,
    Log,
    Report,
    Configuration,
    Certificate,
    Attestation,
    AuditReport,
    TrainingRecord,
    AccessReview,
    RiskAssessment,
    IncidentReport,
    ChangeRecord,
    TestResult,
    Inventory,
    Contract,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStatus {
    Draft,
    PendingReview,
    Approved,
    Rejected,
    Expired,
    Superseded,
    Archived,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CollectionMethod {
    Manual,
    Automated,
    Scheduled,
    Triggered,
    Imported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceItem {
    pub id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub evidence_type: EvidenceType,
    pub status: EvidenceStatus,
    pub frameworks: Vec<ComplianceFramework>,
    pub control_ids: Vec<String>,
    pub tsc_categories: Vec<TrustServiceCriteria>,
    pub collection_method: CollectionMethod,
    pub collected_at: DateTime<Utc>,
    pub collected_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<Uuid>,
    pub valid_from: DateTime<Utc>,
    pub valid_until: DateTime<Utc>,
    pub file_path: Option<String>,
    pub file_hash: Option<String>,
    pub file_size_bytes: Option<u64>,
    pub content_type: Option<String>,
    pub source_system: Option<String>,
    pub source_query: Option<String>,
    pub metadata: HashMap<String, String>,
    pub tags: Vec<String>,
    pub version: u32,
    pub previous_version_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMapping {
    pub id: Uuid,
    pub framework: ComplianceFramework,
    pub control_id: String,
    pub control_name: String,
    pub description: String,
    pub tsc_category: Option<TrustServiceCriteria>,
    pub parent_control_id: Option<String>,
    pub required_evidence_types: Vec<EvidenceType>,
    pub collection_frequency_days: u32,
    pub automated_collection: bool,
    pub collection_sources: Vec<CollectionSource>,
    pub validation_rules: Vec<ValidationRule>,
    pub owner_id: Option<Uuid>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSource {
    pub source_type: SourceType,
    pub source_name: String,
    pub connection_config: HashMap<String, String>,
    pub query_template: Option<String>,
    pub transform_script: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Database,
    Api,
    FileSystem,
    CloudProvider,
    IdentityProvider,
    SecurityTool,
    TicketSystem,
    LogAggregator,
    ConfigManagement,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub rule_id: String,
    pub rule_name: String,
    pub rule_type: ValidationRuleType,
    pub condition: String,
    pub error_message: String,
    pub severity: ValidationSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationRuleType {
    Required,
    Format,
    DateRange,
    FileSize,
    ContentMatch,
    Freshness,
    Completeness,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationSeverity {
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionTask {
    pub id: Uuid,
    pub control_id: String,
    pub evidence_type: EvidenceType,
    pub status: TaskStatus,
    pub scheduled_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
    pub retry_count: u32,
    pub max_retries: u32,
    pub result_evidence_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRequest {
    pub id: Uuid,
    pub framework: ComplianceFramework,
    pub control_id: String,
    pub requested_by: Uuid,
    pub requested_at: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub priority: RequestPriority,
    pub status: RequestStatus,
    pub assigned_to: Option<Uuid>,
    pub evidence_ids: Vec<Uuid>,
    pub notes: Option<String>,
    pub audit_period_start: DateTime<Utc>,
    pub audit_period_end: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RequestPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RequestStatus {
    Open,
    InProgress,
    PendingReview,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidencePackage {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub framework: ComplianceFramework,
    pub audit_period_start: DateTime<Utc>,
    pub audit_period_end: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub status: PackageStatus,
    pub evidence_ids: Vec<Uuid>,
    pub control_coverage: HashMap<String, ControlCoverage>,
    pub export_format: Option<ExportFormat>,
    pub exported_at: Option<DateTime<Utc>>,
    pub file_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PackageStatus {
    Draft,
    InProgress,
    Complete,
    Exported,
    Submitted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlCoverage {
    pub control_id: String,
    pub required: bool,
    pub evidence_count: u32,
    pub has_valid_evidence: bool,
    pub last_evidence_date: Option<DateTime<Utc>>,
    pub gaps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    Pdf,
    Zip,
    Excel,
    Json,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSchedule {
    pub id: Uuid,
    pub control_id: String,
    pub evidence_type: EvidenceType,
    pub cron_expression: String,
    pub timezone: String,
    pub enabled: bool,
    pub last_run_at: Option<DateTime<Utc>>,
    pub next_run_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceGap {
    pub control_id: String,
    pub control_name: String,
    pub framework: ComplianceFramework,
    pub required_evidence_type: EvidenceType,
    pub gap_type: GapType,
    pub severity: GapSeverity,
    pub description: String,
    pub recommendation: String,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GapType {
    Missing,
    Expired,
    Incomplete,
    Invalid,
    Stale,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GapSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionReport {
    pub id: Uuid,
    pub generated_at: DateTime<Utc>,
    pub report_period_start: DateTime<Utc>,
    pub report_period_end: DateTime<Utc>,
    pub framework: Option<ComplianceFramework>,
    pub summary: CollectionSummary,
    pub control_status: Vec<ControlStatus>,
    pub gaps: Vec<EvidenceGap>,
    pub collection_metrics: CollectionMetrics,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionSummary {
    pub total_controls: u32,
    pub controls_with_evidence: u32,
    pub controls_missing_evidence: u32,
    pub total_evidence_items: u32,
    pub evidence_collected_this_period: u32,
    pub evidence_expiring_soon: u32,
    pub coverage_percentage: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlStatus {
    pub control_id: String,
    pub control_name: String,
    pub has_evidence: bool,
    pub evidence_count: u32,
    pub latest_evidence_date: Option<DateTime<Utc>>,
    pub evidence_valid: bool,
    pub gaps: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionMetrics {
    pub automated_collections: u32,
    pub manual_collections: u32,
    pub failed_collections: u32,
    pub average_collection_time_seconds: f32,
    pub collection_success_rate: f32,
}

pub struct EvidenceCollectionService {
    evidence: Arc<RwLock<HashMap<Uuid, EvidenceItem>>>,
    control_mappings: Arc<RwLock<HashMap<String, ControlMapping>>>,
    collection_tasks: Arc<RwLock<Vec<CollectionTask>>>,
    evidence_requests: Arc<RwLock<HashMap<Uuid, EvidenceRequest>>>,
    evidence_packages: Arc<RwLock<HashMap<Uuid, EvidencePackage>>>,
    collection_schedules: Arc<RwLock<Vec<CollectionSchedule>>>,
}

impl EvidenceCollectionService {
    pub fn new() -> Self {
        let service = Self {
            evidence: Arc::new(RwLock::new(HashMap::new())),
            control_mappings: Arc::new(RwLock::new(HashMap::new())),
            collection_tasks: Arc::new(RwLock::new(Vec::new())),
            evidence_requests: Arc::new(RwLock::new(HashMap::new())),
            evidence_packages: Arc::new(RwLock::new(HashMap::new())),
            collection_schedules: Arc::new(RwLock::new(Vec::new())),
        };

        tokio::spawn({
            let service_clone = Self {
                evidence: service.evidence.clone(),
                control_mappings: service.control_mappings.clone(),
                collection_tasks: service.collection_tasks.clone(),
                evidence_requests: service.evidence_requests.clone(),
                evidence_packages: service.evidence_packages.clone(),
                collection_schedules: service.collection_schedules.clone(),
            };
            async move {
                service_clone.initialize_soc2_controls().await;
            }
        });

        service
    }

    async fn initialize_soc2_controls(&self) {
        let soc2_controls = vec![
            ControlMapping {
                id: Uuid::new_v4(),
                framework: ComplianceFramework::Soc2TypeII,
                control_id: "CC1.1".to_string(),
                control_name: "COSO Principle 1".to_string(),
                description: "The entity demonstrates a commitment to integrity and ethical values".to_string(),
                tsc_category: Some(TrustServiceCriteria::Security),
                parent_control_id: None,
                required_evidence_types: vec![EvidenceType::Policy, EvidenceType::TrainingRecord],
                collection_frequency_days: 365,
                automated_collection: false,
                collection_sources: Vec::new(),
                validation_rules: vec![
                    ValidationRule {
                        rule_id: "CC1.1-001".to_string(),
                        rule_name: "Code of Conduct Required".to_string(),
                        rule_type: ValidationRuleType::Required,
                        condition: "evidence_type == 'policy'".to_string(),
                        error_message: "Code of Conduct policy is required".to_string(),
                        severity: ValidationSeverity::Error,
                    },
                ],
                owner_id: None,
                enabled: true,
            },
            ControlMapping {
                id: Uuid::new_v4(),
                framework: ComplianceFramework::Soc2TypeII,
                control_id: "CC5.1".to_string(),
                control_name: "Control Activities".to_string(),
                description: "The entity selects and develops control activities that contribute to the mitigation of risks".to_string(),
                tsc_category: Some(TrustServiceCriteria::Security),
                parent_control_id: None,
                required_evidence_types: vec![EvidenceType::Procedure, EvidenceType::Configuration],
                collection_frequency_days: 90,
                automated_collection: true,
                collection_sources: vec![
                    CollectionSource {
                        source_type: SourceType::ConfigManagement,
                        source_name: "Security Configuration".to_string(),
                        connection_config: HashMap::new(),
                        query_template: Some("SELECT * FROM security_configs".to_string()),
                        transform_script: None,
                        enabled: true,
                    },
                ],
                validation_rules: Vec::new(),
                owner_id: None,
                enabled: true,
            },
            ControlMapping {
                id: Uuid::new_v4(),
                framework: ComplianceFramework::Soc2TypeII,
                control_id: "CC6.1".to_string(),
                control_name: "Logical and Physical Access Controls".to_string(),
                description: "The entity implements logical access security measures to protect against unauthorized access".to_string(),
                tsc_category: Some(TrustServiceCriteria::Security),
                parent_control_id: None,
                required_evidence_types: vec![EvidenceType::AccessReview, EvidenceType::Configuration, EvidenceType::Log],
                collection_frequency_days: 30,
                automated_collection: true,
                collection_sources: vec![
                    CollectionSource {
                        source_type: SourceType::IdentityProvider,
                        source_name: "Access Logs".to_string(),
                        connection_config: HashMap::new(),
                        query_template: Some("SELECT * FROM access_logs WHERE timestamp > :start_date".to_string()),
                        transform_script: None,
                        enabled: true,
                    },
                ],
                validation_rules: vec![
                    ValidationRule {
                        rule_id: "CC6.1-001".to_string(),
                        rule_name: "Access Review Freshness".to_string(),
                        rule_type: ValidationRuleType::Freshness,
                        condition: "age_days <= 30".to_string(),
                        error_message: "Access review must be performed within the last 30 days".to_string(),
                        severity: ValidationSeverity::Error,
                    },
                ],
                owner_id: None,
                enabled: true,
            },
            ControlMapping {
                id: Uuid::new_v4(),
                framework: ComplianceFramework::Soc2TypeII,
                control_id: "CC6.2".to_string(),
                control_name: "Access Provisioning".to_string(),
                description: "Prior to issuing system credentials, the entity registers and authorizes new users".to_string(),
                tsc_category: Some(TrustServiceCriteria::Security),
                parent_control_id: Some("CC6.1".to_string()),
                required_evidence_types: vec![EvidenceType::Procedure, EvidenceType::ChangeRecord],
                collection_frequency_days: 30,
                automated_collection: true,
                collection_sources: Vec::new(),
                validation_rules: Vec::new(),
                owner_id: None,
                enabled: true,
            },
            ControlMapping {
                id: Uuid::new_v4(),
                framework: ComplianceFramework::Soc2TypeII,
                control_id: "CC6.3".to_string(),
                control_name: "Access Removal".to_string(),
                description: "The entity removes access to protected information assets when appropriate".to_string(),
                tsc_category: Some(TrustServiceCriteria::Security),
                parent_control_id: Some("CC6.1".to_string()),
                required_evidence_types: vec![EvidenceType::Procedure, EvidenceType::ChangeRecord],
                collection_frequency_days: 30,
                automated_collection: true,
                collection_sources: Vec::new(),
                validation_rules: Vec::new(),
                owner_id: None,
                enabled: true,
            },
            ControlMapping {
                id: Uuid::new_v4(),
                framework: ComplianceFramework::Soc2TypeII,
                control_id: "CC7.1".to_string(),
                control_name: "System Operations".to_string(),
                description: "To meet its objectives, the entity uses detection and monitoring procedures".to_string(),
                tsc_category: Some(TrustServiceCriteria::Security),
                parent_control_id: None,
                required_evidence_types: vec![EvidenceType::Log, EvidenceType::Report, EvidenceType::Configuration],
                collection_frequency_days: 7,
                automated_collection: true,
                collection_sources: vec![
                    CollectionSource {
                        source_type: SourceType::LogAggregator,
                        source_name: "Security Monitoring".to_string(),
                        connection_config: HashMap::new(),
                        query_template: Some("SELECT * FROM security_events".to_string()),
                        transform_script: None,
                        enabled: true,
                    },
                ],
                validation_rules: Vec::new(),
                owner_id: None,
                enabled: true,
            },
            ControlMapping {
                id: Uuid::new_v4(),
                framework: ComplianceFramework::Soc2TypeII,
                control_id: "CC7.2".to_string(),
                control_name: "Incident Response".to_string(),
                description: "The entity monitors system components for anomalies and evaluates events to determine incidents".to_string(),
                tsc_category: Some(TrustServiceCriteria::Security),
                parent_control_id: Some("CC7.1".to_string()),
                required_evidence_types: vec![EvidenceType::IncidentReport, EvidenceType::Procedure],
                collection_frequency_days: 30,
                automated_collection: true,
                collection_sources: Vec::new(),
                validation_rules: Vec::new(),
                owner_id: None,
                enabled: true,
            },
            ControlMapping {
                id: Uuid::new_v4(),
                framework: ComplianceFramework::Soc2TypeII,
                control_id: "CC8.1".to_string(),
                control_name: "Change Management".to_string(),
                description: "The entity authorizes, designs, develops, configures, tests, and implements changes".to_string(),
                tsc_category: Some(TrustServiceCriteria::ProcessingIntegrity),
                parent_control_id: None,
                required_evidence_types: vec![EvidenceType::ChangeRecord, EvidenceType::Procedure, EvidenceType::TestResult],
                collection_frequency_days: 30,
                automated_collection: true,
                collection_sources: Vec::new(),
                validation_rules: Vec::new(),
                owner_id: None,
                enabled: true,
            },
            ControlMapping {
                id: Uuid::new_v4(),
                framework: ComplianceFramework::Soc2TypeII,
                control_id: "CC9.1".to_string(),
                control_name: "Risk Mitigation".to_string(),
                description: "The entity identifies, selects, and develops risk mitigation activities".to_string(),
                tsc_category: Some(TrustServiceCriteria::Security),
                parent_control_id: None,
                required_evidence_types: vec![EvidenceType::RiskAssessment, EvidenceType::Report],
                collection_frequency_days: 90,
                automated_collection: false,
                collection_sources: Vec::new(),
                validation_rules: Vec::new(),
                owner_id: None,
                enabled: true,
            },
            ControlMapping {
                id: Uuid::new_v4(),
                framework: ComplianceFramework::Soc2TypeII,
                control_id: "A1.1".to_string(),
                control_name: "Availability Commitments".to_string(),
                description: "The entity maintains, monitors, and evaluates current processing capacity and availability".to_string(),
                tsc_category: Some(TrustServiceCriteria::Availability),
                parent_control_id: None,
                required_evidence_types: vec![EvidenceType::Report, EvidenceType::Configuration],
                collection_frequency_days: 30,
                automated_collection: true,
                collection_sources: Vec::new(),
                validation_rules: Vec::new(),
                owner_id: None,
                enabled: true,
            },
            ControlMapping {
                id: Uuid::new_v4(),
                framework: ComplianceFramework::Soc2TypeII,
                control_id: "A1.2".to_string(),
                control_name: "Backup and Recovery".to_string(),
                description: "The entity authorizes, designs, and tests recovery procedures".to_string(),
                tsc_category: Some(TrustServiceCriteria::Availability),
                parent_control_id: Some("A1.1".to_string()),
                required_evidence_types: vec![EvidenceType::TestResult, EvidenceType::Procedure, EvidenceType::Report],
                collection_frequency_days: 30,
                automated_collection: true,
                collection_sources: Vec::new(),
                validation_rules: Vec::new(),
                owner_id: None,
                enabled: true,
            },
            ControlMapping {
                id: Uuid::new_v4(),
                framework: ComplianceFramework::Soc2TypeII,
                control_id: "C1.1".to_string(),
                control_name: "Confidentiality Commitments".to_string(),
                description: "The entity identifies and maintains confidential information".to_string(),
                tsc_category: Some(TrustServiceCriteria::Confidentiality),
                parent_control_id: None,
                required_evidence_types: vec![EvidenceType::Policy, EvidenceType::Inventory],
                collection_frequency_days: 90,
                automated_collection: false,
                collection_sources: Vec::new(),
                validation_rules: Vec::new(),
                owner_id: None,
                enabled: true,
            },
            ControlMapping {
                id: Uuid::new_v4(),
                framework: ComplianceFramework::Soc2TypeII,
                control_id: "PI1.1".to_string(),
                control_name: "Processing Integrity Policies".to_string(),
                description: "The entity obtains or generates, uses, and communicates relevant quality information".to_string(),
                tsc_category: Some(TrustServiceCriteria::ProcessingIntegrity),
                parent_control_id: None,
                required_evidence_types: vec![EvidenceType::Policy, EvidenceType::Report],
                collection_frequency_days: 90,
                automated_collection: false,
                collection_sources: Vec::new(),
                validation_rules: Vec::new(),
                owner_id: None,
                enabled: true,
            },
            ControlMapping {
                id: Uuid::new_v4(),
                framework: ComplianceFramework::Soc2TypeII,
                control_id: "P1.1".to_string(),
                control_name: "Privacy Notice".to_string(),
                description: "The entity provides notice to data subjects about its privacy practices".to_string(),
                tsc_category: Some(TrustServiceCriteria::Privacy),
                parent_control_id: None,
                required_evidence_types: vec![EvidenceType::Policy, EvidenceType::Screenshot],
                collection_frequency_days: 365,
                automated_collection: false,
                collection_sources: Vec::new(),
                validation_rules: Vec::new(),
                owner_id: None,
                enabled: true,
            },
        ];

        let mut mappings = self.control_mappings.write().await;
        for control in soc2_controls {
            mappings.insert(control.control_id.clone(), control);
        }
    }

    pub async fn create_evidence(&self, evidence: EvidenceItem) -> Result<EvidenceItem, CollectionError> {
        let mut evidence_store = self.evidence.write().await;
        evidence_store.insert(evidence.id, evidence.clone());
        Ok(evidence)
    }

    pub async fn get_evidence(&self, id: Uuid) -> Option<EvidenceItem> {
        let evidence_store = self.evidence.read().await;
        evidence_store.get(&id).cloned()
    }

    pub async fn get_all_evidence(&self) -> Vec<EvidenceItem> {
        let evidence_store = self.evidence.read().await;
        evidence_store.values().cloned().collect()
    }

    pub async fn get_evidence_for_control(&self, control_id: &str) -> Vec<EvidenceItem> {
        let evidence_store = self.evidence.read().await;
        evidence_store
            .values()
            .filter(|e| e.control_ids.contains(&control_id.to_string()))
            .cloned()
            .collect()
    }

    pub async fn get_evidence_by_framework(&self, framework: ComplianceFramework) -> Vec<EvidenceItem> {
        let evidence_store = self.evidence.read().await;
        evidence_store
            .values()
            .filter(|e| e.frameworks.contains(&framework))
            .cloned()
            .collect()
    }

    pub async fn update_evidence_status(
        &self,
        id: Uuid,
        status: EvidenceStatus,
        reviewer_id: Option<Uuid>,
    ) -> Result<EvidenceItem, CollectionError> {
        let mut evidence_store = self.evidence.write().await;

        let evidence = evidence_store
            .get_mut(&id)
            .ok_or_else(|| CollectionError::NotFound("Evidence not found".to_string()))?;

        evidence.status = status;
        evidence.updated_at = Utc::now();

        if reviewer_id.is_some() {
            evidence.reviewed_at = Some(Utc::now());
            evidence.reviewed_by = reviewer_id;
        }

        Ok(evidence.clone())
    }

    pub async fn collect_automated_evidence(
        &self,
        control_id: &str,
    ) -> Result<EvidenceItem, CollectionError> {
        let mappings = self.control_mappings.read().await;

        let mapping = mappings
            .get(control_id)
            .ok_or_else(|| CollectionError::NotFound("Control mapping not found".to_string()))?
            .clone();

        drop(mappings);

        if !mapping.automated_collection {
            return Err(CollectionError::NotAutomated(
                "This control does not support automated collection".to_string(),
            ));
        }

        let mut collected_data = HashMap::new();
        collected_data.insert("control_id".to_string(), control_id.to_string());
        collected_data.insert("collection_time".to_string(), Utc::now().to_rfc3339());
        collected_data.insert("automated".to_string(), "true".to_string());

        for source in &mapping.collection_sources {
            if source.enabled {
                let source_data = self.collect_from_source(source).await?;
                collected_data.extend(source_data);
            }
        }

        let evidence = EvidenceItem {
            id: Uuid::new_v4(),
            title: format!("Automated Evidence - {}", mapping.control_name),
            description: Some(format!(
                "Automatically collected evidence for control {}",
                control_id
            )),
            evidence_type: mapping
                .required_evidence_types
                .first()
                .cloned()
                .unwrap
