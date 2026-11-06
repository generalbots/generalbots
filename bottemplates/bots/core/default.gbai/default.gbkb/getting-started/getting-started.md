# Getting Started with General Bots

## Overview

Welcome to General Bots! This guide will help you understand the basic features available in your default bot installation.

## Available Features

### Calculator

Perform mathematical calculations by asking the bot to calculate expressions.

**Examples:**
- "Calculate 25 * 4"
- "What is 1500 / 12?"
- "Calculate 15% of 200"

### Send Email

Send emails directly through the bot.

**How to use:**
1. Say "Send email" or "Send an email"
2. Provide the recipient's email address
3. Enter the subject line
4. Type your message content

**Example:**
- "Send an email to john@example.com"
- "I need to email my team"

### Send SMS

Send text messages to mobile phones.

**How to use:**
1. Say "Send SMS" or "Send a text message"
2. Provide the phone number (with country code)
3. Enter your message

**Example:**
- "Send SMS to +1234567890"
- "Text message to my contact"

### Translation

Translate text between different languages.

**How to use:**
1. Say "Translate" followed by the text
2. Specify the target language

**Examples:**
- "Translate 'Hello, how are you?' to Spanish"
- "Translate this text to Portuguese"
- "How do you say 'thank you' in French?"

### Weather

Get current weather information for any location.

**How to use:**
1. Ask about the weather for a specific location

**Examples:**
- "What's the weather in New York?"
- "Weather forecast for London"
- "Is it going to rain in Tokyo?"

## Tips for Better Interactions

### Be Specific
The more specific your request, the better the bot can help you. Include relevant details like:
- Email addresses for sending emails
- Phone numbers with country codes for SMS
- City names for weather queries

### Natural Language
You can speak naturally to the bot. It understands various ways of asking for the same thing:
- "Calculate 10 + 5" or "What is 10 plus 5?"
- "Send email" or "I need to email someone"
- "Translate to Spanish" or "How do you say this in Spanish?"

### Confirmation
The bot will ask for confirmation before performing actions like sending emails or SMS to ensure accuracy.

## Extending Your Bot

This default template provides basic functionality. You can extend your bot by:

1. **Adding Knowledge Base**: Create `.md` files in the `.gbkb` folder to give your bot domain-specific knowledge
2. **Creating Dialogs**: Add `.bas` files in the `.gbdialog` folder for custom conversations
3. **Installing Templates**: Add pre-built templates for CRM, HR, helpdesk, and more
4. **Connecting APIs**: Integrate external services for expanded functionality

## Frequently Asked Questions

**Q: How do I add more features to my bot?**
A: Install additional templates or create custom dialog scripts in the `.gbdialog` folder.

**Q: Can the bot remember previous conversations?**
A: Yes, the bot maintains context within a session. For persistent memory, use the memory features in custom dialogs.

**Q: What languages are supported?**
A: The bot supports multiple languages for both interface and translation. Common languages include English, Portuguese, Spanish, French, German, and many others.

**Q: How do I change the bot's appearance?**
A: Modify the `config.csv` file in the `.gbot` folder to change colors, logo, and title.

**Q: Is my data secure?**
A: Yes, all communications are encrypted. Sensitive data like passwords should never be shared in chat.

## Getting Help

If you need assistance:
- Ask the bot "Help" for available commands
- Check the documentation at docs.pragmatismo.com.br
- Contact support for technical issues

## Next Steps

1. Try out each feature to see how it works
2. Explore the template library for pre-built solutions
3. Customize your bot with your own knowledge base
4. Create custom dialogs for your specific use cases

Welcome aboard, and enjoy using General Bots!