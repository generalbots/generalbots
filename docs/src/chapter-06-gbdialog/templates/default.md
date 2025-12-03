# Default Template

The default template is the starter bot that comes with General Bots, providing essential utility tools like weather forecasts, email sending, SMS messaging, calculations, and translations.

## Topic: Starter Bot with Essential Tools

This template is perfect for:
- Quick start with General Bots
- Basic utility functions
- Learning BASIC syntax
- Foundation for custom bots

## Available Tools

The default template includes these ready-to-use tools:

| Tool | File | Description |
|------|------|-------------|
| Weather | `weather.bas` | Get weather forecasts for any city |
| Send Email | `send-email.bas` | Send emails to any address |
| Send SMS | `send-sms.bas` | Send text messages to mobile phones |
| Calculate | `calculate.bas` | Perform mathematical calculations |
| Translate | `translate.bas` | Translate text between languages |

## The Code: weather.bas

```basic
PARAM location AS STRING LIKE "New York" DESCRIPTION "City or location to get weather forecast"

DESCRIPTION "Get current weather forecast for any city or location"

lat = 40.7128
lon = -74.0060

location_lower = LCASE(location)

IF INSTR(location_lower, "new york") > 0 THEN
    lat = 40.7128
    lon = -74.0060
ELSE IF INSTR(location_lower, "london") > 0 THEN
    lat = 51.5074
    lon = -0.1278
ELSE IF INSTR(location_lower, "tokyo") > 0 THEN
    lat = 35.6762
    lon = 139.6503
ELSE IF INSTR(location_lower, "sao paulo") > 0 THEN
    lat = -23.5505
    lon = -46.6333
END IF

weather_url = "https://api.open-meteo.com/v1/forecast?latitude=" + lat + "&longitude=" + lon + "&current_weather=true"

weather_data = GET weather_url

IF weather_data.current_weather THEN
    current = weather_data.current_weather
    
    code = current.weathercode
    condition = "Clear"
    icon = "☀️"
    
    IF code = 0 THEN
        condition = "Clear sky"
        icon = "☀️"
    ELSE IF code >= 1 AND code <= 3 THEN
        condition = "Partly cloudy"
        icon = "⛅"
    ELSE IF code >= 51 AND code <= 67 THEN
        condition = "Rainy"
        icon = "🌧️"
    ELSE IF code >= 95 AND code <= 99 THEN
        condition = "Thunderstorm"
        icon = "⛈️"
    END IF
    
    TALK icon + " Weather for " + location + ":"
    TALK "Temperature: " + current.temperature + "°C"
    TALK "Condition: " + condition
    TALK "Wind: " + current.windspeed + " km/h"
ELSE
    TALK "Could not fetch weather for: " + location
END IF
```

## Sample Dialogs

These conversations show how the default template works in real-world scenarios.

### Dialog 1: Weather Forecast

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🤖</div>
    <div class="wa-header-info">
      <div class="wa-header-name">General Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-date"><span>Today</span></div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>What's the weather in Tokyo?</p>
      <div class="wa-time">10:00 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">General Bot</div>
      <p>☀️ Weather for Tokyo:</p>
      <p>Temperature: 18°C</p>
      <p>Condition: Clear sky</p>
      <p>Wind: 12 km/h</p>
      <div class="wa-time">10:00</div>
    </div>
  </div>
</div>

### Dialog 2: Send Email

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🤖</div>
    <div class="wa-header-info">
      <div class="wa-header-name">General Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Send an email to john@example.com</p>
      <div class="wa-time">11:30 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Sure! What should be the subject of the email?</p>
      <div class="wa-time">11:30</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Meeting Tomorrow</p>
      <div class="wa-time">11:31 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Got it. What would you like to say in the email?</p>
      <div class="wa-time">11:31</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Hi John, just a reminder about our meeting tomorrow at 2 PM. See you then!</p>
      <div class="wa-time">11:32 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Email sent successfully!</p>
      <p>📧 To: john@example.com</p>
      <p>📋 Subject: Meeting Tomorrow</p>
      <div class="wa-time">11:32</div>
    </div>
  </div>
</div>

### Dialog 3: Translation

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🤖</div>
    <div class="wa-header-info">
      <div class="wa-header-name">General Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Translate "Good morning, how are you?" to Spanish</p>
      <div class="wa-time">09:00 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🌐 Translation (English → Spanish):</p>
      <p><strong>"Buenos días, ¿cómo estás?"</strong></p>
      <div class="wa-time">09:00</div>
    </div>
  </div>
</div>

### Dialog 4: Calculation

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🤖</div>
    <div class="wa-header-info">
      <div class="wa-header-name">General Bot</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Calculate 15% of 850</p>
      <div class="wa-time">14:15 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🔢 15% of 850 = <strong>127.50</strong></p>
      <div class="wa-time">14:15</div>
    </div>
  </div>
</div>

## Template Structure

```
default.gbai/
├── default.gbdialog/
│   ├── calculate.bas      # Math calculations
│   ├── send-email.bas     # Email sending
│   ├── send-sms.bas       # SMS messaging
│   ├── translate.bas      # Text translation
│   └── weather.bas        # Weather forecasts
└── default.gbot/
    └── config.csv         # Bot configuration
```

## Keywords Used

| Keyword | Purpose |
|---------|---------|
| `PARAM` | Define tool parameters |
| `DESCRIPTION` | Tool description for AI |
| `GET` | HTTP GET request |
| `TALK` | Send message to user |
| `SEND MAIL` | Send email |
| `SEND SMS` | Send text message |
| `INSTR` | Find substring position |
| `LCASE` | Convert to lowercase |

## Supported Cities (Weather)

The weather tool includes coordinates for these cities:
- New York, Los Angeles, Chicago (USA)
- London, Paris, Berlin, Madrid (Europe)
- Tokyo, Beijing, Singapore, Mumbai, Dubai (Asia)
- Sydney (Australia)
- São Paulo, Rio de Janeiro (Brazil)
- Toronto (Canada)

## Customization Ideas

### Add More Cities

```basic
ELSE IF INSTR(location_lower, "amsterdam") > 0 THEN
    lat = 52.3676
    lon = 4.9041
ELSE IF INSTR(location_lower, "moscow") > 0 THEN
    lat = 55.7558
    lon = 37.6173
END IF
```

### Add Extended Forecast

```basic
' Get 7-day forecast
weather_url = weather_url + "&daily=temperature_2m_max,temperature_2m_min&forecast_days=7"

weather_data = GET weather_url

TALK "📅 7-Day Forecast for " + location + ":"
FOR i = 1 TO 7
    TALK "Day " + i + ": " + weather_data.daily.temperature_2m_max[i] + "°C / " + weather_data.daily.temperature_2m_min[i] + "°C"
NEXT
```

### Add Email Templates

```basic
PARAM template AS STRING LIKE "meeting-reminder" DESCRIPTION "Email template to use"

IF template = "meeting-reminder" THEN
    subject = "Meeting Reminder"
    body = "Hi {name},\n\nThis is a reminder about our upcoming meeting.\n\nBest regards"
    body = REPLACE(body, "{name}", recipient_name)
END IF

SEND MAIL recipient, subject, body
```

### Add SMS Confirmation

```basic
PARAM phone AS PHONE DESCRIPTION "Phone number with country code"
PARAM message AS STRING DESCRIPTION "Message to send"

DESCRIPTION "Send SMS with delivery confirmation"

SEND SMS phone, message

TALK "📱 SMS sent to " + phone
TALK "Message: " + LEFT(message, 50) + "..."

' Log the message
WITH smsLog
    timestamp = NOW()
    recipient = phone
    content = message
    status = "sent"
END WITH

SAVE "sms_log.csv", smsLog
```

## Using as a Base Template

The default template is designed to be extended. Here's how to build on it:

### 1. Copy the Template

```bash
cp -r templates/default.gbai packages/my-bot.gbai
```

### 2. Add Your Tools

Create new `.bas` files in the `.gbdialog` folder for your custom functionality.

### 3. Add a Start Script

Create `start.bas` to configure your bot:

```basic
ADD TOOL "weather"
ADD TOOL "send-email"
ADD TOOL "send-sms"
ADD TOOL "calculate"
ADD TOOL "translate"

' Add your custom tools
ADD TOOL "my-custom-tool"

CLEAR SUGGESTIONS

ADD SUGGESTION "weather" AS "Check weather"
ADD SUGGESTION "email" AS "Send email"
ADD SUGGESTION "translate" AS "Translate text"

BEGIN TALK
Welcome! I can help you with weather, emails, translations, and more.
END TALK
```

## Related Templates

- [start.bas](./start.md) - Basic greeting flow
- [broadcast.bas](./broadcast.md) - Mass messaging
- [store.bas](./store.md) - E-commerce features

---

<style>
.wa-chat{background-color:#e5ddd5;border-radius:8px;padding:20px 15px;margin:20px 0;max-width:600px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;font-size:14px}
.wa-chat::after{content:'';display:table;clear:both}
.wa-message{clear:both;margin-bottom:10px;max-width:85%;position:relative}
.wa-message.user{float:right}
.wa-message.user .wa-bubble{background-color:#dcf8c6;border-radius:8px 0 8px 8px;margin-left:40px}
.wa-message.bot{float:left}
.wa-message.bot .wa-bubble{background-color:#fff;border-radius:0 8px 8px 8px;margin-right:40px}
.wa-bubble{padding:8px 12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-bubble p{margin:0 0 4px 0;line-height:1.4;color:#303030}
.wa-bubble p:last-child{margin-bottom:0}
.wa-time{font-size:11px;color:#8696a0;text-align:right;margin-top:4px}
.wa-message.user .wa-time{color:#61a05e}
.wa-sender{font-size:12px;font-weight:600;color:#06cf9c;margin-bottom:2px}
.wa-status.read::after{content:'✓✓';color:#53bdeb;margin-left:4px}
.wa-date{text-align:center;margin:15px 0;clear:both}
.wa-date span{background-color:#fff;color:#54656f;padding:5px 12px;border-radius:8px;font-size:12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-header{background-color:#075e54;color:#fff;padding:10px 15px;margin:-20px -15px 15px -15px;border-radius:8px 8px 0 0;display:flex;align-items:center;gap:10px}
.wa-header-avatar{width:40px;height:40px;background-color:#25d366;border-radius:50%;display:flex;align-items:center;justify-content:center;font-size:18px}
.wa-header-info{flex:1}
.wa-header-name{font-weight:600;font-size:16px}
.wa-header-status{font-size:12px;opacity:.8}
</style>