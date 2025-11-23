# Creating Custom Keywords

BotServer's BASIC scripting language can be extended with custom keywords. All keywords are implemented as Rust functions in the `src/basic/keywords/` directory.

## Overview

Keywords in BotServer are Rust functions that get registered with the Rhai scripting engine. They provide the core functionality that BASIC scripts can use to interact with the system.

## Keyword Implementation Structure

### File Organization

Each keyword is typically implemented in its own module file:

```
src/basic/keywords/
├── mod.rs                    # Module registration
├── hear_talk.rs             # HEAR and TALK keywords
├── llm_keyword.rs           # LLM keyword
├── bot_memory.rs            # GET BOT MEMORY, SET BOT MEMORY
├── use_kb.rs                # USE KB keyword
├── clear_kb.rs              # CLEAR KB keyword
├── get.rs                   # GET keyword
├── format.rs                # FORMAT keyword
└── [other keywords].rs
```

## Creating a New Keyword

### Step 1: Create the Module File

Create a new file in `src/basic/keywords/` for your keyword:

```
src/basic/keywords/my_keyword.rs
```

### Step 2: Implement the Keyword Function

Keywords are implemented using one of two Rhai registration methods:

#### Method 1: Simple Function Registration

For basic keywords that return values:

```rust
use rhai::Engine;
use std::sync::Arc;
use crate::core::shared::state::AppState;
use crate::core::session::UserSession;

pub fn my_keyword(
    state: Arc<AppState>,
    user_session: UserSession,
    engine: &mut Engine
) {
    let state_clone = Arc::clone(&state);
    let user_clone = user_session.clone();
    
    engine.register_fn("MY_KEYWORD", move |param: String| -> String {
        // Your keyword logic here
        format!("Processed: {}", param)
    });
}
```

#### Method 2: Custom Syntax Registration

For keywords with special syntax or side effects:

```rust
use rhai::{Engine, EvalAltResult};
use std::sync::Arc;
use crate::core::shared::state::AppState;
use crate::core::session::BotSession;

pub fn register_my_keyword(
    state: Arc<AppState>,
    session: Arc<BotSession>,
    engine: &mut Engine
) -> Result<(), Box<EvalAltResult>> {
    let state_clone = Arc::clone(&state);
    let session_clone = Arc::clone(&session);
    
    engine.register_custom_syntax(
        &["MY_KEYWORD", "$expr$"],  // Syntax pattern
        true,                        // Is statement (not expression)
        move |context, inputs| {
            let param = context.eval_expression_tree(&inputs[0])?.to_string();
            
            // Your keyword logic here
            info!("MY_KEYWORD executed with: {}", param);
            
            Ok(().into())
        }
    )?;
    
    Ok(())
}
```

### Step 3: Register in mod.rs

Add your module to `src/basic/keywords/mod.rs`:

```rust
pub mod my_keyword;
```

### Step 4: Add to Keyword Registry

Keywords are registered in the BASIC interpreter initialization. The registration happens in the main interpreter setup where all keywords are added to the Rhai engine.

## Keyword Patterns

### Pattern 1: Database Operations

Keywords that interact with the database (like `GET BOT MEMORY`):

```rust
pub fn database_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    let state_clone = Arc::clone(&state);
    let user_clone = user.clone();
    
    engine.register_fn("DB_KEYWORD", move |key: String| -> String {
        let state = Arc::clone(&state_clone);
        let conn_result = state.conn.get();
        
        if let Ok(mut conn) = conn_result {
            // Database query using Diesel
            // Return result
        } else {
            String::new()
        }
    });
}
```

### Pattern 2: Async Operations

Keywords that need async operations (like `WEATHER`):

```rust
pub fn async_keyword(state: Arc<AppState>, user: UserSession, engine: &mut Engine) {
    engine.register_custom_syntax(&["ASYNC_OP", "$expr$"], false, move |context, inputs| {
        let param = context.eval_expression_tree(&inputs[0])?;
        
        // Create channel for async result
        let (tx, rx) = std::sync::mpsc::channel();
        
        // Spawn blocking task
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                // Async operation here
                "result".to_string()
            });
            let _ = tx.send(result);
        });
        
        // Wait for result
        match rx.recv_timeout(Duration::from_secs(30)) {
            Ok(result) => Ok(result.into()),
            Err(_) => Ok("Timeout".into()),
        }
    });
}
```

### Pattern 3: Session Management

Keywords that modify session state (like `USE KB`, `CLEAR KB`):

```rust
pub fn register_session_keyword(
    state: Arc<AppState>,
    session: Arc<BotSession>,
    engine: &mut Engine
) -> Result<(), Box<EvalAltResult>> {
    let session_clone = Arc::clone(&session);
    
    engine.register_custom_syntax(&["SESSION_OP", "$expr$"], true, move |context, inputs| {
        let param = context.eval_expression_tree(&inputs[0])?.to_string();
        
        // Modify session state
        let mut session_lock = session_clone.blocking_write();
        // Update session fields
        
        Ok(().into())
    })?;
    
    Ok(())
}
```

## Available Dependencies

Keywords have access to:

1. **AppState**: Application-wide state including:
   - Database connection pool (`state.conn`)
   - Drive client for S3-compatible storage (`state.drive`)
   - Cache client (`state.cache`)
   - Configuration (`state.config`)
   - LLM provider (`state.llm_provider`)

2. **UserSession**: Current user's session data:
   - User ID (`user_session.user_id`)
   - Bot ID (`user_session.bot_id`)
   - Session ID (`user_session.session_id`)

3. **BotSession**: Bot conversation state:
   - Context collections
   - Tool definitions
   - Conversation history
   - Session variables

## Error Handling

Keywords should handle errors gracefully:

```rust
engine.register_fn("SAFE_KEYWORD", move |param: String| -> String {
    match risky_operation(&param) {
        Ok(result) => result,
        Err(e) => {
            error!("Keyword error: {}", e);
            format!("Error: {}", e)
        }
    }
});
```

## Testing Keywords

Keywords can be tested with unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_my_keyword() {
        // Create test engine
        let mut engine = Engine::new();
        
        // Register keyword
        // Test keyword execution
        // Assert results
    }
}
```

## Best Practices

1. **Clone Arc References**: Always clone Arc-wrapped state before moving into closures
2. **Use Logging**: Add info/debug logging for keyword execution
3. **Handle Errors**: Don't panic, return error messages as strings
4. **Timeout Async Ops**: Use timeouts for network operations
5. **Document Parameters**: Use clear parameter names and add comments
6. **Keep It Simple**: Each keyword should do one thing well
7. **Thread Safety**: Ensure all operations are thread-safe

## Example: Complete Keyword Implementation

Here's a complete example of a custom keyword that saves data:

```rust
// src/basic/keywords/save_data.rs

use rhai::Engine;
use std::sync::Arc;
use log::{info, error};
use crate::core::shared::state::AppState;
use crate::core::session::UserSession;

pub fn save_data_keyword(
    state: Arc<AppState>,
    user_session: UserSession,
    engine: &mut Engine
) {
    let state_clone = Arc::clone(&state);
    let user_clone = user_session.clone();
    
    engine.register_fn("SAVE_DATA", move |key: String, value: String| -> String {
        info!("SAVE_DATA called: key={}, value={}", key, value);
        
        let state = Arc::clone(&state_clone);
        let conn_result = state.conn.get();
        
        match conn_result {
            Ok(mut conn) => {
                // Save to database using Diesel
                // (actual implementation would use proper Diesel queries)
                info!("Data saved successfully");
                "OK".to_string()
            }
            Err(e) => {
                error!("Database error: {}", e);
                format!("Error: {}", e)
            }
        }
    });
}
```

## Limitations

- Keywords must be synchronous or use blocking operations
- Direct async/await is not supported (use channels for async)
- Keywords are registered globally for all scripts
- Cannot dynamically add keywords at runtime
- All keywords must be compiled into the binary

## Summary

Creating custom keywords extends BotServer's BASIC language capabilities. Keywords are Rust functions registered with the Rhai engine that provide access to system features, databases, external APIs, and more. Follow the patterns shown above to create robust, thread-safe keywords that integrate seamlessly with the BotServer ecosystem.