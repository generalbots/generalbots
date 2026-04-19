# System Limits & Rate Limiting

General Bots enforces strict system limits to ensure fair resource usage, prevent abuse, and maintain platform stability for all users.

## Overview

All limits are defined in `botlib/src/limits.rs` and enforced throughout the platform. When limits are exceeded, the system returns HTTP 429 (Too Many Requests) with appropriate `Retry-After` headers.

## Resource Limits

### Loop & Recursion Protection

| Limit | Value | Purpose |
|-------|-------|---------|
| `MAX_LOOP_ITERATIONS` | 100,000 | Prevents infinite loops in BASIC scripts |
| `MAX_RECURSION_DEPTH` | 100 | Prevents stack overflow from deep recursion |
| `MAX_SCRIPT_EXECUTION_SECONDS` | 300 | Maximum script runtime (5 minutes) |

### File & Data Limits

| Limit | Value | Purpose |
|-------|-------|---------|
| `MAX_FILE_SIZE_BYTES` | 100 MB | Maximum file size for processing |
| `MAX_UPLOAD_SIZE_BYTES` | 50 MB | Maximum upload size per request |
| `MAX_REQUEST_BODY_BYTES` | 10 MB | Maximum HTTP request body |
| `MAX_STRING_LENGTH` | 10 MB | Maximum string length in scripts |
| `MAX_ARRAY_LENGTH` | 1,000,000 | Maximum array elements |

### Connection Limits

| Limit | Value | Purpose |
|-------|-------|---------|
| `MAX_CONCURRENT_REQUESTS_PER_USER` | 100 | Per-user concurrent request limit |
| `MAX_CONCURRENT_REQUESTS_GLOBAL` | 10,000 | Platform-wide concurrent limit |
| `MAX_WEBSOCKET_CONNECTIONS_PER_USER` | 10 | WebSocket connections per user |
| `MAX_WEBSOCKET_CONNECTIONS_GLOBAL` | 50,000 | Platform-wide WebSocket limit |
| `MAX_DB_CONNECTIONS_PER_TENANT` | 20 | Database connections per tenant |

### API Rate Limits

| Limit | Value | Purpose |
|-------|-------|---------|
| `MAX_API_CALLS_PER_MINUTE` | 1,000 | Requests per user per minute |
| `MAX_API_CALLS_PER_HOUR` | 10,000 | Requests per user per hour |
| `MAX_LLM_REQUESTS_PER_MINUTE` | 60 | LLM API calls per minute |

### LLM & Knowledge Base Limits

| Limit | Value | Purpose |
|-------|-------|---------|
| `MAX_LLM_TOKENS_PER_REQUEST` | 128,000 | Maximum tokens per LLM request |
| `MAX_KB_DOCUMENTS_PER_BOT` | 100,000 | Documents per bot knowledge base |
| `MAX_KB_DOCUMENT_SIZE_BYTES` | 50 MB | Maximum document size for KB |
| `MAX_DB_QUERY_RESULTS` | 10,000 | Maximum query result rows |

### Tenant & Resource Limits

| Limit | Value | Purpose |
|-------|-------|---------|
| `MAX_DRIVE_STORAGE_BYTES` | 10 GB | Storage per tenant |
| `MAX_SESSIONS_PER_USER` | 10 | Concurrent sessions per user |
| `MAX_SESSION_IDLE_SECONDS` | 3,600 | Session timeout (1 hour) |
| `MAX_BOTS_PER_TENANT` | 100 | Bots per tenant |
| `MAX_TOOLS_PER_BOT` | 500 | Tools per bot |
| `MAX_PENDING_TASKS` | 1,000 | Pending automation tasks |

## Rate Limiting Implementation

### Using the Rate Limiter

```rust
use botlib::{RateLimiter, SystemLimits, format_limit_error_response};

let rate_limiter = RateLimiter::new(SystemLimits::default());

async fn handle_request(user_id: &str) -> Response {
    if let Err(limit_error) = rate_limiter.check_rate_limit(user_id).await {
        let (status, body) = format_limit_error_response(&limit_error);
        return (StatusCode::TOO_MANY_REQUESTS, body);
    }
    
    // Process request...
}
```

### Checking Loop Limits in Scripts

```rust
use botlib::{check_loop_limit, MAX_LOOP_ITERATIONS};

let mut iterations = 0;
loop {
    check_loop_limit(iterations, MAX_LOOP_ITERATIONS)?;
    iterations += 1;
    
    // Loop body...
    
    if done {
        break;
    }
}
```

### Response Format

When a limit is exceeded, the API returns:

```json
{
    "error": "rate_limit_exceeded",
    "message": "Limit exceeded for api_calls_minute: 1001 > 1000 (max)",
    "limit_type": "api_calls_minute",
    "current": 1001,
    "maximum": 1000,
    "retry_after_secs": 60
}
```

HTTP Headers:
- `Status: 429 Too Many Requests`
- `Retry-After: 60`
- `X-RateLimit-Limit: 1000`
- `X-RateLimit-Remaining: 0`
- `X-RateLimit-Reset: 1234567890`

## BASIC Script Limits

Scripts written in `.gbdialog` files are automatically protected:

```basic
' This loop is safe - system enforces MAX_LOOP_ITERATIONS
WHILE condition
    ' Loop body
WEND

' FOR loops are also protected
FOR i = 1 TO 1000000
    ' Will stop at MAX_LOOP_ITERATIONS
NEXT
```

If a script exceeds limits:
- Loop iterations: Script terminates with "Maximum iterations exceeded" error
- Execution time: Script terminates after `MAX_SCRIPT_EXECUTION_SECONDS`
- Memory/string size: Script terminates with "Limit exceeded" error

## Customizing Limits

Administrators can customize limits per tenant via environment variables or configuration:

```toml
[limits]
max_api_calls_per_minute = 2000
max_drive_storage_bytes = 21474836480  # 20 GB
max_bots_per_tenant = 200
```

## Best Practices

### For Bot Developers

1. **Avoid unbounded loops** - Always include exit conditions
2. **Paginate queries** - Don't fetch unlimited results
3. **Cache responses** - Reduce API calls with caching
4. **Use webhooks** - Instead of polling, use event-driven patterns
5. **Batch operations** - Combine multiple operations when possible

### For System Administrators

1. **Monitor rate limit hits** - Track 429 responses in analytics
2. **Set appropriate limits** - Balance security with usability
3. **Configure burst allowance** - Use `RATE_LIMIT_BURST_MULTIPLIER` for temporary spikes
4. **Clean up stale entries** - Rate limiter auto-cleans after 2 hours

## Error Handling

```rust
use botlib::{LimitExceeded, LimitType};

match result {
    Err(LimitExceeded { limit_type, current, maximum, retry_after_secs }) => {
        match limit_type {
            LimitType::ApiCallsMinute => {
                // Handle minute rate limit
            }
            LimitType::LoopIterations => {
                // Handle infinite loop detection
            }
            _ => {
                // Handle other limits
            }
        }
    }
    Ok(value) => {
        // Success
    }
}
```

## Security Considerations

- **DDoS Protection**: Rate limits prevent resource exhaustion attacks
- **Abuse Prevention**: Per-user limits prevent single-user abuse
- **Fair Usage**: Ensures resources are shared fairly across all users
- **Cost Control**: LLM token limits prevent unexpected costs

## See Also

- [Security Features](./security-features.md) - Overall security architecture
- [API Endpoints](./api-endpoints.md) - API documentation
- [Compliance Requirements](./compliance-requirements.md) - Regulatory compliance