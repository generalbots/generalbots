"""
Generic Anomaly Detection API Endpoints
"""

from fastapi import APIRouter, HTTPException, Header
from typing import Optional
from pydantic import BaseModel

from ...dependencies import get_api_key
from ....services.anomaly_service import get_anomaly_service

router = APIRouter()


class AnomalyRequest(BaseModel):
    data: list[dict]
    value_field: str = "value"


@router.post("/detect")
async def detect_anomalies(
    request: AnomalyRequest,
    x_api_key: str = Header(None),
):
    """
    Generic anomaly detection endpoint
    Works with any numerical data - salaries, sensors, metrics, etc.
    """
    api_key = get_api_key(x_api_key)

    service = get_anomaly_service()

    values = []
    for r in request.data:
        val = r.get(request.value_field)
        if val is not None:
            try:
                values.append(float(val))
            except (TypeError, ValueError):
                pass

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
                    "index": i,
                    "record": request.data[i],
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
            "median": sorted(values)[len(values) // 2] if values else 0,
            "std": service.detect_zscore(values, threshold=0)
            and (
                sum((x - sum(values) / len(values)) ** 2 for x in values) / len(values)
            )
            ** 0.5
            or 0,
            "min": min(values) if values else 0,
            "max": max(values) if values else 0,
        },
    }
