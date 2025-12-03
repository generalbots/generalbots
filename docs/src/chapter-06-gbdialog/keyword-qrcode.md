# QR CODE

Generate QR code images from text or data.

## Syntax

```basic
' Basic QR code generation
path = QR CODE data

' With custom size (pixels)
path = QR CODE data, size

' With size and output path
path = QR CODE data, size, output_path
```

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `data` | String | Yes | The data to encode in the QR code (URL, text, etc.) |
| `size` | Integer | No | Image size in pixels (default: 256) |
| `output_path` | String | No | Custom output file path |

## Return Value

Returns the file path to the generated QR code image (PNG format).

## Examples

### Basic QR Code

```basic
' Generate a QR code for a URL
qr_path = QR CODE "https://example.com"
TALK "Scan this QR code:"
SEND FILE qr_path
```

### QR Code with Custom Size

```basic
' Generate a larger QR code (512x512 pixels)
qr_path = QR CODE "https://mywebsite.com/signup", 512
SEND FILE qr_path
```

### Dynamic Content

```basic
HEAR user_id AS TEXT "Enter your user ID:"

' Generate QR code with dynamic data
profile_url = "https://app.example.com/profile/" + user_id
qr_path = QR CODE profile_url, 300

TALK "Here's your profile QR code:"
SEND FILE qr_path
```

### Event Check-in

```basic
' Generate unique check-in codes for events
event_id = "EVT-2025-001"
attendee_email = user.email

checkin_data = "CHECKIN:" + event_id + ":" + attendee_email
qr_path = QR CODE checkin_data, 400

TALK "Show this QR code at the event entrance:"
SEND FILE qr_path
```

### Payment QR Code

```basic
' Generate PIX payment QR code (Brazil)
HEAR amount AS NUMBER "Enter payment amount:"

pix_payload = "00020126580014br.gov.bcb.pix0136" + merchant_key
pix_payload = pix_payload + "5204000053039865802BR"
pix_payload = pix_payload + "5913MerchantName6008CityName62070503***"

qr_path = QR CODE pix_payload, 400
TALK "Scan to pay R$ " + amount + ":"
SEND FILE qr_path
```

### WiFi QR Code

```basic
' Generate WiFi connection QR code
wifi_ssid = "MyNetwork"
wifi_password = "SecurePass123"
wifi_type = "WPA"

wifi_data = "WIFI:T:" + wifi_type + ";S:" + wifi_ssid + ";P:" + wifi_password + ";;"
qr_path = QR CODE wifi_data, 300

TALK "Scan to connect to WiFi:"
SEND FILE qr_path
```

### Contact Card (vCard)

```basic
' Generate QR code with contact information
vcard = "BEGIN:VCARD\n"
vcard = vcard + "VERSION:3.0\n"
vcard = vcard + "N:Doe;John\n"
vcard = vcard + "TEL:+1234567890\n"
vcard = vcard + "EMAIL:john@example.com\n"
vcard = vcard + "END:VCARD"

qr_path = QR CODE vcard, 350
TALK "Scan to add contact:"
SEND FILE qr_path
```

### Custom Output Location

```basic
' Save QR code to specific path
output_file = "work/qrcodes/user_" + user.id + ".png"
qr_path = QR CODE "https://example.com", 256, output_file

TALK "QR code saved to: " + qr_path
```

## Supported Data Types

The QR CODE keyword can encode various types of data:

| Type | Format | Example |
|------|--------|---------|
| URL | `https://...` | `https://example.com` |
| Plain Text | Any text | `Hello World` |
| WiFi | `WIFI:T:WPA;S:ssid;P:pass;;` | Network credentials |
| vCard | `BEGIN:VCARD...END:VCARD` | Contact information |
| Email | `mailto:email@example.com` | Email link |
| Phone | `tel:+1234567890` | Phone number |
| SMS | `sms:+1234567890?body=Hello` | SMS with message |
| Geo | `geo:lat,lon` | Geographic coordinates |

## Size Guidelines

| Use Case | Recommended Size |
|----------|------------------|
| Mobile scanning | 256-300px |
| Print (business card) | 300-400px |
| Print (poster) | 512-1024px |
| Digital display | 256-512px |

## Error Handling

```basic
' Check if QR code was generated
qr_path = QR CODE data

IF qr_path = "" THEN
    TALK "Failed to generate QR code"
ELSE
    SEND FILE qr_path
END IF
```

## File Storage

Generated QR codes are stored in the bot's `.gbdrive` storage:
- Default location: `work/qrcodes/`
- Format: PNG
- Naming: UUID-based unique filenames

## Limitations

- Maximum data length depends on QR code version (up to ~4,296 alphanumeric characters)
- Larger data requires larger image sizes for reliable scanning
- Binary data should be Base64 encoded

## See Also

- [SEND FILE](./keyword-send-mail.md) - Send generated QR codes
- [TALK](./keyword-talk.md) - Display messages with QR codes
- [FORMAT](./keyword-format.md) - Format data before encoding

## Implementation

The QR CODE keyword is implemented in `src/basic/keywords/qrcode.rs` using the `qrcode` and `image` crates for generation.