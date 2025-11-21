REM General Bots: TRANSLATE Keyword - Universal Translation
REM Free translation using LibreTranslate API - No authentication required
REM Can be used by ANY template that needs translation

PARAM text AS string LIKE "Hello, how are you?"
PARAM from_lang AS string LIKE "en"
PARAM to_lang AS string LIKE "es"
DESCRIPTION "Translate text between languages using free API"

REM Validate input
IF NOT text OR text = "" THEN
    TALK "❌ Please provide text to translate"
    RETURN NULL
END IF

REM Set default languages if not provided
IF NOT from_lang OR from_lang = "" THEN
    from_lang = "en"
END IF

IF NOT to_lang OR to_lang = "" THEN
    to_lang = "es"
END IF

TALK "🌐 Translating from " + from_lang + " to " + to_lang + "..."

REM Try LibreTranslate API (free, open source)
REM Note: Public instance may have rate limits
translate_url = "https://libretranslate.com/translate"

REM Prepare POST data
post_data = NEW OBJECT
post_data.q = text
post_data.source = from_lang
post_data.target = to_lang
post_data.format = "text"

REM Set headers
SET HEADER "Content-Type" = "application/json"

REM Make translation request
translation_result = POST translate_url, post_data

IF translation_result.translatedText THEN
    translated = translation_result.translatedText

    result = NEW OBJECT
    result.original = text
    result.translated = translated
    result.from = from_lang
    result.to = to_lang

    TALK "✅ Translation complete!"
    TALK ""
    TALK "📝 Original (" + from_lang + "):"
    TALK text
    TALK ""
    TALK "✨ Translated (" + to_lang + "):"
    TALK translated

    RETURN result
ELSE
    REM Fallback: Try alternative API or show error
    TALK "❌ Translation failed. Trying alternative method..."

    REM Alternative: Use MyMemory Translation API (free, no key)
    mymemory_url = "https://api.mymemory.translated.net/get?q=" + text + "&langpair=" + from_lang + "|" + to_lang

    fallback_result = GET mymemory_url

    IF fallback_result.responseData.translatedText THEN
        translated = fallback_result.responseData.translatedText

        result = NEW OBJECT
        result.original = text
        result.translated = translated
        result.from = from_lang
        result.to = to_lang
        result.confidence = fallback_result.responseData.match

        TALK "✅ Translation complete (alternative API)!"
        TALK ""
        TALK "📝 Original (" + from_lang + "):"
        TALK text
        TALK ""
        TALK "✨ Translated (" + to_lang + "):"
        TALK translated

        IF result.confidence THEN
            TALK "🎯 Confidence: " + result.confidence
        END IF

        RETURN result
    ELSE
        TALK "❌ Could not translate text"
        TALK ""
        TALK "💡 Supported language codes:"
        TALK "en = English, es = Spanish, fr = French"
        TALK "de = German, it = Italian, pt = Portuguese"
        TALK "ru = Russian, ja = Japanese, zh = Chinese"
        TALK "ar = Arabic, hi = Hindi, ko = Korean"
        RETURN NULL
    END IF
END IF
