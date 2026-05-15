REM Attendant Helper - LLM Assist Tools for Human Agents
REM Provides AI-powered assistance to human attendants during conversations.
REM Can be called manually by attendant or triggered automatically.
REM
REM Usage from WhatsApp: /tips, /polish, /replies, /summary
REM Usage from Web Console: Buttons in attendant UI

PARAM action AS STRING LIKE "tips" DESCRIPTION "Action: tips, polish, replies, summary, sentiment, suggest_transfer"
PARAM session_id AS STRING DESCRIPTION "Session ID of the conversation"
PARAM message AS STRING DESCRIPTION "Message to analyze or polish"
PARAM tone AS STRING LIKE "professional" DESCRIPTION "Tone for polish: professional, friendly, empathetic, formal"

DESCRIPTION "AI-powered tools to help human attendants respond faster and better"

' Validate session
IF session_id IS NULL OR session_id = "" THEN
    RETURN {"success": FALSE, "error": "Session ID required"}
END IF

' Get session context
customer_name = GET SESSION session_id, "name"
channel = GET SESSION session_id, "channel"
customer_tier = GET SESSION session_id, "customer_tier"
tags = GET SESSION session_id, "tags"

' =====================================================================
' ACTION: TIPS - Generate contextual tips for the attendant
' =====================================================================
IF action = "tips" THEN
    IF message IS NULL OR message = "" THEN
        ' Get last customer message from session
        message = GET SESSION session_id, "last_customer_message"
    END IF

    IF message IS NULL OR message = "" THEN
        RETURN {"success": FALSE, "error": "No message to analyze"}
    END IF

    ' Generate tips using LLM assist
    tips = GET TIPS session_id, message

    ' Add context-specific tips
    context_tips = []

    ' VIP customer tip
    IF customer_tier = "vip" OR customer_tier = "enterprise" THEN
        APPEND context_tips, {
            "type": "warning",
            "content": "⭐ VIP Customer - Handle with extra care and priority",
            "priority": 1
        }
    END IF

    ' Check customer history
    history = GET CUSTOMER HISTORY GET SESSION session_id, "user_id"
    IF history.session_count > 5 THEN
        APPEND context_tips, {
            "type": "history",
            "content": "Returning customer with " + history.session_count + " previous interactions",
            "priority": 2
        }
    END IF

    ' Check for open issues
    open_cases = FIND "cases", "customer_id='" + GET SESSION session_id, "customer_id" + "' AND status='open'"
    IF UBOUND(open_cases) > 0 THEN
        APPEND context_tips, {
            "type": "warning",
            "content": "Customer has " + UBOUND(open_cases) + " open support cases",
            "priority": 1
        }
    END IF

    ' Merge context tips with LLM tips
    all_tips = []
    FOR EACH tip IN context_tips
        APPEND all_tips, tip
    NEXT
    FOR EACH tip IN tips.items
        APPEND all_tips, tip
    NEXT

    ' Sort by priority
    all_tips = SORT all_tips BY "priority" ASC

    RETURN {
        "success": TRUE,
        "customer": customer_name,
        "channel": channel,
        "tips": all_tips
    }
END IF

' =====================================================================
' ACTION: POLISH - Improve message before sending
' =====================================================================
IF action = "polish" THEN
    IF message IS NULL OR message = "" THEN
        RETURN {"success": FALSE, "error": "No message to polish"}
    END IF

    ' Default tone if not specified
    IF tone IS NULL OR tone = "" THEN
        tone = "professional"

        ' Adjust tone based on context
        sentiment = GET SESSION session_id, "last_sentiment"
        IF sentiment = "negative" THEN
            tone = "empathetic"
        END IF

        IF customer_tier = "vip" THEN
            tone = "formal"
        END IF
    END IF

    ' Polish the message
    result = POLISH MESSAGE message, tone

    ' Add customer name if not present
    polished = result.polished
    IF NOT (polished CONTAINS customer_name) AND customer_name IS NOT NULL AND customer_name <> "Unknown" THEN
        ' Check if it starts with a greeting
        IF polished STARTS WITH "Olá" OR polished STARTS WITH "Oi" OR polished STARTS WITH "Hi" OR polished STARTS WITH "Hello" THEN
            ' Add name after greeting
            polished = REPLACE(polished, "Olá", "Olá " + customer_name)
            polished = REPLACE(polished, "Oi", "Oi " + customer_name)
            polished = REPLACE(polished, "Hi", "Hi " + customer_name)
            polished = REPLACE(polished, "Hello", "Hello " + customer_name)
        END IF
    END IF

    RETURN {
        "success": TRUE,
        "original": message,
        "polished": polished,
        "tone": tone,
        "changes": result.changes
    }
END IF

' =====================================================================
' ACTION: REPLIES - Get smart reply suggestions
' =====================================================================
IF action = "replies" THEN
    ' Get conversation history for context
    history_items = GET SESSION session_id, "message_history"

    ' Build history array for API
    history = []
    IF history_items IS NOT NULL THEN
        FOR EACH msg IN history_items LAST 10
            APPEND history, {
                "role": msg.sender,
                "content": msg.text,
                "timestamp": msg.timestamp
            }
        NEXT
    END IF

    ' Get smart replies
    replies = GET SMART REPLIES session_id

    ' Customize based on context
    customized_replies = []

    FOR EACH reply IN replies.items
        custom_reply = reply

        ' Add customer name to greeting replies
        IF reply.category = "greeting" AND customer_name IS NOT NULL AND customer_name <> "Unknown" THEN
            custom_reply.text = REPLACE(reply.text, "!", ", " + customer_name + "!")
        END IF

        ' Add urgency for VIP
        IF customer_tier = "vip" AND reply.category = "acknowledgment" THEN
            custom_reply.text = "Como cliente prioritário, " + LOWER(LEFT(reply.text, 1)) + MID(reply.text, 2)
        END IF

        APPEND customized_replies, custom_reply
    NEXT

    ' Add context-specific suggestions
    last_intent = GET SESSION session_id, "intent"

    IF last_intent = "pricing" THEN
        APPEND customized_replies, {
            "text": "Posso preparar uma proposta personalizada para você. Qual seria o melhor email para enviar?",
            "tone": "professional",
            "category": "action",
            "confidence": 0.9
        }
    END IF

    IF last_intent = "support" THEN
        APPEND customized_replies, {
            "text": "Vou criar um ticket de suporte para acompanhar sua solicitação. Pode me dar mais detalhes do problema?",
            "tone": "helpful",
            "category": "action",
            "confidence": 0.85
        }
    END IF

    IF last_intent = "cancellation" THEN
        APPEND customized_replies, {
            "text": "Antes de prosseguir, gostaria de entender melhor o motivo. Há algo que possamos fazer para resolver sua insatisfação?",
            "tone": "empathetic",
            "category": "retention",
            "confidence": 0.9
        }
    END IF

    RETURN {
        "success": TRUE,
        "replies": customized_replies,
        "context": {
            "customer": customer_name,
            "tier": customer_tier,
            "intent": last_intent
        }
    }
END IF

' =====================================================================
' ACTION: SUMMARY - Get conversation summary
' =====================================================================
IF action = "summary" THEN
    ' Get LLM summary
    summary = GET SUMMARY session_id

    ' Enhance with CRM data
    customer_id = GET SESSION session_id, "customer_id"

    IF customer_id IS NOT NULL THEN
        ' Get account info
        account = FIND "accounts", "id='" + customer_id + "'"
        IF account IS NOT NULL THEN
            summary.account_name = account.name
            summary.account_type = account.type
            summary.account_since = account.created_at
        END IF

        ' Get recent orders
        recent_orders = FIND "orders", "customer_id='" + customer_id + "' ORDER BY created_at DESC LIMIT 3"
        summary.recent_orders = []
        FOR EACH order IN recent_orders
            APPEND summary.recent_orders, {
                "id": order.id,
                "date": order.created_at,
                "total": order.total,
                "status": order.status
            }
        NEXT

        ' Get open cases
        open_cases = FIND "cases", "customer_id='" + customer_id + "' AND status='open'"
        summary.open_cases = UBOUND(open_cases)

        ' Calculate customer value
        total_orders = FIND "orders", "customer_id='" + customer_id + "'"
        total_value = 0
        FOR EACH order IN total_orders
            total_value = total_value + order.total
        NEXT
        summary.lifetime_value = total_value
    END IF

    ' Get notes from this session
    notes = GET SESSION session_id, "notes"
    summary.internal_notes = notes

    ' Get tags
    summary.tags = tags

    RETURN {
        "success": TRUE,
        "summary": summary
    }
END IF

' =====================================================================
' ACTION: SENTIMENT - Analyze current sentiment
' =====================================================================
IF action = "sentiment" THEN
    IF message IS NULL OR message = "" THEN
        message = GET SESSION session_id, "last_customer_message"
    END IF

    IF message IS NULL OR message = "" THEN
        RETURN {"success": FALSE, "error": "No message to analyze"}
    END IF

    ' Get sentiment analysis
    sentiment = ANALYZE SENTIMENT session_id, message

    ' Get trend
    previous_sentiment = GET SESSION session_id, "sentiment_history"
    trend = "stable"

    IF previous_sentiment IS NOT NULL AND UBOUND(previous_sentiment) >= 2 THEN
        last_two = SLICE(previous_sentiment, -2)
        IF sentiment.score > last_two[1].score AND last_two[1].score > last_two[0].score THEN
            trend = "improving"
        ELSE IF sentiment.score < last_two[1].score AND last_two[1].score < last_two[0].score THEN
            trend = "declining"
        END IF
    END IF

    ' Store for tracking
    IF previous_sentiment IS NULL THEN
        previous_sentiment = []
    END IF
    APPEND previous_sentiment, {"score": sentiment.score, "timestamp": NOW()}
    SET SESSION session_id, "sentiment_history", previous_sentiment

    ' Add recommendations based on sentiment
    recommendations = []

    IF sentiment.escalation_risk = "high" THEN
        APPEND recommendations, "Consider offering compensation or immediate resolution"
        APPEND recommendations, "Use empathetic language and acknowledge frustration"
        APPEND recommendations, "Avoid technical jargon - keep it simple"
    END IF

    IF sentiment.urgency = "urgent" THEN
        APPEND recommendations, "Respond quickly - customer is time-sensitive"
        APPEND recommendations, "Provide immediate action or timeline"
    END IF

    IF sentiment.overall = "positive" THEN
        APPEND recommendations, "Good opportunity for upsell or feedback request"
        APPEND recommendations, "Ask for referral or review"
    END IF

    RETURN {
        "success": TRUE,
        "sentiment": sentiment,
        "trend": trend,
        "recommendations": recommendations
    }
END IF

' =====================================================================
' ACTION: SUGGEST_TRANSFER - Check if transfer is recommended
' =====================================================================
IF action = "suggest_transfer" THEN
    ' Analyze situation
    sentiment = ANALYZE SENTIMENT session_id, GET SESSION session_id, "last_customer_message"
    frustration_count = GET SESSION session_id, "frustration_count"
    message_count = GET SESSION session_id, "message_count"
    intent = GET SESSION session_id, "intent"

    should_transfer = FALSE
    transfer_reason = ""
    transfer_department = "support"
    transfer_priority = "normal"

    ' High escalation risk
    IF sentiment.escalation_risk = "high" THEN
        should_transfer = TRUE
        transfer_reason = "High escalation risk - customer very frustrated"
        transfer_priority = "urgent"
    END IF

    ' Technical issue beyond first-line
    IF intent = "technical" AND message_count > 5 THEN
        should_transfer = TRUE
        transfer_reason = "Complex technical issue - needs specialist"
        transfer_department = "technical"
    END IF

    ' Billing/refund issues
    IF intent = "cancellation" OR intent = "refund" THEN
        IF customer_tier = "vip" OR customer_tier = "enterprise" THEN
            should_transfer = TRUE
            transfer_reason = "VIP customer with billing/refund request"
            transfer_department = "finance"
            transfer_priority = "high"
        END IF
    END IF

    ' Long conversation without resolution
    IF message_count > 10 AND NOT should_transfer THEN
        should_transfer = TRUE
        transfer_reason = "Extended conversation - may need escalation"
    END IF

    ' Get available attendants for department
    available = GET ATTENDANTS "online"
    department_available = []
    FOR EACH att IN available.items
        IF att.department = transfer_department OR att.channel = "all" THEN
            APPEND department_available, att
        END IF
    NEXT

    RETURN {
        "success": TRUE,
        "should_transfer": should_transfer,
        "reason": transfer_reason,
        "suggested_department": transfer_department,
        "suggested_priority": transfer_priority,
        "available_attendants": department_available,
        "context": {
            "sentiment": sentiment.overall,
            "escalation_risk": sentiment.escalation_risk,
            "message_count": message_count,
            "intent": intent,
            "customer_tier": customer_tier
        }
    }
END IF

' Unknown action
RETURN {"success": FALSE, "error": "Unknown action: " + action}
