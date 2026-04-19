# External APIs

botserver enables integration with external APIs through BASIC scripts, allowing bots to connect with third-party services and extend functionality beyond built-in capabilities.

## Overview

External API integration in botserver is achieved through:
- The `GET` keyword for HTTP/HTTPS requests
- LLM function calling for API interactions
- BASIC script logic for response processing
- Bot memory for storing API credentials and state

## HTTP Requests with GET

The primary method for calling external APIs is the `GET` keyword:

```basic
# Fetch data from an external API
let response = GET "https://api.example.com/data"

# Process the response
let parsed = LLM "Extract the key information from this JSON: " + response
TALK parsed
```

### Supported Protocols

- **HTTP**: Basic unencrypted requests
- **HTTPS**: Secure encrypted requests (recommended)

## API Response Handling

### JSON Responses

Most modern APIs return JSON data:

```basic
let weather = GET "https://api.weather.com/current?city=Seattle"
# Response: {"temp": 65, "conditions": "cloudy"}

let report = LLM "Create a weather report from: " + weather
TALK report
```

### Text Responses

Plain text responses are used directly:

```basic
let quote = GET "https://api.quotes.com/daily"
TALK "Quote of the day: " + quote
```

## Authentication Patterns

### API Key in URL

```basic
let api_key = GET BOT MEMORY "weather_api_key"
let url = "https://api.weather.com/data?key=" + api_key
let data = GET url
```

### Bearer Token (via Headers)

Currently, botserver's GET keyword doesn't support custom headers directly. For APIs requiring Bearer tokens or custom headers, you need to:
1. Use proxy endpoints that add authentication
2. Or use APIs that support key-in-URL authentication

## Common Integration Patterns

### Weather Service

```basic
PARAM city AS string LIKE "Seattle" DESCRIPTION "City for weather"
DESCRIPTION "Gets current weather for a city"

let api_key = GET BOT MEMORY "openweather_key"
let url = "https://api.openweathermap.org/data/2.5/weather?q=" + city + "&appid=" + api_key

let response = GET url
let weather = LLM "Describe the weather based on: " + response
TALK weather
```

### News API

```basic
DESCRIPTION "Fetches latest news headlines"

let api_key = GET BOT MEMORY "newsapi_key"
let url = "https://newsapi.org/v2/top-headlines?country=us&apiKey=" + api_key

let news = GET url
let summary = LLM "Summarize the top 3 news stories from: " + news
TALK summary
```

### Currency Exchange

```basic
PARAM amount AS number LIKE 100 DESCRIPTION "Amount to convert"
PARAM from_currency AS string LIKE "USD" DESCRIPTION "Source currency"
PARAM to_currency AS string LIKE "EUR" DESCRIPTION "Target currency"

DESCRIPTION "Converts currency using exchange rates"

let url = "https://api.exchangerate-api.com/v4/latest/" + from_currency
let rates = GET url

' Parse rates and calculate conversion
let rate = PARSE_JSON(rates, "rates." + to_currency)
let converted = amount * rate
TALK amount + " " + from_currency + " = " + converted + " " + to_currency
```

## Error Handling

### Network Failures

```basic
let response = GET "https://api.example.com/data"

if (response == "") {
    TALK "Unable to reach the service. Please try again later."
} else {
    # Process successful response
    TALK response
}
```

### API Errors

```basic
let data = GET "https://api.service.com/endpoint"

if (data CONTAINS "error") {
    TALK "The service returned an error. Please check your request."
} else {
    # Process valid data
}
```

## Rate Limiting Considerations

When integrating with external APIs:

1. **Respect Rate Limits**: Most APIs have usage limits
2. **Cache Responses**: Use BOT_MEMORY to store frequently accessed data
3. **Batch Requests**: Combine multiple data needs into single calls
4. **Handle 429 Errors**: Too Many Requests responses

### Caching Pattern

```basic
# Check cache first
let cached = GET BOT MEMORY "weather_cache"
let cache_time = GET BOT MEMORY "weather_cache_time"

let current_time = NOW()
let age = current_time - cache_time

if (cached != "" && age < 3600) {
    # Use cached data (less than 1 hour old)
    TALK cached
} else {
    # Fetch fresh data
    let fresh = GET "https://api.weather.com/current"
    SET BOT MEMORY "weather_cache", fresh
    SET BOT MEMORY "weather_cache_time", current_time
    TALK fresh
}
```

## Security Best Practices

### Credential Storage

```basic
# Store API keys in bot memory, not in scripts
let api_key = GET BOT MEMORY "api_key"

# Never hardcode credentials
# BAD: let key = "sk-1234567890abcdef"
# GOOD: let key = GET BOT MEMORY "api_key"
```

### Input Validation

```basic
PARAM user_input AS string LIKE "Seattle" DESCRIPTION "User provided input"

# Sanitize before using in URLs
let safe_input = REPLACE(user_input, " ", "%20")
let url = "https://api.example.com/search?q=" + safe_input
```

## Limitations

Current limitations for external API integration:

1. **No POST/PUT/DELETE**: Only GET requests supported
2. **No Custom Headers**: Cannot set Authorization headers directly
3. **No Request Body**: Cannot send JSON payloads
4. **Timeout Fixed**: 30-second timeout cannot be configured
5. **No Streaming**: Responses fully buffered before processing

## Workarounds

### For POST Requests

Create a proxy service that:
1. Accepts GET requests
2. Converts to POST internally
3. Returns the response

### For Complex APIs

Use the LLM to:
1. Interpret API responses
2. Extract relevant data
3. Format for user consumption

## Example: Complete API Integration

```basic
PARAM location AS string LIKE "New York" DESCRIPTION "Location to check"
DESCRIPTION "Provides weather and news for a location"

# Weather API
let weather_key = GET BOT MEMORY "weather_api_key"
let weather_url = "https://api.openweathermap.org/data/2.5/weather?q=" + location + "&appid=" + weather_key
let weather = GET weather_url

# News API  
let news_key = GET BOT MEMORY "news_api_key"
let news_url = "https://newsapi.org/v2/everything?q=" + location + "&apiKey=" + news_key
let news = GET news_url

# Present the information
TALK "Here's your local update for " + location + ":"
TALK "Weather: " + weather
TALK "Latest news: " + news
```

## Best Practices

1. **Store Keys Securely**: Use BOT_MEMORY for API credentials
2. **Handle Failures Gracefully**: Always check for empty responses
3. **Cache When Possible**: Reduce API calls and improve response time
4. **Document API Usage**: Comment which APIs your tools depend on
5. **Monitor Usage**: Track API calls to avoid exceeding limits
6. **Use HTTPS**: Always prefer secure connections
7. **Validate Inputs**: Sanitize user inputs before including in URLs

## Summary

While botserver's external API capabilities are currently limited to GET requests, creative use of response processing and bot memory for state management enables integration with many third-party services. For more complex API interactions, consider using proxy services or custom integrations.