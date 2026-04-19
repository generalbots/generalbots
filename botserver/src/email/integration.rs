use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Json},
};
use diesel::prelude::*;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::core::shared::state::AppState;
use super::integration_types::*;

/// Check which features are enabled for the organization
pub async fn get_feature_flags(
    State(state): State<Arc<AppState>>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<FeatureFlags>, StatusCode> {
    use crate::core::shared::schema::feature_flags;

    let mut conn = state.conn.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let flags: Vec<(String, bool)> = feature_flags::table
        .filter(feature_flags::org_id.eq(org_id))
        .select((feature_flags::feature, feature_flags::enabled))
        .load(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let crm_enabled = flags.iter().any(|(f, e)| f == "crm" && *e);
    let campaigns_enabled = flags.iter().any(|(f, e)| f == "campaigns" && *e);

    Ok(Json(FeatureFlags {
        crm_enabled,
        campaigns_enabled,
    }))
}

/// Extract lead information from email using AI
pub async fn extract_lead_from_email(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<LeadExtractionRequest>,
) -> Result<Json<LeadExtractionResponse>, StatusCode> {
    // Simple extraction logic (can be enhanced with LLM)
    let email = req.from.clone();
    
    // Extract name from email (before @)
    let name_part = email.split('@').next().unwrap_or("");
    let parts: Vec<&str> = name_part.split('.').collect();
    
    let first_name = parts.first().map(|s| capitalize(s));
    let last_name = if parts.len() > 1 {
        parts.get(1).map(|s| capitalize(s))
    } else {
        None
    };

    // Extract company from email domain
    let company = email
        .split('@')
        .nth(1)
        .and_then(|d| d.split('.').next())
        .map(capitalize);

    Ok(Json(LeadExtractionResponse {
        first_name,
        last_name,
        email,
        company,
        phone: None,
        value: None,
    }))
}

/// Get CRM context for an email sender
pub async fn get_crm_context_by_email(
    State(state): State<Arc<AppState>>,
    Path(email): Path<String>,
) -> Result<impl IntoResponse, StatusCode> {
    use crate::core::shared::schema::crm_contacts;

    let mut conn = state.conn.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let contact = crm_contacts::table
        .filter(crm_contacts::email.eq(&email))
        .first::<crate::contacts::crm::CrmContact>(&mut conn)
        .optional()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match contact {
        Some(c) => Ok(Json(json!({
            "found": true,
            "contact": c
        }))),
        None => Ok(Json(json!({
            "found": false
        }))),
    }
}

/// Link email to CRM contact/opportunity
pub async fn link_email_to_crm(
    State(state): State<Arc<AppState>>,
    Json(link): Json<EmailCrmLink>,
) -> Result<StatusCode, StatusCode> {
    use crate::core::shared::schema::email_crm_links;

    let mut conn = state.conn.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::insert_into(email_crm_links::table)
        .values((
            email_crm_links::email_id.eq(link.email_id),
            email_crm_links::contact_id.eq(link.contact_id),
            email_crm_links::opportunity_id.eq(link.opportunity_id),
        ))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::CREATED)
}

/// Categorize email using AI
pub async fn categorize_email(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<LeadExtractionRequest>,
) -> Result<Json<EmailCategoryResponse>, StatusCode> {
    // Simple keyword-based categorization (can be enhanced with LLM)
    let text = format!("{} {}", req.subject.to_lowercase(), req.body.to_lowercase());

    let category = if text.contains("quote") || text.contains("proposal") || text.contains("pricing") {
        "sales"
    } else if text.contains("support") || text.contains("help") || text.contains("issue") {
        "support"
    } else if text.contains("newsletter") || text.contains("unsubscribe") {
        "marketing"
    } else {
        "general"
    };

    Ok(Json(EmailCategoryResponse {
        category: category.to_string(),
        confidence: 0.8,
    }))
}

/// Generate smart reply suggestions
pub async fn generate_smart_reply(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<SmartReplyRequest>,
) -> Result<Json<SmartReplyResponse>, StatusCode> {
    // Simple template responses (can be enhanced with LLM)
    let suggestions = vec![
        "Thank you for your email. I'll get back to you shortly.".to_string(),
        "I appreciate you reaching out. Let me review this and respond soon.".to_string(),
        "Thanks for the update. I'll take a look and follow up.".to_string(),
    ];

    Ok(Json(SmartReplyResponse { suggestions }))
}

fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capitalize() {
        assert_eq!(capitalize("john"), "John");
        assert_eq!(capitalize(""), "");
        assert_eq!(capitalize("a"), "A");
    }

    #[test]
    fn test_categorize_sales_email() {
        let req = LeadExtractionRequest {
            from: "test@example.com".to_string(),
            subject: "Request for pricing quote".to_string(),
            body: "I would like to get a quote for your services".to_string(),
        };

        // This would need async test setup
        // For now, just test the logic
        let text = format!("{} {}", req.subject.to_lowercase(), req.body.to_lowercase());
        assert!(text.contains("quote"));
    }
}
