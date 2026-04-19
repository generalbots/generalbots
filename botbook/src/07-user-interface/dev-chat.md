# Dev Chat Widget

> **Test Your App Without Leaving the Page**

---

## Overview

The Dev Chat Widget is a floating chat interface that lets you interact with your bot while developing. Talk to modify files, query data, or test features - all without leaving your app.

---

## Activating Dev Mode

The widget appears automatically when:

- URL contains `?dev=1`
- Cookie `dev_mode=1` is set
- Running on `localhost` or `127.0.0.1`

---

## Usage

### Opening the Chat

- **Click** the floating purple button (bottom-right)
- **Keyboard:** `Ctrl+Shift+D`

### Closing

- Click the X button
- Press `Escape`
- Click the floating button again

---

## Quick Actions

| Button | Command | What It Does |
|--------|---------|--------------|
| 📋 Tables | `show tables` | List all tables in your app |
| 📁 Files | `list files` | Show app files |
| 🔄 Reload | `reload app` | Refresh the page |
| ⚠️ Errors | `show errors` | Display any errors |
| 🗑️ Clear | - | Clear chat history |

---

## Example Commands

### Query Data

```
Show all customers
```

```
Find sales from last week
```

```
Count products with stock < 10
```

### Modify App

```
Add a notes field to the customer form
```

```
Change the status options to: new, in-progress, done
```

```
Make the table sortable by date
```

### File Operations

```
Show me index.html
```

```
Add a search box to the header
```

```
Update the page title to "My CRM"
```

---

## Adding to Your App

### Automatic (Dev Mode)

Apps generated with autonomous tasks include the dev chat automatically in dev mode.

### Manual

Add the script to any page:

```html
<script src="/_assets/dev-chat.js"></script>
```

---

## Data Storage

Chat history is stored in the `user_data` virtual table:

```
Namespace: dev_chat
Key: {app_name}_history
```

This means:
- History persists across sessions
- Each app has separate history
- No additional tables needed

---

## WebSocket Connection

The widget connects via WebSocket for real-time updates:

```
ws://localhost:9000/ws/dev
```

When connected:
- Instant responses
- File change notifications
- Auto-reload on updates

Fallback to HTTP if WebSocket unavailable.

---

## File Change Indicators

When botserver modifies files, you'll see:

| Icon | Type | Meaning |
|------|------|---------|
| ➕ | created | New file added |
| ✏️ | modified | File updated |
| ➖ | deleted | File removed |

After file changes, the app auto-reloads.

---

## JavaScript API

Access the widget programmatically:

```javascript
// Open the chat
gbDevChat.open();

// Close the chat
gbDevChat.close();

// Send a message
gbDevChat.send("show tables");

// Add a custom message
gbDevChat.addMessage("Custom notification", "system");

// Show a file change
gbDevChat.showFileChange("index.html", "modified");
```

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+D` | Toggle chat |
| `Escape` | Close chat |
| `Enter` | Send message |
| `Shift+Enter` | New line in message |

---

## Styling

The widget uses CSS custom properties you can override:

```css
#gb-dev-chat-btn {
    --dev-primary: #667eea;
    --dev-secondary: #764ba2;
}
```

---

## Security

The dev chat widget:

- Only loads in dev mode (localhost or explicit flag)
- Requires same authentication as your app
- All operations go through botserver's permission system
- Never exposed in production builds

---

## Troubleshooting

### Widget Not Appearing

1. Check you're in dev mode (`?dev=1` or localhost)
2. Verify the script is loaded
3. Check browser console for errors

### Connection Failed

1. Ensure botserver is running
2. Check WebSocket endpoint is accessible
3. Falls back to HTTP automatically

### Commands Not Working

1. Check you have permissions
2. Verify the app context is correct
3. Try more specific commands

---

## See Also

- [Autonomous Tasks](../17-autonomous-tasks/README.md) - How apps are generated
- [HTMX Architecture](./htmx-architecture.md) - Frontend patterns
- [REST API](../08-rest-api-tools/README.md) - API reference