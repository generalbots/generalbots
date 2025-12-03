# Console Module (XtreeUI)

Terminal-based admin interface for managing General Bots instances.

## Overview

XtreeUI is a TUI (Terminal User Interface) for administering bots directly from the command line. It provides file browsing, log viewing, chat testing, and status monitoring in a single terminal window.

## Feature Flag

Enabled via Cargo feature:

```toml
[features]
console = []
```

## Panels

| Panel | Key | Description |
|-------|-----|-------------|
| File Tree | `1` | Browse bot files and packages |
| Editor | `2` | View/edit configuration files |
| Status | `3` | System status and metrics |
| Logs | `4` | Real-time log viewer |
| Chat | `5` | Test bot conversations |

## Keyboard Navigation

| Key | Action |
|-----|--------|
| `1-5` | Switch between panels |
| `Tab` | Cycle panels |
| `↑/↓` | Navigate within panel |
| `Enter` | Select/open item |
| `q` | Quit console |
| `?` | Show help |

## Components

### File Tree

Browse `.gbai` folder structure:
- View packages (.gbkb, .gbdialog, .gbtheme)
- Open config.csv for editing
- Navigate bot resources

### Status Panel

Real-time system metrics:
- CPU/memory usage
- Active connections
- Bot status
- Database connectivity

### Log Panel

Live log streaming with filtering:
- Error highlighting
- Log level filtering
- Search functionality

### Chat Panel

Interactive bot testing:
- Send messages to bot
- View responses
- Debug conversation flow

### Editor

Basic file editing:
- Syntax highlighting
- Save/reload files
- Config validation

## Starting the Console

```bash
./botserver --console
```

Or programmatically:

```rust
let mut ui = XtreeUI::new();
ui.set_app_state(app_state);
ui.start_ui()?;
```

## Progress Channel

Monitor background tasks:

```rust
let (tx, rx) = tokio::sync::mpsc::channel(100);
ui.set_progress_channel(rx);

// Send progress updates
tx.send(ProgressUpdate::new("Loading KB...", 50)).await;
```

## Use Cases

- Server administration without web UI
- SSH-based remote management
- Development and debugging
- Headless server deployments
- Quick configuration changes

## See Also

- [Building from Source](../chapter-07-gbapp/building.md)
- [Bot Configuration](../chapter-08-config/README.md)