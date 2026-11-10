ADD TOOL "add-stock"
ADD TOOL "sync-erp"
ADD TOOL "sync-inventory"
ADD TOOL "sync-accounts"
ADD TOOL "sync-suppliers"
ADD TOOL "data-analysis"
ADD TOOL "refresh-llm"

CLEAR SUGGESTIONS

ADD SUGGESTION "estoque" AS "bot-suggest-check-stock"
ADD SUGGESTION "pedido" AS "bot-suggest-place-an-order"
ADD SUGGESTION "sync" AS "bot-suggest-sync-erp"
ADD SUGGESTION "analise" AS "bot-suggest-data-analysis"

BEGIN TALK
**BlingBot** - Assistente ERP

Olá! Posso ajudar com:
• 📦 Consulta de estoque
• 🛒 Pedidos e vendas
• 🔄 Sincronização com Bling
• 📊 Análise de dados

Qual o seu pedido?
END TALK

BEGIN SYSTEM PROMPT
Você é um assistente de loja integrado ao Bling ERP.

Ao receber pedido, ofereça opções de cor e tamanho do JSON de produtos.
Retorne JSON do pedido com itens e nome do cliente.
Mantenha itensPedido com apenas um item por vez.
Use o mesmo id do JSON de produtos para correlação.
ItensAcompanhamento contém itens adicionais do pedido (ex: Quadro com Caixa de Giz).
END SYSTEM PROMPT
