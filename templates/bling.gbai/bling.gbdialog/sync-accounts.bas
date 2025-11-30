REM Executa a cada dois dias, 23h.
SET SCHEDULE "0 0 0 */2 * *"

REM Variables from config.csv: admin1, admin2, host, limit, pages
REM Using admin1 for notifications
admin = admin1

REM Pagination settings for Bling API
pageVariable = "pagina"
limitVariable = "limite"
syncLimit = 100

REM ============================================
REM Sync Contas a Receber (Accounts Receivable)
REM ============================================
SEND EMAIL admin, "Sincronizando Contas a Receber..."

page = 1
totalReceber = 0

DO WHILE page > 0 AND page <= pages
    url = host + "/contas/receber?" + pageVariable + "=" + page + "&" + limitVariable + "=" + syncLimit
    res = GET url
    WAIT 0.33

    IF res.data THEN
        items = res.data
        itemCount = UBOUND(items)

        IF itemCount > 0 THEN
            MERGE "maria.ContasAReceber" WITH items BY "Id"
            totalReceber = totalReceber + itemCount
            page = page + 1

            IF itemCount < syncLimit THEN
                page = 0
            END IF
        ELSE
            page = 0
        END IF
    ELSE
        page = 0
    END IF

    res = null
    items = null
LOOP

SEND EMAIL admin, "Contas a Receber sincronizadas: " + totalReceber + " registros."

REM ============================================
REM Sync Contas a Pagar (Accounts Payable)
REM ============================================
SEND EMAIL admin, "Sincronizando Contas a Pagar..."

page = 1
totalPagar = 0

DO WHILE page > 0 AND page <= pages
    url = host + "/contas/pagar?" + pageVariable + "=" + page + "&" + limitVariable + "=" + syncLimit
    res = GET url
    WAIT 0.33

    IF res.data THEN
        items = res.data
        itemCount = UBOUND(items)

        IF itemCount > 0 THEN
            MERGE "maria.ContasAPagar" WITH items BY "Id"
            totalPagar = totalPagar + itemCount
            page = page + 1

            IF itemCount < syncLimit THEN
                page = 0
            END IF
        ELSE
            page = 0
        END IF
    ELSE
        page = 0
    END IF

    res = null
    items = null
LOOP

SEND EMAIL admin, "Contas a Pagar sincronizadas: " + totalPagar + " registros."

REM ============================================
REM Summary
REM ============================================
SEND EMAIL admin, "Transferência do ERP (Contas) para BlingBot concluído. Total: " + (totalReceber + totalPagar) + " registros."
