# KB and TOOL System Documentation

## Overview

The General Bots system provides four essential keywords for managing Knowledge Bases and Tools dynamically during conversation sessions. The USE KB keyword loads and embeds files from `.gbkb` folders into the vector database. The CLEAR KB keyword removes a knowledge base from the current session. The USE TOOL keyword makes a tool available for the LLM to call. The CLEAR TOOLS keyword removes all tools from the current session. Together, these keywords give you complete control over what information and capabilities your bot has access to at any moment.


## Knowledge Base System

### What is a KB?

A Knowledge Base is a folder containing documents (using the `.gbkb` folder structure) that are vectorized, embedded, and stored in a vector database. When users ask questions, the vector database retrieves relevant chunks and excerpts to inject into prompts, giving the LLM context-aware responses based on your specific documentation and data.

### Folder Structure

Knowledge bases are organized within your bot's work directory. The structure places all knowledge base folders inside a `.gbkb` container that shares your bot's name. Within this container, you create separate folders for different topics or document collections. Each folder can contain PDF files, markdown documents, plain text files, Word documents, CSV files, and other supported formats.

```
work/
  {bot_name}/
    {bot_name}.gbkb/
      circular/
        document1.pdf
        document2.md
        document3.txt
      comunicado/
        info.docx
        data.csv
      docs/
        README.md
        guide.pdf
```

### KB Loading Process

When you load a knowledge base, the system goes through several stages to make your documents searchable. First, the system scans the specified `.gbkb` folder to identify all documents. Then it processes each file by extracting text from PDFs, Word documents, text files, markdown, CSV files, and other supported formats. The extracted text is split into chunks of approximately 1000 characters with overlap between chunks to preserve context at boundaries. Each chunk is then converted into a vector representation using an embedding model. These vectors are stored in the vector database with metadata about their source, enabling fast similarity search. Once this process completes, the knowledge base is ready to answer semantic queries.

### Supported File Types

The system supports a variety of document formats. PDF files receive full text extraction using the pdf-extract library. Microsoft Word documents in both DOCX and DOC formats are supported. Plain text files and markdown documents are processed directly. CSV files treat each row as a separate searchable entry. HTML files have their text content extracted while ignoring markup. JSON files are parsed and their structured data becomes searchable.

### USE KB Keyword

The USE KB keyword loads a knowledge base folder into your current session. You can load multiple knowledge bases, and all of them become active simultaneously. This allows you to combine different document collections for comprehensive responses.

```basic
USE KB "circular"
' The circular KB folder is now loaded and searchable
' All documents in that folder are available for semantic queries

USE KB "comunicado"
' Now both circular and comunicado are active
' The LLM can draw from both collections when responding
```

### CLEAR KB Keyword

The CLEAR KB keyword removes all loaded knowledge bases from the current session. This frees up memory and context space, which is particularly useful when switching between different topics or when you need to ensure the LLM only uses specific information.

```basic
CLEAR KB
' All loaded knowledge bases are removed
' Memory is freed and context space is reclaimed
```


## Tool System

### What are Tools?

Tools are callable functions that the LLM can invoke to perform specific actions beyond its training data. Tools enable your bot to query databases, call external APIs, process data, execute workflows, and integrate with external systems. When the LLM determines that a tool would help answer a user's question, it generates a tool call with the appropriate parameters.

### Tool Definition

Tools are defined in `.bas` files that automatically generate MCP and OpenAI-compatible tool definitions. When you create a BASIC file with PARAM declarations and a DESCRIPTION, the system compiles it into a tool specification that the LLM understands.

```basic
' weather.bas - becomes a tool automatically
PARAM location AS string
PARAM units AS string DEFAULT "celsius"
DESCRIPTION "Get current weather for a location"

' Tool implementation
weather_data = GET "https://api.weather.com/v1/current?location=" + location
SET CONTEXT "weather_data", weather_data
TALK "Here's the current weather for " + location
```

### Tool Registration

Tools become available through two mechanisms. Auto-discovery scans all `.bas` files in your `.gbdialog` folder (except `start.bas`) and registers them as tools automatically. Dynamic loading uses the USE TOOL keyword to make external tools available during a session.

### USE TOOL Keyword

The USE TOOL keyword makes a specific tool available for the LLM to call. You can enable multiple tools, giving your bot access to various capabilities during a conversation.

```basic
USE TOOL "weather"
' The weather tool is now available

USE TOOL "database_query"
' Database querying capability is added

USE TOOL "email_sender"
' The bot can now send emails when appropriate
```

### CLEAR TOOLS Keyword

The CLEAR TOOLS keyword removes all tools from the current session. After clearing, the LLM can no longer call external functions and must rely solely on its training and any loaded knowledge bases.

```basic
CLEAR TOOLS
' All tools are disabled
' LLM cannot call external functions
```


## Session Management

### Context Lifecycle

Each conversation session follows a predictable lifecycle. When a session starts, the bot has a clean slate with no knowledge bases or tools loaded. During the conversation, you load resources as needed using USE KB and USE TOOL commands. The LLM actively uses these loaded resources to provide informed, capable responses. When the topic changes or resources are no longer needed, you clear them with CLEAR KB and CLEAR TOOLS. When the session ends, automatic cleanup releases all remaining resources.

### Best Practices for KB Management

Load only the knowledge bases relevant to the current conversation. Overloading context with unnecessary KBs reduces response quality and increases costs. Clear knowledge bases when switching topics to keep the context focused on what matters. Update your KB files regularly to keep the information current. Monitor token usage because vector search results add tokens to each query.

### Best Practices for Tool Management

Enable only the minimum set of tools needed for the current task. Having too many tools available can confuse the LLM about which one to use. Always validate tool responses and check for errors before presenting results to users. Log tool usage for audit purposes and debugging. Consider implementing rate limits to prevent abuse in production environments.

### Performance Considerations

Memory usage varies based on your configuration. Each loaded knowledge base typically uses 100-500MB of RAM depending on document count and size. Tools use minimal memory, usually less than 1MB each. Vector search operations add 10-50ms latency to responses. Clear unused resources promptly to free memory for other operations.

Token optimization is important for controlling costs. KB chunks add 500-2000 tokens per query depending on the number of relevant chunks retrieved. Each tool description uses 50-200 tokens. Clearing resources when they are no longer needed reduces token usage. Using specific KB folders rather than loading entire databases improves both performance and relevance.


## Implementation Details

### Vector Database

The vector database configuration uses one collection per bot instance to maintain isolation. The embedding model is text-embedding-ada-002, which produces 1536-dimensional vectors. Distance calculations use cosine similarity for semantic matching. The index uses HNSW (Hierarchical Navigable Small World) with M=16 and ef=100 for fast approximate nearest neighbor search.

### File Processing Pipeline

When USE KB processes files, it follows a systematic pipeline. The system scans the specified directory to identify all files. Text is extracted based on each file's type using appropriate parsers. The extracted text is cleaned and normalized to remove artifacts. Content is split into chunks of approximately 1000 characters with 200 character overlap to preserve context across boundaries. Embeddings are generated via the OpenAI API for each chunk. The vectors are stored in the vector database along with metadata about their source. Finally, the session context is updated to reflect the newly available knowledge base.

### Tool Execution Engine

When USE TOOL prepares a tool for use, it parses the tool definition into a JSON schema that describes parameters and expected behavior. This schema is registered with the LLM context so the model knows the tool is available. The system listens for tool invocations in the LLM's responses. When a tool call is detected, parameters are validated against the schema. The tool executes its logic, which might involve HTTP requests or function calls. Results return to the LLM for incorporation into the response. All executions are logged for audit purposes.


## Error Handling

### Common Issues

Several error conditions can occur when working with knowledge bases and tools. The KB_NOT_FOUND error indicates that the specified KB folder does not exist, so you should verify the folder name and path. A VECTORDB_ERROR suggests a connection issue with the vector database service that needs investigation. EMBEDDING_FAILED errors typically indicate problems with the embedding API, often related to API keys or rate limits. TOOL_NOT_FOUND means the specified tool is not registered, so verify the tool name matches exactly. TOOL_EXECUTION_ERROR indicates the tool failed during execution, requiring investigation of the tool endpoint or logic. MEMORY_LIMIT errors occur when too many knowledge bases are loaded simultaneously, requiring you to clear unused KBs.

### Debugging Approach

Check logs for detailed information about issues. KB loading progress shows which documents are being processed. Embedding generation logs reveal any failures during vectorization. Vector search query logs help diagnose relevance problems. Tool invocation logs show parameter values and execution results. Error details provide stack traces and specific failure reasons.


## Examples

### Customer Support Bot

This example shows a customer support bot that loads product documentation and FAQs, enables ticket management tools, and provides informed assistance.

```basic
' Load product documentation
USE KB "product_docs"
USE KB "faqs"

' Enable support tools
USE TOOL "ticket_system"
USE TOOL "knowledge_search"

' The bot now has access to documentation and can work with tickets
TALK "How can I help you with your support needs today?"

' When the session ends, clean up
CLEAR KB
CLEAR TOOLS
```

### Research Assistant

This example demonstrates a research assistant that can switch between different knowledge base collections depending on the research topic.

```basic
' Load research papers for current topic
USE KB "papers_2024"
USE KB "citations"

' Enable research tools
USE TOOL "arxiv_search"
USE TOOL "citation_formatter"

TALK "What research topic would you like to explore?"

' When switching to a different research area
CLEAR KB
USE KB "papers_biology"
```

### Enterprise Integration

This example shows an enterprise bot with access to company policies and integration with internal systems like Active Directory, Jira, and Slack.

```basic
' Load company policies
USE KB "hr_policies"
USE KB "it_procedures"

' Enable enterprise integration tools
USE TOOL "active_directory"
USE TOOL "jira_integration"
USE TOOL "slack_notifier"

' The bot can now query AD, work with Jira tickets, and send Slack notifications
' Handle employee requests throughout the conversation

' Clean up at end of shift
CLEAR KB
CLEAR TOOLS
```


## Security Considerations

### KB Security

Knowledge base security involves multiple layers of protection. Access control ensures that knowledge bases require proper authorization before loading. Files are encrypted at rest to protect sensitive information. All KB access is logged for audit purposes. Per-session KB separation ensures that one user's loaded knowledge bases cannot leak to another session.

### Tool Security

Tool security protects against misuse and unauthorized access. Authentication requirements ensure tools only execute within valid sessions. Rate limiting prevents tool abuse through excessive calls. Parameter validation sanitizes all inputs before execution. Execution sandboxing isolates tool operations from the core system.

### Best Practices

Follow the principle of least privilege by loading only the resources needed for the current task. Conduct regular audits to review KB and tool usage patterns. Ensure sensitive knowledge bases use encrypted storage. Rotate API keys used by tools on a regular schedule. Maintain session isolation by clearing resources between different users.


## Configuration

Configuration options for knowledge bases and tools are set in your bot's config.csv file. The vector database connection settings specify where embeddings are stored. Chunk size and overlap parameters control how documents are split. Embedding model selection determines vector quality and dimension. Tool timeout settings prevent long-running operations from blocking conversations.


## Troubleshooting

### KB Issues

If a knowledge base is not loading, first verify that the folder exists at the expected path within `work/{bot_name}/{bot_name}.gbkb/`. Check file permissions to ensure the system can read the documents. Verify the vector database connection is healthy. Review logs for any embedding errors during processing.

If search results are poor quality, consider adjusting the chunk overlap to provide more context at boundaries. Experiment with different chunk sizes for your content type. Ensure your embedding model is appropriate for the content language. Pre-process documents to remove noise and improve text quality before indexing.

### Tool Issues

If a tool is not executing, first verify that the tool registration completed successfully by checking logs. Confirm parameter validation rules match the values being passed. Test the tool endpoint directly outside of the bot to isolate the issue. Review execution logs for specific error messages.

If tools are timing out, increase the timeout setting in configuration. Check network connectivity between the bot and tool endpoints. Optimize the tool endpoint to respond faster. Consider adding retry logic for transient failures.


## Migration Guide

### From File-based to Vector Search

If you are migrating from a file-based knowledge system to vector search, start by exporting your existing files into a clean directory structure. Organize the files into logical `.gbkb` folders based on topic or department. Run the embedding pipeline by loading each KB with USE KB. Test vector search queries to verify results match expectations. Update your bot logic to use the new KB keywords instead of file operations.

### From Static to Dynamic Tools

If you have static function calls that should become dynamic tools, convert each function into a tool definition with PARAM declarations. Create a `.bas` file with the DESCRIPTION and parameter specifications. Implement the endpoint or handler that the tool will call. Test the tool using USE TOOL and verify it executes correctly. Remove the static function registration from your startup logic.


## See Also

### Documentation

The Vector Collections page explains how vector search works under the hood. The Document Indexing page covers automatic document processing in detail. The Semantic Search page describes meaning-based retrieval algorithms. The Context Compaction page explains how conversation context is managed. The Caching page covers performance optimization through semantic caching. The Chapter 6 BASIC Reference provides complete dialog scripting documentation. The Chapter 9 API and Tools reference covers tool integration in depth.

### Further Reading

The Pragmatismo blog post on BASIC LLM Tools explains how to extend LLMs with custom tools. The MCP is the new API article covers modern tool integration patterns. The Beyond Chatbots post discusses using knowledge bases effectively for sophisticated applications.

### Next Chapter

Continue to Chapter 4 on User Interface to learn about creating bot interfaces that present your knowledge base and tool capabilities to users effectively.