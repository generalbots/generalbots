# Research API 🟡 BETA

> **API for managing research collections, web search, content summarization, and deep research workflows.**

---

## Base URL

```
/api/ui/research
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Endpoints

### List Collections

**`GET /api/ui/research/collections`**

Returns all research collections.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `page` | integer | No | Page number (default: 1) |
| `limit` | integer | No | Items per page (default: 20) |
| `sort` | string | No | Sort: `created_at`, `updated_at`, `name`, `item_count` |
| `order` | string | No | `asc` or `desc` (default: `desc`) |

**Response:**
```json
{
  "collections": [
    {
      "id": "uuid-string",
      "name": "Market Analysis Q1",
      "description": "Research on Q1 market trends and competitor analysis",
      "item_count": 24,
      "source_count": 18,
      "created_at": "2026-01-10T09:00:00Z",
      "updated_at": "2026-01-20T14:30:00Z",
      "tags": ["market", "q1", "competitors"]
    }
  ],
  "total": 8
}
```

---

### Create Collection

**`POST /api/ui/research/collections/new`**

Creates a new research collection.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Collection name |
| `description` | string | No | Collection description |
| `tags` | array | No | Array of tag strings |

**Request Body:**
```json
{
  "name": "AI in Healthcare",
  "description": "Research on artificial intelligence applications in healthcare",
  "tags": ["ai", "healthcare", "medical"]
}
```

**Response:**
```json
{
  "id": "uuid-string",
  "name": "AI in Healthcare",
  "description": "Research on artificial intelligence applications in healthcare",
  "item_count": 0,
  "created_at": "2026-01-20T15:00:00Z",
  "tags": ["ai", "healthcare", "medical"]
}
```

---

### Get Collection

**`GET /api/ui/research/collections/:id`**

Returns a collection with all its items.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string (path) | Yes | Collection ID |

**Response:**
```json
{
  "id": "uuid-string",
  "name": "Market Analysis Q1",
  "description": "Research on Q1 market trends and competitor analysis",
  "items": [
    {
      "id": "item-uuid",
      "type": "webpage",
      "title": "Q1 Market Trends Report",
      "url": "https://example.com/report",
      "summary": "Market grew 12% in Q1 driven by...",
      "source": "web_search",
      "added_at": "2026-01-12T10:00:00Z",
      "tags": ["trends", "growth"]
    },
    {
      "id": "item-uuid-2",
      "type": "note",
      "title": "Key Competitor Analysis",
      "content": "Main competitors include...",
      "source": "manual",
      "added_at": "2026-01-15T14:00:00Z"
    }
  ],
  "item_count": 24,
  "tags": ["market", "q1", "competitors"],
  "created_at": "2026-01-10T09:00:00Z",
  "updated_at": "2026-01-20T14:30:00Z"
}
```

---

### Search

**`POST /api/ui/research/search`**

Search across all research collections and items.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | Yes | Search query |
| `collection_id` | string | No | Limit to specific collection |
| `type` | string | No | Filter: `webpage`, `note`, `document`, `all` |
| `limit` | integer | No | Max results (default: 10) |

**Request Body:**
```json
{
  "query": "artificial intelligence medical diagnosis",
  "type": "webpage",
  "limit": 5
}
```

**Response:**
```json
{
  "results": [
    {
      "id": "item-uuid",
      "collection_id": "collection-uuid",
      "collection_name": "AI in Healthcare",
      "type": "webpage",
      "title": "AI in Medical Diagnosis: A Review",
      "url": "https://example.com/ai-diagnosis",
      "snippet": "Deep learning models have shown remarkable accuracy in medical imaging...",
      "relevance_score": 0.94
    }
  ],
  "total": 12,
  "query": "artificial intelligence medical diagnosis"
}
```

---

### Recent Items

**`GET /api/ui/research/recent`**

Returns recently added research items across all collections.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `limit` | integer | No | Max results (default: 20) |

**Response:**
```json
{
  "recent_items": [
    {
      "id": "item-uuid",
      "collection_id": "collection-uuid",
      "collection_name": "Market Analysis Q1",
      "type": "webpage",
      "title": "Latest Market Trends",
      "url": "https://example.com/trends",
      "summary": "Emerging trends include...",
      "added_at": "2026-01-20T14:30:00Z"
    }
  ],
  "total": 45
}
```

---

### Trending Topics

**`GET /api/ui/research/trending`**

Returns trending research topics based on collection activity and web trends.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `limit` | integer | No | Max results (default: 10) |

**Response:**
```json
{
  "trending": [
    {
      "topic": "Artificial Intelligence",
      "item_count": 156,
      "collection_count": 5,
      "recent_activity": 12,
      "related_topics": ["Machine Learning", "Deep Learning", "NLP"]
    },
    {
      "topic": "Sustainability",
      "item_count": 89,
      "collection_count": 3,
      "recent_activity": 8,
      "related_topics": ["ESG", "Green Energy", "Climate"]
    }
  ]
}
```

---

### Get Prompts

**`GET /api/ui/research/prompts`**

Returns available research prompts and templates.

**Response:**
```json
{
  "prompts": [
    {
      "id": "prompt-uuid",
      "name": "Literature Review",
      "description": "Systematic review of academic papers on a topic",
      "instruction": "Search for peer-reviewed papers, summarize key findings, identify research gaps",
      "category": "academic"
    },
    {
      "id": "prompt-uuid-2",
      "name": "Competitor Analysis",
      "description": "Analyze competitor products, pricing, and market position",
      "instruction": "Research competitor websites, pricing pages, reviews, and recent news",
      "category": "business"
    },
    {
      "id": "prompt-uuid-3",
      "name": "Technology Deep Dive",
      "description": "Comprehensive analysis of a technology or framework",
      "instruction": "Find official docs, tutorials, benchmarks, community feedback, and comparison articles",
      "category": "technical"
    },
    {
      "id": "prompt-uuid-4",
      "name": "Market Research",
      "description": "Market size, trends, and opportunity analysis",
      "instruction": "Search for market reports, industry analysis, growth projections, and key players",
      "category": "business"
    }
  ]
}
```

---

### Web Search

**`POST /api/ui/research/web/search`**

Performs a web search and returns results with summaries.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `query` | string | Yes | Search query |
| `count` | integer | No | Number of results (default: 5) |
| `language` | string | No | Search language (default: `pt`) |
| `safe_search` | boolean | No | Enable safe search (default: true) |
| `collection_id` | string | No | Auto-add results to collection |

**Request Body:**
```json
{
  "query": "artificial intelligence healthcare applications 2026",
  "count": 5,
  "language": "en"
}
```

**Response:**
```json
{
  "results": [
    {
      "title": "AI Transforming Healthcare in 2026",
      "url": "https://example.com/ai-healthcare-2026",
      "snippet": "AI applications in healthcare have expanded to include...",
      "domain": "example.com",
      "published_date": "2026-01-15"
    },
    {
      "title": "Medical AI: Current State and Future Directions",
      "url": "https://example.com/medical-ai",
      "snippet": "Recent advances in deep learning have enabled...",
      "domain": "example.com",
      "published_date": "2026-01-10"
    }
  ],
  "query": "artificial intelligence healthcare applications 2026",
  "total_results": 5,
  "search_time_ms": 340
}
```

---

### Web Summarize

**`POST /api/ui/research/web/summarize`**

Extracts and summarizes content from a web page URL.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `url` | string | Yes | URL to summarize |
| `max_words` | integer | No | Maximum summary length (default: 200) |

**Request Body:**
```json
{
  "url": "https://example.com/ai-healthcare-2026",
  "max_words": 150
}
```

**Response:**
```json
{
  "url": "https://example.com/ai-healthcare-2026",
  "title": "AI Transforming Healthcare in 2026",
  "summary": "AI in healthcare has expanded significantly in 2026, with key applications including diagnostic imaging, drug discovery, and personalized treatment plans. The global market for AI healthcare solutions is projected to reach $45 billion by end of year. Major challenges remain in data privacy, regulatory compliance, and integration with existing clinical workflows.",
  "word_count": 58,
  "extracted_at": "2026-01-20T15:10:00Z"
}
```

---

### Web Deep Research

**`POST /api/ui/research/web/deep`**

Performs deep research on a topic — searches multiple sources, extracts key information, and compiles a comprehensive report.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `topic` | string | Yes | Research topic |
| `depth` | string | No | Research depth: `quick`, `standard`, `deep` (default: `standard`) |
| `sources` | integer | No | Max sources to analyze (default: 10) |
| `collection_id` | string | No | Auto-save results to collection |
| `format` | string | No | Output format: `report`, `bullets`, `timeline` |

**Request Body:**
```json
{
  "topic": "Impact of AI on Brazilian healthcare system",
  "depth": "deep",
  "sources": 15,
  "format": "report"
}
```

**Response:**
```json
{
  "topic": "Impact of AI on Brazilian healthcare system",
  "report": {
    "title": "Impact of AI on Brazilian Healthcare System",
    "executive_summary": "Artificial intelligence is transforming Brazil's healthcare system through improved diagnostics, operational efficiency, and personalized medicine. Key areas include radiology AI, telemedicine integration, and public health prediction models.",
    "sections": [
      {
        "heading": "Current State",
        "content": "Brazil has seen a 340% increase in AI healthcare startups since 2024...",
        "sources": [
          {"url": "https://example.com/source1", "title": "Source Title 1"},
          {"url": "https://example.com/source2", "title": "Source Title 2"}
        ]
      },
      {
        "heading": "Key Applications",
        "content": "Primary applications include diagnostic imaging (42%), drug discovery (28%), and patient monitoring (18%)...",
        "sources": [
          {"url": "https://example.com/source3", "title": "Source Title 3"}
        ]
      },
      {
        "heading": "Challenges",
        "content": "Main challenges include infrastructure gaps, data privacy concerns, and workforce training needs...",
        "sources": [
          {"url": "https://example.com/source4", "title": "Source Title 4"}
        ]
      }
    ],
    "key_findings": [
      "AI adoption in Brazilian hospitals increased from 15% to 38% between 2024-2026",
      "Radiology departments lead AI integration with 67% adoption rate",
      "Estimated R$ 2.3 billion investment in healthcare AI by 2027"
    ],
    "sources_analyzed": 15,
    "research_time_seconds": 28
  },
  "collection_id": "collection-uuid",
  "completed_at": "2026-01-20T15:15:00Z"
}
```

---

### Web History

**`GET /api/ui/research/web/history`**

Returns the user's web search and research history.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `limit` | integer | No | Max results (default: 20) |
| `type` | string | No | Filter: `search`, `summarize`, `deep`, `all` |

**Response:**
```json
{
  "history": [
    {
      "id": "history-uuid",
      "type": "deep",
      "query": "Impact of AI on Brazilian healthcare system",
      "results_count": 15,
      "collection_id": "collection-uuid",
      "executed_at": "2026-01-20T15:15:00Z"
    },
    {
      "id": "history-uuid-2",
      "type": "search",
      "query": "artificial intelligence healthcare applications 2026",
      "results_count": 5,
      "executed_at": "2026-01-20T14:30:00Z"
    },
    {
      "id": "history-uuid-3",
      "type": "summarize",
      "query": "https://example.com/ai-healthcare-2026",
      "word_count": 58,
      "executed_at": "2026-01-20T14:00:00Z"
    }
  ],
  "total": 45
}
```

---

### Instant Research

**`GET /api/ui/research/web/instant`**

Provides instant search suggestions as the user types.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `q` | string | Yes | Partial query string |

**Response:**
```json
{
  "suggestions": [
    "artificial intelligence healthcare",
    "artificial intelligence in medicine",
    "artificial intelligence diagnosis",
    "artificial intelligence drug discovery",
    "artificial intelligence patient care"
  ],
  "related_topics": [
    "Machine Learning in Healthcare",
    "Deep Learning Medical Imaging",
    "AI Ethics in Medicine"
  ]
}
```

---

## Research Workflow Example

```bash
# 1. Create a collection
curl -X POST /api/ui/research/collections/new \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"name": "AI Research", "tags": ["ai", "tech"]}'

# 2. Search the web
curl -X POST /api/ui/research/web/search \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"query": "latest AI developments 2026", "count": 10}'

# 3. Summarize a promising result
curl -X POST /api/ui/research/web/summarize \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"url": "https://example.com/ai-2026", "max_words": 200}'

# 4. Run deep research on the topic
curl -X POST /api/ui/research/web/deep \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"topic": "AI developments 2026", "depth": "deep", "collection_id": "uuid"}'

# 5. Review collection
curl /api/ui/research/collections/uuid \
  -H "Authorization: Bearer $TOKEN"
```

---

## RAG-Powered Search with Real KB Data

The research app now runs **real RAG** against the bot's knowledge base
(`kb_documents`/`kb_collections`) and synthesizes answers with the configured LLM.
Search results are scoped to the active bot (`bots.is_default_for_branch`).

### Search (form-encoded)

```http
POST /api/ui/research/search
Content-Type: application/x-www-form-urlencoded

query=How does billing work&collection=my-collection
```

When an LLM provider is configured the response contains an `#answer-content`
block (synthesized answer with citations) followed by the matching documents.

### Source Counts & Sources

```http
GET /api/ui/research/source-counts      # { all, web, docs, kb }
GET /api/ui/research/sources?category=  # { sources: [...] }
```

### Collections

```http
GET  /api/ui/research/collections
POST /api/ui/research/collections/new   # { name, description }
GET  /api/ui/research/collections/:id
POST /api/ui/research/collections/save  # save prompt to a collection
```

### Export & Paper Bridge

```http
GET  /api/ui/research/export/citations?q=<query>   # BibTeX via clipboard
POST /api/ui/paper/import                            # { title, content } -> Paper
```

Recent searches and trending tags are real aggregates over `research_searches`;
search history reads real rows instead of placeholders.

---

## See Also

- [Docs API](docs-api.md) - Word processor documents
- [Paper API](paper-api.md) - Lightweight notes
- [Slides API](slides-api.md) - Presentation creation
- [Video API](video-api.md) - Video editing
