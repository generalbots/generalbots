"""Pydantic request/response models for lead scoring."""

from datetime import datetime
from typing import Dict, List, Optional

from pydantic import BaseModel, EmailStr, Field


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


