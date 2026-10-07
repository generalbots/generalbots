"""Lead scoring HTTP endpoints.

Thin transport layer over :mod:`src.services.scoring.engine`. The scoring rules
themselves live there; nothing here computes a score.
"""

from typing import Dict, List, Optional

from fastapi import APIRouter, Depends, HTTPException
from pydantic import BaseModel

from ....core.logging import get_logger
from ...dependencies import verify_api_key
from .engine import ENGINE_VERSION, generate_recommendations, score_lead
from .schemas import (
    BatchScoreRequest,
    BatchScoreResponse,
    LeadScoreResponse,
    ModelInfoResponse,
    ScoreLeadRequest,
)

logger = get_logger("scoring")

router = APIRouter(prefix="/scoring", tags=["Lead Scoring"])

# The features the engine consumes, reported by /model-info.
FEATURES_USED: List[str] = [
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
]

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

    Reports the scoring configuration and the features it consumes.

    This is a **deterministic rules engine**, not a trained model: there are no
    weights and no evaluation metrics. `accuracy_metrics` is therefore empty —
    the previous response reported `mql_precision: 0.85` and friends for a model
    that does not exist.

    Args:
        api_key: API key for authentication

    Returns:
        ModelInfoResponse with scoring metadata
    """
    return ModelInfoResponse(
        model_version=ENGINE_VERSION,
        features_used=FEATURES_USED,
        # No trained model, so no training date and no measured accuracy.
        last_trained=None,
        accuracy_metrics={},
    )


@router.get("/health")
async def scoring_health(
    api_key: str = Depends(verify_api_key),
):
    """Health check for scoring service."""
    return {"status": "healthy", "service": "lead_scoring", "kind": "rules-engine"}
