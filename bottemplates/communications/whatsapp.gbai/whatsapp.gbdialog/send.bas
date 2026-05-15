PARAM phone AS PHONE LIKE "122233333333" DESCRIPTION "WhatsApp phone number with country code"
PARAM template AS STRING LIKE "newsletter-zap.txt" DESCRIPTION "Template file name to send"
PARAM variables AS OBJECT LIKE "{name: 'John'}" DESCRIPTION "Template variables for personalization" OPTIONAL

DESCRIPTION "Send a WhatsApp template message to a phone number"

SEND TEMPLATE TO phone, template, variables

WITH log
    timestamp = NOW()
    phoneNumber = phone
    templateFile = template
    status = "sent"
END WITH

SAVE "whatsapp_log.csv", log

TALK "WhatsApp message sent to " + phone

RETURN phone
