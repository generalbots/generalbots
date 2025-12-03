# Weather API Integration

The `WEATHER` and `FORECAST` keywords provide real-time weather information and multi-day forecasts using the OpenWeatherMap API.

## Keywords Overview

| Keyword | Purpose |
|---------|---------|
| `WEATHER` | Get current weather conditions for a location |
| `FORECAST` | Get extended weather forecast for multiple days |

## WEATHER

Retrieves current weather conditions for a specified location.

### Syntax

```basic
result = WEATHER location
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `location` | String | City name, optionally with country code (e.g., "London" or "London,UK") |

### Return Value

Returns a formatted string containing:
- Temperature (current and feels-like)
- Weather conditions description
- Humidity percentage
- Wind speed and direction
- Visibility
- Atmospheric pressure

### Example

```basic
' Get current weather for London
weather = WEATHER "London"
TALK weather

' Output:
' Current weather in London:
' 🌡️ Temperature: 15.2°C (feels like 14.5°C)
' ☁️ Conditions: Partly cloudy
' 💧 Humidity: 65%
' 💨 Wind: 3.5 m/s NE
' 🔍 Visibility: 10.0 km
' 📊 Pressure: 1013 hPa
```

## FORECAST

Retrieves an extended weather forecast for multiple days.

### Syntax

```basic
result = FORECAST location, days
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `location` | String | City name, optionally with country code |
| `days` | Integer | Number of days to forecast (1-5, default: 5) |

### Example

```basic
' Get 5-day forecast for Paris
forecast = FORECAST "Paris,FR", 5
TALK forecast

' Output:
' Weather forecast for Paris:
'
' 📅 2024-03-15
' 🌡️ High: 18.5°C, Low: 12.3°C
' ☁️ Scattered clouds
' ☔ Rain chance: 20%
'
' 📅 2024-03-16
' 🌡️ High: 20.1°C, Low: 13.0°C
' ☁️ Clear sky
' ☔ Rain chance: 5%
' ...
```

## Complete Example: Weather Bot

```basic
' weather-assistant.bas
' A conversational weather assistant

TALK "Hello! I can help you with weather information."
TALK "Which city would you like to know about?"

HEAR city

TALK "Would you like the current weather or a forecast?"
HEAR choice

IF INSTR(LOWER(choice), "forecast") > 0 THEN
    TALK "How many days? (1-5)"
    HEAR days
    
    IF NOT IS_NUMERIC(days) THEN
        days = 5
    END IF
    
    result = FORECAST city, days
    TALK result
ELSE
    result = WEATHER city
    TALK result
END IF

TALK "Is there another city you'd like to check?"
```

## Weather-Based Automation

```basic
' weather-alert.bas
' Send alerts based on weather conditions

cities = ["New York", "London", "Tokyo", "Sydney"]

FOR EACH city IN cities
    weather = WEATHER city
    
    ' Check for extreme conditions
    IF INSTR(weather, "storm") > 0 OR INSTR(weather, "heavy rain") > 0 THEN
        SEND MAIL "alerts@company.com", "Weather Alert: " + city, weather
    END IF
NEXT
```

## Daily Weather Report

```basic
' daily-weather.bas
' Generate a daily weather report for multiple locations

locations = ["San Francisco,US", "Austin,US", "Seattle,US"]
report = "☀️ Daily Weather Report\n\n"

FOR EACH loc IN locations
    weather = WEATHER loc
    report = report + weather + "\n\n---\n\n"
NEXT

' Send the compiled report
SEND MAIL "team@company.com", "Daily Weather Update", report
```

## Travel Planning Assistant

```basic
' travel-weather.bas
' Help users plan travel based on weather

TALK "Where are you planning to travel?"
HEAR destination

TALK "When are you planning to go? (Please provide a date)"
HEAR travel_date

' Get forecast for destination
forecast = FORECAST destination, 5
TALK "Here's the weather forecast for " + destination + ":"
TALK forecast

TALK "Based on the forecast, would you like packing suggestions?"
HEAR wants_suggestions

IF LOWER(wants_suggestions) = "yes" THEN
    weather = WEATHER destination
    
    IF INSTR(weather, "rain") > 0 THEN
        TALK "🌂 Don't forget to pack an umbrella and rain jacket!"
    END IF
    
    IF INSTR(weather, "Temperature: 2") > 0 OR INSTR(weather, "Temperature: 3") > 0 THEN
        TALK "🩳 It's warm! Pack light clothing and sunscreen."
    ELSE IF INSTR(weather, "Temperature: 0") > 0 OR INSTR(weather, "Temperature: 1") > 0 THEN
        TALK "🧥 It's cool. Bring a light jacket."
    ELSE
        TALK "🧣 It's cold! Pack warm layers and a coat."
    END IF
END IF
```

## Weather Data Structure

The `WeatherData` object returned internally contains:

| Field | Type | Description |
|-------|------|-------------|
| `location` | String | Resolved location name |
| `temperature` | Float | Current temperature in Celsius |
| `temperature_unit` | String | Temperature unit (°C) |
| `description` | String | Weather condition description |
| `humidity` | Integer | Humidity percentage (0-100) |
| `wind_speed` | Float | Wind speed in m/s |
| `wind_direction` | String | Compass direction (N, NE, E, etc.) |
| `feels_like` | Float | "Feels like" temperature |
| `pressure` | Integer | Atmospheric pressure in hPa |
| `visibility` | Float | Visibility in kilometers |
| `uv_index` | Float (optional) | UV index if available |
| `forecast` | Array | Forecast data (for FORECAST keyword) |

## Forecast Day Structure

Each forecast day contains:

| Field | Type | Description |
|-------|------|-------------|
| `date` | String | Date in YYYY-MM-DD format |
| `temp_high` | Float | Maximum temperature |
| `temp_low` | Float | Minimum temperature |
| `description` | String | Weather conditions |
| `rain_chance` | Integer | Probability of precipitation (0-100%) |

## Configuration

To use the weather keywords, configure your OpenWeatherMap API key in `config.csv`:

| Key | Description | Required |
|-----|-------------|----------|
| `weather-api-key` | OpenWeatherMap API key | Yes |

### Getting an API Key

1. Visit [OpenWeatherMap](https://openweathermap.org/api)
2. Create a free account
3. Navigate to "API Keys" in your dashboard
4. Generate a new API key
5. Add to your bot's `config.csv`:

```csv
weather-api-key,your-api-key-here
```

## Wind Direction Compass

Wind direction is converted from degrees to compass directions:

| Degrees | Direction |
|---------|-----------|
| 0° | N (North) |
| 45° | NE (Northeast) |
| 90° | E (East) |
| 135° | SE (Southeast) |
| 180° | S (South) |
| 225° | SW (Southwest) |
| 270° | W (West) |
| 315° | NW (Northwest) |

## Error Handling

```basic
' Handle weather API errors gracefully
ON ERROR GOTO weather_error

weather = WEATHER "Unknown City XYZ"
TALK weather
END

weather_error:
    TALK "Sorry, I couldn't get weather information for that location."
    TALK "Please check the city name and try again."
END
```

## Rate Limits

The OpenWeatherMap free tier includes:
- 60 calls per minute
- 1,000,000 calls per month

For higher limits, consider upgrading to a paid plan.

## Best Practices

1. **Use country codes**: For accuracy, include country codes (e.g., "Paris,FR" instead of just "Paris").

2. **Cache results**: Weather data doesn't change frequently—consider caching results for 10-15 minutes.

3. **Handle timeouts**: Weather API calls have a 10-second timeout. Handle failures gracefully.

4. **Validate locations**: Check if the location is valid before making API calls.

5. **Localization**: Consider user preferences for temperature units (Celsius vs Fahrenheit).

## Fallback Behavior

If the OpenWeatherMap API is unavailable, the system will:
1. Log the error
2. Attempt a fallback weather service (if configured)
3. Return a user-friendly error message

## Related Keywords

- [GET](../chapter-06-gbdialog/keyword-get.md) - Make custom HTTP requests to weather APIs
- [SET SCHEDULE](../chapter-06-gbdialog/keyword-set-schedule.md) - Schedule regular weather checks
- [SEND MAIL](../chapter-06-gbdialog/keyword-send-mail.md) - Send weather alerts via email
- [SEND SMS](../chapter-06-gbdialog/keyword-sms.md) - Send weather alerts via SMS

## See Also

- [OpenWeatherMap API Documentation](https://openweathermap.org/api)
- [API Tool Generator](../chapter-06-gbdialog/keyword-use-tool.md) - Create custom weather integrations