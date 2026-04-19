REM Attendance CRM Bot - Intelligent Routing with AI-Assisted Human Handoff
REM This bot handles customer inquiries, detects frustration, and seamlessly
REM transfers to human attendants when needed. Uses LLM assist features.

DESCRIPTION "Main entry point for Attendance CRM - routes to bot or human based on context"

' Check if this is a returning customer
customer_id = GET "session.customer_id"
is_returning = customer_id IS NOT NULL

' Check if session already needs human
needs_human = GET "session.needs_human"
IF needs_human = TRUE THEN
    ' Already in human mode - message goes to attendant automatically
    RETURN
END IF

' Get the customer's message
message = GET "user.message"

' Analyze sentiment immediately
sentiment = ANALYZE SENTIMENT session.id, message

' Store sentiment for tracking
SET "session.last_sentiment", sentiment.overall
SET "session.escalation_risk", sentiment.escalation_risk

' Track frustration count
frustration_count = GET "session.frustration_count"
IF frustration_count IS NULL THEN
    frustration_count = 0
END IF

IF sentiment.overall = "negative" THEN
    frustration_count = frustration_count + 1
    SET "session.frustration_count", frustration_count
END IF

' Auto-transfer on high frustration or explicit request
transfer_keywords = ["falar com humano", "talk to human", "atendente", "pessoa real", "real person", "speak to someone", "manager", "gerente", "supervisor"]

should_transfer = FALSE
transfer_reason = ""

' Check for explicit transfer request
FOR EACH keyword IN transfer_keywords
    IF LOWER(message) CONTAINS keyword THEN
        should_transfer = TRUE
        transfer_reason = "Customer requested human assistance"
    END IF
NEXT

' Check for high escalation risk
IF sentiment.escalation_risk = "high" THEN
    should_transfer = TRUE
    transfer_reason = "High escalation risk detected - sentiment: " + sentiment.overall
END IF

' Check for repeated frustration
IF frustration_count >= 3 THEN
    should_transfer = TRUE
    transfer_reason = "Customer frustrated after " + frustration_count + " messages"
END IF

' Execute transfer if needed
IF should_transfer THEN
    ' Get tips for the attendant before transfer
    tips = GET TIPS session.id, message

    ' Build context for attendant
    context_summary = "Customer: " + (GET "session.customer_name" OR "Unknown") + "\n"
    context_summary = context_summary + "Channel: " + (GET "session.channel" OR "web") + "\n"
    context_summary = context_summary + "Sentiment: " + sentiment.emoji + " " + sentiment.overall + "\n"
    context_summary = context_summary + "Escalation Risk: " + sentiment.escalation_risk + "\n"
    context_summary = context_summary + "Frustration Count: " + frustration_count + "\n"

    IF tips.success AND UBOUND(tips.items) > 0 THEN
        context_summary = context_summary + "\nAI Tips:\n"
        FOR EACH tip IN tips.items
            context_summary = context_summary + "- " + tip.content + "\n"
        NEXT
    END IF

    ' Set priority based on sentiment
    priority = "normal"
    IF sentiment.escalation_risk = "high" OR sentiment.urgency = "urgent" THEN
        priority = "urgent"
    ELSE IF sentiment.overall = "negative" THEN
        priority = "high"
    END IF

    ' Transfer to human
    result = TRANSFER TO HUMAN "support", priority, context_summary

    IF result.success THEN
        IF result.status = "assigned" THEN
            TALK "Estou transferindo você para " + result.assigned_to_name + ". Um momento, por favor."
            TALK "I'm connecting you with " + result.assigned_to_name + ". Please hold."
        ELSE IF result.status = "queued" THEN
            TALK "Você está na posição " + result.queue_position + " da fila. Tempo estimado: " + (result.estimated_wait_seconds / 60) + " minutos."
            TALK "You are #" + result.queue_position + " in queue. Estimated wait: " + (result.estimated_wait_seconds / 60) + " minutes."
        END IF

        ' Tag the conversation
        TAG CONVERSATION session.id, "transferred"
        TAG CONVERSATION session.id, sentiment.overall

        ' Add note for attendant
        ADD NOTE session.id, "Auto-transferred: " + transfer_reason

        RETURN
    ELSE
        ' Transfer failed - continue with bot but apologize
        TALK "Nossos atendentes estão ocupados no momento. Vou fazer o meu melhor para ajudá-lo."
        TALK "Our agents are currently busy. I'll do my best to help you."
    END IF
END IF

' Continue with bot processing
' Check for common intents
message_lower = LOWER(message)

IF message_lower CONTAINS "preço" OR message_lower CONTAINS "price" OR message_lower CONTAINS "quanto custa" OR message_lower CONTAINS "how much" THEN
    ' Pricing inquiry - could create lead
    SET "session.intent", "pricing"
    USE TOOL "pricing-inquiry"
    RETURN
END IF

IF message_lower CONTAINS "problema" OR message_lower CONTAINS "problem" OR message_lower CONTAINS "não funciona" OR message_lower CONTAINS "not working" THEN
    ' Support issue
    SET "session.intent", "support"
    USE TOOL "support-ticket"
    RETURN
END IF

IF message_lower CONTAINS "status" OR message_lower CONTAINS "pedido" OR message_lower CONTAINS "order" OR message_lower CONTAINS "entrega" OR message_lower CONTAINS "delivery" THEN
    ' Order status
    SET "session.intent", "order_status"
    USE TOOL "order-status"
    RETURN
END IF

IF message_lower CONTAINS "cancelar" OR message_lower CONTAINS "cancel" OR message_lower CONTAINS "reembolso" OR message_lower CONTAINS "refund" THEN
    ' Cancellation/Refund - often needs human
    SET "session.intent", "cancellation"
    SET PRIORITY session.id, "high"

    TALK "Entendo que você deseja cancelar ou obter reembolso. Deixe-me verificar sua conta."
    TALK "I understand you want to cancel or get a refund. Let me check your account."

    ' Check if VIP customer - auto transfer
    IF GET "session.customer_tier" = "vip" OR GET "session.customer_tier" = "enterprise" THEN
        result = TRANSFER TO HUMAN "support", "high", "VIP customer requesting cancellation/refund"
        IF result.success THEN
            TALK "Como cliente VIP, estou conectando você diretamente com nossa equipe especializada."
            RETURN
        END IF
    END IF

    USE TOOL "cancellation-flow"
    RETURN
END IF

' Default: Use LLM for general conversation
response = LLM "Respond helpfully to: " + message
TALK response

' After each bot response, check if we should suggest human
IF sentiment.overall = "negative" THEN
    TALK "Se preferir falar com um atendente humano, é só me avisar."
    TALK "If you'd prefer to speak with a human agent, just let me know."
END IF
