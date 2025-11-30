person = FIND "People.xlsx", "id=" + mobile
vendor = FIND "maria.Vendedores", "id=" + person.erpId

TALK "Olá " + vendor.Contato_Nome + "!"

REM Estoque pelo nome em caso de não presente na planilha
TALK "Qual o SKU do Produto?"
HEAR sku

produto = FIND "maria.Produtos", "sku=" + sku

TALK "Qual a quantidade que se deseja acrescentar?"
HEAR qtd

estoque = {
    produto: {
        id: produto.Id
    },
    deposito: {
        id: person.deposito_Id
    },
    preco: produto.Preco,
    operacao: "B",
    quantidade: qtd,
    observacoes: "Acréscimo de estoque."
}

rec = POST host + "/estoques", estoque

TALK "Estoque atualizado, obrigado."
TALK TO admin1, "Estoque do ${sku} foi atualizado com ${qtd}."
TALK TO admin2, "Estoque do ${sku} foi atualizado com ${qtd}."
