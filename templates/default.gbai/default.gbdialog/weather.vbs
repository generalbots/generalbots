REM General Bots: WEATHER Keyword - Universal Weather Data
REM Free weather API using 7Timer! - No authentication required
REM Can be used by ANY template that needs weather information

PARAM location AS string LIKE "New York"
DESCRIPTION "Get current weather forecast for any city or location"

REM Default coordinates for common cities
lat = 40.7128
lon = -74.0060

REM Parse location to get approximate coordinates
REM In production, use geocoding API
location_lower = LCASE(location)

IF INSTR(location_lower, "new york") > 0 THEN
    lat = 40.7128
    lon = -74.0060
ELSE IF INSTR(location_lower, "london") > 0 THEN
    lat = 51.5074
    lon = -0.1278
ELSE IF INSTR(location_lower, "paris") > 0 THEN
    lat = 48.8566
    lon = 2.3522
ELSE IF INSTR(location_lower, "tokyo") > 0 THEN
    lat = 35.6762
    lon = 139.6503
ELSE IF INSTR(location_lower, "sydney") > 0 THEN
    lat = -33.8688
    lon = 151.2093
ELSE IF INSTR(location_lower, "berlin") > 0 THEN
    lat = 52.5200
    lon = 13.4050
ELSE IF INSTR(location_lower, "madrid") > 0 THEN
    lat = 40.4168
    lon = -3.7038
ELSE IF INSTR(location_lower, "rome") > 0 THEN
    lat = 41.9028
    lon = 12.4964
ELSE IF INSTR(location_lower, "moscow") > 0 THEN
    lat = 55.7558
    lon = 37.6173
ELSE IF INSTR(location_lower, "beijing") > 0 THEN
    lat = 39.9042
    lon = 116.4074
ELSE IF INSTR(location_lower, "mumbai") > 0 THEN
    lat = 19.0760
    lon = 72.8777
ELSE IF INSTR(location_lower, "sao paulo") > 0 OR INSTR(location_lower, "são paulo") > 0 THEN
    lat = -23.5505
    lon = -46.6333
ELSE IF INSTR(location_lower, "mexico city") > 0 THEN
    lat = 19.4326
    lon = -99.1332
ELSE IF INSTR(location_lower, "los angeles") > 0 THEN
    lat = 34.0522
    lon = -118.2437
ELSE IF INSTR(location_lower, "chicago") > 0 THEN
    lat = 41.8781
    lon = -87.6298
ELSE IF INSTR(location_lower, "toronto") > 0 THEN
    lat = 43.6532
    lon = -79.3832
ELSE IF INSTR(location_lower, "buenos aires") > 0 THEN
    lat = -34.6037
    lon = -58.3816
ELSE IF INSTR(location_lower, "cairo") > 0 THEN
    lat = 30.0444
    lon = 31.2357
ELSE IF INSTR(location_lower, "dubai") > 0 THEN
    lat = 25.2048
    lon = 55.2708
ELSE IF INSTR(location_lower, "singapore") > 0 THEN
    lat = 1.3521
    lon = 103.8198
END IF

REM Call Open-Meteo API (free, no key required)
weather_url = "https://api.open-meteo.com/v1/forecast?latitude=" + lat + "&longitude=" + lon + "&current_weather=true&hourly=temperature_2m,precipitation_probability,weathercode&timezone=auto"

weather_data = GET weather_url

IF weather_data.current_weather THEN
    current = weather_data.current_weather

    REM Create result object
    result = NEW OBJECT
    result.location = location
    result.temperature = current.temperature
    result.windspeed = current.windspeed
    result.winddirection = current.winddirection
    result.weathercode = current.weathercode
    result.time = current.time

    REM Interpret weather code
    code = current.weathercode
    condition = "Clear"
    icon = "☀️"

    IF code = 0 THEN
        condition = "Clear sky"
        icon = "☀️"
    ELSE IF code >= 1 AND code <= 3 THEN
        condition = "Partly cloudy"
        icon = "⛅"
    ELSE IF code >= 45 AND code <= 48 THEN
        condition = "Foggy"
        icon = "🌫️"
    ELSE IF code >= 51 AND code <= 67 THEN
        condition = "Rainy"
        icon = "🌧️"
    ELSE IF code >= 71 AND code <= 77 THEN
        condition = "Snowy"
        icon = "❄️"
    ELSE IF code >= 80 AND code <= 82 THEN
        condition = "Rain showers"
        icon = "🌦️"
    ELSE IF code >= 85 AND code <= 86 THEN
        condition = "Snow showers"
        icon = "🌨️"
    ELSE IF code >= 95 AND code <= 99 THEN
        condition = "Thunderstorm"
        icon = "⛈️"
    END IF

    result.condition = condition
    result.icon = icon

    REM Display weather
    TALK icon + " Weather for " + location + ":"
    TALK "🌡️ Temperature: " + result.temperature + "°C"
    TALK "🌤️ Condition: " + condition
    TALK "💨 Wind Speed: " + result.windspeed + " km/h"
    TALK "⏰ Last Updated: " + result.time

    RETURN result
ELSE
    TALK "❌ Could not fetch weather data for: " + location
    TALK "💡 Try: 'London', 'New York', 'Tokyo', 'Paris', etc."
    RETURN NULL
END IF
