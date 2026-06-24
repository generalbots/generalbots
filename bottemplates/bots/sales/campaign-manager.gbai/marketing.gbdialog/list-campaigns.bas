PARAM status AS STRING LIKE "draft" DESCRIPTION "Filtro de status (draft, scheduled, sending, sent, completed)" OPTIONAL
PARAM limit AS INTEGER LIKE 10 DESCRIPTION "Número máximo de campanhas a exibir" OPTIONAL

DESCRIPTION "Lista as campanhas de marketing."

IF NOT limit THEN
    limit = 10
END IF

url = "/api/marketing/campaigns?limit=" + limit

IF status THEN
    url = url + "&status=" + status
END IF

campaigns = GET url
count = UBOUND(campaigns)

IF count = 0 THEN
    TALK "Nenhuma campanha encontrada."
    RETURN
END IF

TALK "📊 **Campanhas — " + count + " resultado(s)**"

FOR EACH c IN campaigns
    TALK "---"
    TALK "**" + c.name + "** (" + c.status + ")"
    TALK "📣 Canal: " + UCASE(c.channel)
    TALK "🔑 ID: " + c.id
    
    IF c.scheduled_at THEN
        TALK "🕒 Agendada: " + c.scheduled_at
    END IF

    IF c.metrics THEN
        TALK "📈 Enviados: " + c.metrics.sent + " | Erros: " + c.metrics.failed
        IF c.channel = "email" THEN
            TALK "   Aberturas: " + c.metrics.opened + " | Clicks: " + c.metrics.clicked
        END IF
    END IF
NEXT c

RETURN campaigns
