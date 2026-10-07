"""Lead scoring: deterministic rules engine behind thin HTTP endpoints.

`engine` holds the scoring rules, `schemas` the pydantic models.
"""

from .engine import ENGINE_VERSION, score_lead
from .schemas import (
    BatchScoreRequest,
    BatchScoreResponse,
    LeadBehavior,
    LeadProfile,
    LeadScoreResponse,
    ModelInfoResponse,
    ScoreBreakdown,
    ScoreLeadRequest,
)

__all__ = [
    "ENGINE_VERSION",
    "score_lead",
    "BatchScoreRequest",
    "BatchScoreResponse",
    "LeadBehavior",
    "LeadProfile",
    "LeadScoreResponse",
    "ModelInfoResponse",
    "ScoreBreakdown",
    "ScoreLeadRequest",
]