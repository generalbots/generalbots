ALLOW ROLE "analiseDados"

BEGIN TALK
Exemplos de perguntas para o *BlingBot*:

1. Quais são os produtos que têm estoque excessivo em uma loja e podem ser transferidos para outra loja com menor estoque?

2. Quais são os 10 produtos mais vendidos na loja {nome_loja} no período {periodo}?

3. Qual é o ticket médio da loja {nome_loja}?

4. Qual a quantidade disponível do produto {nome_produto} na loja {nome_loja}?

5. Quais produtos precisam ser transferidos da loja {origem} para a loja {destino}?

6. Quais produtos estão com estoque crítico na loja {nome_loja}?

7. Qual a sugestão de compra para o fornecedor {nome_fornecedor}?

8. Quantos pedidos são realizados por dia na loja {nome_loja}?

9. Quantos produtos ativos existem no sistema?

10. Qual o estoque disponível na loja {nome_loja}?
END TALK

REM SET SCHEDULE

SET CONTEXT "As lojas B, L e R estão identificadas no final dos nomes das colunas da tabela de Análise de Compras. Dicionário de dados AnaliseCompras.qtEstoqueL: Descrição quantidade do Leblon. AnaliseCompras.qtEstoqueB: Descrição quantidade da Barra AnaliseCompras.qtEstoqueR: Descrição quantidade do Rio Sul. Com base no comportamento de compra registrado, analise os dados fornecidos para identificar oportunidades de otimização de estoque. Aplique regras básicas de transferência de produtos entre as lojas, considerando a necessidade de balanceamento de inventário. Retorne um relatório das 10 ações mais críticas, detalhe a movimentação sugerida para cada produto. Deve indicar a loja de origem, a loja de destino e o motivo da transferência. A análise deve ser objetiva e pragmática, focando na melhoria da disponibilidade de produtos nas lojas. Sempre use LIKE %% para comparar nomes. IMPORTANTE: Compare sempre com a função LOWER ao filtrar valores, em ambos os operandos de texto em SQL, para ignorar case, exemplo WHERE LOWER(loja.nome) LIKE LOWER(%Leblon%)."

SET ANSWER MODE "sql"

TALK "Pergunte-me qualquer coisa sobre os seus dados."

REM IF mobile = "5521992223002" THEN
REM ELSE
REM TALK "Não autorizado."
REM END IF
