REM CRM Automations - Follow-ups, Collections, Lead Nurturing, Sales Pipeline
REM Scheduled jobs that run automatically to handle common CRM workflows.
REM Uses the new attendance keywords for queue integration.
REM
REM Schedules:
REM   SET SCHEDULE "follow-ups", "0 9 * * 1-5"       (9am weekdays)
REM   SET SCHEDULE "collections", "0 8 * * 1-5"      (8am weekdays)
REM   SET SCHEDULE "lead-nurture", "0 10 * * 1-5"    (10am weekdays)
REM   SET SCHEDULE "pipeline-review", "0 14 * * 5"   (2pm Fridays)

PARAM job_name AS STRING LIKE "follow-ups" DESCRIPTION "Job to run: follow-ups, collections, lead-nurture, pipeline-review, all"

DESCRIPTION "Automated CRM workflows for follow-ups, collections, lead nurturing, and pipeline management"

PRINT "=== CRM Automations Starting: " + job_name + " ==="
PRINT "Time: " + FORMAT(NOW(), "yyyy-MM-dd HH:mm:ss")

results = {}
results.job = job_name
results.started_at = NOW()
results.actions = []

' =====================================================================
' FOLLOW-UPS - Automated follow-up sequences
' =====================================================================
IF job_name = "follow-ups" OR job_name = "all" THEN
    PRINT ""
    PRINT "--- Running Follow-ups ---"

    ' 1-day follow-up: Thank you message for new leads
    leads_1_day = FIND "leads", "status='new' AND DATEDIFF(NOW(), created_at) = 1"
    PRINT "1-day follow-ups: " + UBOUND(leads_1_day)

    FOR EACH lead IN leads_1_day
        ' Send thank you message
        IF lead.phone IS NOT NULL THEN
            SEND TEMPLATE lead.phone, "follow_up_thanks", {
                "name": lead.name,
                "interest": lead.interest OR "our services"
            }
        END IF

        IF lead.email IS NOT NULL THEN
            SEND MAIL lead.email, "Obrigado pelo seu interesse!", "Olá " + lead.name + ",\n\nObrigado por entrar em contato conosco. Estamos à disposição para ajudá-lo.\n\nAtenciosamente,\nEquipe de Vendas"
        END IF

        ' Update lead status
        UPDATE "leads", "id='" + lead.id + "'", {
            "status": "contacted",
            "last_contact": NOW(),
            "follow_up_1_sent": NOW()
        }

        ' Log activity
        SAVE "activities", {
            "type": "follow_up",
            "lead_id": lead.id,
            "description": "1-day thank you sent",
            "created_at": NOW()
        }

        APPEND results.actions, {"type": "follow_up_1", "lead": lead.name}
    NEXT

    ' 3-day follow-up: Value proposition
    leads_3_day = FIND "leads", "status='contacted' AND DATEDIFF(NOW(), last_contact) = 3 AND follow_up_3_sent IS NULL"
    PRINT "3-day follow-ups: " + UBOUND(leads_3_day)

    FOR EACH lead IN leads_3_day
        IF lead.phone IS NOT NULL THEN
            SEND TEMPLATE lead.phone, "follow_up_value", {
                "name": lead.name,
                "interest": lead.interest OR "our services"
            }
        END IF

        UPDATE "leads", "id='" + lead.id + "'", {
            "last_contact": NOW(),
            "follow_up_3_sent": NOW()
        }

        SAVE "activities", {
            "type": "follow_up",
            "lead_id": lead.id,
            "description": "3-day value proposition sent",
            "created_at": NOW()
        }

        APPEND results.actions, {"type": "follow_up_3", "lead": lead.name}
    NEXT

    ' 7-day follow-up: Special offer for hot leads
    leads_7_day = FIND "leads", "status IN ('contacted', 'nurturing') AND DATEDIFF(NOW(), last_contact) = 7 AND follow_up_7_sent IS NULL AND score >= 50"
    PRINT "7-day follow-ups: " + UBOUND(leads_7_day)

    FOR EACH lead IN leads_7_day
        IF lead.phone IS NOT NULL THEN
            SEND TEMPLATE lead.phone, "follow_up_offer", {
                "name": lead.name,
                "discount": "10%"
            }
        END IF

        UPDATE "leads", "id='" + lead.id + "'", {
            "status": "offer_sent",
            "last_contact": NOW(),
            "follow_up_7_sent": NOW()
        }

        ' Alert sales for hot leads
        IF lead.score >= 70 THEN
            attendants = GET ATTENDANTS "online"
            FOR EACH att IN attendants.items
                IF att.department = "commercial" OR att.preferences CONTAINS "sales" THEN
                    SEND MAIL att.email, "🔥 Hot Lead Follow-up: " + lead.name, "Lead " + lead.name + " recebeu oferta de 7 dias.\nScore: " + lead.score + "\nTelefone: " + lead.phone + "\n\nRecomendado: Entrar em contato nas próximas 24h."
                END IF
            NEXT
        END IF

        APPEND results.actions, {"type": "follow_up_7", "lead": lead.name, "score": lead.score}
    NEXT

    results.follow_ups_completed = UBOUND(leads_1_day) + UBOUND(leads_3_day) + UBOUND(leads_7_day)
    PRINT "Follow-ups completed: " + results.follow_ups_completed
END IF

' =====================================================================
' COLLECTIONS - Automated payment reminders (Cobranças)
' =====================================================================
IF job_name = "collections" OR job_name = "all" THEN
    PRINT ""
    PRINT "--- Running Collections ---"

    ' Due today: Friendly reminder
    due_today = FIND "invoices", "status='pending' AND due_date = CURDATE()"
    PRINT "Due today: " + UBOUND(due_today)

    FOR EACH invoice IN due_today
        customer = FIND "customers", "id='" + invoice.customer_id + "'"

        IF customer.phone IS NOT NULL THEN
            SEND TEMPLATE customer.phone, "payment_due_today", {
                "name": customer.name,
                "invoice_id": invoice.id,
                "amount": FORMAT(invoice.amount, "R$ #,##0.00")
            }
        END IF

        SAVE "collection_log", {
            "invoice_id": invoice.id,
            "action": "reminder_due_today",
            "created_at": NOW()
        }

        APPEND results.actions, {"type": "payment_reminder", "customer": customer.name, "days": 0}
    NEXT

    ' 3 days overdue: First notice
    overdue_3 = FIND "invoices", "status='pending' AND DATEDIFF(NOW(), due_date) = 3"
    PRINT "3 days overdue: " + UBOUND(overdue_3)

    FOR EACH invoice IN overdue_3
        customer = FIND "customers", "id='" + invoice.customer_id + "'"

        IF customer.phone IS NOT NULL THEN
            SEND TEMPLATE customer.phone, "payment_overdue_3", {
                "name": customer.name,
                "invoice_id": invoice.id,
                "amount": FORMAT(invoice.amount, "R$ #,##0.00")
            }
        END IF

        IF customer.email IS NOT NULL THEN
            SEND MAIL customer.email, "Pagamento Pendente - Fatura #" + invoice.id, "Prezado(a) " + customer.name + ",\n\nSua fatura #" + invoice.id + " no valor de R$ " + invoice.amount + " está vencida há 3 dias.\n\nPor favor, regularize o pagamento para evitar encargos adicionais.\n\nEm caso de dúvidas, entre em contato conosco.\n\nAtenciosamente,\nDepartamento Financeiro"
        END IF

        UPDATE "invoices", "id='" + invoice.id + "'", {
            "first_notice_sent": NOW()
        }

        SAVE "collection_log", {
            "invoice_id": invoice.id,
            "action": "first_notice",
            "created_at": NOW()
        }

        APPEND results.actions, {"type": "collection_notice_1", "customer": customer.name}
    NEXT

    ' 7 days overdue: Second notice with urgency
    overdue_7 = FIND "invoices", "status='pending' AND DATEDIFF(NOW(), due_date) = 7"
    PRINT "7 days overdue: " + UBOUND(overdue_7)

    FOR EACH invoice IN overdue_7
        customer = FIND "customers", "id='" + invoice.customer_id + "'"

        IF customer.phone IS NOT NULL THEN
            SEND TEMPLATE customer.phone, "payment_overdue_7", {
                "name": customer.name,
                "invoice_id": invoice.id,
                "amount": FORMAT(invoice.amount, "R$ #,##0.00")
            }
        END IF

        UPDATE "invoices", "id='" + invoice.id + "'", {
            "second_notice_sent": NOW()
        }

        ' Notify collections team
        SEND MAIL "cobranca@company.com", "Cobrança 7 dias: " + customer.name, "Cliente: " + customer.name + "\nFatura: " + invoice.id + "\nValor: R$ " + invoice.amount + "\nTelefone: " + customer.phone

        SAVE "collection_log", {
            "invoice_id": invoice.id,
            "action": "second_notice",
            "created_at": NOW()
        }

        APPEND results.actions, {"type": "collection_notice_2", "customer": customer.name}
    NEXT

    ' 15 days overdue: Final notice + queue for human call
    overdue_15 = FIND "invoices", "status='pending' AND DATEDIFF(NOW(), due_date) = 15"
    PRINT "15 days overdue: " + UBOUND(overdue_15)

    FOR EACH invoice IN overdue_15
        customer = FIND "customers", "id='" + invoice.customer_id + "'"

        ' Calculate late fees
        late_fee = invoice.amount * 0.02
        interest = invoice.amount * 0.01 * 15
        total_due = invoice.amount + late_fee + interest

        IF customer.phone IS NOT NULL THEN
            SEND TEMPLATE customer.phone, "payment_final_notice", {
                "name": customer.name,
                "invoice_id": invoice.id,
                "total": FORMAT(total_due, "R$ #,##0.00")
            }
        END IF

        UPDATE "invoices", "id='" + invoice.id + "'", {
            "late_fee": late_fee,
            "interest": interest,
            "total_due": total_due,
            "final_notice_sent": NOW()
        }

        ' Create task for human follow-up call
        CREATE TASK "Ligar para cliente: " + customer.name + " - Cobrança 15 dias", "cobranca@company.com", NOW()

        ' Find finance attendant and assign
        attendants = GET ATTENDANTS
        FOR EACH att IN attendants.items
            IF att.department = "finance" OR att.preferences CONTAINS "collections" THEN
                ' Create session for outbound call
                SAVE "outbound_queue", {
                    "customer_id": customer.id,
                    "customer_name": customer.name,
                    "customer_phone": customer.phone,
                    "reason": "collection_15_days",
                    "invoice_id": invoice.id,
                    "amount_due": total_due,
                    "assigned_to": att.id,
                    "priority": "high",
                    "created_at": NOW()
                }
            END IF
        NEXT

        SAVE "collection_log", {
            "invoice_id": invoice.id,
            "action": "final_notice",
            "total_due": total_due,
            "created_at": NOW()
        }

        APPEND results.actions, {"type": "collection_final", "customer": customer.name, "total": total_due}
    NEXT

    ' 30+ days: Send to legal/collections agency
    overdue_30 = FIND "invoices", "status='pending' AND DATEDIFF(NOW(), due_date) >= 30 AND status <> 'collections'"
    PRINT "30+ days overdue: " + UBOUND(overdue_30)

    FOR EACH invoice IN overdue_30
        customer = FIND "customers", "id='" + invoice.customer_id + "'"

        UPDATE "invoices", "id='" + invoice.id + "'", {
            "status": "collections",
            "sent_to_collections": NOW()
        }

        UPDATE "customers", "id='" + customer.id + "'", {
            "status": "suspended"
        }

        SEND MAIL "juridico@company.com", "Inadimplência 30+ dias: " + customer.name, "Cliente enviado para cobrança jurídica.\n\nCliente: " + customer.name + "\nFatura: " + invoice.id + "\nValor total: R$ " + invoice.total_due + "\nDias em atraso: " + DATEDIFF(NOW(), invoice.due_date)

        SAVE "collection_log", {
            "invoice_id": invoice.id,
            "action": "sent_to_legal",
            "created_at": NOW()
        }

        APPEND results.actions, {"type": "collection_legal", "customer": customer.name}
    NEXT

    results.collections_processed = UBOUND(due_today) + UBOUND(overdue_3) + UBOUND(overdue_7) + UBOUND(overdue_15) + UBOUND(overdue_30)
    PRINT "Collections processed: " + results.collections_processed
END IF

' =====================================================================
' LEAD NURTURE - Re-engage cold leads
' =====================================================================
IF job_name = "lead-nurture" OR job_name = "all" THEN
    PRINT ""
    PRINT "--- Running Lead Nurture ---"

    ' Find cold leads that haven't been contacted in 30 days
    cold_leads = FIND "leads", "status IN ('cold', 'nurturing') AND DATEDIFF(NOW(), last_contact) >= 30 AND DATEDIFF(NOW(), last_contact) < 90"
    PRINT "Cold leads to nurture: " + UBOUND(cold_leads)

    FOR EACH lead IN cold_leads
        ' Send nurture content based on interest
        content_type = "general"
        IF lead.interest CONTAINS "pricing" OR lead.interest CONTAINS "preço" THEN
            content_type = "pricing_update"
        ELSE IF lead.interest CONTAINS "feature" OR lead.interest CONTAINS "funcionalidade" THEN
            content_type = "feature_update"
        END IF

        IF lead.email IS NOT NULL THEN
            SEND TEMPLATE lead.email, "nurture_" + content_type, {
                "name": lead.name,
                "company": lead.company
            }
        END IF

        UPDATE "leads", "id='" + lead.id + "'", {
            "last_contact": NOW(),
            "nurture_count": (lead.nurture_count OR 0) + 1
        }

        SAVE "activities", {
            "type": "nurture",
            "lead_id": lead.id,
            "description": "Nurture email sent: " + content_type,
            "created_at": NOW()
        }

        APPEND results.actions, {"type": "nurture", "lead": lead.name, "content": content_type}
    NEXT

    ' Archive very old leads (90+ days, low score)
    stale_leads = FIND "leads", "DATEDIFF(NOW(), last_contact) >= 90 AND score < 30 AND status <> 'archived'"
    PRINT "Stale leads to archive: " + UBOUND(stale_leads)

    FOR EACH lead IN stale_leads
        UPDATE "leads", "id='" + lead.id + "'", {
            "status": "archived",
            "archived_at": NOW(),
            "archive_reason": "No engagement after 90 days"
        }

        APPEND results.actions, {"type": "archive", "lead": lead.name}
    NEXT

    results.leads_nurtured = UBOUND(cold_leads)
    results.leads_archived = UBOUND(stale_leads)
    PRINT "Leads nurtured: " + results.leads_nurtured + ", Archived: " + results.leads_archived
END IF

' =====================================================================
' PIPELINE REVIEW - Weekly pipeline analysis
' =====================================================================
IF job_name = "pipeline-review" OR job_name = "all" THEN
    PRINT ""
    PRINT "--- Running Pipeline Review ---"

    ' Get all active opportunities
    opportunities = FIND "opportunities", "stage NOT IN ('closed_won', 'closed_lost')"
    PRINT "Active opportunities: " + UBOUND(opportunities)

    ' Calculate pipeline metrics
    total_value = 0
    weighted_value = 0
    stale_count = 0
    at_risk_count = 0

    FOR EACH opp IN opportunities
        total_value = total_value + opp.amount
        weighted_value = weighted_value + (opp.amount * opp.probability / 100)

        ' Check for stale opportunities
        days_since_activity = DATEDIFF(NOW(), opp.last_activity)
        IF days_since_activity > 14 THEN
            stale_count = stale_count + 1

            ' Alert owner
            owner = FIND "attendants", "id='" + opp.owner_id + "'"
            IF owner IS NOT NULL AND owner.email IS NOT NULL THEN
                SEND MAIL owner.email, "⚠️ Oportunidade Estagnada: " + opp.name, "A oportunidade '" + opp.name + "' está sem atividade há " + days_since_activity + " dias.\n\nValor: R$ " + opp.amount + "\nEstágio: " + opp.stage + "\n\nPor favor, atualize o status ou registre uma atividade."
            END IF

            ' Create task
            CREATE TASK "Atualizar oportunidade: " + opp.name, owner.email, NOW()

            APPEND results.actions, {"type": "stale_alert", "opportunity": opp.name, "days": days_since_activity}
        END IF

        ' Check for at-risk (past close date)
        IF opp.close_date < NOW() THEN
            at_risk_count = at_risk_count + 1

            UPDATE "opportunities", "id='" + opp.id + "'", {
                "at_risk": TRUE,
                "risk_reason": "Past expected close date"
            }
        END IF
    NEXT

    ' Generate weekly report
    report = "📊 PIPELINE SEMANAL\n"
    report = report + "════════════════════════════════════════\n\n"
    report = report + "Total Pipeline: R$ " + FORMAT(total_value, "#,##0.00") + "\n"
    report = report + "Valor Ponderado: R$ " + FORMAT(weighted_value, "#,##0.00") + "\n"
    report = report + "Oportunidades Ativas: " + UBOUND(opportunities) + "\n"
    report = report + "Estagnadas (14+ dias): " + stale_count + "\n"
    report = report + "Em Risco: " + at_risk_count + "\n\n"

    ' Top 5 opportunities
    top_opps = FIND "opportunities", "stage NOT IN ('closed_won', 'closed_lost') ORDER BY amount DESC LIMIT 5"
    report = report + "TOP 5 OPORTUNIDADES:\n"
    FOR EACH opp IN top_opps
        report = report + "• " + opp.name + " - R$ " + FORMAT(opp.amount, "#,##0.00") + " (" + opp.probability + "%)\n"
    NEXT

    ' Send to sales leadership
    SEND MAIL "vendas@company.com", "Pipeline Semanal - " + FORMAT(NOW(), "dd/MM/yyyy"), report

    results.pipeline_total = total_value
    results.pipeline_weighted = weighted_value
    results.stale_opportunities = stale_count
    results.at_risk = at_risk_count
    PRINT "Pipeline review completed. Total: R$ " + total_value
END IF

' =====================================================================
' FINISH
' =====================================================================
results.completed_at = NOW()
results.duration_seconds = DATEDIFF("second", results.started_at, results.completed_at)

PRINT ""
PRINT "=== CRM Automations Completed ==="
PRINT "Duration: " + results.duration_seconds + " seconds"
PRINT "Actions taken: " + UBOUND(results.actions)

' Log results
SAVE "automation_logs", results

RETURN results
