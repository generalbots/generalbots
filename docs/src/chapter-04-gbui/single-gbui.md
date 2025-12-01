# single.gbui - Simplified Chat Interface

The `single.gbui` template provides a streamlined, single-page chat interface focused on conversation without distractions.

## Overview

Location: `ui/suite/single.gbui`

This minimalist chat interface delivers a clean, focused chat experience with WebSocket real-time messaging, dark mode support, mobile-responsive design, and fast loading under 50KB.

## Features

### Core Components

The interface consists of four main components. The header displays the bot name, status, and connection indicator with minimal branding. The messages area provides an auto-scrolling message list with clear user and bot message distinction, timestamps, and smooth animations. The input area offers a single-line text input with a send button, Enter key support, and auto-focus on load. The typing indicator shows a three-dot animation when the bot is processing a response.

## Design Philosophy

The single.gbui template embraces minimalism by eliminating unnecessary UI elements. Speed is prioritized so the interface loads instantly and works on slow connections. Accessibility features include keyboard navigation and screen reader support. Visual clarity comes from a clear hierarchy that guides users naturally through the conversation.

## Responsive Behavior

### Desktop

On desktop displays, the interface uses a centered container with 800px maximum width for comfortable reading, ample whitespace, and optimal line lengths for extended conversations.

### Mobile

On mobile devices, the layout expands to full width with larger touch targets meeting the 44px minimum requirement. The input remains bottom-aligned and adjusts appropriately when the virtual keyboard appears.

## Styling

The interface uses minimal inline CSS for maximum performance:

```css
/* Core styles only */
body {
  font-family: system-ui, -apple-system, sans-serif;
  margin: 0;
  height: 100vh;
  display: flex;
  flex-direction: column;
}

.chat-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  max-width: 800px;
  margin: 0 auto;
  width: 100%;
}
```

## Dark Mode

Automatic dark mode activates based on system preference:

```css
@media (prefers-color-scheme: dark) {
  :root {
    --background: #111827;
    --text: #f9fafb;
    --message-bot: #374151;
  }
}
```

## WebSocket Integration

Connection handling is simplified for reliability:

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  addMessage(data.content, 'bot');
};

function sendMessage() {
  const message = input.value.trim();
  if (message) {
    ws.send(JSON.stringify({
      type: 'message',
      content: message
    }));
    addMessage(message, 'user');
    input.value = '';
  }
}
```

## Use Cases

### Embedded Widget

The single.gbui template is perfect for embedding in existing websites:

```html
<iframe src="http://localhost:8080/ui/suite/single.gbui" 
        width="400" 
        height="600">
</iframe>
```

### Kiosk Mode

The interface works well for public terminals with no navigation elements, focus on conversation, and easy reset between users.

### Mobile-First

Optimization for mobile devices includes fast loading, minimal data usage, and touch-friendly controls.

## Customization

### Change Colors

Edit the CSS variables to match your brand:

```css
:root {
  --primary: #3b82f6;     /* Your brand color */
  --background: #ffffff;   /* Background */
  --text: #1f2937;        /* Text color */
}
```

### Modify Welcome Message

Update the initial bot message in the HTML:

```html
<div class="message bot">
  <div class="message-content">
    Your custom welcome message here
  </div>
</div>
```

### Add Logo

Insert a logo in the header:

```html
<header class="header">
  <img src="logo.png" alt="Logo" height="32">
  <span>Bot Name</span>
</header>
```

## Performance

The single.gbui template achieves first paint in under 100ms and becomes interactive within 200ms. Total size stays under 50KB with no external dependencies since everything is inline.

## Accessibility

The template uses semantic HTML structure throughout, ARIA labels on interactive elements, full keyboard navigation support, proper focus management, and high contrast mode support for users who need it.

## Browser Support

The interface works on all modern browsers including Chrome 90+, Firefox 88+, Safari 14+, Edge 90+, and their mobile counterparts. It degrades gracefully on older browsers, maintaining core functionality.

## See Also

- [default.gbui](./default-gbui.md) - Full-featured interface
- [Console Mode](./console-mode.md) - Terminal interface
- [Chapter 5: Themes](../chapter-05-gbtheme/README.md) - Custom styling
- [Chapter 6: BASIC](../chapter-06-gbdialog/README.md) - Dialog scripting

## Next Step

For terminal users, see [Console Mode](./console-mode.md).