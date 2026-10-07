"""
Generic Anomaly Detection API Endpoints
"""

import math

from fastapi import APIRouter, Depends
from pydantic import BaseModel

from ....services.anomaly_service import get_anomaly_service
from ...dependencies import require_api_key

router = APIRouter()


class AnomalyRequest(BaseModel):
    data: list[dict]
    value_field: str = "value"


def _median(values: list[float]) -> float:
    """True median: average of the two middle values for even-length input."""
    ordered = sorted(values)
    midpoint = len(ordered) // 2
    if len(ordered) % 2 == 1:
        return ordered[midpoint]
    return (ordered[midpoint - 1] + ordered[midpoint]) / 2


def _population_std(values: list[float]) -> float:
    if not values:
        return 0.0
    mean = sum(values) / len(values)
    return math.sqrt(sum((x - mean) ** 2 for x in values) / len(values))


@router.post("/detect")
async def detect_anomalies(
    request: AnomalyRequest,
    api_key: str = Depends(require_api_key),
):
    """
    Generic anomaly detection endpoint
    Works with any numerical data - salaries, sensors, metrics, etc.
    """
    service = get_anomaly_service()

    # Track the original row index alongside each parsed value: filtering out
    # non-numeric rows and then indexing request.data positionally misaligned
    # every anomaly after the first gap.
    valid_indices: list[int] = []
    values: list[float] = []

    for index, row in enumerate(request.data):
        raw = row.get(request.value_field)
        if raw is None:
            continue
        try:
            values.append(float(raw))
            valid_indices.append(index)
        except (TypeError, ValueError):
            continue

    if not values:
        return {
            "error": f"Field '{request.value_field}' not found in data",
            "data_sample": request.data[0] if request.data else None,
            "available_fields": list(request.data[0].keys()) if request.data else [],
        }

    zscore_results = service.detect_zscore(values, threshold=2.5)
    iqr_results = service.detect_iqr(values, multiplier=1.5)

    anomalies = []
    for i in range(len(values)):
        votes = sum(
            [
                zscore_results[i].is_anomaly if zscore_results else False,
                iqr_results[i].is_anomaly if iqr_results else False,
            ]
        )

        if votes >= 1:
            anomalies.append(
                {
                    "index": valid_indices[i],
                    "record": request.data[valid_indices[i]],
                    "value": values[i],
                    "confidence": votes / 2,
                    "methods": {
                        "zscore": zscore_results[i].is_anomaly
                        if zscore_results
                        else False,
                        "iqr": iqr_results[i].is_anomaly if iqr_results else False,
                    },
                    "zscore_score": zscore_results[i].score if zscore_results else 0,
                    "iqr_details": iqr_results[i].details if iqr_results else {},
                }
            )

    return {
        "detected": len(anomalies) > 0,
        "total_records": len(request.data),
        "anomalies_found": len(anomalies),
        "anomaly_rate": len(anomalies) / len(request.data) if request.data else 0,
        "anomalies": anomalies,
        "summary": {
            "mean": float(sum(values) / len(values)),
            "median": _median(values),
            "std": _population_std(values),
            "min": min(values) if values else 0,
            "max": max(values) if values else 0,
            "analyzed": len(values),
        },
    }
