PARAM name AS STRING LIKE "Promoção Dia das Mães" DESCRIPTION "Nome descritivo da campanha"
PARAM channel AS STRING LIKE "email" DESCRIPTION "Canal de envio: whatsapp, email, sms, telegram"
PARAM scheduled_date AS DATE LIKE "2026-05-01 09:00:00" DESCRIPTION "Data/Hora de agendamento do envio (opcional)" OPTIONAL
PARAM list_id AS STRING LIKE "uuid_da_lista" DESCRIPTION "ID da Lista de Contatos alvo" OPTIONAL
PARAM template_id AS STRING LIKE "uuid_do_template" DESCRIPTION "ID do Template de Conteúdo" OPTIONAL
PARAM ai_generate_template AS STRING LIKE "Crie um email rápido de 3 parágrafos sobre sapatos com desconto de 15%" DESCRIPTION "Prompt para IA caso queira gerar um template agora" OPTIONAL

DESCRIPTION "Cria uma Campanha de Marketing e agenda seu envio. Pode integrar/vincular a listas e templates existentes."

IF NOT list_id THEN
    TALK "Qual lista de contatos você deseja usar para esta campanha?"
    ' Aqui seria uma busca na db ou listagem interativa
    HEAR list_id AS STRING
END IF

// Create template if an AI prompt was given but no template_id
IF ai_generate_template AND NOT template_id THEN
    TALK "🤖 Gerando e salvando um template de " + channel + " baseado no seu pedido..."
    new_template = POST "/api/marketing/templates", #{
        name: name + " Template",
        channel: channel,
        ai_prompt: ai_generate_template
    }
    template_id = new_template.id
    TALK "Template ID retornado: " + template_id
ELSE IF NOT template_id THEN
    TALK "Tem o ID do template guardado? Diga-me por favor."
    HEAR template_id AS STRING
END IF

new_campaign = POST "/api/marketing/campaigns", #{
    name: name,
    channel: channel,
    status: "draft",
    scheduled_at: scheduled_date,
    template_id: template_id,
    list_id: list_id
}

TALK "📣 **Campanha Criada!**"
TALK "Nome: " + name
TALK "Canal: " + UCASE(channel)
TALK "Status: Draft"
TALK "ID: " + new_campaign.id

IF scheduled_date THEN
    TALK "Agendada para: " + scheduled_date
ELSE
    TALK "Pendente de envio. Use 'send-campaign' para dispará-la agora."
END IF

RETURN new_campaign.id
