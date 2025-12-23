//! Lead Scoring Keywords - Wrapper module
//!
//! This module serves as a wrapper for CRM lead scoring functionality,
//! re-exporting the functions from the crm module for backward compatibility.
//!
//! BASIC Keywords provided:
//! - SCORE LEAD - Calculate lead score based on criteria
//! - GET LEAD SCORE - Retrieve stored lead score
//! - QUALIFY LEAD - Check if lead meets qualification threshold
//! - UPDATE LEAD SCORE - Manually adjust lead score
//! - AI SCORE LEAD - LLM-enhanced lead scoring
//!
//! Examples:
//!   ' Calculate lead score
//!   lead = #{"email": "cto@company.com", "job_title": "CTO", "company_size": 500}
//!   score = SCORE LEAD(lead)
//!
//!   ' Check if lead is qualified
//!   IF QUALIFY LEAD(lead_id) THEN
//!     TALK "Lead is qualified for sales!"
//!   END IF
//!
//!   ' Get AI-enhanced scoring with recommendations
//!   result = AI SCORE LEAD(lead)
//!   TALK "Score: " + result.score + ", Priority: " + result.priority

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::debug;
use rhai::Engine;
use std::sync::Arc;

use super::crm::register_crm_keywords;

/// Register all lead scoring keywords
///
/// This function delegates to the CRM module's registration function,
/// providing a convenient alias for backward compatibility and clearer intent.
///
/// ## Keywords Registered
///
/// ### SCORE LEAD
/// Calculate a lead score based on provided data.
/// ```basic
/// lead = #{"email": "john@example.com", "job_title": "Manager"}
/// score = SCORE LEAD(lead)
/// ```
///
/// ### GET LEAD SCORE
/// Retrieve a previously stored lead score from the database.
/// ```basic
/// score = GET LEAD SCORE("lead_123")
/// full_data = GET LEAD SCORE("lead_123", "full")
/// ```
///
/// ### QUALIFY LEAD
/// Check if a lead meets the qualification threshold.
/// ```basic
/// is_qualified = QUALIFY LEAD("lead_123")
/// is_qualified = QUALIFY LEAD("lead_123", 80)  ' Custom threshold
/// ```
///
/// ### UPDATE LEAD SCORE
/// Manually adjust a lead's score.
/// ```basic
/// new_score = UPDATE LEAD SCORE("lead_123", 10)  ' Add 10 points
/// new_score = UPDATE LEAD SCORE("lead_123", -5, "Unsubscribed from newsletter")
/// ```
///
/// ### AI SCORE LEAD
/// Get AI-enhanced lead scoring with recommendations.
/// ```basic
/// result = AI SCORE LEAD(lead_data)
/// TALK "Score: " + result.score
/// TALK "Priority: " + result.priority
/// TALK "Recommendation: " + result.recommendation
/// ```
pub fn register_lead_scoring_keywords(
    state: Arc<AppState>,
    user: UserSession,
    engine: &mut Engine,
) {
    debug!("Registering lead scoring keywords...");

    // Delegate to CRM module which contains the actual implementation
    register_crm_keywords(state, user, engine);

    debug!("Lead scoring keywords registered successfully");
}
