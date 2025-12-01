# Chapter 04: User Interface

General Bots UI system built with HTMX and server-side rendering.

## UI Modes

| Mode | Description | Use Case |
|------|-------------|----------|
| **default.gbui** | Full desktop suite | Complete productivity |
| **single.gbui** | Simple chat widget | Embedded chat |
| **console** | Terminal interface | Development/testing |

## Architecture

- **HTMX** - Dynamic updates without JavaScript frameworks
- **Server-Side Rendering** - Fast, SEO-friendly pages
- **Minimal JS** - No build process required

## Quick Access

```
http://localhost:8080           → Main interface
http://localhost:8080/chat      → Chat app
http://localhost:8080/drive     → File manager
http://localhost:8080/console   → Terminal mode
```

## Suite Applications

| App | Purpose |
|-----|---------|
| Chat | AI assistant conversations |
| Drive | File management |
| Tasks | To-do lists |
| Mail | Email client |
| Calendar | Scheduling |
| Meet | Video calls |
| Paper | AI writing |
| Research | AI search |

## Chapter Contents

- [Suite User Manual](./suite-manual.md) - End-user guide
- [UI Structure](./ui-structure.md) - Component layout
- [default.gbui](./default-gbui.md) - Full desktop mode
- [single.gbui](./single-gbui.md) - Chat widget mode
- [Console Mode](./console-mode.md) - Terminal interface
- [HTMX Architecture](./htmx-architecture.md) - Technical details
- [Suite Applications](./apps/README.md) - App documentation
- [How-To Tutorials](./how-to/README.md) - Step-by-step guides

## See Also

- [.gbtheme Package](../chapter-05-gbtheme/README.md) - Styling and themes
- [.gbui Structure](../chapter-02/gbui.md) - Package format