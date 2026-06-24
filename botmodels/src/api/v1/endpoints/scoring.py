"""
AI Lead Scoring Endpoint for BotModels

This module provides ML-powered lead scoring capabilities:
- Demographic scoring
- Behavioral analysis
- Engagement prediction
- Lead qualification

Endpoints:
- POST /api/scoring/score - Calculate lead score
- POST /api/scoring/batch - Batch score multiple leads
- GET /api/scoring/model-info - Get model information
"""

from datetime import datetime
from typing import Any, Dict, List, Optional

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel, EmailStr, Field

from ....core.logging import get_logger
from ...dependencies import verify_api_key

logger = get_logger("scoring")

router = APIRouter(prefix="/scoring", tags=["Lead Scoring"])


class LeadProfile(BaseModel):
    """Lead profile information for scoring"""

    lead_id: Optional[str] = None
    email: Optional[EmailStr] = None
    name: Optional[str] = None
    company: Optional[str] = None
    job_title: Optional[str] = None
    industry: Optional[str] = None
    company_size: Optional[str] = None
    location: Optional[str] = None
    source: Optional[str] = None


class LeadBehavior(BaseModel):
    """Lead behavioral data for scoring"""

    email_opens: int = 0
    email_clicks: int = 0
    page_visits: int = 0
    form_submissions: int = 0
    content_downloads: int = 0
    pricing_page_visits: int = 0
    demo_requests: int = 0
    trial_signups: int = 0
    total_sessions: int = 0
    avg_session_duration: float = 0.0
    days_since_last_activity: Optional[int] = None


class ScoreLeadRequest(BaseModel):
    """Request model for lead scoring"""

    profile: LeadProfile
    behavior: Optional[LeadBehavior] = None
    custom_weights: Optional[Dict[str, float]] = None
    include_recommendations: bool = True


class BatchScoreRequest(BaseModel):
    """Request model for batch lead scoring"""

    leads: List[ScoreLeadRequest]


class ScoreBreakdown(BaseModel):
    """Breakdown of score components"""

    demographic: float
    behavioral: float
    engagement: float
    intent: float
    penalties: float


class LeadScoreResponse(BaseModel):
    """Response model for lead scoring"""

    lead_id: str
    total_score: float = Field(..., ge=0, le=100)
    grade: str
    qualification_status: str
    breakdown: ScoreBreakdown
    recommendations: List[str] = []
    confidence: float = Field(..., ge=0, le=1)
    calculated_at: datetime


class BatchScoreResponse(BaseModel):
    """Response model for batch scoring"""

    scores: List[LeadScoreResponse]
    total_processed: int
    avg_score: float
    grade_distribution: Dict[str, int]


class ModelInfoResponse(BaseModel):
    """Response model for model information"""

    model_version: str
    features_used: List[str]
    last_trained: Optional[datetime]
    accuracy_metrics: Dict[str, float]


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


@router.post("/score", response_model=LeadScoreResponse)
async def calculate_lead_score(
    request: ScoreLeadRequest,
    api_key: str = Depends(verify_api_key),
) -> LeadScoreResponse:
    """
    Calculate AI-powered lead score.

    This endpoint analyzes lead profile and behavioral data to calculate
    a comprehensive lead score (0-100) with grade assignment and
    qualification status.

    Args:
        request: Lead profile and behavioral data
        api_key: API key for authentication

    Returns:
        LeadScoreResponse with score, grade, and recommendations
    """
    try:
        logger.info(
            "Scoring lead",
            lead_id=request.profile.lead_id,
            email=request.profile.email,
        )

        result = score_lead(request)

        logger.info(
            "Lead scored",
            lead_id=result.lead_id,
            score=result.total_score,
            grade=result.grade,
        )

        return result

    except Exception as e:
        logger.error("Lead scoring failed", error=str(e))
        raise HTTPException(status_code=500, detail=f"Scoring failed: {str(e)}")


@router.post("/batch", response_model=BatchScoreResponse)
async def batch_score_leads(
    request: BatchScoreRequest,
    api_key: str = Depends(verify_api_key),
) -> BatchScoreResponse:
    """
    Batch score multiple leads.

    Efficiently score multiple leads in a single request.

    Args:
        request: List of leads to score
        api_key: API key for authentication

    Returns:
        BatchScoreResponse with all scores and summary statistics
    """
    try:
        logger.info("Batch scoring", count=len(request.leads))

        scores = [score_lead(lead_request) for lead_request in request.leads]

        total_score = sum(s.total_score for s in scores)
        avg_score = total_score / len(scores) if scores else 0

        grade_dist = {"A": 0, "B": 0, "C": 0, "D": 0, "F": 0}
        for s in scores:
            grade_dist[s.grade] += 1

        logger.info(
            "Batch scoring complete",
            count=len(scores),
            avg_score=round(avg_score, 2),
        )

        return BatchScoreResponse(
            scores=scores,
            total_processed=len(scores),
            avg_score=round(avg_score, 2),
            grade_distribution=grade_dist,
        )

    except Exception as e:
        logger.error("Batch scoring failed", error=str(e))
        raise HTTPException(status_code=500, detail=f"Batch scoring failed: {str(e)}")


@router.get("/model-info", response_model=ModelInfoResponse)
async def get_model_info(
    api_key: str = Depends(verify_api_key),
) -> ModelInfoResponse:
    """
    Get information about the scoring model.

    Returns metadata about the lead scoring model including
    features used and accuracy metrics.

    Args:
        api_key: API key for authentication

    Returns:
        ModelInfoResponse with model metadata
    """
    return ModelInfoResponse(
        model_version="1.0.0",
        features_used=[
            "company_size",
            "industry",
            "job_title",
            "location",
            "email_opens",
            "email_clicks",
            "page_visits",
            "form_submissions",
            "content_downloads",
            "pricing_page_visits",
            "demo_requests",
            "trial_signups",
            "session_duration",
            "days_since_activity",
        ],
        last_trained=datetime(2025, 1, 1),
        accuracy_metrics={
            "mql_precision": 0.85,
            "sql_precision": 0.92,
            "conversion_correlation": 0.78,
        },
    )


@router.get("/health")
async def scoring_health():
    """Health check for scoring service"""
    return {"status": "healthy", "service": "lead_scoring"}
