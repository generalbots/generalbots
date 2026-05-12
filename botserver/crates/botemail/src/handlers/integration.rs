use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::Json,
};
use diesel::prelude::*;
use serde_json::json;
use std::sync::Arc;
use uuid::Uuid;

use crate::models::AppState;
use crate::types::*;

pub async fn get_feature_flags(
    State(state): State<Arc<AppState>>,
    Path(org_id): Path<Uuid>,
) -> Result<Json<FeatureFlags>, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    #[derive(QueryableByName)]
    struct FlagRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        feature: String,
        #[diesel(sql_type = diesel::sql_types::Bool)]
        enabled: bool,
    }

    let flags: Vec<FlagRow> = diesel::sql_query(
        "SELECT feature, enabled FROM feature_flags WHERE org_id = $1"
    )
    .bind::<diesel::sql_types::Uuid, _>(org_id)
    .load(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let crm_enabled = flags.iter().any(|f| f.feature == "crm" && f.enabled);
    let campaigns_enabled = flags.iter().any(|f| f.feature == "campaigns" && f.enabled);

    Ok(Json(FeatureFlags {
        crm_enabled,
        campaigns_enabled,
    }))
}

pub async fn extract_lead_from_email(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<LeadExtractionRequest>,
) -> Result<Json<LeadExtractionResponse>, StatusCode> {
    let email = req.from.clone();

    let name_part = email.split('@').next().unwrap_or("");
    let parts: Vec<&str> = name_part.split('.').collect();

    let first_name = parts.first().map(|s| capitalize(s));
    let last_name = if parts.len() > 1 {
        parts.get(1).map(|s| capitalize(s))
    } else {
        None
    };

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

pub async fn get_crm_context_by_email(
    State(state): State<Arc<AppState>>,
    Path(email): Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    #[derive(QueryableByName)]
    struct ContactJsonRow {
        #[diesel(sql_type = diesel::sql_types::Text)]
        row_to_json: String,
    }

    let contact: Option<ContactJsonRow> = diesel::sql_query(
        "SELECT row_to_json(c.*) as row_to_json FROM crm_contacts c WHERE c.email = $1"
    )
    .bind::<diesel::sql_types::Text, _>(&email)
    .get_result(&mut conn)
    .ok();

    match contact {
        Some(c) => {
            let contact_value: serde_json::Value = serde_json::from_str(&c.row_to_json).unwrap_or(json!({}));
            Ok(Json(json!({ "found": true, "contact": contact_value })))
        }
        None => Ok(Json(json!({ "found": false }))),
    }
}

pub async fn link_email_to_crm(
    State(state): State<Arc<AppState>>,
    Json(link): Json<EmailCrmLink>,
) -> Result<StatusCode, StatusCode> {
    let mut conn = state.pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    diesel::sql_query(
        "INSERT INTO email_crm_links (email_id, contact_id, opportunity_id) VALUES ($1, $2, $3)"
    )
    .bind::<diesel::sql_types::Uuid, _>(link.email_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(link.contact_id)
    .bind::<diesel::sql_types::Nullable<diesel::sql_types::Uuid>, _>(link.opportunity_id)
    .execute(&mut conn)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(StatusCode::CREATED)
}

pub async fn categorize_email(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<LeadExtractionRequest>,
) -> Result<Json<EmailCategoryResponse>, StatusCode> {
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

pub async fn generate_smart_reply(
    State(_state): State<Arc<AppState>>,
    Json(_req): Json<SmartReplyRequest>,
) -> Result<Json<SmartReplyResponse>, StatusCode> {
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
}
