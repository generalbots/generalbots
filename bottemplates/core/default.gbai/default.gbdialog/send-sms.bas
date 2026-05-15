PARAM phone_number AS PHONE LIKE "+1234567890" DESCRIPTION "Phone number with country code"
PARAM message AS STRING LIKE "Hello, this is your message" DESCRIPTION "SMS message content"
PARAM from_number AS PHONE LIKE "+1987654321" DESCRIPTION "Sender phone number" OPTIONAL

DESCRIPTION "Send an SMS message to any phone number"

message_length = LEN(message)
segments = INT((message_length - 1) / 160) + 1

IF message_length > 160 THEN
    TALK "Message will be split into " + segments + " segments"
END IF

WITH sms
    to = phone_number
    from = from_number
    body = message
    timestamp = NOW()
    segmentCount = segments
END WITH

SEND SMS phone_number, message

SAVE "sms_log.csv", sms

TALK "SMS sent to " + phone_number

RETURN sms
