# Media & Messaging Keywords

Keywords for displaying media content and sending messages across various channels.

## Overview

These keywords handle media playback, QR code generation, and messaging operations that extend beyond the basic TALK/HEAR conversation flow.

## Keywords in This Section

| Keyword | Description |
|---------|-------------|
| [PLAY](./keyword-play.md) | Display videos, images, documents, and presentations |
| [QR CODE](./keyword-qrcode.md) | Generate QR code images from data |
| [SEND SMS](./keyword-sms.md) | Send SMS text messages |

## Quick Reference

### Media Display

```basic
' Play video with controls
PLAY "training.mp4" WITH OPTIONS "controls"

' Display image fullscreen
PLAY "banner.png" WITH OPTIONS "fullscreen"

' Show PDF document
PLAY "contract.pdf"

' Display PowerPoint presentation
PLAY "slides.pptx"
```

### QR Code Generation

```basic
' Generate basic QR code
qr_path = QR CODE "https://example.com"
SEND FILE qr_path

' Generate with custom size
qr_path = QR CODE "payment-data", 512

' WiFi QR code
wifi_data = "WIFI:T:WPA;S:MyNetwork;P:password123;;"
qr_path = QR CODE wifi_data
```

### SMS Messaging

```basic
' Send basic SMS
SEND SMS "+1234567890", "Hello from General Bots!"

' Send with specific provider
SEND SMS phone, message, "twilio"

' Two-factor authentication
otp = RANDOM(100000, 999999)
SEND SMS user.phone, "Your code: " + otp
```

## Channel Behavior

These keywords adapt their behavior based on the active channel:

| Keyword | Web | WhatsApp | Teams | SMS |
|---------|-----|----------|-------|-----|
| PLAY | Modal player | Send as media | Adaptive card | N/A |
| QR CODE | Display inline | Send as image | Embed in card | N/A |
| SEND SMS | N/A | N/A | N/A | Direct send |

## Configuration

### SMS Providers

Configure in `config.csv`:

```csv
sms-provider,twilio
twilio-account-sid,YOUR_SID
twilio-auth-token,YOUR_TOKEN
twilio-phone-number,+15551234567
```

### Supported Providers

- **Twilio** - Global coverage, reliable
- **AWS SNS** - AWS integration, cost-effective
- **Vonage** - Good international rates
- **MessageBird** - European coverage

## Common Patterns

### Interactive Media Training

```basic
TALK "Welcome to the training module!"
PLAY "intro-video.mp4" WITH OPTIONS "controls"

HEAR ready AS TEXT "Type 'next' when ready:"
PLAY "chapter-1.pptx"

HEAR quiz AS TEXT "What did you learn?"
' Process quiz response
```

### QR Code Payment Flow

```basic
HEAR amount AS NUMBER "Enter payment amount:"

payment_data = GENERATE_PAYMENT_CODE(amount)
qr_path = QR CODE payment_data, 400

TALK "Scan to pay $" + amount + ":"
SEND FILE qr_path
```

### SMS Verification

```basic
otp = RANDOM(100000, 999999)
REMEMBER "otp_" + user.id, otp, "5 minutes"

SEND SMS user.phone, "Your code: " + otp

HEAR code AS TEXT "Enter verification code:"

IF code = RECALL("otp_" + user.id) THEN
    TALK "✅ Verified!"
ELSE
    TALK "❌ Invalid code"
END IF
```

## See Also

- [Universal Messaging](./universal-messaging.md) - Multi-channel messaging
- [SEND MAIL](./keyword-send-mail.md) - Email messaging
- [TALK](./keyword-talk.md) - Basic text output
- [File Operations](./keywords-file.md) - File handling