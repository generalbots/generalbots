# Example: Creating a New gbapp Virtual Crate

This guide walks through creating a new gbapp virtual crate called `analytics` that adds analytics capabilities to BotServer.

## Step 1: Create the Module Structure

Create your gbapp directory in `src/`:

```
src/analytics/              # analytics.gbapp virtual crate
├── mod.rs                  # Module definition
├── keywords.rs             # BASIC keywords
├── services.rs             # Core functionality
├── models.rs               # Data structures
└── tests.rs                # Unit tests
```

## Step 2: Define the Module

**src/analytics/mod.rs**
```rust
//! Analytics gbapp - Provides analytics and reporting functionality
//! 
//! This virtual crate adds analytics keywords to BASIC and provides
//! services for tracking and reporting bot interactions.

pub mod keywords;
pub mod services;
pub mod models;

#[cfg(test)]
mod tests;

use crate::shared::state::AppState;
use std::sync::Arc;

/// Initialize the analytics gbapp
pub fn init(state: Arc<AppState>) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("Initializing analytics.gbapp virtual crate");
    
    // Initialize analytics services
    services::init_analytics_service(&state)?;
    
    Ok(())
}
```

## Step 3: Add BASIC Keywords

**src/analytics/keywords.rs**
```rust
use crate::shared::state::AppState;
use rhai::{Engine, Dynamic};
use std::sync::Arc;

/// Register analytics keywords with the BASIC interpreter
pub fn register_keywords(engine: &mut Engine, state: Arc<AppState>) {
    let state_clone = state.clone();
    
    // TRACK EVENT keyword
    engine.register_fn("TRACK EVENT", move |event_name: String, properties: String| -> String {
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                crate::analytics::services::track_event(&state_clone, &event_name, &properties).await
            })
        });
        
        match result {
            Ok(_) => format!("Event '{}' tracked", event_name),
            Err(e) => format!("Failed to track event: {}", e),
        }
    });
    
    // GET ANALYTICS keyword
    engine.register_fn("GET ANALYTICS", move |metric: String, timeframe: String| -> Dynamic {
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                crate::analytics::services::get_analytics(&metric, &timeframe).await
            })
        });
        
        match result {
            Ok(data) => Dynamic::from(data),
            Err(_) => Dynamic::UNIT,
        }
    });
    
    // GENERATE REPORT keyword
    engine.register_fn("GENERATE REPORT", move |report_type: String| -> String {
        // Use LLM to generate natural language report
        let data = crate::analytics::services::get_report_data(&report_type);
        
        let prompt = format!(
            "Generate a {} report from this data: {}",
            report_type, data
        );
        
        // This would call the LLM service
        format!("Report generated for: {}", report_type)
    });
}
```

## Step 4: Implement Services

**src/analytics/services.rs**
```rust
use crate::shared::state::AppState;
use crate::shared::models::AnalyticsEvent;
use std::sync::Arc;
use anyhow::Result;

/// Initialize analytics service
pub fn init_analytics_service(state: &Arc<AppState>) -> Result<()> {
    // Set up database tables, connections, etc.
    log::debug!("Analytics service initialized");
    Ok(())
}

/// Track an analytics event
pub async fn track_event(
    state: &Arc<AppState>,
    event_name: &str,
    properties: &str,
) -> Result<()> {
    // Store event in database
    let conn = state.conn.get()?;
    
    // Implementation details...
    log::debug!("Tracked event: {}", event_name);
    
    Ok(())
}

/// Get analytics data
pub async fn get_analytics(metric: &str, timeframe: &str) -> Result<String> {
    // Query analytics data
    let results = match metric {
        "user_count" => get_user_count(timeframe).await?,
        "message_volume" => get_message_volume(timeframe).await?,
        "engagement_rate" => get_engagement_rate(timeframe).await?,
        _ => return Err(anyhow::anyhow!("Unknown metric: {}", metric)),
    };
    
    Ok(results)
}

/// Get data for report generation
pub fn get_report_data(report_type: &str) -> String {
    // Gather data based on report type
    match report_type {
        "daily" => get_daily_report_data(),
        "weekly" => get_weekly_report_data(),
        "monthly" => get_monthly_report_data(),
        _ => "{}".to_string(),
    }
}

// Helper functions
async fn get_user_count(timeframe: &str) -> Result<String> {
    // Implementation
    Ok("100".to_string())
}

async fn get_message_volume(timeframe: &str) -> Result<String> {
    // Implementation
    Ok("5000".to_string())
}

async fn get_engagement_rate(timeframe: &str) -> Result<String> {
    // Implementation
    Ok("75%".to_string())
}

fn get_daily_report_data() -> String {
    // Gather daily metrics
    r#"{"users": 100, "messages": 1500, "sessions": 50}"#.to_string()
}

fn get_weekly_report_data() -> String {
    // Gather weekly metrics
    r#"{"users": 500, "messages": 8000, "sessions": 300}"#.to_string()
}

fn get_monthly_report_data() -> String {
    // Gather monthly metrics
    r#"{"users": 2000, "messages": 35000, "sessions": 1200}"#.to_string()
}
```

## Step 5: Define Data Models

**src/analytics/models.rs**
```rust
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalyticsEvent {
    pub id: uuid::Uuid,
    pub event_name: String,
    pub properties: serde_json::Value,
    pub user_id: Option<String>,
    pub session_id: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricSnapshot {
    pub metric_name: String,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
    pub dimensions: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Report {
    pub report_type: String,
    pub generated_at: DateTime<Utc>,
    pub data: serde_json::Value,
    pub summary: String,
}
```

## Step 6: Register with Core

Update `src/basic/keywords/mod.rs` to include your gbapp:

```rust
use crate::analytics;

pub fn register_all_keywords(engine: &mut Engine, state: Arc<AppState>) {
    // ... existing keywords
    
    // Register analytics.gbapp keywords
    analytics::keywords::register_keywords(engine, state.clone());
}
```

Update `src/main.rs` or initialization code:

```rust
// Initialize analytics gbapp
analytics::init(state.clone())?;
```

## Step 7: Add Tests

**src/analytics/tests.rs**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_track_event() {
        // Test event tracking
        let event_name = "user_login";
        let properties = r#"{"user_id": "123"}"#;
        
        // Test implementation
        assert!(true);
    }
    
    #[tokio::test]
    async fn test_get_analytics() {
        // Test analytics retrieval
        let metric = "user_count";
        let timeframe = "daily";
        
        // Test implementation
        assert!(true);
    }
}
```

## Step 8: Use in BASIC Scripts

Now your gbapp keywords are available in BASIC:

```basic
' Track user actions
TRACK EVENT "button_clicked", "button=submit"

' Get metrics
daily_users = GET ANALYTICS "user_count", "daily"
TALK "Daily active users: " + daily_users

' Generate AI-powered report
report = GENERATE REPORT "weekly"
TALK report

' Combine with LLM for insights
metrics = GET ANALYTICS "all", "monthly"
insights = LLM "Analyze these metrics and provide insights: " + metrics
TALK insights
```

## Step 9: Add Feature Flag (Optional)

If your gbapp should be optional, add it to `Cargo.toml`:

```toml
[features]
analytics = []

# Include in default features if always needed
default = ["ui-server", "chat", "analytics"]
```

Then conditionally compile:

```rust
#[cfg(feature = "analytics")]
pub mod analytics;

#[cfg(feature = "analytics")]
analytics::keywords::register_keywords(engine, state.clone());
```

## Benefits of This Approach

1. **Clean Separation**: Your gbapp is self-contained
2. **Easy Discovery**: Visible in `src/analytics/`
3. **Type Safety**: Rust compiler checks everything
4. **Native Performance**: Compiles into the main binary
5. **Familiar Structure**: Like the old `.gbapp` packages

## Best Practices

✅ **DO:**
- Keep your gbapp focused on one domain
- Provide clear BASIC keywords
- Use LLM for complex logic
- Write comprehensive tests
- Document your keywords

❌ **DON'T:**
- Create overly complex implementations
- Duplicate existing functionality
- Skip error handling
- Forget about async/await
- Ignore the BASIC-first philosophy

## Summary

Creating a gbapp virtual crate is straightforward:
1. Create a module in `src/`
2. Define keywords for BASIC
3. Implement services
4. Register with core
5. Use in BASIC scripts

Your gbapp becomes part of BotServer's compiled binary, providing native performance while maintaining the conceptual clarity of the package system. Most importantly, remember that the implementation should be minimal - let BASIC + LLM handle the complexity!