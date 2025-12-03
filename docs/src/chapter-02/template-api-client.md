# API Client Template (api-client.gbai)

A General Bots template demonstrating REST API integration patterns for connecting to external services and data sources.

---

## Overview

The API Client template provides examples and patterns for integrating General Bots with external REST APIs. It includes examples for weather services, Microsoft Partner Center integration, and general HTTP request patterns that can be adapted for any API.

## Features

- **REST API Integration** - GET, POST, PUT, DELETE request patterns
- **Authentication** - OAuth, Bearer tokens, API keys
- **Header Management** - Custom headers for API requirements
- **Pagination Support** - Handle paginated API responses
- **Data Synchronization** - Sync external data to local tables
- **Scheduled Jobs** - Automated API polling and sync

---

## Package Structure

```
api-client.gbai/
├── api-client.gbdialog/
│   ├── climate.bas              # Weather API example
│   └── msft-partner-center.bas  # Microsoft Partner Center integration
└── api-client.gbot/
    └── config.csv               # Bot configuration
```

## Scripts

| File | Description |
|------|-------------|
| `climate.bas` | Weather API tool for getting current conditions |
| `msft-partner-center.bas` | Full Microsoft Partner Center billing sync |

---

## Basic API Patterns

### Simple GET Request

```basic
' Get data from an API
response = GET "https://api.example.com/data"

IF response THEN
    TALK "Data received: " + response.value
ELSE
    TALK "Failed to fetch data"
END IF
```

### POST Request with Body

```basic
' Send data to an API
SET HEADER "Content-Type" AS "application/json"

payload = {"name": "John", "email": "john@example.com"}
response = POST "https://api.example.com/users", payload

IF response.id THEN
    TALK "User created with ID: " + response.id
END IF
```

### PUT Request for Updates

```basic
' Update existing resource
SET HEADER "Content-Type" AS "application/json"

updates = {"status": "active", "role": "admin"}
response = PUT "https://api.example.com/users/123", updates

TALK "User updated: " + response.status
```

### DELETE Request

```basic
' Delete a resource
response = DELETE "https://api.example.com/users/123"

IF response.deleted THEN
    TALK "User deleted successfully"
END IF
```

---

## Authentication Patterns

### API Key Authentication

```basic
SET HEADER "X-API-Key" AS "your-api-key-here"

response = GET "https://api.example.com/protected-resource"
```

### Bearer Token Authentication

```basic
SET HEADER "Authorization" AS "Bearer " + access_token

response = GET "https://api.example.com/user/profile"
```

### OAuth 2.0 Token Exchange

```basic
' Get OAuth token
SET HEADER "Content-Type" AS "application/x-www-form-urlencoded"

tokenResponse = POST "https://auth.example.com/oauth/token", 
    "grant_type=client_credentials&client_id=" + clientId + 
    "&client_secret=" + clientSecret

access_token = tokenResponse.access_token

' Use token for API calls
SET HEADER "Authorization" AS "Bearer " + access_token
data = GET "https://api.example.com/resources"
```

### Basic Authentication

```basic
credentials = BASE64(username + ":" + password)
SET HEADER "Authorization" AS "Basic " + credentials

response = GET "https://api.example.com/secure-endpoint"
```

---

## Weather API Example

The `climate.bas` tool demonstrates a simple API integration:

```basic
PARAM location AS "The city and state, e.g. San Francisco, CA"
PARAM unit AS "celsius", "fahrenheit"

DESCRIPTION "Get the current weather in a given location"

' Implementation would call weather API
' response = GET "https://api.weather.com/current?location=" + location

RETURN weather_info
```

### Sample Conversation

```
User: What's the weather in New York?

Bot:  [Calls climate tool with location="New York"]
      It's currently 72°F and sunny in New York, NY.
      
      Today's forecast:
      🌡️ High: 78°F / Low: 65°F
      💧 Humidity: 45%
      💨 Wind: 8 mph NW

User: What about São Paulo in celsius?

Bot:  [Calls climate tool with location="São Paulo", unit="celsius"]
      It's currently 24°C and partly cloudy in São Paulo, Brazil.
      
      Today's forecast:
      🌡️ High: 28°C / Low: 19°C
      💧 Humidity: 62%
      💨 Wind: 12 km/h SE
```

---

## Microsoft Partner Center Integration

The `msft-partner-center.bas` demonstrates a complex enterprise API integration:

### Features

- OAuth token authentication with Azure AD
- Multi-resource synchronization (Customers, Subscriptions, Billing)
- Scheduled execution
- Pagination handling
- Database table management

### Configuration

```basic
' Required parameters
tenantId = GET ENV "AZURE_TENANT_ID"
clientId = GET ENV "AZURE_CLIENT_ID"
clientSecret = GET ENV "AZURE_CLIENT_SECRET"
host = "https://api.partnercenter.microsoft.com"
```

### Scheduled Sync

```basic
SET SCHEDULE "1 * * * * *"  ' Run periodically

' Set required headers
SET HEADER "MS-Contract-Version" AS "v1"
SET HEADER "MS-CorrelationId" AS UUID()
SET HEADER "MS-RequestId" AS UUID()
SET HEADER "MS-PartnerCenter-Application" AS "General Bots"
SET HEADER "X-Locale" AS "en-US"
```

### Sync Customers and Subscriptions

```basic
SET PAGE MODE "none"
customers = GET host + "/v1/customers?size=20000"

MERGE "Customers" WITH customers.items BY "Id"

FOR EACH customer IN customers
    subs = GET host + "/v1/customers/" + customer.id + "/subscriptions"
    MERGE "Subscriptions" WITH subs.items BY "Id"
END FOR
```

### Billing Data Sync

```basic
SET PAGE MODE "auto"
billingItems = GET host + "/v1/invoices/unbilled/lineitems" + 
    "?provider=onetime&invoicelineitemtype=usagelineitems&currencycode=USD"

FOR EACH item IN billingItems
    SAVE "Billing", item.alternateId, item.customerId, item.productName,
         item.quantity, item.unitPrice, item.subtotal, item.chargeStartDate
END FOR
```

### Table Definitions

```basic
TABLE Billing
    CustomerId Customers
    ResourceGroup string(200)
    CustomerName string(400)
    ProductName string(400)
    Quantity double
    UnitPrice double
    Subtotal double
    ChargeStartDate date
    ChargeEndDate date
END TABLE

TABLE Customers
    TenantId guid
    CompanyName string(100)
    Id guid
END TABLE

TABLE Subscriptions
    CustomerId Customers
    Id guid
    OfferName string(50)
END TABLE
```

---

## Custom API Integration

### Creating Your Own API Client

```basic
' my-api-client.bas
PARAM resource AS STRING LIKE "users" DESCRIPTION "API resource to fetch"
PARAM filters AS STRING LIKE "status=active" DESCRIPTION "Query filters" OPTIONAL

DESCRIPTION "Fetch data from custom API"

' Configuration
api_base = GET ENV "MY_API_BASE_URL"
api_key = GET ENV "MY_API_KEY"

' Set authentication
SET HEADER "Authorization" AS "Bearer " + api_key
SET HEADER "Content-Type" AS "application/json"

' Build URL
url = api_base + "/" + resource
IF filters THEN
    url = url + "?" + filters
END IF

' Make request
response = GET url

IF response.error THEN
    RETURN {"success": false, "error": response.error}
END IF

RETURN {"success": true, "data": response.data, "count": UBOUND(response.data)}
```

### Handling Pagination

```basic
' paginated-fetch.bas
PARAM endpoint AS STRING DESCRIPTION "API endpoint"

DESCRIPTION "Fetch all pages from a paginated API"

all_results = []
page = 1
has_more = true

DO WHILE has_more
    response = GET endpoint + "?page=" + page + "&per_page=100"
    
    IF response.data THEN
        all_results = MERGE all_results, response.data
        
        IF UBOUND(response.data) < 100 THEN
            has_more = false
        ELSE
            page = page + 1
        END IF
    ELSE
        has_more = false
    END IF
LOOP

RETURN all_results
```

### Error Handling with Retry

```basic
' api-with-retry.bas
PARAM url AS STRING DESCRIPTION "API URL to call"
PARAM max_retries AS INTEGER LIKE 3 DESCRIPTION "Maximum retry attempts"

DESCRIPTION "API call with automatic retry on failure"

retries = 0
success = false

DO WHILE retries < max_retries AND NOT success
    response = GET url
    
    IF response.error THEN
        retries = retries + 1
        WAIT retries * 2  ' Exponential backoff
    ELSE
        success = true
    END IF
LOOP

IF success THEN
    RETURN response
ELSE
    RETURN {"error": "Max retries exceeded", "last_error": response.error}
END IF
```

---

## Configuration

Configure in `api-client.gbot/config.csv`:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `API Base URL` | Default API endpoint | `https://api.example.com` |
| `API Timeout` | Request timeout in seconds | `30` |
| `Retry Count` | Number of retry attempts | `3` |
| `Log Requests` | Enable request logging | `true` |

### Environment Variables

Store sensitive values as environment variables:

```bash
MY_API_KEY=your-api-key
MY_API_SECRET=your-secret
AZURE_TENANT_ID=your-tenant-id
AZURE_CLIENT_ID=your-client-id
AZURE_CLIENT_SECRET=your-client-secret
```

Access in BASIC:

```basic
api_key = GET ENV "MY_API_KEY"
```

---

## Common HTTP Headers

| Header | Purpose | Example |
|--------|---------|---------|
| `Content-Type` | Request body format | `application/json` |
| `Accept` | Expected response format | `application/json` |
| `Authorization` | Authentication | `Bearer token` |
| `X-API-Key` | API key auth | `your-key` |
| `User-Agent` | Client identification | `GeneralBots/1.0` |

---

## Best Practices

1. **Secure credentials** - Never hardcode API keys; use environment variables
2. **Handle errors** - Always check for error responses
3. **Rate limiting** - Respect API rate limits with delays
4. **Pagination** - Handle paginated responses properly
5. **Logging** - Log API calls for debugging
6. **Timeouts** - Set appropriate timeout values
7. **Retries** - Implement retry logic for transient failures
8. **Caching** - Cache responses when appropriate

---

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| 401 Unauthorized | Invalid credentials | Check API key/token |
| 403 Forbidden | Missing permissions | Verify API access rights |
| 404 Not Found | Wrong endpoint | Verify URL path |
| 429 Too Many Requests | Rate limited | Add delays between requests |
| 500 Server Error | API issue | Retry with backoff |
| Timeout | Slow API | Increase timeout setting |

---

## Use Cases

- **Data Synchronization** - Sync data from external systems
- **Service Integration** - Connect to SaaS platforms
- **Automation** - Automate cross-system workflows
- **Monitoring** - Poll external systems for changes
- **Reporting** - Aggregate data from multiple APIs

---

## Related Templates

- [Public APIs](./template-public-apis.md) - Pre-built integrations for public APIs
- [Bling ERP](./template-bling.md) - ERP API integration example
- [LLM Server](./template-llm-server.md) - Building your own API endpoints
- [CRM](./template-crm.md) - CRM with external API sync

---

## See Also

- [Templates Reference](./templates.md) - Full template list
- [Template Samples](./template-samples.md) - Example conversations
- [gbdialog Reference](../chapter-06-gbdialog/README.md) - BASIC scripting guide