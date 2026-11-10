# =============================================================================
# General Bots — pt-BR — Compliance
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# TELAS DE FERRAMENTAS / CONFORMIDADE
# =============================================================================
# -----------------------------------------------------------------------------
# Conformidade Principal
# -----------------------------------------------------------------------------
compliance-title = Relatório de Conformidade da API
compliance-subtitle = Verificação de segurança para todos os bots - Verifica senhas, código frágil e configurações incorretas
compliance-export-report = Exportar Relatório
compliance-run-scan = Executar Verificação de Conformidade
compliance-scanning = Verificando...

# -----------------------------------------------------------------------------
# Seletor de Bot
# -----------------------------------------------------------------------------
compliance-all-bots = Todos os Bots
compliance-select-bots = Selecionar Bots

# -----------------------------------------------------------------------------
# Cards de Estatísticas
# -----------------------------------------------------------------------------
compliance-critical = Crítico
compliance-critical-desc = Requer ação imediata
compliance-high = Alto
compliance-high-desc = Risco de segurança
compliance-medium = Médio
compliance-medium-desc = Deve ser corrigido
compliance-low = Baixo
compliance-low-desc = Boa prática
compliance-info = Info
compliance-info-desc = Informativo

# -----------------------------------------------------------------------------
# Filtros
# -----------------------------------------------------------------------------
compliance-filter-severity = Severidade
compliance-filter-type = Tipo
compliance-filter-all-severities = Todas as Severidades
compliance-filter-all-types = Todos os Tipos
compliance-search-issues = Buscar problemas...

# -----------------------------------------------------------------------------
# Tipos de Problemas
# -----------------------------------------------------------------------------
compliance-type-password = Senha na Configuração
compliance-type-hardcoded = Segredos Hardcoded
compliance-type-deprecated = Palavras-chave Obsoletas
compliance-type-fragile = Código Frágil
compliance-type-config = Problemas de Configuração

# -----------------------------------------------------------------------------
# Tabela de Resultados
# -----------------------------------------------------------------------------
compliance-results = Resultados
compliance-results-count = { $count ->
    [one] { $count } problema encontrado
   *[other] { $count } problemas encontrados
}
compliance-col-severity = Severidade
compliance-col-issue = Problema
compliance-col-location = Localização
compliance-col-details = Detalhes
compliance-col-action = Ação
compliance-view-details = Ver Detalhes
compliance-fix-issue = Corrigir Problema
compliance-ignore = Ignorar
compliance-no-issues = Nenhum problema encontrado
compliance-no-issues-desc = Ótimo! Seus bots estão em conformidade.

# -----------------------------------------------------------------------------
# Progresso da Verificação
# -----------------------------------------------------------------------------
compliance-scan-in-progress = Verificação em andamento...
compliance-scan-checking = Verificando { $item }...
compliance-scan-complete = Verificação concluída
compliance-scan-failed = Verificação falhou

## Estruturas de compliance (release enterprise)
compliance-frameworks = Estruturas
compliance-new-framework = Novo
compliance-framework-name = Nome
compliance-framework-version = Versão
compliance-framework-description = Descrição
compliance-create-framework = Criar Estrutura
compliance-controls = Controles
compliance-add-control = Adicionar Controle
compliance-control-id = ID do Controle
compliance-control-title = Título
compliance-control-category = Categoria
compliance-control-description = Descrição
compliance-mandatory = Obrigatório
compliance-optional = Opcional
compliance-evidence = Evidências
compliance-attach-evidence = Anexar Evidência
compliance-evidence-path = Caminho do arquivo (artefato no drive)
compliance-evidence-type = Tipo
compliance-approve = Aprovar
compliance-covered = Coberto
compliance-no-evidence = Sem evidência
compliance-export-csv = Exportar CSV
compliance-archive = Arquivar
compliance-total-controls = Total de Controles
compliance-coverage = Cobertura
compliance-no-frameworks = Nenhuma estrutura configurada ainda.
