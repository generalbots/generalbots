# =============================================================================
# General Bots — pt-BR — Desktop shell, navigation, controls
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# General Bots - Traduções da Interface (Português Brasileiro)
# =============================================================================
# -----------------------------------------------------------------------------
# Navegação
# -----------------------------------------------------------------------------
nav-home = Início
nav-chat = Chat
nav-drive = Arquivos
nav-tasks = Tarefas
nav-mail = E-mail
nav-calendar = Calendário
nav-meet = Reuniões
nav-paper = Documentos
nav-research = Pesquisa
nav-analytics = Análises
nav-settings = Configurações
nav-admin = Administração
nav-monitoring = Monitoramento
nav-sources = Fontes
nav-tools = Ferramentas
nav-attendant = Atendente
nav-crm = CRM
nav-billing = Faturamento
nav-products = Produtos
nav-tickets = Chamados
nav-docs = Documentos
nav-sheet = Planilhas
nav-slides = Apresentações
nav-social = Social
nav-all-apps = Todos os Apps
nav-people = Pessoas
nav-editor = Editor
nav-dashboards = Painéis
nav-security = Segurança
nav-designer = Designer
nav-project = Projeto
nav-canvas = Canvas
nav-goals = Metas
nav-player = Player
nav-workspace = Área de Trabalho
nav-video = Vídeo
nav-learn = Aprender

# -----------------------------------------------------------------------------
# Painel Principal
# -----------------------------------------------------------------------------
dashboard-title = Painel
dashboard-welcome = Bem-vindo de volta, { $name }!
dashboard-quick-actions = Ações Rápidas
dashboard-recent-activity = Atividade Recente
dashboard-no-activity = Nenhuma atividade recente. Comece a explorar!
dashboard-analytics = Análises

# -----------------------------------------------------------------------------
# Ações Rápidas
# -----------------------------------------------------------------------------
quick-start-chat = Iniciar Chat
quick-upload-files = Enviar Arquivos
quick-new-task = Nova Tarefa
quick-compose-email = Escrever E-mail
quick-start-meeting = Iniciar Reunião
quick-new-event = Novo Evento

# -----------------------------------------------------------------------------
# Cards dos Aplicativos
# -----------------------------------------------------------------------------
app-chat-name = Chat
app-chat-desc = Conversas com IA. Faça perguntas, obtenha ajuda e automatize tarefas.
app-drive-name = Arquivos
app-drive-desc = Armazenamento em nuvem para seus arquivos. Envie, organize e compartilhe.
app-tasks-name = Tarefas
app-tasks-desc = Mantenha-se organizado com listas de tarefas, prioridades e prazos.
app-mail-name = E-mail
app-mail-desc = Cliente de e-mail com escrita assistida por IA e organização inteligente.
app-calendar-name = Calendário
app-calendar-desc = Agende reuniões, eventos e gerencie seu tempo de forma eficaz.
app-meet-name = Reuniões
app-meet-desc = Videoconferência com compartilhamento de tela e transcrição ao vivo.
app-paper-name = Documentos
app-paper-desc = Escreva documentos com assistência de IA. Notas, relatórios e mais.
app-research-name = Pesquisa
app-research-desc = Busca e descoberta com IA em todas as suas fontes.
app-analytics-name = Análises
app-analytics-desc = Painéis e relatórios para acompanhar uso e insights.

# -----------------------------------------------------------------------------
# Cabeçalho do Suite
# -----------------------------------------------------------------------------
suite-title = General Bots Suite
suite-tagline = Seu espaço de trabalho com IA. Converse, colabore e crie.
suite-new-intent = Nova Intenção

# -----------------------------------------------------------------------------
# Painel de IA
# -----------------------------------------------------------------------------
ai-developer = Desenvolvedor IA
ai-developing = Desenvolvendo: { $project }
ai-quick-actions = Ações Rápidas
ai-add-field = Adicionar campo
ai-change-color = Mudar cor
ai-add-validation = Adicionar validação
ai-export-data = Exportar dados
ai-placeholder = Digite suas modificações...
ai-thinking = IA está pensando...
ai-status-online = Online
ai-status-offline = Offline

# -----------------------------------------------------------------------------
# Busca
# -----------------------------------------------------------------------------
search-placeholder = Buscar...
search-no-results = Nenhum resultado encontrado
search-results = { $count ->
    [one] { $count } resultado
   *[other] { $count } resultados
}
search-in-progress = Buscando...
search-advanced = Busca Avançada
search-filters = Filtros
search-clear-filters = Limpar Filtros

# -----------------------------------------------------------------------------
# Paginação
# -----------------------------------------------------------------------------
# -----------------------------------------------------------------------------
# Tabelas
# -----------------------------------------------------------------------------
table-no-data = Nenhum dado disponível
table-loading = Carregando dados...
table-actions = Ações
table-select-all = Selecionar Todos
table-deselect-all = Desmarcar Todos
table-export = Exportar
table-import = Importar

# -----------------------------------------------------------------------------
# Formulários
# -----------------------------------------------------------------------------
form-required = Obrigatório
form-optional = Opcional
form-submit = Enviar
form-reset = Redefinir
form-clear = Limpar
form-uploading = Enviando...
form-processing = Processando...

# -----------------------------------------------------------------------------
# Modais / Diálogos
# -----------------------------------------------------------------------------
modal-confirm-title = Confirmar Ação
modal-confirm-message = Tem certeza que deseja prosseguir?
modal-delete-title = Confirmação de Exclusão
modal-delete-message = Esta ação não pode ser desfeita. Tem certeza?

# -----------------------------------------------------------------------------
# Dicas de Ferramentas
# -----------------------------------------------------------------------------
tooltip-copy = Copiar para área de transferência
tooltip-copied = Copiado!
tooltip-expand = Expandir
tooltip-collapse = Recolher
tooltip-refresh = Atualizar
tooltip-download = Baixar
tooltip-upload = Enviar
tooltip-print = Imprimir
tooltip-fullscreen = Tela Cheia
tooltip-exit-fullscreen = Sair da Tela Cheia

# -----------------------------------------------------------------------------
# Rótulos Comuns (adicionais)
# -----------------------------------------------------------------------------
label-username = Nome de usuário
label-email = E-mail
label-display-name = Nome de Exibição
label-password = Senha
label-role = Função
label-group-name = Nome do Grupo
label-hostname = Nome do Host
label-record-type = Tipo de Registro
label-target = Destino
label-your-name = Seu Nome

# -----------------------------------------------------------------------------
# Ações (adicionais)
# -----------------------------------------------------------------------------
action-register = Registrar

# -----------------------------------------------------------------------------
# Notificações
# -----------------------------------------------------------------------------
notifications-title = Notificações
notifications-clear = Limpar tudo
notifications-empty = Nenhuma notificação

# #1853 — desktop chrome, launchers and notification settings.
window-ready = Pronto
window-loading = Carregando aplicativo…
window-minimize = Minimizar
window-maximize = Maximizar
window-fullscreen = Tela cheia
window-exit-fullscreen = Sair da tela cheia
window-close = Fechar
app-loading = Aplicativo carregando…
start-menu-search-placeholder = Digite para encontrar um aplicativo…
start-menu-recent = Recentes
start-menu-no-apps = Nenhum aplicativo encontrado
start-menu-user = Usuário
category-ai = IA
category-business = Negócios
category-office = Escritório
category-dev = Desenvolvimento
category-system = Sistema
category-tools = Ferramentas
sidebar-apps = Aplicativos
sidebar-apps-toggle = Mostrar/ocultar aplicativos
sidebar-apps-toggle-aria = Alternar painel de aplicativos
sidebar-apps-filter = Filtrar aplicativos…
sidebar-apps-filter-aria = Filtrar aplicativos
sidebar-preview = Modo de visualização
sidebar-preview-hint = Mostrar os aplicativos que ainda estão em visualização
sidebar-preview-toggle = Alternar modo de visualização
sidebar-app-preview-title = {app} (visualização)
sidebar-apps-count = {count} aplicativos
sidebar-apps-count-preview = {count} (+ {preview} em visualização)
spotlight-placeholder = Pesquise aplicativos, arquivos… ou digite um objetivo para o Concierge
spotlight-hint = ↑↓ navegar · Enter executar · Ctrl+Space ou Esc fechar
spotlight-empty = Nenhum resultado
spotlight-goal = Executar objetivo: “{goal}”
spotlight-goal-subtitle = Concierge · planejar e executar entre aplicativos
palette-placeholder = Pesquise aplicativos e comandos. Ou digite @ <comando> para enviar ao assistente...
palette-empty = Nenhum aplicativo ou comando correspondente
palette-kind-app = Aplicativo
palette-hint = ↑↓ navegar · Enter executar · Esc fechar · Ctrl+K alternar
palette-open-app = Abrir {app}
palette-value-prompt = {name}: {summary} — valor para {key}:
palette-execute = Executar {method} {path} e resumir o resultado.
palette-use-command = Use o comando {command} para {summary}.
toast-dismiss = Dispensar
settings-language-display = Idioma de exibição
settings-language-display-desc = Escolha o idioma da interface do aplicativo
settings-language-regional = Configurações regionais
settings-language-regional-desc = Configure formatos de data, hora e números
settings-language-date-format = Formato de data
settings-language-time-format = Formato de hora
settings-language-timezone = Fuso horário
settings-language-first-day = Primeiro dia da semana
settings-language-time-12h = 12 horas (1:30 PM)
settings-language-time-24h = 24 horas (13:30)
settings-language-sunday = Domingo
settings-language-monday = Segunda-feira
settings-email-notifications-desc = Enviadas pelo remetente de notificações do espaço de trabalho, nunca por uma caixa Gmail/Outlook conectada.
settings-push-enable = Ativar notificações push
settings-push-enable-desc = Receba notificações push do navegador
settings-push-sound = Som
settings-push-sound-desc = Reproduzir som para notificações
settings-desktop-notifications = Notificações da área de trabalho
settings-desktop-notifications-desc = Mostrar notificações na área de trabalho
settings-badge-count = Contador de indicador
settings-badge-count-desc = Mostrar indicador de não lidas
settings-save-preferences = Salvar preferências
settings-notifications-saved = Preferências de notificação salvas
settings-notification-recipient = Caixa de entrega
settings-notification-recipient-desc = Deixe em branco para usar sua conta principal do Mail ou o e-mail do perfil.
language-search-placeholder = Pesquisar idiomas…
settings-profile-title = Configurações de Perfil
settings-security-title = Configurações de Segurança
