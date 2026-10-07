"""Deterministic lead-scoring rules engine.

This is a weighted rules engine, not a trained model: there are no weights to
load and no evaluation metrics to report. It is separated from the HTTP layer so
the rules are testable on their own.
"""

from datetime import datetime

from .schemas import (
    LeadBehavior,
    LeadProfile,
    LeadScoreResponse,
    ScoreBreakdown,
    ScoreLeadRequest,
)

# Version of the rules engine itself (a scoring-config version, not a model).
ENGINE_VERSION = "1.0.0"

class ScoringWeights:
    """Default weights for scoring components"""

    COMPANY_SIZE_WEIGHT = 10.0
    INDUSTRY_MATCH_WEIGHT = 15.0
    LOCATION_MATCH_WEIGHT = 5.0
    JOB_TITLE_WEIGHT = 15.0

    EMAIL_OPENS_WEIGHT = 5.0
    EMAIL_CLICKS_WEIGHT = 10.0
    PAGE_VISITS_WEIGHT = 5.0
    FORM_SUBMISSIONS_WEIGHT = 15.0
    CONTENT_DOWNLOADS_WEIGHT = 10.0

    RESPONSE_TIME_WEIGHT = 10.0
    INTERACTION_FREQUENCY_WEIGHT = 10.0
    SESSION_DURATION_WEIGHT = 5.0

    PRICING_PAGE_WEIGHT = 20.0
    DEMO_REQUEST_WEIGHT = 25.0
    TRIAL_SIGNUP_WEIGHT = 30.0

    INACTIVITY_PENALTY = -15.0


TARGET_INDUSTRIES = {
    "technology": 1.0,
    "software": 1.0,
    "saas": 1.0,
    "finance": 0.9,
    "fintech": 0.9,
    "banking": 0.9,
    "healthcare": 0.8,
    "medical": 0.8,
    "retail": 0.7,
    "ecommerce": 0.7,
    "manufacturing": 0.6,
    "education": 0.5,
    "nonprofit": 0.5,
}

TITLE_SCORES = {
    "ceo": 1.0,
    "cto": 1.0,
    "cfo": 1.0,
    "chief": 1.0,
    "founder": 1.0,
    "president": 0.95,
    "vp": 0.9,
    "vice president": 0.9,
    "director": 0.85,
    "head": 0.8,
    "manager": 0.7,
    "senior": 0.6,
    "lead": 0.6,
}

COMPANY_SIZE_SCORES = {
    "enterprise": 1.0,
    "1000+": 1.0,
    ">1000": 1.0,
    "mid-market": 0.8,
    "100-999": 0.8,
    "mid": 0.8,
    "smb": 0.6,
    "small": 0.6,
    "10-99": 0.6,
    "startup": 0.4,
    "1-9": 0.4,
    "<10": 0.4,
}


def calculate_demographic_score(profile: LeadProfile) -> float:
    """Calculate demographic component of lead score"""
    score = 0.0
    weights = ScoringWeights()

    if profile.company_size:
        size_lower = profile.company_size.lower()
        for key, value in COMPANY_SIZE_SCORES.items():
            if key in size_lower:
                score += value * weights.COMPANY_SIZE_WEIGHT
                break
        else:
            score += 0.3 * weights.COMPANY_SIZE_WEIGHT

    if profile.industry:
        industry_lower = profile.industry.lower()
        for key, value in TARGET_INDUSTRIES.items():
            if key in industry_lower:
                score += value * weights.INDUSTRY_MATCH_WEIGHT
                break
        else:
            score += 0.4 * weights.INDUSTRY_MATCH_WEIGHT

    if profile.job_title:
        title_lower = profile.job_title.lower()
        title_score = 0.3
        for key, value in TITLE_SCORES.items():
            if key in title_lower:
                title_score = max(title_score, value)
        score += title_score * weights.JOB_TITLE_WEIGHT

    if profile.location:
        score += 0.5 * weights.LOCATION_MATCH_WEIGHT

    return score


def calculate_behavioral_score(behavior: LeadBehavior) -> float:
    """Calculate behavioral component of lead score"""
    score = 0.0
    weights = ScoringWeights()

    email_open_score = min(behavior.email_opens / 10.0, 1.0)
    score += email_open_score * weights.EMAIL_OPENS_WEIGHT

    email_click_score = min(behavior.email_clicks / 5.0, 1.0)
    score += email_click_score * weights.EMAIL_CLICKS_WEIGHT

    visit_score = min(behavior.page_visits / 20.0, 1.0)
    score += visit_score * weights.PAGE_VISITS_WEIGHT

    form_score = min(behavior.form_submissions / 3.0, 1.0)
    score += form_score * weights.FORM_SUBMISSIONS_WEIGHT

    download_score = min(behavior.content_downloads / 5.0, 1.0)
    score += download_score * weights.CONTENT_DOWNLOADS_WEIGHT

    return score


def calculate_engagement_score(behavior: LeadBehavior) -> float:
    """Calculate engagement component of lead score"""
    score = 0.0
    weights = ScoringWeights()

    frequency_score = min(behavior.total_sessions / 10.0, 1.0)
    score += frequency_score * weights.INTERACTION_FREQUENCY_WEIGHT

    duration_score = min(behavior.avg_session_duration / 300.0, 1.0)
    score += duration_score * weights.SESSION_DURATION_WEIGHT

    if behavior.days_since_last_activity is not None:
        days = behavior.days_since_last_activity
        if days <= 1:
            recency_score = 1.0
        elif days <= 7:
            recency_score = 0.8
        elif days <= 14:
            recency_score = 0.6
        elif days <= 30:
            recency_score = 0.4
        elif days <= 60:
            recency_score = 0.2
        else:
            recency_score = 0.0
        score += recency_score * weights.RESPONSE_TIME_WEIGHT

    return score


def calculate_intent_score(behavior: LeadBehavior) -> float:
    """Calculate intent signal component of lead score"""
    score = 0.0
    weights = ScoringWeights()

    if behavior.pricing_page_visits > 0:
        pricing_score = min(behavior.pricing_page_visits / 3.0, 1.0)
        score += pricing_score * weights.PRICING_PAGE_WEIGHT

    if behavior.demo_requests > 0:
        score += weights.DEMO_REQUEST_WEIGHT

    if behavior.trial_signups > 0:
        score += weights.TRIAL_SIGNUP_WEIGHT

    return score


def calculate_penalty_score(behavior: LeadBehavior) -> float:
    """Calculate penalty deductions"""
    penalty = 0.0
    weights = ScoringWeights()

    if behavior.days_since_last_activity is not None:
        if behavior.days_since_last_activity > 60:
            penalty += weights.INACTIVITY_PENALTY
        elif behavior.days_since_last_activity > 30:
            penalty += weights.INACTIVITY_PENALTY * 0.5
    elif behavior.total_sessions == 0:
        penalty += weights.INACTIVITY_PENALTY

    return penalty


def get_grade(score: float) -> str:
    """Determine lead grade based on score"""
    if score >= 80:
        return "A"
    elif score >= 60:
        return "B"
    elif score >= 40:
        return "C"
    elif score >= 20:
        return "D"
    else:
        return "F"


def get_qualification_status(
    score: float, has_demo: bool = False, has_trial: bool = False
) -> str:
    """Determine qualification status"""
    if has_trial or score >= 90:
        return "sql"
    elif has_demo or score >= 70:
        return "mql"
    else:
        return "unqualified"


def generate_recommendations(
    profile: LeadProfile, behavior: LeadBehavior, score: float
) -> List[str]:
    """Generate actionable recommendations for the lead"""
    recommendations = []

    if score >= 80:
        recommendations.append("Hot lead! Prioritize immediate sales outreach.")
    elif score >= 60:
        recommendations.append("Warm lead - consider scheduling a discovery call.")
    elif score >= 40:
        recommendations.append("Continue nurturing with targeted content.")
    else:
        recommendations.append("Low priority - add to nurturing campaign.")

    if behavior.pricing_page_visits > 0 and behavior.demo_requests == 0:
        recommendations.append("Visited pricing page - send personalized demo invite.")

    if behavior.content_downloads > 2 and behavior.form_submissions == 1:
        recommendations.append(
            "High content engagement - offer exclusive webinar access."
        )

    if behavior.email_opens > 5 and behavior.email_clicks < 2:
        recommendations.append("Opens emails but doesn't click - try different CTAs.")

    if not profile.company:
        recommendations.append("Missing company info - enrich profile data.")

    if not profile.job_title:
        recommendations.append("Unknown job title - request more information.")

    if behavior.days_since_last_activity and behavior.days_since_last_activity > 14:
        recommendations.append("Inactive for 2+ weeks - send re-engagement email.")

    return recommendations


def score_lead(request: ScoreLeadRequest) -> LeadScoreResponse:
    """Calculate comprehensive lead score"""
    profile = request.profile
    behavior = request.behavior or LeadBehavior()

    demographic_score = calculate_demographic_score(profile)
    behavioral_score = calculate_behavioral_score(behavior)
    engagement_score = calculate_engagement_score(behavior)
    intent_score = calculate_intent_score(behavior)
    penalty_score = calculate_penalty_score(behavior)

    raw_score = (
        demographic_score
        + behavioral_score
        + engagement_score
        + intent_score
        + penalty_score
    )
    total_score = max(0, min(100, raw_score))

    grade = get_grade(total_score)
    qualification_status = get_qualification_status(
        total_score,
        has_demo=behavior.demo_requests > 0,
        has_trial=behavior.trial_signups > 0,
    )

    recommendations = []
    if request.include_recommendations:
        recommendations = generate_recommendations(profile, behavior, total_score)

    data_points = sum(
        [
            1 if profile.email else 0,
            1 if profile.name else 0,
            1 if profile.company else 0,
            1 if profile.job_title else 0,
            1 if profile.industry else 0,
            1 if profile.company_size else 0,
            1 if behavior.total_sessions > 0 else 0,
            1 if behavior.email_opens > 0 else 0,
        ]
    )
    confidence = min(data_points / 8.0, 1.0)

    return LeadScoreResponse(
        lead_id=profile.lead_id or profile.email or "unknown",
        total_score=round(total_score, 2),
        grade=grade,
        qualification_status=qualification_status,
        breakdown=ScoreBreakdown(
            demographic=round(demographic_score, 2),
            behavioral=round(behavioral_score, 2),
            engagement=round(engagement_score, 2),
            intent=round(intent_score, 2),
            penalties=round(penalty_score, 2),
        ),
        recommendations=recommendations,
        confidence=round(confidence, 2),
        calculated_at=datetime.utcnow(),
    )


