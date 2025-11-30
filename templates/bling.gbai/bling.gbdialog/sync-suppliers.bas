REM Geral
REM Produto Fornecedor

FUNCTION SyncProdutoFornecedor(idProduto)
    REM Sincroniza ProdutoFornecedor.
    DELETE "maria.ProdutoFornecedor", "Produto_id=" + idProduto
    
    i1 = 1
    DO WHILE i1 > 0 AND i1 < pages
        res = GET host + "/produtos/fornecedores?pagina=${i1}&limite=${limit}&idProduto=${idProduto}"
        list1 = res.data
        res = null
        WAIT 0.33
        
        REM Sincroniza itens
        let j1 = 0
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
        items1= null
        i1 = i1 + 1
        
        IF list1?.length < limit THEN
            i1 = 0
        END IF
        res=null
        list1=null
    LOOP
END FUNCTION

i = 1
SEND EMAIL admin, "Sincronismo Fornecedores iniciado..."

fullList = FIND "maria.Produtos"

REM Initialize chunk parameters
chunkSize = 100
startIndex = 0

REM ubound(fullList)
DO WHILE startIndex < ubound(fullList)
    list = mid( fullList, startIndex, chunkSize)
    
    REM Sincroniza itens de Produto
    prd1 = ""
    j = 0
    items = NEW ARRAY
    
    DO WHILE j < ubound(list)
        produto_id = list[j].id
        prd1 = prd1 + "&idsProdutos%5B%5D=" + produto_id
        CALL SyncProdutoFornecedor(produto_id)
        j = j +1
    LOOP
    
    list = null
    
    REM Update startIndex for the next chunk
    startIndex = startIndex + chunkSize
    items = null
LOOP

fullList = null
SEND EMAIL admin, "Fornecedores concluído."
