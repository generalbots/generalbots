//! Training Tracker Module
//!
//! Provides comprehensive security training tracking and compliance
//! management for ensuring all personnel meet training requirements.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Training type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrainingType {
    SecurityAwareness,
    DataProtection,
    PhishingPrevention,
    IncidentResponse,
    ComplianceRegulation,
    PasswordManagement,
    AccessControl,
    EmergencyProcedures,
}

/// Training status
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrainingStatus {
    NotStarted,
    InProgress,
    Completed,
    Expired,
    Failed,
    Exempted,
}

/// Training priority
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TrainingPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Training course definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingCourse {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub training_type: TrainingType,
    pub duration_hours: f32,
    pub validity_days: i64,
    pub priority: TrainingPriority,
    pub required_for_roles: Vec<String>,
    pub prerequisites: Vec<Uuid>,
    pub content_url: Option<String>,
    pub passing_score: u32,
    pub max_attempts: u32,
}

/// Training assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingAssignment {
    pub id: Uuid,
    pub user_id: Uuid,
    pub course_id: Uuid,
    pub assigned_date: DateTime<Utc>,
    pub due_date: DateTime<Utc>,
    pub status: TrainingStatus,
    pub attempts: Vec<TrainingAttempt>,
    pub completion_date: Option<DateTime<Utc>>,
    pub expiry_date: Option<DateTime<Utc>>,
    pub assigned_by: String,
    pub notes: Option<String>,
}

/// Training attempt record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingAttempt {
    pub id: Uuid,
    pub attempt_number: u32,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub score: Option<u32>,
    pub passed: bool,
    pub time_spent_minutes: Option<u32>,
}

/// Training certificate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingCertificate {
    pub id: Uuid,
    pub user_id: Uuid,
    pub course_id: Uuid,
    pub issued_date: DateTime<Utc>,
    pub expiry_date: DateTime<Utc>,
    pub certificate_number: String,
    pub verification_code: String,
}

/// Training compliance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    pub user_id: Uuid,
    pub compliant: bool,
    pub required_trainings: Vec<Uuid>,
    pub completed_trainings: Vec<Uuid>,
    pub overdue_trainings: Vec<Uuid>,
    pub upcoming_trainings: Vec<Uuid>,
    pub compliance_percentage: f64,
}

/// Training tracker service
#[derive(Debug, Clone)]
pub struct TrainingTracker {
    courses: HashMap<Uuid, TrainingCourse>,
    assignments: HashMap<Uuid, TrainingAssignment>,
    certificates: HashMap<Uuid, TrainingCertificate>,
    user_roles: HashMap<Uuid, Vec<String>>,
}

impl TrainingTracker {
    /// Create new training tracker
    pub fn new() -> Self {
        let mut tracker = Self {
            courses: HashMap::new(),
            assignments: HashMap::new(),
            certificates: HashMap::new(),
            user_roles: HashMap::new(),
        };

        // Initialize with default courses
        tracker.initialize_default_courses();
        tracker
    }

    /// Initialize default training courses
    fn initialize_default_courses(&mut self) {
        let security_awareness = TrainingCourse {
            id: Uuid::new_v4(),
            title: "Security Awareness Fundamentals".to_string(),
            description: "Basic security awareness training for all employees".to_string(),
            training_type: TrainingType::SecurityAwareness,
            duration_hours: 2.0,
            validity_days: 365,
            priority: TrainingPriority::High,
            required_for_roles: vec!["all".to_string()],
            prerequisites: vec![],
            content_url: Some("https://training.example.com/security-awareness".to_string()),
            passing_score: 80,
            max_attempts: 3,
        };

        self.courses
            .insert(security_awareness.id, security_awareness);

        let data_protection = TrainingCourse {
            id: Uuid::new_v4(),
            title: "Data Protection and Privacy".to_string(),
            description: "Training on data protection regulations and best practices".to_string(),
            training_type: TrainingType::DataProtection,
            duration_hours: 3.0,
            validity_days: 365,
            priority: TrainingPriority::High,
            required_for_roles: vec!["admin".to_string(), "manager".to_string()],
            prerequisites: vec![],
            content_url: Some("https://training.example.com/data-protection".to_string()),
            passing_score: 85,
            max_attempts: 3,
        };

        self.courses.insert(data_protection.id, data_protection);
    }

    /// Create a training course
    pub fn create_course(&mut self, course: TrainingCourse) -> Result<()> {
        if self.courses.contains_key(&course.id) {
            return Err(anyhow!("Course already exists"));
        }

        log::info!("Creating training course: {}", course.title);
        self.courses.insert(course.id, course);
        Ok(())
    }

    /// Assign training to user
    pub fn assign_training(
        &mut self,
        user_id: Uuid,
        course_id: Uuid,
        due_days: i64,
        assigned_by: String,
    ) -> Result<TrainingAssignment> {
        let course = self
            .courses
            .get(&course_id)
            .ok_or_else(|| anyhow!("Course not found"))?
            .clone();

        let assignment = TrainingAssignment {
            id: Uuid::new_v4(),
            user_id,
            course_id,
            assigned_date: Utc::now(),
            due_date: Utc::now() + Duration::days(due_days),
            status: TrainingStatus::NotStarted,
            attempts: vec![],
            completion_date: None,
            expiry_date: None,
            assigned_by,
            notes: None,
        };

        self.assignments.insert(assignment.id, assignment.clone());

        log::info!("Assigned training '{}' to user {}", course.title, user_id);

        Ok(assignment)
    }

    /// Start training attempt
    pub fn start_training(&mut self, assignment_id: Uuid) -> Result<TrainingAttempt> {
        let assignment = self
            .assignments
            .get_mut(&assignment_id)
            .ok_or_else(|| anyhow!("Assignment not found"))?;

        let course = self
            .courses
            .get(&assignment.course_id)
            .ok_or_else(|| anyhow!("Course not found"))?;

        if assignment.attempts.len() >= course.max_attempts as usize {
            return Err(anyhow!("Maximum attempts exceeded"));
        }

        let attempt = TrainingAttempt {
            id: Uuid::new_v4(),
            attempt_number: (assignment.attempts.len() + 1) as u32,
            start_time: Utc::now(),
            end_time: None,
            score: None,
            passed: false,
            time_spent_minutes: None,
        };

        assignment.status = TrainingStatus::InProgress;
        assignment.attempts.push(attempt.clone());

        Ok(attempt)
    }

    /// Complete training attempt
    pub fn complete_training(
        &mut self,
        assignment_id: Uuid,
        attempt_id: Uuid,
        score: u32,
    ) -> Result<bool> {
        // Get course info first
        let (course_id, passing_score, validity_days, max_attempts, course_title) = {
            let assignment = self
                .assignments
                .get(&assignment_id)
                .ok_or_else(|| anyhow!("Assignment not found"))?;
            let course = self
                .courses
                .get(&assignment.course_id)
                .ok_or_else(|| anyhow!("Course not found"))?;
            (course.id, course.passing_score, course.validity_days, course.max_attempts, course.title.clone())
        };

        let assignment = self
            .assignments
            .get_mut(&assignment_id)
            .ok_or_else(|| anyhow!("Assignment not found"))?;

        let attempt_idx = assignment
            .attempts
            .iter()
            .position(|a| a.id == attempt_id)
            .ok_or_else(|| anyhow!("Attempt not found"))?;

        let end_time = Utc::now();
        let start_time = assignment.attempts[attempt_idx].start_time;
        let time_spent = (end_time - start_time).num_minutes() as u32;
        let passed = score >= passing_score;

        // Update attempt
        assignment.attempts[attempt_idx].end_time = Some(end_time);
        assignment.attempts[attempt_idx].score = Some(score);
        assignment.attempts[attempt_idx].time_spent_minutes = Some(time_spent);
        assignment.attempts[attempt_idx].passed = passed;

        let user_id = assignment.user_id;
        let attempts_count = assignment.attempts.len();

        if passed {
            assignment.status = TrainingStatus::Completed;
            assignment.completion_date = Some(end_time);
            assignment.expiry_date = Some(end_time + Duration::days(validity_days));

            // Issue certificate
            let certificate = TrainingCertificate {
                id: Uuid::new_v4(),
                user_id,
                course_id,
                issued_date: end_time,
                expiry_date: end_time + Duration::days(validity_days),
                certificate_number: format!(
                    "CERT-{}",
                    Uuid::new_v4().to_string()[..8].to_uppercase()
                ),
                verification_code: Uuid::new_v4().to_string(),
            };

            self.certificates.insert(certificate.id, certificate);

            log::info!(
                "User {} completed training '{}' with score {}",
                user_id,
                course_title,
                score
            );
        } else if attempts_count >= max_attempts as usize {
            assignment.status = TrainingStatus::Failed;
        }

        Ok(passed)
    }

    /// Get user compliance status
    pub fn get_compliance_status(&self, user_id: Uuid) -> ComplianceStatus {
        let user_roles = self
            .user_roles
            .get(&user_id)
            .cloned()
            .unwrap_or_else(|| vec!["all".to_string()]);

        let mut required_trainings = vec![];
        let mut completed_trainings = vec![];
        let mut overdue_trainings = vec![];
        let mut upcoming_trainings = vec![];

        for course in self.courses.values() {
            if course
                .required_for_roles
                .iter()
                .any(|r| user_roles.contains(r) || r == "all")
            {
                required_trainings.push(course.id);

                // Check if user has completed this training
                let assignment = self
                    .assignments
                    .values()
                    .find(|a| a.user_id == user_id && a.course_id == course.id);

                if let Some(assignment) = assignment {
                    match assignment.status {
                        TrainingStatus::Completed => {
                            if let Some(expiry) = assignment.expiry_date {
                                if expiry > Utc::now() {
                                    completed_trainings.push(course.id);
                                } else {
                                    overdue_trainings.push(course.id);
                                }
                            }
                        }
                        TrainingStatus::NotStarted | TrainingStatus::InProgress => {
                            if assignment.due_date < Utc::now() {
                                overdue_trainings.push(course.id);
                            } else {
                                upcoming_trainings.push(course.id);
                            }
                        }
                        _ => {}
                    }
                } else {
                    overdue_trainings.push(course.id);
                }
            }
        }

        let compliance_percentage = if required_trainings.is_empty() {
            100.0
        } else {
            (completed_trainings.len() as f64 / required_trainings.len() as f64) * 100.0
        };

        ComplianceStatus {
            user_id,
            compliant: overdue_trainings.is_empty(),
            required_trainings,
            completed_trainings,
            overdue_trainings,
            upcoming_trainings,
            compliance_percentage,
        }
    }

    /// Get training report
    pub fn get_training_report(&self) -> TrainingReport {
        let total_courses = self.courses.len();
        let total_assignments = self.assignments.len();
        let total_certificates = self.certificates.len();

        let mut assignments_by_status = HashMap::new();
        for assignment in self.assignments.values() {
            *assignments_by_status
                .entry(assignment.status.clone())
                .or_insert(0) += 1;
        }

        let overdue_count = self
            .assignments
            .values()
            .filter(|a| a.status != TrainingStatus::Completed && a.due_date < Utc::now())
            .count();

        let expiring_soon = self
            .certificates
            .values()
            .filter(|c| {
                c.expiry_date > Utc::now() && c.expiry_date < Utc::now() + Duration::days(30)
            })
            .count();

        let average_score = self.calculate_average_score();

        TrainingReport {
            generated_at: Utc::now(),
            total_courses,
            total_assignments,
            total_certificates,
            assignments_by_status,
            overdue_count,
            expiring_soon,
            average_score,
        }
    }

    /// Calculate average training score
    fn calculate_average_score(&self) -> f64 {
        let mut total_score = 0;
        let mut count = 0;

        for assignment in self.assignments.values() {
            for attempt in &assignment.attempts {
                if let Some(score) = attempt.score {
                    total_score += score;
                    count += 1;
                }
            }
        }

        if count == 0 {
            0.0
        } else {
            total_score as f64 / count as f64
        }
    }

    /// Set user roles
    pub fn set_user_roles(&mut self, user_id: Uuid, roles: Vec<String>) {
        self.user_roles.insert(user_id, roles);
    }

    /// Get overdue trainings
    pub fn get_overdue_trainings(&self) -> Vec<TrainingAssignment> {
        self.assignments
            .values()
            .filter(|a| a.status != TrainingStatus::Completed && a.due_date < Utc::now())
            .cloned()
            .collect()
    }

    /// Get expiring certificates
    pub fn get_expiring_certificates(&self, days_ahead: i64) -> Vec<TrainingCertificate> {
        let cutoff = Utc::now() + Duration::days(days_ahead);
        self.certificates
            .values()
            .filter(|c| c.expiry_date > Utc::now() && c.expiry_date <= cutoff)
            .cloned()
            .collect()
    }
}

/// Training report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingReport {
    pub generated_at: DateTime<Utc>,
    pub total_courses: usize,
    pub total_assignments: usize,
    pub total_certificates: usize,
    pub assignments_by_status: HashMap<TrainingStatus, usize>,
    pub overdue_count: usize,
    pub expiring_soon: usize,
    pub average_score: f64,
}

impl Default for TrainingTracker {
    fn default() -> Self {
        Self::new()
    }
}
