SET SCHEDULE "0 0 22 * * *"

DESCRIPTION "Sync product suppliers from Bling ERP to local database"

SEND EMAIL admin, "Suppliers sync started..."

FUNCTION SyncProdutoFornecedor(idProduto)
    DELETE "maria.ProdutoFornecedor", "Produto_id=" + idProduto

    i1 = 1
    DO WHILE i1 > 0 AND i1 < pages
        res = GET host + "/produtos/fornecedores?pagina=${i1}&limite=${limit}&idProduto=${idProduto}"
        list1 = res.data
        res = null
        WAIT 0.33

        j1 = 0
        items1 = NEW ARRAY

        DO WHILE j1 < ubound(list1)
            produtoFornecedor_id = list1[j1].id
            res = GET host + "/produtos/fornecedores/${produtoFornecedor_id}"
            items1[j1] = res.data
            res = null
            WAIT 0.33
            j1 = j1 + 1
        LOOP

        SAVE "maria.ProdutoFornecedor", items1
        items1 = null
        i1 = i1 + 1

        IF list1?.length < limit THEN
            i1 = 0
        END IF
        res = null
        list1 = null
    LOOP
END FUNCTION

fullList = FIND "maria.Produtos"

chunkSize = 100
startIndex = 0

DO WHILE startIndex < ubound(fullList)
    list = mid(fullList, startIndex, chunkSize)

    j = 0

    DO WHILE j < ubound(list)
        produto_id = list[j].id
        CALL SyncProdutoFornecedor(produto_id)
        j = j + 1
    LOOP

    list = null
    startIndex = startIndex + chunkSize
LOOP

fullList = null
SEND EMAIL admin, "Suppliers sync completed."
