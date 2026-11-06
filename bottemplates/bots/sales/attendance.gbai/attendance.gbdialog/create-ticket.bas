PARAM title AS STRING LIKE "Equipamento com defeito" DESCRIPTION "Título do problema"
PARAM category AS STRING LIKE "support" DESCRIPTION "Categoria: support, billing, sales, bug" OPTIONAL
PARAM priority AS STRING LIKE "high" DESCRIPTION "Nível: low, normal, high, urgent" OPTIONAL
PARAM contact_email AS EMAIL LIKE "usuario@dominio.com" DESCRIPTION "Email do reclamante" OPTIONAL
PARAM description AS STRING LIKE "O mouse parou" DESCRIPTION "Detalhes da solicitação"

DESCRIPTION "Cria um Ticket de suporte de longo prazo no sistema, fora da fila de atendimento tempo-real."

IF NOT category THEN
    category = "support"
END IF

IF NOT priority THEN
    priority = "normal"
END IF

contact_id = ""
IF contact_email THEN
    existing = GET "/api/crm/contacts?search=" + contact_email
    IF UBOUND(existing) > 0 THEN
        contact_id = FIRST(existing).id
    END IF
END IF

new_ticket = POST "/api/tickets", #{
    title: title,
    description: description,
    category: category,
    priority: priority,
    status: "open",
    contact_id: contact_id
}

TALK "🎫 **Ticket Criado com Sucesso!**"
TALK "ID: " + new_ticket.id
TALK "Título: " + title
TALK "Prioridade: " + priority
TALK "A equipe será notificada para escalonamento conforme SLA."

RETURN new_ticket.id
