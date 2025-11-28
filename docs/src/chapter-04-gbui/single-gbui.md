# single.gbui - Simplified Chat Interface

The `single.gbui` template provides a streamlined, single-page chat interface focused on conversation without distractions.

## Overview

Location: `ui/suite/single.gbui`

A minimalist chat interface that includes:
- Clean, focused chat experience
- WebSocket real-time messaging
- Dark mode support
- Mobile-responsive design
- Fast loading (< 50KB)

## Features

### Core Components

1. **Header**
   - Bot name and status
   - Connection indicator
   - Minimal branding

2. **Messages Area**
   - Auto-scrolling message list
   - User/bot message distinction
   - Timestamps
   - Smooth animations

3. **Input Area**
   - Single-line text input
   - Send button
   - Enter key support
   - Auto-focus on load

4. **Typing Indicator**
   - Three-dot animation
   - Shows bot processing

## Design Philosophy

- **Minimalism**: No unnecessary UI elements
- **Speed**: Loads instantly, works on slow connections
- **Accessibility**: Keyboard navigation, screen reader support
- **Clarity**: Clear visual hierarchy

## Responsive Behavior

### Desktop
- Centered 800px max-width container
- Comfortable reading width
- Ample whitespace

### Mobile
- Full-width layout
- Larger touch targets (44px minimum)
- Bottom-aligned input
- Virtual keyboard aware

## Styling

Uses minimal inline CSS for maximum performance:

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

Automatic dark mode based on system preference:

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

Simplified connection handling:

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
Perfect for embedding in existing websites:

```html
<iframe src="http://localhost:8080/ui/suite/single.gbui" 
        width="400" 
        height="600">
</iframe>
```

### Kiosk Mode
Ideal for public terminals:
- No navigation elements
- Focus on conversation
- Easy to reset

### Mobile-First
Optimized for mobile devices:
- Fast loading
- Minimal data usage
- Touch-friendly

## Customization

### Change Colors

Edit the CSS variables:

```css
:root {
  --primary: #3b82f6;     /* Your brand color */
  --background: #ffffff;   /* Background */
  --text: #1f2937;        /* Text color */
}
```

### Modify Welcome Message

Update the initial bot message:

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

- **First Paint**: < 100ms
- **Interactive**: < 200ms
- **Total Size**: < 50KB
- **No External Dependencies**: Everything inline

## Accessibility

- Semantic HTML structure
- ARIA labels on interactive elements
- Keyboard navigation support
- Focus management
- High contrast mode support

## Browser Support

Works on all modern browsers:
- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+
- Mobile browsers

Even degrades gracefully on older browsers.

## See Also

- [default.gbui](./default-gbui.md) - Full-featured interface
- [Console Mode](./console-mode.md) - Terminal interface
- [Chapter 5: Themes](../chapter-05-gbtheme/README.md) - Custom styling
- [Chapter 6: BASIC](../chapter-06-gbdialog/README.md) - Dialog scripting

## Next Step

For terminal users, see [Console Mode](./console-mode.md).