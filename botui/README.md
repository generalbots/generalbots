# BotUI - General Bots Web Interface

<p align="center"><img src="../logo.svg" alt="General Bots" width="200"></p>

**Version:** 6.2.0  
**Purpose:** Web UI server for General Bots (Axum + HTMX + CSS)

---

## Overview

BotUI is a modern web interface for General Bots, built with Rust, Axum, and HTMX. It provides a clean, responsive interface for interacting with the General Bots platform, featuring real-time updates via WebSocket connections and a minimalist JavaScript approach powered by HTMX.

The interface supports multiple features including chat, file management, tasks, calendar, analytics, and more - all served through a fast, efficient Rust backend with a focus on server-rendered HTML and minimal client-side JavaScript.

For comprehensive documentation, see **[docs.generalbots.org](https://docs.generalbots.org)** or the **[BotBook](./botbook)** for detailed guides and API references.

---

## Quick Start

```bash
# Development mode - starts Axum server on port 9000
cargo run

# Desktop mode (Tauri) - starts native window
cargo tauri dev
```

### Environment Variables

- `BOTUI_PORT` - Server port (default: 9000)

---

## ZERO TOLERANCE POLICY

**EVERY SINGLE WARNING MUST BE FIXED. NO EXCEPTIONS.**

---

## ABSOLUTE PROHIBITIONS

```
❌ NEVER use #![allow()] or #[allow()] in source code
❌ NEVER use _ prefix for unused variables - DELETE or USE them
❌ NEVER use .unwrap() - use ? or proper error handling
❌ NEVER use .expect() - use ? or proper error handling  
❌ NEVER use panic!() or unreachable!()
❌ NEVER use todo!() or unimplemented!()
❌ NEVER leave unused imports or dead code
❌ NEVER add comments - code must be self-documenting
❌ NEVER use CDN links - all assets must be local
```

---

## ARCHITECTURE

### Dual Modes

| Mode | Command | Description |
|------|---------|-------------|
| Web | `cargo run` | Axum server on port 9000 |
| Desktop | `cargo tauri dev` | Tauri native window |

### Code Organization

```
src/
├── main.rs           # Entry point - mode detection
├── lib.rs            # Feature-gated module exports
├── http_client.rs    # HTTP wrapper for botserver
├── ui_server/
│   └── mod.rs        # Axum router + UI serving
├── desktop/
│   ├── mod.rs        # Desktop module organization
│   ├── drive.rs      # File operations via Tauri
│   └── tray.rs       # System tray
└── shared/
    └── state.rs      # Shared application state

ui/
├── suite/            # Main UI (HTML/CSS/JS)
│   ├── js/vendor/    # Local JS libraries
│   └── css/          # Stylesheets
└── minimal/          # Minimal chat UI
```

---

## HTMX-FIRST FRONTEND

### Core Principle
- **Use HTMX** to minimize JavaScript
- **Server returns HTML fragments**, not JSON
- **Delegate ALL logic** to Rust server

### HTMX Usage

| Use Case | Solution |
|----------|----------|
| Data fetching | `hx-get`, `hx-post` |
| Form submission | `hx-post`, `hx-put` |
| Real-time updates | `hx-ext="ws"` |
| Content swapping | `hx-target`, `hx-swap` |
| Polling | `hx-trigger="every 5s"` |
| Loading states | `hx-indicator` |

### When JS is Required

| Use Case | Why JS Required |
|----------|-----------------|
| Modal show/hide | DOM manipulation |
| Toast notifications | Dynamic element creation |
| Clipboard operations | `navigator.clipboard` API |
| Keyboard shortcuts | `keydown` event handling |
| Complex animations | GSAP or custom |

---

## LOCAL ASSETS ONLY - NO CDN

```
ui/suite/js/vendor/
├── htmx.min.js
├── htmx-ws.js
├── marked.min.js
├── gsap.min.js
└── livekit-client.umd.min.js
```

```html
<!-- ✅ CORRECT -->
<script src="js/vendor/htmx.min.js"></script>

<!-- ❌ WRONG -->
<script src="https://unpkg.com/htmx.org@1.9.10"></script>
```

---

## OFFICIAL ICONS - MANDATORY

**NEVER generate icons with LLM. Use official SVG icons:**

```
ui/suite/assets/icons/
├── gb-logo.svg        # Main GB logo
├── gb-bot.svg         # Bot/assistant
├── gb-analytics.svg   # Analytics
├── gb-calendar.svg    # Calendar
├── gb-chat.svg        # Chat
├── gb-drive.svg       # File storage
├── gb-mail.svg        # Email
├── gb-meet.svg        # Video meetings
├── gb-tasks.svg       # Task management
└── ...
```

All icons use `stroke="currentColor"` for CSS theming.

---

## SECURITY ARCHITECTURE

### Centralized Auth Engine

All authentication is handled by `security-bootstrap.js` which MUST be loaded immediately after HTMX:

```html
<head>
    <!-- 1. HTMX first -->
    <script src="js/vendor/htmx.min.js"></script>
    <script src="js/vendor/htmx-ws.js"></script>
    
    <!-- 2. Security bootstrap immediately after -->
    <script src="js/security-bootstrap.js"></script>
    
    <!-- 3. Other scripts -->
    <script src="js/api-client.js"></script>
</head>
```

### DO NOT Duplicate Auth Logic

```javascript
// ❌ WRONG - Don't add auth headers manually
fetch("/api/data", {
    headers: { "Authorization": "Bearer " + token }
});

// ✅ CORRECT - Let security-bootstrap.js handle it
fetch("/api/data");
```

---

## DESIGN SYSTEM

### Layout Standards

```css
.app-container {
    display: flex;
    flex-direction: column;
    height: 100vh;
    overflow: hidden;
}

.main-content {
    display: grid;
    grid-template-columns: 320px 1fr;
    flex: 1;
    overflow: hidden;
}

.list-panel {
    overflow-y: scroll;
    scrollbar-width: auto;
}

.detail-panel {
    display: flex;
    flex-direction: column;
    overflow: hidden;
}
```

### Theme Variables Required

```css
[data-theme="your-theme"] {
    --bg: #0a0a0a;
    --surface: #161616;
    --surface-hover: #1e1e1e;
    --border: #2a2a2a;
    --text: #ffffff;
    --text-secondary: #888888;
    --primary: #c5f82a;
    --success: #22c55e;
    --warning: #f59e0b;
    --error: #ef4444;
}
```

---

## CODE PATTERNS

### Error Handling

```rust
// ❌ WRONG
let value = something.unwrap();

// ✅ CORRECT
let value = something?;
let value = something.ok_or_else(|| Error::NotFound)?;
```

### Self Usage

```rust
impl MyStruct {
    fn new() -> Self { Self { } }  // ✅ Not MyStruct
}
```

### Format Strings

```rust
format!("Hello {name}")  // ✅ Not format!("{}", name)
```

### Derive Eq with PartialEq

```rust
#[derive(PartialEq, Eq)]  // ✅ Always both
struct MyStruct { }
```

---

## KEY DEPENDENCIES

| Library | Version | Purpose |
|---------|---------|---------|
| axum | 0.7.5 | Web framework |
| reqwest | 0.12 | HTTP client |
| tokio | 1.41 | Async runtime |
| askama | 0.12 | HTML Templates |

---

## Documentation

For complete documentation, guides, and API references:

- **[docs.generalbots.org](https://docs.generalbots.org)** - Full online documentation
- **[BotBook](./botbook)** - Local comprehensive guide
- **[General Bots Repository](https://github.com/GeneralBots/BotServer)** - Main project repository

---

## REMEMBER

- **ZERO WARNINGS** - Every clippy warning must be fixed
- **NO ALLOW IN CODE** - Never use #[allow()] in source files
- **NO DEAD CODE** - Delete unused code
- **NO UNWRAP/EXPECT** - Use ? operator
- **HTMX first** - Minimize JS, delegate to server
- **Local assets** - No CDN, all vendor files local
- **No business logic** - All logic in botserver
- **HTML responses** - Server returns fragments, not JSON
- **Version 6.2.0** - do not change without approval