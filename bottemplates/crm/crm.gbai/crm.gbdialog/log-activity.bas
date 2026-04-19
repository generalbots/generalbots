PARAM activity_type AS STRING LIKE "call" DESCRIPTION "Tipo: call, email, meeting, note, follow_up"
PARAM subject AS STRING LIKE "Ligação de follow-up" DESCRIPTION "Assunto da atividade"
PARAM contact_email AS EMAIL LIKE "joao@empresa.com" DESCRIPTION "Email do contato relacionado" OPTIONAL
PARAM deal_id AS STRING LIKE "uuid" DESCRIPTION "ID do deal relacionado" OPTIONAL
PARAM description AS STRING LIKE "Discutimos termos do contrato" DESCRIPTION "Detalhes da atividade" OPTIONAL
PARAM due_date AS DATE LIKE "2026-04-01" DESCRIPTION "Data de follow-up" OPTIONAL

DESCRIPTION "Registrar uma atividade de CRM: ligação, email, reunião, nota ou follow-up. Pode vincular a contato e/ou deal."

' Resolve contact_id from email
contact_id = ""
IF contact_email THEN
    contacts = GET "/api/crm/contacts?search=" + contact_email
    IF UBOUND(contacts) > 0 THEN
        contact_id = FIRST(contacts).id
    END IF
END IF

' Resolve deal_id if empty or uuid but we need it
IF NOT deal_id OR deal_id = "uuid" THEN
    TALK "A qual deal esta atividade se refere? (Opcional: Diga 'nenhum' ou o nome da empresa)"
    HEAR query AS STRING
    IF LCASE(query) != "nenhum" AND LCASE(query) != "none" THEN
        deals = GET "/api/crm/deals?search=" + query + "&limit=5"
        IF UBOUND(deals) > 0 THEN
            deal_id = FIRST(deals).id
            TALK "Atividade será vinculada ao deal: " + FIRST(deals).title
        END IF
    ELSE
        deal_id = ""
    END IF
END IF

' Create activity
activity = POST "/api/crm/activities", #{
    activity_type: activity_type,
    subject: subject,
    description: description,
    contact_id: contact_id,
    due_date: due_date
}

TALK "✅ **Atividade registrada!**"
TALK "📋 " + activity_type + ": " + subject

IF contact_email THEN
    TALK "👤 Contato: " + contact_email
END IF

IF due_date THEN
    TALK "📅 Follow-up: " + due_date
END IF

RETURN activity.id
