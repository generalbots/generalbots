//! Access Review Module
//!
//! Provides automated access review and permission auditing capabilities
//! for compliance with security policies and regulations.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Access level enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AccessLevel {
    Read,
    Write,
    Admin,
    Owner,
}

/// Resource type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceType {
    File,
    Database,
    API,
    System,
    Application,
}

/// Access permission structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessPermission {
    pub id: Uuid,
    pub user_id: Uuid,
    pub resource_id: String,
    pub resource_type: ResourceType,
    pub access_level: AccessLevel,
    pub granted_at: DateTime<Utc>,
    pub granted_by: Uuid,
    pub expires_at: Option<DateTime<Utc>>,
    pub justification: String,
    pub is_active: bool,
}

/// Access review request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessReviewRequest {
    pub id: Uuid,
    pub user_id: Uuid,
    pub reviewer_id: Uuid,
    pub permissions: Vec<AccessPermission>,
    pub requested_at: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub status: ReviewStatus,
    pub comments: Option<String>,
}

/// Review status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReviewStatus {
    Pending,
    InProgress,
    Approved,
    Rejected,
    Expired,
}

/// Access review result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessReviewResult {
    pub review_id: Uuid,
    pub reviewer_id: Uuid,
    pub reviewed_at: DateTime<Utc>,
    pub approved_permissions: Vec<Uuid>,
    pub revoked_permissions: Vec<Uuid>,
    pub modified_permissions: Vec<(Uuid, AccessLevel)>,
    pub comments: String,
}

/// Access violation detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessViolation {
    pub id: Uuid,
    pub user_id: Uuid,
    pub resource_id: String,
    pub attempted_action: String,
    pub denied_reason: String,
    pub occurred_at: DateTime<Utc>,
    pub severity: ViolationSeverity,
}

/// Violation severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Access review service
#[derive(Debug, Clone)]
pub struct AccessReviewService {
    permissions: HashMap<Uuid, Vec<AccessPermission>>,
    reviews: HashMap<Uuid, AccessReviewRequest>,
    violations: Vec<AccessViolation>,
}

impl AccessReviewService {
    /// Create new access review service
    pub fn new() -> Self {
        Self {
            permissions: HashMap::new(),
            reviews: HashMap::new(),
            violations: Vec::new(),
        }
    }

    /// Grant access permission
    pub fn grant_permission(
        &mut self,
        user_id: Uuid,
        resource_id: String,
        resource_type: ResourceType,
        access_level: AccessLevel,
        granted_by: Uuid,
        justification: String,
        expires_in: Option<Duration>,
    ) -> Result<AccessPermission> {
        let permission = AccessPermission {
            id: Uuid::new_v4(),
            user_id,
            resource_id,
            resource_type,
            access_level,
            granted_at: Utc::now(),
            granted_by,
            expires_at: expires_in.map(|d| Utc::now() + d),
            justification,
            is_active: true,
        };

        self.permissions
            .entry(user_id)
            .or_insert_with(Vec::new)
            .push(permission.clone());

        log::info!(
            "Granted {} access to user {} for resource {}",
            serde_json::to_string(&permission.access_level)?,
            user_id,
            permission.resource_id
        );

        Ok(permission)
    }

    /// Revoke access permission
    pub fn revoke_permission(&mut self, permission_id: Uuid, revoked_by: Uuid) -> Result<()> {
        for permissions in self.permissions.values_mut() {
            if let Some(perm) = permissions.iter_mut().find(|p| p.id == permission_id) {
                perm.is_active = false;
                log::info!(
                    "Revoked permission {} for user {} by {}",
                    permission_id,
                    perm.user_id,
                    revoked_by
                );
                return Ok(());
            }
        }
        Err(anyhow!("Permission not found"))
    }

    /// Check if user has access
    pub fn check_access(
        &mut self,
        user_id: Uuid,
        resource_id: &str,
        required_level: AccessLevel,
    ) -> Result<bool> {
        let user_permissions = self.permissions.get(&user_id);

        if let Some(permissions) = user_permissions {
            for perm in permissions {
                if perm.resource_id == resource_id && perm.is_active {
                    // Check expiration
                    if let Some(expires) = perm.expires_at {
                        if expires < Utc::now() {
                            continue;
                        }
                    }

                    // Check access level
                    if self.has_sufficient_access(&perm.access_level, &required_level) {
                        return Ok(true);
                    }
                }
            }
        }

        // Log access denial
        let violation = AccessViolation {
            id: Uuid::new_v4(),
            user_id,
            resource_id: resource_id.to_string(),
            attempted_action: format!("{:?} access", required_level),
            denied_reason: "Insufficient permissions".to_string(),
            occurred_at: Utc::now(),
            severity: ViolationSeverity::Medium,
        };

        self.violations.push(violation);

        Ok(false)
    }

    /// Check if access level is sufficient
    fn has_sufficient_access(&self, user_level: &AccessLevel, required: &AccessLevel) -> bool {
        match required {
            AccessLevel::Read => true,
            AccessLevel::Write => matches!(
                user_level,
                AccessLevel::Write | AccessLevel::Admin | AccessLevel::Owner
            ),
            AccessLevel::Admin => matches!(user_level, AccessLevel::Admin | AccessLevel::Owner),
            AccessLevel::Owner => matches!(user_level, AccessLevel::Owner),
        }
    }

    /// Create access review request
    pub fn create_review_request(
        &mut self,
        user_id: Uuid,
        reviewer_id: Uuid,
        days_until_due: i64,
    ) -> Result<AccessReviewRequest> {
        let user_permissions = self.permissions.get(&user_id).cloned().unwrap_or_default();

        let review = AccessReviewRequest {
            id: Uuid::new_v4(),
            user_id,
            reviewer_id,
            permissions: user_permissions,
            requested_at: Utc::now(),
            due_date: Utc::now() + Duration::days(days_until_due),
            status: ReviewStatus::Pending,
            comments: None,
        };

        self.reviews.insert(review.id, review.clone());

        log::info!(
            "Created access review {} for user {} assigned to {}",
            review.id,
            user_id,
            reviewer_id
        );

        Ok(review)
    }

    /// Process access review
    pub fn process_review(
        &mut self,
        review_id: Uuid,
        approved: Vec<Uuid>,
        revoked: Vec<Uuid>,
        modified: Vec<(Uuid, AccessLevel)>,
        comments: String,
    ) -> Result<AccessReviewResult> {
        let review = self
            .reviews
            .get_mut(&review_id)
            .ok_or_else(|| anyhow!("Review not found"))?;

        if review.status != ReviewStatus::Pending && review.status != ReviewStatus::InProgress {
            return Err(anyhow!("Review already completed"));
        }

        // Process revocations
        for perm_id in &revoked {
            self.revoke_permission(*perm_id, review.reviewer_id)?;
        }

        // Process modifications
        for (perm_id, new_level) in &modified {
            if let Some(permissions) = self.permissions.get_mut(&review.user_id) {
                if let Some(perm) = permissions.iter_mut().find(|p| p.id == *perm_id) {
                    perm.access_level = new_level.clone();
                }
            }
        }

        review.status = ReviewStatus::Approved;
        review.comments = Some(comments.clone());

        let result = AccessReviewResult {
            review_id,
            reviewer_id: review.reviewer_id,
            reviewed_at: Utc::now(),
            approved_permissions: approved,
            revoked_permissions: revoked,
            modified_permissions: modified,
            comments,
        };

        log::info!("Completed access review {} with result", review_id);

        Ok(result)
    }

    /// Get expired permissions
    pub fn get_expired_permissions(&self) -> Vec<AccessPermission> {
        let now = Utc::now();
        let mut expired = Vec::new();

        for permissions in self.permissions.values() {
            for perm in permissions {
                if let Some(expires) = perm.expires_at {
                    if expires < now && perm.is_active {
                        expired.push(perm.clone());
                    }
                }
            }
        }

        expired
    }

    /// Get user permissions
    pub fn get_user_permissions(&self, user_id: Uuid) -> Vec<AccessPermission> {
        self.permissions
            .get(&user_id)
            .cloned()
            .unwrap_or_default()
            .into_iter()
            .filter(|p| p.is_active)
            .collect()
    }

    /// Get pending reviews
    pub fn get_pending_reviews(&self, reviewer_id: Option<Uuid>) -> Vec<AccessReviewRequest> {
        self.reviews
            .values()
            .filter(|r| {
                r.status == ReviewStatus::Pending
                    && reviewer_id.map_or(true, |id| r.reviewer_id == id)
            })
            .cloned()
            .collect()
    }

    /// Get access violations
    pub fn get_violations(
        &self,
        user_id: Option<Uuid>,
        severity: Option<ViolationSeverity>,
        since: Option<DateTime<Utc>>,
    ) -> Vec<AccessViolation> {
        self.violations
            .iter()
            .filter(|v| {
                user_id.map_or(true, |id| v.user_id == id)
                    && severity.as_ref().map_or(true, |s| &v.severity == s)
                    && since.map_or(true, |d| v.occurred_at >= d)
            })
            .cloned()
            .collect()
    }

    /// Generate access compliance report
    pub fn generate_compliance_report(&self) -> AccessComplianceReport {
        let total_permissions = self.permissions.values().map(|p| p.len()).sum::<usize>();

        let active_permissions = self
            .permissions
            .values()
            .flat_map(|p| p.iter())
            .filter(|p| p.is_active)
            .count();

        let expired_permissions = self.get_expired_permissions().len();

        let pending_reviews = self
            .reviews
            .values()
            .filter(|r| r.status == ReviewStatus::Pending)
            .count();

        let violations_last_30_days = self
            .violations
            .iter()
            .filter(|v| v.occurred_at > Utc::now() - Duration::days(30))
            .count();

        let critical_violations = self
            .violations
            .iter()
            .filter(|v| v.severity == ViolationSeverity::Critical)
            .count();

        AccessComplianceReport {
            generated_at: Utc::now(),
            total_permissions,
            active_permissions,
            expired_permissions,
            pending_reviews,
            violations_last_30_days,
            critical_violations,
            compliance_score: self.calculate_compliance_score(),
        }
    }

    /// Calculate compliance score
    fn calculate_compliance_score(&self) -> f64 {
        let mut score = 100.0;

        // Deduct for expired permissions
        let expired = self.get_expired_permissions().len();
        score -= expired as f64 * 2.0;

        // Deduct for overdue reviews
        let overdue_reviews = self
            .reviews
            .values()
            .filter(|r| r.status == ReviewStatus::Pending && r.due_date < Utc::now())
            .count();
        score -= overdue_reviews as f64 * 5.0;

        // Deduct for violations
        for violation in &self.violations {
            match violation.severity {
                ViolationSeverity::Low => score -= 1.0,
                ViolationSeverity::Medium => score -= 3.0,
                ViolationSeverity::High => score -= 5.0,
                ViolationSeverity::Critical => score -= 10.0,
            }
        }

        score.max(0.0).min(100.0)
    }
}

/// Access compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessComplianceReport {
    pub generated_at: DateTime<Utc>,
    pub total_permissions: usize,
    pub active_permissions: usize,
    pub expired_permissions: usize,
    pub pending_reviews: usize,
    pub violations_last_30_days: usize,
    pub critical_violations: usize,
    pub compliance_score: f64,
}

impl Default for AccessReviewService {
    fn default() -> Self {
        Self::new()
    }
}
