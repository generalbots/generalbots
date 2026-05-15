PARAM session_id AS STRING LIKE "uuid" DESCRIPTION "ID da sessão (opcional). Se vazio, pega o próximo da fila." OPTIONAL

DESCRIPTION "Atender o próximo cliente da fila ou um cliente específico."

IF NOT session_id THEN
    ' Tentar pegar o mais antigo da fila ("waiting", sem assignee)
    queue = GET "/api/attendance/queue?status=waiting"
    IF UBOUND(queue) = 0 THEN
        TALK "A fila está vazia no momento. Bom trabalho!"
        RETURN
    END IF
    
    ' Pega o primeiro (mais antigo, se a API ordenar por tempo de espera)
    first_item = FIRST(queue)
    session_id = first_item.session_id
END IF

' Endpoint hipotético para "tomar posse" da sessão
assign_result = POST "/api/attendance/sessions/" + session_id + "/assign", #{
    attendant_id: GET "session.user_id"
}

TALK "✅ **Sessão atribuída a você!**"
TALK "Você agora está atendendo: " + assign_result.user_name + " via " + assign_result.channel
TALK "Use as ferramentas de chat para conversar com o cliente."

RETURN session_id
