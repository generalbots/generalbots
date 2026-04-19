# HTTP & API Operations

This section covers keywords for making HTTP requests and integrating with external APIs. These keywords enable bots to communicate with REST APIs, GraphQL endpoints, SOAP services, and any HTTP-based web service.

---

## Overview

General Bots provides a complete set of HTTP keywords for API integration:

| Keyword | HTTP Method | Purpose |
|---------|-------------|---------|
| [GET](keyword-get.md) | GET | Retrieve data from URLs or files |
| [POST](keyword-post.md) | POST | Create resources, submit data |
| [PUT](keyword-put.md) | PUT | Replace/update entire resources |
| [PATCH](keyword-patch.md) | PATCH | Partial resource updates |
| [DELETE HTTP](keyword-delete-http.md) | DELETE | Remove resources |
| [SET HEADER](keyword-set-header.md) | — | Set request headers |
| [GRAPHQL](keyword-graphql.md) | POST | GraphQL queries and mutations |
| [SOAP](keyword-soap.md) | POST | SOAP/XML web services |

---

## Quick Examples

### REST API Call

```basic
' GET request
data = GET "https://api.example.com/users/123"
TALK "User name: " + data.name

' POST request
result = POST "https://api.example.com/users" WITH
    name = "John",
    email = "john@example.com"
TALK "Created user ID: " + result.id

' PUT request (full update)
PUT "https://api.example.com/users/123" WITH
    name = "John Doe",
    email = "johndoe@example.com",
    status = "active"

' PATCH request (partial update)
PATCH "https://api.example.com/users/123" WITH status = "inactive"

' DELETE request
DELETE HTTP "https://api.example.com/users/123"
```

### With Authentication

```basic
' Set authorization header
SET HEADER "Authorization", "Bearer " + api_token
SET HEADER "Content-Type", "application/json"

' Make authenticated request
result = GET "https://api.example.com/protected/resource"

' Clear headers when done
SET HEADER "Authorization", ""
```

### GraphQL Query

```basic
query = '
    query GetUser($id: ID!) {
        user(id: $id) {
            name
            email
            orders { id total }
        }
    }
'

result = GRAPHQL "https://api.example.com/graphql", query WITH id = "123"
TALK "User: " + result.data.user.name
```

### SOAP Service

```basic
' Call a SOAP web service
request = '
    <GetWeather xmlns="http://weather.example.com">
        <City>New York</City>
    </GetWeather>
'

result = SOAP "https://weather.example.com/service", "GetWeather", request
TALK "Temperature: " + result.Temperature
```

---

## Common Patterns

### API Client Setup

```basic
' Configure API base URL and authentication
api_base = "https://api.myservice.com/v1"
SET HEADER "Authorization", "Bearer " + GET BOT MEMORY "api_token"
SET HEADER "X-API-Version", "2025-01"

' Helper function pattern
' GET users
users = GET api_base + "/users"

' GET specific user
user = GET api_base + "/users/" + user_id

' CREATE user
new_user = POST api_base + "/users", user_data

' UPDATE user
PUT api_base + "/users/" + user_id, updated_data

' DELETE user
DELETE HTTP api_base + "/users/" + user_id
```

### Error Handling

```basic
ON ERROR RESUME NEXT

result = POST "https://api.example.com/orders", order_data

IF ERROR THEN
    PRINT "API Error: " + ERROR_MESSAGE
    TALK "Sorry, I couldn't process your order. Please try again."
ELSE IF result.error THEN
    TALK "Order failed: " + result.error.message
ELSE
    TALK "Order placed! ID: " + result.id
END IF
```

### Retry Logic

```basic
max_retries = 3
retry_count = 0
success = false

WHILE retry_count < max_retries AND NOT success
    ON ERROR RESUME NEXT
    result = POST api_url, data
    
    IF NOT ERROR AND NOT result.error THEN
        success = true
    ELSE
        retry_count = retry_count + 1
        WAIT 2  ' Wait 2 seconds before retry
    END IF
WEND

IF success THEN
    TALK "Request successful!"
ELSE
    TALK "Request failed after " + max_retries + " attempts."
END IF
```

### Pagination

```basic
' Fetch all pages of results
all_items = []
page = 1
has_more = true

WHILE has_more
    result = GET api_base + "/items?page=" + page + "&limit=100"
    
    FOR EACH item IN result.items
        all_items = APPEND(all_items, item)
    NEXT
    
    has_more = result.has_more
    page = page + 1
WEND

TALK "Fetched " + LEN(all_items) + " total items"
```

---

## Request Headers

Common headers you might need to set:

| Header | Purpose | Example |
|--------|---------|---------|
| `Authorization` | API authentication | `Bearer token123` |
| `Content-Type` | Request body format | `application/json` |
| `Accept` | Response format preference | `application/json` |
| `X-API-Key` | API key authentication | `key_abc123` |
| `X-Request-ID` | Request tracking | `req-uuid-here` |

```basic
SET HEADER "Authorization", "Bearer " + token
SET HEADER "Content-Type", "application/json"
SET HEADER "Accept", "application/json"
SET HEADER "X-Request-ID", GUID()
```

---

## Response Handling

### JSON Responses

Most APIs return JSON, automatically parsed:

```basic
result = GET "https://api.example.com/user"

' Access properties directly
TALK "Name: " + result.name
TALK "Email: " + result.email

' Access nested objects
TALK "City: " + result.address.city

' Access arrays
FOR EACH order IN result.orders
    TALK "Order: " + order.id
NEXT
```

### Check Response Status

```basic
result = POST api_url, data

IF result.status = 201 THEN
    TALK "Resource created!"
ELSE IF result.status = 400 THEN
    TALK "Bad request: " + result.error.message
ELSE IF result.status = 401 THEN
    TALK "Authentication failed. Please log in again."
ELSE IF result.status = 404 THEN
    TALK "Resource not found."
ELSE IF result.status >= 500 THEN
    TALK "Server error. Please try again later."
END IF
```

---

## Configuration

Configure HTTP settings in `config.csv`:

```csv
name,value
http-timeout,30
http-retry-count,3
http-retry-delay,1000
http-base-url,https://api.mycompany.com
http-user-agent,GeneralBots/1.0
http-max-redirects,10
http-verify-ssl,true
```

---

## Security Best Practices

1. **Store credentials securely** — Use Vault or environment variables for API keys
2. **Use HTTPS** — Never send credentials over unencrypted connections
3. **Validate responses** — Check status codes and handle errors
4. **Set timeouts** — Prevent hanging on slow APIs
5. **Rate limit** — Respect API rate limits to avoid being blocked
6. **Log requests** — Enable logging for debugging without exposing secrets

```basic
' Good: Token from secure storage
token = GET BOT MEMORY "api_token"
SET HEADER "Authorization", "Bearer " + token

' Bad: Hardcoded token
' SET HEADER "Authorization", "Bearer sk-abc123"  ' NEVER DO THIS
```

---

## See Also

- [GET](keyword-get.md) — Retrieve data
- [POST](keyword-post.md) — Create resources
- [PUT](keyword-put.md) — Update resources
- [PATCH](keyword-patch.md) — Partial updates
- [DELETE HTTP](keyword-delete-http.md) — Delete resources
- [SET HEADER](keyword-set-header.md) — Set request headers
- [GRAPHQL](keyword-graphql.md) — GraphQL operations
- [SOAP](keyword-soap.md) — SOAP web services