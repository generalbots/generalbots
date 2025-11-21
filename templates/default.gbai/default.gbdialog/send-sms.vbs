REM General Bots: SEND SMS Keyword - Universal SMS Sending
REM Free SMS sending using Twilio, Nexmo, or other SMS APIs
REM Can be used by ANY template that needs to send SMS messages

PARAM phone_number AS string LIKE "+1234567890"
PARAM message AS string LIKE "Hello, this is your SMS message"
PARAM from_number AS string LIKE "+1987654321"

DESCRIPTION "Send an SMS message to any phone number"

REM Validate inputs
IF NOT phone_number OR phone_number = "" THEN
    TALK "❌ Phone number is required"
    TALK "💡 Format: +[country code][number] (e.g., +1234567890)"
    RETURN NULL
END IF

IF NOT message OR message = "" THEN
    TALK "❌ Message content is required"
    RETURN NULL
END IF

REM Validate phone number format (basic check)
IF LEFT(phone_number, 1) <> "+" THEN
    TALK "⚠️ Phone number should start with + and country code"
    TALK "Example: +1234567890"
END IF

REM Check message length (SMS limit is typically 160 characters)
message_length = LEN(message)
IF message_length > 160 THEN
    TALK "⚠️ Warning: Message is " + message_length + " characters"
    TALK "SMS messages over 160 characters may be split into multiple messages"
END IF

TALK "📱 Preparing to send SMS..."
TALK "To: " + phone_number
TALK "Message length: " + message_length + " characters"

REM Create SMS object
sms_data = NEW OBJECT
sms_data.to = phone_number
sms_data.from = from_number
sms_data.message = message
sms_data.timestamp = NOW()
sms_data.status = "pending"
sms_data.length = message_length

REM Calculate estimated cost (example: $0.01 per message)
segments = INT((message_length - 1) / 160) + 1
sms_data.segments = segments
sms_data.estimated_cost = segments * 0.01

REM In production, this would integrate with:
REM 1. Twilio SMS API (https://www.twilio.com/docs/sms)
REM 2. Nexmo/Vonage API (https://developer.vonage.com/messaging/sms/overview)
REM 3. AWS SNS (https://aws.amazon.com/sns/)
REM 4. Azure Communication Services
REM 5. MessageBird API

REM Example Twilio integration (requires API key):
REM SET HEADER "Authorization" = "Basic " + BASE64(account_sid + ":" + auth_token)
REM SET HEADER "Content-Type" = "application/x-www-form-urlencoded"
REM twilio_url = "https://api.twilio.com/2010-04-01/Accounts/" + account_sid + "/Messages.json"
REM post_data = "To=" + phone_number + "&From=" + from_number + "&Body=" + message
REM result = POST twilio_url, post_data

REM For now, save to queue for processing
SAVE "sms_queue.csv", sms_data.timestamp, sms_data.from, sms_data.to, sms_data.message, sms_data.segments, sms_data.estimated_cost, sms_data.status

TALK "✅ SMS queued successfully!"
TALK ""
TALK "📊 SMS Details:"
IF from_number AND from_number <> "" THEN
    TALK "From: " + from_number
END IF
TALK "To: " + phone_number
TALK "Message: " + LEFT(message, 50) + "..."
TALK "Segments: " + segments
TALK "Estimated Cost: $" + sms_data.estimated_cost
TALK "Time: " + sms_data.timestamp
TALK ""
TALK "⚙️ Status: Queued for delivery"
TALK ""
TALK "💡 Setup Guide to enable SMS:"
TALK "1. Sign up for Twilio (free trial available)"
TALK "2. Get your Account SID and Auth Token"
TALK "3. Get a Twilio phone number"
TALK "4. Configure in .gbot settings:"
TALK "   - SMS_PROVIDER (twilio/nexmo/aws)"
TALK "   - SMS_ACCOUNT_SID"
TALK "   - SMS_AUTH_TOKEN"
TALK "   - SMS_FROM_NUMBER"
TALK ""
TALK "📚 Twilio Quick Start: https://www.twilio.com/docs/sms/quickstart"

REM Return SMS data
RETURN sms_data
