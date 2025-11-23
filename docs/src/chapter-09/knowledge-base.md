# Knowledge Base

BotServer's knowledge base system enables semantic search and retrieval of documents, providing context for intelligent bot responses.

## Overview

The knowledge base system provides:
- Document storage and indexing
- Vector embeddings for semantic search
- Integration with Qdrant vector database
- Multi-collection management
- Context retrieval for LLM augmentation

## Architecture

### Storage Layers

1. **MinIO/S3 Storage** - Raw document files
2. **PostgreSQL** - Document metadata and references
3. **Qdrant** - Vector embeddings for semantic search

### Database Schema

#### kb_collections Table

Stores knowledge base collections:
- Collection ID and name
- Bot association
- Description and metadata
- Creation/update timestamps

#### kb_documents Table

Individual documents within collections:
- Document ID
- Collection reference
- Content (text)
- Metadata (JSON)
- Embedding ID (Qdrant reference)
- Indexed flag

#### user_kb_associations Table

Links sessions to active knowledge bases:
- Session ID
- Collection ID
- Activation timestamp

## Document Organization

### Collection Structure

```
bot-name.gbai/
└── bot-name.gbkb/
    ├── policies/
    │   ├── vacation-policy.pdf
    │   └── code-of-conduct.pdf
    ├── procedures/
    │   └── onboarding.docx
    └── faqs/
        └── common-questions.txt
```

### Supported Formats

- PDF documents
- Text files (.txt, .md)
- Word documents (.docx)
- CSV files
- JSON files
- HTML documents

## BASIC Keywords

### USE_KB

Activate a knowledge base collection:

```basic
USE_KB "policies"
USE_KB "procedures"

# Now LLM has access to these collections
let answer = LLM "What's the vacation policy?"
```

### CLEAR_KB

Remove knowledge bases from context:

```basic
CLEAR_KB "policies"  # Remove specific
CLEAR_KB             # Clear all
```

### ADD_KB (Implicit)

Documents in .gbkb folders are automatically:
- Uploaded to MinIO
- Indexed into Qdrant
- Available for USE_KB

## Indexing Process

### Document Processing Pipeline

1. **Upload**: Files uploaded to MinIO bucket
2. **Extraction**: Text extracted from documents
3. **Chunking**: Documents split into segments
4. **Embedding**: Generate vector embeddings
5. **Storage**: Store in Qdrant with metadata
6. **Registration**: Update PostgreSQL references

### Chunking Strategy

Documents are chunked for optimal retrieval:
- Chunk size: ~500 tokens
- Overlap: 50 tokens
- Maintains context boundaries
- Preserves paragraph integrity

### Embedding Generation

Using OpenAI or local models:
- Model: text-embedding-ada-002 (OpenAI)
- Dimension: 1536 (OpenAI) or varies (local)
- Batch processing for efficiency

## Qdrant Integration

### Vector Storage

Embeddings stored in Qdrant:
- Collection per bot
- Metadata includes:
  - Document ID
  - Chunk index
  - Source file
  - Creation date

### Semantic Search

Query process:
1. Convert query to embedding
2. Search Qdrant for similar vectors
3. Retrieve top-k results
4. Return document chunks
5. Include in LLM context

### Search Parameters

- Top-k results: 5 (default)
- Similarity threshold: 0.7
- Distance metric: Cosine similarity

## Context Management

### Context Assembly

When answering questions:
1. Retrieve relevant documents
2. Order by relevance score
3. Truncate to context limit
4. Prepend to user query
5. Send to LLM

### Context Limits

Managing token constraints:
- Maximum context: 4000 tokens
- Document chunks: 500 tokens each
- Reserve tokens for conversation
- Automatic truncation

## Usage Examples

### Customer Support Bot

```basic
# Load knowledge bases
USE_KB "product-docs"
USE_KB "support-faqs"
USE_KB "troubleshooting"

# Answer customer question
let question = HEAR
let answer = LLM "Answer based on our documentation: " + question
TALK answer
```

### Policy Assistant

```basic
# Load company policies
USE_KB "hr-policies"
USE_KB "compliance"

# Interactive Q&A
TALK "I can answer questions about company policies."
let query = HEAR
let response = LLM "Based on company policy: " + query
TALK response
```

### Research Assistant

```basic
# Load research papers
USE_KB "research-papers"
USE_KB "citations"

# Summarize findings
let topic = HEAR
let summary = LLM "Summarize research on: " + topic
TALK summary
```

## Performance Optimization

### Caching

- Embedding cache for repeated queries
- Document cache in Redis
- Search result caching
- Metadata caching

### Batch Operations

- Bulk document upload
- Batch embedding generation
- Parallel indexing
- Async processing

### Index Management

- Periodic reindexing
- Orphan cleanup
- Duplicate detection
- Version control

## Monitoring

### Metrics

- Documents indexed
- Search queries per second
- Average retrieval time
- Cache hit rate
- Storage usage

### Health Checks

- Qdrant connectivity
- Index consistency
- Document accessibility
- Embedding quality

## Best Practices

1. **Organize Collections**: Group related documents
2. **Quality Content**: Ensure documents are accurate
3. **Regular Updates**: Keep knowledge current
4. **Monitor Usage**: Track which documents are accessed
5. **Optimize Chunks**: Tune chunk size for your content
6. **Cache Effectively**: Cache frequently accessed documents
7. **Clean Data**: Remove outdated information

## Limitations

- Document size limits (varies by format)
- Indexing time for large collections
- Context window constraints
- Language support (primarily English)
- No real-time document updates

## Troubleshooting

### Common Issues

1. **Documents Not Found**
   - Check indexing completed
   - Verify collection name
   - Ensure USE_KB called

2. **Poor Search Results**
   - Review document quality
   - Adjust chunk size
   - Check embedding model

3. **Slow Retrieval**
   - Optimize Qdrant queries
   - Increase cache size
   - Reduce result count

4. **Context Too Long**
   - Reduce top-k results
   - Smaller chunk size
   - Prioritize relevance

## Configuration

### Environment Variables

```bash
# Qdrant Configuration
QDRANT_URL=http://localhost:6333
QDRANT_API_KEY=optional-api-key

# Embedding Configuration
EMBEDDING_MODEL=text-embedding-ada-002
EMBEDDING_DIMENSION=1536

# Search Configuration
SEARCH_TOP_K=5
SEARCH_THRESHOLD=0.7
```

## Future Enhancements

Planned improvements:
- Multi-modal search (images)
- Real-time indexing
- Advanced chunking strategies
- Cross-lingual support
- Document versioning
- Incremental updates
- Custom embedding models

## Summary

The knowledge base system is fundamental to BotServer's intelligence, enabling bots to access and retrieve relevant information from document collections. Through integration with Qdrant and semantic search, bots can provide accurate, context-aware responses based on organizational knowledge.