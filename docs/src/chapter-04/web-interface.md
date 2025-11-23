# Web Interface

The **gbtheme** web interface provides the front-end experience for end users through a simple REST API architecture.

## Interface Components

| Component | Purpose |
|-----------|---------|
| Chat UI | Main conversation interface with input and message display |
| REST API | HTTP endpoints for message exchange |
| CSS Theme | Visual customization through CSS variables |

## REST API Endpoints

The bot communicates through standard HTTP REST endpoints:

```
POST /api/message      Send user message
GET  /api/session      Get session info
POST /api/upload       Upload files
GET  /api/history      Get conversation history
```

## Message Flow

1. **User Input** - User types message in chat interface
2. **API Call** - Frontend sends POST to `/api/message`
3. **Processing** - Server processes with LLM and tools
4. **Response** - JSON response with bot message
5. **Display** - Frontend renders response in chat

## Simple Integration

```javascript
// Send message to bot
async function sendMessage(text) {
  const response = await fetch('/api/message', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ message: text })
  });
  const data = await response.json();
  displayMessage(data.response);
}
```

## Theme Customization

The interface uses CSS variables for easy customization:

```css
/* In your theme's default.css */
:root {
  --primary-color: #0d2b55;
  --secondary-color: #fff9c2;
  --background: #ffffff;
  --text-color: #333333;
  --font-family: 'Inter', sans-serif;
}
```

## Response Format

The API returns simple JSON responses:

```json
{
  "response": "Bot message text",
  "session_id": "uuid",
  "timestamp": "2024-01-01T12:00:00Z",
  "tools_used": ["weather", "calendar"]
}
```

## File Uploads

Users can upload files through the standard multipart form:

```
POST /api/upload
Content-Type: multipart/form-data

Returns:
{
  "file_id": "uuid",
  "status": "processed",
  "extracted_text": "..."
}
```

## Session Management

Sessions are handled automatically through cookies or tokens:
- Session created on first message
- Persisted across conversations
- Context maintained server-side

## Mobile Responsive

The default interface is mobile-first:
- Touch-friendly input
- Responsive layout
- Optimized for small screens
- Progressive enhancement

## Accessibility

Built-in accessibility features:
- Keyboard navigation
- Screen reader support
- High contrast mode support
- Focus indicators

## Performance

Optimized for speed:
- Minimal JavaScript
- CSS-only animations
- Lazy loading
- CDN-ready assets

## Browser Support

Works on all modern browsers:
- Chrome 90+
- Firefox 88+
- Safari 14+
- Edge 90+
- Mobile browsers

## Integration Examples

### Embed in Website
```html
<iframe src="https://bot.example.com" 
        width="400" 
        height="600">
</iframe>
```

### Custom Frontend
```javascript
// Use any frontend framework
const BotClient = {
  async send(message) {
    return fetch('/api/message', {
      method: 'POST',
      body: JSON.stringify({ message })
    });
  }
};
```

## Security

- CORS configured for embedding
- CSRF protection on POST requests
- Rate limiting per session
- Input sanitization
- XSS prevention

All theming is done through simple CSS files as described in the [Theme Structure](./structure.md).