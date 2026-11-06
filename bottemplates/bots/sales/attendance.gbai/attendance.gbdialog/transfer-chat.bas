PARAM session_id AS STRING LIKE "uuid" DESCRIPTION "ID da sessão" OPTIONAL
PARAM skill AS STRING LIKE "suporte" DESCRIPTION "Habilidade ou fila destino (ex: financeiro, vendas)" OPTIONAL
PARAM priority AS STRING LIKE "high" DESCRIPTION "Nova prioridade (urgent, high, normal, low)" OPTIONAL
PARAM reason AS STRING LIKE "Dúvida sobre boleto" DESCRIPTION "Motivo da transferência (obrigatório)"

DESCRIPTION "Transfere o chat para outro nível de suporte ou fila usando Habilidades, ajustando prioridade."

IF NOT reason THEN
    TALK "Qual é o motivo da transferência para o próximo atendente?"
    HEAR reason AS STRING
END IF

IF NOT priority THEN
    priority = "normal"
END IF

' Tratar ID da sessão conversacionalmente
IF NOT session_id THEN
    ' Busca sessões ativas do atendente
    my_sessions = GET "/api/attendance/queue?status=active"
    IF UBOUND(my_sessions) = 0 THEN
        TALK "Você não tem nenhuma sessão de atendimento ativa no momento para transferir."
        RETURN
    END IF
    IF UBOUND(my_sessions) = 1 THEN
        session_id = FIRST(my_sessions).session_id
    ELSE
        TALK "Você tem " + UBOUND(my_sessions) + " atendimentos ativos. Vou mostrar os nomes, por favor diga qual deseja transferir:"
        FOR EACH s IN my_sessions
            TALK "- **" + s.user_name + "** via " + s.channel
        NEXT s
        HEAR query AS STRING
        ' Busca simplificada
        FOR EACH s IN my_sessions
            IF INSTR(LCASE(s.user_name), LCASE(query)) > 0 THEN
                session_id = s.session_id
            END IF
        NEXT s
        IF NOT session_id THEN
            TALK "Não consegui identificar a sessão."
            RETURN
        END IF
    END IF
END IF

' TRANSFER TO HUMAN envia para a roleta novamente
' No BASIC real, seria executado ou POSTado na API de transferência

transfer_payload = #{
    required_skill: skill,
    priority: priority,
    transfer_reason: reason,
    transferred_by: GET "session.user_id"
}

POST "/api/attendance/sessions/" + session_id + "/transfer", transfer_payload

TALK "✅ **Sessão Transferida!**"
TALK "Motivo: " + reason
IF skill THEN
    TALK "Aguardando um especialista em **" + skill + "**"
ELSE
    TALK "Aguardando o próximo atendente livre."
END IF

RETURN session_id
