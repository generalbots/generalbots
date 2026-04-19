PARAM campaign_id AS STRING LIKE "uuid" DESCRIPTION "ID da Campanha" OPTIONAL
PARAM allow_over_budget AS BOOLEAN LIKE FALSE DESCRIPTION "Permitir envio mesmo se ultrapassar o orçamento do canal?" OPTIONAL

DESCRIPTION "Confirma e inicia o disparo (envio) imediato de uma campanha."

IF NOT campaign_id OR campaign_id = "uuid" THEN
    TALK "Qual campanha você deseja enviar? (Diga o nome)"
    HEAR query AS STRING
    camps = GET "/api/marketing/campaigns?search=" + query + "&limit=5"
    IF UBOUND(camps) = 0 THEN
        TALK "Não encontrei nenhuma campanha com o termo '" + query + "'."
        RETURN
    END IF
    IF UBOUND(camps) = 1 THEN
        campaign_id = FIRST(camps).id
    ELSE
        TALK "Encontrei várias campanhas. Qual delas?"
        FOR EACH c IN camps
            TALK "- **" + c.name + "** (" + c.status + ")"
        NEXT c
        RETURN
    END IF
END IF

camp = GET "/api/marketing/campaigns/" + campaign_id
IF NOT camp THEN
    TALK "Campanha não encontrada: " + campaign_id
    RETURN
END IF

IF camp.status = "sent" OR camp.status = "completed" THEN
    TALK "A campanha '" + camp.name + "' já foi enviada."
    RETURN
END IF

IF NOT camp.list_id THEN
    TALK "Esta campanha não tem uma lista de contatos vinculada. Não é possível enviar."
    RETURN
END IF

list_info = GET "/api/marketing/lists/" + camp.list_id
TALK "🔍 **Análise Pré-Envio:**"
TALK "• Campanha: " + camp.name
TALK "• Canal: " + UCASE(camp.channel)
TALK "• Segmentação/Lista: " + list_info.name + " (" + list_info.contact_count + " contatos)"

TALK "Deseja realmente iniciar o envio AGORA para " + list_info.contact_count + " pessoas?"
HEAR confirm AS BOOLEAN

IF confirm THEN
    POST "/api/marketing/campaigns/" + campaign_id + "/send"
    TALK "🚀 **Envio Iniciado!**"
    TALK "As automações do " + UCASE(camp.channel) + " foram engatilhadas."
    TALK "Você pode acompanhar as métricas em 'list-campaigns' depois de alguns minutos."
ELSE
    TALK "Envio cancelado. A campanha continua no status '" + camp.status + "'."
END IF

RETURN camp_id
