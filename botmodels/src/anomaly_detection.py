#!/usr/bin/env python3
"""
Anomaly Detection Service for BotModels
Detects outliers in payroll data using statistical methods
"""

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
from typing import List, Dict, Any, Optional
import numpy as np
from scipy import stats
import uvicorn

app = FastAPI(title="BotModels Anomaly Detection", version="1.0.0")


class DetectRequest(BaseModel):
    data: List[Dict[str, Any]]
    value_field: str


class AnomalyResult(BaseModel):
    anomalies_found: int
    total_records: int
    anomaly_percentage: float
    anomalies: List[Dict[str, Any]]
    statistics: Dict[str, Any]


@app.get("/health")
def health():
    return {"status": "healthy", "service": "anomaly-detection"}


@app.post("/api/detect", response_model=AnomalyResult)
def detect_anomalies(request: DetectRequest):
    if not request.data:
        raise HTTPException(status_code=400, detail="No data provided")

    if not request.value_field:
        raise HTTPException(status_code=400, detail="value_field not specified")

    # Extract numeric values
    values = []
    valid_indices = []

    for i, record in enumerate(request.data):
        value = record.get(request.value_field)
        if value is not None:
            try:
                numeric_value = float(value)
                values.append(numeric_value)
                valid_indices.append(i)
            except (ValueError, TypeError):
                pass

    if not values:
        return AnomalyResult(
            anomalies_found=0,
            total_records=len(request.data),
            anomaly_percentage=0.0,
            anomalies=[],
            statistics={},
        )

    values_array = np.array(values)

    # Calculate statistics
    mean = np.mean(values_array)
    std = np.std(values_array)
    q1 = np.percentile(values_array, 25)
    q3 = np.percentile(values_array, 75)
    iqr = q3 - q1

    # Z-score method (|z| > 3)
    z_scores = np.abs(stats.zscore(values_array))
    z_outliers = z_scores > 3

    # IQR method (outside 1.5 * IQR)
    iqr_lower = q1 - 1.5 * iqr
    iqr_upper = q3 + 1.5 * iqr
    iqr_outliers = (values_array < iqr_lower) | (values_array > iqr_upper)

    # Combined outlier detection
    outliers = z_outliers | iqr_outliers

    # Build anomaly list
    anomalies = []
    for i, is_outlier in enumerate(outliers):
        if is_outlier:
            idx = valid_indices[i]
            record = request.data[idx].copy()
            record["_anomaly_score"] = float(z_scores[i])
            record["_detection_method"] = "z_score" if z_outliers[i] else "iqr"
            anomalies.append(record)

    return AnomalyResult(
        anomalies_found=len(anomalies),
        total_records=len(request.data),
        anomaly_percentage=(len(anomalies) / len(request.data)) * 100,
        anomalies=anomalies,
        statistics={
            "mean": float(mean),
            "std": float(std),
            "min": float(np.min(values_array)),
            "max": float(np.max(values_array)),
            "median": float(np.median(values_array)),
            "q1": float(q1),
            "q3": float(q3),
        },
    )


if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=8082)
