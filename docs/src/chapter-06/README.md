# Extending BotServer

This chapter covers how to extend and customize BotServer to meet specific requirements, from creating custom keywords to building new channel adapters and integrating with external systems.

## Overview

BotServer is designed to be extensible at multiple levels:
- **BASIC Keywords**: Add new commands to the scripting language
- **Channel Adapters**: Support new messaging platforms
- **Storage Backends**: Integrate different storage systems
- **Authentication Providers**: Connect to various identity services
- **LLM Providers**: Add support for new language models

## Extension Points

### 1. Custom BASIC Keywords

Create new keywords by implementing them in Rust:

```rust
// src/basic/keywords/my_keyword.rs
use rhai::{Engine, Dynamic};
use crate::shared::state::AppState;

pub fn register_my_keyword(engine: &mut Engine, state: Arc<AppState>) {
    engine.register_fn("MY_KEYWORD", move |param: String| -> String {
        // Your implementation here
        format!("Processed: {}", param)
    });
}
```

Register in the keyword module:
```rust
// src/basic/keywords/mod.rs
pub fn register_all_keywords(engine: &mut Engine, state: Arc<AppState>) {
    // ... existing keywords
    register_my_keyword(engine, state.clone());
}
```

Use in BASIC scripts:
```basic
result = MY_KEYWORD "input data"
TALK result
```

### 2. Channel Adapters

Implement a new messaging channel:

```rust
// src/channels/my_channel.rs
use async_trait::async_trait;
use crate::channels::traits::ChannelAdapter;

pub struct MyChannelAdapter {
    config: MyChannelConfig,
}

#[async_trait]
impl ChannelAdapter for MyChannelAdapter {
    async fn send_message(&self, recipient: &str, message: &str) -> Result<()> {
        // Send message implementation
    }
    
    async fn receive_message(&self) -> Result<Message> {
        // Receive message implementation
    }
    
    async fn send_attachment(&self, recipient: &str, file: &[u8]) -> Result<()> {
        // Send file implementation
    }
}
```

### 3. Storage Providers

Add support for new storage backends:

```rust
// src/storage/my_storage.rs
use async_trait::async_trait;
use crate::storage::traits::StorageProvider;

pub struct MyStorageProvider {
    client: MyStorageClient,
}

#[async_trait]
impl StorageProvider for MyStorageProvider {
    async fn get(&self, key: &str) -> Result<Vec<u8>> {
        // Retrieve object
    }
    
    async fn put(&self, key: &str, data: &[u8]) -> Result<()> {
        // Store object
    }
    
    async fn delete(&self, key: &str) -> Result<()> {
        // Delete object
    }
    
    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        // List objects
    }
}
```

## Architecture for Extensions

### Plugin System

BotServer uses a modular architecture that supports plugins:

```
botserver/
├── src/
│   ├── core/           # Core functionality
│   ├── basic/          # BASIC interpreter
│   │   └── keywords/   # Keyword implementations
│   ├── channels/       # Channel adapters
│   ├── storage/        # Storage providers
│   ├── auth/          # Authentication modules
│   └── llm/           # LLM integrations
```

### Dependency Injection

Extensions use dependency injection for configuration:

```rust
// Configuration
#[derive(Deserialize)]
pub struct ExtensionConfig {
    pub enabled: bool,
    pub options: HashMap<String, String>,
}

// Registration
pub fn register_extension(app_state: &mut AppState, config: ExtensionConfig) {
    if config.enabled {
        let extension = MyExtension::new(config.options);
        app_state.extensions.push(Box::new(extension));
    }
}
```

## Common Extension Patterns

### 1. API Integration

Create a keyword for external API calls:

```rust
pub fn register_api_keyword(engine: &mut Engine) {
    engine.register_fn("CALL_API", |url: String, method: String| -> Dynamic {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let client = reqwest::Client::new();
            let response = match method.as_str() {
                "GET" => client.get(&url).send().await,
                "POST" => client.post(&url).send().await,
                _ => return Dynamic::from("Invalid method"),
            };
            
            match response {
                Ok(resp) => Dynamic::from(resp.text().await.unwrap_or_default()),
                Err(_) => Dynamic::from("Error calling API"),
            }
        })
    });
}
```

### 2. Database Operations

Add custom database queries:

```rust
pub fn register_db_keyword(engine: &mut Engine, state: Arc<AppState>) {
    let state_clone = state.clone();
    engine.register_fn("QUERY_DB", move |sql: String| -> Vec<Dynamic> {
        let mut conn = state_clone.conn.get().unwrap();
        
        // Execute query (with proper sanitization)
        let results = diesel::sql_query(sql)
            .load::<CustomResult>(&mut conn)
            .unwrap_or_default();
        
        // Convert to Dynamic
        results.into_iter()
            .map(|r| Dynamic::from(r))
            .collect()
    });
}
```

### 3. Event Handlers

Implement custom event processing:

```rust
pub trait EventHandler: Send + Sync {
    fn handle_event(&self, event: Event) -> Result<()>;
}

pub struct CustomEventHandler;

impl EventHandler for CustomEventHandler {
    fn handle_event(&self, event: Event) -> Result<()> {
        match event {
            Event::MessageReceived(msg) => {
                // Process incoming message
            },
            Event::SessionStarted(session) => {
                // Initialize session
            },
            Event::Error(err) => {
                // Handle errors
            },
            _ => Ok(()),
        }
    }
}
```

## Testing Extensions

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_keyword() {
        let mut engine = Engine::new();
        register_my_keyword(&mut engine);
        
        let result: String = engine.eval(r#"MY_KEYWORD "test""#).unwrap();
        assert_eq!(result, "Processed: test");
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_channel_adapter() {
    let adapter = MyChannelAdapter::new(test_config());
    
    // Test sending
    let result = adapter.send_message("user123", "Test message").await;
    assert!(result.is_ok());
    
    // Test receiving
    let message = adapter.receive_message().await.unwrap();
    assert_eq!(message.content, "Expected response");
}
```

## Deployment Considerations

### Configuration

Extensions are configured in `config.csv`:

```csv
name,value
extension_my_feature,enabled
extension_my_feature_option1,value1
extension_my_feature_option2,value2
```

### Performance Impact

Consider performance when adding extensions:
- Use async operations for I/O
- Implement caching where appropriate
- Profile resource usage
- Add metrics and monitoring

### Security

Ensure extensions are secure:
- Validate all input
- Use prepared statements for database queries
- Implement rate limiting
- Add authentication where needed
- Follow least privilege principle

## Best Practices

### 1. Error Handling

Always handle errors gracefully:

```rust
pub fn my_extension_function() -> Result<String> {
    // Use ? operator for error propagation
    let data = fetch_data()?;
    let processed = process_data(data)?;
    Ok(format!("Success: {}", processed))
}
```

### 2. Logging

Add comprehensive logging:

```rust
use log::{info, warn, error, debug};

pub fn process_request(req: Request) {
    debug!("Processing request: {:?}", req);
    
    match handle_request(req) {
        Ok(result) => info!("Request successful: {}", result),
        Err(e) => error!("Request failed: {}", e),
    }
}
```

### 3. Documentation

Document your extensions:

```rust
/// Custom keyword for data processing
/// 
/// # Arguments
/// * `input` - The data to process
/// 
/// # Returns
/// Processed data as a string
/// 
/// # Example
/// ```basic
/// result = PROCESS_DATA "raw input"
/// ```
pub fn process_data_keyword(input: String) -> String {
    // Implementation
}
```

## Examples of Extensions

### Weather Integration

```rust
pub fn register_weather_keyword(engine: &mut Engine) {
    engine.register_fn("GET_WEATHER", |city: String| -> String {
        // Call weather API
        let api_key = std::env::var("WEATHER_API_KEY").unwrap_or_default();
        let url = format!("https://api.weather.com/v1/weather?city={}&key={}", city, api_key);
        
        // Fetch and parse response
        // Return weather information
    });
}
```

### Custom Analytics

```rust
pub struct AnalyticsExtension {
    client: AnalyticsClient,
}

impl AnalyticsExtension {
    pub fn track_event(&self, event: &str, properties: HashMap<String, String>) {
        self.client.track(Event {
            name: event.to_string(),
            properties,
            timestamp: Utc::now(),
        });
    }
}
```

## Summary

Extending BotServer allows you to:
- Add domain-specific functionality
- Integrate with existing systems
- Support new communication channels
- Implement custom business logic
- Enhance the platform's capabilities

The modular architecture and clear extension points make it straightforward to add new features while maintaining system stability and performance.