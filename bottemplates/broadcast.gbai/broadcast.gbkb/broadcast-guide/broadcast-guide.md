# Broadcast Messaging Guide

## Overview

The Broadcast feature allows you to send messages to multiple contacts simultaneously using WhatsApp or other messaging channels. This is ideal for announcements, marketing campaigns, and bulk notifications.

## How to Send a Broadcast

### Basic Broadcast

To send a broadcast message, you need:
1. A message template with optional personalization variables
2. A CSV file containing your contact list

### Message Variables

You can personalize messages using these variables:
- `{name}` - Replaced with the contact's name
- `{mobile}` - Replaced with the contact's phone number

**Example:**
```
Hello {name}, we have exciting news to share with you!
```

### Contact List Format

Your CSV file should have the following columns:
- `name` - Contact's name
- `mobile` - Phone number in international format (e.g., +5511999999999)
- Additional columns can be used for filtering

**Example broadcast.csv:**
```
name,mobile,status
John Smith,+5511999999999,active
Maria Garcia,+5521888888888,active
Carlos Santos,+5531777777777,inactive
```

## Filtering Contacts

You can filter your contact list using conditions:
- `status=active` - Only send to active contacts
- `region=south` - Filter by custom fields
- Multiple filters can be combined

## Best Practices

### Message Content

1. **Keep it concise** - Short messages have higher engagement
2. **Personalize** - Use `{name}` to make messages feel personal
3. **Clear call-to-action** - Tell recipients what to do next
4. **Timing** - Send during appropriate business hours

### Contact Management

1. **Clean your list** - Remove invalid numbers regularly
2. **Respect opt-outs** - Remove contacts who don't want messages
3. **Update regularly** - Keep contact information current
4. **Segment audiences** - Use filters for targeted messaging

### Compliance

1. **Get consent** - Only message contacts who opted in
2. **Identify yourself** - Make clear who is sending the message
3. **Provide opt-out** - Include instructions to unsubscribe
4. **Follow local laws** - LGPD, GDPR, TCPA requirements apply

## Rate Limits

To prevent spam detection and ensure delivery:
- Messages are sent with a 5-second delay between each
- WhatsApp Business API limits apply
- Large broadcasts may take time to complete

## Logging and Tracking

All broadcast operations are logged to `Log.xlsx` with:
- Timestamp
- User who initiated the broadcast
- Recipient mobile number
- Recipient name
- Delivery status

## Troubleshooting

### Messages Not Sending

- Verify phone numbers are in international format
- Check that the CSV file exists and has correct columns
- Ensure the bot has messaging permissions

### Some Contacts Skipped

- Contact may have blocked the number
- Phone number format may be incorrect
- WhatsApp account may not exist for that number

### Slow Delivery

- Large lists take time due to rate limiting
- This is intentional to prevent spam flags
- Check Log.xlsx for progress

## Frequently Asked Questions

**Q: How many contacts can I message at once?**
A: There's no hard limit, but larger lists will take longer due to rate limiting.

**Q: Can I schedule broadcasts for later?**
A: Yes, use scheduled jobs to set up future broadcasts.

**Q: Will I know if messages were delivered?**
A: The log file tracks send status. Delivery confirmation depends on the messaging platform.

**Q: Can I send images or files?**
A: The basic broadcast sends text. For media, use dedicated media broadcast tools.

**Q: How do I stop a broadcast in progress?**
A: Contact an administrator to stop the process if needed.