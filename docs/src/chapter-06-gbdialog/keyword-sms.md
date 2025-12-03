# SEND SMS

Send SMS text messages to phone numbers using various providers.

## Syntax

```basic
' Basic SMS sending
SEND SMS phone, message

' With specific provider
SEND SMS phone, message, provider
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `phone` | String | Yes | Recipient phone number (E.164 format recommended) |
| `message` | String | Yes | The text message to send (max 160 chars for single SMS) |
| `provider` | String | No | SMS provider: `twilio`, `aws_sns`, `vonage`, `messagebird` |

## Return Value

Returns `true` if the SMS was sent successfully, `false` otherwise.

## Configuration

Configure SMS provider credentials in `config.csv`:

```csv
key,value
sms-provider,twilio
twilio-account-sid,YOUR_ACCOUNT_SID
twilio-auth-token,YOUR_AUTH_TOKEN
twilio-phone-number,+15551234567
```

### Provider-Specific Configuration

**Twilio:**
```csv
sms-provider,twilio
twilio-account-sid,ACxxxxx
twilio-auth-token,your_token
twilio-phone-number,+15551234567
```

**AWS SNS:**
```csv
sms-provider,aws_sns
aws-access-key-id,AKIAXXXXXXXX
aws-secret-access-key,your_secret
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

### Order Confirmation

```basic
' Send order confirmation via SMS
order_id = "ORD-2025-001"
phone = customer.phone

message = "Your order " + order_id + " has been confirmed. "
message = message + "Estimated delivery: 2-3 business days."

result = SEND SMS phone, message

IF result THEN
    TALK "Confirmation SMS sent to " + phone
ELSE
    TALK "Failed to send SMS. We'll email you instead."
    SEND MAIL customer.email, "Order Confirmation", message
END IF
```

### Two-Factor Authentication

```basic
' Generate and send OTP
otp = RANDOM(100000, 999999)
REMEMBER "otp_" + user.id, otp, "5 minutes"

message = "Your verification code is: " + otp + ". Valid for 5 minutes."
SEND SMS user.phone, message

HEAR entered_code AS TEXT "Enter the code sent to your phone:"

stored_otp = RECALL "otp_" + user.id

IF entered_code = stored_otp THEN
    TALK "✅ Phone verified successfully!"
    SET USER MEMORY "phone_verified", true
ELSE
    TALK "❌ Invalid code. Please try again."
END IF
```

### Appointment Reminder

```basic
' Send appointment reminder
appointment_date = FORMAT(appointment.datetime, "MMMM D, YYYY")
appointment_time = FORMAT(appointment.datetime, "h:mm A")

message = "Reminder: Your appointment is on " + appointment_date
message = message + " at " + appointment_time + ". Reply YES to confirm."

SEND SMS patient.phone, message

' Set up response handler
ON "sms:received" FROM patient.phone
    IF UPPER(params.message) = "YES" THEN
        UPDATE "appointments", appointment.id, "status", "confirmed"
        SEND SMS patient.phone, "Thank you! Your appointment is confirmed."
    END IF
END ON
```

### Multi-Language SMS

```basic
' Send SMS in user's preferred language
lang = GET USER MEMORY "language"

IF lang = "es" THEN
    message = "Gracias por tu compra. Tu pedido está en camino."
ELSE IF lang = "pt" THEN
    message = "Obrigado pela sua compra. Seu pedido está a caminho."
ELSE
    message = "Thank you for your purchase. Your order is on the way."
END IF

SEND SMS user.phone, message
```

### Using Different Providers

```basic
' Use specific provider for different regions
country_code = LEFT(phone, 3)

IF country_code = "+1 " THEN
    ' Use Twilio for US/Canada
    SEND SMS phone, message, "twilio"
ELSE IF country_code = "+55" THEN
    ' Use local provider for Brazil
    SEND SMS phone, message, "vonage"
ELSE
    ' Default provider
    SEND SMS phone, message
END IF
```

### Emergency Alert

```basic
' Send emergency notification to multiple recipients
alert_message = "⚠️ ALERT: System maintenance in 30 minutes. Save your work."

contacts = FIND "emergency_contacts", "notify=true"

FOR EACH contact IN contacts
    SEND SMS contact.phone, alert_message
    WAIT 100  ' Small delay between messages
NEXT

TALK "Emergency alert sent to " + COUNT(contacts) + " contacts"
```

### Delivery Tracking

```basic
' Send delivery status updates
ON "delivery:status_changed"
    order = FIND "orders", "id=" + params.order_id
    
    SWITCH params.status
        CASE "shipped"
            message = "📦 Your order has shipped! Tracking: " + params.tracking_number
        CASE "out_for_delivery"
            message = "🚚 Your package is out for delivery today!"
        CASE "delivered"
            message = "✅ Your package has been delivered. Enjoy!"
        DEFAULT
            message = "Order update: " + params.status
    END SWITCH
    
    SEND SMS order.phone, message
END ON
```

## Phone Number Formats

The keyword accepts various phone number formats:

| Format | Example | Recommended |
|--------|---------|-------------|
| E.164 | `+14155551234` | ✅ Yes |
| National | `(415) 555-1234` | ⚠️ Converted |
| Digits only | `4155551234` | ⚠️ Needs country |

**Best Practice:** Always use E.164 format (`+` followed by country code and number).

## Message Length

| Type | Characters | Notes |
|------|------------|-------|
| Single SMS | 160 | Standard ASCII |
| Unicode SMS | 70 | Emojis, non-Latin scripts |
| Concatenated | 153 × segments | Long messages split |

```basic
' Check message length before sending
IF LEN(message) > 160 THEN
    TALK "Warning: Message will be sent as multiple SMS"
END IF

SEND SMS phone, message
```

## Error Handling

```basic
' Handle SMS errors gracefully
TRY
    result = SEND SMS phone, message
    
    IF NOT result THEN
        ' Log the failure
        INSERT "sms_failures", phone, message, NOW()
        
        ' Fallback to email if available
        IF user.email <> "" THEN
            SEND MAIL user.email, "Notification", message
        END IF
    END IF
CATCH error
    TALK "SMS service unavailable: " + error.message
END TRY
```

## Cost Considerations

SMS messages incur costs per message sent. Consider:

- Using [SEND WHATSAPP](./universal-messaging.md) for free messaging when possible
- Batching non-urgent messages
- Using templates to keep messages under 160 characters

## Compliance

When sending SMS messages, ensure compliance with:

- **TCPA** (US) - Require consent before sending
- **GDPR** (EU) - Document consent and provide opt-out
- **LGPD** (Brazil) - Similar consent requirements

```basic
' Check opt-in before sending
IF GET USER MEMORY "sms_opt_in" = true THEN
    SEND SMS phone, message
ELSE
    TALK "User has not opted in to SMS notifications"
END IF
```

## See Also

- [SEND WHATSAPP](./universal-messaging.md) - WhatsApp messaging
- [SEND MAIL](./keyword-send-mail.md) - Email messaging
- [SEND TEMPLATE](./universal-messaging.md) - Template messages
- [Universal Messaging](./universal-messaging.md) - Multi-channel messaging

## Implementation

The SEND SMS keyword is implemented in `src/basic/keywords/sms.rs` with support for multiple providers through a unified interface.