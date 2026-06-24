"""
Anomaly Detection Service - Detecção de desvios/anomalias em dados tabulares
Compatible with salary data, sensor readings, and other numerical time series
"""

import numpy as np
from typing import Optional
from dataclasses import dataclass
from ..core.logging import get_logger

logger = get_logger("anomaly_service")


@dataclass
class AnomalyResult:
    is_anomaly: bool
    score: float
    method: str
    threshold: float
    details: dict


class AnomalyDetectionService:
    def __init__(self):
        self.initialized = True

    def detect_zscore(
        self, data: list[float], threshold: float = 3.0
    ) -> list[AnomalyResult]:
        """
        Z-Score based anomaly detection
        Identifies values that are more than N standard deviations from mean
        """
        if len(data) < 3:
            return []

        arr = np.array(data)
        mean = np.mean(arr)
        std = np.std(arr)

        if std == 0:
            return []

        z_scores = np.abs((arr - mean) / std)
        results = []

        for i, z in enumerate(z_scores):
            is_anomaly = z > threshold
            results.append(
                AnomalyResult(
                    is_anomaly=is_anomaly,
                    score=float(z),
                    method="zscore",
                    threshold=threshold,
                    details={
                        "index": i,
                        "value": float(arr[i]),
                        "mean": float(mean),
                        "std": float(std),
                        "deviation": float(arr[i] - mean),
                    },
                )
            )

        return results

    def detect_iqr(
        self, data: list[float], multiplier: float = 1.5
    ) -> list[AnomalyResult]:
        """
        IQR (Interquartile Range) based anomaly detection
        Identifies values outside Q1 - 1.5*IQR and Q3 + 1.5*IQR
        """
        if len(data) < 4:
            return []

        arr = np.array(data)
        q1 = np.percentile(arr, 25)
        q3 = np.percentile(arr, 75)
        iqr = q3 - q1

        lower_bound = q1 - multiplier * iqr
        upper_bound = q3 + multiplier * iqr

        results = []
        for i, val in enumerate(arr):
            is_anomaly = val < lower_bound or val > upper_bound
            distance = 0
            if val < lower_bound:
                distance = lower_bound - val
            elif val > upper_bound:
                distance = val - upper_bound

            results.append(
                AnomalyResult(
                    is_anomaly=is_anomaly,
                    score=float(distance),
                    method="iqr",
                    threshold=multiplier,
                    details={
                        "index": i,
                        "value": float(val),
                        "q1": float(q1),
                        "q3": float(q3),
                        "iqr": float(iqr),
                        "lower_bound": float(lower_bound),
                        "upper_bound": float(upper_bound),
                    },
                )
            )

        return results

    def detect_isolation_forest(
        self, data: list[float], contamination: float = 0.1
    ) -> list[AnomalyResult]:
        """
        Simplified Isolation Forest-like detection using average absolute deviation
        More robust to outliers than Z-score
        """
        if len(data) < 3:
            return []

        arr = np.array(data)
        median = np.median(arr)
        mad = np.median(np.abs(arr - median))

        if mad == 0:
            return []

        modified_z = np.abs(0.6745 * (arr - median) / mad)
        threshold = 3.5

        results = []
        for i, z in enumerate(modified_z):
            is_anomaly = z > threshold
            results.append(
                AnomalyResult(
                    is_anomaly=is_anomaly,
                    score=float(z),
                    method="isolation_forest",
                    threshold=threshold,
                    details={
                        "index": i,
                        "value": float(arr[i]),
                        "median": float(median),
                        "mad": float(mad),
                    },
                )
            )

        return results

    def detect_salary_anomalies(
        self, records: list[dict], value_field: str = "salarioBase"
    ) -> dict:
        """
        Specialized detection for salary/payroll data
        Combines multiple methods for robust anomaly detection
        """
        values = [float(r.get(value_field, 0)) for r in records if value_field in r]

        if not values:
            return {"error": f"Field '{value_field}' not found in records"}

        zscore_results = self.detect_zscore(values, threshold=2.5)
        iqr_results = self.detect_iqr(values, multiplier=1.5)
        iso_results = self.detect_isolation_forest(values)

        anomalies = []
        for i in range(len(values)):
            votes = sum(
                [
                    zscore_results[i].is_anomaly if zscore_results else False,
                    iqr_results[i].is_anomaly if iqr_results else False,
                    iso_results[i].is_anomaly if iso_results else False,
                ]
            )

            if votes >= 2:
                anomalies.append(
                    {
                        "index": i,
                        "record": records[i],
                        "value": values[i],
                        "confidence": votes / 3,
                        "methods": {
                            "zscore": zscore_results[i].is_anomaly
                            if zscore_results
                            else False,
                            "iqr": iqr_results[i].is_anomaly if iqr_results else False,
                            "isolation": iso_results[i].is_anomaly
                            if iso_results
                            else False,
                        },
                        "zscore_details": zscore_results[i].details
                        if zscore_results
                        else {},
                    }
                )

        return {
            "total_records": len(records),
            "anomalies_found": len(anomalies),
            "anomaly_rate": len(anomalies) / len(records) if records else 0,
            "anomalies": anomalies,
            "summary": {
                "mean": float(np.mean(values)),
                "median": float(np.median(values)),
                "std": float(np.std(values)),
                "min": float(np.min(values)),
                "max": float(np.max(values)),
            },
        }

    def detect_sensor_anomalies(
        self, readings: list[dict], value_field: str = "value"
    ) -> dict:
        """
        Detection for sensor/IoT data with time-series characteristics
        """
        values = [float(r.get(value_field, 0)) for r in readings if value_field in r]

        if not values:
            return {"error": f"Field '{value_field}' not found in readings"}

        arr = np.array(values)
        diff = np.diff(arr)
        mean_diff = np.mean(np.abs(diff))
        std_diff = np.std(diff)

        anomalies = []
        for i in range(1, len(values)):
            change = abs(values[i] - values[i - 1])
            z_change = (change - mean_diff) / std_diff if std_diff > 0 else 0

            if z_change > 2.5 or change > 3 * mean_diff:
                anomalies.append(
                    {
                        "index": i,
                        "record": readings[i],
                        "previous_value": values[i - 1],
                        "current_value": values[i],
                        "change": change,
                        "change_zscore": float(z_change),
                        "type": "sudden_change",
                    }
                )

        return {
            "total_readings": len(readings),
            "anomalies_found": len(anomalies),
            "anomalies": anomalies,
            "baseline": {
                "mean_change": float(mean_diff),
                "std_change": float(std_diff),
            },
        }


_service: Optional[AnomalyDetectionService] = None


def get_anomaly_service() -> AnomalyDetectionService:
    global _service
    if _service is None:
        _service = AnomalyDetectionService()
    return _service
