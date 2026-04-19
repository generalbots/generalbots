REM Queue Monitor - Automated Queue Management
REM Runs on schedule to monitor queue health, reassign stale conversations,
REM notify supervisors of issues, and generate queue metrics.
REM
REM Schedule: SET SCHEDULE "queue-monitor", "*/5 * * * *" (every 5 minutes)

DESCRIPTION "Monitor queue health, reassign stale conversations, alert on issues"

' Get current queue status
queue = GET QUEUE

PRINT "Queue Monitor Running - " + FORMAT(NOW(), "yyyy-MM-dd HH:mm:ss")
PRINT "Total: " + queue.total + " | Waiting: " + queue.waiting + " | Active: " + queue.active

' === Check for stale waiting conversations ===
' Conversations waiting more than 10 minutes without assignment
stale_threshold_minutes = 10

FOR EACH item IN queue.items
    IF item.status = "waiting" THEN
        ' Calculate wait time
        created_at = GET SESSION item.session_id, "created_at"
        wait_minutes = DATEDIFF("minute", created_at, NOW())

        IF wait_minutes > stale_threshold_minutes THEN
            PRINT "ALERT: Stale conversation " + item.session_id + " waiting " + wait_minutes + " minutes"

            ' Try to find available attendant
            attendants = GET ATTENDANTS "online"

            IF attendants.count > 0 THEN
                ' Find attendant with least active conversations
                best_attendant = NULL
                min_active = 999

                FOR EACH att IN attendants.items
                    stats = GET ATTENDANT STATS att.id
                    IF stats.active_conversations < min_active THEN
                        min_active = stats.active_conversations
                        best_attendant = att
                    END IF
                NEXT

                IF best_attendant IS NOT NULL AND min_active < 5 THEN
                    ' Auto-assign
                    result = ASSIGN CONVERSATION item.session_id, best_attendant.id

                    IF result.success THEN
                        PRINT "Auto-assigned " + item.session_id + " to " + best_attendant.name
                        ADD NOTE item.session_id, "Auto-assigned after " + wait_minutes + " min wait"

                        ' Notify customer
                        SEND TO SESSION item.session_id, "Obrigado por aguardar! " + best_attendant.name + " irá atendê-lo agora."
                    END IF
                END IF
            ELSE
                ' No attendants available - escalate
                IF wait_minutes > 20 THEN
                    ' Critical - notify supervisor
                    SEND MAIL "supervisor@company.com", "URGENTE: Fila sem atendentes", "Conversa " + item.session_id + " aguardando há " + wait_minutes + " minutos sem atendentes disponíveis."

                    ' Set high priority
                    SET PRIORITY item.session_id, "urgent"
                    TAG CONVERSATION item.session_id, "no-attendants"

                    ' Send apology to customer
                    SEND TO SESSION item.session_id, "Pedimos desculpas pela espera. Nossa equipe está com alta demanda. Você será atendido em breve."
                END IF
            END IF
        END IF
    END IF
NEXT

' === Check for inactive assigned conversations ===
' Conversations assigned but no activity for 15 minutes
inactive_threshold_minutes = 15

FOR EACH item IN queue.items
    IF item.status = "assigned" OR item.status = "active" THEN
        last_activity = GET SESSION item.session_id, "last_activity_at"

        IF last_activity IS NOT NULL THEN
            inactive_minutes = DATEDIFF("minute", last_activity, NOW())

            IF inactive_minutes > inactive_threshold_minutes THEN
                PRINT "WARNING: Inactive conversation " + item.session_id + " - " + inactive_minutes + " min since last activity"

                ' Get assigned attendant
                assigned_to = GET SESSION item.session_id, "assigned_to"

                IF assigned_to IS NOT NULL THEN
                    ' Check attendant status
                    att_status = GET ATTENDANT STATUS assigned_to

                    IF att_status = "offline" OR att_status = "away" THEN
                        ' Attendant went away - reassign
                        PRINT "Attendant " + assigned_to + " is " + att_status + " - reassigning"

                        ' Find new attendant
                        new_attendants = GET ATTENDANTS "online"
                        IF new_attendants.count > 0 THEN
                            new_att = new_attendants.items[0]

                            ' Transfer conversation
                            old_ctx = GET SESSION item.session_id, "context"
                            result = ASSIGN CONVERSATION item.session_id, new_att.id

                            IF result.success THEN
                                ADD NOTE item.session_id, "Reatribuído de " + assigned_to + " (status: " + att_status + ") para " + new_att.name
                                SEND TO SESSION item.session_id, "Desculpe a espera. " + new_att.name + " continuará seu atendimento."
                            END IF
                        END IF
                    ELSE
                        ' Attendant is online but inactive - send reminder
                        ' Only remind once every 10 minutes
                        last_reminder = GET SESSION item.session_id, "last_attendant_reminder"

                        IF last_reminder IS NULL OR DATEDIFF("minute", last_reminder, NOW()) > 10 THEN
                            ' Get customer sentiment
                            sentiment = GET SESSION item.session_id, "last_sentiment"

                            reminder = "⚠️ Conversa inativa há " + inactive_minutes + " min"
                            IF sentiment = "negative" THEN
                                reminder = reminder + " - Cliente frustrado!"
                            END IF

                            ' Send reminder via WebSocket to attendant UI
                            NOTIFY ATTENDANT assigned_to, reminder
                            SET SESSION item.session_id, "last_attendant_reminder", NOW()
                        END IF
                    END IF
                END IF
            END IF
        END IF
    END IF
NEXT

' === Check for abandoned conversations ===
' Customer hasn't responded in 30 minutes
abandon_threshold_minutes = 30

FOR EACH item IN queue.items
    IF item.status = "active" OR item.status = "assigned" THEN
        last_customer_msg = GET SESSION item.session_id, "last_customer_message_at"

        IF last_customer_msg IS NOT NULL THEN
            silent_minutes = DATEDIFF("minute", last_customer_msg, NOW())

            IF silent_minutes > abandon_threshold_minutes THEN
                ' Check if already marked
                already_marked = GET SESSION item.session_id, "abandon_warning_sent"

                IF already_marked IS NULL THEN
                    ' Send follow-up
                    SEND TO SESSION item.session_id, "Ainda está aí? Se precisar de mais ajuda, é só responder."
                    SET SESSION item.session_id, "abandon_warning_sent", NOW()
                    PRINT "Sent follow-up to potentially abandoned session " + item.session_id
                ELSE
                    ' Check if warning was sent more than 15 min ago
                    warning_minutes = DATEDIFF("minute", already_marked, NOW())

                    IF warning_minutes > 15 THEN
                        ' Mark as abandoned
                        RESOLVE CONVERSATION item.session_id, "Abandoned - no customer response"
                        TAG CONVERSATION item.session_id, "abandoned"
                        PRINT "Marked session " + item.session_id + " as abandoned"
                    END IF
                END IF
            END IF
        END IF
    END IF
NEXT

' === Generate queue metrics ===
' Calculate averages and store for analytics

metrics = {}
metrics.timestamp = NOW()
metrics.total_waiting = queue.waiting
metrics.total_active = queue.active
metrics.total_resolved = queue.resolved

' Calculate average wait time for waiting conversations
total_wait = 0
wait_count = 0

FOR EACH item IN queue.items
    IF item.status = "waiting" THEN
        created_at = GET SESSION item.session_id, "created_at"
        wait_minutes = DATEDIFF("minute", created_at, NOW())
        total_wait = total_wait + wait_minutes
        wait_count = wait_count + 1
    END IF
NEXT

IF wait_count > 0 THEN
    metrics.avg_wait_minutes = total_wait / wait_count
ELSE
    metrics.avg_wait_minutes = 0
END IF

' Get attendant utilization
attendants = GET ATTENDANTS
online_count = 0
busy_count = 0

FOR EACH att IN attendants.items
    IF att.status = "online" THEN
        online_count = online_count + 1
    ELSE IF att.status = "busy" THEN
        busy_count = busy_count + 1
    END IF
NEXT

metrics.attendants_online = online_count
metrics.attendants_busy = busy_count
metrics.utilization_pct = 0

IF online_count + busy_count > 0 THEN
    metrics.utilization_pct = (busy_count / (online_count + busy_count)) * 100
END IF

' Store metrics for dashboard
SAVE "queue_metrics", metrics

' Alert if queue is getting long
IF queue.waiting > 10 THEN
    SEND MAIL "supervisor@company.com", "Alerta: Fila com " + queue.waiting + " aguardando", "A fila de atendimento está com " + queue.waiting + " conversas aguardando. Tempo médio de espera: " + metrics.avg_wait_minutes + " minutos."
END IF

' Alert if no attendants online during business hours
hour_now = HOUR(NOW())
day_now = WEEKDAY(NOW())

IF hour_now >= 9 AND hour_now < 18 AND day_now >= 1 AND day_now <= 5 THEN
    IF online_count = 0 AND busy_count = 0 THEN
        SEND MAIL "supervisor@company.com", "CRÍTICO: Sem atendentes online", "Não há atendentes online durante o horário comercial. Fila: " + queue.waiting + " aguardando."
    END IF
END IF

PRINT "Queue monitor completed. Metrics saved."
RETURN metrics
