# Drive REST API Integration

## Endpoints

### GET /files/list
List files and folders in S3 bucket
Query params: `bucket` (optional), `path` (optional)
Response: `[{ name, path, is_dir, icon }]`

### POST /files/read
Read file content
Body: `{ bucket, path }`
Response: `{ content }`

### POST /files/write
Write file content
Body: `{ bucket, path, content }`
Response: `{ success: true }`

### POST /files/delete
Delete file/folder
Body: `{ bucket, path }`
Response: `{ success: true }`

### POST /files/create-folder
Create new folder
Body: `{ bucket, path, name }`
Response: `{ success: true }`

## Integration

1. Add to main.rs:
```rust
mod drive;

.configure(drive::configure)
```

2. Frontend calls:
```javascript
fetch('/files/list?bucket=mybucket')
fetch('/files/read', { method: 'POST', body: JSON.stringify({ bucket, path }) })
fetch('/files/write', { method: 'POST', body: JSON.stringify({ bucket, path, content }) })
```

## S3 Backend
Uses existing FileTree from ui_tree/file_tree.rs
Wraps S3 operations: list_buckets, list_objects_v2, get_object, put_object, delete_object