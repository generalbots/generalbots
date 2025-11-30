# UI Structure

The BotServer UI system provides two interface implementations designed for different deployment scenarios. Choose the right interface based on your use case and performance requirements.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

## Directory Layout

```
ui/
├── suite/       # Full-featured interface
│   ├── index.html
│   ├── js/
│   ├── css/
│   ├── public/
│   ├── drive/
│   ├── chat/
│   ├── mail/
│   ├── tasks/
│   ├── default.gbui
│   └── single.gbui
│
└── minimal/     # Lightweight interface
    ├── index.html
    ├── styles.css
    └── app.js
```

## Suite Interface

The Suite interface (`ui/suite/`) delivers a comprehensive, desktop-class experience with multi-application integration. It includes Chat, Drive, Tasks, and Mail modules in a unified workspace.

**Capabilities:**
- Multi-application integration with seamless navigation
- Rich interactions and complex workflows
- Responsive design across desktop, tablet, and mobile
- Customizable GBUI templates (`default.gbui` for full layout, `single.gbui` for chat-focused)
- Tauri integration for native desktop packaging

**When to use Suite:**
- Enterprise deployments requiring full functionality
- Power users working with multiple services
- Desktop application distribution via Tauri
- Multi-service integrations where context switching matters

**Access:**
- Web: `http://localhost:8080/suite`
- Desktop: Via Tauri build with `--desktop` flag

## Minimal Interface

The Minimal interface (`ui/minimal/`) prioritizes speed and simplicity. It loads fast, uses minimal resources, and focuses on essential chat interactions.

**Capabilities:**
- Core chat and basic interactions only
- Fast loading with minimal dependencies
- Low resource usage for constrained environments
- Easy embedding into existing applications
- Mobile-first design approach

**When to use Minimal:**
- Mobile web access
- Embedded chatbots in external websites
- Low-bandwidth environments
- Quick access terminals and kiosks
- Scenarios where simplicity matters more than features

**Access:**
- Default: `http://localhost:8080` (served at root)
- Explicit: `http://localhost:8080/minimal`
- Embedded: Via iframe or WebView

## Configuration

### Server Configuration

UI paths are configured in several locations:

**Main Server** (`src/main.rs`):
```rust
let static_path = std::path::Path::new("./web/suite");
```

**UI Server Module** (`src/core/ui_server/mod.rs`):
```rust
let static_path = PathBuf::from("./ui/suite");
```

**Tauri Configuration** (`tauri.conf.json`):
```json
{
  "build": {
    "frontendDist": "./ui/suite"
  }
}
```

### Routing

Both interfaces can be served simultaneously with different routes:

```rust
Router::new()
    .route("/", get(serve_minimal))
    .route("/minimal", get(serve_minimal))
    .route("/suite", get(serve_suite))
```

The minimal interface serves at root by default, providing faster loading for most users.

## API Compliance

The Minimal UI implements full compliance with the Bot Core API. Both interfaces support the same backend endpoints.

**Supported Endpoints:**

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/ws` | WebSocket | Real-time messaging |
| `/api/auth` | GET | Authentication |
| `/api/sessions` | GET/POST | Session management |
| `/api/sessions/{id}` | GET | Session details |
| `/api/sessions/{id}/history` | GET | Message history |
| `/api/sessions/{id}/start` | POST | Start session |
| `/api/voice/start` | POST | Voice input start |
| `/api/voice/stop` | POST | Voice input stop |

**WebSocket Protocol:**

Both interfaces use the same message types:
- `TEXT (1)` - Regular text messages
- `VOICE (2)` - Voice messages
- `CONTINUE (3)` - Continue interrupted responses
- `CONTEXT (4)` - Context changes
- `SYSTEM (5)` - System messages

## Performance Characteristics

### Suite Interface

| Metric | Typical Value |
|--------|---------------|
| Initial load | ~500KB |
| Time to interactive | ~1.5s |
| Memory usage | ~80MB |
| Best for | Full productivity |

### Minimal Interface

| Metric | Typical Value |
|--------|---------------|
| Initial load | ~50KB |
| Time to interactive | ~200ms |
| Memory usage | ~20MB |
| Best for | Quick interactions |

## Browser Support

Both interfaces support modern browsers:

| Browser | Minimum Version | WebSocket | Voice |
|---------|----------------|-----------|-------|
| Chrome | 90+ | ✅ | ✅ |
| Firefox | 88+ | ✅ | ✅ |
| Safari | 14+ | ✅ | ✅ |
| Edge | 90+ | ✅ | ✅ |
| Mobile Chrome | 90+ | ✅ | ✅ |
| Mobile Safari | 14+ | ✅ | ✅ |

## Switching Interfaces

Users can switch between interfaces by navigating to the appropriate URL. For programmatic switching, update the `ui_server/mod.rs` to change the default:

```rust
// Serve minimal at root (default)
match fs::read_to_string("ui/minimal/index.html")

// Or serve suite at root
match fs::read_to_string("ui/suite/index.html")
```

## Troubleshooting

**404 Errors:**
- Clear browser cache
- Rebuild: `cargo clean && cargo build`
- Verify files exist in `ui/suite/` or `ui/minimal/`

**Tauri Build Failures:**
- Check `tauri.conf.json` has correct `frontendDist` path
- Ensure `ui/suite/index.html` exists

**Static Files Not Loading:**
- Verify `ServeDir` configuration in router
- Check subdirectories (js, css, public) exist

**Debug Commands:**
```bash
# Verify UI structure
ls -la ui/suite/
ls -la ui/minimal/

# Test interfaces
curl http://localhost:8080/
curl http://localhost:8080/suite/

# Check static file serving
curl http://localhost:8080/js/app.js
```

## Customization

### GBUI Templates

The Suite interface uses GBUI templates for layout customization:

- `default.gbui` - Full multi-app layout with sidebar
- `single.gbui` - Streamlined chat-focused view

Edit these files to customize the interface structure without modifying core code.

### CSS Theming

Both interfaces support CSS customization through their respective stylesheets. The Suite interface provides more theming options through CSS custom properties.

## Future Enhancements

Planned improvements include:

- Dynamic UI selection based on device capabilities
- Progressive enhancement from minimal to suite
- Service worker implementation for offline support
- WebAssembly components for high-performance features

## See Also

- [default.gbui Reference](./default-gbui.md) - Full desktop template
- [single.gbui Reference](./single-gbui.md) - Simple chat template
- [Console Mode](./console-mode.md) - Terminal interface
- [Monitoring Dashboard](./monitoring.md) - System observability