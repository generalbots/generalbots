# =============================================================================
# General Bots — pt-BR — Admin, monitoring and system apps
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Painel Admin (chaves UI adicionais)
# -----------------------------------------------------------------------------
admin-panel-title = Painel Admin

# =============================================================================
# TELAS DE MONITORAMENTO
# =============================================================================
# -----------------------------------------------------------------------------
# Painel de Monitoramento
# -----------------------------------------------------------------------------
monitoring-title = Painel de Monitoramento
monitoring-toggle-view = Alternar Visualização
monitoring-last-updated = Última Atualização
monitoring-live-view = Visualização ao Vivo
monitoring-grid-view = Visualização em Grade

# -----------------------------------------------------------------------------
# Painéis de Monitoramento
# -----------------------------------------------------------------------------
monitoring-sessions = Sessões
monitoring-messages = Mensagens
monitoring-resources = Recursos
monitoring-services = Serviços
monitoring-active-bots = Bots Ativos
monitoring-loading = Carregando...

# -----------------------------------------------------------------------------
# Status do Serviço
# -----------------------------------------------------------------------------
monitoring-status-running = Em Execução
monitoring-status-warning = Alerta
monitoring-status-stopped = Parado
monitoring-status-healthy = Saudável
monitoring-status-degraded = Degradado
monitoring-status-down = Fora do Ar

# -----------------------------------------------------------------------------
# Métricas de Recursos
# -----------------------------------------------------------------------------
monitoring-cpu = CPU
monitoring-memory = Memória
monitoring-disk = Disco
monitoring-network = Rede
monitoring-requests-per-sec = Requisições/seg
monitoring-active-connections = Conexões Ativas
monitoring-uptime = Tempo Ativo

# -----------------------------------------------------------------------------
# Logs
# -----------------------------------------------------------------------------
monitoring-logs-title = Logs do Sistema
monitoring-logs-filter = Filtrar Logs
monitoring-logs-level = Nível de Log
monitoring-logs-all = Todos os Níveis
monitoring-logs-debug = Debug
monitoring-logs-info = Info
monitoring-logs-warning = Aviso
monitoring-logs-error = Erro
monitoring-logs-critical = Crítico
monitoring-logs-search = Buscar logs...
monitoring-logs-no-results = Nenhum log encontrado

# -----------------------------------------------------------------------------
# Saúde
# -----------------------------------------------------------------------------
monitoring-health-title = Saúde do Sistema
monitoring-health-status = Status de Saúde
monitoring-health-services = Saúde dos Serviços
monitoring-health-database = Banco de Dados
monitoring-health-cache = Cache
monitoring-health-queue = Fila de Mensagens
monitoring-health-storage = Armazenamento
monitoring-health-external = Serviços Externos

# -----------------------------------------------------------------------------
# Métricas
# -----------------------------------------------------------------------------
monitoring-metrics-title = Métricas de Desempenho
monitoring-metrics-response-time = Tempo de Resposta
monitoring-metrics-throughput = Taxa de Transferência
monitoring-metrics-error-rate = Taxa de Erros
monitoring-metrics-latency = Latência

# -----------------------------------------------------------------------------
# Alertas
# -----------------------------------------------------------------------------
monitoring-alerts-title = Alertas do Sistema
monitoring-alerts-active = Alertas Ativos
monitoring-alerts-resolved = Resolvidos
monitoring-alerts-all = Todos os Alertas
monitoring-alert-severity = Severidade
monitoring-alert-critical = Crítico
monitoring-alert-high = Alto
monitoring-alert-medium = Médio
monitoring-alert-low = Baixo
monitoring-alert-info = Info
monitoring-alert-acknowledge = Reconhecer
monitoring-alert-resolve = Resolver
monitoring-no-alerts = Nenhum alerta ativo

# -----------------------------------------------------------------------------
# Security Module
# -----------------------------------------------------------------------------
security-tab-compliance = Relatório de Conformidade API
security-tab-protection = Proteção
security-export-report = Exportar Relatório
security-run-scan = Executar Verificação
security-critical = Crítico
security-critical-desc = Ação imediata necessária
security-high = Alto
security-high-desc = Risco de segurança
security-medium = Médio
security-medium-desc = Deve ser resolvido
security-low = Baixo
security-low-desc = Melhores práticas
security-info = Info
security-info-desc = Informativo
security-filter-severity = Severidade:
security-filter-all-severities = Todas as Severidades
security-filter-type = Tipo:
security-filter-all-types = Todos os Tipos
security-type-password = Senha em Config
security-type-hardcoded = Segredos Hardcoded
security-type-deprecated = Palavras-Chave Obsoletas
security-type-fragile = Código Frágil
security-type-config = Problemas de Configuração
security-results = Problemas de Conformidade
security-col-severity = Severidade
security-col-issue = Tipo de Problema
security-col-location = Localização
security-col-details = Descrição
security-col-action = Ação

# -----------------------------------------------------------------------------
# Learn Module
# -----------------------------------------------------------------------------
learn-title = Aprender
learn-my-progress = Meu Progresso
learn-completed = Concluídos
learn-in-progress = Em Progresso
learn-certificates = Certificados
learn-time-spent = Tempo Investido
learn-categories = Categorias
learn-all-courses = Todos os Cursos
learn-mandatory = Obrigatório
learn-compliance = Conformidade
learn-security = Segurança
learn-skills = Habilidades
learn-onboarding = Integração
learn-difficulty = Dificuldade
learn-my-certificates = Meus Certificados
learn-view-all = Ver Tudo

# -----------------------------------------------------------------------------
# Workspace Module
# -----------------------------------------------------------------------------
workspace-title = Espaço de Trabalho
workspace-search-pages = Buscar páginas...
workspace-recent = Recentes
workspace-favorites = Favoritos
workspace-pages = Páginas
workspace-templates = Modelos
workspace-trash = Lixeira
workspace-settings = Configurações

# -----------------------------------------------------------------------------
# Player Module
# -----------------------------------------------------------------------------
player-title = Reprodutor de Mídia
player-no-file = Nenhum arquivo selecionado
player-search = Buscar arquivos...
player-recent = Recentes
player-files = Arquivos

# -----------------------------------------------------------------------------
# Goals Module
# -----------------------------------------------------------------------------
goals-title = Metas e OKRs
goals-dashboard = Painel
goals-objectives = Objetivos
goals-alignment = Alinhamento
goals-ai-suggestions = Sugestões da IA

# semantic i18n key (#2542)
admin-ops-dashboard-title = Painel de operações

# semantic i18n key (#2542)
admin-ops-dashboard-subtitle = Métricas do sistema em tempo real, rastreamento distribuído e monitoramento de desempenho

# semantic i18n key (#2542)
admin-ops-auto-refresh-on = Atualização automática: ATIVADA

# semantic i18n key (#2542)
admin-ops-avg-response = Resposta média

# semantic i18n key (#2542)
admin-ops-requests-min = Requisições/min

# semantic i18n key (#2542)
admin-ops-disk-io = E/S de disco

# semantic i18n key (#2542)
admin-ops-iops-read-write = IOPS (leitura/gravação)

# semantic i18n key (#2542)
admin-ops-in-95-out-50 = Entrada: 95 / Saída: 50

# semantic i18n key (#2542)
admin-ops-stable = Estável

# semantic i18n key (#2542)
admin-ops-request-metrics = Métricas de requisições

# semantic i18n key (#2542)
admin-ops-errors-label = Erros

# semantic i18n key (#2542)
admin-ops-distributed-tracing = Rastreamento distribuído

# semantic i18n key (#2542)
admin-ops-all-traces = Todos os rastros

# semantic i18n key (#2542)
admin-ops-errors-only = Apenas erros

# semantic i18n key (#2542)
admin-ops-api-requests = Requisições de API

# semantic i18n key (#2542)
admin-ops-post-api-chat-message = POST /api/chat/message

# semantic i18n key (#2542)
admin-ops-trace-a1b2c3d4e5f6 = trace_a1b2c3d4e5f6

# semantic i18n key (#2542)
admin-ops-get-api-kb-search = GET /api/kb/search

# semantic i18n key (#2542)
admin-ops-trace-b2c3d4e5f6g7 = trace_b2c3d4e5f6g7

# semantic i18n key (#2542)
admin-ops-post-api-billing-checkout = POST /api/billing/checkout

# semantic i18n key (#2542)
admin-ops-trace-c3d4e5f6g7h8 = trace_c3d4e5f6g7h8

# semantic i18n key (#2542)
admin-ops-get-api-user-profile = GET /api/user/profile

# semantic i18n key (#2542)
admin-ops-trace-d4e5f6g7h8i9 = trace_d4e5f6g7h8i9

# semantic i18n key (#2542)
admin-ops-api-gateway = Gateway de API

# semantic i18n key (#2542)
admin-ops-primary-2-replicas = Primário + 2 réplicas

# semantic i18n key (#2542)
admin-ops-qdrant-vector-db = Qdrant (BD vetorial)

# semantic i18n key (#2542)
admin-ops-redis-cache = Cache Redis

# semantic i18n key (#2542)
admin-ops-cluster-mode-3-shards = Modo cluster | 3 shards

# semantic i18n key (#2542)
admin-ops-llm-service = Serviço de LLM

# semantic i18n key (#2542)
admin-ops-openai-gpt-4-high-latency = OpenAI GPT-4 | Alta latência

# semantic i18n key (#2542)
admin-ops-object-storage = Armazenamento de objetos

# semantic i18n key (#2542)
admin-ops-s3-compatible = Compatível com S3

# semantic i18n key (#2542)
admin-ops-error-tracking = Rastreamento de erros

# semantic i18n key (#2542)
admin-ops-view-all-errors = Ver todos os erros

# semantic i18n key (#2542)
admin-ops-warnings = Avisos

# semantic i18n key (#2542)
admin-ops-payment-processing-failed-stripe-timeout = Falha no processamento do pagamento: tempo limite da API do Stripe

# semantic i18n key (#2542)
admin-ops-billing-checkout-rs-145 = billing/checkout.rs:145

# semantic i18n key (#2542)
admin-ops-rate-limit-approaching-openai = Limite de taxa se aproximando para a API da OpenAI

# semantic i18n key (#2542)
admin-ops-llm-openai-rs-89 = llm/openai.rs:89

# semantic i18n key (#2542)
admin-ops-core-analytics-rs-234 = core/analytics.rs:234

# semantic i18n key (#2542)
admin-ops-cache-miss-rate-above-threshold = Taxa de cache miss acima do limite

# semantic i18n key (#2542)
admin-ops-cache-redis-rs-67 = cache/redis.rs:67

# semantic i18n key (#2542)
admin-ops-endpoint-performance = Desempenho dos endpoints

# semantic i18n key (#2542)
admin-ops-by-requests = Por requisições

# semantic i18n key (#2542)
admin-ops-by-latency = Por latência

# semantic i18n key (#2542)
admin-ops-by-error-rate = Por taxa de erro

# semantic i18n key (#2542)
admin-ops-post = POST

# semantic i18n key (#2542)
admin-ops-api-chat-message = /api/chat/message

# semantic i18n key (#2542)
admin-ops-req-min = req/min

# semantic i18n key (#2542)
admin-ops-errors-metric = erros

# semantic i18n key (#2542)
admin-ops-api-kb-search = /api/kb/search

# semantic i18n key (#2542)
admin-roles-title = Gerenciamento de funções

# semantic i18n key (#2542)
admin-roles-subtitle = Configure funções e atribua permissões para controlar o acesso em toda a suíte

# semantic i18n key (#2542)
admin-roles-sidebar-title = Funções

# semantic i18n key (#2542)
admin-roles-system-filter = Funções do sistema

# semantic i18n key (#2542)
admin-roles-custom-filter = Funções personalizadas

# semantic i18n key (#2542)
admin-roles-select-title = Selecione uma função

# semantic i18n key (#2542)
admin-roles-select-hint = Escolha uma função na lista para ver e gerenciar suas permissões

# semantic i18n key (#2542)
admin-roles-name-label = Nome da função

# semantic i18n key (#2542)
admin-roles-name-badge-placeholder = role_name

# semantic i18n key (#2542)
admin-roles-description-placeholder = A descrição da função aparece aqui

# semantic i18n key (#2542)
admin-roles-permissions-label = Permissões

# semantic i18n key (#2542)
admin-roles-assigned-users-tab = Usuários atribuídos

# semantic i18n key (#2542)
admin-roles-assigned-groups-tab = Grupos atribuídos

# semantic i18n key (#2542)
admin-roles-permission-assignment = Atribuição de permissões

# semantic i18n key (#2542)
admin-roles-permission-assignment-help = Use a lista dupla para atribuir permissões a esta função

# semantic i18n key (#2542)
admin-roles-available-permissions = Permissões disponíveis

# semantic i18n key (#2542)
admin-roles-category-documents = Documentos

# semantic i18n key (#2542)
admin-roles-category-spreadsheets = Planilhas

# semantic i18n key (#2542)
admin-roles-category-presentations = Apresentações

# semantic i18n key (#2542)
admin-roles-category-meetings = Reuniões

# semantic i18n key (#2542)
admin-roles-category-ai-bots = IA e bots

# semantic i18n key (#2542)
admin-roles-category-automation = Automação

# semantic i18n key (#2542)
admin-roles-assigned-permissions = Permissões atribuídas

# semantic i18n key (#2542)
admin-roles-no-permissions-assigned = Nenhuma permissão atribuída

# semantic i18n key (#2542)
admin-roles-reset-to-default = Redefinir para o padrão

# semantic i18n key (#2542)
admin-roles-users-with-role = Usuários com esta função

# semantic i18n key (#2542)
admin-roles-groups-with-role = Grupos com esta função

# semantic i18n key (#2542)
admin-roles-change-history = Histórico de alterações de função

# semantic i18n key (#2542)
admin-roles-create-new-role = Criar nova função

# semantic i18n key (#2542)
admin-roles-name-help = Minúsculas, sem espaços (use sublinhados)

# semantic i18n key (#2542)
admin-roles-copy-permissions-from = Copiar permissões de

# semantic i18n key (#2542)
admin-roles-start-with-no-permissions = Começar sem permissões

# semantic i18n key (#2542)
admin-roles-standard-user = Usuário padrão

# semantic i18n key (#2542)
admin-roles-power-user = Usuário avançado

# semantic i18n key (#2542)
admin-roles-viewer = Visualizador

# semantic i18n key (#2542)
admin-roles-guest-user = Usuário convidado

# semantic i18n key (#2542)
admin-roles-create-role-btn = Criar função

# semantic i18n key (#2542)
admin-roles-delete-role-title = Excluir função

# semantic i18n key (#2542)
admin-roles-delete-warning = Isso removerá a função de todos os usuários e grupos. Esta ação não pode ser desfeita.

# semantic i18n key (#2542)
admin-roles-assign-to-users = Atribuir função a usuários

# semantic i18n key (#2542)
admin-roles-search-users-label = Buscar usuários

# semantic i18n key (#2542)
admin-roles-type-to-search-users = Digite para buscar usuários

# semantic i18n key (#2542)
admin-roles-expiration-optional = Expiração (opcional)

# semantic i18n key (#2542)
admin-roles-expiration-help = Deixe em branco para atribuição permanente

# semantic i18n key (#2542)
admin-roles-assign-to-groups = Atribuir função a grupos

# semantic i18n key (#2542)
admin-roles-search-groups-label = Buscar grupos

# semantic i18n key (#2542)
admin-roles-type-to-search-groups = Digite para buscar grupos

# semantic i18n key (#2542)
admin-roles-assign-role-btn = Atribuir função
