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

The Suite interface provides multi-application integration with seamless navigation between modules, rich interactions for complex workflows, and responsive design that adapts across desktop, tablet, and mobile form factors. Customizable GBUI templates allow you to choose between `default.gbui` for the full layout or `single.gbui` for a chat-focused experience. Tauri integration enables native desktop packaging for distribution outside the browser.

The Suite interface is best suited for enterprise deployments requiring full functionality, power users working with multiple services simultaneously, desktop application distribution via Tauri builds, and multi-service integrations where context switching between modules matters.

You can access the Suite interface via web at `http://localhost:8080/suite` or as a desktop application through the Tauri build using the `--desktop` flag.

## Minimal Interface

The Minimal interface (`ui/minimal/`) prioritizes speed and simplicity. It loads fast, uses minimal resources, and focuses on essential chat interactions.

This lightweight interface provides core chat and basic interactions only, fast loading with minimal dependencies, and low resource usage suitable for constrained environments. The design supports easy embedding into existing applications and takes a mobile-first approach to responsive layout.

The Minimal interface excels for mobile web access, embedded chatbots in external websites, low-bandwidth environments, quick access terminals and kiosks, and scenarios where simplicity matters more than features.

Access the Minimal interface at the root URL `http://localhost:8080` where it is served by default, explicitly at `http://localhost:8080/minimal`, or embedded via iframe or WebView in your own applications.

## Configuration

### Server Configuration

UI paths are configured in several locations throughout the codebase.

The main server configuration in `src/main.rs` sets the static path:

```rust
let static_path = std::path::Path::new("./web/suite");
```

The UI server module at `src/core/ui_server/mod.rs` defines its own path:

```rust
let static_path = PathBuf::from("./ui/suite");
```

For Tauri desktop builds, `tauri.conf.json` specifies the frontend distribution:

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

The minimal interface serves at root by default, providing faster loading for most users who need quick chat interactions.

## API Compliance

The Minimal UI implements full compliance with the Bot Core API. Both interfaces support the same backend endpoints, ensuring consistent functionality regardless of which interface you choose.

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

Both interfaces use the same WebSocket message types for communication. TEXT (1) handles regular text messages, VOICE (2) handles voice messages, CONTINUE (3) continues interrupted responses, CONTEXT (4) manages context changes, and SYSTEM (5) delivers system messages.

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

Both interfaces support modern browsers with full functionality:

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

If you encounter 404 errors, clear your browser cache, rebuild the project with `cargo clean && cargo build`, and verify the files exist in the `ui/suite/` or `ui/minimal/` directories.

For Tauri build failures, check that `tauri.conf.json` has the correct `frontendDist` path and ensure `ui/suite/index.html` exists.

When static files aren't loading, verify the `ServeDir` configuration in the router and check that subdirectories (js, css, public) exist with their expected contents.

Debug commands can help diagnose issues:

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

The Suite interface uses GBUI templates for layout customization. The `default.gbui` template provides the full multi-app layout with sidebar navigation, while `single.gbui` offers a streamlined chat-focused view. Edit these files to customize the interface structure without modifying core code.

### CSS Theming

Both interfaces support CSS customization through their respective stylesheets. The Suite interface provides more extensive theming options through CSS custom properties, allowing you to adjust colors, spacing, and typography to match your brand.

## Future Enhancements

Planned improvements include dynamic UI selection based on device capabilities to automatically serve the most appropriate interface, progressive enhancement from minimal to suite as users need additional features, service worker implementation for offline support, and WebAssembly components for high-performance features that require client-side computation.

## See Also

- [default.gbui Reference](./default-gbui.md) - Full desktop template
- [single.gbui Reference](./single-gbui.md) - Simple chat template
- [Console Mode](./console-mode.md) - Terminal interface
- [Monitoring Dashboard](./monitoring.md) - System observability