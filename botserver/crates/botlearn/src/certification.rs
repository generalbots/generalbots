type Result<T> = std::result::Result<T, String>;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::models::*;
use crate::types::*;

pub struct CertificationService;

impl CertificationService {
    pub fn issue_certificate(
        user_id: Uuid,
        course_id: Uuid,
        user_name: &str,
        course_title: &str,
        _score: i32,
    ) -> Result<CertificationResponse> {
        let verification_code = format!(
            "GBO-{}-{}-{}",
            Uuid::new_v4().to_string().split('-').next().unwrap_or("0000"),
            user_id.to_string().split('-').next().unwrap_or("0000"),
            Utc::now().format("%Y%m%d%H%M%S")
        );

        let credential_url = Self::generate_credential_url(
            &verification_code,
            user_name,
            course_title,
        );

        let issued = Utc::now();
        let valid_until = issued + chrono::Duration::days(365 * 3);

        let response = CertificationResponse {
            id: Uuid::new_v4(),
            user_id,
            user_name: user_name.to_string(),
            course_id,
            course_title: course_title.to_string(),
            issued_at: issued,
            valid_until: Some(valid_until),
            credential_url: Some(credential_url.clone()),
            verification_code,
            is_valid: true,
        };

        Ok(response)
    }

    pub fn verify_certificate(
        verification_code: &str,
    ) -> Result<CertificateVerification> {
        let parts: Vec<&str> = verification_code.split('-').collect();
        if parts.len() < 4 || parts[0] != "GBO" {
            return Ok(CertificateVerification {
                is_valid: false,
                certificate: None,
                message: "Invalid certificate code format".to_string(),
            });
        }

        Ok(CertificateVerification {
            is_valid: true,
            certificate: None,
            message: "Certificate format is valid. Full verification requires database lookup.".to_string(),
        })
    }

    pub fn generate_credential_url(
        verification_code: &str,
        user_name: &str,
        course_title: &str,
    ) -> String {
        let encoded_name = urlencoding(user_name);
        let encoded_course = urlencoding(course_title);
        format!(
            "/api/learn/certificates/verify?code={}&name={}&course={}",
            verification_code, encoded_name, encoded_course
        )
    }

    pub fn render_certificate_html(
        cert: &CertificationResponse,
    ) -> String {
        format!(
            r#"<!DOCTYPE html>
<html><head><meta charset="utf-8"><title>Certificate of Completion</title>
<style>
  body {{ font-family: 'Georgia', serif; text-align: center; padding: 60px; background: #fafafa; }}
  .certificate {{ max-width: 800px; margin: 0 auto; border: 8px double #1a5276; padding: 48px; background: #fff; }}
  h1 {{ color: #1a5276; font-size: 32px; margin-bottom: 8px; }}
  .ribbon {{ font-size: 14px; color: #666; margin-bottom: 32px; }}
  h2 {{ font-size: 24px; color: #333; margin: 16px 0; }}
  .recipient {{ font-size: 28px; font-weight: bold; color: #1a5276; margin: 16px 0; }}
  .course {{ font-size: 20px; color: #555; margin: 8px 0; }}
  .details {{ font-size: 14px; color: #888; margin-top: 32px; }}
  .verify {{ font-size: 12px; color: #aaa; margin-top: 32px; padding-top: 16px; border-top: 1px solid #ddd; }}
</style></head><body>
<div class="certificate">
  <h1>Certificate of Completion</h1>
  <div class="ribbon">General Bots Learning Platform</div>
  <p>This certifies that</p>
  <div class="recipient">{user_name}</div>
  <p>has successfully completed the course</p>
  <div class="course">{course_title}</div>
  <p>with a score of {score}%</p>
  <div class="details">
    Issued: {issued}<br>
    Valid Until: {valid}<br>
    Verification Code: {code}
  </div>
  <div class="verify">Verify at: /api/learn/certificates/verify?code={code}</div>
</div></body></html>"#,
            user_name = cert.user_name,
            course_title = cert.course_title,
            score = "85",
            issued = cert.issued_at.format("%B %d, %Y"),
            valid = cert.valid_until.map(|d| d.format("%B %d, %Y").to_string()).unwrap_or_else(|| "N/A".to_string()),
            code = cert.verification_code,
        )
    }
}

fn urlencoding(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            other => format!("%{:02X}", other as u8),
        })
        .collect()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyQuery {
    pub code: String,
    pub name: Option<String>,
    pub course: Option<String>,
}
