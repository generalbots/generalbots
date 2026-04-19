# SEND SMS

Send SMS text messages to phone numbers using various providers with optional priority levels.

## Syntax

```basic
' Basic SMS sending (default priority: normal)
SEND SMS phone, message

' With priority level
SEND SMS phone, message, priority

' With specific provider
SEND SMS phone, message, provider

' With provider AND priority (full syntax)
SEND SMS phone, message, provider, priority
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `phone` | String | Yes | Recipient phone number (E.164 format recommended) |
| `message` | String | Yes | The text message to send (max 160 chars for single SMS) |
| `priority` | String | No | Priority level: `low`, `normal`, `high`, `urgent` |
| `provider` | String | No | SMS provider: `twilio`, `aws_sns`, `vonage`, `messagebird`, or custom |

### Priority Levels

| Priority | Description | Provider Behavior |
|----------|-------------|-------------------|
| `low` | Non-urgent, promotional messages | Standard delivery |
| `normal` | Default priority | Standard delivery |
| `high` | Important messages | Transactional routing (AWS SNS), priority prefix |
| `urgent` | Critical/time-sensitive | Flash message (Vonage), [URGENT] prefix (Twilio) |

## Return Value

Returns a map object with the following properties:

| Property | Type | Description |
|----------|------|-------------|
| `success` | Boolean | `true` if SMS was sent successfully |
| `message_id` | String | Provider's message ID for tracking |
| `provider` | String | The provider used to send the message |
| `to` | String | Normalized recipient phone number |
| `priority` | String | The priority level used |
| `error` | String | Error message (only present if `success` is `false`) |

## Configuration

Configure SMS provider credentials in `config.csv`:

```csv
key,value
sms-provider,twilio
sms-default-priority,normal
twilio-account-sid,YOUR_ACCOUNT_SID
twilio-auth-token,YOUR_AUTH_TOKEN
twilio-from-number,+15551234567
```

### Provider-Specific Configuration

**Twilio:**
```csv
sms-provider,twilio
twilio-account-sid,ACxxxxx
twilio-auth-token,your_token
twilio-from-number,+15551234567
```

**AWS SNS:**
```csv
sms-provider,aws_sns
aws-access-key,AKIAXXXXXXXX
aws-secret-key,your_secret
aws-region,us-east-1
```

**Vonage (Nexmo):**
```csv
sms-provider,vonage
vonage-api-key,your_api_key
vonage-api-secret,your_secret
vonage-from-number,+15551234567
```

**MessageBird:**
```csv
sms-provider,messagebird
messagebird-access-key,your_access_key
messagebird-originator,YourBrand
```

## Examples

### Basic SMS

```basic
HEAR phone AS TEXT "Enter phone number:"
SEND SMS phone, "Hello from General Bots!"
TALK "SMS sent successfully!"
```

### SMS with Priority

```basic
' Send urgent notification
result = SEND SMS "+15551234567", "Server is DOWN! Immediate action required.", "urgent"

IF result.success THEN
    TALK "Urgent alert sent with ID: " + result.message_id
ELSE
    TALK "Failed to send alert: " + result.error
END IF
```

### Order Confirmation (Normal Priority)

```basic
' Send order confirmation via SMS
order_id = "ORD-2025-001"
phone = customer.phone

message = "Your order " + order_id + " has been confirmed. "
message = message + "Estimated delivery: 2-3 business days."

result = SEND SMS phone, message, "normal"

IF result.success THEN
    TALK "Confirmation SMS sent to " + phone
ELSE
    TALK "Failed to send SMS. We'll email you instead."
    SEND MAIL customer.email, "Order Confirmation", message, []
END IF
```

### Two-Factor Authentication (High Priority)

```basic
' Generate and send OTP with high priority for faster delivery
otp = RANDOM(100000, 999999)
REMEMBER "otp_" + user.id, otp, "5 minutes"

message = "Your verification code is: " + otp + ". Valid for 5 minutes."
result = SEND SMS user.phone, message, "high"

IF NOT result.success THEN
    TALK "Failed to send verification code. Please try again."
    RETURN
END IF

HEAR entered_code AS TEXT "Enter the code sent to your phone:"

stored_otp = RECALL "otp_" + user.id

IF entered_code = stored_otp THEN
    TALK "✅ Phone verified successfully!"
    SET USER MEMORY "phone_verified", true
ELSE
    TALK "❌ Invalid code. Please try again."
END IF
```

### Emergency Alert (Urgent Priority)

```basic
' Send emergency notification to multiple recipients
alert_message = "⚠️ ALERT: System maintenance in 30 minutes. Save your work."

contacts = FIND "emergency_contacts", "notify=true"

sent_count = 0
failed_count = 0

FOR EACH contact IN contacts
    result = SEND SMS contact.phone, alert_message, "urgent"
    
    IF result.success THEN
        sent_count = sent_count + 1
    ELSE
        failed_count = failed_count + 1
        PRINT "Failed to send to " + contact.phone + ": " + result.error
    END IF
    
    WAIT 100  ' Small delay between messages
NEXT

TALK "Emergency alert sent to " + sent_count + " contacts (" + failed_count + " failed)"
```

### Using Specific Provider with Priority

```basic
' Use AWS SNS for high-priority transactional messages
result = SEND SMS "+15551234567", "Your appointment is in 1 hour!", "aws", "high"

IF result.success THEN
    TALK "Reminder sent via " + result.provider + " with " + result.priority + " priority"
END IF
```

### Priority-Based Routing

```basic
' Route messages based on urgency
SUB send_notification(phone, message, urgency)
    SELECT CASE urgency
        CASE "critical"
            ' Use multiple channels for critical messages
            result = SEND SMS phone, message, "urgent"
            SEND MAIL user.email, "CRITICAL: " + message, message, []
            
        CASE "important"
            result = SEND SMS phone, message, "high"
            
        CASE "info"
            result = SEND SMS phone, message, "low"
            
        CASE ELSE
            result = SEND SMS phone, message, "normal"
    END SELECT
    
    RETURN result
END SUB

' Usage
send_notification(customer.phone, "Your package has been delivered!", "important")
```

### Appointment Reminder with Priority

```basic
' Send appointment reminder based on time until appointment
hours_until = DATEDIFF(appointment.datetime, NOW(), "hour")

IF hours_until <= 1 THEN
    ' Urgent - appointment is very soon
    priority = "urgent"
    message = "⏰ REMINDER: Your appointment is in " + hours_until + " hour(s)!"
ELSE IF hours_until <= 4 THEN
    ' High priority - same day
    priority = "high"
    message = "Reminder: Your appointment is today at " + FORMAT(appointment.datetime, "h:mm A")
ELSE
    ' Normal priority - advance reminder
    priority = "normal"
    message = "Reminder: You have an appointment on " + FORMAT(appointment.datetime, "MMMM D")
END IF

result = SEND SMS patient.phone, message, priority

IF result.success THEN
    UPDATE "appointments", appointment.id, "reminder_sent", true
END IF
```

### Bulk SMS with Priority Levels

```basic
' Send promotional messages with low priority (cost-effective)
customers = FIND "customers.csv", "marketing_opt_in = true"

FOR EACH customer IN customers
    message = "Hi " + customer.first_name + "! Check out our weekend sale - 20% off!"
    
    ' Use low priority for promotional bulk messages
    result = SEND SMS customer.phone, message, "low"
    
    IF result.success THEN
        INSERT "sms_log", customer.phone, message, result.message_id, NOW()
    END IF
    
    WAIT 500  ' Rate limiting for bulk sends
NEXT

TALK "Campaign completed!"
```

### Multi-Channel Fallback with Priority

```basic
' Try SMS first, fall back to other channels
SUB notify_user(user, message, priority)
    ' Try SMS first
    result = SEND SMS user.phone, message, priority
    
    IF result.success THEN
        RETURN "sms"
    END IF
    
    ' SMS failed, try WhatsApp
    wa_result = SEND WHATSAPP user.phone, message
    
    IF wa_result.success THEN
        RETURN "whatsapp"
    END IF
    
    ' Fall back to email
    SEND MAIL user.email, "Notification", message, []
    RETURN "email"
    
END SUB

' Usage
channel_used = notify_user(customer, "Your order has shipped!", "high")
TALK "Notification sent via " + channel_used
```

## Phone Number Formats

The keyword accepts various phone number formats and normalizes them:

| Format | Example | Result |
|--------|---------|--------|
| E.164 | `+14155551234` | `+14155551234` |
| National (US) | `(415) 555-1234` | `+14155551234` |
| Digits only (10) | `4155551234` | `+14155551234` |
| Digits only (11) | `14155551234` | `+14155551234` |

**Best Practice:** Always use E.164 format (`+` followed by country code and number).

## Message Length

| Type | Characters | Notes |
|------|------------|-------|
| Single SMS | 160 | Standard ASCII |
| Unicode SMS | 70 | Emojis, non-Latin scripts |
| Concatenated | 153 × segments | Long messages split |

> **Note:** High and urgent priority messages may have prefixes added (e.g., `[URGENT]`), which reduces available characters.

```basic
' Check message length before sending
IF LEN(message) > 140 AND priority = "urgent" THEN
    TALK "Warning: Urgent prefix may cause message to split"
END IF

SEND SMS phone, message, priority
```

## Priority Behavior by Provider

| Provider | Low | Normal | High | Urgent |
|----------|-----|--------|------|--------|
| **Twilio** | Standard | Standard | `[HIGH]` prefix | `[URGENT]` prefix |
| **AWS SNS** | Promotional | Promotional | Transactional | Transactional |
| **Vonage** | Standard | Standard | Standard | Flash message (class 0) |
| **MessageBird** | Standard | Standard | Class 1 | Flash message (class 0) |

## Error Handling

```basic
' Handle SMS errors gracefully
result = SEND SMS phone, message, "high"

IF NOT result.success THEN
    ' Log the failure
    INSERT "sms_failures", phone, message, result.error, NOW()
    
    ' Check error type and respond
    IF result.error LIKE "*INVALID_PHONE*" THEN
        TALK "The phone number appears to be invalid."
    ELSE IF result.error LIKE "*INSUFFICIENT_FUNDS*" THEN
        TALK "SMS service temporarily unavailable."
        ' Alert admin
        SEND MAIL admin.email, "SMS Balance Low", "Please top up SMS credits", []
    ELSE
        TALK "Could not send SMS: " + result.error
    END IF
    
    ' Fallback to email if available
    IF user.email <> "" THEN
        SEND MAIL user.email, "Notification", message, []
    END IF
END IF
```

## Cost Considerations

SMS messages incur costs per message sent. Consider:

- Use `low` priority for promotional/non-urgent messages (may use cheaper routes)
- Use `high`/`urgent` only when delivery speed is critical
- Use [SEND WHATSAPP](./universal-messaging.md) for free messaging when possible
- Batch non-urgent messages to optimize costs

## Compliance

When sending SMS messages, ensure compliance with:

- **TCPA** (US) - Require consent before sending
- **GDPR** (EU) - Document consent and provide opt-out
- **LGPD** (Brazil) - Similar consent requirements

```basic
' Check opt-in before sending
IF GET USER MEMORY "sms_opt_in" = true THEN
    SEND SMS phone, message, priority
ELSE
    TALK "User has not opted in to SMS notifications"
END IF
```

## See Also

- [SEND WHATSAPP](./universal-messaging.md) - WhatsApp messaging
- [SEND MAIL](./keyword-send-mail.md) - Email messaging
- [SEND TEMPLATE](./keyword-send-template.md) - Template messages
- [Universal Messaging](./universal-messaging.md) - Multi-channel messaging
- [SMS Provider Configuration](../10-configuration-deployment/sms-providers.md) - Provider setup guide

## Implementation

The SEND SMS keyword is implemented in `src/basic/keywords/sms.rs` with support for multiple providers through a unified interface. Priority levels are mapped to provider-specific features for optimal delivery.