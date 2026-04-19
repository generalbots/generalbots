PARAM deal_id AS STRING LIKE "uuid" DESCRIPTION "ID do deal a atualizar" OPTIONAL
PARAM stage AS STRING LIKE "qualified" DESCRIPTION "Novo estágio: new, qualified, proposal, negotiation, won, lost" OPTIONAL
PARAM value AS MONEY LIKE 75000 DESCRIPTION "Novo valor do deal" OPTIONAL
PARAM lost_reason AS STRING LIKE "Preço" DESCRIPTION "Motivo da perda (obrigatório se stage=lost)" OPTIONAL
PARAM notes AS STRING LIKE "Reunião positiva" DESCRIPTION "Observações adicionais" OPTIONAL

DESCRIPTION "Atualizar um deal existente: mudar estágio, valor, adicionar notas. Se mudar para 'won' ou 'lost', o deal é fechado."

' Validate deal exists
IF NOT deal_id OR deal_id = "uuid" THEN
    TALK "Qual deal você deseja atualizar? (Pode dizer o nome da empresa ou o título)"
    HEAR query AS STRING
    ' Usar a ferramenta recém-criada (ou lógica similar) para buscar
    deals = GET "/api/crm/deals?search=" + query + "&limit=5"
    IF UBOUND(deals) = 0 THEN
        TALK "Não encontrei nenhum deal relacionado a '" + query + "'."
        RETURN
    END IF
    IF UBOUND(deals) = 1 THEN
        deal_id = FIRST(deals).id
    ELSE
        TALK "Encontrei várias opções. Seguem as 3 primeiras:"
        count = 1
        FOR EACH d IN deals
            IF count <= 3 THEN
                TALK count + ". **" + d.title + "** (R$ " + FORMAT(d.value, "#,##0") + ", " + d.stage + ")"
            END IF
            count = count + 1
        NEXT d
        TALK "Por favor, especifique melhor informando o título exato ou o e-mail do contato."
        RETURN
    END IF
END IF

deal = GET "/api/crm/deals/" + deal_id
IF NOT deal THEN
    TALK "Deal não encontrado com ID: " + deal_id
    RETURN
END IF

TALK "Deal atual: **" + deal.title + "** — " + deal.stage + " — R$ " + FORMAT(deal.value, "#,##0")

' Build update payload
update = #{}

IF stage THEN
    ' Validate stage
    valid_stages = "new,qualified,proposal,negotiation,won,lost"
    IF INSTR(valid_stages, stage) = 0 THEN
        TALK "Estágio inválido. Use: new, qualified, proposal, negotiation, won, lost"
        RETURN
    END IF

    update.stage = stage

    ' Set probability
    IF stage = "new" THEN
        update.probability = 10
    ELSE IF stage = "qualified" THEN
        update.probability = 30
    ELSE IF stage = "proposal" THEN
        update.probability = 50
    ELSE IF stage = "negotiation" THEN
        update.probability = 70
    ELSE IF stage = "won" THEN
        update.probability = 100
        update.won = TRUE
        update.actual_close_date = FORMAT(TODAY(), "YYYY-MM-DD")
    ELSE IF stage = "lost" THEN
        update.probability = 0
        update.won = FALSE
        update.actual_close_date = FORMAT(TODAY(), "YYYY-MM-DD")
        IF NOT lost_reason THEN
            TALK "Por que o deal foi perdido?"
            HEAR lost_reason AS STRING
        END IF
        update.lost_reason = lost_reason
    END IF
END IF

IF value THEN
    update.value = value
END IF

IF notes THEN
    update.notes = notes
END IF

' Execute update
PUT "/api/crm/deals/" + deal_id, update

' Log activity
POST "/api/crm/activities", #{
    activity_type: "stage_change",
    subject: "Deal atualizado para " + stage,
    description: notes,
    contact_id: deal.contact_id
}

' Response by stage
IF stage = "won" THEN
    TALK "🎉 **Parabéns! Deal ganho!**"
    TALK "**" + deal.title + "** fechado com sucesso!"
    TALK "💰 Valor: R$ " + FORMAT(COALESCE(value, deal.value), "#,##0")
ELSE IF stage = "lost" THEN
    TALK "📋 **Deal perdido**"
    TALK "**" + deal.title + "** marcado como perdido."
    TALK "📝 Motivo: " + lost_reason
    TALK "Analise o que aconteceu para melhorar nas próximas!"
ELSE IF stage THEN
    TALK "✅ **Deal atualizado!**"
    TALK "**" + deal.title + "** → **" + stage + "**"
    TALK "📊 Probabilidade: " + update.probability + "%"
ELSE
    TALK "✅ Deal atualizado com sucesso."
END IF

RETURN deal_id
