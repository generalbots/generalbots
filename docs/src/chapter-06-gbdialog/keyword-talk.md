# TALK Keyword

Sends a message to the user through the current channel.

## Syntax
```
TALK message
```

## Parameters
- `message` - The text to send to the user (string expression)

## Description
The `TALK` keyword outputs a message to the user through whatever channel the conversation is happening on (web, voice, WhatsApp, etc.). This is the primary way for the bot to communicate with users.

## Examples

### Basic Usage
```basic
TALK "Hello! Welcome to our service."
```

### With Variables
```basic
SET user_name = "John"
TALK "Hello, " + user_name + "! How can I help you today?"
```

### Multi-line Messages
```basic
TALK "Here are your options:" + CHR(10) + "1. Check balance" + CHR(10) + "2. Make payment"
```

## Usage Notes

- Messages are sent immediately when the TALK command executes
- Multiple TALK commands in sequence will send multiple messages
- The message content can include variables and expressions
- Special characters and emoji are supported
- Message length may be limited by the channel (e.g., SMS character limits)

## Channel Behavior

Different channels may handle TALK messages differently:

- **Web**: Messages appear in the chat interface
- **Voice**: Text is converted to speech
- **WhatsApp**: Sent as text messages
- **Email**: Added to email conversation thread

## Related Keywords
- `HEAR` - Receive user input
- `WAIT` - Pause before sending
- `FORMAT` - Format message content
