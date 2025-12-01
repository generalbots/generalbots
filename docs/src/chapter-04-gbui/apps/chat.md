# Chat - AI Assistant

> **Your intelligent conversation partner**

<img src="../../assets/suite/chat-flow.svg" alt="Chat Flow Diagram" style="max-width: 100%; height: auto;">

---

## Overview

Chat is the heart of General Bots Suite - your AI-powered assistant that understands context, remembers conversations, and helps you get things done. Built with WebSocket for real-time communication and HTMX for seamless updates.

## Interface Layout

```
┌─────────────────────────────────────────────────────────────────┐
│                     Connection Status [●]                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ 🤖 Bot                                         10:30 AM  │   │
│  │ Hello! How can I help you today?                         │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │                                              You 10:31 AM│   │
│  │ What meetings do I have today?                           │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐   │
│  │ 🤖 Bot                                         10:31 AM  │   │
│  │ You have 2 meetings scheduled:                           │   │
│  │ • 2:00 PM - Team Standup (30 min)                       │   │
│  │ • 4:00 PM - Project Review (1 hour)                     │   │
│  └──────────────────────────────────────────────────────────┘   │
│                                                                  │
├─────────────────────────────────────────────────────────────────┤
│  [📊 Tasks] [📧 Check mail] [📅 Schedule] [❓ Help]             │
├─────────────────────────────────────────────────────────────────┤
│  ┌────────────────────────────────────────────┐ [🎤] [↑]       │
│  │ Type your message...                        │               │
│  └────────────────────────────────────────────┘               │
└─────────────────────────────────────────────────────────────────┘
```

## Features

### Real-Time Messaging

Messages are sent and received instantly via WebSocket connection:

```html
<div id="chat-app" hx-ext="ws" ws-connect="/ws">
    <main id="messages"
          hx-get="/api/sessions/current/history"
          hx-trigger="load"
          hx-swap="innerHTML">
    </main>
    
    <form ws-send>
        <input name="content" type="text" placeholder="Message...">
        <button type="submit">↑</button>
    </form>
</div>
```

### Voice Input

Click the microphone button to speak your message:

1. Click **🎤** to start recording
2. Speak your message clearly
3. Click again to stop
4. Message converts to text automatically

```html
<button type="button" id="voiceBtn"
        hx-post="/api/voice/start"
        hx-swap="none">
    🎤
</button>
```

### Quick Suggestions

Pre-built action chips for common requests:

| Chip | Action |
|------|--------|
| 📊 Tasks | Show your task list |
| 📧 Check mail | Display unread emails |
| 📅 Schedule | Today's calendar |
| ❓ Help | Available commands |

```html
<div class="suggestions-container"
     hx-get="/api/suggestions"
     hx-trigger="load"
     hx-swap="innerHTML">
</div>
```

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

## HTMX Integration

### Message Submission

```html
<form hx-post="/api/sessions/current/message"
      hx-target="#messages"
      hx-swap="beforeend"
      hx-on::after-request="this.reset()">
    <input name="content" type="text" required>
    <button type="submit">Send</button>
</form>
```

### History Loading

```html
<main id="messages"
      hx-get="/api/sessions/current/history"
      hx-trigger="load"
      hx-swap="innerHTML">
    <!-- Messages rendered here -->
</main>
```

### Connection Status

```html
<div id="connectionStatus" class="connection-status disconnected">
    <!-- Updates via WebSocket events -->
</div>
```

## Example Conversations

### Getting Information

```
You: What's the weather like today?
Bot: Currently in your area:
     🌤 Partly cloudy, 72°F (22°C)
     Wind: 8 mph from the west
     Humidity: 45%
```

### Creating Tasks

```
You: Remind me to call John tomorrow at 3pm
Bot: ✅ Task created:
     📋 Call John
     📅 Tomorrow at 3:00 PM
     
     Would you like me to set a notification?
```

### Drafting Emails

```
You: Write an email declining the meeting tomorrow
Bot: Here's a draft:

     Subject: Unable to Attend Tomorrow's Meeting
     
     Hi [Name],
     
     Thank you for the invitation. Unfortunately, I have 
     a scheduling conflict and won't be able to attend 
     tomorrow's meeting.
     
     Would it be possible to reschedule, or could someone 
     share the meeting notes with me afterward?
     
     Best regards,
     [Your name]
     
     [📧 Send] [✏️ Edit] [🗑 Discard]
```

## CSS Classes

```css
.chat-layout {
    display: flex;
    flex-direction: column;
    height: 100%;
}

#messages {
    flex: 1;
    overflow-y: auto;
    padding: 1rem;
}

.message {
    margin-bottom: 1rem;
    padding: 0.75rem 1rem;
    border-radius: 12px;
    max-width: 80%;
}

.message.bot {
    background: var(--surface);
    margin-right: auto;
}

.message.user {
    background: var(--primary);
    color: white;
    margin-left: auto;
}

.suggestions-container {
    display: flex;
    gap: 0.5rem;
    padding: 0.5rem 1rem;
    overflow-x: auto;
}

.suggestion-chip {
    padding: 0.5rem 1rem;
    border-radius: 20px;
    background: var(--surface);
    cursor: pointer;
    white-space: nowrap;
}

.input-container {
    display: flex;
    gap: 0.5rem;
    padding: 1rem;
    border-top: 1px solid var(--border);
}

.connection-status {
    height: 4px;
    transition: background 0.3s;
}

.connection-status.connected {
    background: var(--success);
}

.connection-status.disconnected {
    background: var(--error);
}
```

## JavaScript Events

```javascript
// Connection status handling
document.body.addEventListener('htmx:wsOpen', () => {
    document.getElementById('connectionStatus')
        .classList.replace('disconnected', 'connected');
});

document.body.addEventListener('htmx:wsClose', () => {
    document.getElementById('connectionStatus')
        .classList.replace('connected', 'disconnected');
});

// Auto-scroll to new messages
document.body.addEventListener('htmx:afterSwap', (e) => {
    if (e.detail.target.id === 'messages') {
        e.detail.target.scrollTop = e.detail.target.scrollHeight;
    }
});

// Voice input handling
document.getElementById('voiceBtn').addEventListener('click', async () => {
    const recognition = new webkitSpeechRecognition();
    recognition.onresult = (event) => {
        document.getElementById('messageInput').value = 
            event.results[0][0].transcript;
    };
    recognition.start();
});
```

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