REM General Bots: Weather APIs - Free Weather Data Integration
REM Based on public-apis list - No authentication required

REM ============================================
REM WEATHER KEYWORD - 7Timer! Astro Weather
REM ============================================
PARAM location AS string LIKE "116.39,39.90"
DESCRIPTION "Get 7-day astronomical weather forecast for stargazing and astronomy"

coordinates = SPLIT(location, ",")
lat = coordinates[0]
lon = coordinates[1]

weather_data = GET "http://www.7timer.info/bin/api.pl?lon=" + lon + "&lat=" + lat + "&product=astro&output=json"

result = NEW OBJECT
result.location = location
result.dataseries = weather_data.dataseries
result.init = weather_data.init

TALK "📡 7Timer Astro Weather for " + location + ":"
TALK "Initialization time: " + result.init

FOR EACH forecast IN result.dataseries
    timepoint = forecast.timepoint
    cloudcover = forecast.cloudcover
    seeing = forecast.seeing
    transparency = forecast.transparency

    TALK "⏰ +" + timepoint + "h | ☁️ Cloud: " + cloudcover + " | 👁️ Seeing: " + seeing + " | 🔭 Transparency: " + transparency
END FOR

RETURN result

REM ============================================
REM WEATHER KEYWORD - 7Timer! Civil Weather
REM ============================================
PARAM location AS string LIKE "116.39,39.90"
DESCRIPTION "Get 7-day civil weather forecast with temperature and precipitation"

coordinates = SPLIT(location, ",")
lat = coordinates[0]
lon = coordinates[1]

weather_data = GET "http://www.7timer.info/bin/api.pl?lon=" + lon + "&lat=" + lat + "&product=civil&output=json"

result = NEW OBJECT
result.location = location
result.forecast = weather_data.dataseries

TALK "🌤️ 7-Day Weather Forecast for " + location + ":"

FOR EACH day IN result.forecast
    timepoint = day.timepoint
    temp = day.temp2m
    weather_type = day.weather
    wind = day.wind10m.speed

    TALK "Day " + timepoint + ": " + weather_type + " | 🌡️ " + temp + "°C | 💨 Wind: " + wind + " km/h"
END FOR

RETURN result

REM ============================================
REM WEATHER KEYWORD - Open-Meteo (No API Key)
REM ============================================
PARAM latitude AS number LIKE 52.52
PARAM longitude AS number LIKE 13.41
DESCRIPTION "Get real-time weather data from Open-Meteo (70+ years of historical data available)"

weather_url = "https://api.open-meteo.com/v1/forecast?latitude=" + latitude + "&longitude=" + longitude + "&current_weather=true&hourly=temperature_2m,relativehumidity_2m,precipitation,weathercode,windspeed_10m"

weather_data = GET weather_url

current = weather_data.current_weather

result = NEW OBJECT
result.temperature = current.temperature
result.windspeed = current.windspeed
result.winddirection = current.winddirection
result.weathercode = current.weathercode
result.time = current.time

TALK "🌍 Open-Meteo Weather Report"
TALK "📍 Location: " + latitude + ", " + longitude
TALK "🌡️ Temperature: " + result.temperature + "°C"
TALK "💨 Wind Speed: " + result.windspeed + " km/h"
TALK "🧭 Wind Direction: " + result.winddirection + "°"
TALK "⏰ Updated: " + result.time

RETURN result

REM ============================================
REM WEATHER KEYWORD - MetaWeather Location Search
REM ============================================
PARAM city AS string LIKE "London"
DESCRIPTION "Search for weather location ID by city name"

search_url = "https://www.metaweather.com/api/location/search/?query=" + city

locations = GET search_url

IF UBOUND(locations) > 0 THEN
    result = NEW OBJECT
    result.locations = locations

    TALK "🔍 Found " + UBOUND(locations) + " location(s) for '" + city + "':"

    FOR EACH loc IN locations
        TALK "📍 " + loc.title + " (WOEID: " + loc.woeid + ") - " + loc.location_type
    END FOR

    RETURN result
ELSE
    TALK "❌ No locations found for: " + city
    RETURN NULL
END IF

REM ============================================
REM WEATHER KEYWORD - Rain Viewer Radar Map
REM ============================================
DESCRIPTION "Get available rain radar map timestamps for visualization"

radar_data = GET "https://api.rainviewer.com/public/weather-maps.json"

result = NEW OBJECT
result.host = radar_data.host
result.radar_past = radar_data.radar.past
result.radar_nowcast = radar_data.radar.nowcast

TALK "🌧️ Rain Viewer Radar Data Available"
TALK "📡 Host: " + result.host
TALK "📊 Past radar images: " + UBOUND(result.radar_past)
TALK "🔮 Nowcast images: " + UBOUND(result.radar_nowcast)

IF UBOUND(result.radar_past) > 0 THEN
    latest = result.radar_past[UBOUND(result.radar_past) - 1]
    map_url = result.host + latest.path + "/256/{z}/{x}/{y}/2/1_1.png"
    TALK "🗺️ Latest radar map template: " + map_url
END IF

RETURN result

REM ============================================
REM WEATHER KEYWORD - OpenSenseMap Weather Stations
REM ============================================
PARAM bbox AS string LIKE "7.6,51.2,7.8,51.3"
DESCRIPTION "Get data from personal weather stations (senseBoxes) in a bounding box"

stations_url = "https://api.opensensemap.org/boxes?bbox=" + bbox + "&phenomenon=temperature"

stations = GET stations_url

result = NEW OBJECT
result.count = UBOUND(stations)
result.stations = stations

TALK "🌡️ OpenSenseMap Weather Stations in area: " + bbox
TALK "📊 Found " + result.count + " active stations"

counter = 0
FOR EACH station IN stations
    IF counter < 5 THEN
        TALK "📍 " + station.name + " (" + station.exposure + ")"
        IF station.currentLocation THEN
            TALK "   Location: " + station.currentLocation.coordinates[1] + ", " + station.currentLocation.coordinates[0]
        END IF
    END IF
    counter = counter + 1
END FOR

IF result.count > 5 THEN
    TALK "... and " + (result.count - 5) + " more stations"
END IF

RETURN result

REM ============================================
REM WEATHER KEYWORD - AQICN Air Quality
REM ============================================
PARAM city AS string LIKE "beijing"
DESCRIPTION "Get Air Quality Index data for major cities (Note: API key recommended for production)"

aqi_url = "https://api.waqi.info/feed/" + city + "/?token=demo"

aqi_data = GET aqi_url

IF aqi_data.status = "ok" THEN
    result = NEW OBJECT
    result.aqi = aqi_data.data.aqi
    result.city = aqi_data.data.city.name
    result.time = aqi_data.data.time.s

    aqi_level = ""
    IF result.aqi <= 50 THEN
        aqi_level = "Good 😊"
    ELSE IF result.aqi <= 100 THEN
        aqi_level = "Moderate 😐"
    ELSE IF result.aqi <= 150 THEN
        aqi_level = "Unhealthy for Sensitive Groups 😷"
    ELSE IF result.aqi <= 200 THEN
        aqi_level = "Unhealthy 😨"
    ELSE IF result.aqi <= 300 THEN
        aqi_level = "Very Unhealthy 🤢"
    ELSE
        aqi_level = "Hazardous ☠️"
    END IF

    TALK "🌫️ Air Quality Index for " + result.city
    TALK "📊 AQI: " + result.aqi + " - " + aqi_level
    TALK "⏰ Updated: " + result.time

    RETURN result
ELSE
    TALK "❌ Could not fetch AQI data for: " + city
    RETURN NULL
END IF

REM ============================================
REM WEATHER KEYWORD - Get Weather Icon
REM ============================================
PARAM condition AS string LIKE "sunny"
DESCRIPTION "Get weather emoji/icon based on condition"

condition_lower = LCASE(condition)
icon = "🌡️"

IF INSTR(condition_lower, "sun") > 0 OR INSTR(condition_lower, "clear") > 0 THEN
    icon = "☀️"
ELSE IF INSTR(condition_lower, "cloud") > 0 THEN
    icon = "☁️"
ELSE IF INSTR(condition_lower, "rain") > 0 THEN
    icon = "🌧️"
ELSE IF INSTR(condition_lower, "snow") > 0 THEN
    icon = "❄️"
ELSE IF INSTR(condition_lower, "storm") > 0 OR INSTR(condition_lower, "thunder") > 0 THEN
    icon = "⛈️"
ELSE IF INSTR(condition_lower, "fog") > 0 OR INSTR(condition_lower, "mist") > 0 THEN
    icon = "🌫️"
ELSE IF INSTR(condition_lower, "wind") > 0 THEN
    icon = "💨"
END IF

RETURN icon
