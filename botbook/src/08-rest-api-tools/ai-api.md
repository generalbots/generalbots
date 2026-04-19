# AI API

The AI API provides endpoints for managing AI models, inference, training, and advanced AI operations.

## Status: Roadmap

This API is on the development roadmap. The endpoints documented below represent the planned interface design.

## Base URL

```
http://localhost:9000/api/v1/ai
```

## Authentication

Uses the standard botserver authentication mechanism with appropriate role-based permissions.

## Endpoints

### Model Management

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/ai/models` | List available models |
| GET | `/api/v1/ai/models/{model_id}` | Get model details |
| POST | `/api/v1/ai/models/deploy` | Deploy a new model |
| DELETE | `/api/v1/ai/models/{model_id}` | Remove a model |

### Inference

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/ai/inference` | Run inference on input data |
| POST | `/api/v1/ai/chat/completions` | Chat completion endpoint |
| POST | `/api/v1/ai/embeddings` | Generate embeddings |

### Training

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | `/api/v1/ai/training/start` | Start a training job |
| GET | `/api/v1/ai/training/{job_id}/status` | Get training job status |
| POST | `/api/v1/ai/training/{job_id}/cancel` | Cancel training job |

### Model Configuration

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/api/v1/ai/models/{model_id}/config` | Get model configuration |
| PUT | `/api/v1/ai/models/{model_id}/config` | Update model configuration |

## Request Examples

### List Available Models

```bas
models = GET "/api/v1/ai/models"
FOR EACH model IN models
    TALK model.name + " - " + model.status
NEXT
```

### Chat Completion

```bas
request = NEW OBJECT
request.model = "gpt-4"
request.messages = NEW ARRAY
request.messages.ADD({"role": "user", "content": "Hello, how are you?"})

response = POST "/api/v1/ai/chat/completions", request
TALK response.choices[0].message.content
```

### Generate Embeddings

```bas
request = NEW OBJECT
request.input = "Convert this text to embeddings"
request.model = "text-embedding-3-small"

result = POST "/api/v1/ai/embeddings", request
embedding = result.data[0].embedding
```

### Start Training Job

```bas
training_config = NEW OBJECT
training_config.base_model = "llama-2-7b"
training_config.dataset = "my-training-data"
training_config.epochs = 3

job = POST "/api/v1/ai/training/start", training_config
TALK "Training job started: " + job.id
```

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Created |
| 202 | Accepted (for async operations) |
| 400 | Bad Request |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Model or resource not found |
| 429 | Rate limit exceeded |
| 500 | Internal Server Error |

## Required Permissions

| Endpoint Category | Required Role |
|-------------------|---------------|
| Model Management | `admin` or `model_manager` |
| Inference | `user` or higher |
| Training | `admin` or `trainer` |
| Model Configuration | `admin` |