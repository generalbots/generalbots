# SET HEADER

The `SET HEADER` keyword configures HTTP request headers for subsequent API calls, enabling authentication, content type specification, and custom headers.

---

## Syntax

```basic
SET HEADER "header-name", "value"
SET HEADER "header-name", ""
```

---

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `header-name` | String | The HTTP header name (e.g., "Authorization") |
| `value` | String | The header value (empty string to clear) |

---

## Description

`SET HEADER` configures headers that will be sent with subsequent HTTP requests (GET, POST, PUT, PATCH, DELETE HTTP). Headers persist until explicitly cleared or the script ends.

Common uses include:
- Setting authentication tokens
- Specifying content types
- Adding API keys
- Setting custom request identifiers
- Configuring accept headers

---

## Examples

### Basic Authentication Header

```basic
' Set Bearer token for API authentication
SET HEADER "Authorization", "Bearer " + api_token

' Make authenticated request
result = GET "https://api.example.com/protected/resource"

' Clear header when done
SET HEADER "Authorization", ""
```

### API Key Header

```basic
' Set API key in custom header
SET HEADER "X-API-Key", api_key

result = POST "https://api.service.com/data" WITH
    query = user_query

SET HEADER "X-API-Key", ""
```

### Multiple Headers

```basic
' Set multiple headers for a request
SET HEADER "Authorization", "Bearer " + token
SET HEADER "Content-Type", "application/json"
SET HEADER "Accept", "application/json"
SET HEADER "X-Request-ID", request_id

result = POST "https://api.example.com/orders" WITH
    product_id = "SKU-001",
    quantity = 5

' Clear all headers
SET HEADER "Authorization", ""
SET HEADER "Content-Type", ""
SET HEADER "Accept", ""
SET HEADER "X-Request-ID", ""
```

### Content Type for Form Data

```basic
' Set content type for form submission
SET HEADER "Content-Type", "application/x-www-form-urlencoded"

result = POST "https://api.legacy.com/submit", form_data

SET HEADER "Content-Type", ""
```

---

## Common Headers

| Header | Purpose | Example Value |
|--------|---------|---------------|
| `Authorization` | Authentication | `Bearer token123` |
| `Content-Type` | Request body format | `application/json` |
| `Accept` | Expected response format | `application/json` |
| `X-API-Key` | API key authentication | `key_abc123` |
| `X-Request-ID` | Request tracking/correlation | `req-uuid-here` |
| `User-Agent` | Client identification | `MyBot/1.0` |
| `Accept-Language` | Preferred language | `en-US` |
| `If-Match` | Conditional update (ETag) | `"abc123"` |
| `If-None-Match` | Conditional fetch | `"abc123"` |

---

## Authentication Patterns

### Bearer Token (OAuth2/JWT)

```basic
' Most common for modern APIs
SET HEADER "Authorization", "Bearer " + access_token

result = GET "https://api.service.com/user/profile"

SET HEADER "Authorization", ""
```

### Basic Authentication

```basic
' Encode credentials as Base64
credentials = BASE64_ENCODE(username + ":" + password)
SET HEADER "Authorization", "Basic " + credentials

result = GET "https://api.legacy.com/data"

SET HEADER "Authorization", ""
```

### API Key in Header

```basic
' API key as custom header
SET HEADER "X-API-Key", api_key

' Or in Authorization header
SET HEADER "Authorization", "Api-Key " + api_key

result = POST "https://api.provider.com/query" WITH
    question = user_input
```

### Custom Token

```basic
' Some APIs use custom authentication schemes
SET HEADER "X-Auth-Token", auth_token
SET HEADER "X-Client-ID", client_id

result = GET "https://api.custom.com/resources"
```

---

## Common Use Cases

### Authenticated API Call

```basic
' Complete authenticated API interaction
SET HEADER "Authorization", "Bearer " + GET BOT MEMORY "api_token"
SET HEADER "Content-Type", "application/json"

result = POST "https://api.crm.com/leads" WITH
    name = customer_name,
    email = customer_email,
    source = "chatbot"

IF result.id THEN
    TALK "Lead created: " + result.id
ELSE
    TALK "Error creating lead: " + result.error
END IF

' Always clean up
SET HEADER "Authorization", ""
SET HEADER "Content-Type", ""
```

### Request Tracing

```basic
' Add request ID for debugging/tracing
request_id = GUID()
SET HEADER "X-Request-ID", request_id
SET HEADER "X-Correlation-ID", session.id

PRINT "Request ID: " + request_id

result = POST "https://api.example.com/process" WITH
    data = payload

SET HEADER "X-Request-ID", ""
SET HEADER "X-Correlation-ID", ""
```

### Conditional Requests

```basic
' Only fetch if resource changed (using ETag)
SET HEADER "If-None-Match", cached_etag

result = GET "https://api.example.com/data"

IF result.status = 304 THEN
    TALK "Data unchanged, using cached version"
ELSE
    ' Process new data
    cached_data = result.data
    cached_etag = result.headers.etag
END IF

SET HEADER "If-None-Match", ""
```

---

## Header Persistence

Headers persist across multiple requests until cleared:

```basic
' Set header once
SET HEADER "Authorization", "Bearer " + token

' Used in all these requests
result1 = GET "https://api.example.com/users"
result2 = GET "https://api.example.com/orders"
result3 = POST "https://api.example.com/actions" WITH action = "process"

' Clear when done with authenticated calls
SET HEADER "Authorization", ""
```

---

## Best Practices

1. **Always clear sensitive headers** — Remove authentication headers after use
2. **Use Vault for tokens** — Never hardcode API keys or tokens
3. **Set Content-Type when needed** — JSON is usually the default
4. **Add request IDs** — Helps with debugging and support requests
5. **Check API documentation** — Header names and formats vary by API

```basic
' Good practice pattern
' 1. Get token from secure storage
token = GET BOT MEMORY "api_token"

' 2. Set headers
SET HEADER "Authorization", "Bearer " + token
SET HEADER "X-Request-ID", GUID()

' 3. Make request
result = GET api_url

' 4. Clear sensitive headers
SET HEADER "Authorization", ""
SET HEADER "X-Request-ID", ""
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

' Token might be expired
SET HEADER "Authorization", "Bearer " + old_token

result = GET "https://api.example.com/protected"

IF result.status = 401 THEN
    ' Token expired, refresh it
    TALK "Refreshing authentication..."
    new_token = REFRESH_TOKEN(refresh_token)
    SET BOT MEMORY "api_token", new_token
    
    SET HEADER "Authorization", "Bearer " + new_token
    result = GET "https://api.example.com/protected"
END IF

SET HEADER "Authorization", ""
```

---

## Configuration

HTTP defaults can be set in `config.csv`:

```csv
name,value
http-timeout,30
http-default-content-type,application/json
http-user-agent,GeneralBots/6.1.0
```

---

## Implementation Notes

- Implemented in Rust under `src/web_automation/http.rs`
- Headers are stored in thread-local storage
- Case-insensitive header names (HTTP standard)
- Special characters in values are properly escaped
- Empty string clears the header

---

## Related Keywords

- [GET](keyword-get.md) — Retrieve data from URLs
- [POST](keyword-post.md) — Create new resources
- [PUT](keyword-put.md) — Replace entire resources
- [PATCH](keyword-patch.md) — Partial resource updates
- [DELETE HTTP](keyword-delete-http.md) — Remove resources
- [GRAPHQL](keyword-graphql.md) — GraphQL operations

---

## Summary

`SET HEADER` configures HTTP headers for API requests. Use it to add authentication tokens, specify content types, and include custom headers. Always clear sensitive headers after use and store credentials securely in Vault rather than hardcoding them. Headers persist until explicitly cleared, so you can set them once for multiple related requests.