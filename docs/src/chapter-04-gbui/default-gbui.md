# default.gbui - Full Desktop Interface

The `default.gbui` template provides a complete desktop interface with multiple integrated applications for web, desktop, and mobile platforms.

## Overview

Location: `ui/desktop/default.gbui`

The default template includes:
- Multi-application layout (Chat, Drive, Tasks, Mail)
- Responsive design for all screen sizes
- Theme selector with dark mode support
- WebSocket real-time communication
- Keyboard shortcuts for power users

## Features

### Applications

1. **Chat** - Main conversational interface
   - Rich text messages
   - File attachments
   - Voice input
   - Typing indicators
   - Message history

2. **Drive** - File management
   - Browse and upload files
   - Preview documents
   - Share with chat

3. **Tasks** - Task tracking
   - Create and manage tasks
   - Due dates and priorities
   - Integration with chat

4. **Mail** - Email integration
   - Send/receive through bot
   - AI-powered composition
   - Thread management

### Navigation

- **Header Bar**: Logo, app selector, theme switcher, user menu
- **App Grid**: Quick access to all applications
- **Keyboard Shortcuts**:
  - `Alt+1` → Chat
  - `Alt+2` → Drive  
  - `Alt+3` → Tasks
  - `Alt+4` → Mail
  - `Esc` → Close menus

## Responsive Design

### Desktop (>1024px)
- Full multi-panel layout
- Persistent navigation
- Side-by-side applications

### Tablet (768-1024px)
- Collapsible sidebar
- Touch-optimized controls
- Adaptive layouts

### Mobile (<768px)
- Single column layout
- Bottom navigation
- Swipe gestures
- Large touch targets

## Theme Integration

Automatically applies styles from active `.gbtheme`:

```css
:root {
  --gb-primary: /* from theme */
  --gb-background: /* from theme */
  --gb-text: /* from theme */
}
```

## WebSocket Communication

Built-in real-time messaging:

```javascript
// Auto-connects on load
const ws = new WebSocket('ws://localhost:8080/ws');

// Handles reconnection
ws.onclose = () => {
  // Automatic reconnect logic
};
```

## Customization

### Modify Applications

Edit the app grid in the template:

```html
<div class="app-grid">
  <a href="#chat" data-section="chat">Chat</a>
  <a href="#drive" data-section="drive">Drive</a>
  <!-- Add your custom apps here -->
</div>
```

### Change Default Theme

Update theme selector options:

```html
<select class="gb-theme-selector">
  <option value="default">Default</option>
  <option value="dark">Dark</option>
  <!-- Add custom themes -->
</select>
```

## Usage

### As Desktop App

Used automatically when running with `--desktop`:

```bash
./botserver --desktop
# Opens default.gbui in native window
```

### As Web Interface

Default template for browser access:

```bash
./botserver
# Browse to http://localhost:8080
# Loads default.gbui
```

### As Mobile PWA

Install as Progressive Web App:
1. Open in mobile browser
2. Add to home screen
3. Launches as app

## Performance

- **Initial Load**: < 200KB
- **WebSocket Latency**: < 50ms
- **Touch Response**: 60fps animations
- **Offline Support**: Service worker caching

## Browser Support

- Chrome/Edge 90+
- Firefox 88+
- Safari 14+
- Mobile browsers (iOS Safari, Chrome Mobile)

## See Also

- [single.gbui](./single-gbui.md) - Minimal chat interface
- [Console Mode](./console-mode.md) - Terminal interface
- [Chapter 5: Themes](../chapter-05-gbtheme/README.md) - Styling the interface
- [Chapter 6: BASIC](../chapter-06-gbdialog/README.md) - Dialog scripting

## Next Step

For a simpler interface, see [single.gbui](./single-gbui.md).