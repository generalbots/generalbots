// Tests for the weather formatters (#2065), split out of `weather.rs` to keep
// that file under the AGENTS.md size limit.

use super::{degrees_to_compass, format_weather_response, ForecastDay, WeatherData};

fn sample() -> WeatherData {
    WeatherData {
        location: "São Paulo".to_string(),
        temperature: 23.4,
        temperature_unit: "°C".to_string(),
        description: "Partly cloudy".to_string(),
        humidity: 78,
        wind_speed: 4.5,
        wind_direction: "SE".to_string(),
        feels_like: 24.1,
        pressure: 1013,
        visibility: 10.0,
        uv_index: Some(6.0),
        forecast: vec![ForecastDay {
            date: "2026-11-11".to_string(),
            temp_high: 27.0,
            temp_low: 18.0,
            description: "Sunny".to_string(),
            rain_chance: 10,
        }],
    }
}

#[test]
fn compass_covers_all_sixteen_points() {
    assert_eq!(degrees_to_compass(0.0), "N");
    assert_eq!(degrees_to_compass(22.5), "NNE");
    assert_eq!(degrees_to_compass(45.0), "NE");
    assert_eq!(degrees_to_compass(67.5), "ENE");
    assert_eq!(degrees_to_compass(90.0), "E");
    assert_eq!(degrees_to_compass(135.0), "SE");
    assert_eq!(degrees_to_compass(180.0), "S");
    assert_eq!(degrees_to_compass(225.0), "SW");
    assert_eq!(degrees_to_compass(270.0), "W");
    assert_eq!(degrees_to_compass(315.0), "NW");
    assert_eq!(degrees_to_compass(337.5), "NNW");
}

#[test]
fn compass_wraps_around_the_circle() {
    assert_eq!(degrees_to_compass(360.0), "N");
    assert_eq!(degrees_to_compass(359.9), "N");
    assert_eq!(degrees_to_compass(720.0), "N");
}

#[test]
fn compass_rounds_to_the_nearest_point() {
    // 11.2 is closer to N; 11.3 crosses into NNE.
    assert_eq!(degrees_to_compass(11.2), "N");
    assert_eq!(degrees_to_compass(11.3), "NNE");
    // The boundary between ENE and E sits at 78.75.
    assert_eq!(degrees_to_compass(78.7), "ENE");
    assert_eq!(degrees_to_compass(78.8), "E");
}

#[test]
fn the_response_reports_every_field_with_one_decimal() {
    let text = format_weather_response(&sample());

    assert!(text.contains("Current weather in São Paulo"));
    assert!(text.contains("Temperature: 23.4°C (feels like 24.1°C)"));
    assert!(text.contains("Conditions: Partly cloudy"));
    assert!(text.contains("Humidity: 78%"));
    assert!(text.contains("Wind: 4.5 m/s SE"));
    assert!(text.contains("Visibility: 10.0 km"));
    assert!(text.contains("Pressure: 1013 hPa"));
}
