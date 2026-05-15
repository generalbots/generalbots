PARAM session_id AS STRING LIKE "uuid" DESCRIPTION "ID da sessão atendida" OPTIONAL
PARAM status AS STRING LIKE "resolved" DESCRIPTION "Finalizar o chat ou aguardar (resolved, pending_customer)" OPTIONAL
PARAM summary AS STRING LIKE "Dúvida resolvida com sucesso" DESCRIPTION "Breve resumo da solução" OPTIONAL

DESCRIPTION "Resolve a conversa, registrando o final no CRM (atividade) ou deixa pendente pelo cliente."

IF NOT status THEN
    status = "resolved"
END IF

IF status = "resolved" THEN
    IF NOT summary THEN
        TALK "A sessão foi totalmente concluída? Qual seria o resumo?"
        HEAR summary AS STRING
    END IF
END IF

' Tratar ID da sessão conversacionalmente
IF NOT session_id THEN
    my_sessions = GET "/api/attendance/queue?status=active"
    IF UBOUND(my_sessions) = 0 THEN
        TALK "Você não tem nenhuma sessão de atendimento ativa no momento."
        RETURN
    END IF
    IF UBOUND(my_sessions) = 1 THEN
        session_id = FIRST(my_sessions).session_id
    ELSE
        TALK "Você tem várias sessões ativas. Qual delas quer resolver? Diga o nome do cliente."
        FOR EACH s IN my_sessions
            TALK "- **" + s.user_name + "**"
        NEXT s
        HEAR query AS STRING
        FOR EACH s IN my_sessions
            IF INSTR(LCASE(s.user_name), LCASE(query)) > 0 THEN
                session_id = s.session_id
            END IF
        NEXT s
        IF NOT session_id THEN
            TALK "Sessão não identificada."
            RETURN
        END IF
    END IF
END IF

resolution_payload = #{
    status: status,
    resolution_summary: summary,
    resolved_by: GET "session.user_id",
    resolved_at: FORMAT(NOW(), "YYYY-MM-DD HH:mm:ss")
}

PUT "/api/attendance/sessions/" + session_id + "/resolve", resolution_payload

IF status = "resolved" THEN
    TALK "✅ **Atendimento Encerrado!**"
    TALK "A conversa foi marcada como **Resolvida**."
    TALK "Resumo: " + summary
ELSE
    TALK "⏳ **Aguardando Cliente**"
    TALK "O atendimento ficará pausado aguardando retorno do cliente."
END IF

RETURN session_id
