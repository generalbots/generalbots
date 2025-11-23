# UI Migration Summary

## Overview
This document summarizes the migration from `web/` directory structure to `ui/` directory structure and the introduction of `.gbui` file format for user interface templates.

## Directory Structure Changes

### Before
```
botserver/
└── web/
    ├── desktop/
    │   └── index.html
    └── html/
```

### After
```
botserver/
└── ui/
    ├── desktop/
    │   ├── default.gbui    # Full-featured interface (copy of index.html)
    │   ├── single.gbui     # Simplified single-page chat interface
    │   └── index.html      # Original file retained for compatibility
    └── html/
```

## Code Changes

### 1. Module Renaming
- **Before**: `botserver/src/core/web_server/`
- **After**: `botserver/src/core/ui_server/`

### 2. Import Updates

#### lib.rs
```rust
// Before
pub use core::web_server;

// After
pub use core::ui_server;
```

#### main.rs
```rust
// Before
use botserver::core::web_server;
.route("/", get(crate::web_server::index))

// After
use botserver::core::ui_server;
.route("/", get(crate::ui_server::index))
```

### 3. Path Updates in ui_server/mod.rs
```rust
// Before
match fs::read_to_string("web/desktop/index.html")
let static_path = PathBuf::from("./web/desktop");

// After
match fs::read_to_string("ui/desktop/index.html")
let static_path = PathBuf::from("./ui/desktop");
```

### 4. Configuration Updates

#### tauri.conf.json
```json
// Before
"frontendDist": "./web/desktop"

// After
"frontendDist": "./ui/desktop"
```

#### Cargo.toml Features
```toml
# Before
default = ["web-server", ...]
desktop = [..., "web-server"]
web-server = []
minimal = ["web-server", "chat"]
lightweight = ["web-server", "chat", "drive", "tasks"]

# After
default = ["ui-server", ...]
desktop = [..., "ui-server"]
ui-server = []
minimal = ["ui-server", "chat"]
lightweight = ["ui-server", "chat", "drive", "tasks"]
```

### 5. Service Name Updates
```rust
// Before (in admin.rs)
name: "web_server".to_string()
service: "web_server".to_string()

// After
name: "ui_server".to_string()
service: "ui_server".to_string()
```

## Documentation Updates

### 1. Chapter Structure
- Renamed `chapter-04/web-interface.md` to `chapter-04/ui-interface.md`
- Renamed `chapter-09/web-automation.md` to `chapter-09/ui-automation.md`
- Added new **Chapter 10: .gbui Files Reference** with comprehensive documentation

### 2. Terminology Updates Throughout Documentation
- "Web server" → "UI server"
- "Web interface" → "UI interface"
- "Access web interface" → "Access UI interface"
- "Starting web server" → "Starting UI server"

### 3. New .gbui Documentation Structure
```
chapter-10-gbui/
├── README.md           # .gbui overview and reference
├── structure.md        # Template structure details
├── components.md       # Component library reference
├── javascript-api.md   # JavaScript API documentation
├── mobile.md          # Mobile optimization guide
├── custom-templates.md # Creating custom templates
└── embedding.md       # Embedding .gbui files
```

## New .gbui File Format

### Purpose
The `.gbui` (General Bots User Interface) format provides HTML-based templates for bot interfaces with:
- Built-in WebSocket communication
- Theme integration
- Responsive design
- Component library
- JavaScript API

### Available Templates
1. **default.gbui** - Full-featured desktop interface with multiple apps (Chat, Drive, Tasks, Mail)
2. **single.gbui** - Streamlined single-page chat interface with minimal footprint

### Key Features
- Template variables: `{{bot_name}}`, `{{api_endpoint}}`, `{{websocket_url}}`
- Component system with `data-gb-component` attributes
- CSS class prefix: `gb-` for all General Bots components
- Theme variables: `--gb-primary`, `--gb-secondary`, etc.

## Important Notes

### Channel Names Remain Unchanged
The term "web" as a channel identifier (alongside "whatsapp", "teams", etc.) remains unchanged in:
- Channel adapter references
- Session channel identification
- Message routing logic

These refer to the communication channel type, not the server or interface.

### UI Automation Clarification
The UI automation module now encompasses:
- **Web Automation**: Browser automation, web scraping
- **Desktop Automation**: Desktop application control
- **Mobile Automation**: Mobile app UI control
- **Screen Capture**: Cross-platform screenshot and recording capabilities

## Migration Checklist

- [x] Rename `web/` directory to `ui/`
- [x] Create `default.gbui` from `index.html`
- [x] Create `single.gbui` simplified template
- [x] Update module name from `web_server` to `ui_server`
- [x] Update all import statements
- [x] Update path references in code
- [x] Update tauri.conf.json
- [x] Update Cargo.toml features
- [x] Update service names in admin endpoints
- [x] Update documentation terminology
- [x] Create .gbui documentation chapter
- [x] Update SUMMARY.md with new chapter

## Testing Required

After migration, test:
1. UI server starts correctly on port 8080
2. Static files are served from `ui/desktop/`
3. Both .gbui templates load correctly
4. WebSocket connections work
5. Theme switching functions properly
6. Mobile responsiveness is maintained
7. Desktop app (Tauri) builds successfully

## Rollback Plan

If issues occur:
1. Rename `ui/` back to `web/`
2. Revert module name to `web_server`
3. Restore original import statements
4. Update paths back to `web/desktop`
5. Revert configuration files
6. Restore original documentation

## Benefits of Migration

1. **Clearer Naming**: "UI" better represents the interface layer
2. **.gbui Format**: Standardized template format for bot interfaces
3. **Better Documentation**: Dedicated chapter for UI templates
4. **Extensibility**: Easy to add new template types (mobile, kiosk, embedded)
5. **Consistency**: Aligns with other .gb* file formats (.gbapp, .gbot, .gbtheme, etc.)