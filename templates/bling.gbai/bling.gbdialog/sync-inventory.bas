SET SCHEDULE "0 30 23 * * *"

SEND EMAIL admin, "Inventory sync started..."

fullList = FIND "maria.Produtos"

chunkSize = 100
startIndex = 0

DO WHILE startIndex < ubound(fullList)
    list = mid(fullList, startIndex, chunkSize)
    prd1 = ""
    j = 0
    items = NEW ARRAY

    DO WHILE j < ubound(list)
        produto_id = list[j].id
        prd1 = prd1 + "&idsProdutos%5B%5D=" + produto_id
        j = j + 1
    LOOP

    list = null

    IF j > 0 THEN
        res = GET host + "/estoques/saldos?${prd1}"
        WAIT 0.33
        items = res.data
        res = null

        k = 0
        DO WHILE k < ubound(items)
            depositos = items[k].depositos
            pSku = FIND "maria.Produtos", "id=${items[k].produto.id}"

            IF pSku THEN
                prdSku = pSku.sku
                DELETE "maria.Depositos", "Sku=" + prdSku

                l = 0
                DO WHILE l < ubound(depositos)
                    depositos[l].sku = prdSku
                    l = l + 1
                LOOP

                SAVE "maria.Depositos", depositos
                depositos = null
            END IF

            pSku = null
            k = k + 1
        LOOP
        items = null
    END IF

    startIndex = startIndex + chunkSize
    items = null
LOOP

fullList = null
SEND EMAIL admin, "Inventory sync completed."
