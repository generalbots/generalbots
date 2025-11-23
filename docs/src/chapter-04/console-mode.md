# Console Mode

BotServer includes a powerful terminal-based UI for monitoring, debugging, and managing bots directly from the console.

## Overview

Console mode (`--console`) provides a text-based user interface (TUI) using the terminal for full system control without a web browser.

## Launching Console Mode

```bash
# Start BotServer with console UI
./botserver --console

# Console with specific bot
./botserver --console --bot edu

# Console with custom refresh rate
./botserver --console --refresh 500
```

## UI Tree Structure

The console interface uses a tree-based layout for navigation and display:

```
┌─ BotServer Console ────────────────────────────────┐
│                                                     │
│ ▼ System Status                                    │
│   ├─ CPU: 45%                                      │
│   ├─ Memory: 2.3GB / 8GB                          │
│   ├─ Uptime: 2d 14h 23m                           │
│   └─ Active Sessions: 127                         │
│                                                     │
│ ▼ Bots                                            │
│   ├─ ● default (8 sessions)                       │
│   ├─ ● edu (45 sessions)                          │
│   ├─ ○ crm (offline)                              │
│   └─ ● announcements (74 sessions)                │
│                                                     │
│ ▶ Services                                        │
│ ▶ Logs                                            │
│ ▶ Sessions                                        │
│                                                     │
└─────────────────────────────────────────────────────┘
[q]uit [↑↓]navigate [←→]expand [enter]select [h]elp
```

## Navigation

### Keyboard Controls

| Key | Action |
|-----|--------|
| `↑` `↓` | Navigate up/down |
| `←` `→` | Collapse/expand nodes |
| `Enter` | Select/activate item |
| `Tab` | Switch panels |
| `Space` | Toggle item |
| `q` | Quit console |
| `h` | Show help |
| `f` | Filter/search |
| `r` | Refresh display |
| `/` | Quick search |
| `Esc` | Cancel operation |

### Mouse Support

When terminal supports mouse:
- Click to select items
- Double-click to expand/collapse
- Scroll wheel for navigation
- Right-click for context menu

## Console Components

### System Monitor

Real-time system metrics display:
```
System Resources
├─ CPU
│  ├─ Core 0: 23%
│  ├─ Core 1: 45%
│  ├─ Core 2: 67%
│  └─ Core 3: 12%
├─ Memory
│  ├─ Used: 4.2GB
│  ├─ Free: 3.8GB
│  └─ Swap: 0.5GB
└─ Disk
   ├─ /: 45GB/100GB
   └─ /data: 234GB/500GB
```

### Bot Manager

Interactive bot control panel:
```
Bots Management
├─ default.gbai [RUNNING]
│  ├─ Status: Active
│  ├─ Sessions: 23
│  ├─ Memory: 234MB
│  ├─ Requests/min: 45
│  └─ Actions
│     ├─ [R]estart
│     ├─ [S]top
│     ├─ [C]onfig
│     └─ [L]ogs
```

### Service Dashboard

Monitor all services:
```
Services
├─ Database [✓]
│  ├─ Type: PostgreSQL
│  ├─ Connections: 12/100
│  └─ Response: 2ms
├─ Cache [✓]
│  ├─ Type: Valkey
│  ├─ Memory: 234MB
│  └─ Hit Rate: 94%
├─ Storage [✓]
│  ├─ Type: Drive (S3-compatible)
│  ├─ Buckets: 21
│  └─ Usage: 45GB
└─ Vector DB [✗]
   └─ Status: Offline
```

### Session Viewer

Live session monitoring:
```
Active Sessions
├─ Session #4f3a2b1c
│  ├─ User: john@example.com
│  ├─ Bot: edu
│  ├─ Duration: 00:12:34
│  ├─ Messages: 23
│  └─ State: active
├─ Session #8d9e7f6a
│  ├─ User: anonymous
│  ├─ Bot: default
│  ├─ Duration: 00:03:21
│  ├─ Messages: 7
│  └─ State: idle
```

### Log Viewer

Filtered log display with levels:
```
Logs [ERROR|WARN|INFO|DEBUG]
├─ 12:34:56 [INFO] Bot 'edu' started
├─ 12:34:57 [DEBUG] Session created: 4f3a2b1c
├─ 12:34:58 [WARN] Cache miss for key: user_123
├─ 12:35:01 [ERROR] Database connection timeout
│  └─ Details: Connection pool exhausted
├─ 12:35:02 [INFO] Reconnecting to database...
```

## Console Features

### Real-time Updates

- Auto-refresh configurable (100ms - 10s)
- WebSocket-based live data
- Efficient diff rendering
- Smooth scrolling

### Interactive Commands

```
Commands (press : to enter command mode)
:help              Show help
:quit              Exit console
:restart <bot>     Restart specific bot
:stop <bot>        Stop bot
:start <bot>       Start bot
:clear             Clear screen
:export <file>     Export logs
:filter <pattern>  Filter display
:connect <session> Connect to session
```

### BASIC Debugger Integration

Debug BASIC scripts directly in console:
```
BASIC Debugger - enrollment.bas
├─ Breakpoints
│  ├─ Line 12: PARAM validation
│  └─ Line 34: SAVE operation
├─ Variables
│  ├─ name: "John Smith"
│  ├─ email: "john@example.com"
│  └─ course: "Computer Science"
├─ Call Stack
│  ├─ main()
│  ├─ validate_input()
│  └─ > save_enrollment()
└─ Controls
   [F5]Run [F10]Step [F11]Into [F9]Break
```

### Performance Monitoring

```
Performance Metrics
├─ Response Times
│  ├─ P50: 45ms
│  ├─ P90: 123ms
│  ├─ P95: 234ms
│  └─ P99: 567ms
├─ Throughput
│  ├─ Current: 234 req/s
│  ├─ Average: 189 req/s
│  └─ Peak: 456 req/s
└─ Errors
   ├─ Rate: 0.02%
   └─ Last: 2 min ago
```

## Console Layouts

### Split Views

```
┌─ Bots ─────────┬─ Logs ──────────┐
│                │                  │
│ ● default      │ [INFO] Ready    │
│ ● edu          │ [DEBUG] Session │
│                │                  │
├─ Sessions ─────┼─ Metrics ────────┤
│                │                  │
│ 4f3a2b1c      │ CPU: 45%         │
│ 8d9e7f6a      │ RAM: 2.3GB       │
│                │                  │
└────────────────┴──────────────────┘
```

### Focus Mode

Press `F` to focus on single component:
```
┌─ Focused: Log Viewer ───────────────┐
│                                      │
│ 12:35:01 [ERROR] Connection failed  │
│   Stack trace:                      │
│     at connect() line 234           │
│     at retry() line 123             │
│     at main() line 45               │
│                                      │
│ 12:35:02 [INFO] Retrying...        │
│                                      │
└──────────────────────────────────────┘
[ESC] to exit focus mode
```

## Color Schemes

### Default Theme
- Background: Terminal default
- Text: White
- Headers: Cyan
- Success: Green
- Warning: Yellow
- Error: Red
- Selection: Blue background

### Custom Themes
Configure in `~/.botserver/console.toml`:
```toml
[colors]
background = "#1e1e1e"
foreground = "#d4d4d4"
selection = "#264f78"
error = "#f48771"
warning = "#dcdcaa"
success = "#6a9955"
```

## Console Configuration

### Settings File
`~/.botserver/console.toml`:
```toml
[general]
refresh_rate = 500
mouse_support = true
unicode_borders = true
time_format = "24h"

[layout]
default = "split"
show_tree_lines = true
indent_size = 2

[shortcuts]
quit = "q"
help = "h"
filter = "f"
```

## Performance Considerations

### Terminal Requirements
- Minimum 80x24 characters
- 256 color support recommended
- UTF-8 encoding for borders
- Fast refresh rate capability

### Optimization Tips
- Use `--refresh 1000` for slower terminals
- Disable unicode with `--ascii`
- Limit log tail with `--log-lines 100`
- Filter unnecessary components

## Remote Console

### SSH Access
```bash
# SSH with console auto-start
ssh user@server -t "./botserver --console"

# Persistent session with tmux
ssh user@server -t "tmux attach || tmux new './botserver --console'"
```

### Security
- Read-only mode: `--console-readonly`
- Audit logging of console actions
- Session timeout configuration
- IP-based access control

## Troubleshooting

### Display Issues

1. **Garbled characters**
   - Set `TERM=xterm-256color`
   - Ensure UTF-8 locale
   - Try `--ascii` mode

2. **Slow refresh**
   - Increase refresh interval
   - Reduce displayed components
   - Check network latency (remote)

3. **Colors not working**
   - Verify terminal color support
   - Check TERM environment
   - Try different terminal emulator

## Integration with Development Tools

### VSCode Integration
- Terminal panel for console
- Task runner integration
- Debug console connection

### Tmux/Screen
- Persistent console sessions
- Multiple console windows
- Session sharing for collaboration

## Console API

### Programmatic Access
```python
# Python example
from botserver_console import Console

console = Console("localhost:8080")
console.connect()

# Get system stats
stats = console.get_system_stats()
print(f"CPU: {stats.cpu}%")

# Monitor sessions
for session in console.watch_sessions():
    print(f"Session {session.id}: {session.state}")
```

## Summary

Console mode provides a powerful, efficient interface for managing BotServer without leaving the terminal. Perfect for server administration, debugging, and monitoring in headless environments or over SSH connections.