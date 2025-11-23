# .gbdrive File Storage

The `.gbdrive` system manages file storage and retrieval using object storage (S3-compatible drive).

## What is .gbdrive?

`.gbdrive` provides:
- Centralized file storage for all packages
- Versioning and backup capabilities
- Access control and organization
- Integration with knowledge bases and tools

## Storage Structure

Files are organized in a bucket-per-bot structure:

```
org-prefixbot-name.gbai/
  .gbdialog/
    [script files]
  .gbkb/
    [collection folders]
  .gbot/
    config.csv
  .gbtheme/
    [theme files]
  user-uploads/
    [user files]
```

## File Operations

### Uploading Files
```basic
REM Files can be uploaded via API or interface
REM They are stored in the bot's storage bucket
```

### Retrieving Files  
```basic
REM Get file content as text
GET ".gbdialog/start.bas"

REM Download files via URL
GET "user-uploads/document.pdf"
```

### File Management
- Automatic synchronization on bot start
- Change detection for hot reloading
- Version history maintenance
- Backup and restore capabilities

## Integration Points

### Knowledge Bases
- Documents are stored in .gbkb collections
- Automatic processing and embedding
- Version tracking for updates

### Themes
- Static assets served from .gbtheme
- CSS, JS, and HTML files
- Caching for performance

### Tools
- Tool scripts stored in .gbdialog
- AST and compiled versions
- Dependency management

## Access Control

Files have different access levels:
- **Public**: Accessible without authentication
- **Authenticated**: Requires user login
- **Bot-only**: Internal bot files
- **Admin**: Configuration and sensitive files

## Storage Backends

- **Object Storage** (default): Self-hosted S3-compatible drive
- **AWS S3**: Cloud object storage
- **Local filesystem**: Development and testing
- **Hybrid**: Multiple backends with fallback
