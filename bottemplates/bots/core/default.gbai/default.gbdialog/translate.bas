PARAM text AS STRING LIKE "Hello, how are you?" DESCRIPTION "Text to translate"
PARAM from_lang AS STRING LIKE "en" DESCRIPTION "Source language code (en, es, pt, fr, de, etc)" OPTIONAL
PARAM to_lang AS STRING LIKE "es" DESCRIPTION "Target language code (en, es, pt, fr, de, etc)" OPTIONAL

DESCRIPTION "Translate text between languages (disabled by default; no external API calls)"

' The default experience must not send user text to third-party translation
' services. Configure a trusted, self-hosted translation endpoint before
' enabling external translation for this bot.
TALK "Translation is not configured for this bot."
TALK "To enable it, connect a trusted translation service in your bot configuration. Your text was not sent to any external service."
RETURN NULL
