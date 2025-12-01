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
use log::{debug, trace};
use rhai::{Dynamic, Engine, Map};
use std::sync::Arc;

/// SCORE LEAD - Calculate lead score based on provided criteria
///
/// BASIC Syntax:
///   score = SCORE LEAD(lead_data)
///   score = SCORE LEAD(lead_data, scoring_rules)
///
/// Examples:
///   lead = #{"email": "john@company.com", "job_title": "CTO", "company_size": 500}
///   score = SCORE LEAD(lead)
pub fn score_lead_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();
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

    let state_clone2 = state.clone();
    let user_clone2 = user.clone();

    // score lead lowercase version
    engine.register_fn("score lead", move |lead_data: Map| -> i64 {
        trace!(
            "score lead called for user {} with data: {:?}",
            user_clone2.user_id,
            lead_data
        );
        calculate_lead_score(&lead_data, None)
    });

    // SCORE LEAD with custom scoring rules
    let _state_clone3 = state.clone();
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

    let _ = state_clone;
    debug!("Registered SCORE LEAD keyword");
}

/// GET LEAD SCORE - Retrieve stored lead score from database
///
/// BASIC Syntax:
///   score = GET LEAD SCORE(lead_id)
///   score_data = GET LEAD SCORE(lead_id, "full")
pub fn get_lead_score_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let _state_clone = state.clone();
    let user_clone = user.clone();

    // GET LEAD SCORE - returns numeric score
    engine.register_fn("GET LEAD SCORE", move |lead_id: &str| -> i64 {
        trace!(
            "GET LEAD SCORE called for lead {} by user {}",
            lead_id,
            user_clone.user_id
        );
        // TODO: Implement database lookup
        // For now, return a placeholder score
        50
    });

    let _state_clone2 = state.clone();
    let user_clone2 = user.clone();

    // get lead score lowercase
    engine.register_fn("get lead score", move |lead_id: &str| -> i64 {
        trace!(
            "get lead score called for lead {} by user {}",
            lead_id,
            user_clone2.user_id
        );
        50
    });

    // GET LEAD SCORE with "full" option - returns map with score details
    let _state_clone3 = state.clone();
    let user_clone3 = user.clone();

    engine.register_fn(
        "GET LEAD SCORE",
        move |lead_id: &str, option: &str| -> Map {
            trace!(
                "GET LEAD SCORE (full) called for lead {} by user {}",
                lead_id,
                user_clone3.user_id
            );

            let mut result = Map::new();
            result.insert("lead_id".into(), Dynamic::from(lead_id.to_string()));
            result.insert("score".into(), Dynamic::from(50_i64));
            result.insert("qualified".into(), Dynamic::from(false));
            result.insert("last_updated".into(), Dynamic::from("2024-01-01T00:00:00Z"));

            if option.eq_ignore_ascii_case("full") {
                result.insert("engagement_score".into(), Dynamic::from(30_i64));
                result.insert("demographic_score".into(), Dynamic::from(20_i64));
                result.insert("behavioral_score".into(), Dynamic::from(0_i64));
            }

            result
        },
    );

    debug!("Registered GET LEAD SCORE keyword");
}

/// QUALIFY LEAD - Check if lead meets qualification threshold
///
/// BASIC Syntax:
///   is_qualified = QUALIFY LEAD(lead_id)
///   is_qualified = QUALIFY LEAD(lead_id, threshold)
pub fn qualify_lead_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let _state_clone = state.clone();
    let user_clone = user.clone();

    // QUALIFY LEAD with default threshold (70)
    engine.register_fn("QUALIFY LEAD", move |lead_id: &str| -> bool {
        trace!(
            "QUALIFY LEAD called for lead {} by user {}",
            lead_id,
            user_clone.user_id
        );
        // TODO: Get actual score from database
        let score = 50_i64;
        score >= 70
    });

    let _state_clone2 = state.clone();
    let user_clone2 = user.clone();

    // qualify lead lowercase
    engine.register_fn("qualify lead", move |lead_id: &str| -> bool {
        trace!(
            "qualify lead called for lead {} by user {}",
            lead_id,
            user_clone2.user_id
        );
        let score = 50_i64;
        score >= 70
    });

    // QUALIFY LEAD with custom threshold
    let _state_clone3 = state.clone();
    let user_clone3 = user.clone();

    engine.register_fn(
        "QUALIFY LEAD",
        move |lead_id: &str, threshold: i64| -> bool {
            trace!(
                "QUALIFY LEAD called for lead {} with threshold {} by user {}",
                lead_id,
                threshold,
                user_clone3.user_id
            );
            // TODO: Get actual score from database
            let score = 50_i64;
            score >= threshold
        },
    );

    // IS QUALIFIED alias
    let _state_clone4 = state.clone();
    let user_clone4 = user.clone();

    engine.register_fn("IS QUALIFIED", move |lead_id: &str| -> bool {
        trace!(
            "IS QUALIFIED called for lead {} by user {}",
            lead_id,
            user_clone4.user_id
        );
        let score = 50_i64;
        score >= 70
    });

    debug!("Registered QUALIFY LEAD keyword");
}

/// UPDATE LEAD SCORE - Manually adjust lead score
///
/// BASIC Syntax:
///   UPDATE LEAD SCORE lead_id, adjustment
///   UPDATE LEAD SCORE lead_id, adjustment, "reason"
pub fn update_lead_score_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let _state_clone = state.clone();
    let user_clone = user.clone();

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
            // TODO: Update database and return new score
            50 + adjustment
        },
    );

    let _state_clone2 = state.clone();
    let user_clone2 = user.clone();

    // UPDATE LEAD SCORE with reason
    engine.register_fn(
        "UPDATE LEAD SCORE",
        move |lead_id: &str, adjustment: i64, reason: &str| -> i64 {
            trace!(
                "UPDATE LEAD SCORE called for lead {} with adjustment {} reason '{}' by user {}",
                lead_id,
                adjustment,
                reason,
                user_clone2.user_id
            );
            // TODO: Update database with audit trail
            50 + adjustment
        },
    );

    // SET LEAD SCORE - set absolute score
    let _state_clone3 = state.clone();
    let user_clone3 = user.clone();

    engine.register_fn("SET LEAD SCORE", move |lead_id: &str, score: i64| -> i64 {
        trace!(
            "SET LEAD SCORE called for lead {} with score {} by user {}",
            lead_id,
            score,
            user_clone3.user_id
        );
        // TODO: Update database
        score
    });

    debug!("Registered UPDATE LEAD SCORE keyword");
}

/// AI SCORE LEAD - LLM-enhanced lead scoring
///
/// BASIC Syntax:
///   score = AI SCORE LEAD(lead_data)
///   score = AI SCORE LEAD(lead_data, context)
///
/// Uses AI to analyze lead data and provide intelligent scoring
pub fn ai_score_lead_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = state.clone();
    let user_clone = user.clone();

    // AI SCORE LEAD with lead data
    engine.register_fn("AI SCORE LEAD", move |lead_data: Map| -> Map {
        trace!(
            "AI SCORE LEAD called for user {} with data: {:?}",
            user_clone.user_id,
            lead_data
        );

        // Calculate base score
        let base_score = calculate_lead_score(&lead_data, None);

        // TODO: Call LLM service for enhanced scoring
        // For now, return enhanced result with placeholder AI analysis

        let mut result = Map::new();
        result.insert("score".into(), Dynamic::from(base_score));
        result.insert("confidence".into(), Dynamic::from(0.85_f64));
        result.insert(
            "recommendation".into(),
            Dynamic::from("Follow up within 24 hours"),
        );
        result.insert(
            "priority".into(),
            Dynamic::from(determine_priority(base_score)),
        );

        // Add scoring breakdown
        let mut breakdown = Map::new();
        breakdown.insert("engagement".into(), Dynamic::from(30_i64));
        breakdown.insert("demographics".into(), Dynamic::from(25_i64));
        breakdown.insert("behavior".into(), Dynamic::from(20_i64));
        breakdown.insert("fit".into(), Dynamic::from(base_score - 75));
        result.insert("breakdown".into(), Dynamic::from(breakdown));

        result
    });

    let _state_clone2 = state.clone();
    let user_clone2 = user.clone();

    // ai score lead lowercase
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
            "recommendation".into(),
            Dynamic::from("Follow up within 24 hours"),
        );
        result.insert(
            "priority".into(),
            Dynamic::from(determine_priority(base_score)),
        );

        result
    });

    // AI SCORE LEAD with context
    let _state_clone3 = state.clone();
    let user_clone3 = user.clone();

    engine.register_fn(
        "AI SCORE LEAD",
        move |lead_data: Map, context: &str| -> Map {
            trace!(
                "AI SCORE LEAD called for user {} with context: {}",
                user_clone3.user_id,
                context
            );

            let base_score = calculate_lead_score(&lead_data, None);

            let mut result = Map::new();
            result.insert("score".into(), Dynamic::from(base_score));
            result.insert("confidence".into(), Dynamic::from(0.90_f64));
            result.insert("context_used".into(), Dynamic::from(context.to_string()));
            result.insert(
                "priority".into(),
                Dynamic::from(determine_priority(base_score)),
            );

            result
        },
    );

    let _ = state_clone;
    debug!("Registered AI SCORE LEAD keyword");
}

/// Calculate lead score based on lead data and optional custom rules
fn calculate_lead_score(lead_data: &Map, custom_rules: Option<&Map>) -> i64 {
    let mut score: i64 = 0;

    // Default scoring criteria
    let default_weights: Vec<(&str, i64)> = vec![
        ("email", 10),
        ("phone", 10),
        ("company", 15),
        ("job_title", 20),
        ("company_size", 15),
        ("industry", 10),
        ("budget", 20),
    ];

    // Job title bonuses
    let title_bonuses: Vec<(&str, i64)> = vec![
        ("cto", 25),
        ("ceo", 30),
        ("cfo", 25),
        ("vp", 20),
        ("director", 15),
        ("manager", 10),
        ("head", 15),
        ("chief", 25),
    ];

    // Apply default scoring
    for (field, weight) in &default_weights {
        if lead_data.contains_key(*field) {
            let value = lead_data.get(*field).unwrap();
            if !value.is_unit() && !value.to_string().is_empty() {
                score += weight;
            }
        }
    }

    // Apply job title bonuses
    if let Some(title) = lead_data.get("job_title") {
        let title_str = title.to_string().to_lowercase();
        for (keyword, bonus) in &title_bonuses {
            if title_str.contains(keyword) {
                score += bonus;
                break; // Only apply one bonus
            }
        }
    }

    // Apply company size scoring
    if let Some(size) = lead_data.get("company_size") {
        if let Ok(size_num) = size.as_int() {
            score += match size_num {
                0..=10 => 5,
                11..=50 => 10,
                51..=200 => 15,
                201..=1000 => 20,
                _ => 25,
            };
        }
    }

    // Apply custom rules if provided
    if let Some(rules) = custom_rules {
        for (field, weight) in rules.iter() {
            if lead_data.contains_key(field.as_str()) {
                if let Ok(w) = weight.as_int() {
                    score += w;
                }
            }
        }
    }

    // Normalize score to 0-100 range
    score.clamp(0, 100)
}

/// Determine priority based on score
fn determine_priority(score: i64) -> &'static str {
    match score {
        0..=30 => "low",
        31..=60 => "medium",
        61..=80 => "high",
        _ => "critical",
    }
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
        lead_data.insert("email".into(), Dynamic::from("test@example.com"));
        lead_data.insert("company".into(), Dynamic::from("Acme Inc"));

        let score = calculate_lead_score(&lead_data, None);
        assert!(score > 0);
        assert!(score <= 100);
    }

    #[test]
    fn test_calculate_lead_score_with_title() {
        let mut lead_data = Map::new();
        lead_data.insert("email".into(), Dynamic::from("cto@example.com"));
        lead_data.insert("job_title".into(), Dynamic::from("CTO"));

        let score = calculate_lead_score(&lead_data, None);
        // Should include email (10) + job_title (20) + CTO bonus (25) = 55
        assert!(score >= 50);
    }

    #[test]
    fn test_determine_priority() {
        assert_eq!(determine_priority(20), "low");
        assert_eq!(determine_priority(50), "medium");
        assert_eq!(determine_priority(70), "high");
        assert_eq!(determine_priority(90), "critical");
    }

    #[test]
    fn test_score_clamping() {
        let mut lead_data = Map::new();
        // Add lots of data to potentially exceed 100
        lead_data.insert("email".into(), Dynamic::from("test@example.com"));
        lead_data.insert("phone".into(), Dynamic::from("555-1234"));
        lead_data.insert("company".into(), Dynamic::from("Big Corp"));
        lead_data.insert("job_title".into(), Dynamic::from("CEO"));
        lead_data.insert("company_size".into(), Dynamic::from(5000_i64));
        lead_data.insert("industry".into(), Dynamic::from("Technology"));
        lead_data.insert("budget".into(), Dynamic::from("$1M"));

        let score = calculate_lead_score(&lead_data, None);
        assert!(score <= 100);
    }
}
