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

# semantic i18n key (#2542)
admin-compliance-dashboard-title = Painel de conformidade

# semantic i18n key (#2542)
admin-compliance-subtitle = Status de conformidade SOC 2 Tipo II, monitoramento de controles e evidências de auditoria

# semantic i18n key (#2542)
admin-compliance-compliant = Em conformidade

# semantic i18n key (#2542)
admin-compliance-overall-score = Pontuação geral de conformidade

# semantic i18n key (#2542)
admin-compliance-controls-passing = Controles aprovados

# semantic i18n key (#2542)
admin-compliance-evidence-items-due = Itens de evidência pendentes

# semantic i18n key (#2542)
admin-compliance-until-next-audit = Até a próxima auditoria

# semantic i18n key (#2542)
admin-compliance-next-audit-date = Agendado: 7 de março de 2025

# semantic i18n key (#2542)
admin-compliance-trust-service-criteria = Critérios de serviços de confiança

# semantic i18n key (#2542)
admin-compliance-availability = Disponibilidade

# semantic i18n key (#2542)
admin-compliance-processing-integrity = Integridade do processamento

# semantic i18n key (#2542)
admin-compliance-confidentiality = Confidencialidade

# semantic i18n key (#2542)
admin-compliance-controls-requiring-attention = Controles que exigem atenção

# semantic i18n key (#2542)
admin-compliance-control-mfa-privileged = Autenticação multifator para contas privilegiadas

# semantic i18n key (#2542)
admin-compliance-due-jan-25-2025 = Vencimento: 25 jan 2025

# semantic i18n key (#2542)
admin-compliance-remediate = Corrigir

# semantic i18n key (#2542)
admin-compliance-control-incident-response-testing = Teste do plano de resposta a incidentes de segurança

# semantic i18n key (#2542)
admin-compliance-issue-annual-test-overdue = Teste anual atrasado há 15 dias

# semantic i18n key (#2542)
admin-compliance-control-dpa-documentation = Documentação de acordo de processamento de dados

# semantic i18n key (#2542)
admin-compliance-due-feb-1-2025 = Vencimento: 1 fev 2025

# semantic i18n key (#2542)
admin-compliance-control-data-classification-review = Revisão da política de classificação de dados

# semantic i18n key (#2542)
admin-compliance-issue-policy-review-pending = Revisão da política aguardando aprovação

# semantic i18n key (#2542)
admin-compliance-due-feb-15-2025 = Vencimento: 15 fev 2025

# semantic i18n key (#2542)
admin-compliance-control-privacy-notice-updates = Atualizações do aviso de privacidade

# semantic i18n key (#2542)
admin-compliance-issue-privacy-policy-update = A política de privacidade precisa ser atualizada para nova coleta de dados

# semantic i18n key (#2542)
admin-compliance-due-feb-28-2025 = Vencimento: 28 fev 2025

# semantic i18n key (#2542)
admin-compliance-evidence-collection = Coleta de evidências

# semantic i18n key (#2542)
admin-compliance-current = Atual

# semantic i18n key (#2542)
admin-compliance-due-soon = Vence em breve

# semantic i18n key (#2542)
admin-compliance-evidence-pentest-q4 = Relatório de teste de penetração T4 2024

# semantic i18n key (#2542)
admin-compliance-evidence-cc4-vuln = CC4.1.1 - Avaliação de vulnerabilidades

# semantic i18n key (#2542)
admin-compliance-evidence-access-review-jan = Relatório de revisão de acesso - Janeiro de 2025

# semantic i18n key (#2542)
admin-compliance-evidence-cc6-access = CC6.2.1 - Controle de acesso

# semantic i18n key (#2542)
admin-compliance-due-in-3-days = Vence em 3 dias

# semantic i18n key (#2542)
admin-compliance-evidence-backup-restore = Resultados do teste de restauração de backup

# semantic i18n key (#2542)
admin-compliance-evidence-a1-backup = A1.2.3 - Backup e recuperação

# semantic i18n key (#2542)
admin-compliance-due-in-5-days = Vence em 5 dias

# semantic i18n key (#2542)
admin-compliance-evidence-security-training = Registros de treinamento de conscientização em segurança

# semantic i18n key (#2542)
admin-compliance-evidence-cc1-training = CC1.4.1 - Treinamento de segurança

# semantic i18n key (#2542)
admin-compliance-view-all-evidence = Ver todas as evidências

# semantic i18n key (#2542)
admin-compliance-access-events = Eventos de acesso

# semantic i18n key (#2542)
admin-compliance-configuration-changes = Alterações de configuração

# semantic i18n key (#2542)
admin-compliance-security-events = Eventos de segurança

# semantic i18n key (#2542)
admin-compliance-compliance-events = Eventos de conformidade

# semantic i18n key (#2542)
admin-compliance-failed-login-attempt = Tentativa de login falha

# semantic i18n key (#2542)
admin-compliance-failed-login-details = Usuário: admin@company.com | IP: 192.168.1.105 | Motivo: Senha inválida

# semantic i18n key (#2542)
admin-compliance-user-logged-in = Usuário conectado

# semantic i18n key (#2542)
admin-compliance-user-logged-in-details = Usuário: sarah.johnson@company.com | IP: 10.0.0.45
# semantic i18n key (#2542) — compliance app shell
compliance-run-check = Executar verificação
compliance-stat-total-checks = Total de verificações
compliance-stat-passed = Aprovadas
compliance-stat-failed = Reprovadas
compliance-stat-warnings = Avisos
compliance-stat-score = Pontuação
compliance-tab-dashboard = Painel
compliance-tab-checks = Verificações
compliance-tab-issues = Problemas
compliance-tab-audit-log = Registro de auditoria
compliance-tab-risk-matrix = Matriz de riscos
compliance-tab-training = Treinamento
compliance-recent-activity = Atividade recente
compliance-score-heading = Pontuação de conformidade
compliance-check-results-heading = Resultados das verificações
compliance-checks-heading = Verificações de conformidade
compliance-issues-heading = Problemas
compliance-audit-log-heading = Registro de auditoria
compliance-risk-matrix-title = Matriz de riscos (Probabilidade x Impacto)
compliance-impact-axis = Impacto →
compliance-likelihood-axis = Probabilidade
compliance-training-records = Registros de treinamento
compliance-loading = Carregando...
compliance-no-recent-activity = Nenhuma atividade recente
compliance-no-checks-found = Nenhuma verificação encontrada
compliance-no-issues-found = Nenhum problema encontrado
compliance-no-audit-entries = Nenhuma entrada de auditoria
compliance-no-risks-recorded = Nenhum risco registrado
compliance-no-training-records = Nenhum registro de treinamento
compliance-overall-compliance-score = Pontuação geral de conformidade
compliance-col-id = ID
compliance-col-framework = Estrutura
compliance-col-status = Status
compliance-col-created = Criado em
compliance-col-timestamp = Data/Hora
compliance-col-user = Usuário
compliance-action-run = Executar
compliance-create-issue = Criar problema
compliance-new-training = Novo treinamento
compliance-framework-details = Detalhes da estrutura
compliance-failed-load-framework = Falha ao carregar a estrutura
compliance-risk-score = Pontuação: { $score }
compliance-issue-title-placeholder = Título do problema
compliance-issue-assignee-placeholder = Nome do responsável
compliance-issue-description-placeholder = Descreva o problema...
compliance-training-title-placeholder = Título do treinamento
compliance-training-assignee-placeholder = Nome do responsável
compliance-due-date = Data de vencimento
compliance-due-label = Vencimento:
compliance-col-assignee = Responsável
compliance-create-training = Criar treinamento
