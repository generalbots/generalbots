use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DocumentType {
    CPF,
    RG,
    CNH,
    Passport,
    ProofOfAddress,
    ProofOfIncome,
    BankStatement,
    CompanyDocument,
    PowerOfAttorney,
    Selfie,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum KycStatus {
    NotStarted,
    InProgress,
    Pending,
    DocumentsRequired,
    UnderReview,
    Approved,
    Rejected,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycProfile {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub user_id: Uuid,
    pub status: KycStatus,
    pub risk_level: RiskLevel,
    pub documents: Vec<KycDocument>,
    pub checks: Vec<ComplianceCheck>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub reviewed_by: Option<Uuid>,
    pub review_notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Prohibited,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KycDocument {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub document_type: DocumentType,
    pub document_number: Option<String>,
    pub issuer: Option<String>,
    pub issued_at: Option<NaiveDate>,
    pub expires_at: Option<NaiveDate>,
    pub front_image_url: String,
    pub back_image_url: Option<String>,
    pub selfie_url: Option<String>,
    pub ocr_extracted: Option<serde_json::Value>,
    pub verification_status: DocumentStatus,
    pub verification_score: Option<f64>,
    pub verified_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum DocumentStatus {
    Uploaded,
    Processing,
    Verified,
    Mismatch,
    Rejected,
    Expired,
    Suspicious,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheck {
    pub id: Uuid,
    pub profile_id: Uuid,
    pub check_type: CheckType,
    pub provider: String,
    pub status: CheckStatus,
    pub score: Option<f64>,
    pub result: serde_json::Value,
    pub checked_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckType {
    SanctionsList,
    PEP,
    AdverseMedia,
    CreditBureau,
    IdentityVerification,
    AddressVerification,
    PhoneVerification,
    EmailVerification,
    DeviceFingerprint,
    Biometric,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CheckStatus {
    Pending,
    Running,
    Pass,
    Warning,
    Fail,
    Error,
    ManualReview,
}

pub struct KycService;

impl KycService {
    pub fn new() -> Self {
        Self
    }

    pub fn start_profile(tenant_id: Uuid, user_id: Uuid) -> KycProfile {
        let now = Utc::now();
        KycProfile {
            id: Uuid::new_v4(),
            tenant_id,
            user_id,
            status: KycStatus::InProgress,
            risk_level: RiskLevel::Low,
            documents: Vec::new(),
            checks: Vec::new(),
            created_at: now,
            updated_at: now,
            completed_at: None,
            expires_at: None,
            reviewed_by: None,
            review_notes: None,
        }
    }

    pub fn add_document(profile: &mut KycProfile, doc: KycDocument) {
        profile.documents.push(doc);
        profile.updated_at = Utc::now();
    }

    pub fn add_check(profile: &mut KycProfile, check: ComplianceCheck) {
        profile.checks.push(check);
        KycService::recompute_status(profile);
        profile.updated_at = Utc::now();
    }

    pub fn recompute_status(profile: &mut KycProfile) {
        let has_pending = profile
            .checks
            .iter()
            .any(|c| matches!(c.status, CheckStatus::Pending | CheckStatus::Running));
        if has_pending {
            profile.status = KycStatus::UnderReview;
            return;
        }
        let failed = profile
            .checks
            .iter()
            .filter(|c| matches!(c.status, CheckStatus::Fail | CheckStatus::Error))
            .count();
        let warned = profile
            .checks
            .iter()
            .filter(|c| matches!(c.status, CheckStatus::Warning | CheckStatus::ManualReview))
            .count();
        if failed > 0 {
            profile.status = KycStatus::Rejected;
            profile.risk_level = RiskLevel::High;
        } else if warned > 0 {
            profile.status = KycStatus::Pending;
            profile.risk_level = RiskLevel::Medium;
        } else {
            profile.status = KycStatus::Approved;
            profile.completed_at = Some(Utc::now());
            profile.expires_at = Some(Utc::now() + chrono::Duration::days(365));
        }
    }

    pub fn is_expired(profile: &KycProfile) -> bool {
        match profile.expires_at {
            Some(exp) => Utc::now() > exp,
            None => false,
        }
    }
}

impl Default for KycService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_profile_starts_in_progress() {
        let p = KycService::start_profile(Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(p.status, KycStatus::InProgress);
    }

    #[test]
    fn all_passing_checks_approves() {
        let mut p = KycService::start_profile(Uuid::new_v4(), Uuid::new_v4());
        let profile_id = p.id;
        KycService::add_check(&mut p, ComplianceCheck {
            id: Uuid::new_v4(),
            profile_id,
            check_type: CheckType::SanctionsList,
            provider: "OFAC".into(),
            status: CheckStatus::Pass,
            score: Some(1.0),
            result: serde_json::json!({}),
            checked_at: Utc::now(),
        });
        assert_eq!(p.status, KycStatus::Approved);
    }
}
