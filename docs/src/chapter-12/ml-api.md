# Machine Learning API

BotServer provides RESTful endpoints for machine learning operations including embeddings, semantic search, and model management.

## Overview

The ML API enables:
- Text embeddings generation
- Semantic similarity search
- Document vectorization
- Model configuration
- Inference endpoints

## Base URL

```
http://localhost:8080/api/v1/ml
```

## Authentication

All ML API requests require authentication:

```http
Authorization: Bearer <token>
```

## Endpoints

### Generate Embeddings

**POST** `/embeddings`

Generate vector embeddings for text input.

**Request Body:**
```json
{
  "text": "Text to embed",
  "model": "bge-small-en-v1.5",
  "dimensions": 384
}
```

**Response:**
```json
{
  "embeddings": [0.123, -0.456, 0.789, ...],
  "model": "bge-small-en-v1.5",
  "dimensions": 384,
  "tokens": 12
}
```

### Batch Embeddings

**POST** `/embeddings/batch`

Generate embeddings for multiple texts.

**Request Body:**
```json
{
  "texts": [
    "First text",
    "Second text",
    "Third text"
  ],
  "model": "bge-small-en-v1.5"
}
```

**Response:**
```json
{
  "embeddings": [
    [0.123, -0.456, ...],
    [0.234, -0.567, ...],
    [0.345, -0.678, ...]
  ],
  "count": 3,
  "model": "bge-small-en-v1.5"
}
```

### Semantic Search

**POST** `/search`

Search for semantically similar documents.

**Request Body:**
```json
{
  "query": "How to reset password",
  "collection": "knowledge_base",
  "limit": 10,
  "threshold": 0.7
}
```

**Response:**
```json
{
  "results": [
    {
      "id": "doc_123",
      "text": "To reset your password, click on...",
      "score": 0.95,
      "metadata": {
        "source": "manual.pdf",
        "page": 42
      }
    }
  ],
  "query": "How to reset password",
  "count": 5,
  "processing_time_ms": 23
}
```

### Text Completion

**POST** `/completions`

Generate text completions using LLM.

**Request Body:**
```json
{
  "prompt": "Write a welcome message for",
  "max_tokens": 100,
  "temperature": 0.7,
  "model": "local-llm"
}
```

**Response:**
```json
{
  "completion": "Write a welcome message for our valued customers. We're delighted to have you here!",
  "tokens_used": 15,
  "model": "local-llm",
  "finish_reason": "stop"
}
```

### Chat Completion

**POST** `/chat/completions`

Generate chat responses with context.

**Request Body:**
```json
{
  "messages": [
    {"role": "system", "content": "You are a helpful assistant"},
    {"role": "user", "content": "What's the weather like?"}
  ],
  "model": "local-llm",
  "temperature": 0.7,
  "max_tokens": 150
}
```

**Response:**
```json
{
  "choices": [{
    "message": {
      "role": "assistant",
      "content": "I don't have access to real-time weather data..."
    },
    "finish_reason": "stop"
  }],
  "usage": {
    "prompt_tokens": 20,
    "completion_tokens": 30,
    "total_tokens": 50
  }
}
```

### Document Processing

**POST** `/documents/process`

Extract and vectorize document content.

**Request Body:**
```json
{
  "document_url": "s3://bucket/document.pdf",
  "chunk_size": 512,
  "overlap": 50,
  "extract_metadata": true
}
```

**Response:**
```json
{
  "document_id": "doc_456",
  "chunks": 42,
  "metadata": {
    "title": "User Manual",
    "author": "Documentation Team",
    "pages": 120
  },
  "status": "processed"
}
```

### Similarity Comparison

**POST** `/similarity`

Compare similarity between two texts.

**Request Body:**
```json
{
  "text1": "How to reset password",
  "text2": "Password reset instructions",
  "method": "cosine"
}
```

**Response:**
```json
{
  "similarity": 0.89,
  "method": "cosine",
  "normalized": true
}
```

### Model Information

**GET** `/models`

List available models.

**Response:**
```json
{
  "models": [
    {
      "id": "bge-small-en-v1.5",
      "type": "embedding",
      "dimensions": 384,
      "max_tokens": 512,
      "status": "ready"
    },
    {
      "id": "local-llm",
      "type": "completion",
      "max_context": 4096,
      "status": "ready"
    }
  ]
}
```

### Health Check

**GET** `/health`

Check ML service status.

**Response:**
```json
{
  "status": "healthy",
  "embedding_service": "ready",
  "llm_service": "ready",
  "vector_db": "connected",
  "models_loaded": 2
}
```

## Vector Collections

### Create Collection

**POST** `/collections`

Create a new vector collection.

**Request Body:**
```json
{
  "name": "product_catalog",
  "dimensions": 384,
  "distance_metric": "cosine",
  "index_type": "hnsw"
}
```

### Add Vectors

**POST** `/collections/{name}/vectors`

Add vectors to a collection.

**Request Body:**
```json
{
  "vectors": [
    {
      "id": "item_1",
      "vector": [0.1, 0.2, ...],
      "metadata": {"name": "Product A"}
    }
  ]
}
```

### Query Collection

**POST** `/collections/{name}/query`

Query vectors in a collection.

**Request Body:**
```json
{
  "vector": [0.1, 0.2, ...],
  "top_k": 5,
  "filter": {
    "category": "electronics"
  }
}
```

## Error Responses

### 400 Bad Request
```json
{
  "error": "invalid_input",
  "message": "Text exceeds maximum token limit",
  "max_tokens": 512,
  "provided_tokens": 1024
}
```

### 503 Service Unavailable
```json
{
  "error": "model_not_ready",
  "message": "Model is still loading",
  "model": "local-llm",
  "retry_after": 30
}
```

## Usage Examples

### Generate Embedding with cURL

```bash
curl -X POST \
  -H "Authorization: Bearer token123" \
  -H "Content-Type: application/json" \
  -d '{"text": "Hello world", "model": "bge-small-en-v1.5"}' \
  http://localhost:8080/api/v1/ml/embeddings
```

### Semantic Search

```bash
curl -X POST \
  -H "Authorization: Bearer token123" \
  -H "Content-Type: application/json" \
  -d '{
    "query": "password reset",
    "collection": "docs",
    "limit": 5
  }' \
  http://localhost:8080/api/v1/ml/search
```

## Performance Considerations

1. **Batch Processing**: Use batch endpoints for multiple items
2. **Caching**: Embeddings are cached for repeated texts
3. **Model Warmup**: First request may be slower
4. **Context Limits**: Respect model token limits
5. **Async Processing**: Use webhooks for long operations

## Rate Limits

| Endpoint | Limit | Window |
|----------|-------|--------|
| Embeddings | 100 req/min | Per user |
| Completions | 20 req/min | Per user |
| Search | 200 req/min | Per user |
| Document Processing | 10 req/min | Per user |

## Model Configuration

Models are configured in `config.csv`:

```csv
name,value
llm-model,path/to/model.gguf
llm-url,http://localhost:8081
embedding-model,path/to/embedding.gguf
embedding-url,http://localhost:8082
```

## Related APIs

- [Storage API](./storage-api.md) - Document storage
- [Knowledge Base API](../chapter-03/kb-and-tools.md) - KB management
- [Search API](../chapter-03/semantic-search.md) - Advanced search