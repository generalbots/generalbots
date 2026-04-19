# GET Keyword Integration

The `GET` keyword in botserver provides file retrieval capabilities from both local filesystem and drive (S3-compatible) storage, enabling tools to access documents, data files, and other resources.

## Overview

The `GET` keyword is a fundamental BASIC command that retrieves file contents as strings. It supports local file system access with safety checks, drive (S3-compatible) bucket retrieval, URL fetching via HTTP and HTTPS, and integration with knowledge base documents.

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

The GET keyword determines the source based on the path format. URL detection occurs for paths starting with `http://` or `https://`, which triggers HTTP fetching. All other paths are retrieved from drive storage in the bot's dedicated bucket. Safety validation checks all paths for directory traversal attempts before processing.

### Drive (S3-compatible) Integration

When retrieving from drive storage, the system connects to drive using configured credentials and retrieves files from the bot's dedicated bucket. File contents are returned as strings, with binary files converted to text automatically.

```basic
# Retrieves from: {bot-name}.gbai bucket
let doc = GET "knowledge/document.txt"

# Full path within bucket
let report = GET "reports/2024/quarterly.pdf"
```

### URL Fetching

For external resources, the GET keyword supports both HTTP and HTTPS protocols with automatic redirect following. A 30-second timeout protects against hanging requests, and comprehensive error handling manages failed requests gracefully.

```basic
let api_data = GET "https://api.example.com/data"
let webpage = GET "http://example.com/page.html"
```

## Safety Features

### Path Validation

The `is_safe_path` function prevents directory traversal attacks by blocking paths containing `..` sequences and rejecting absolute paths. Character sets are validated to ensure only safe characters appear in paths, and sandbox isolation ensures scripts cannot escape their designated storage areas.

### Access Control

Files are limited to the bot's own bucket, preventing access to other bots' data. System directories receive protection from all access attempts, and credentials are never exposed through the GET interface regardless of the path requested.

## Error Handling

GET operations handle various error conditions gracefully. When a file is not found, the operation returns an empty string rather than throwing an error. Access denied conditions return an error message, network timeouts return a timeout error, and invalid paths return a security error.

```basic
let content = GET "missing-file.txt"
# Returns empty string if file not found

if (content == "") {
    TALK "File not found or empty"
}
```

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

GET results are not cached by default, so frequent reads should use BOT_MEMORY for caching to improve performance. Large files impact memory usage significantly since the entire file is loaded into memory at once.

### Timeouts

URL fetches enforce a 30-second timeout to prevent indefinite hanging. Drive operations depend on network conditions and may vary in response time. Local files are accessed immediately when accessible.

### File Size Limits

No hard limit is enforced on file sizes, but large files consume substantial memory. Binary files converted to text may result in particularly large string representations.

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

Always check for empty results to verify GET returned content successfully. Use relative paths rather than hardcoding absolute paths to maintain portability. Handle binary files carefully since text conversion may be lossy for non-text content. Cache frequently used files in BOT_MEMORY to avoid repeated retrieval operations. Validate external URLs and ensure HTTPS is used for sensitive data transfers. Log access failures to track missing or inaccessible files for debugging purposes.

## Limitations

The GET keyword is a read-only operation and cannot write files. Binary files are converted to text which may corrupt data that isn't text-based. No streaming support exists, meaning the entire file loads into memory at once. Path traversal is blocked for security, and system directories cannot be accessed under any circumstances.

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

Never GET files with user-controlled paths directly without validation. Always validate and sanitize path inputs before passing them to GET. Use allowlists for acceptable file paths when possible. Log all file access attempts for security auditing, and monitor for unusual access patterns that might indicate attempted exploitation.

## Summary

The GET keyword provides essential file retrieval capabilities for BASIC tools, enabling access to documents, configuration, and external resources while maintaining security through path validation and sandboxing.