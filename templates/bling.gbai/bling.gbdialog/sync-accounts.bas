SET SCHEDULE "0 0 0 */2 * *"

pageVariable = "pagina"
limitVariable = "limite"
syncLimit = 100

' Contas a Receber
SEND EMAIL admin, "Syncing Accounts Receivable..."

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

SEND EMAIL admin, "Accounts Receivable: " + totalReceber + " records."

' Contas a Pagar
SEND EMAIL admin, "Syncing Accounts Payable..."

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

SEND EMAIL admin, "Accounts Payable: " + totalPagar + " records."
SEND EMAIL admin, "Accounts sync completed. Total: " + (totalReceber + totalPagar) + " records."
