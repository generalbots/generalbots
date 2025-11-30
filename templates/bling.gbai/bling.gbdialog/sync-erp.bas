SET SCHEDULE "0 30 22 * * *"

daysToSync = -7
ontem = DATEADD today, "days", daysToSync
ontem = FORMAT ontem, "yyyy-MM-dd"
tomorrow = DATEADD today, "days", 1
tomorrow = FORMAT tomorrow, "yyyy-MM-dd"
dateFilter = "&dataAlteracaoInicial=${ontem}&dataAlteracaoFinal=${tomorrow}"

SEND EMAIL admin, "Sync: ${ontem} to ${tomorrow} started..."

' Produtos
i = 1
SEND EMAIL admin, "Syncing Products..."

DO WHILE i > 0 AND i < pages
    res = GET host + "/produtos?pagina=${i}&criterio=5&tipo=P&limite=${limit}${dateFilter}"
    WAIT 0.33
    list = res.data
    res = null

    prd1 = ""
    j = 0
    k = 0
    items = NEW ARRAY

    DO WHILE j < ubound(list)
        produto_id = list[j].id
        res = GET host + "/produtos/${produto_id}"
        WAIT 0.33
        produto = res.data
        res = null

        IF produto.codigo && produto.codigo.trim().length THEN
            prd1 = prd1 + "&idsProdutos%5B%5D=" + list[j].id
            items[k] = produto
            produto.sku = items[k].codigo

            IF produto.variacoes.length > 0 THEN
                produto.hierarquia = "p"
            ELSE
                produto.hierarquia = "s"
            END IF

            produtoDB = FIND "maria.Produtos", "sku=" + produto.codigo
            IF produtoDB THEN
                IF produtoDB.preco <> produto.preco THEN
                    hist = NEW OBJECT
                    hist.sku = produto.sku
                    hist.precoAntigo = produtoDB.preco
                    hist.precoAtual = produto.preco
                    hist.produto_id = produto.id
                    hist.dataModificado = FORMAT today, "yyyy-MM-dd"
                    SAVE "maria.HistoricoPreco", hist
                    hist = null
                END IF
            END IF
            k = k + 1
        END IF
        j = j + 1
    LOOP

    list = null
    list = items

    MERGE "maria.Produtos" WITH list BY "Id"
    list = items

    j = 0
    DO WHILE j < ubound(list)
        listV = list[j].variacoes
        IF listV THEN
            k = 0
            prd2 = ""
            DO WHILE k < ubound(listV)
                IF listV[k].codigo && listV[k].codigo.trim().length THEN
                    listV[k].skuPai = list[j].sku
                    listV[k].sku = listV[k].codigo
                    listV[k].hierarquia = "f"
                    k = k + 1
                ELSE
                    listV.splice(k, 1)
                END IF
            LOOP

            k = 0
            DO WHILE k < ubound(listV)
                listV[k].hierarquia = 'f'
                DELETE "maria.ProdutoImagem", "sku=" + listV[k].sku

                images = listV[k]?.midia?.imagens?.externas
                l = 0
                DO WHILE l < ubound(images)
                    images[l].ordinal = k
                    images[l].sku = listV[k].sku
                    images[l].id = random()
                    l = l + 1
                LOOP
                SAVE "maria.ProdutoImagem", images
                images = null
                k = k + 1
            LOOP

            MERGE "maria.Produtos" WITH listV BY "Id"
        END IF
        listV = null

        DELETE "maria.ProdutoImagem", "sku=" + list[j].sku
        k = 0
        images = list[j].midia?.imagens?.externas
        DO WHILE k < ubound(images)
            images[k].ordinal = k
            images[k].sku = list[j].sku
            images[k].id = random()
            k = k + 1
        LOOP
        SAVE "maria.ProdutoImagem", images
        j = j + 1
    LOOP

    i = i + 1
    IF list?.length < limit THEN
        i = 0
    END IF
    list = null
    res = null
    items = null
LOOP

SEND EMAIL admin, "Products completed."
RESET REPORT

' Pedidos
SEND EMAIL admin, "Syncing Orders..."
i = 1

DO WHILE i > 0 AND i < pages
    res = GET host + "/pedidos/vendas?pagina=${i}&limite=${limit}${dateFilter}"
    list = res.data
    res = null

    j = 0
    fullList = []

    DO WHILE j < ubound(list)
        pedido_id = list[j].id
        res = GET host + "/pedidos/vendas/${pedido_id}"
        items = res.data.itens

        k = 0
        DO WHILE k < ubound(items)
            items[k].pedido_id = pedido_id
            items[k].sku = items[k].codigo
            items[k].numero = list[j].numero
            items[k].custo = items[k].valor / 2
            k = k + 1
        LOOP
        MERGE "maria.PedidosItem" WITH items BY "Id"

        items = res.data.parcelas
        k = 0
        DO WHILE k < ubound(items)
            items[k].pedido_id = pedido_id
            k = k + 1
        LOOP
        MERGE "maria.Parcela" WITH items BY "Id"

        fullList[j] = res.data
        res = null
        j = j + 1
    LOOP

    MERGE "maria.Pedidos" WITH fullList BY "Id"
    i = i + 1
    IF list?.length < limit THEN
        i = 0
    END IF
    list = null
    res = null
LOOP

SEND EMAIL admin, "Orders completed."

' Common entities
pageVariable = "pagina"
limitVariable = "limite"
syncLimit = 100

' CategoriaReceita
SEND EMAIL admin, "Syncing CategoriaReceita..."
syncPage = 1
totalCategoria = 0

DO WHILE syncPage > 0 AND syncPage <= pages
    syncUrl = host + "/categorias/receitas-despesas?" + pageVariable + "=" + syncPage + "&" + limitVariable + "=" + syncLimit
    syncRes = GET syncUrl
    WAIT 0.33

    IF syncRes.data THEN
        syncItems = syncRes.data
        syncCount = UBOUND(syncItems)

        IF syncCount > 0 THEN
            MERGE "maria.CategoriaReceita" WITH syncItems BY "Id"
            totalCategoria = totalCategoria + syncCount
            syncPage = syncPage + 1

            IF syncCount < syncLimit THEN
                syncPage = 0
            END IF
        ELSE
            syncPage = 0
        END IF
    ELSE
        syncPage = 0
    END IF

    syncRes = null
    syncItems = null
LOOP

SEND EMAIL admin, "CategoriaReceita: " + totalCategoria + " records."

' FormaDePagamento
SEND EMAIL admin, "Syncing Payment Methods..."
syncPage = 1
totalForma = 0

DO WHILE syncPage > 0 AND syncPage <= pages
    syncUrl = host + "/formas-pagamentos?" + pageVariable + "=" + syncPage + "&" + limitVariable + "=" + syncLimit
    syncRes = GET syncUrl
    WAIT 0.33

    IF syncRes.data THEN
        syncItems = syncRes.data
        syncCount = UBOUND(syncItems)

        IF syncCount > 0 THEN
            MERGE "maria.FormaDePagamento" WITH syncItems BY "Id"
            totalForma = totalForma + syncCount
            syncPage = syncPage + 1

            IF syncCount < syncLimit THEN
                syncPage = 0
            END IF
        ELSE
            syncPage = 0
        END IF
    ELSE
        syncPage = 0
    END IF

    syncRes = null
    syncItems = null
LOOP

SEND EMAIL admin, "Payment Methods: " + totalForma + " records."

' Contatos
SEND EMAIL admin, "Syncing Contacts..."
i = 1

DO WHILE i > 0 AND i < pages
    res = GET host + "/contatos?pagina=${i}&limite=${limit}${dateFilter}"
    list = res.data

    j = 0
    items = NEW ARRAY

    DO WHILE j < ubound(list)
        contato_id = list[j].id
        res = GET host + "/contatos/${contato_id}"
        items[j] = res.data
        WAIT 0.33
        j = j + 1
    LOOP

    MERGE "maria.Contatos" WITH items BY "Id"
    i = i + 1
    IF list?.length < limit THEN
        i = 0
    END IF
    list = null
    res = null
LOOP

SEND EMAIL admin, "Contacts completed."

' Vendedores
SEND EMAIL admin, "Syncing Sellers..."
i = 1

DO WHILE i > 0 AND i < pages
    res = GET host + "/vendedores?pagina=${i}&situacaoContato=T&limite=${limit}${dateFilter}"
    list = res.data

    j = 0
    items = NEW ARRAY

    DO WHILE j < ubound(list)
        vendedor_id = list[j].id
        res = GET host + "/vendedores/${vendedor_id}"
        items[j] = res.data
        WAIT 0.33
        j = j + 1
    LOOP

    MERGE "maria.Vendedores" WITH items BY "Id"
    i = i + 1
    IF list?.length < limit THEN
        i = 0
    END IF
    list = null
    res = null
LOOP

SEND EMAIL admin, "Sellers completed."
SEND EMAIL admin, "ERP sync completed."
