PARAM deal_id AS STRING LIKE "uuid" DESCRIPTION "ID do deal a fechar" OPTIONAL
PARAM won AS BOOLEAN LIKE TRUE DESCRIPTION "TRUE se ganhou, FALSE se perdeu"
PARAM lost_reason AS STRING LIKE "Preço acima do mercado" DESCRIPTION "Motivo da perda (obrigatório se won=FALSE)" OPTIONAL

DESCRIPTION "Fechar um deal como ganho (won) ou perdido (lost). Se perdido, informe o motivo."

IF NOT deal_id OR deal_id = "uuid" THEN
    TALK "Qual deal você deseja fechar? (Diga o nome da empresa ou título)"
    HEAR query AS STRING
    deals = GET "/api/crm/deals?search=" + query + "&limit=5"
    IF UBOUND(deals) = 0 THEN
        TALK "Não encontrei nenhum deal relacionado a '" + query + "'."
        RETURN
    END IF
    IF UBOUND(deals) = 1 THEN
        deal_id = FIRST(deals).id
    ELSE
        TALK "Encontrei várias opções. Qual delas quer fechar? Especifique melhor, por favor."
        FOR EACH d IN deals
            TALK "- **" + d.title + "** (" + d.stage + ")"
        NEXT d
        RETURN
    END IF
END IF

deal = GET "/api/crm/deals/" + deal_id
IF NOT deal THEN
    TALK "Deal não encontrado: " + deal_id
    RETURN
END IF

IF won THEN
    PUT "/api/crm/deals/" + deal_id, #{
        stage: "won",
        won: TRUE,
        probability: 100,
        actual_close_date: FORMAT(TODAY(), "YYYY-MM-DD")
    }

    POST "/api/crm/activities", #{
        activity_type: "deal_won",
        subject: "Deal ganho: " + deal.title,
        contact_id: deal.contact_id,
        account_id: deal.account_id
    }

    TALK "🎉 **Deal ganho!**"
    TALK "**" + deal.title + "** — R$ " + FORMAT(deal.value, "#,##0")
    TALK "Parabéns pela conquista!"
ELSE
    IF NOT lost_reason THEN
        TALK "Qual foi o motivo da perda?"
        HEAR lost_reason AS STRING
    END IF

    PUT "/api/crm/deals/" + deal_id, #{
        stage: "lost",
        won: FALSE,
        probability: 0,
        lost_reason: lost_reason,
        actual_close_date: FORMAT(TODAY(), "YYYY-MM-DD")
    }

    POST "/api/crm/activities", #{
        activity_type: "deal_lost",
        subject: "Deal perdido: " + deal.title,
        description: "Motivo: " + lost_reason,
        contact_id: deal.contact_id,
        account_id: deal.account_id
    }

    TALK "📋 **Deal perdido**"
    TALK "**" + deal.title + "** — Motivo: " + lost_reason
    TALK "Use essa experiência para os próximos negócios."
END IF

RETURN deal_id
