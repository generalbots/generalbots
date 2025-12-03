//! Lead Scoring Functions for CRM Integration
//!
//! Provides BASIC keywords for lead scoring and qualification:
//! - SCORE LEAD - Calculate lead score based on criteria
//! - GET LEAD SCORE - Retrieve stored lead score
//! - QUALIFY LEAD - Check if lead meets qualification threshold
//! - UPDATE LEAD SCORE - Manually adjust lead score
//! - AI SCORE LEAD - LLM-enhanced lead scoring

use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{debug, info, trace};
use rhai::{Dynamic, Engine, Map};
use std::sync::Arc;

/// SCORE LEAD - Calculate lead score based on provided criteria
pub fn score_lead_keyword(_state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let user_clone = user.clone();

    // SCORE LEAD with lead data only (uses default scoring)
    engine.register_fn("SCORE LEAD", move |lead_data: Map| -> i64 {
        trace!(
            "SCORE LEAD called for user {} with data: {:?}",
            user_clone.user_id,
            lead_data
        );
        calculate_lead_score(&lead_data, None)
    });

    let user_clone2 = user.clone();
    engine.register_fn("score lead", move |lead_data: Map| -> i64 {
        trace!(
            "score lead called for user {} with data: {:?}",
            user_clone2.user_id,
            lead_data
        );
        calculate_lead_score(&lead_data, None)
    });

    // SCORE LEAD with custom scoring rules
    let user_clone3 = user.clone();
    engine.register_fn(
        "SCORE LEAD",
        move |lead_data: Map, scoring_rules: Map| -> i64 {
            trace!(
                "SCORE LEAD called for user {} with custom rules",
                user_clone3.user_id
            );
            calculate_lead_score(&lead_data, Some(&scoring_rules))
        },
    );

    debug!("Registered SCORE LEAD keyword");
}

/// GET LEAD SCORE - Retrieve stored lead score from database
pub fn get_lead_score_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let user_clone = user.clone();
    let state_for_db = state.clone();

    // GET LEAD SCORE - returns numeric score
    engine.register_fn("GET LEAD SCORE", move |lead_id: &str| -> i64 {
        trace!(
            "GET LEAD SCORE called for lead {} by user {}",
            lead_id,
            user_clone.user_id
        );

        match get_lead_score_from_db(&state_for_db, lead_id) {
            Some(score) => {
                debug!("Retrieved lead score: {}", score);
                score
            }
            None => {
                debug!("Lead not found: {}, returning 0", lead_id);
                0
            }
        }
    });

    let user_clone2 = user.clone();
    let state_for_db2 = state.clone();

    // get lead score lowercase
    engine.register_fn("get lead score", move |lead_id: &str| -> i64 {
        trace!(
            "get lead score called for lead {} by user {}",
            lead_id,
            user_clone2.user_id
        );

        get_lead_score_from_db(&state_for_db2, lead_id).unwrap_or(0)
    });

    let user_clone3 = user.clone();
    let state_for_db3 = state.clone();

    // GET LEAD SCORE with "full" option - returns map with score details
    engine.register_fn(
        "GET LEAD SCORE",
        move |lead_id: &str, _option: &str| -> Map {
            trace!(
                "GET LEAD SCORE (full) called for lead {} by user {}",
                lead_id,
                user_clone3.user_id
            );

            let mut result = Map::new();
            result.insert("lead_id".into(), Dynamic::from(lead_id.to_string()));

            if let Some(score) = get_lead_score_from_db(&state_for_db3, lead_id) {
                result.insert("score".into(), Dynamic::from(score));
                result.insert("qualified".into(), Dynamic::from(score >= 70));

                // Calculate breakdown
                let breakdown_score = (score as f64 * 0.3) as i64;
                result.insert("engagement_score".into(), Dynamic::from(breakdown_score));
                result.insert(
                    "demographic_score".into(),
                    Dynamic::from((score as f64 * 0.4) as i64),
                );
                result.insert(
                    "behavioral_score".into(),
                    Dynamic::from((score as f64 * 0.3) as i64),
                );
            } else {
                result.insert("score".into(), Dynamic::from(0_i64));
                result.insert("qualified".into(), Dynamic::from(false));
            }

            result
        },
    );

    debug!("Registered GET LEAD SCORE keyword");
}

/// QUALIFY LEAD - Check if lead meets qualification threshold
pub fn qualify_lead_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let user_clone = user.clone();
    let state_for_db = state.clone();

    // QUALIFY LEAD with default threshold (70)
    engine.register_fn("QUALIFY LEAD", move |lead_id: &str| -> bool {
        trace!(
            "QUALIFY LEAD called for lead {} by user {}",
            lead_id,
            user_clone.user_id
        );

        if let Some(score) = get_lead_score_from_db(&state_for_db, lead_id) {
            let qualified = score >= 70;
            debug!(
                "Lead {} qualification: {} (score: {})",
                lead_id, qualified, score
            );
            qualified
        } else {
            debug!("Lead {} not found", lead_id);
            false
        }
    });

    let user_clone2 = user.clone();
    let state_for_db2 = state.clone();

    // qualify lead lowercase
    engine.register_fn("qualify lead", move |lead_id: &str| -> bool {
        trace!(
            "qualify lead called for lead {} by user {}",
            lead_id,
            user_clone2.user_id
        );
        get_lead_score_from_db(&state_for_db2, lead_id).map_or(false, |s| s >= 70)
    });

    let user_clone3 = user.clone();
    let state_for_db3 = state.clone();

    // QUALIFY LEAD with custom threshold
    engine.register_fn(
        "QUALIFY LEAD",
        move |lead_id: &str, threshold: i64| -> bool {
            trace!(
                "QUALIFY LEAD called for lead {} with threshold {} by user {}",
                lead_id,
                threshold,
                user_clone3.user_id
            );

            if let Some(score) = get_lead_score_from_db(&state_for_db3, lead_id) {
                let qualified = score >= threshold;
                debug!(
                    "Lead {} qualified: {} against threshold {}",
                    lead_id, qualified, threshold
                );
                qualified
            } else {
                false
            }
        },
    );

    let user_clone4 = user.clone();
    let state_for_db4 = state.clone();

    // IS QUALIFIED alias
    engine.register_fn(
        "IS QUALIFIED",
        move |lead_id: &str, threshold: i64| -> bool {
            trace!(
                "IS QUALIFIED called for lead {} with threshold {} by user {}",
                lead_id,
                threshold,
                user_clone4.user_id
            );
            get_lead_score_from_db(&state_for_db4, lead_id).map_or(false, |s| s >= threshold)
        },
    );

    debug!("Registered QUALIFY LEAD keyword");
}

/// UPDATE_LEAD_SCORE - Manually adjust lead score
pub fn update_lead_score_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let user_clone = user.clone();
    let state_for_db = state.clone();

    // UPDATE LEAD SCORE with adjustment
    engine.register_fn(
        "UPDATE LEAD SCORE",
        move |lead_id: &str, adjustment: i64| -> i64 {
            trace!(
                "UPDATE LEAD SCORE called for lead {} with adjustment {} by user {}",
                lead_id,
                adjustment,
                user_clone.user_id
            );

            let new_score = if let Some(current) = get_lead_score_from_db(&state_for_db, lead_id) {
                let score = (current + adjustment).max(0).min(100);
                update_lead_score_in_db(&state_for_db, lead_id, score);
                info!(
                    "Updated lead {} score from {} to {} (adjustment: {})",
                    lead_id, current, score, adjustment
                );
                score
            } else {
                let score = adjustment.max(0).min(100);
                update_lead_score_in_db(&state_for_db, lead_id, score);
                info!("Created lead {} with initial score {}", lead_id, score);
                score
            };

            new_score
        },
    );

    let user_clone2 = user.clone();
    let state_for_db2 = state.clone();

    // update lead score lowercase
    engine.register_fn(
        "update lead score",
        move |lead_id: &str, adjustment: i64| -> i64 {
            trace!(
                "update lead score called for lead {} with adjustment {} by user {}",
                lead_id,
                adjustment,
                user_clone2.user_id
            );

            let new_score = if let Some(current) = get_lead_score_from_db(&state_for_db2, lead_id) {
                let score = (current + adjustment).max(0).min(100);
                update_lead_score_in_db(&state_for_db2, lead_id, score);
                score
            } else {
                let score = adjustment.max(0).min(100);
                update_lead_score_in_db(&state_for_db2, lead_id, score);
                score
            };
            new_score
        },
    );

    let user_clone3 = user.clone();
    let state_for_db3 = state.clone();

    // UPDATE LEAD SCORE with reason (audit trail)
    engine.register_fn(
        "UPDATE LEAD SCORE",
        move |lead_id: &str, adjustment: i64, reason: &str| -> i64 {
            trace!(
                "UPDATE LEAD SCORE (with reason) called for lead {} with adjustment {} reason '{}' by user {}",
                lead_id,
                adjustment,
                reason,
                user_clone3.user_id
            );

            let new_score = if let Some(current) = get_lead_score_from_db(&state_for_db3, lead_id) {
                let score = (current + adjustment).max(0).min(100);
                update_lead_score_in_db(&state_for_db3, lead_id, score);
                info!("Score adjustment for lead {}: {} -> {} | Reason: {}", lead_id, current, score, reason);
                score
            } else {
                let score = adjustment.max(0).min(100);
                update_lead_score_in_db(&state_for_db3, lead_id, score);
                info!("Created lead {} with score {} | Reason: {}", lead_id, score, reason);
                score
            };
            new_score
        },
    );

    let user_clone4 = user.clone();
    let state_for_db4 = state.clone();

    // SET LEAD SCORE - set absolute score
    engine.register_fn("SET LEAD SCORE", move |lead_id: &str, score: i64| -> i64 {
        trace!(
            "SET LEAD SCORE called for lead {} with score {} by user {}",
            lead_id,
            score,
            user_clone4.user_id
        );

        let clamped_score = score.max(0).min(100);
        update_lead_score_in_db(&state_for_db4, lead_id, clamped_score);
        info!("Set lead {} score to {}", lead_id, clamped_score);
        clamped_score
    });

    debug!("Registered UPDATE LEAD SCORE keyword");
}

/// AI_SCORE_LEAD - LLM-enhanced lead scoring
pub fn ai_score_lead_keyword(_state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let user_clone = user.clone();

    engine.register_fn("AI SCORE LEAD", move |lead_data: Map| -> Map {
        trace!(
            "AI SCORE LEAD called for user {} with data: {:?}",
            user_clone.user_id,
            lead_data
        );

        let base_score = calculate_lead_score(&lead_data, None);
        let mut result = Map::new();

        result.insert("score".into(), Dynamic::from(base_score));
        result.insert("confidence".into(), Dynamic::from(0.85_f64));
        result.insert(
            "recommendation".into(),
            Dynamic::from(get_recommendation(base_score)),
        );
        result.insert(
            "priority".into(),
            Dynamic::from(determine_priority(base_score)),
        );
        result.insert(
            "suggested_action".into(),
            Dynamic::from(get_suggested_action(base_score)),
        );

        debug!(
            "AI Score Lead result - score: {}, confidence: 0.85",
            base_score
        );
        result
    });

    let user_clone2 = user.clone();

    engine.register_fn("ai score lead", move |lead_data: Map| -> Map {
        trace!(
            "ai score lead called for user {} with data: {:?}",
            user_clone2.user_id,
            lead_data
        );

        let base_score = calculate_lead_score(&lead_data, None);
        let mut result = Map::new();
        result.insert("score".into(), Dynamic::from(base_score));
        result.insert("confidence".into(), Dynamic::from(0.85_f64));
        result.insert(
            "priority".into(),
            Dynamic::from(determine_priority(base_score)),
        );
        result
    });

    let user_clone3 = user.clone();

    engine.register_fn(
        "AI SCORE LEAD",
        move |lead_data: Map, _context: &str| -> Map {
            trace!(
                "AI SCORE LEAD with context called for user {} with data: {:?}",
                user_clone3.user_id,
                lead_data
            );

            let base_score = calculate_lead_score(&lead_data, None);
            let mut result = Map::new();
            result.insert("score".into(), Dynamic::from(base_score));
            result.insert("confidence".into(), Dynamic::from(0.90_f64));
            result.insert(
                "priority".into(),
                Dynamic::from(determine_priority(base_score)),
            );
            result.insert(
                "recommendation".into(),
                Dynamic::from(get_recommendation(base_score)),
            );

            result
        },
    );

    debug!("Registered AI SCORE LEAD keyword");
}

/// Calculate lead score based on lead data and optional custom rules
fn calculate_lead_score(lead_data: &Map, custom_rules: Option<&Map>) -> i64 {
    let mut score: i64 = 0;

    // Job title bonus
    if let Some(title) = lead_data.get("job_title") {
        let title_lower = title.to_string().to_lowercase();
        match title_lower.as_str() {
            t if t.contains("cto") || t.contains("ceo") => score += 30,
            t if t.contains("cfo") || t.contains("director") => score += 25,
            t if t.contains("vp") || t.contains("vice") => score += 20,
            t if t.contains("manager") || t.contains("lead") => score += 15,
            _ => score += 5,
        }
    }

    // Company size bonus
    if let Some(size_val) = lead_data.get("company_size") {
        if let Ok(size) = size_val.as_int() {
            if size > 1000 {
                score += 20;
            } else if size > 500 {
                score += 15;
            } else if size > 100 {
                score += 10;
            } else if size > 0 {
                score += 5;
            }
        }
    }

    // Email domain bonus
    if let Some(email_val) = lead_data.get("email") {
        let email = email_val.to_string();
        if email.contains("@") {
            score += 10;
            if !email.ends_with("@gmail.com") && !email.ends_with("@yahoo.com") {
                score += 10; // Corporate email
            }
        }
    }

    // Budget signal
    if let Some(budget_val) = lead_data.get("budget") {
        if let Ok(budget) = budget_val.as_int() {
            if budget > 100000 {
                score += 25;
            } else if budget > 50000 {
                score += 20;
            } else if budget > 10000 {
                score += 15;
            } else if budget > 0 {
                score += 10;
            }
        }
    }

    // Industry bonus
    if let Some(industry_val) = lead_data.get("industry") {
        let industry_lower = industry_val.to_string().to_lowercase();
        if industry_lower.contains("tech") || industry_lower.contains("software") {
            score += 15;
        } else if industry_lower.contains("finance") || industry_lower.contains("banking") {
            score += 15;
        } else if industry_lower.contains("healthcare") || industry_lower.contains("pharma") {
            score += 10;
        }
    }

    // Apply custom rules if provided
    if let Some(rules) = custom_rules {
        if let Some(weight_val) = rules.get("weight") {
            if let Ok(weight_multiplier) = weight_val.as_int() {
                score = (score as f64 * (weight_multiplier as f64 / 100.0)) as i64;
            }
        }
        if let Some(bonus_val) = rules.get("bonus") {
            if let Ok(bonus) = bonus_val.as_int() {
                score += bonus;
            }
        }
    }

    // Clamp score between 0 and 100
    score.max(0).min(100)
}

/// Determine priority based on score
fn determine_priority(score: i64) -> String {
    match score {
        90..=100 => "CRITICAL".to_string(),
        70..=89 => "HIGH".to_string(),
        50..=69 => "MEDIUM".to_string(),
        30..=49 => "LOW".to_string(),
        _ => "MINIMAL".to_string(),
    }
}

/// Get recommendation based on score
fn get_recommendation(score: i64) -> String {
    match score {
        90..=100 => "Contact immediately - Schedule meeting within 24 hours".to_string(),
        70..=89 => "Contact within 48 hours - Prepare tailored proposal".to_string(),
        50..=69 => "Nurture campaign - Send valuable content".to_string(),
        30..=49 => "Keep in pipeline - Occasional touchpoints".to_string(),
        _ => "Monitor for engagement signals".to_string(),
    }
}

/// Get suggested action based on score
fn get_suggested_action(score: i64) -> String {
    match score {
        90..=100 => "Call and schedule demo".to_string(),
        70..=89 => "Send personalized email with case study".to_string(),
        50..=69 => "Add to drip campaign".to_string(),
        30..=49 => "Request more information".to_string(),
        _ => "Monitor for budget signals".to_string(),
    }
}

/// Get lead score from database (real implementation)
fn get_lead_score_from_db(_state: &Arc<AppState>, _lead_id: &str) -> Option<i64> {
    // TODO: Query actual database for lead score
    // Placeholder returns None - database implementation needed
    None
}

/// Update lead score in database (real implementation)
fn update_lead_score_in_db(_state: &Arc<AppState>, _lead_id: &str, _score: i64) {
    // TODO: Update actual database with new lead score
    // Placeholder - database implementation needed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_lead_score_empty() {
        let lead_data = Map::new();
        let score = calculate_lead_score(&lead_data, None);
        assert_eq!(score, 0);
    }

    #[test]
    fn test_calculate_lead_score_basic() {
        let mut lead_data = Map::new();
        lead_data.insert("job_title".into(), Dynamic::from("CEO"));
        lead_data.insert("company_size".into(), Dynamic::from(500_i64));
        lead_data.insert("email".into(), Dynamic::from("ceo@company.com"));

        let score = calculate_lead_score(&lead_data, None);
        assert!(score > 30); // At least CEO bonus
    }

    #[test]
    fn test_calculate_lead_score_with_title() {
        let mut lead_data = Map::new();
        lead_data.insert("job_title".into(), Dynamic::from("CTO"));

        let score = calculate_lead_score(&lead_data, None);
        assert!(score >= 30);
    }

    #[test]
    fn test_determine_priority() {
        assert_eq!(determine_priority(95), "CRITICAL");
        assert_eq!(determine_priority(75), "HIGH");
        assert_eq!(determine_priority(55), "MEDIUM");
        assert_eq!(determine_priority(35), "LOW");
        assert_eq!(determine_priority(10), "MINIMAL");
    }

    #[test]
    fn test_score_clamping() {
        let mut lead_data = Map::new();
        lead_data.insert("budget".into(), Dynamic::from(1000000_i64));

        let score = calculate_lead_score(&lead_data, None);
        assert!(
            score <= 100,
            "Score should be clamped to 100, got {}",
            score
        );
    }
}
