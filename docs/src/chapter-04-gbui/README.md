# Chapter 04: .gbui Interface Reference

User interfaces for General Bots.

## What You'll Learn

- Built-in UI options
- Desktop vs web interface
- Console mode for servers
- How to choose an interface

## Available Interfaces

### default.gbui - Full Desktop
- Complete chat interface
- Side panel for history
- Rich message formatting
- Best for: Desktop users

### single.gbui - Simple Chat
- Minimal chat window
- Mobile-friendly
- No distractions
- Best for: Embedded bots, mobile

### Console Mode
- Terminal-based interface
- No GUI required
- Server deployments
- Best for: Headless systems

## How It Works

1. **Auto-selection**: System picks best UI based on environment
2. **Override**: Specify UI in config if needed
3. **Fallback**: Console mode when no GUI available

## Key Features

- WebSocket real-time messaging
- Markdown support
- File uploads
- Session persistence
- Auto-reconnect

## Topics Covered

- [default.gbui - Full Desktop](./default-gbui.md) - Desktop interface details
- [single.gbui - Simple Chat](./single-gbui.md) - Minimal interface
- [Console Mode](./console-mode.md) - Terminal interface

---

<div align="center">
  <img src="https://pragmatismo.com.br/icons/general-bots-text.svg" alt="General Bots" width="200">
</div>