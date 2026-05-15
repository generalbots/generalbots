PARAM status AS STRING LIKE "waiting" DESCRIPTION "Filtrar por status: waiting, active, all" OPTIONAL

DESCRIPTION "Ver a fila de atendimento: quem está aguardando, há quanto tempo, e por qual canal."

IF NOT status THEN
    status = "waiting"
END IF

queue = GET "/api/attendance/queue?status=" + status

item_count = UBOUND(queue)

IF item_count = 0 THEN
    TALK "✅ Fila vazia! Nenhum cliente aguardando."
    RETURN
END IF

TALK "📋 **Fila de Atendimento — " + item_count + " cliente(s)**"
TALK ""

FOR EACH item IN queue
    channel_icon = "💬"
    IF item.channel = "whatsapp" THEN
        channel_icon = "📱"
    ELSE IF item.channel = "telegram" THEN
        channel_icon = "✈️"
    ELSE IF item.channel = "email" THEN
        channel_icon = "📧"
    END IF

    TALK "---"
    TALK channel_icon + " **" + item.user_name + "** via " + item.channel
    TALK "⏱️ Aguardando há " + item.wait_time
    TALK "💬 Última mensagem: " + item.last_message

    IF item.assigned_to THEN
        TALK "👤 Atendente: " + item.assigned_to
    END IF

    TALK "🔗 Session: " + item.session_id
NEXT item

RETURN queue
