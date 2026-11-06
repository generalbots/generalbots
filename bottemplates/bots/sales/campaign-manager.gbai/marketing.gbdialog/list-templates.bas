PARAM search AS STRING DESCRIPTION "Termo de busca (opcional)" OPTIONAL
PARAM channel AS STRING LIKE "whatsapp" DESCRIPTION "Filtrar por canal: whatsapp, email, sms, telegram" OPTIONAL
PARAM status_filter AS STRING DESCRIPTION "Filtrar por status: draft, approved, active" OPTIONAL

DESCRIPTION "Lista todos os Templates de Marketing disponíveis, com opção de busca e filtro."

query = "/api/marketing/templates?"
has_param = FALSE

IF search THEN
    query = query + "search=" + search
    has_param = TRUE
END IF

IF channel THEN
    IF has_param THEN
        query = query + "&"
    END IF
    query = query + "channel=" + channel
    has_param = TRUE
END IF

IF status_filter THEN
    IF has_param THEN
        query = query + "&"
    END IF
    query = query + "status=" + status_filter
END IF

templates = GET query

IF UBOUND(templates) = 0 THEN
    TALK "Nenhum template encontrado."
    IF search OR channel OR status_filter THEN
        TALK "Tente remover os filtros ou usar outros termos."
    END IF
    RETURN
END IF

TALK "📋 **Templates Encontrados (" + UBOUND(templates) + "):**"

FOR EACH t IN templates
    status_icon = IIF(t.status = "approved", "✅", IIF(t.status = "active", "🟢", "📝"))
    TALK status_icon + " **" + t.name + "**"
    TALK "   Canal: " + UCASE(t.channel)
    TALK "   Status: " + UCASE(t.status)
    IF t.content THEN
        preview = LEFT(t.content, 80)
        IF LEN(t.content) > 80 THEN
            preview = preview + "..."
        END IF
        TALK "   Preview: " + preview
    END IF
    TALK "   ID: " + t.id
    TALK ""
NEXT t

TALK "Para usar um template em uma campanha, anote o ID."
