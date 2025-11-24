# Storage API

> *⚠️ Note: This API is not yet implemented and is planned for a future release.*

BotServer provides a RESTful API for managing file storage and object management through its S3-compatible storage backend.

## Overview

The Storage API allows you to:
- Upload and download files
- Manage buckets and objects
- Generate presigned URLs
- Handle binary data and documents
- Organize bot assets

## Base URL

```
http://localhost:8080/api/v1/storage
```

## Authentication

All storage API requests require authentication:

```http
Authorization: Bearer <token>
```

## Endpoints

### List Buckets

**GET** `/buckets`

List all available storage buckets.

**Response:**
```json
{
  "buckets": [
    {
      "name": "mybot.gbai",
      "created": "2024-01-15T10:00:00Z",
      "size": 1048576
    }
  ]
}
```

### Create Bucket

**POST** `/buckets`

Create a new storage bucket.

**Request Body:**
```json
{
  "name": "newbot.gbai",
  "region": "us-east-1",
  "versioning": false
}
```

**Response:**
```json
{
  "bucket": "newbot.gbai",
  "created": true,
  "location": "/newbot.gbai"
}
```

### List Objects

**GET** `/buckets/{bucket}/objects`

List objects in a bucket.

**Query Parameters:**
- `prefix` - Filter objects by prefix
- `delimiter` - Delimiter for grouping
- `max_keys` - Maximum number of results (default: 1000)
- `continuation_token` - Pagination token

**Response:**
```json
{
  "objects": [
    {
      "key": "documents/manual.pdf",
      "size": 2048576,
      "last_modified": "2024-01-15T10:30:00Z",
      "etag": "d41d8cd98f00b204e9800998ecf8427e"
    }
  ],
  "is_truncated": false,
  "continuation_token": null
}
```

### Upload Object

**PUT** `/buckets/{bucket}/objects/{key}`

Upload a file to storage.

**Headers:**
- `Content-Type` - MIME type of the file
- `Content-Length` - Size of the file
- `x-amz-meta-*` - Custom metadata

**Request Body:** Binary file data

**Response:**
```json
{
  "bucket": "mybot.gbai",
  "key": "documents/report.pdf",
  "etag": "d41d8cd98f00b204e9800998ecf8427e",
  "version_id": null
}
```

### Download Object

**GET** `/buckets/{bucket}/objects/{key}`

Download a file from storage.

**Headers:**
- `Range` - Partial content request (optional)
- `If-None-Match` - ETag for caching (optional)

**Response:** Binary file data with appropriate headers

### Delete Object

**DELETE** `/buckets/{bucket}/objects/{key}`

Delete an object from storage.

**Response:**
```json
{
  "deleted": true,
  "key": "documents/old-file.pdf"
}
```

### Copy Object

**POST** `/buckets/{bucket}/objects/{key}/copy`

Copy an object to a new location.

**Request Body:**
```json
{
  "source_bucket": "source.gbai",
  "source_key": "file.pdf",
  "destination_bucket": "dest.gbai",
  "destination_key": "copied-file.pdf"
}
```

**Response:**
```json
{
  "copied": true,
  "source": "source.gbai/file.pdf",
  "destination": "dest.gbai/copied-file.pdf"
}
```

### Generate Presigned URL

**POST** `/buckets/{bucket}/objects/{key}/presign`

Generate a presigned URL for temporary access.

**Request Body:**
```json
{
  "operation": "GET",
  "expires_in": 3600,
  "content_type": "application/pdf"
}
```

**Response:**
```json
{
  "url": "http://localhost:9000/mybot.gbai/file.pdf?X-Amz-Algorithm=...",
  "expires_at": "2024-01-15T11:30:00Z"
}
```

### Object Metadata

**HEAD** `/buckets/{bucket}/objects/{key}`

Get object metadata without downloading.

**Response Headers:**
- `Content-Type` - MIME type
- `Content-Length` - File size
- `Last-Modified` - Modification time
- `ETag` - Entity tag
- `x-amz-meta-*` - Custom metadata

### Multipart Upload

**POST** `/buckets/{bucket}/objects/{key}/multipart`

Initiate multipart upload for large files.

**Response:**
```json
{
  "upload_id": "abc123...",
  "bucket": "mybot.gbai",
  "key": "large-file.zip"
}
```

**Upload Part:**
**PUT** `/buckets/{bucket}/objects/{key}/multipart/{uploadId}/{partNumber}`

**Complete Upload:**
**POST** `/buckets/{bucket}/objects/{key}/multipart/{uploadId}/complete`

## Error Responses

### 404 Not Found
```json
{
  "error": "not_found",
  "message": "Object not found",
  "resource": "mybot.gbai/missing.pdf"
}
```

### 409 Conflict
```json
{
  "error": "conflict",
  "message": "Bucket already exists",
  "bucket": "existing.gbai"
}
```

### 507 Insufficient Storage
```json
{
  "error": "insufficient_storage",
  "message": "Storage quota exceeded",
  "quota": 10737418240,
  "used": 10737418240
}
```

## Usage Examples

### Upload File with cURL

```bash
curl -X PUT \
  -H "Authorization: Bearer token123" \
  -H "Content-Type: application/pdf" \
  --data-binary @document.pdf \
  http://localhost:8080/api/v1/storage/buckets/mybot.gbai/objects/docs/manual.pdf
```

### Download File

```bash
curl -X GET \
  -H "Authorization: Bearer token123" \
  http://localhost:8080/api/v1/storage/buckets/mybot.gbai/objects/docs/manual.pdf \
  -o downloaded.pdf
```

### List Objects with Prefix

```bash
curl -X GET \
  -H "Authorization: Bearer token123" \
  "http://localhost:8080/api/v1/storage/buckets/mybot.gbai/objects?prefix=docs/"
```

## Storage Organization

### Recommended Structure

```
bucket/
├── .gbkb/           # Knowledge base files
│   ├── docs/
│   └── data/
├── .gbdialog/       # Dialog scripts
│   ├── scripts/
│   └── tools/
├── .gbtheme/        # Theme assets
│   ├── css/
│   └── images/
└── .gbdrive/        # User uploads
    ├── attachments/
    └── temp/
```

## Quotas and Limits

| Limit | Default Value |
|-------|--------------|
| Max file size | 100 MB |
| Max bucket size | 10 GB |
| Max objects per bucket | 10,000 |
| Presigned URL validity | 7 days |
| Multipart chunk size | 5 MB |

## Performance Tips

1. **Use Multipart Upload** for files > 5MB
2. **Enable Caching** with ETags
3. **Compress Large Files** before upload
4. **Use Presigned URLs** for direct client uploads
5. **Implement Retry Logic** for network failures

## Security Considerations

- All uploads are scanned for malware
- File types are validated
- Presigned URLs expire automatically
- Access control per bucket
- Encryption at rest
- SSL/TLS for transfers

## Related APIs

- [Document Processing API](./document-processing.md)
- [Reports API](./reports-api.md)
- [ML API](./ml-api.md)