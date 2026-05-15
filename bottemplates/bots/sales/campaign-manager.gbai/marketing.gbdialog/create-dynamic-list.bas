PARAM name AS STRING LIKE "VIPs de São Paulo" DESCRIPTION "Nome da lista dinâmica"
PARAM filter AS STRING LIKE "Perfil=VIP AND cidade=São Paulo" DESCRIPTION "Condição de filtro (SQL-like)"
PARAM description AS STRING DESCRIPTION "Descrição da lista (opcional)" OPTIONAL

DESCRIPTION "Cria uma Lista Dinâmica baseada em filtros de contatos do CRM."

IF NOT filter THEN
    TALK "Qual é a condição de filtro para esta lista?"
    TALK "Exemplos:"
    TALK "- Perfil=VIP"
    TALK "- cidade=São Paulo AND compras>5"
    TALK "- ultima_compra>2024-01-01"
    HEAR filter AS STRING
END IF

TALK "🔍 Verificando quantos contatos matching o filtro..."
preview = GET "/api/crm/contacts/count?filter=" + filter

IF preview = 0 THEN
    TALK "⚠️ Nenhum contato encontrada com este filtro!"
    TALK "Deseja ajustar o filtro ou criar a lista mesmo assim?"
    HEAR proceed AS BOOLEAN
    IF NOT proceed THEN
        RETURN
    END IF
ELSE
    TALK "✅ " + preview + " contatos matching o filtro."
    TALK "Prosseguir com a criação da lista dinâmica?"
    HEAR confirm AS BOOLEAN
    IF NOT confirm THEN
        RETURN
    END IF
END IF

new_list = POST "/api/marketing/lists", #{
    name: name,
    filter: filter,
    description: description,
    type: "dynamic"
}

TALK "📋 **Lista Dinâmica Criada!**"
TALK "Nome: " + name
TALK "Filtro: " + filter
TALK "Contatos: " + preview + " (atualizado automaticamente)"
TALK "Tipo: Dinâmica (atualiza automaticamente)"
TALK "ID: " + new_list.id

TALK "Use esta lista em campanhas - ela será atualizada automaticamente!"
