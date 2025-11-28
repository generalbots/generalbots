# UI Structure Documentation

## Overview

The BotServer UI system consists of two main interface implementations designed for different use cases and deployment scenarios.

## Directory Structure

```
ui/
├── suite/       # Full-featured suite interface (formerly desktop)
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
└── minimal/     # Lightweight minimal interface (formerly html)
    ├── index.html
    ├── styles.css
    └── app.js
```

## Interface Types

### Suite Interface (`ui/suite/`)

The **Suite** interface is the comprehensive, full-featured UI that provides:

- **Multi-application integration**: Chat, Drive, Tasks, Mail modules
- **Desktop-class experience**: Rich interactions and complex workflows
- **Responsive design**: Works on desktop, tablet, and mobile
- **GBUI templates**: Customizable interface templates
  - `default.gbui`: Full multi-app layout
  - `single.gbui`: Streamlined chat-focused interface
- **Tauri integration**: Can be packaged as a desktop application

**Use Cases:**
- Enterprise deployments
- Power users requiring full functionality
- Desktop application distribution
- Multi-service integrations

**Access:**
- Web: `http://localhost:8080/suite` (explicit suite access)
- Desktop: Via Tauri build with `--desktop` flag

### Minimal Interface (`ui/minimal/`)

The **Minimal** interface is a lightweight, fast-loading UI that provides:

- **Essential features only**: Core chat and basic interactions
- **Fast loading**: Minimal dependencies and assets
- **Low resource usage**: Optimized for constrained environments
- **Easy embedding**: Simple to integrate into existing applications
- **Mobile-first**: Designed primarily for mobile and embedded use

**Use Cases:**
- Mobile web access
- Embedded chatbots
- Low-bandwidth environments
- Quick access terminals
- Kiosk deployments

**Access:**
- Direct: `http://localhost:8080` (default)
- Explicit: `http://localhost:8080/minimal`
- Embedded: Via iframe or WebView

## Configuration

### Server Configuration

The UI paths are configured in multiple locations:

1. **Main Server** (`src/main.rs`):
   ```rust
   let static_path = std::path::Path::new("./web/suite");
   ```

2. **UI Server Module** (`src/core/ui_server/mod.rs`):
   ```rust
   let static_path = PathBuf::from("./ui/suite");
   ```

3. **Tauri Configuration** (`tauri.conf.json`):
   ```json
   {
     "build": {
       "frontendDist": "./ui/suite"
     }
   }
   ```

### Switching Between Interfaces

#### Default Interface Selection

The minimal interface is served by default at the root path. This provides faster loading and lower resource usage for most users.

1. Update `ui_server/mod.rs`:
   ```rust
   // For minimal (default)
   match fs::read_to_string("ui/minimal/index.html")
   
   // For suite
   match fs::read_to_string("ui/suite/index.html")
   ```

#### Routing Configuration

Both interfaces can be served simultaneously with different routes:

```rust
Router::new()
    .route("/", get(serve_minimal))          // Minimal at root (default)
    .route("/minimal", get(serve_minimal))  // Explicit minimal route
    .route("/suite", get(serve_suite))      // Suite at /suite
```

## Development Guidelines

### When to Use Suite Interface

Choose the Suite interface when you need:
- Full application functionality
- Multi-module integration
- Desktop-like user experience
- Complex workflows and data management
- Rich media handling

### When to Use Minimal Interface

Choose the Minimal interface when you need:
- Fast, lightweight deployment
- Mobile-optimized experience
- Embedded chatbot functionality
- Limited bandwidth scenarios
- Simple, focused interactions

## Migration Notes

### From Previous Structure

The UI directories were renamed for clarity:
- `ui/desktop` → `ui/suite` (reflects full-featured nature)
- `ui/html` → `ui/minimal` (reflects lightweight design)

### Updating Existing Code

When migrating existing code:

1. Update static file paths:
   ```rust
   // Old
   let static_path = PathBuf::from("./ui/desktop");
   
   // New
   let static_path = PathBuf::from("./ui/suite");
   ```

2. Update documentation references:
   ```markdown
   <!-- Old -->
   Location: `ui/desktop/default.gbui`
   
   <!-- New -->
   Location: `ui/suite/default.gbui`
   ```

3. Update build configurations:
   ```json
   // Old
   "frontendDist": "./ui/desktop"
   
   // New
   "frontendDist": "./ui/suite"
   ```

## Future Enhancements

### Planned Features

1. **Dynamic UI Selection**: Runtime switching between suite and minimal
2. **Progressive Enhancement**: Start with minimal, upgrade to suite as needed
3. **Custom Themes**: User-selectable themes for both interfaces
4. **Module Lazy Loading**: Load suite modules on-demand
5. **Offline Support**: Service worker implementation for both UIs

### Interface Convergence

Future versions may introduce:
- **Adaptive Interface**: Single UI that adapts based on device capabilities
- **Micro-frontends**: Independent module deployment
- **WebAssembly Components**: High-performance UI components
- **Native Mobile Apps**: React Native or Flutter implementations

## Troubleshooting

### Common Issues

1. **404 Errors After Rename**:
   - Clear browser cache
   - Rebuild the project: `cargo clean && cargo build`
   - Verify file paths in `ui/suite/` or `ui/minimal/`

2. **Tauri Build Failures**:
   - Update `tauri.conf.json` with correct `frontendDist` path
   - Ensure `ui/suite/index.html` exists

3. **Static Files Not Loading**:
   - Check `ServeDir` configuration in router
   - Verify subdirectories (js, css, public) exist in new location

### Debug Commands

```bash
# Verify UI structure
ls -la ui/suite/
ls -la ui/minimal/

# Test minimal interface (default)
curl http://localhost:8080/

# Test suite interface
curl http://localhost:8080/suite/

# Check static file serving
curl http://localhost:8080/js/app.js
curl http://localhost:8080/css/styles.css
```

## Related Documentation

- [GBUI Templates](./chapter-04-gbui/README.md)
- [UI Server Module](../src/core/ui_server/README.md)
- [Desktop Application](./DESKTOP.md)
- [Web Deployment](./WEB_DEPLOYMENT.md)