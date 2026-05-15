PARAM message AS STRING DESCRIPTION "Mensagem com variáveis (ex: Olá {name}!)"
PARAM list_id AS STRING DESCRIPTION "ID da lista de contatos"
PARAM template_file AS STRING DESCRIPTION "URL da imagem de cabeçalho (opcional)" OPTIONAL
PARAM filter AS STRING DESCRIPTION "Filtro adicional (opcional)" OPTIONAL

DESCRIPTION "Envia broadcast WhatsApp em massa para uma lista de contatos."

IF NOT message THEN
    TALK "Qual é a mensagem para o broadcast?"
    HEAR message AS STRING
END IF

TALK "🤖 Verificando aprovação do template..."
approval_check = LLM "Esta mensagem será aprovada pelo WhatsApp META como Template? Responda OK se sim, ou explique o problema: " + message

IF approval_check <> "OK" THEN
    TALK "⚠️ **Atenção:** " + approval_check
    TALK "Deseja ajustar a mensagem ou continuar mesmo assim?"
    HEAR proceed AS BOOLEAN
    IF NOT proceed THEN
        RETURN
    END IF
ELSE
    TALK "✅ Mensagem aprovada para template WhatsApp!"
END IF

IF NOT list_id THEN
    TALK "Qual lista de contatos devo usar?"
    HEAR list_name AS STRING
    lists = GET "/api/marketing/lists?search=" + list_name
    IF UBOUND(lists) = 0 THEN
        TALK "Lista não encontrada."
        RETURN
    END IF
    list_id = FIRST(lists).id
END IF

list_info = GET "/api/marketing/lists/" + list_id
TALK "📤 **Broadcast Preview:**"
TALK "Mensagem: " + message
IF template_file THEN
    TALK "Imagem: " + template_file
END IF
TALK "Destinatários: " + list_info.contact_count + " contatos"
TALK "Lista: " + list_info.name

TALK "Confirmar envio?"
HEAR confirm AS BOOLEAN

IF NOT confirm THEN
    TALK "Broadcast cancelado."
    RETURN
END IF

TALK "🚀 Enviando broadcast..."

result = POST "/api/marketing/broadcast", #{
    message: message,
    list_id: list_id,
    template_file: template_file,
    filter: filter
}

TALK "✅ **Broadcast Enviado!**"
TALK "ID: " + result.broadcast_id
TALK "Status: " + result.status

TALK "Acompanhe as métricas em alguns minutos."
