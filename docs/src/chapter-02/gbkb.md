# .gbkb Knowledge Base

The `.gbkb` package manages knowledge base collections that provide contextual information to the bot during conversations.

## What is .gbkb?

`.gbkb` (General Bot Knowledge Base) collections store:
- Document collections for semantic search
- Vector embeddings for similarity matching
- Metadata and indexing information
- Access control and organization

## Knowledge Base Structure

Each `.gbkb` collection is organized as:

```
collection-name.gbkb/
├── documents/
│   ├── doc1.pdf
│   ├── doc2.txt
│   └── doc3.html
├── embeddings/          # Auto-generated
├── metadata.json       # Collection info
└── index.json         # Search indexes
```

## Supported Formats

The knowledge base can process:
- **Text files**: .txt, .md, .html
- **Documents**: .pdf, .docx
- **Web content**: URLs and web pages
- **Structured data**: .csv, .json

## Vector Embeddings

Each document is processed into vector embeddings using:
- BGE-small-en-v1.5 model (default)
- Chunking for large documents
- Metadata extraction and indexing
- Semantic similarity scoring

## Collection Management

### Creating Collections
```basic
USE KB "company-policies"
ADD WEBSITE "https://company.com/docs"
```

### Using Collections
```basic
USE KB "company-policies"
LLM "What is the vacation policy?"
```

### Multiple Collections
```basic
USE KB "policies"
USE KB "procedures"
USE KB "faqs"
REM All active collections contribute to context
```

## Semantic Search

The knowledge base provides:
- **Similarity search**: Find relevant documents
- **Hybrid search**: Combine semantic and keyword
- **Context injection**: Automatically add to LLM prompts
- **Relevance scoring**: Filter by similarity threshold

## Integration with Dialogs

Knowledge bases are automatically used when:
- `USE KB` is called
- Answer mode is set to use documents
- LLM queries benefit from contextual information
