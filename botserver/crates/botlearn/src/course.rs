use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

use crate::models::*;
use crate::types::*;

pub struct CourseService;

impl CourseService {
    pub fn create_course(
        request: CreateCourseRequest,
    ) -> Result<Course> {
        let course = Course {
            id: Uuid::new_v4(),
            organization_id: None,
            title: request.title,
            description: request.description,
            category: request.category,
            difficulty: request.difficulty.unwrap_or_else(|| "beginner".to_string()),
            duration_minutes: request.duration_minutes.unwrap_or(0),
            thumbnail_url: request.thumbnail_url,
            is_mandatory: request.is_mandatory.unwrap_or(false),
            due_days: request.due_days,
            is_published: false,
            created_by: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };
        Ok(course)
    }

    pub fn publish_course(course: &mut Course) -> Result<()> {
        if course.title.is_empty() {
            anyhow::bail!("Course title cannot be empty");
        }
        if course.duration_minutes <= 0 {
            anyhow::bail!("Course duration must be positive");
        }
        course.is_published = true;
        course.updated_at = Utc::now();
        Ok(())
    }

    pub fn enroll_user(
        user_id: Uuid,
        course_id: Uuid,
    ) -> Result<Enrollment> {
        let enrollment = Enrollment {
            id: Uuid::new_v4(),
            user_id,
            course_id,
            progress_percent: 0.0,
            started_at: Utc::now(),
            completed_at: None,
            certificate_id: None,
        };
        Ok(enrollment)
    }

    pub fn track_progress(
        enrollment: &mut Enrollment,
        progress_percent: f32,
    ) -> Result<()> {
        let clamped = progress_percent.clamp(0.0, 100.0);
        enrollment.progress_percent = clamped;
        if (clamped - 100.0).abs() < f32::EPSILON {
            enrollment.completed_at = Some(Utc::now());
        }
        Ok(())
    }

    pub fn complete_course(
        enrollment: &mut Enrollment,
    ) -> Result<Certification> {
        enrollment.progress_percent = 100.0;
        enrollment.completed_at = Some(Utc::now());

        let verification_code = format!(
            "CERT-{}-{}-{}",
            Uuid::new_v4().to_string().split('-').next().unwrap_or("0000"),
            enrollment.user_id.to_string().split('-').next().unwrap_or("0000"),
            Utc::now().timestamp()
        );

        let cert = Certification {
            id: Uuid::new_v4(),
            user_id: enrollment.user_id,
            course_id: enrollment.course_id,
            issued_at: Utc::now(),
            valid_until: Some(Utc::now() + chrono::Duration::days(365 * 3)),
            credential_url: None,
            verification_code,
        };

        enrollment.certificate_id = Some(cert.id);
        Ok(cert)
    }
}
