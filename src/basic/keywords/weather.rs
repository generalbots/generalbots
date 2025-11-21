use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{error, trace};
use reqwest::Client;
use rhai::{Dynamic, Engine};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;

#[derive(Debug, Deserialize, Serialize)]
struct WeatherData {
    pub location: String,
    pub temperature: String,
    pub condition: String,
    pub forecast: String,
}

/// Fetches weather data from 7Timer! API (free, no auth)
async fn fetch_weather(location: &str) -> Result<WeatherData, Box<dyn std::error::Error>> {
    // Parse location to get coordinates (simplified - in production use geocoding)
    let (lat, lon) = parse_location(location)?;

    // 7Timer! API endpoint
    let url = format!(
        "http://www.7timer.info/bin/api.pl?lon={}&lat={}&product=civil&output=json",
        lon, lat
    );

    trace!("Fetching weather from: {}", url);

    let client = Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(format!("Weather API returned status: {}", response.status()).into());
    }

    let json: serde_json::Value = response.json().await?;

    // Parse 7Timer response
    let dataseries = json["dataseries"]
        .as_array()
        .ok_or("Invalid weather response")?;

    if dataseries.is_empty() {
        return Err("No weather data available".into());
    }

    let current = &dataseries[0];
    let temp = current["temp2m"].as_i64().unwrap_or(0);
    let weather_code = current["weather"].as_str().unwrap_or("unknown");

    let condition = match weather_code {
        "clear" => "Clear sky",
        "pcloudy" => "Partly cloudy",
        "cloudy" => "Cloudy",
        "rain" => "Rain",
        "lightrain" => "Light rain",
        "humid" => "Humid",
        "snow" => "Snow",
        "lightsnow" => "Light snow",
        _ => "Unknown",
    };

    // Build forecast string
    let mut forecast_parts = Vec::new();
    for (i, item) in dataseries.iter().take(3).enumerate() {
        if let (Some(temp), Some(weather)) = (
            item["temp2m"].as_i64(),
            item["weather"].as_str(),
        ) {
            forecast_parts.push(format!("{}h: {}°C, {}", i * 3, temp, weather));
        }
    }
    let forecast = forecast_parts.join("; ");

    Ok(WeatherData {
        location: location.to_string(),
        temperature: format!("{}°C", temp),
        condition: condition.to_string(),
        forecast,
    })
}

/// Simple location parser (lat,lon or city name)
fn parse_location(location: &str) -> Result<(f64, f64), Box<dyn std::error::Error>> {
    // Check if it's coordinates (lat,lon)
    if let Some((lat_str, lon_str)) = location.split_once(',') {
        let lat = lat_str.trim().parse::<f64>()?;
        let lon = lon_str.trim().parse::<f64>()?;
        return Ok((lat, lon));
    }

    // Default city coordinates (extend as needed)
    let coords = match location.to_lowercase().as_str() {
        "london" => (51.5074, -0.1278),
        "paris" => (48.8566, 2.3522),
        "new york" | "newyork" => (40.7128, -74.0060),
        "tokyo" => (35.6762, 139.6503),
        "sydney" => (-33.8688, 151.2093),
        "são paulo" | "sao paulo" => (-23.5505, -46.6333),
        "rio de janeiro" | "rio" => (-22.9068, -43.1729),
        "brasília" | "brasilia" => (-15.8267, -47.9218),
        "buenos aires" => (-34.6037, -58.3816),
        "berlin" => (52.5200, 13.4050),
        "madrid" => (40.4168, -3.7038),
        "rome" => (41.9028, 12.4964),
        "moscow" => (55.7558, 37.6173),
        "beijing" => (39.9042, 116.4074),
        "mumbai" => (19.0760, 72.8777),
        "dubai" => (25.2048, 55.2708),
        "los angeles" | "la" => (34.0522, -118.2437),
        "chicago" => (41.8781, -87.6298),
        "toronto" => (43.6532, -79.3832),
        "mexico city" => (19.4326, -99.1332),
        _ => return Err(format!("Unknown location: {}. Use 'lat,lon' format or known city", location).into()),
    };

    Ok(coords)
}

/// Register WEATHER keyword in Rhai engine
pub fn weather_keyword(
    _state: Arc<AppState>,
    _user_session: UserSession,
    engine: &mut Engine,
) {
    engine.register_custom_syntax(
        &["WEATHER", "$expr$"],
        false,
        move |context, inputs| {
            let location = context.eval_expression_tree(&inputs[0])?;
            let location_str = location.to_string();

            trace!("WEATHER keyword called for: {}", location_str);

            // Create channel for async result
            let (tx, rx) = std::sync::mpsc::channel();

            // Spawn blocking task
            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build();

                let result = if let Ok(rt) = rt {
                    rt.block_on(async {
                        match fetch_weather(&location_str).await {
                            Ok(weather) => {
                                let msg = format!(
                                    "Weather for {}: {} ({}). Forecast: {}",
                                    weather.location,
                                    weather.temperature,
                                    weather.condition,
                                    weather.forecast
                                );
                                Ok(msg)
                            }
                            Err(e) => {
                                error!("Weather fetch failed: {}", e);
                                Err(format!("Could not fetch weather: {}", e))
                            }
                        }
                    })
                } else {
                    Err("Failed to create runtime".to_string())
                };

                let _ = tx.send(result);
            });

            // Wait for result
            match rx.recv() {
                Ok(Ok(weather_msg)) => Ok(Dynamic::from(weather_msg)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    e.into(),
                    rhai::Position::NONE,
                ))),
                Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    "Weather request timeout".into(),
                    rhai::Position::NONE,
                ))),
            }
        },
    );
}
