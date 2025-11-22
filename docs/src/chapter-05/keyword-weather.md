# WEATHER

Get current weather information for any location.

## Syntax

```basic
WEATHER location
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `location` | String | City name, coordinates, or zip code |

## Description

The `WEATHER` keyword retrieves real-time weather information from weather services. It provides:

- Current temperature
- Weather conditions
- Humidity and pressure
- Wind speed and direction
- Feels-like temperature
- Forecast data

## Examples

### Basic Weather Query
```basic
weather = WEATHER "New York"
TALK weather
```

### Weather with User Location
```basic
city = HEAR "What city are you in?"
current_weather = WEATHER city
TALK "The weather in " + city + " is: " + current_weather
```

### Conditional Weather Responses
```basic
weather_data = WEATHER "London"
temp = EXTRACT_TEMP(weather_data)

IF temp < 10 THEN
    TALK "It's cold in London (" + temp + "°C). Bundle up!"
ELSE IF temp > 25 THEN
    TALK "It's warm in London (" + temp + "°C). Perfect for outdoor activities!"
ELSE
    TALK "Mild weather in London: " + temp + "°C"
END IF
```

### Multiple Location Weather
```basic
cities = ["Tokyo", "Paris", "Sydney", "Cairo"]
FOR EACH city IN cities
    weather = WEATHER city
    TALK city + ": " + weather
    WAIT 1
NEXT
```

## Return Format

The keyword returns a formatted string with weather details:
```
"New York: 72°F (22°C), Partly Cloudy, Humidity: 65%, Wind: 10 mph NW"
```

For programmatic access, use the extended version:
```basic
data = WEATHER_DATA "San Francisco"
' Returns object with properties:
' data.temperature
' data.condition
' data.humidity
' data.wind_speed
' data.wind_direction
' data.pressure
' data.feels_like
```

## Location Formats

Supported location formats:
- City name: `"Paris"`
- City, Country: `"Paris, France"`
- Coordinates: `"48.8566, 2.3522"`
- Zip/Postal code: `"10001"` (US), `"SW1A 1AA"` (UK)
- Airport code: `"LAX"`

## Configuration

Configure weather service in `config.csv`:

```csv
weatherApiKey,your-api-key
weatherProvider,openweather
weatherUnits,metric
weatherLanguage,en
```

Supported providers:
- OpenWeather
- WeatherAPI
- AccuWeather
- DarkSky (legacy)

## Error Handling

```basic
TRY
    weather = WEATHER location
    TALK weather
CATCH "location_not_found"
    TALK "I couldn't find weather for " + location
CATCH "api_error"
    TALK "Weather service is temporarily unavailable"
CATCH "rate_limit"
    TALK "Too many weather requests. Please try again later."
END TRY
```

## Advanced Usage

### Weather Alerts
```basic
alerts = WEATHER_ALERTS "Miami"
IF alerts != "" THEN
    TALK "⚠️ Weather Alert: " + alerts
END IF
```

### Forecast
```basic
' Get 5-day forecast
forecast = WEATHER_FORECAST "Seattle", 5
FOR EACH day IN forecast
    TALK day.date + ": " + day.high + "/" + day.low + " - " + day.condition
NEXT
```

### Historical Weather
```basic
' Get weather for specific date
historical = WEATHER_HISTORY "Chicago", "2024-01-15"
TALK "Weather on Jan 15: " + historical
```

### Weather Comparison
```basic
city1_weather = WEATHER_DATA "Los Angeles"
city2_weather = WEATHER_DATA "New York"

temp_diff = city1_weather.temperature - city2_weather.temperature
TALK "LA is " + ABS(temp_diff) + " degrees " + 
     IF(temp_diff > 0, "warmer", "cooler") + " than NYC"
```

## Caching

Weather data is cached to reduce API calls:
- Current weather: 10 minutes
- Forecast: 1 hour
- Historical: 24 hours

Force refresh with:
```basic
fresh_weather = WEATHER location, refresh=true
```

## Localization

Weather descriptions are localized based on user language:
```basic
SET_LANGUAGE "es"
weather = WEATHER "Madrid"
' Returns: "Madrid: 25°C, Parcialmente nublado..."
```

## Units

Temperature units based on configuration or override:
```basic
' Force Fahrenheit
weather_f = WEATHER "Toronto", units="imperial"

' Force Celsius
weather_c = WEATHER "Toronto", units="metric"
```

## Integration Examples

### Travel Assistant
```basic
destination = HEAR "Where are you traveling to?"
weather = WEATHER destination
TALK "Current weather at " + destination + ": " + weather

IF weather CONTAINS "rain" THEN
    TALK "Don't forget an umbrella!"
ELSE IF weather CONTAINS "snow" THEN
    TALK "Pack warm clothes and boots!"
END IF
```

### Event Planning
```basic
event_date = HEAR "When is your event?"
location = HEAR "Where will it be held?"
forecast = WEATHER_FORECAST location, event_date

IF forecast CONTAINS "rain" OR forecast CONTAINS "storm" THEN
    TALK "Consider an indoor venue - rain is expected"
ELSE
    TALK "Weather looks good for an outdoor event!"
END IF
```

### Agriculture Bot
```basic
farm_location = "Iowa"
weather_data = WEATHER_DATA farm_location

IF weather_data.temperature < 32 THEN
    TALK "Frost warning! Protect sensitive crops"
END IF

IF weather_data.humidity < 30 THEN
    TALK "Low humidity - increase irrigation"
END IF
```

## Performance Tips

1. **Cache responses**: Avoid repeated API calls
2. **Batch requests**: Get multiple locations at once
3. **Use coordinates**: More accurate than city names
4. **Handle timeouts**: Set reasonable timeout values
5. **Rate limit**: Respect API limits

## Limitations

- Requires valid API key
- Subject to rate limits
- Accuracy depends on provider
- Some locations may not be available
- Historical data may be limited

## Related Keywords

- [GET](./keyword-get.md) - Fetch data from URLs
- [SET_SCHEDULE](./keyword-set-schedule.md) - Schedule weather updates
- [FORMAT](./keyword-format.md) - Format weather display

## Implementation

Located in `src/basic/keywords/weather.rs`

Integrates with multiple weather API providers and includes fallback mechanisms.