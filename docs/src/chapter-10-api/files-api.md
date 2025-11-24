# Files API Reference

Complete file and document management operations including upload, download, copy, move, search, sharing, and synchronization.

## Overview

The Files API provides comprehensive file management capabilities built on top of S3-compatible storage. All file operations support both single files and folders with recursive operations.

**Base Path**: `/files`

## Authentication

All endpoints require authentication. Include session token in headers:

```
Authorization: Bearer <token>
```

## File Operations

### List Files

List files and folders in a bucket or path.

**Endpoint**: `GET /files/list`

**Query Parameters**:
- `bucket` (optional) - Bucket name
- `path` (optional) - Folder path

**Response**:
```json
{
  "success": true,
  "data": [
    {
      "name": "document.pdf",
      "path": "/documents/document.pdf",
      "is_dir": false,
      "size": 1048576,
      "modified": "2024-01-15T10:30:00Z",
      "icon": "📄"
    },
    {
      "name": "images",
      "path": "/images",
      "is_dir": true,
      "size": null,
      "modified": "2024-01-15T09:00:00Z",
      "icon": "📁"
    }
  ]
}
```

**Example**:
```bash
curl -X GET "http://localhost:3000/files/list?bucket=my-bucket&path=/documents" \
  -H "Authorization: Bearer <token>"
```

### Read File

Read file content from storage.

**Endpoint**: `POST /files/read`

**Request Body**:
```json
{
  "bucket": "my-bucket",
  "path": "/documents/file.txt"
}
```

**Response**:
```json
{
  "content": "File content here..."
}
```

**Example**:
```bash
curl -X POST "http://localhost:3000/files/read" \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"bucket":"my-bucket","path":"/file.txt"}'
```

### Get File Contents

Alias for read file with alternative naming.

**Endpoint**: `POST /files/getContents`

Same parameters and response as `/files/read`.

### Write File

Write or update file content.

**Endpoint**: `POST /files/write`

**Request Body**:
```json
{
  "bucket": "my-bucket",
  "path": "/documents/file.txt",
  "content": "New file content"
}
```

**Response**:
```json
{
  "success": true,
  "message": "File written successfully"
}
```

### Save File

Alias for write file.

**Endpoint**: `POST /files/save`

Same parameters and response as `/files/write`.

### Upload File

Upload file to storage.

**Endpoint**: `POST /files/upload`

**Request Body**:
```json
{
  "bucket": "my-bucket",
  "path": "/documents/upload.pdf",
  "content": "base64_encoded_content_or_text"
}
```

**Response**:
```json
{
  "success": true,
  "message": "File uploaded successfully"
}
```

### Download File

Download file from storage.

**Endpoint**: `POST /files/download`

**Request Body**:
```json
{
  "bucket": "my-bucket",
  "path": "/documents/file.pdf"
}
```

**Response**:
```json
{
  "content": "file_content"
}
```

### Copy File

Copy file or folder to another location.

**Endpoint**: `POST /files/copy`

**Request Body**:
```json
{
  "source_bucket": "my-bucket",
  "source_path": "/documents/original.pdf",
  "dest_bucket": "my-bucket",
  "dest_path": "/backup/copy.pdf"
}
```

**Response**:
```json
{
  "success": true,
  "message": "File copied successfully"
}
```

### Move File

Move file or folder to another location.

**Endpoint**: `POST /files/move`

**Request Body**:
```json
{
  "source_bucket": "my-bucket",
  "source_path": "/documents/file.pdf",
  "dest_bucket": "archive-bucket",
  "dest_path": "/archived/file.pdf"
}
```

**Response**:
```json
{
  "success": true,
  "message": "File moved successfully"
}
```

**Note**: Move operation copies the file and then deletes the source.

### Delete File

Delete file or folder.

**Endpoint**: `POST /files/delete`

**Request Body**:
```json
{
  "bucket": "my-bucket",
  "path": "/documents/file.pdf"
}
```

**Response**:
```json
{
  "success": true,
  "message": "Deleted successfully"
}
```

**Note**: If path ends with `/`, all objects with that prefix are deleted (recursive folder deletion).

### Create Folder

Create a new folder.

**Endpoint**: `POST /files/createFolder`

**Request Body**:
```json
{
  "bucket": "my-bucket",
  "path": "/documents",
  "name": "new-folder"
}
```

**Response**:
```json
{
  "success": true,
  "message": "Folder created successfully"
}
```

**Alternative Endpoint**: `POST /files/create-folder` (dash notation)

### List Folder Contents

List contents of a specific folder.

**Endpoint**: `POST /files/dirFolder`

**Request Body**:
```json
{
  "bucket": "my-bucket",
  "path": "/documents"
}
```

**Response**:
```json
[
  {
    "name": "file1.pdf",
    "path": "/documents/file1.pdf",
    "is_dir": false,
    "size": 1024,
    "modified": "2024-01-15T10:30:00Z",
    "icon": "📄"
  }
]
```

## Search and Discovery

### Search Files

Search for files across buckets.

**Endpoint**: `GET /files/search`

**Query Parameters**:
- `bucket` (optional) - Limit search to specific bucket
- `query` (required) - Search term
- `file_type` (optional) - File extension filter (e.g., ".pdf")

**Response**:
```json
[
  {
    "name": "matching-file.pdf",
    "path": "/documents/matching-file.pdf",
    "is_dir": false,
    "size": 2048576,
    "modified": "2024-01-15T10:30:00Z",
    "icon": "📄"
  }
]
```

**Example**:
```bash
curl -X GET "http://localhost:3000/files/search?query=report&file_type=.pdf" \
  -H "Authorization: Bearer <token>"
```

### Recent Files

Get recently modified files.

**Endpoint**: `GET /files/recent`

**Query Parameters**:
- `bucket` (optional) - Filter by bucket

**Response**:
```json
[
  {
    "name": "recent-file.txt",
    "path": "/documents/recent-file.txt",
    "is_dir": false,
    "size": 1024,
    "modified": "2024-01-15T14:30:00Z",
    "icon": "📃"
  }
]
```

**Note**: Returns up to 50 most recently modified files, sorted by modification date descending.

### Favorite Files

List user's favorite files.

**Endpoint**: `GET /files/favorite`

**Response**:
```json
[]
```

**Note**: Currently returns empty array. Favorite functionality to be implemented.

## Sharing and Permissions

### Share Folder

Share folder with other users.

**Endpoint**: `POST /files/shareFolder`

**Request Body**:
```json
{
  "bucket": "my-bucket",
  "path": "/documents/shared",
  "users": ["user1@example.com", "user2@example.com"],
  "permissions": "read-write"
}
```

**Response**:
```json
{
  "share_id": "550e8400-e29b-41d4-a716-446655440000",
  "url": "https://share.example.com/550e8400-e29b-41d4-a716-446655440000",
  "expires_at": "2024-01-22T10:30:00Z"
}
```

### List Shared Files

Get files and folders shared with user.

**Endpoint**: `GET /files/shared`

**Response**:
```json
[]
```

### Get Permissions

Get permissions for file or folder.

**Endpoint**: `GET /files/permissions`

**Query Parameters**:
- `bucket` (required) - Bucket name
- `path` (required) - File/folder path

**Response**:
```json
{
  "bucket": "my-bucket",
  "path": "/documents/file.pdf",
  "permissions": {
    "read": true,
    "write": true,
    "delete": true,
    "share": true
  },
  "shared_with": []
}
```

## Storage Management

### Get Quota

Check storage quota information.

**Endpoint**: `GET /files/quota`

**Response**:
```json
{
  "total_bytes": 100000000000,
  "used_bytes": 45678901234,
  "available_bytes": 54321098766,
  "percentage_used": 45.68
}
```

**Example**:
```bash
curl -X GET "http://localhost:3000/files/quota" \
  -H "Authorization: Bearer <token>"
```

## Synchronization

### Sync Status

Get current synchronization status.

**Endpoint**: `GET /files/sync/status`

**Response**:
```json
{
  "status": "idle",
  "last_sync": "2024-01-15T10:30:00Z",
  "files_synced": 0,
  "bytes_synced": 0
}
```

**Status values**:
- `idle` - No sync in progress
- `syncing` - Sync in progress
- `error` - Sync error occurred
- `paused` - Sync paused

### Start Sync

Start file synchronization.

**Endpoint**: `POST /files/sync/start`

**Response**:
```json
{
  "success": true,
  "message": "Sync started"
}
```

### Stop Sync

Stop file synchronization.

**Endpoint**: `POST /files/sync/stop`

**Response**:
```json
{
  "success": true,
  "message": "Sync stopped"
}
```

## File Icons

Files are automatically assigned icons based on extension:

| Extension | Icon | Type |
|-----------|------|------|
| .bas | ⚙️ | BASIC script |
| .ast | 🔧 | AST file |
| .csv | 📊 | Spreadsheet |
| .gbkb | 📚 | Knowledge base |
| .json | 🔖 | JSON data |
| .txt, .md | 📃 | Text |
| .pdf | 📕 | PDF document |
| .zip, .tar, .gz | 📦 | Archive |
| .jpg, .png, .gif | 🖼️ | Image |
| folder | 📁 | Directory |
| .gbai | 🤖 | Bot package |
| default | 📄 | Generic file |

## Error Handling

Common error responses:

**Service Unavailable**:
```json
{
  "error": "S3 service not available"
}
```
Status: 503

**File Not Found**:
```json
{
  "error": "Failed to read file: NoSuchKey"
}
```
Status: 500

**Invalid UTF-8**:
```json
{
  "error": "File is not valid UTF-8"
}
```
Status: 500

## Best Practices

1. **Large Files**: For files > 5MB, consider chunked uploads
2. **Batch Operations**: Use batch endpoints when operating on multiple files
3. **Path Naming**: Use forward slashes, avoid special characters
4. **Permissions**: Always check permissions before operations
5. **Error Handling**: Implement retry logic for transient failures
6. **Quotas**: Monitor quota usage to prevent storage exhaustion

## Examples

### Upload and Share Workflow

```javascript
// 1. Upload file
const uploadResponse = await fetch('/files/upload', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer token',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    bucket: 'my-bucket',
    path: '/documents/report.pdf',
    content: fileContent
  })
});

// 2. Share with team
const shareResponse = await fetch('/files/shareFolder', {
  method: 'POST',
  headers: {
    'Authorization': 'Bearer token',
    'Content-Type': 'application/json'
  },
  body: JSON.stringify({
    bucket: 'my-bucket',
    path: '/documents',
    users: ['team@example.com'],
    permissions: 'read-write'
  })
});

const { url } = await shareResponse.json();
console.log('Share URL:', url);
```

### Search and Download

```python
import requests

# Search for files
response = requests.get(
    'http://localhost:3000/files/search',
    params={'query': 'report', 'file_type': '.pdf'},
    headers={'Authorization': 'Bearer token'}
)

files = response.json()

# Download first result
if files:
    download_response = requests.post(
        'http://localhost:3000/files/download',
        json={
            'bucket': 'my-bucket',
            'path': files[0]['path']
        },
        headers={'Authorization': 'Bearer token'}
    )
    
    content = download_response.json()['content']
    with open('downloaded.pdf', 'w') as f:
        f.write(content)
```

## Next Steps

- [Document Processing API](./document-processing.md) - Convert and merge documents
- [Storage API](./storage-api.md) - Advanced storage operations
- [Backup API](./backup-api.md) - Backup and restore