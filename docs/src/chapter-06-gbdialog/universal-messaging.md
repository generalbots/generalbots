# Universal Messaging & Multi-Channel

BotServer automatically handles conversations across different channels (Web, WhatsApp, Email, etc.) using the same BASIC scripts. Write once, deploy everywhere.

## How It Works

Your BASIC scripts don't need to know which channel they're running on. The same `TALK` and `HEAR` commands work universally:

```basic
TALK "Hello! How can I help you?"
HEAR response
TALK "You said: " + response
```

This script works identically whether the user is:
- Chatting via web browser
- Messaging on WhatsApp
- Sending emails
- Using Microsoft Teams

## Supported Channels

### Web (Default)
The primary channel. Users access via browser at `http://localhost:8080`.

### WhatsApp Business
Requires WhatsApp Business API configuration. Messages are automatically formatted for WhatsApp's constraints.

### Email
Bots can receive and respond to emails. Each email thread becomes a conversation session.

### Microsoft Teams
Deploy bots directly to Teams channels and direct messages.

## Channel Detection

BotServer automatically detects the channel based on the session context. No special code needed:

```basic
' This works on ALL channels
TALK "Welcome to our service!"
TALK "What's your name?"
HEAR name
TALK "Nice to meet you, " + name
```

## Channel-Specific Formatting

While your code stays the same, BotServer automatically handles channel-specific formatting:

### Web
- Full HTML support
- Rich formatting
- Images and media
- Interactive elements

### WhatsApp
- Plain text with emoji
- Media as attachments
- Quick reply buttons
- 1024 character limit per message

### Email
- HTML email format
- Subject line handling
- Attachments
- Proper threading

### Teams
- Adaptive cards
- @mentions
- Channel vs DM detection
- Teams-specific formatting

## Media Handling

Send files and media universally:

```basic
' Works on all channels that support files
SEND FILE "report.pdf"
TALK "I've sent you the report."
```

Each channel handles files appropriately:
- Web: Download link
- WhatsApp: Document attachment
- Email: Email attachment
- Teams: File card

## Session Management

Each channel maintains its own session handling:

- **Web**: Cookie-based sessions
- **WhatsApp**: Phone number as session ID
- **Email**: Thread ID as session
- **Teams**: User/channel context

## Configuration

Channel configuration is done in the bot's `config.csv`:

```csv
channel-web,enabled
channel-whatsapp,enabled
channel-email,enabled
channel-teams,disabled
```

## Best Practices

1. **Keep messages concise** - Some channels have length limits
2. **Use simple formatting** - Not all channels support rich text
3. **Test on target channels** - Ensure your bot works well on each
4. **Handle media gracefully** - Not all channels support all file types
5. **Consider response times** - Email is async, chat is real-time

## Channel Limitations

| Channel | Message Length | Media Support | Rich Text | Real-time |
|---------|---------------|---------------|-----------|-----------|
| Web | Unlimited | Full | Yes | Yes |
| WhatsApp | 1024 chars | Images, Docs | Limited | Yes |
| Email | Unlimited | Attachments | HTML | No |
| Teams | 28KB | Full | Adaptive Cards | Yes |


## Summary

Universal messaging means your BASIC scripts work across all channels without modification. BotServer handles the complexity of channel-specific formatting and delivery, letting you focus on the conversation logic.
