# GET Keyword Integration

The `GET` keyword in BotServer provides file retrieval capabilities from both local filesystem and drive (S3-compatible) storage, enabling tools to access documents, data files, and other resources.

## Overview

The `GET` keyword is a fundamental BASIC command that retrieves file contents as strings. It supports:
- Local file system access (with safety checks)
- Drive (S3-compatible) bucket retrieval
- URL fetching (HTTP/HTTPS)
- Integration with knowledge base documents

## Basic Usage

```basic
# Get a file from the bot's bucket
let content = GET "documents/policy.pdf"

# Get a file with full path
let data = GET "announcements.gbkb/news/news.pdf"

# Get from URL
let webpage = GET "https://example.com/data.json"
```

## Implementation Details

### File Path Resolution

The GET keyword determines the source based on the path format:

1. **URL Detection**: Paths starting with `http://` or `https://`
2. **Drive Storage**: All other paths (retrieved from bot's bucket)
3. **Safety Validation**: Paths are checked for directory traversal attempts

### Drive (S3-compatible) Integration

When retrieving from drive storage:

```basic
# Retrieves from: {bot-name}.gbai bucket
let doc = GET "knowledge/document.txt"

# Full path within bucket
let report = GET "reports/2024/quarterly.pdf"
```

The implementation:
1. Connects to drive using configured credentials
2. Retrieves from the bot's dedicated bucket
3. Returns file contents as string
4. Handles binary files by converting to text

### URL Fetching

For external resources:

```basic
let api_data = GET "https://api.example.com/data"
let webpage = GET "http://example.com/page.html"
```

URL fetching includes:
- HTTP/HTTPS support
- Automatic redirect following
- Timeout protection (30 seconds)
- Error handling for failed requests

## Safety Features

### Path Validation

The `is_safe_path` function prevents directory traversal:
- Blocks paths containing `..`
- Prevents absolute paths
- Validates character sets
- Ensures sandbox isolation

### Access Control

- Files limited to bot's own bucket
- Cannot access other bots' data
- System directories protected
- Credentials never exposed

## Error Handling

GET operations handle various error conditions:

```basic
let content = GET "missing-file.txt"
# Returns empty string if file not found

if (content == "") {
    TALK "File not found or empty"
}
```

Common errors:
- File not found: Returns empty string
- Access denied: Returns error message
- Network timeout: Returns timeout error
- Invalid path: Returns security error

## Use Cases

### Loading Knowledge Base Documents

```basic
# In update-summary.bas - background processing script
let text = GET "announcements.gbkb/news/news.pdf"
let summary = LLM "Summarize this: " + text  # LLM for background processing only
SET BOT MEMORY "news_summary", summary  # Stored for all users
```

### Reading Configuration Files

```basic
let config = GET "settings.json"
# Parse and use configuration
```

### Fetching External Data

```basic
let weather_data = GET "https://api.weather.com/current"
# Process weather information
```

### Loading Templates

```basic
let template = GET "templates/email-template.html"
let filled = REPLACE(template, "{{name}}", customer_name)
```

## Performance Considerations

### Caching

- GET results are not cached by default
- Frequent reads should use BOT_MEMORY for caching
- Large files impact memory usage

### Timeouts

- URL fetches: 30-second timeout
- Drive operations: Network-dependent
- Local files: Immediate (if accessible)

### File Size Limits

- No hard limit enforced
- Large files consume memory
- Binary files converted to text (may be large)

## Integration with Tools

### Tool Parameters from Files

```basic
PARAM config_file AS string LIKE "config.json" DESCRIPTION "Configuration file path"

let config = GET config_file
# Use configuration in tool logic
```

### Dynamic Resource Loading

```basic
DESCRIPTION "Process documents from a folder"

let file_list = GET "documents/index.txt"
let files = SPLIT(file_list, "\n")

FOR EACH file IN files {
    let content = GET "documents/" + file
    # Process each document
}
```

## Best Practices

1. **Check for Empty Results**: Always verify GET returned content
2. **Use Relative Paths**: Avoid hardcoding absolute paths
3. **Handle Binary Files Carefully**: Text conversion may be lossy
4. **Cache Frequently Used Files**: Store in BOT_MEMORY
5. **Validate External URLs**: Ensure HTTPS for sensitive data
6. **Log Access Failures**: Track missing or inaccessible files

## Limitations

- Cannot write files (read-only operation)
- Binary files converted to text (may corrupt data)
- No streaming support (entire file loaded to memory)
- Path traversal blocked for security
- Cannot access system directories

## Examples

### Document Summarization Tool

```basic
PARAM doc_path AS string LIKE "reports/annual.pdf" DESCRIPTION "Document to summarize"
DESCRIPTION "Summarizes a document"

let content = GET doc_path

if (content == "") {
    TALK "Document not found: " + doc_path
} else {
    # Set document as context for system AI
    SET CONTEXT "document", content
    TALK "I've loaded the document. What would you like to know about it?"
}
```

### Data Processing Tool

```basic
PARAM data_file AS string LIKE "data/sales.csv" DESCRIPTION "Data file to process"
DESCRIPTION "Analyzes sales data"

let csv_data = GET data_file
# Set data as context for system AI
SET CONTEXT "sales_data", csv_data
TALK "I've loaded the sales data. What analysis would you like me to perform?"
```

## Security Considerations

- Never GET files with user-controlled paths directly
- Validate and sanitize path inputs
- Use allowlists for acceptable file paths
- Log all file access attempts
- Monitor for unusual access patterns

## Summary

The GET keyword provides essential file retrieval capabilities for BASIC tools, enabling access to documents, configuration, and external resources while maintaining security through path validation and sandboxing.