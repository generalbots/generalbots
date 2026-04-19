# ML API

The ML API provides endpoints for machine learning operations, model training, and predictive analytics.

## Status: Roadmap

This API is on the development roadmap. The endpoints documented below represent the planned interface design.

## Base URL

```
http://localhost:9000/api/v1/ml
```

## Authentication

Uses the standard botserver authentication mechanism with appropriate role-based permissions.

## Endpoints

### Dataset Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/ml/datasets` | Create a new dataset |
| GET | `/api/v1/ml/datasets` | List all datasets |
| GET | `/api/v1/ml/datasets/{dataset_id}` | Get dataset details |
| PUT | `/api/v1/ml/datasets/{dataset_id}` | Update dataset |
| DELETE | `/api/v1/ml/datasets/{dataset_id}` | Delete dataset |

### Model Training

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/ml/train` | Start a training job |
| GET | `/api/v1/ml/jobs` | List all training jobs |
| GET | `/api/v1/ml/jobs/{job_id}` | Get job details |
| POST | `/api/v1/ml/jobs/{job_id}/stop` | Stop a training job |

### Predictions

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/ml/predict` | Make a single prediction |
| POST | `/api/v1/ml/batch-predict` | Make batch predictions |
| GET | `/api/v1/ml/predictions/{prediction_id}` | Get prediction result |

### Model Evaluation

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/ml/models/{model_id}/metrics` | Get model metrics |
| POST | `/api/v1/ml/models/{model_id}/evaluate` | Evaluate model on test data |
| GET | `/api/v1/ml/models/{model_id}/confusion-matrix` | Get confusion matrix |

### Feature Engineering

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/ml/features/extract` | Extract features from data |
| GET | `/api/v1/ml/features/importance` | Get feature importance scores |
| POST | `/api/v1/ml/features/transform` | Transform features |

## Request Examples

### Create Dataset

```bas
dataset = NEW OBJECT
dataset.name = "customer_churn"
dataset.source = "customers_table"
dataset.target_column = "churned"

result = POST "/api/v1/ml/datasets", dataset
TALK "Dataset created: " + result.id
```

### Start Training Job

```bas
training_config = NEW OBJECT
training_config.dataset_id = "ds-123"
training_config.model_type = "classification"
training_config.algorithm = "random_forest"
training_config.parameters = {"n_estimators": 100, "max_depth": 10}

job = POST "/api/v1/ml/train", training_config
TALK "Training job started: " + job.job_id
```

### Make Prediction

```bas
input = NEW OBJECT
input.model_id = "model-456"
input.features = {"age": 35, "tenure": 24, "monthly_charges": 75.50}

result = POST "/api/v1/ml/predict", input
TALK "Prediction: " + result.prediction
TALK "Confidence: " + result.confidence
```

### Batch Predictions

```bas
batch_input = NEW OBJECT
batch_input.model_id = "model-456"
batch_input.data = [
    {"age": 25, "tenure": 12, "monthly_charges": 50.00},
    {"age": 45, "tenure": 36, "monthly_charges": 95.00},
    {"age": 30, "tenure": 6, "monthly_charges": 65.00}
]

results = POST "/api/v1/ml/batch-predict", batch_input
FOR EACH result IN results.predictions
    TALK result.id + ": " + result.prediction
NEXT
```

### Get Model Metrics

```bas
metrics = GET "/api/v1/ml/models/model-456/metrics"
TALK "Accuracy: " + metrics.accuracy
TALK "Precision: " + metrics.precision
TALK "Recall: " + metrics.recall
TALK "F1 Score: " + metrics.f1_score
```

## Supported Algorithms

### Classification

| Algorithm | Description |
|-----------|-------------|
| `random_forest` | Random Forest Classifier |
| `gradient_boosting` | Gradient Boosting Classifier |
| `logistic_regression` | Logistic Regression |
| `svm` | Support Vector Machine |
| `neural_network` | Neural Network Classifier |

### Regression

| Algorithm | Description |
|-----------|-------------|
| `linear_regression` | Linear Regression |
| `random_forest_regressor` | Random Forest Regressor |
| `gradient_boosting_regressor` | Gradient Boosting Regressor |
| `neural_network_regressor` | Neural Network Regressor |

### Clustering

| Algorithm | Description |
|-----------|-------------|
| `kmeans` | K-Means Clustering |
| `dbscan` | DBSCAN Clustering |
| `hierarchical` | Hierarchical Clustering |

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Created |
| 202 | Accepted (async job started) |
| 400 | Bad Request (invalid parameters) |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Model or dataset not found |
| 422 | Unprocessable Entity (invalid data format) |
| 500 | Internal Server Error |

## Job Status Values

| Status | Description |
|--------|-------------|
| `queued` | Job is waiting to start |
| `running` | Job is in progress |
| `completed` | Job finished successfully |
| `failed` | Job encountered an error |
| `cancelled` | Job was manually stopped |

## Required Permissions

| Endpoint Category | Required Role |
|-------------------|---------------|
| Dataset Management | `ml_user` or higher |
| Model Training | `ml_trainer` or `admin` |
| Predictions | `ml_user` or higher |
| Model Evaluation | `ml_user` or higher |
| Feature Engineering | `ml_trainer` or `admin` |