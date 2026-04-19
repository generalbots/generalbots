# .gbdrive File Storage

The .gbdrive system provides centralized file storage for all bot packages, leveraging S3-compatible object storage to deliver reliable, scalable storage infrastructure. This chapter explains how file storage works, how files are organized, and how to interact with stored content.

## Understanding File Storage in General Bots

Every bot requires storage for its various components—scripts, documents, configuration files, user uploads, and generated content. Rather than managing files across disparate locations, General Bots consolidates storage through the .gbdrive system, which provides a consistent interface regardless of the underlying storage backend.

The storage system builds on S3-compatible object storage, meaning it works with self-hosted solutions like MinIO as well as cloud providers like AWS S3, Backblaze B2, or DigitalOcean Spaces. This flexibility allows deployments to choose storage solutions that match their requirements for cost, performance, and data residency.

Beyond simple file storage, the system provides versioning capabilities, access control, automatic synchronization, and integration with other bot components like knowledge bases and themes.

## Storage Organization

Files are organized using a bucket-per-bot structure that keeps each bot's content isolated and manageable. Within a bot's storage bucket, the familiar package structure appears: .gbdialog for scripts, .gbkb for knowledge base collections, .gbot for configuration, and .gbtheme for interface customization.

Additionally, each bot has space for user-uploaded files, generated content, and other runtime data. This organization mirrors the logical structure you work with during development, making it intuitive to understand where files reside and how they relate to bot functionality.

The system maintains this structure automatically when bots are deployed or updated, ensuring that the storage state reflects the current bot configuration without manual intervention.

## .gbusers - Per-User Storage

The `.gbusers` folder within `.gbdrive` provides isolated storage space for each user interacting with the bot. This enables personalized document storage, user-specific settings, and application data that persists across sessions.

### User Folder Structure

User folders are identified by the user's email address or phone number:

```
mybot.gbai/
  mybot.gbdrive/
    users/
      john@example.com/           # User identified by email
        papers/
          current/                # Active/working documents
            untitled-1.md
            meeting-notes.md
          named/                  # Saved/named documents
            quarterly-report/
              document.md
              attachments/
            project-proposal/
              document.md
        uploads/                  # User file uploads
        exports/                  # Generated exports (PDF, DOCX, etc.)
        settings/                 # User preferences
          preferences.json
      +5511999887766/             # User identified by phone number
        papers/
          current/
          named/
        uploads/
```

### User Identifier Format

Users are identified by their primary contact method:

- **Email**: `john@example.com`, `maria@company.com.br`
- **Phone**: `+5511999887766`, `+1234567890` (E.164 format)

The identifier is sanitized for filesystem compatibility while remaining human-readable.

### Paper Document Storage

The Paper application stores user documents in the `papers/` directory:

- **`papers/current/`**: Working documents that are actively being edited. These may be auto-saved drafts or recently accessed files.
- **`papers/named/`**: Documents that have been explicitly saved with a name. Each named document gets its own folder to support attachments and metadata.

Example document structure:
```
papers/
  current/
    untitled-1.md           # Auto-saved draft
    untitled-2.md           # Another working document
  named/
    meeting-notes-2024/
      document.md           # The main document content
      metadata.json         # Title, created_at, updated_at, etc.
      attachments/          # Embedded images or files
        image-001.png
    research-paper/
      document.md
      metadata.json
```

### Accessing User Storage from BASIC

BASIC scripts can access user storage using the `USER DRIVE` keyword:

```basic
' Read a user's document
content = READ USER DRIVE "papers/current/notes.md"

' Write to user's storage
SAVE USER DRIVE "papers/named/report/document.md", report_content

' List user's papers
papers = LIST USER DRIVE "papers/named/"

' Delete a user document
DELETE USER DRIVE "papers/current/draft.md"
```

### User Storage API

The REST API provides endpoints for user storage operations:

```
GET  /api/drive/user/list?path=papers/current/
POST /api/drive/user/read
     { "path": "papers/named/report/document.md" }
POST /api/drive/user/write
     { "path": "papers/current/notes.md", "content": "..." }
POST /api/drive/user/delete
     { "path": "papers/current/draft.md" }
```

All user storage API calls require authentication and automatically scope operations to the authenticated user's folder.

### Storage Quotas

Each user has configurable storage limits:

| Setting | Default | Description |
|---------|---------|-------------|
| `user-storage-quota` | 100MB | Maximum total storage per user |
| `user-file-limit` | 5MB | Maximum single file size |
| `user-file-count` | 500 | Maximum number of files |

Configure in `config.csv`:
```csv
user-storage-quota,104857600
user-file-limit,5242880
user-file-count,500
```

## Working with Files

File operations in General Bots happen through several interfaces depending on your needs. The BASIC scripting language provides keywords for reading file content directly into scripts, enabling bots to process documents, load data, or access configuration dynamically.

Files can also be managed through the administrative API for bulk operations, migrations, or integration with external systems. The web interface provides user-facing upload and download capabilities where appropriate.

When files change in storage, the system detects modifications and triggers appropriate responses. Script changes cause recompilation, document changes trigger knowledge base reindexing, and configuration changes reload bot settings. This hot-reloading capability accelerates development and enables runtime updates without service interruption.

## Integration with Bot Components

The storage system integrates deeply with other bot components, serving as the foundation for several capabilities.

Knowledge bases draw their source documents from storage, with the indexing system monitoring for changes and updating embeddings accordingly. When you add a document to a .gbkb folder, it automatically becomes part of the bot's searchable knowledge.

Theme assets including CSS files and images are served from storage, with appropriate caching to ensure good performance. Changes to theme files take effect quickly without requiring restarts.

Tool scripts in .gbdialog folders are loaded from storage, parsed, and made available for execution. The compilation system tracks dependencies and rebuilds as needed when source files change.

### Paper Application Integration

The Paper document editor automatically saves to the user's `.gbusers` folder:

1. **Auto-save**: Every 30 seconds, working documents are saved to `papers/current/`
2. **Explicit save**: When users click "Save", documents move to `papers/named/{document-name}/`
3. **Export**: Generated exports (PDF, DOCX) are saved to `exports/` and offered for download
4. **AI-generated content**: AI responses can be inserted into documents and saved automatically

## Access Control

Different files require different access levels, and the storage system enforces appropriate controls:

- **Public files**: Accessible without authentication, suitable for shared resources
- **Authenticated access**: Requires valid user credentials, protects user-specific content
- **User-scoped access**: Users can only access their own `.gbusers` folder content
- **Bot-internal files**: Accessible only to the bot system itself
- **Administrative files**: Require elevated privileges to access or modify

User storage in `.gbusers` is strictly isolated—users cannot access other users' folders through any API or BASIC keyword.

## Storage Backend Options

The storage system supports multiple backends to accommodate different deployment scenarios. The default configuration uses self-hosted S3-compatible object storage, providing full control over where data resides. Any S3-compatible service works as an alternative, including major cloud providers.

For development and testing, local filesystem storage offers simplicity and easy inspection of files. Production deployments might use hybrid configurations with multiple backends providing redundancy or geographic distribution.

Backend selection happens through configuration, and the rest of the system interacts with storage through a consistent interface regardless of which backend is active. This abstraction allows deployments to change storage strategies without modifying bot code.

## Directory Structure Reference

Complete `.gbdrive` structure with all components:

```
mybot.gbai/
  mybot.gbdrive/
    dialogs/              # Compiled dialog scripts cache
    kb/                   # Knowledge base index data
    cache/                # Temporary cache files
    exports/              # Bot-level exports
    uploads/              # Bot-level uploads
    users/                # Per-user storage (.gbusers)
      user@email.com/
        papers/
          current/        # Working documents
          named/          # Saved documents
        uploads/          # User uploads
        exports/          # User exports
        settings/         # User preferences
      +1234567890/
        papers/
        uploads/
        exports/
        settings/
```

## Summary

The .gbdrive storage system provides the foundation for all file-based operations in General Bots. Through S3-compatible object storage, organized bucket structures, automatic synchronization, and deep integration with other components, it delivers reliable file management that supports both development workflows and production operation.

The `.gbusers` folder structure enables personalized storage for each user, supporting applications like Paper that require persistent document storage. By organizing user data under their email or phone identifier, the system maintains clear separation while enabling powerful per-user features.

Understanding how storage works helps you organize bot content effectively and leverage the automatic capabilities the system provides.