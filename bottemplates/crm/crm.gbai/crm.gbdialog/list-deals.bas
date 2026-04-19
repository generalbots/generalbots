PARAM stage AS STRING LIKE "proposal" DESCRIPTION "Filtrar por estágio: new, qualified, proposal, negotiation, won, lost" OPTIONAL
PARAM owner AS STRING LIKE "joao@empresa.com" DESCRIPTION "Filtrar por dono do deal" OPTIONAL

DESCRIPTION "Listar deals do pipeline de vendas. Pode filtrar por estágio e por dono."

url = "/api/crm/deals?"

IF stage THEN
    url = url + "stage=" + stage + "&"
END IF

IF owner THEN
    url = url + "search=" + owner + "&"
END IF

url = url + "limit=20"

deals = GET url

deal_count = UBOUND(deals)

IF deal_count = 0 THEN
    TALK "Nenhum deal encontrado com os filtros informados."
    RETURN
END IF

TALK "📊 **Pipeline — " + deal_count + " deal(s) encontrado(s)**"
TALK ""

total_value = 0
total_weighted = 0

FOR EACH deal IN deals
    prob = deal.probability
    weighted = deal.value * prob / 100
    total_value = total_value + deal.value
    total_weighted = total_weighted + weighted

    TALK "---"
    TALK "**" + deal.title + "**"
    TALK "💰 R$ " + FORMAT(deal.value, "#,##0") + " | " + deal.stage + " (" + prob + "%)"

    IF deal.expected_close_date THEN
        TALK "📅 Previsão: " + deal.expected_close_date
    END IF

    IF deal.contact_id THEN
        contact = GET "/api/crm/contacts/" + deal.contact_id
        IF contact THEN
            TALK "👤 " + contact.first_name + " " + contact.last_name
        END IF
    END IF
NEXT deal

TALK ""
TALK "**Totais:**"
TALK "💰 Valor total: R$ " + FORMAT(total_value, "#,##0")
TALK "📊 Valor ponderado: R$ " + FORMAT(total_weighted, "#,##0")

RETURN deals
