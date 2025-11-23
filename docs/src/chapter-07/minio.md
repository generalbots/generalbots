# Drive Integration

The drive component provides S3-compatible object storage for BotServer, storing bot packages, documents, and user files.

## Overview

BotServer uses the drive component as its primary storage backend for:
- Bot packages (`.gbai` directories)
- Knowledge base documents (`.gbkb` files)
- Configuration files (`config.csv`)
- Media and attachments
- User-uploaded content

## Configuration

The drive is configured through environment variables that are automatically generated during bootstrap:

- `DRIVE_SERVER` - Drive endpoint URL (default: `http://localhost:9000`)
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

When deploying a bot package, BotServer automatically:
1. Creates a bucket if it doesn't exist
2. Uploads all package files
3. Maintains directory structure
4. Preserves file permissions

### Real-time Synchronization

The bot monitors its bucket for changes:
- Configuration updates trigger automatic reload
- New knowledge base files are indexed immediately
- Deleted files are removed from the index

### Drive Monitor

The `DriveMonitor` service watches for changes in drive storage:
- Detects configuration updates
- Triggers bot reloads
- Syncs local cache with drive

## Bootstrap Integration

During bootstrap, BotServer:

### 1. Installation
- Installs the drive binary if not present
- Configures with generated credentials
- Creates data directories
- Uploads template files to drive

### 2. Knowledge Base Storage

Knowledge base files are:
- Uploaded to drive buckets
- Indexed for vector search
- Cached locally for performance

### 3. File Retrieval

The BASIC `GET` keyword can retrieve files from drive:

```basic
content = GET "knowledge.gbkb/document.pdf"
```

This retrieves files from the bot's bucket in drive storage.

## Media Handling

The multimedia handler uses drive for:
- Storing uploaded images
- Serving media files
- Managing attachments
- Processing thumbnails

## Console Integration

The built-in console provides a file browser for drive:

```
/media/                 # Browse uploaded media
/files/{bot}/          # Browse bot files
/download/{bot}/{file} # Download specific file
```

## AWS SDK Configuration

BotServer uses the AWS SDK S3 client configured for drive:

```rust
let config = aws_config::from_env()
    .endpoint_url(&drive_endpoint)
    .region("us-east-1")
    .load()
    .await;
```

This is configured with `force_path_style(true)` for compatibility with S3-compatible storage.

## Deployment Modes

### Cloud Storage

While the drive typically runs locally alongside BotServer, it can be configured to use:
- Remote S3-compatible instances
- AWS S3 (change endpoint URL)
- Azure Blob Storage (with S3 compatibility)
- Google Cloud Storage (with S3 compatibility)

### Local Mode

Default mode where drive runs on the same machine:
- Binary downloaded to `{{BIN_PATH}}/drive`
- Data stored in `{{DATA_PATH}}`
- Logs written to `{{LOGS_PATH}}/drive.log`

### Container Mode

Drive can run in a container with mapped volumes for persistent storage.

### External Storage

Configure BotServer to use existing S3-compatible infrastructure by updating the drive configuration.

## Security

- Access keys are generated with 32 random bytes
- Secret keys are generated with 64 random bytes
- TLS can be enabled for secure communication
- Bucket policies control access per bot

## Monitoring

- Drive console on port 9001 (optional)
- API endpoint on port 9000
- Health checks via `/health/live`
- Metrics available via `/metrics`

## Troubleshooting

### Check Drive Status

The package manager monitors drive status with:
```
ps -ef | grep drive | grep -v grep
```

### Console Access

Drive console available at `http://localhost:9001` for:
- Bucket management
- User management
- Policy configuration
- Access logs

## Common Issues

1. **Connection Failed**: Check drive is running and ports are accessible
2. **Access Denied**: Verify credentials in environment variables
3. **Bucket Not Found**: Ensure bot deployment completed successfully
4. **Upload Failed**: Check disk space and permissions

### Debug Logging

Enable trace logging to see drive operations:
- File retrieval details
- Bucket operations
- Authentication attempts

## Best Practices

1. **Regular Backups**: Back up drive data directory regularly
2. **Monitor Disk Usage**: Ensure adequate storage space
3. **Access Control**: Use bucket policies to restrict access
4. **Versioning**: Enable object versioning for critical data
5. **Lifecycle Policies**: Configure automatic cleanup for old files