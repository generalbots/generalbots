PARAM sku AS STRING LIKE "ABC123" DESCRIPTION "Product SKU code to update stock"
PARAM qtd AS INTEGER LIKE 10 DESCRIPTION "Quantity to add to stock"

DESCRIPTION "Add stock quantity for a product by SKU"

person = FIND "People.xlsx", "id=" + mobile
vendor = FIND "maria.Vendedores", "id=" + person.erpId

TALK "Olá " + vendor.Contato_Nome + "!"

produto = FIND "maria.Produtos", "sku=" + sku

IF NOT produto THEN
    TALK "Produto não encontrado."
    RETURN
END IF

WITH estoque
    produto = { id: produto.Id }
    deposito = { id: person.deposito_Id }
    preco = produto.Preco
    operacao = "B"
    quantidade = qtd
    observacoes = "Acréscimo de estoque."
END WITH

rec = POST host + "/estoques", estoque

TALK "Estoque atualizado."
TALK TO admin1, "Estoque do ${sku} atualizado com ${qtd}."
TALK TO admin2, "Estoque do ${sku} atualizado com ${qtd}."

RETURN rec
