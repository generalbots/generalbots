# SMS Provider Configuration

This guide covers configuration for SMS messaging in General Bots, supporting multiple providers including Twilio, AWS SNS, Vonage, and MessageBird.

---

## Overview

General Bots supports sending SMS messages through the `SEND SMS` keyword, with automatic provider detection based on your configuration. Multiple providers can be configured, with fallback support for reliability.

The SMS system supports **priority levels** (`low`, `normal`, `high`, `urgent`) that affect delivery routing and message handling based on the provider.

---

## Quick Start

### Minimal Configuration (Twilio)

```csv
name,value
sms-provider,twilio
sms-default-priority,normal
twilio-account-sid,ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
twilio-auth-token,your_auth_token
twilio-from-number,+15551234567
```

### Usage in BASIC

```basic
' Basic SMS (uses default priority)
SEND SMS "+15559876543", "Hello from General Bots!"

' SMS with priority
SEND SMS "+15559876543", "Urgent notification!", "urgent"

' SMS with provider and priority
SEND SMS "+15559876543", "Important message", "twilio", "high"
```

---

## Supported Providers

| Provider | Key | Best For | Priority Support |
|----------|-----|----------|------------------|
| Twilio | `twilio` | General purpose, global reach | Prefix tags |
| AWS SNS | `aws` or `aws_sns` | AWS ecosystem, high volume | SMSType routing |
| Vonage (Nexmo) | `vonage` or `nexmo` | Europe, competitive pricing | Flash messages |
| MessageBird | `messagebird` | Europe, omnichannel | Message class |
| Custom | `custom` | Self-hosted or other providers | Via payload |

---

## Priority Levels

General Bots supports four priority levels for SMS messages:

| Priority | Description | Use Case |
|----------|-------------|----------|
| `low` | Non-urgent, promotional | Marketing, newsletters |
| `normal` | Standard delivery (default) | General notifications |
| `high` | Important, faster routing | Order confirmations, alerts |
| `urgent` | Critical, immediate delivery | Security codes, emergencies |

### Priority Behavior by Provider

| Provider | Low/Normal | High | Urgent |
|----------|------------|------|--------|
| **Twilio** | Standard delivery | `[HIGH]` prefix added | `[URGENT]` prefix added |
| **AWS SNS** | Promotional routing | Transactional routing | Transactional routing |
| **Vonage** | Standard delivery | Standard delivery | Flash message (class 0) |
| **MessageBird** | Standard delivery | Message class 1 | Flash message (class 0) |

### Default Priority Configuration

```csv
name,value
sms-default-priority,normal
```

Set the default priority for all SMS messages when not explicitly specified.

---

## Twilio Configuration

Twilio is the default provider, offering reliable global SMS delivery.

### Required Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `sms-provider` | Set to `twilio` | `twilio` |
| `twilio-account-sid` | Your Twilio Account SID | `ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx` |
| `twilio-auth-token` | Your Twilio Auth Token | `your_auth_token` |
| `twilio-from-number` | Your Twilio phone number | `+15551234567` |

### Priority Handling

Twilio doesn't have native priority routing, so General Bots adds prefixes to high-priority messages:
- **High**: Message prefixed with `[HIGH]`
- **Urgent**: Message prefixed with `[URGENT]`

### Optional Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `twilio-messaging-service-sid` | Messaging Service SID (for advanced routing) | Not set |
| `twilio-status-callback` | Webhook URL for delivery status | Not set |

### Complete Example

```csv
name,value
sms-provider,twilio
sms-default-priority,normal
twilio-account-sid,ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
twilio-auth-token,xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
twilio-from-number,+15551234567
twilio-messaging-service-sid,MGxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
twilio-status-callback,https://yourbot.example.com/webhooks/sms-status
```

### Getting Twilio Credentials

1. Sign up at [twilio.com](https://www.twilio.com)
2. Find your Account SID and Auth Token in the Console Dashboard
3. Purchase a phone number with SMS capabilities
4. Copy the phone number in E.164 format (+1XXXXXXXXXX)

---

## AWS SNS Configuration

Amazon SNS provides high-volume SMS delivery integrated with the AWS ecosystem.

### Required Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `sms-provider` | Set to `aws` or `aws_sns` | `aws` |
| `aws-access-key` | AWS Access Key ID | `AKIAIOSFODNN7EXAMPLE` |
| `aws-secret-key` | AWS Secret Access Key | `wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY` |
| `aws-region` | AWS Region | `us-east-1` |

### Priority Handling

AWS SNS supports native priority routing via SMSType:
- **Low/Normal**: Uses `Promotional` SMSType (cost-optimized)
- **High/Urgent**: Uses `Transactional` SMSType (delivery-optimized)

Transactional messages have higher delivery rates and are prioritized by carriers.

### Optional Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `aws-sns-sender-id` | Sender ID (alphanumeric, where supported) | Not set |
| `aws-sns-message-type` | `Promotional` or `Transactional` | `Transactional` |

### Complete Example

```csv
name,value
sms-provider,aws
sms-default-priority,normal
aws-access-key,AKIAIOSFODNN7EXAMPLE
aws-secret-key,wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY
aws-region,us-east-1
aws-sns-sender-id,MyBot
```

### AWS IAM Policy

Ensure your IAM user has the following permissions:

```json
{
    "Version": "2012-10-17",
    "Statement": [
        {
            "Effect": "Allow",
            "Action": [
                "sns:Publish"
            ],
            "Resource": "*"
        }
    ]
}
```

### Getting AWS Credentials

1. Log into AWS Console
2. Go to IAM → Users → Create User
3. Attach the SNS publish policy
4. Create Access Key and save the credentials securely

---

## Vonage (Nexmo) Configuration

Vonage offers competitive pricing and strong European coverage.

### Required Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `sms-provider` | Set to `vonage` or `nexmo` | `vonage` |
| `vonage-api-key` | Vonage API Key | `abcd1234` |
| `vonage-api-secret` | Vonage API Secret | `AbCdEf123456` |
| `vonage-from-number` | Sender number or name | `+15551234567` or `MyBot` |

### Priority Handling

Vonage supports message classes for priority:
- **Low/Normal/High**: Standard SMS delivery
- **Urgent**: Flash message (class 0) - displays immediately on screen without user interaction

### Optional Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `vonage-callback-url` | Delivery receipt webhook | Not set |
| `vonage-client-ref` | Custom reference for tracking | Not set |

### Complete Example

```csv
name,value
sms-provider,vonage
sms-default-priority,normal
vonage-api-key,abcd1234
vonage-api-secret,AbCdEf123456789
vonage-from-number,+15551234567
vonage-callback-url,https://yourbot.example.com/webhooks/vonage
```

### Getting Vonage Credentials

1. Sign up at [vonage.com](https://www.vonage.com)
2. Find API Key and Secret in the Dashboard
3. Purchase a virtual number or use alphanumeric sender ID

---

## MessageBird Configuration

MessageBird provides omnichannel messaging with strong European presence.

### Required Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `sms-provider` | Set to `messagebird` | `messagebird` |
| `messagebird-access-key` | MessageBird Access Key | `live_xxxxxxxxxxxxx` |
| `messagebird-originator` | Sender number or name | `+15551234567` |

### Priority Handling

MessageBird supports message classes via typeDetails:
- **Low/Normal**: Standard SMS delivery
- **High**: Message class 1
- **Urgent**: Flash message (class 0) - displays immediately on screen

### Optional Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `messagebird-report-url` | Status report webhook | Not set |
| `messagebird-validity` | Message validity in seconds | `86400` |
| `messagebird-gateway` | Gateway ID for routing | Not set |

### Complete Example

```csv
name,value
sms-provider,messagebird
sms-default-priority,normal
messagebird-access-key,live_AbCdEfGhIjKlMnOpQrSt
messagebird-originator,+15551234567
messagebird-report-url,https://yourbot.example.com/webhooks/messagebird
```

### Getting MessageBird Credentials

1. Sign up at [messagebird.com](https://www.messagebird.com)
2. Create an Access Key in the Developers section
3. Purchase a number or configure alphanumeric sender

---

## Custom Provider Configuration

For self-hosted SMS gateways or providers not directly supported.

### Parameters

| Parameter | Description | Example |
|-----------|-------------|---------|
| `sms-provider` | Set to `custom` | `custom` |
| `sms-custom-url` | API endpoint URL | `https://sms.example.com/send` |
| `sms-custom-method` | HTTP method | `POST` |
| `sms-custom-auth-header` | Authorization header | `Bearer your-token` |
| `sms-custom-body-template` | JSON body template | See below |

### Body Template

Use placeholders for dynamic values:

| Placeholder | Description |
|-------------|-------------|
| `{{to}}` | Recipient phone number |
| `{{message}}` | Message content |
| `{{from}}` | Sender number (if configured) |
| `{{priority}}` | Priority level (low, normal, high, urgent) |

### Complete Example

```csv
name,value
sms-provider,custom
sms-default-priority,normal
custom-webhook-url,https://sms.example.com/api/v1/send
custom-api-key,abc123xyz
```

The custom webhook receives a JSON payload:

```json
{
    "to": "+15551234567",
    "message": "Your message content",
    "provider": "custom",
    "priority": "high"
}
```

---

## Using Multiple Providers

Configure fallback providers for reliability:

```csv
name,value
sms-provider,twilio
sms-fallback-provider,vonage

# Primary: Twilio
twilio-account-sid,ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
twilio-auth-token,your_auth_token
twilio-phone-number,+15551234567

# Fallback: Vonage
vonage-api-key,abcd1234
vonage-api-secret,AbCdEf123456789
vonage-from,+15559876543
```

When Twilio fails, General Bots automatically retries with Vonage.

---

## BASIC Keyword Usage

### Basic SMS

```basic
' Send SMS using default provider and priority
SEND SMS "+15559876543", "Your order has shipped!"
```

### SMS with Priority

```basic
' Send urgent notification
SEND SMS "+15559876543", "Security alert: New login detected!", "urgent"

' Send low-priority promotional message
SEND SMS "+15559876543", "Check out our weekend sale!", "low"
```

### SMS with Specific Provider

```basic
' Use AWS for transactional messages with high priority
SEND SMS "+15559876543", "Your verification code is 123456", "aws", "high"
```

### SMS with Error Handling

```basic
result = SEND SMS customer_phone, "Your appointment is tomorrow at " + appointment_time, "high"

IF NOT result.success THEN
    PRINT "SMS failed: " + result.error
    ' Fallback to email
    SEND MAIL customer_email, "Appointment Reminder", 
        "Your appointment is tomorrow at " + appointment_time, []
ELSE
    TALK "Reminder sent! ID: " + result.message_id
END IF
```

### Bulk SMS with Priority

```basic
customers = FIND "customers.csv", "notify_sms = true"

sent = 0
failed = 0

FOR EACH customer IN customers
    ' Use low priority for promotional bulk messages
    result = SEND SMS customer.phone, "Flash sale! 20% off today only.", "low"
    
    IF result.success THEN
        sent = sent + 1
    ELSE
        PRINT "Failed to send to " + customer.phone + ": " + result.error
        failed = failed + 1
    END IF
    
    WAIT 0.5  ' Rate limiting
NEXT

TALK "Sent " + sent + " messages, " + failed + " failed"
```

### Priority-Based Routing Example

```basic
' Route based on message urgency
SUB send_with_priority(phone, message, urgency)
    SELECT CASE urgency
        CASE "critical"
            result = SEND SMS phone, message, "urgent"
        CASE "important"
            result = SEND SMS phone, message, "high"
        CASE "promotional"
            result = SEND SMS phone, message, "low"
        CASE ELSE
            result = SEND SMS phone, message, "normal"
    END SELECT
    
    RETURN result
END SUB

' Usage
send_with_priority(customer.phone, "Your 2FA code: 123456", "critical")
send_with_priority(customer.phone, "Check out our sale!", "promotional")
```

---

## Phone Number Formats

Always use E.164 format for phone numbers:

| Country | Format | Example |
|---------|--------|---------|
| USA/Canada | +1XXXXXXXXXX | +15551234567 |
| UK | +44XXXXXXXXXX | +447911123456 |
| Brazil | +55XXXXXXXXXXX | +5511987654321 |
| Germany | +49XXXXXXXXXXX | +491512345678 |
| India | +91XXXXXXXXXX | +919876543210 |

### Formatting in BASIC

```basic
' Clean and format phone number
phone = REPLACE(raw_phone, " ", "")
phone = REPLACE(phone, "-", "")
phone = REPLACE(phone, "(", "")
phone = REPLACE(phone, ")", "")

IF NOT phone LIKE "+*" THEN
    phone = "+1" + phone  ' Assume US if no country code
END IF

SEND SMS phone, message
```

---

## Best Practices

### 1. Respect Opt-Out

```basic
' Check opt-out status before sending
customer = FIND "customers", "phone = '" + phone + "'"

IF customer.sms_opt_out = true THEN
    PRINT "Customer has opted out of SMS"
    RETURN
END IF

SEND SMS phone, message
```

### 2. Message Length

- Standard SMS: 160 characters (GSM-7 encoding)
- Unicode SMS: 70 characters
- Longer messages are split and may incur additional charges

```basic
' Check message length
IF LEN(message) > 160 THEN
    PRINT "Warning: Message will be split into multiple SMS"
END IF
```

### 3. Rate Limiting

Most providers have rate limits. Implement delays for bulk sending:

```basic
FOR EACH recipient IN recipients
    SEND SMS recipient.phone, message
    WAIT 0.5  ' 2 messages per second max
NEXT
```

### 4. Secure Credentials

Never hardcode credentials. Use `config.csv` or environment variables:

```csv
# In config.csv - this file should not be committed to version control
twilio-account-sid,ACxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
twilio-auth-token,${TWILIO_AUTH_TOKEN}
```

---

## Troubleshooting

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `INVALID_PHONE_NUMBER` | Wrong format | Use E.164 format (+1XXXXXXXXXX) |
| `AUTHENTICATION_ERROR` | Bad credentials | Verify API keys/tokens |
| `INSUFFICIENT_FUNDS` | Account balance low | Top up provider account |
| `RATE_LIMIT_EXCEEDED` | Too many requests | Add delays between messages |
| `UNREGISTERED_PHONE` | Number not active | Verify recipient number |
| `CARRIER_BLOCKED` | Carrier filtering | Contact provider support |

### Testing

Use provider test numbers during development:

```csv
# Twilio Magic Numbers for testing
# +15005550006 - Valid number
# +15005550001 - Invalid number
# +15005550009 - Cannot route
```

### Checking Delivery Status

If webhooks are configured, delivery status is received automatically. Otherwise, check provider dashboards for delivery reports.

---

## Related Documentation

- [SEND SMS Keyword](../06-gbdialog/keyword-sms.md) — BASIC keyword reference
- [Universal Messaging](../06-gbdialog/universal-messaging.md) — Multi-channel messaging
- [Secrets Management](./secrets-management.md) — Secure credential storage
- [WhatsApp Configuration](./whatsapp-channel.md) — WhatsApp setup guide
- [Teams Configuration](./teams-channel.md) — Microsoft Teams setup guide

---

## Summary

SMS messaging in General Bots supports multiple providers with a unified interface and priority levels. Configure your preferred provider in `config.csv`, set a default priority with `sms-default-priority`, then use the `SEND SMS` keyword in your BASIC scripts. 

Priority levels allow you to:
- Use `low` for cost-effective promotional messages
- Use `normal` for standard notifications
- Use `high` for important transactional messages
- Use `urgent` for critical time-sensitive alerts

For production, configure fallback providers and implement proper error handling to ensure message delivery.