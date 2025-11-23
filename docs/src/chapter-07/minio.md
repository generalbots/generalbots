# MinIO Drive Integration

MinIO provides S3-compatible object storage for BotServer, storing bot packages, documents, and user files.

## Overview

BotServer uses MinIO as its primary storage backend for:
- Bot packages (`.gbai` directories)
- Knowledge base documents (`.gbkb` files)
- Configuration files (`config.csv`)
- Media and attachments
- User-uploaded content

## Configuration

MinIO is configured through environment variables that are automatically generated during bootstrap:

- `DRIVE_SERVER` - MinIO endpoint URL (default: `http://localhost:9000`)
- `DRIVE_ACCESSKEY` - Access key for authentication
- `DRIVE_SECRET` - Secret key for authentication

These credentials are auto-generated with secure random values during the bootstrap process.

## Storage Structure

### Bucket Organization

Each bot gets its own bucket named after the bot package:

```
announcements.gbai/     # Bucket for announcements bot
├── announcements.gbdialog/
│   ├── start.bas
│   └── auth.bas
├── announcements.gbkb/
│   └── documents/
└── announcements.gbot/
    └── config.csv
```

### Bucket Naming Convention

- Bot buckets: `{bot-name}.gbai`
- Media bucket: `botserver-media`
- Each bucket contains the complete bot package structure

## Features

### Automatic Upload

During bootstrap, BotServer automatically:
1. Creates buckets for each bot in `templates/`
2. Uploads all bot files to their respective buckets
3. Maintains the directory structure within buckets

### File Operations

The system provides file operations through the drive client:

- **Get Object**: Retrieve files from buckets
- **Put Object**: Store files in buckets
- **List Objects**: Browse bucket contents
- **Create Bucket**: Initialize new storage buckets

### Drive Monitor

The `DriveMonitor` service watches for changes in MinIO storage:
- Detects configuration updates
- Monitors document additions
- Triggers re-indexing when knowledge base files change
- Broadcasts theme changes from `config.csv` updates

## Integration Points

### 1. Bootstrap Process

During initialization, the bootstrap manager:
- Installs MinIO if not present
- Configures with generated credentials
- Creates buckets for each bot template
- Uploads template files to MinIO

### 2. Knowledge Base Storage

Documents in `.gbkb` directories are:
- Uploaded to MinIO buckets
- Indexed for vector search
- Retrieved on-demand for context

### 3. GET Keyword

The BASIC `GET` keyword can retrieve files from MinIO:

```basic
let data = GET "documents/policy.pdf";
```

This retrieves files from the bot's bucket in MinIO.

### 4. Media Handler

The multimedia handler uses MinIO for:
- Storing uploaded images
- Saving audio recordings
- Managing video files
- Serving media content

## Console Interface

The built-in console provides a file browser for MinIO:

- Browse buckets and folders
- View and edit files
- Navigate the storage hierarchy
- Real-time file operations

### File Tree Navigation

The console file tree shows:
- 🤖 Bot packages (`.gbai` buckets)
- 📦 Other storage buckets
- 📁 Folders within buckets
- 📄 Individual files

## Performance Considerations

### Path-Style Access

BotServer uses path-style bucket access:
```
http://localhost:9000/bucket-name/object-key
```

This is configured with `force_path_style(true)` for compatibility with MinIO.

### Connection Pooling

The AWS SDK S3 client manages connection pooling automatically for efficient operations.

### Local vs Remote

While MinIO typically runs locally alongside BotServer, it can be configured to use:
- Remote MinIO instances
- AWS S3 (change endpoint URL)
- Other S3-compatible storage

## Deployment Modes

### Local Installation

Default mode where MinIO runs on the same machine:
- Binary downloaded to `{{BIN_PATH}}/minio`
- Data stored in `{{DATA_PATH}}`
- Logs written to `{{LOGS_PATH}}/minio.log`

### Container Mode

MinIO can run in a container with mapped volumes for persistent storage.

### External Storage

Configure BotServer to use existing MinIO or S3 infrastructure by updating the drive configuration.

## Security

### Access Control

- Credentials are generated with cryptographically secure random values
- Access keys are at least 20 characters
- Secret keys are at least 40 characters

### Network Security

- MinIO console on port 9001 (optional)
- API endpoint on port 9000
- Can be configured with TLS for production

## Monitoring

### Health Checks

The package manager monitors MinIO status with:
```
ps -ef | grep minio | grep -v grep
```

### Console Access

MinIO console available at `http://localhost:9001` for:
- Bucket management
- Access policy configuration
- Usage statistics
- Performance metrics

## Troubleshooting

### Common Issues

1. **Connection Failed**: Check MinIO is running and ports are accessible
2. **Access Denied**: Verify credentials in environment variables
3. **Bucket Not Found**: Ensure bootstrap completed successfully
4. **Upload Failed**: Check disk space and permissions

### Debug Mode

Enable trace logging to see MinIO operations:
- File retrieval details
- Upload confirmations
- Bucket operations
- Error responses

## Best Practices

1. **Regular Backups**: Back up MinIO data directory regularly
2. **Monitor Disk Usage**: Ensure adequate storage space
3. **Secure Credentials**: Never commit credentials to version control
4. **Use Buckets Wisely**: One bucket per bot for isolation
5. **Clean Up**: Remove unused files to save storage space