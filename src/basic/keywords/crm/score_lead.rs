








use crate::core::shared::schema::bot_memories;
use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use chrono::Utc;
use diesel::prelude::*;
use log::{debug, error, info, trace};
use rhai::{Dynamic, Engine, Map};
use std::sync::Arc;
use uuid::Uuid;


pub fn score_lead_keyword(_state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let user_clone = user.clone();


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


pub fn get_lead_score_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let user_clone = user.clone();
    let state_for_db = state.clone();


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


pub fn qualify_lead_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let user_clone = user.clone();
    let state_for_db = state.clone();


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


pub fn update_lead_score_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let user_clone = user.clone();
    let state_for_db = state.clone();


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


fn calculate_lead_score(lead_data: &Map, custom_rules: Option<&Map>) -> i64 {
    let mut score: i64 = 0;


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


    if let Some(email_val) = lead_data.get("email") {
        let email = email_val.to_string();
        if email.contains("@") {
            score += 10;
            if !email.ends_with("@gmail.com") && !email.ends_with("@yahoo.com") {
                score += 10;
            }
        }
    }


    if let Some(budget_val) = lead_data.get("budget") {
        if let Ok(budget) = budget_val.as_int() {
            if budget > 100_000 {
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


    score.max(0).min(100)
}


fn determine_priority(score: i64) -> String {
    match score {
        90..=100 => "CRITICAL".to_string(),
        70..=89 => "HIGH".to_string(),
        50..=69 => "MEDIUM".to_string(),
        30..=49 => "LOW".to_string(),
        _ => "MINIMAL".to_string(),
    }
}


fn get_recommendation(score: i64) -> String {
    match score {
        90..=100 => "Contact immediately - Schedule meeting within 24 hours".to_string(),
        70..=89 => "Contact within 48 hours - Prepare tailored proposal".to_string(),
        50..=69 => "Nurture campaign - Send valuable content".to_string(),
        30..=49 => "Keep in pipeline - Occasional touchpoints".to_string(),
        _ => "Monitor for engagement signals".to_string(),
    }
}


fn get_suggested_action(score: i64) -> String {
    match score {
        90..=100 => "Call and schedule demo".to_string(),
        70..=89 => "Send personalized email with case study".to_string(),
        50..=69 => "Add to drip campaign".to_string(),
        30..=49 => "Request more information".to_string(),
        _ => "Monitor for budget signals".to_string(),
    }
}



fn get_lead_score_from_db(state: &Arc<AppState>, lead_id: &str) -> Option<i64> {
    let memory_key = format!("lead_score:{}", lead_id);

    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            error!(
                "Failed to get database connection for lead score lookup: {}",
                e
            );
            return None;
        }
    };



    let result = bot_memories::table
        .filter(bot_memories::key.eq(&memory_key))
        .select(bot_memories::value)
        .first::<String>(&mut conn)
        .optional();

    match result {
        Ok(Some(value)) => match value.parse::<i64>() {
            Ok(score) => {
                debug!("Retrieved lead score {} for lead {}", score, lead_id);
                Some(score)
            }
            Err(e) => {
                error!(
                    "Failed to parse lead score '{}' for lead {}: {}",
                    value, lead_id, e
                );
                None
            }
        },
        Ok(None) => {
            debug!("No lead score found for lead {}", lead_id);
            None
        }
        Err(e) => {
            error!(
                "Database error retrieving lead score for {}: {}",
                lead_id, e
            );
            None
        }
    }
}



fn update_lead_score_in_db(state: &Arc<AppState>, lead_id: &str, score: i64) {
    let memory_key = format!("lead_score:{}", lead_id);
    let score_value = score.to_string();
    let now = Utc::now();

    let mut conn = match state.conn.get() {
        Ok(c) => c,
        Err(e) => {
            error!(
                "Failed to get database connection for lead score update: {}",
                e
            );
            return;
        }
    };


    let existing = bot_memories::table
        .filter(bot_memories::key.eq(&memory_key))
        .select(bot_memories::id)
        .first::<Uuid>(&mut conn)
        .optional();

    match existing {
        Ok(Some(existing_id)) => {

            let update_result = diesel::update(bot_memories::table.find(existing_id))
                .set((
                    bot_memories::value.eq(&score_value),
                    bot_memories::updated_at.eq(now),
                ))
                .execute(&mut conn);

            match update_result {
                Ok(_) => {
                    info!("Updated lead score to {} for lead {}", score, lead_id);
                }
                Err(e) => {
                    error!("Failed to update lead score for {}: {}", lead_id, e);
                }
            }
        }
        Ok(None) => {

            let new_id = Uuid::new_v4();
            let bot_id = Uuid::nil();

            let insert_result = diesel::insert_into(bot_memories::table)
                .values((
                    bot_memories::id.eq(new_id),
                    bot_memories::bot_id.eq(bot_id),
                    bot_memories::key.eq(&memory_key),
                    bot_memories::value.eq(&score_value),
                    bot_memories::created_at.eq(now),
                    bot_memories::updated_at.eq(now),
                ))
                .execute(&mut conn);

            match insert_result {
                Ok(_) => {
                    info!("Inserted new lead score {} for lead {}", score, lead_id);
                }
                Err(e) => {
                    error!("Failed to insert lead score for {}: {}", lead_id, e);
                }
            }
        }
        Err(e) => {
            error!(
                "Database error checking existing lead score for {}: {}",
                lead_id, e
            );
        }
    }
}
