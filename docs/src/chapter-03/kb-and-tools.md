# KB and TOOL System Documentation

## Overview

The General Bots system provides **4 essential keywords** for managing Knowledge Bases (KB) and Tools dynamically during conversation sessions:

1. **USE KB** - Load and embed files from `.gbkb` folders into vector database
2. **CLEAR KB** - Remove KB from current session
3. **USE TOOL** - Make a tool available for LLM to call
4. **CLEAR TOOLS** - Remove all tools from current session

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
5. **Store in VectorDB** - Saves to vector database for similarity search
6. **Ready for queries** - KB available for semantic search

### Supported File Types

- **PDF** - Full text extraction with pdf-extract
- **DOCX/DOC** - Microsoft Word documents
- **TXT** - Plain text files
- **MD** - Markdown documents
- **CSV** - Structured data (each row as entry)
- **HTML** - Web pages (text only)
- **JSON** - Structured data

### USE KB Keyword

```basic
USE KB "circular"
# Loads the 'circular' KB folder into session
# All documents in that folder are now searchable

USE KB "comunicado"
# Adds another KB to the session
# Both 'circular' and 'comunicado' are now active
```

### CLEAR KB Keyword

```basic
CLEAR KB
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

Tools are defined in .bas files that generate MCP and OpenAI-compatible tool definitions:

```basic
' weather.bas - becomes a tool automatically
PARAM location AS string
PARAM units AS string DEFAULT "celsius"
DESCRIPTION "Get current weather for a location"

' Tool implementation
weather_data = GET "https://api.weather.com/v1/current?location=" + location
' System AI will format and present the data naturally
SET CONTEXT "weather_data", weather_data
TALK "Here's the current weather for " + location
```

### Tool Registration

Tools are registered in two ways:

1. **Auto-discovery** - All `.bas` files in `.gbdialog` folder (except start.bas) become tools
2. **Dynamic Loading** - Via USE TOOL keyword for external tools

### USE TOOL Keyword

```basic
USE TOOL "weather"
# Makes the weather tool available to LLM

USE TOOL "database_query"
# Adds database query tool to session

USE TOOL "email_sender"
# Enables email sending capability
```

### CLEAR TOOLS Keyword

```basic
CLEAR TOOLS
# Removes all tools from current session
# LLM can no longer call external functions
```

---

## Session Management

### Context Lifecycle

1. **Session Start** - Clean slate, no KB or tools
2. **Load Resources** - USE KB and USE TOOL as needed
3. **Active Use** - LLM uses loaded resources
4. **Clear Resources** - CLEAR KB/CLEAR TOOLS when done
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

## Implementation Details

### Vector Database

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
6. Store in vector database with metadata
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
### Common Issues

| Error | Cause | Solution |
|-------|-------|----------|
| `KB_NOT_FOUND` | KB folder doesn't exist | Check folder name and path |
| `VECTORDB_ERROR` | Vector database connection issue | Check vector database service |
| `EMBEDDING_FAILED` | Embedding API error | Check API key and limits |
| `TOOL_NOT_FOUND` | Tool not registered | Verify tool name |
| `TOOL_EXECUTION_ERROR` | Tool failed to execute | Check tool endpoint/logic |
| `MEMORY_LIMIT` | Too many KBs loaded | Clear unused KBs |



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
USE KB "product_docs"
USE KB "faqs"

# Enable support tools
USE TOOL "ticket_system"
USE TOOL "knowledge_search"

# System AI now has access to docs and can work with tickets
TALK "How can I help you with your support needs today?"
# System AI automatically searches KB and uses tools when responding

# Clean up after session
CLEAR KB
CLEAR TOOLS
```

### Research Assistant

```basic
# Load research papers
USE KB "papers_2024"
USE KB "citations"

# Enable research tools
USE TOOL "arxiv_search"
USE TOOL "citation_formatter"

# System AI can now search papers and format citations
TALK "What research topic would you like to explore?"

# Switch to different topic
CLEAR KB
USE KB "papers_biology"
```

### Enterprise Integration

```basic
# Load company policies
USE KB "hr_policies"
USE KB "it_procedures"

# Enable enterprise tools
USE TOOL "active_directory"
USE TOOL "jira_integration"
USE TOOL "slack_notifier"

# Bot can now query AD, work with Jira, send Slack messages
# ... handle employee request ...

# End of shift cleanup
CLEAR KB
CLEAR TOOLS
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
4. Test with USE TOOL
5. Remove static registration

---

## See Also

### Documentation
- [Vector Collections](./vector-collections.md) - How vector search works
- [Document Indexing](./indexing.md) - Automatic document processing
- [Semantic Search](./semantic-search.md) - Meaning-based retrieval
- [Context Compaction](./context-compaction.md) - Managing conversation context
- [Caching](./caching.md) - Performance optimization
- [Chapter 6: BASIC Reference](../chapter-06-gbdialog/README.md) - Dialog scripting
- [Chapter 9: API and Tools](../chapter-09-api/README.md) - Tool integration

### Further Reading - Blog Posts
- [BASIC LLM Tools](https://pragmatismo.com.br/blog/basic-llm-tools) - Extending LLMs with tools
- [MCP is the new API](https://pragmatismo.com.br/blog/mcp-is-the-new-api) - Modern tool integration
- [Beyond Chatbots](https://pragmatismo.com.br/blog/beyond-chatbots) - Using knowledge bases effectively

### Next Chapter
Continue to [Chapter 4: User Interface](../chapter-04-gbui/README.md) to learn about creating bot interfaces.
