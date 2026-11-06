PARAM title AS STRING LIKE "Contrato Empresa ABC" DESCRIPTION "Título do deal ou oportunidade"
PARAM contact_email AS EMAIL LIKE "joao@empresa.com" DESCRIPTION "Email do contato principal"
PARAM contact_name AS STRING LIKE "João Silva" DESCRIPTION "Nome do contato" OPTIONAL
PARAM company AS STRING LIKE "Empresa ABC Ltda" DESCRIPTION "Nome da empresa/conta" OPTIONAL
PARAM value AS MONEY LIKE 50000 DESCRIPTION "Valor estimado do deal"
PARAM stage AS STRING LIKE "new" DESCRIPTION "Estágio inicial: new, qualified, proposal, negotiation" OPTIONAL
PARAM source AS STRING LIKE "WHATSAPP" DESCRIPTION "Origem: WHATSAPP, EMAIL, CALL, WEBSITE, REFERAL, PARTNER, CHAT" OPTIONAL
PARAM close_date AS DATE LIKE "2026-06-30" DESCRIPTION "Data prevista de fechamento" OPTIONAL
PARAM notes AS STRING LIKE "Conheceu na feira" DESCRIPTION "Observações sobre o deal" OPTIONAL

DESCRIPTION "Criar um novo deal no pipeline de vendas. Registra o negócio com valor, estágio, contato e empresa."

IF NOT stage THEN
    stage = "new"
END IF

IF NOT source THEN
    source = "CHAT"
END IF

IF NOT close_date THEN
    close_date = DATEADD(TODAY(), 30, "day")
END IF

' Determine probability based on stage
probability = 10
IF stage = "qualified" THEN
    probability = 30
ELSE IF stage = "proposal" THEN
    probability = 50
ELSE IF stage = "negotiation" THEN
    probability = 70
END IF

' Check if contact exists, create if not
contact_id = ""
IF contact_email THEN
    existing = GET "/api/crm/contacts?search=" + contact_email
    IF UBOUND(existing) > 0 THEN
        contact_id = FIRST(existing).id
        TALK "Contato encontrado: " + FIRST(existing).first_name + " " + FIRST(existing).last_name
    ELSE
        ' Create contact
        IF NOT contact_name THEN
            contact_name = "Contato"
        END IF
        parts = SPLIT(contact_name, " ")
        first_name = FIRST(parts)
        last_name = LAST(parts)

        new_contact = POST "/api/crm/contacts", #{
            first_name: first_name,
            last_name: last_name,
            email: contact_email,
            company: company,
            source: source
        }
        contact_id = new_contact.id
        TALK "Novo contato criado: " + contact_name
    END IF
END IF

' Check if account exists, create if not
account_id = ""
IF company THEN
    existing_acc = GET "/api/crm/accounts?search=" + company
    IF UBOUND(existing_acc) > 0 THEN
        account_id = FIRST(existing_acc).id
    ELSE
        new_account = POST "/api/crm/accounts", #{
            name: company
        }
        account_id = new_account.id
        TALK "Nova conta criada: " + company
    END IF
END IF

' Create the deal
deal = POST "/api/crm/deals", #{
    title: title,
    value: value,
    stage: stage,
    probability: probability,
    source: source,
    expected_close_date: close_date,
    contact_id: contact_id,
    account_id: account_id,
    notes: notes
}

TALK "✅ **Deal criado com sucesso!**"
TALK ""
TALK "**" + title + "**"
TALK "💰 Valor: R$ " + FORMAT(value, "#,##0")
TALK "📊 Estágio: " + stage + " (" + probability + "% probabilidade)"
TALK "📅 Previsão de fechamento: " + close_date

IF company THEN
    TALK "🏢 Empresa: " + company
END IF

IF contact_email THEN
    TALK "👤 Contato: " + contact_email
END IF

RETURN deal.id
