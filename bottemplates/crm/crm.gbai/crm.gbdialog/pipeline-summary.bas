DESCRIPTION "Exibe resumo do pipeline de vendas: total por estágio, valor, taxa de conversão."

stats = GET "/api/crm/stats"
stages = GET "/api/crm/pipeline"

TALK "📊 **Resumo do Pipeline de Vendas**"
TALK ""
TALK "💰 Valor total no pipeline: R$ " + FORMAT(stats.total_value, "#,##0")
TALK "🏆 Valor ganho: R$ " + FORMAT(stats.won_value, "#,##0")
TALK "📈 Taxa de conversão: " + FORMAT(stats.conversion_rate, "#0.0") + "%"
TALK "📐 Ticket médio: R$ " + FORMAT(stats.avg_deal_size, "#,##0")
TALK ""

IF stats.stages THEN
    TALK "**Por estágio:**"
    FOR EACH sg IN stats.stages
        TALK "  " + sg.name + ": " + sg.count + " deals — R$ " + FORMAT(sg.total_value, "#,##0")
    NEXT sg
END IF

RETURN stats
