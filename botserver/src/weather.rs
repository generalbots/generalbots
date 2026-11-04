//! Server-side proxy for the Weather app.
//!
//! The browser used to call Open-Meteo directly. On dual-stack networks where
//! only IPv4 works, Chrome picked the unreachable IPv6 geocoding host and the
//! app showed "Weather is offline" even though the server could reach it. The
//! botserver already has the outbound connection and a timeout policy, so the
//! app talks to this endpoint instead and gets the same Open-Meteo payload.

use std::time::Duration;

use axum::{
    extract::Query,
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};

#[derive(Debug, Deserialize)]
pub struct WeatherQuery {
    pub city: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
}

fn unavailable(message: &str) -> (StatusCode, Json<Value>) {
    log::warn!("weather proxy: {message}");
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({ "error": "weather_unavailable" })),
    )
}

async fn geocode(client: &reqwest::Client, city: &str) -> Result<(f64, f64, String), (StatusCode, Json<Value>)> {
    let url = format!(
        "https://geocoding-api.open-meteo.com/v1/search?name={}&count=1&language=en&format=json",
        urlencoding::encode(city)
    );
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|error| unavailable(&format!("geocoding request failed: {error}")))?;
    if !response.status().is_success() {
        return Err(unavailable(&format!("geocoding returned {}", response.status())));
    }
    let payload: Value = response
        .json()
        .await
        .map_err(|error| unavailable(&format!("geocoding payload invalid: {error}")))?;
    let hit = payload
        .get("results")
        .and_then(Value::as_array)
        .and_then(|items| items.first())
        .ok_or_else(|| {
            (
                StatusCode::NOT_FOUND,
                Json(json!({ "error": "city_not_found" })),
            )
        })?;
    let latitude = hit.get("latitude").and_then(Value::as_f64).ok_or_else(|| unavailable("geocoding returned no latitude"))?;
    let longitude = hit.get("longitude").and_then(Value::as_f64).ok_or_else(|| unavailable("geocoding returned no longitude"))?;
    let name = hit
        .get("name")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(city)
        .to_string();
    Ok((latitude, longitude, name))
}

async fn forecast(
    client: &reqwest::Client,
    latitude: f64,
    longitude: f64,
) -> Result<Value, (StatusCode, Json<Value>)> {
    let url = format!(
        "https://api.open-meteo.com/v1/forecast?latitude={latitude}&longitude={longitude}\
         &current=temperature_2m,relative_humidity_2m,weather_code,wind_speed_10m\
         &daily=weather_code,temperature_2m_max,temperature_2m_min&timezone=auto&forecast_days=7"
    );
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|error| unavailable(&format!("forecast request failed: {error}")))?;
    if !response.status().is_success() {
        return Err(unavailable(&format!("forecast returned {}", response.status())));
    }
    response
        .json()
        .await
        .map_err(|error| unavailable(&format!("forecast payload invalid: {error}")))
}

/// `GET /api/weather?city=São%20Paulo` or `?lat=..&lon=..`.
pub async fn weather(Query(query): Query<WeatherQuery>) -> impl IntoResponse {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
    {
        Ok(client) => client,
        Err(error) => return unavailable(&format!("client build failed: {error}")).into_response(),
    };

    let (latitude, longitude, label) = match (query.lat, query.lon) {
        (Some(latitude), Some(longitude))
            if (-90.0..=90.0).contains(&latitude) && (-180.0..=180.0).contains(&longitude) =>
        {
            (latitude, longitude, "My location".to_string())
        }
        _ => {
            let city = query.city.map(|value| value.trim().to_string()).unwrap_or_default();
            if city.is_empty() || city.len() > 120 {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": "city_required" })),
                )
                    .into_response();
            }
            match geocode(&client, &city).await {
                Ok(result) => result,
                Err(error) => return error.into_response(),
            }
        }
    };

    match forecast(&client, latitude, longitude).await {
        Ok(mut payload) => {
            if let Some(object) = payload.as_object_mut() {
                object.insert("city".to_string(), Value::String(label));
            }
            (StatusCode::OK, Json(payload)).into_response()
        }
        Err(error) => error.into_response(),
    }
}
