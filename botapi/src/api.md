# API Package - RESTful API Endpoints

## Purpose
Exposes RESTful API endpoints for various system functions. Provides a unified interface for accessing botserver features programmatically.

## Key Files
- **database.rs**: Database operations API
- **editor.rs**: Code editor integration API
- **git.rs**: Git repository management API
- **mod.rs**: Module entry point and route configuration
- **terminal.rs**: Terminal access API

## Features
- **Database Operations**: Query, insert, update, delete operations
- **Code Editor**: File manipulation, syntax highlighting, code execution
- **Git Management**: Repository operations (clone, commit, push, pull)
- **Terminal Access**: Command execution and output streaming
- **API Versioning**: Semantic versioning support
- **Rate Limiting**: API request rate control

## API Endpoint Structure

### Database API
```rust
// GET /api/database/query
async fn execute_query(query: QueryRequest) -> Result<QueryResponse, ApiError> {
    // Implementation
}

// POST /api/database/insert
async fn insert_data(data: InsertRequest) -> Result<InsertResponse, ApiError> {
    // Implementation
}

// PUT /api/database/update
async fn update_data(update: UpdateRequest) -> Result<UpdateResponse, ApiError> {
    // Implementation
}
```

### Editor API
```rust
// GET /api/editor/file
async fn get_file_content(path: String) -> Result<FileContent, ApiError> {
    // Implementation
}

// POST /api/editor/file
async fn save_file_content(path: String, content: String) -> Result<(), ApiError> {
    // Implementation
}

// DELETE /api/editor/file
async fn delete_file(path: String) -> Result<(), ApiError> {
    // Implementation
}
```

### Git API
```rust
// POST /api/git/commit
async fn commit_changes(commit: CommitRequest) -> Result<CommitResponse, ApiError> {
    // Implementation
}

// POST /api/git/push
async fn push_changes(remote: String) -> Result<(), ApiError> {
    // Implementation
}

// POST /api/git/pull
async fn pull_changes(remote: String) -> Result<(), ApiError> {
    // Implementation
}
```

## Request/Response Format
All API endpoints follow standard RESTful conventions:
- **Request**: JSON payload with proper validation
- **Response**: JSON with status codes and error details
- **Errors**: Structured error responses with sanitized messages

## Security
- All endpoints require proper authentication
- API keys or bearer tokens for authentication
- Rate limiting per endpoint and user
- CSRF protection for state-changing operations

## Error Handling
Use `ApiError` type with sanitized messages. Errors are logged with context but returned with minimal information to clients.

## Testing
API endpoints are tested with integration tests:
- Endpoint validation tests
- Error handling tests
- Rate limiting tests
- Authentication tests