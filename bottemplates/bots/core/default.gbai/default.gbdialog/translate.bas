PARAM text AS STRING LIKE "Hello, how are you?" DESCRIPTION "Text to translate"
PARAM from_lang AS STRING LIKE "en" DESCRIPTION "Source language code (en, es, pt, fr, de, etc)" OPTIONAL
PARAM to_lang AS STRING LIKE "es" DESCRIPTION "Target language code (en, es, pt, fr, de, etc)" OPTIONAL

DESCRIPTION "Translate text between languages using free translation API"

IF NOT from_lang THEN
    from_lang = "en"
END IF

IF NOT to_lang THEN
    to_lang = "es"
END IF

TALK "Translating from " + from_lang + " to " + to_lang + "..."

WITH post_data
    q = text
    source = from_lang
    target = to_lang
    format = "text"
END WITH

SET HEADER "Content-Type" = "application/json"

translation_result = POST "https://libretranslate.com/translate", post_data

IF translation_result.translatedText THEN
    WITH result
        original = text
        translated = translation_result.translatedText
        from = from_lang
        to = to_lang
    END WITH

    TALK "Original (" + from_lang + "): " + text
    TALK "Translated (" + to_lang + "): " + result.translated

    RETURN result
ELSE
    mymemory_url = "https://api.mymemory.translated.net/get?q=" + text + "&langpair=" + from_lang + "|" + to_lang
    fallback_result = GET mymemory_url

    IF fallback_result.responseData.translatedText THEN
        WITH result
            original = text
            translated = fallback_result.responseData.translatedText
            from = from_lang
            to = to_lang
        END WITH

        TALK "Original (" + from_lang + "): " + text
        TALK "Translated (" + to_lang + "): " + result.translated

        RETURN result
    ELSE
        TALK "Could not translate text"
        RETURN NULL
    END IF
END IF
