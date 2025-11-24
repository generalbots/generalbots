# gbapp: Virtual Crates Architecture

This chapter explains how BotServer uses the gbapp concept as virtual crates within the `src/` directory, elegantly mapping the old package system to the new Rust architecture.

## The gbapp Evolution: From Packages to Virtual Crates

### Historical Context (Node.js Era)
In previous versions, `.gbapp` packages were external Node.js modules that extended BotServer functionality through a plugin system.

### Current Architecture (Rust Era)
The `.gbapp` concept now lives as **virtual crates** inside `src/`:
- **Virtual Crates**: Each gbapp is a module inside `src/` (like `src/core`, `src/basic`, `src/channels`)
- **Same Mental Model**: Developers familiar with the old system can think of each directory as a "package"
- **Native Performance**: All code compiles into a single optimized binary
- **Contribution Path**: Add new gbapps by creating modules in `src/`

## How gbapp Virtual Crates Work

```
src/
├── core/           # core.gbapp (virtual crate)
├── basic/          # basic.gbapp (BASIC interpreter)
├── channels/       # channels.gbapp (communication)
├── storage/        # storage.gbapp (persistence)
├── auth/           # auth.gbapp (authentication)
├── llm/            # llm.gbapp (AI integration)
└── your_feature/   # your_feature.gbapp (your contribution!)
```

Each directory is conceptually a gbapp - a self-contained module that contributes functionality to the whole.

## Why This Change?

1. **Simplicity**: One cohesive codebase instead of fragmented extensions
2. **Performance**: Native Rust performance without extension overhead
3. **Reliability**: Thoroughly tested core features vs. variable-quality plugins
4. **BASIC Power**: BASIC + LLM combination eliminates need for custom code
5. **Maintenance**: Easier to maintain one strong core than many extensions

## Contributing New Keywords

### Contributing a New gbapp Virtual Crate

To add functionality, create a new gbapp as a module in `src/`:

```rust
// src/your_feature/mod.rs - Your gbapp virtual crate
pub mod keywords;
pub mod services;
pub mod models;

// src/your_feature/keywords/mod.rs
use crate::shared::state::AppState;
use rhai::Engine;

pub fn register_keywords(engine: &mut Engine, state: Arc<AppState>) {
    engine.register_fn("YOUR KEYWORD", move |param: String| -> String {
        // Implementation
        format!("Result: {}", param)
    });
}
```

This maintains the conceptual model of packages while leveraging Rust's module system.

### Contribution Process for gbapp Virtual Crates

1. **Fork** the BotServer repository
2. **Create** your gbapp module in `src/your_feature/`
3. **Structure** it like existing gbapps (core, basic, etc.)
4. **Test** thoroughly with unit and integration tests
5. **Document** in the appropriate chapter
6. **Submit PR** describing your gbapp's purpose

Example structure for a new gbapp:
```
src/analytics/          # analytics.gbapp
├── mod.rs             # Module definition
├── keywords.rs        # BASIC keywords
├── services.rs        # Core services
├── models.rs          # Data models
└── tests.rs           # Unit tests
```

## Adding New Components

Components are features compiled into BotServer via Cargo features:

### Current Components in Cargo.toml

```toml
[features]
# Core features
chat = []           # Chat functionality
drive = []          # Storage system
tasks = []          # Task management
calendar = []       # Calendar integration
meet = []           # Video meetings
mail = []           # Email system

# Enterprise features
compliance = []     # Compliance tools
attendance = []     # Attendance tracking
directory = []      # User directory
```

### Adding a New Component

1. **Define Feature** in `Cargo.toml`:
```toml
[features]
your_feature = ["dep:required_crate"]
```

2. **Implement** in appropriate module:
```rust
#[cfg(feature = "your_feature")]
pub mod your_feature {
    // Implementation
}
```

3. **Register** in `installer.rs`:
```rust
fn register_your_feature(&mut self) {
    self.components.insert(
        "your_feature",
        Component {
            name: "Your Feature",
            description: "Feature description",
            port: None,
            setup_required: false,
        },
    );
}
```

## Understanding the gbapp → Virtual Crate Mapping

The transition from Node.js packages to Rust modules maintains conceptual familiarity:

| Old (Node.js) | New (Rust) | Location | Purpose |
|---------------|------------|----------|---------|
| `core.gbapp` | `core` module | `src/core/` | Core engine functionality |
| `basic.gbapp` | `basic` module | `src/basic/` | BASIC interpreter |
| `whatsapp.gbapp` | `channels::whatsapp` | `src/channels/whatsapp/` | WhatsApp integration |
| `kb.gbapp` | `storage::kb` | `src/storage/kb/` | Knowledge base |
| `custom.gbapp` | `custom` module | `src/custom/` | Your contribution |

### Creating Private gbapp Virtual Crates

For proprietary features, you can still create private gbapps:

```rust
// Fork BotServer, then add your private gbapp
// src/proprietary/mod.rs
#[cfg(feature = "proprietary")]
pub mod my_private_feature {
    // Your private implementation
}
```

Then in `Cargo.toml`:
```toml
[features]
proprietary = []
```

This keeps your code separate while benefiting from core updates.

### Benefits of the Virtual Crate Approach

1. **Familiar Mental Model**: Developers understand "packages"
2. **Clean Separation**: Each gbapp is self-contained
3. **Easy Discovery**: All gbapps visible in `src/`
4. **Native Performance**: Everything compiles together
5. **Type Safety**: Rust ensures interfaces are correct

## Real Examples of gbapp Virtual Crates in src/

```
src/
├── core/               # Core gbapp - Bootstrap, package manager
│   ├── mod.rs
│   ├── bootstrap.rs
│   └── package_manager/
│
├── basic/              # BASIC gbapp - Interpreter and keywords  
│   ├── mod.rs
│   ├── interpreter.rs
│   └── keywords/
│       ├── mod.rs
│       ├── talk.rs
│       ├── hear.rs
│       └── llm.rs
│
├── channels/           # Channels gbapp - Communication adapters
│   ├── mod.rs
│   ├── whatsapp.rs
│   ├── teams.rs
│   └── email.rs
│
└── analytics/          # Your new gbapp!
    ├── mod.rs
    ├── keywords.rs     # ADD ANALYTICS, GET METRICS
    └── services.rs     # Analytics engine
```

## Development Environment

### System Requirements

- **Disk Space**: 8GB minimum for development
- **RAM**: 8GB recommended
- **Database**: Any SQL database (abstracted)
- **Storage**: Any S3-compatible storage (abstracted)

### No Brand Lock-in

BotServer uses generic terms:
- ❌ PostgreSQL → ✅ "database"
- ❌ MinIO → ✅ "drive storage"
- ❌ Qdrant → ✅ "vector database"
- ❌ Redis → ✅ "cache"

This ensures vendor neutrality and flexibility.

## Security Best Practices

### Regular Audits

Run security audits regularly:
```bash
cargo audit
```

This checks for known vulnerabilities in dependencies.

### Secure Coding

When contributing:
- Validate all inputs
- Use safe Rust patterns
- Avoid `unsafe` blocks
- Handle errors properly
- Add security tests

## Testing Your Contributions

### Unit Tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_keyword() {
        // Test your keyword
    }
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_feature() {
    // Test feature integration
}
```

### BASIC Script Tests
```basic
' test_script.bas
result = YOUR KEYWORD "test"
IF result != "expected" THEN
    TALK "Test failed"
ELSE
    TALK "Test passed"
END IF
```

## Documentation Requirements

All contributions must include:

1. **Keyword Documentation** in Chapter 6
2. **Architecture Updates** if structural changes
3. **API Documentation** for new endpoints
4. **BASIC Examples** showing usage
5. **Migration Guide** if breaking changes

## Performance Considerations

### Benchmarking

Before submitting:
```bash
cargo bench
```

### Profiling

Identify bottlenecks:
```bash
cargo flamegraph
```

## Community Guidelines

### What We Accept

✅ New BASIC keywords that benefit many users  
✅ Performance improvements  
✅ Bug fixes with tests  
✅ Documentation improvements  
✅ Security enhancements  

### What We Don't Accept

❌ Vendor-specific integrations (use generic interfaces)  
❌ Extensions that bypass BASIC  
❌ Features achievable with existing keywords  
❌ Undocumented code  
❌ Code without tests  

## The Power of BASIC + LLM

Remember: In 2025, 100% BASIC/LLM applications are reality. Before adding a keyword, consider:

1. Can this be done with existing keywords + LLM?
2. Will this keyword benefit multiple use cases?
3. Does it follow the BASIC philosophy of simplicity?

### Example: No Custom Code Needed

Instead of custom integration code:
```basic
' Everything in BASIC
data = GET "api.example.com/data"
processed = LLM "Process this data: " + data
result = FIND "table", "criteria=" + processed
SEND MAIL user, "Results", result
```

## Future Direction

BotServer's future is:
- **Stronger Core**: More powerful built-in keywords
- **Better LLM Integration**: Smarter AI capabilities
- **Simpler BASIC**: Even easier scripting
- **Community-Driven**: Features requested by users

## How to Get Started

1. **Fork** the repository
2. **Read** existing code in `src/basic/keywords/`
3. **Discuss** your idea in GitHub Issues
4. **Implement** following the patterns
5. **Test** thoroughly
6. **Document** completely
7. **Submit** PR with clear explanation

## Summary

The `.gbapp` concept has elegantly evolved from external Node.js packages to **virtual crates** within `src/`. This approach:
- **Preserves the mental model** developers are familiar with
- **Maps perfectly** to Rust's module system
- **Encourages contribution** by making the structure clear
- **Maintains separation** while compiling to a single binary

Each directory in `src/` is effectively a gbapp - contribute by adding your own! With BASIC + LLM handling the complexity, your gbapp just needs to provide the right keywords and services.

## See Also

- [Philosophy](./philosophy.md) - The gbapp philosophy: Let machines do machine work
- [Architecture](./architecture.md) - System architecture
- [Building](./building.md) - Build process
- [Custom Keywords](./custom-keywords.md) - Keyword implementation
- [Services](./services.md) - Core services
- [Chapter 6: BASIC Reference](../chapter-06-gbdialog/README.md) - BASIC language
- [Chapter 9: API](../chapter-09-api/README.md) - API documentation

---

<div align="center">
  <img src="https://pragmatismo.com.br/icons/general-bots-text.svg" alt="General Bots" width="200">
</div>
