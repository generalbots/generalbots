use crate::shared::models::UserSession;
use crate::shared::state::AppState;
use log::{error, info, trace};
use rhai::{Dynamic, Engine};
use serde::{Deserialize, Serialize};
use std::fmt::Write;
use std::sync::Arc;

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherData {
    pub location: String,
    pub temperature: f32,
    pub temperature_unit: String,
    pub description: String,
    pub humidity: u32,
    pub wind_speed: f32,
    pub wind_direction: String,
    pub feels_like: f32,
    pub pressure: u32,
    pub visibility: f32,
    pub uv_index: Option<f32>,
    pub forecast: Vec<ForecastDay>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ForecastDay {
    pub date: String,
    pub temp_high: f32,
    pub temp_low: f32,
    pub description: String,
    pub rain_chance: u32,
}

pub fn weather_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();

    engine
        .register_custom_syntax(["WEATHER", "$expr$"], false, move |context, inputs| {
            let location = context.eval_expression_tree(&inputs[0])?.to_string();

            trace!(
                "WEATHER command executed: {} for user: {}",
                location,
                user_clone.user_id
            );

            let state_for_task = Arc::clone(&state_clone);
            let user_for_task = user_clone.clone();
            let location_for_task = location;
            let (tx, rx) = std::sync::mpsc::channel();

            std::thread::spawn(move || {
                let rt = tokio::runtime::Builder::new_multi_thread()
                    .worker_threads(2)
                    .enable_all()
                    .build();

                let send_err = if let Ok(rt) = rt {
                    let result = rt.block_on(async move {
                        get_weather(&state_for_task, &user_for_task, &location_for_task).await
                    });
                    tx.send(result).err()
                } else {
                    tx.send(Err("Failed to build tokio runtime".to_string()))
                        .err()
                };

                if send_err.is_some() {
                    error!("Failed to send WEATHER result from thread");
                }
            });

            match rx.recv_timeout(std::time::Duration::from_secs(10)) {
                Ok(Ok(weather_info)) => Ok(Dynamic::from(weather_info)),
                Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    format!("WEATHER failed: {}", e).into(),
                    rhai::Position::NONE,
                ))),
                Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                    "WEATHER request timed out".into(),
                    rhai::Position::NONE,
                ))),
            }
        })
        .expect("valid syntax registration");

    let state_clone2 = Arc::clone(&state);
    let user_clone2 = user;

    engine
        .register_custom_syntax(
            ["FORECAST", "$expr$", ",", "$expr$"],
            false,
            move |context, inputs| {
                let location = context.eval_expression_tree(&inputs[0])?.to_string();
                let days = context
                    .eval_expression_tree(&inputs[1])?
                    .as_int()
                    .unwrap_or(5) as u32;

                trace!(
                    "FORECAST command executed: {} for {} days, user: {}",
                    location,
                    days,
                    user_clone2.user_id
                );

                let state_for_task = Arc::clone(&state_clone2);
                let user_for_task = user_clone2.clone();
                let location_for_task = location;
                let (tx, rx) = std::sync::mpsc::channel();

                std::thread::spawn(move || {
                    let rt = tokio::runtime::Builder::new_multi_thread()
                        .worker_threads(2)
                        .enable_all()
                        .build();

                    let send_err = if let Ok(rt) = rt {
                        let result = rt.block_on(async move {
                            get_forecast(&state_for_task, &user_for_task, &location_for_task, days)
                                .await
                        });
                        tx.send(result).err()
                    } else {
                        tx.send(Err("Failed to build tokio runtime".to_string()))
                            .err()
                    };

                    if send_err.is_some() {
                        error!("Failed to send FORECAST result from thread");
                    }
                });

                match rx.recv_timeout(std::time::Duration::from_secs(10)) {
                    Ok(Ok(forecast_info)) => Ok(Dynamic::from(forecast_info)),
                    Ok(Err(e)) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        format!("FORECAST failed: {}", e).into(),
                        rhai::Position::NONE,
                    ))),
                    Err(_) => Err(Box::new(rhai::EvalAltResult::ErrorRuntime(
                        "FORECAST request timed out".into(),
                        rhai::Position::NONE,
                    ))),
                }
            },
        )
        .expect("valid syntax registration");
}

async fn get_weather(
    state: &AppState,
    _user: &UserSession,
    location: &str,
) -> Result<String, String> {
    let api_key = get_weather_api_key(state)?;

    match fetch_openweathermap_current(&api_key, location).await {
        Ok(weather) => {
            info!("Weather data fetched for {}", location);
            Ok(format_weather_response(&weather))
        }
        Err(e) => {
            error!("OpenWeatherMap API failed: {}", e);

            fetch_fallback_weather(location)
        }
    }
}

async fn get_forecast(
    state: &AppState,
    _user: &UserSession,
    location: &str,
    days: u32,
) -> Result<String, String> {
    let api_key = get_weather_api_key(state)?;

    match fetch_openweathermap_forecast(&api_key, location, days).await {
        Ok(forecast) => {
            info!("Forecast data fetched for {} ({} days)", location, days);
            Ok(format_forecast_response(&forecast))
        }
        Err(e) => {
            error!("Forecast API failed: {}", e);
            Err(format!("Could not get forecast for {}: {}", location, e))
        }
    }
}

async fn fetch_openweathermap_current(
    api_key: &str,
    location: &str,
) -> Result<WeatherData, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.openweathermap.org/data/2.5/weather?q={}&appid={}&units=metric",
        urlencoding::encode(location),
        api_key
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API returned status: {}", response.status()));
    }

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    Ok(WeatherData {
        location: data["name"].as_str().unwrap_or(location).to_string(),
        temperature: data["main"]["temp"].as_f64().unwrap_or(0.0) as f32,
        temperature_unit: "°C".to_string(),
        description: data["weather"][0]["description"]
            .as_str()
            .unwrap_or("Unknown")
            .to_string(),
        humidity: data["main"]["humidity"].as_u64().unwrap_or(0) as u32,
        wind_speed: data["wind"]["speed"].as_f64().unwrap_or(0.0) as f32,
        wind_direction: degrees_to_compass(data["wind"]["deg"].as_f64().unwrap_or(0.0)),
        feels_like: data["main"]["feels_like"].as_f64().unwrap_or(0.0) as f32,
        pressure: data["main"]["pressure"].as_u64().unwrap_or(0) as u32,
        visibility: data["visibility"].as_f64().unwrap_or(0.0) as f32 / 1000.0,
        uv_index: None,
        forecast: Vec::new(),
    })
}

async fn fetch_openweathermap_forecast(
    api_key: &str,
    location: &str,
    days: u32,
) -> Result<WeatherData, String> {
    let client = reqwest::Client::new();
    let url = format!(
        "https://api.openweathermap.org/data/2.5/forecast?q={}&appid={}&units=metric&cnt={}",
        urlencoding::encode(location),
        api_key,
        days * 8
    );

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API returned status: {}", response.status()));
    }

    let data: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let mut forecast_days = Vec::new();
    let mut daily_data: std::collections::HashMap<String, (f32, f32, String, u32)> =
        std::collections::HashMap::new();

    if let Some(list) = data["list"].as_array() {
        for item in list {
            let dt_txt = item["dt_txt"].as_str().unwrap_or("");
            let forecast_date = dt_txt.split(' ').next().unwrap_or("");
            let temp = item["main"]["temp"].as_f64().unwrap_or(0.0) as f32;
            let description = item["weather"][0]["description"]
                .as_str()
                .unwrap_or("Unknown")
                .to_string();
            let rain_chance = (item["pop"].as_f64().unwrap_or(0.0) * 100.0) as u32;

            let entry = daily_data
                .entry(forecast_date.to_string())
                .or_insert_with(|| (temp, temp, description.clone(), rain_chance));

            if temp < entry.0 {
                entry.0 = temp;
            }
            if temp > entry.1 {
                entry.1 = temp;
            }

            if rain_chance > entry.3 {
                entry.3 = rain_chance;
            }
        }
    }

    for (date, (temp_low, temp_high, description, rain_chance)) in daily_data.iter() {
        forecast_days.push(ForecastDay {
            date: date.clone(),
            temp_high: *temp_high,
            temp_low: *temp_low,
            description: description.clone(),
            rain_chance: *rain_chance,
        });
    }

    forecast_days.sort_by(|a, b| a.date.cmp(&b.date));

    Ok(WeatherData {
        location: data["city"]["name"]
            .as_str()
            .unwrap_or(location)
            .to_string(),
        temperature: 0.0,
        temperature_unit: "°C".to_string(),
        description: "Forecast".to_string(),
        humidity: 0,
        wind_speed: 0.0,
        wind_direction: String::new(),
        feels_like: 0.0,
        pressure: 0,
        visibility: 0.0,
        uv_index: None,
        forecast: forecast_days,
    })
}

fn fetch_fallback_weather(location: &str) -> Result<String, String> {
    info!("Using fallback weather for {}", location);

    Ok(format!(
        "Weather information for {} is temporarily unavailable. Please try again later.",
        location
    ))
}

pub fn format_weather_response(weather: &WeatherData) -> String {
    format!(
        "Current weather in {}:\n\
         Temperature: {:.1}{} (feels like {:.1}{})\n\
         Conditions: {}\n\
         Humidity: {}%\n\
         Wind: {:.1} m/s {}\n\
         Visibility: {:.1} km\n\
         Pressure: {} hPa",
        weather.location,
        weather.temperature,
        weather.temperature_unit,
        weather.feels_like,
        weather.temperature_unit,
        weather.description,
        weather.humidity,
        weather.wind_speed,
        weather.wind_direction,
        weather.visibility,
        weather.pressure
    )
}

fn format_forecast_response(weather: &WeatherData) -> String {
    let mut response = format!("Weather forecast for {}:\n\n", weather.location);

    for day in &weather.forecast {
        let _ = write!(
            response,
            " {}\n\
             High: {:.1}°C, Low: {:.1}°C\n\
             {}\n\
             Rain chance: {}%\n\n",
            day.date, day.temp_high, day.temp_low, day.description, day.rain_chance
        );
    }

    response
}

pub fn degrees_to_compass(degrees: f64) -> String {
    let directions = [
        "N", "NNE", "NE", "ENE", "E", "ESE", "SE", "SSE", "S", "SSW", "SW", "WSW", "W", "WNW",
        "NW", "NNW",
    ];
    let index = ((degrees + 11.25) / 22.5) as usize % 16;
    directions[index].to_string()
}

fn get_weather_api_key(_state: &AppState) -> Result<String, String> {
    Err("Weather API key not configured. Please set 'weather-api-key' in config.csv".to_string())
}
