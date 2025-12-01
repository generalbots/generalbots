# Chat - AI Assistant

> **Your intelligent conversation partner**

<img src="../../assets/suite/chat-screen.svg" alt="Chat Interface Screen" style="max-width: 100%; height: auto;">

---

## Overview

Chat is the heart of General Bots Suite - your AI-powered assistant that understands context, remembers conversations, and helps you get things done. Built with WebSocket for real-time communication and HTMX for seamless updates.

---

## Features

### Real-Time Messaging

Messages are sent and received instantly via WebSocket connection.

<div class="wa-chat">
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Hello! How can I help you today?</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>What meetings do I have today?</p>
      <div class="wa-time">10:31</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>You have 2 meetings scheduled:</p>
      <p>• 2:00 PM - Team Standup (30 min)</p>
      <p>• 4:00 PM - Project Review (1 hour)</p>
      <div class="wa-time">10:31</div>
    </div>
  </div>
</div>

### Voice Input

Click the microphone button to speak your message:

1. Click **🎤** to start recording
2. Speak your message clearly
3. Click again to stop
4. Message converts to text automatically

### Quick Suggestions

Pre-built action chips for common requests:

| Chip | Action |
|------|--------|
| 📊 Tasks | Show your task list |
| 📧 Check mail | Display unread emails |
| 📅 Schedule | Today's calendar |
| ❓ Help | Available commands |

### Message History

- Auto-loads previous messages on page open
- Scroll up to load older messages
- Click "Scroll to bottom" button to return to latest

### Markdown Support

Bot responses support full Markdown rendering:

- **Bold** and *italic* text
- `code snippets` and code blocks
- Bullet and numbered lists
- Links and images
- Tables

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Enter` | Send message |
| `Shift+Enter` | New line (without sending) |
| `↑` (Up arrow) | Edit last message |
| `/` | Open command menu |
| `Escape` | Cancel current action |

## API Endpoints

### WebSocket Connection

```
ws://your-server:8080/ws
```

**Message Types:**
- `TEXT (1)` - Regular text messages
- `VOICE (2)` - Voice messages
- `CONTINUE (3)` - Continue interrupted responses
- `CONTEXT (4)` - Context changes
- `SYSTEM (5)` - System messages

### REST Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/sessions/current/message` | POST | Send a message |
| `/api/sessions/current/history` | GET | Get chat history |
| `/api/voice/start` | POST | Start voice recording |
| `/api/voice/stop` | POST | Stop voice recording |
| `/api/suggestions` | GET | Get suggestion chips |



## Example Conversations

### Getting Information

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>What's the weather like today?</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Currently in your area:</p>
      <p>🌤 Partly cloudy, 72°F (22°C)</p>
      <p>Wind: 8 mph from the west</p>
      <p>Humidity: 45%</p>
      <div class="wa-time">10:30</div>
    </div>
  </div>
</div>

### Creating Tasks

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Remind me to call John tomorrow at 3pm</p>
      <div class="wa-time">14:20</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Task created:</p>
      <p>📋 Call John</p>
      <p>📅 Tomorrow at 3:00 PM</p>
      <p>Would you like me to set a notification?</p>
      <div class="wa-time">14:20</div>
    </div>
  </div>
</div>

### Drafting Emails

<div class="wa-chat">
  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Write an email declining the meeting tomorrow</p>
      <div class="wa-time">09:15</div>
    </div>
  </div>
  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Here's a draft:</p>
      <p><strong>Subject:</strong> Unable to Attend Tomorrow's Meeting</p>
      <p>Hi [Name],</p>
      <p>Thank you for the invitation. Unfortunately, I have a scheduling conflict and won't be able to attend tomorrow's meeting.</p>
      <p>Would it be possible to reschedule, or could someone share the meeting notes with me afterward?</p>
      <p>Best regards, [Your name]</p>
      <p>[📧 Send] [✏️ Edit] [🗑 Discard]</p>
      <div class="wa-time">09:15</div>
    </div>
  </div>
</div>



## Accessibility

- Full keyboard navigation
- Screen reader announcements for new messages
- High contrast mode support
- Adjustable font sizes
- ARIA labels on all interactive elements

```html
<main id="messages" 
      role="log" 
      aria-live="polite"
      aria-label="Chat messages">
</main>

<button type="submit" 
        aria-label="Send message"
        title="Send">
    ↑
</button>
```

## Troubleshooting

### Messages Not Sending

1. Check connection status indicator
2. Verify WebSocket is connected
3. Try refreshing the page
4. Check browser console for errors

### Voice Not Working

1. Allow microphone permissions in browser
2. Check device microphone settings
3. Try a different browser
4. Ensure HTTPS connection (required for voice)

### History Not Loading

1. Check network connection
2. Verify API endpoint is accessible
3. Clear browser cache
4. Check for JavaScript errors

## See Also

- [HTMX Architecture](../htmx-architecture.md) - How Chat uses HTMX
- [Suite Manual](../suite-manual.md) - Complete user guide
- [Tasks App](./tasks.md) - Create tasks from chat
- [Mail App](./mail.md) - Email integration