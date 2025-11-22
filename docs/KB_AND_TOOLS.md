# KB and TOOL System Documentation

## Overview

The General Bots system provides **4 essential keywords** for managing Knowledge Bases (KB) and Tools dynamically during conversation sessions:

1. **USE_KB** - Load and embed files from `.gbkb` folders into vector database
2. **CLEAR_KB** - Remove KB from current session
3. **USE_TOOL** - Make a tool available for LLM to call
4. **CLEAR_TOOLS** - Remove all tools from current session

---

## Knowledge Base (KB) System

### What is a KB?

A Knowledge Base (KB) is a **folder containing documents** (`.gbkb` folder structure) that are **vectorized/embedded and stored in a vector database**. The vectorDB retrieves relevant chunks/excerpts to inject into prompts, giving the LLM context-aware responses.

### Folder Structure

```
work/
  {bot_name}/
    {bot_name}.gbkb/          # Knowledge Base root
      circular/               # KB folder 1
        document1.pdf
        document2.md
        document3.txt
      comunicado/             # KB folder 2
        info.docx
        data.csv
      docs/                   # KB folder 3
        README.md
        guide.pdf
```

### KB Loading Process

1. **Scan folder** - System scans `.gbkb` folder for documents
2. **Process files** - Extracts text from PDF, DOCX, TXT, MD, CSV files
3. **Chunk text** - Splits into ~1000 character chunks with overlap
4. **Generate embeddings** - Creates vector representations
5. **Store in VectorDB** - Saves to Qdrant for similarity search
6. **Ready for queries** - KB available for semantic search

### Supported File Types

- **PDF** - Full text extraction with pdf-extract
- **DOCX/DOC** - Microsoft Word documents
- **TXT** - Plain text files
- **MD** - Markdown documents
- **CSV** - Structured data (each row as entry)
- **HTML** - Web pages (text only)
- **JSON** - Structured data

### USE_KB Keyword

```basic
USE_KB "circular"
# Loads the 'circular' KB folder into session
# All documents in that folder are now searchable

USE_KB "comunicado"
# Adds another KB to the session
# Both 'circular' and 'comunicado' are now active
```

### CLEAR_KB Keyword

```basic
CLEAR_KB
# Removes all loaded KBs from current session
# Frees up memory and context space
```

---

## Tool System

### What are Tools?

Tools are **callable functions** that the LLM can invoke to perform specific actions:
- Query databases
- Call APIs
- Process data
- Execute workflows
- Integrate with external systems

### Tool Definition

Tools are defined in `.gbtool` files with JSON schema:

```json
{
  "name": "get_weather",
  "description": "Get current weather for a location",
  "parameters": {
    "type": "object",
    "properties": {
      "location": {
        "type": "string",
        "description": "City name or coordinates"
      },
      "units": {
        "type": "string",
        "enum": ["celsius", "fahrenheit"],
        "default": "celsius"
      }
    },
    "required": ["location"]
  },
  "endpoint": "https://api.weather.com/current",
  "method": "GET"
}
```

### Tool Registration

Tools can be registered in three ways:

1. **Static Registration** - In bot configuration
2. **Dynamic Loading** - Via USE_TOOL keyword
3. **Auto-discovery** - From `.gbtool` files in work directory

### USE_TOOL Keyword

```basic
USE_TOOL "weather"
# Makes the weather tool available to LLM

USE_TOOL "database_query"
# Adds database query tool to session

USE_TOOL "email_sender"
# Enables email sending capability
```

### CLEAR_TOOLS Keyword

```basic
CLEAR_TOOLS
# Removes all tools from current session
# LLM can no longer call external functions
```

---

## Session Management

### Context Lifecycle

1. **Session Start** - Clean slate, no KB or tools
2. **Load Resources** - USE_KB and USE_TOOL as needed
3. **Active Use** - LLM uses loaded resources
4. **Clear Resources** - CLEAR_KB/CLEAR_TOOLS when done
5. **Session End** - Automatic cleanup

### Best Practices

#### KB Management

- **Load relevant KBs only** - Don't overload context
- **Clear when switching topics** - Keep context focused
- **Update KBs regularly** - Keep information current
- **Monitor token usage** - Vector search adds tokens

#### Tool Management

- **Enable minimal tools** - Only what's needed
- **Validate tool responses** - Check for errors
- **Log tool usage** - For audit and debugging
- **Set rate limits** - Prevent abuse

### Performance Considerations

#### Memory Usage

- Each KB uses ~100-500MB RAM (depends on size)
- Tools use minimal memory (<1MB each)
- Vector search adds 10-50ms latency
- Clear unused resources to free memory

#### Token Optimization

- KB chunks add 500-2000 tokens per query
- Tool descriptions use 50-200 tokens each
- Clear resources to reduce token usage
- Use specific KB folders vs entire database

---

## API Integration

### REST Endpoints

```http
# Load KB
POST /api/kb/load
{
  "session_id": "xxx",
  "kb_name": "circular"
}

# Clear KB
POST /api/kb/clear
{
  "session_id": "xxx"
}

# Load Tool
POST /api/tools/load
{
  "session_id": "xxx",
  "tool_name": "weather"
}

# Clear Tools
POST /api/tools/clear
{
  "session_id": "xxx"
}
```

### WebSocket Commands

```javascript
// Load KB
ws.send({
  type: "USE_KB",
  kb_name: "circular"
});

// Clear KB
ws.send({
  type: "CLEAR_KB"
});

// Load Tool
ws.send({
  type: "USE_TOOL",
  tool_name: "weather"
});

// Clear Tools
ws.send({
  type: "CLEAR_TOOLS"
});
```

---

## Implementation Details

### Vector Database (Qdrant)

Configuration:
- **Collection**: Per bot instance
- **Embedding Model**: text-embedding-ada-002
- **Dimension**: 1536
- **Distance**: Cosine similarity
- **Index**: HNSW with M=16, ef=100

### File Processing Pipeline

```rust
// src/basic/keywords/use_kb.rs
1. Scan directory for files
2. Extract text based on file type
3. Clean and normalize text
4. Split into chunks (1000 chars, 200 overlap)
5. Generate embeddings via OpenAI
6. Store in Qdrant with metadata
7. Update session context
```

### Tool Execution Engine

```rust
// src/basic/keywords/use_tool.rs
1. Parse tool definition (JSON schema)
2. Register with LLM context
3. Listen for tool invocation
4. Validate parameters
5. Execute tool (HTTP/function call)
6. Return results to LLM
7. Log execution for audit
```

---

## Error Handling

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `KB_NOT_FOUND` | KB folder doesn't exist | Check folder name and path |
| `VECTORDB_ERROR` | Qdrant connection issue | Check vectorDB service |
| `EMBEDDING_FAILED` | OpenAI API error | Check API key and limits |
| `TOOL_NOT_FOUND` | Tool not registered | Verify tool name |
| `TOOL_EXECUTION_ERROR` | Tool failed to execute | Check tool endpoint/logic |
| `MEMORY_LIMIT` | Too many KBs loaded | Clear unused KBs |

### Debugging

Enable debug logging:
```bash
RUST_LOG=debug cargo run
```

Check logs for:
- KB loading progress
- Embedding generation
- Vector search queries
- Tool invocations
- Error details

---

## Examples

### Customer Support Bot

```basic
# Load product documentation
USE_KB "product_docs"
USE_KB "faqs"

# Enable support tools
USE_TOOL "ticket_system"
USE_TOOL "knowledge_search"

# Bot now has access to docs and can create tickets
HEAR user_question
# ... process with KB context and tools ...

# Clean up after session
CLEAR_KB
CLEAR_TOOLS
```

### Research Assistant

```basic
# Load research papers
USE_KB "papers_2024"
USE_KB "citations"

# Enable research tools
USE_TOOL "arxiv_search"
USE_TOOL "citation_formatter"

# Assistant can now search papers and format citations
# ... research session ...

# Switch to different topic
CLEAR_KB
USE_KB "papers_biology"
```

### Enterprise Integration

```basic
# Load company policies
USE_KB "hr_policies"
USE_KB "it_procedures"

# Enable enterprise tools
USE_TOOL "active_directory"
USE_TOOL "jira_integration"
USE_TOOL "slack_notifier"

# Bot can now query AD, create Jira tickets, send Slack messages
# ... handle employee request ...

# End of shift cleanup
CLEAR_KB
CLEAR_TOOLS
```

---

## Security Considerations

### KB Security

- **Access Control** - KBs require authorization
- **Encryption** - Files encrypted at rest
- **Audit Logging** - All KB access logged
- **Data Isolation** - Per-session KB separation

### Tool Security

- **Authentication** - Tools require valid session
- **Rate Limiting** - Prevent tool abuse
- **Parameter Validation** - Input sanitization
- **Execution Sandboxing** - Tools run isolated

### Best Practices

1. **Principle of Least Privilege** - Only load needed resources
2. **Regular Audits** - Review KB and tool usage
3. **Secure Storage** - Encrypt sensitive KBs
4. **API Key Management** - Rotate tool API keys
5. **Session Isolation** - Clear resources between users

---

## Configuration

### Environment Variables

```bash
# Vector Database
QDRANT_URL=http://localhost:6333
QDRANT_API_KEY=your_key

# Embeddings
OPENAI_API_KEY=your_key
EMBEDDING_MODEL=text-embedding-ada-002
CHUNK_SIZE=1000
CHUNK_OVERLAP=200

# Tools
MAX_TOOLS_PER_SESSION=10
TOOL_TIMEOUT_SECONDS=30
TOOL_RATE_LIMIT=100

# KB
MAX_KB_PER_SESSION=5
MAX_KB_SIZE_MB=500
KB_SCAN_INTERVAL=3600
```

### Configuration File

```toml
# botserver.toml
[kb]
enabled = true
max_per_session = 5
embedding_model = "text-embedding-ada-002"
chunk_size = 1000
chunk_overlap = 200

[tools]
enabled = true
max_per_session = 10
timeout = 30
rate_limit = 100
sandbox = true

[vectordb]
provider = "qdrant"
url = "http://localhost:6333"
collection_prefix = "botserver_"
```

---

## Troubleshooting

### KB Issues

**Problem**: KB not loading
- Check folder exists in work/{bot_name}/{bot_name}.gbkb/
- Verify file permissions
- Check vector database connection
- Review logs for embedding errors

**Problem**: Poor search results
- Increase chunk overlap
- Adjust chunk size
- Update embedding model
- Clean/preprocess documents better

### Tool Issues

**Problem**: Tool not executing
- Verify tool registration
- Check parameter validation
- Test endpoint directly
- Review execution logs

**Problem**: Tool timeout
- Increase timeout setting
- Check network connectivity
- Optimize tool endpoint
- Add retry logic

---

## Migration Guide

### From File-based to Vector Search

1. Export existing files
2. Organize into .gbkb folders
3. Run embedding pipeline
4. Test vector search
5. Update bot logic

### From Static to Dynamic Tools

1. Convert function to tool definition
2. Create .gbtool file
3. Implement endpoint/handler
4. Test with USE_TOOL
5. Remove static registration

---

## Future Enhancements

### Planned Features

- **Incremental KB Updates** - Add/remove single documents
- **Multi-language Support** - Embeddings in multiple languages
- **Tool Chaining** - Tools calling other tools
- **KB Versioning** - Track KB changes over time
- **Smart Caching** - Cache frequent searches
- **Tool Analytics** - Usage statistics and optimization

### Roadmap

- Q1 2024: Incremental updates, multi-language
- Q2 2024: Tool chaining, KB versioning
- Q3 2024: Smart caching, analytics
- Q4 2024: Advanced security, enterprise features