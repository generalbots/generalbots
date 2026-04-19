# WEATHER / FORECAST Keywords

Get weather information for any location using OpenWeatherMap API.

## WEATHER

```basic
result = WEATHER "London"
TALK result
```

Returns current conditions: temperature, humidity, wind, visibility.

## FORECAST

```basic
result = FORECAST "Paris", 5
TALK result
```

Returns multi-day forecast with high/low temps and rain chance.

## Configuration

Add to `config.csv`:

```csv
weather-api-key,your-openweathermap-api-key
```

Get a free API key at [openweathermap.org](https://openweathermap.org/api).

## See Also

- [Weather API Integration](../appendix-external-services/weather.md) - Full documentation