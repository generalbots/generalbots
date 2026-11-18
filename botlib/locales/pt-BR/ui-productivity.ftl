# =============================================================================
# General Bots — pt-BR — Productivity apps (tasks, calendar, meet, mail, research, paper)
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Tarefas
# -----------------------------------------------------------------------------
tasks-title = Tarefas
tasks-new = Nova Tarefa
tasks-due-today = Para Hoje
tasks-overdue = Atrasadas
tasks-completed = Concluídas
tasks-all = Todas as Tarefas
tasks-priority-high = Prioridade Alta
tasks-priority-medium = Prioridade Média
tasks-priority-low = Prioridade Baixa
tasks-no-due-date = Sem prazo definido
tasks-add-subtask = Adicionar subtarefa
tasks-mark-complete = Marcar como concluída
tasks-mark-incomplete = Marcar como pendente
tasks-delete-confirm = Tem certeza que deseja excluir esta tarefa?
tasks-count = { $count ->
    [zero] Nenhuma tarefa
    [one] { $count } tarefa
   *[other] { $count } tarefas
}

# -----------------------------------------------------------------------------
# Calendário
# -----------------------------------------------------------------------------
calendar-title = Calendário
calendar-today = Hoje
calendar-new-event = Novo Evento
calendar-all-day = Dia inteiro
calendar-repeat = Repetir
calendar-reminder = Lembrete
calendar-view-day = Dia
calendar-view-week = Semana
calendar-view-month = Mês
calendar-view-year = Ano
calendar-no-events = Nenhum evento agendado
calendar-event-title = Título do evento
calendar-event-location = Local
calendar-event-description = Descrição
calendar-event-attendees = Participantes

# -----------------------------------------------------------------------------
# Reuniões / Videoconferência
# -----------------------------------------------------------------------------
meet-title = Reuniões
meet-join = Entrar na Reunião
meet-start = Iniciar Reunião
meet-mute = Silenciar
meet-unmute = Ativar Som
meet-video-on = Ligar Câmera
meet-video-off = Desligar Câmera
meet-share-screen = Compartilhar Tela
meet-stop-sharing = Parar Compartilhamento
meet-end-call = Encerrar Chamada
meet-leave = Sair da Reunião
meet-participants = { $count ->
    [one] { $count } participante
   *[other] { $count } participantes
}
meet-waiting-room = Sala de Espera
meet-admit = Admitir
meet-remove = Remover
meet-chat = Chat da Reunião
meet-raise-hand = Levantar a Mão
meet-lower-hand = Abaixar a Mão
meet-recording = Gravando
meet-start-recording = Iniciar Gravação
meet-stop-recording = Parar Gravação

# -----------------------------------------------------------------------------
# E-mail
# -----------------------------------------------------------------------------
mail-title = E-mail
mail-compose = Escrever
mail-inbox = Caixa de Entrada
mail-sent = Enviados
mail-drafts = Rascunhos
mail-trash = Lixeira
mail-spam = Spam
mail-starred = Com Estrela
mail-archive = Arquivar
mail-to = Para
mail-cc = CC
mail-bcc = CCO
mail-subject = Assunto
mail-body = Mensagem
mail-reply = Responder
mail-reply-all = Responder a Todos
mail-forward = Encaminhar
mail-send = Enviar
mail-discard = Descartar
mail-save-draft = Salvar Rascunho
mail-attach = Anexar Arquivos
mail-unread = { $count ->
    [one] { $count } não lido
   *[other] { $count } não lidos
}
mail-empty-inbox = Sua caixa de entrada está vazia
mail-no-subject = (Sem assunto)

# -----------------------------------------------------------------------------
# Paper (Editor de Documentos)
# -----------------------------------------------------------------------------
paper-title = Documentos
paper-new-note = Nova Nota
paper-search-notes = Buscar notas...
paper-quick-start = Início Rápido
paper-template-blank = Em Branco
paper-template-meeting = Reunião
paper-template-todo = Tarefas
paper-template-research = Pesquisa
paper-untitled = Sem título
paper-placeholder = Comece a escrever, ou digite / para comandos...
paper-commands = Comandos
paper-heading1 = Título 1
paper-heading1-desc = Título de seção grande
paper-heading2 = Título 2
paper-heading2-desc = Título de seção médio
paper-heading3 = Título 3
paper-heading3-desc = Título de seção pequeno
paper-paragraph = Parágrafo
paper-paragraph-desc = Texto simples
paper-bullet-list = Lista com Marcadores
paper-bullet-list-desc = Lista não ordenada
paper-numbered-list = Lista Numerada
paper-numbered-list-desc = Lista ordenada
paper-todo-list = Lista de Tarefas
paper-todo-list-desc = Lista com caixas de seleção
paper-quote = Citação
paper-quote-desc = Bloco de citação
paper-divider = Divisor
paper-divider-desc = Linha horizontal
paper-code-block = Bloco de Código
paper-code-block-desc = Código formatado
paper-table = Tabela
paper-table-desc = Inserir tabela
paper-image = Imagem
paper-image-desc = Inserir imagem de URL
paper-callout = Destaque
paper-callout-desc = Caixa de informação destacada
paper-ai-write = IA Escrever
paper-ai-write-desc = Gerar texto com IA
paper-ai-summarize = IA Resumir
paper-ai-summarize-desc = Resumir texto selecionado
paper-ai-expand = IA Expandir
paper-ai-expand-desc = Expandir texto selecionado
paper-ai-improve = IA Melhorar
paper-ai-improve-desc = Melhorar qualidade da escrita
paper-ai-translate = IA Traduzir
paper-ai-translate-desc = Traduzir para outro idioma
paper-ai-assistant = Assistente de IA
paper-ai-quick-actions = Ações Rápidas
paper-ai-rewrite = Reescrever
paper-ai-make-shorter = Encurtar
paper-ai-make-longer = Expandir
paper-ai-fix-grammar = Corrigir Gramática
paper-ai-tone = Tom
paper-ai-tone-professional = Profissional
paper-ai-tone-casual = Casual
paper-ai-tone-friendly = Amigável
paper-ai-tone-formal = Formal
paper-ai-translate-to = Traduzir para
paper-ai-custom-prompt = Prompt Personalizado
paper-ai-custom-placeholder = Descreva o que você quer...
paper-ai-generate = Gerar
paper-ai-response = Resposta da IA
paper-ai-apply = Aplicar
paper-ai-regenerate = Regenerar
paper-ai-copy = Copiar
paper-word-count = { $count } palavras
paper-char-count = { $count } caracteres
paper-saved = Salvo
paper-saving = Salvando...
paper-last-edited = Última edição: { $time }
paper-last-edited-now = Última edição: Agora mesmo
paper-export = Exportar Documento
paper-export-pdf = PDF
paper-export-docx = Word (.docx)
paper-export-markdown = Markdown
paper-export-html = HTML
paper-export-txt = Texto Simples

# Traduções de Tarefas
tasks-active = Intenções Ativas
tasks-awaiting = Aguardando Decisão
tasks-paused = Pausadas
tasks-blocked = Bloqueadas/Problemas
tasks-time-saved = Tempo Economizado:
tasks-input-placeholder = O que você gostaria de fazer? Ex: 'criar um app de CRM' ou 'me lembrar de ligar pro João amanhã'

# Traduções adicionais do Calendário
calendar-my-calendars = Meus Calendários

# Traduções adicionais de E-mail
email-scheduled = Agendados
email-tracking = Rastreamento

# Traduções de pastas de E-mail
email-inbox = Caixa de Entrada
email-starred = Com Estrela
email-sent = Enviados
email-drafts = Rascunhos
email-spam = Spam
email-trash = Lixeira
email-compose = Escrever

# -----------------------------------------------------------------------------
# Pesquisa (Research)
# -----------------------------------------------------------------------------
research-title = Pesquisa
research-search-placeholder = Pergunte qualquer coisa...
research-collections = Coleções
research-new-collection = Nova Coleção
research-recent = Recentes
research-academic = Acadêmico
research-code = Código
research-internal = Interno
research-search-all = Buscar em tudo
research-academic-papers = Artigos acadêmicos
research-code-docs = Código e documentação
research-internal-kb = Base de conhecimento interna
research-sources = Fontes
research-trending = Em alta
research-pro-search = Busca Pro
research-include-images = Incluir Imagens
research-try-asking = Tente perguntar sobre
research-related = Perguntas Relacionadas
research-view-all-sources = Ver Todas as Fontes
research-export-citations = Exportar Citações
research-save-to-collection = Salvar na Coleção

# -----------------------------------------------------------------------------
# Reuniões (chaves adicionais)
# -----------------------------------------------------------------------------
meet-new-meeting = Nova Reunião
meet-join-meeting = Entrar na Reunião
meet-active-rooms = Salas Ativas
meet-room-title = Sala de Reunião
meet-record = Gravar
meet-camera = Câmera
meet-share = Compartilhar
meet-info = Info
meet-more = Mais
meet-share-meeting = Compartilhar Reunião
meet-meeting-title = Título da Reunião
meet-meeting-code = Código da Reunião
meet-meeting-link = Link da Reunião
meet-send-invite = Enviar Convite

# ── #2307 Tasks goal/monitor detail panel ──────────────────────────
tasks-detail-untitled-goal = Meta sem título
tasks-detail-goal-progress = Progresso da meta
tasks-detail-current-value = Valor atual
tasks-detail-target-value = Valor alvo
tasks-detail-percent = Progresso
tasks-detail-no-actions = Nenhuma ação ainda
tasks-detail-pending-info = Informações pendentes
tasks-detail-scheduler-info = Informações do agendador
tasks-detail-monitor-info = Informações do monitor
tasks-detail-unavailable = Indisponível
tasks-detail-load-failed = Não foi possível carregar os detalhes da meta


# ── #2307 Database insert/confirm dialogs ──────────────────────────
db-insert-row = Inserir linha
db-insert-into = Inserir em
db-provide-value = Informe pelo menos um valor
db-insert-failed = Falha ao inserir
db-confirm-title = Confirmar
db-confirm-accept = Confirmar
db-confirm-delete = Excluir esta linha?
db-confirm-drop = Excluir tabela
db-delete-failed = Falha ao excluir
db-drop-failed = Falha ao excluir tabela

# Tasks detail view (#2307)
tasks-detail-type-goal = Goal
tasks-detail-status-pending = Pending
tasks-detail-pending-title = Pending items
tasks-detail-required-reason = A reason is required.
tasks-detail-type-scheduler = Scheduler
tasks-detail-type-monitor = Monitor
tasks-detail-last-action = Last action
tasks-detail-next-run = Next run
tasks-detail-file = File
tasks-detail-interval = Interval
tasks-detail-last-check = Last check
tasks-detail-last-value = Last value
tasks-detail-progress = Progress
tasks-detail-app = App
tasks-detail-field = Field
tasks-detail-reason = Reason
tasks-detail-cron = Cron

# Database dialog copy (#2307)
db-error = Error
db-database = Database
db-empty-query = Execute a query to see results
db-insert = Insert
db-select-table = Select a table
tasks-load-error = Não foi possível carregar as tarefas.
tasks-sign-in-to-load = Entre para carregar as tarefas.
calendar-collapse-sidebar = Recolher barra lateral
calendar-prev-month = Mês anterior
calendar-next-month = Próximo mês
calendar-prev-period = Período anterior
calendar-next-period = Próximo período
calendar-add-calendar = Adicionar calendário
paper-toggle-sidebar = Alternar barra lateral
paper-insert-ai-response = Inserir resposta da IA
paper-replace-selection = Substituir seleção
paper-highlight = Destacar
paper-align-left = Alinhar à esquerda
paper-align-center = Centralizar
paper-align-right = Alinhar à direita
paper-insert-link = Inserir link
paper-insert-image = Inserir imagem
paper-insert-table = Inserir tabela
paper-bold = Negrito
paper-italic = Itálico
paper-underline = Sublinhado
paper-strikethrough = Tachado
products-view-grid = Visualização em grade
products-view-list = Visualização em lista
sheet-freeze-panes = Congelar painéis

# semantic i18n key (#2542)
db-schema-title = 🗄️ Esquema do banco de dados

# semantic i18n key (#2542)
db-loading-tables = Carregando tabelas...

# semantic i18n key (#2542)
db-query-builder = Construtor de consultas

# semantic i18n key (#2542)
db-run-query = Executar consulta

# semantic i18n key (#2542)
db-select-table-hint = Selecione uma tabela para ver os dados

# semantic i18n key (#2542)
db-prev-page = ◀ Anterior

# semantic i18n key (#2542)
db-page-info-default = Página 1 de 1

# semantic i18n key (#2542)
db-next-page = Próximo ▶

# semantic i18n key (#2542)
research-searching-sources = Pesquisando fontes...

# semantic i18n key (#2542)
research-suggestion-latest-ai = Últimos desenvolvimentos em IA

# semantic i18n key (#2542)
research-suggestion-quantum = Computação quântica explicada

# semantic i18n key (#2542)
research-suggestion-software-architecture = Arquitetura de software

# semantic i18n key (#2542)
research-suggestion-ai-productivity = Ferramentas de produtividade com IA

# semantic i18n key (#2542)
research-related-heading = Relacionado

# semantic i18n key (#2542)
research-loading = Carregando Pesquisa…

# semantic i18n key (#2542)
research-service-unavailable = Serviço de pesquisa indisponível

# semantic i18n key (#2542)
calendar-mini-month-placeholder = Janeiro de 2025

# semantic i18n key (#2542)
calendar-upcoming = Próximos

# semantic i18n key (#2542)
calendar-import-export = Importar / Exportar

# semantic i18n key (#2542)
calendar-popup-event-title = Título do evento

# semantic i18n key (#2542)
calendar-popup-location-placeholder = Sala de conferência A

# semantic i18n key (#2542)
calendar-popup-event-description = Descrição do evento

# semantic i18n key (#2542)
calendar-personal = Pessoal

# semantic i18n key (#2542)
calendar-import-ical-title = Importar arquivo iCal

# semantic i18n key (#2542)
calendar-select-ics-file = Selecionar arquivo .ics

# semantic i18n key (#2542)
calendar-import-to-calendar-label = Importar para o calendário

# semantic i18n key (#2542)
calendar-default-calendar = Calendário padrão

# semantic i18n key (#2542)
calendar-loading = Carregando calendário…

# semantic i18n key (#2542)
calendar-unavailable = Calendário indisponível

# semantic i18n key (#2542)
tasks-progress-status = STATUS

# semantic i18n key (#2542)
tasks-progress-sample-title = Implementar sistema de autenticação de usuários

# semantic i18n key (#2542)
tasks-progress-runtime = Tempo de execução:

# semantic i18n key (#2542)
tasks-progress-sample-action = Escolher estratégia de expiração do token

# semantic i18n key (#2542)
tasks-progress-estimated = Estimado:

# semantic i18n key (#2542)
tasks-progress-sample-decision = Precisará de aprovação para a configuração de segurança

# semantic i18n key (#2542)
tasks-progress-log = REGISTRO DE PROGRESSO

# semantic i18n key (#2542)
tasks-progress-terminal-live = TERMINAL (ATIVIDADE DO AGENTE AO VIVO)

# semantic i18n key (#2542)
tasks-progress-processed = Processado:

# semantic i18n key (#2542)
tasks-progress-data-points = pontos de dados

# semantic i18n key (#2542)
tasks-progress-sample-speed = ~8 fontes/min

# semantic i18n key (#2542)
tasks-progress-sample-insight = Sintetizando insights competitivos...

# semantic i18n key (#2542)
tasks-progress-sample-pricing = Cruzando dados de preços...

# semantic i18n key (#2542)
tasks-progress-view-details = Ver detalhes ▸

# semantic i18n key (#2542)
tasks-create-run = Criar e executar

# semantic i18n key (#2542)
tasks-loading-tasks = Carregando tarefas...

# semantic i18n key (#2542)
tasks-select-task = Selecione uma tarefa

# semantic i18n key (#2542)
tasks-select-task-desc = Clique em uma tarefa da lista para ver os detalhes

# semantic i18n key (#2542)
tasks-bot-database-label = Banco de dados do bot:

# semantic i18n key (#2542)
tasks-shared-resources-label = Recursos compartilhados:

# semantic i18n key (#2542)
tasks-starting = Iniciando...

# semantic i18n key (#2542)
tasks-create-new-intent = Criar nova intenção

# semantic i18n key (#2542)
tasks-intent-accomplish-label = O que você gostaria de realizar?

# semantic i18n key (#2542)
tasks-execution-mode = Modo de execução

# semantic i18n key (#2542)
tasks-mode-automatic = Automático

# semantic i18n key (#2542)
tasks-mode-supervised = Supervisionado

# semantic i18n key (#2542)
tasks-mode-manual-approval = Aprovação manual

# semantic i18n key (#2542)
tasks-make-decision = Tomar decisão

# semantic i18n key (#2542)
tasks-decision-required = Decisão necessária

# semantic i18n key (#2542)
tasks-loading-decision-details = Carregando detalhes da decisão...

# semantic i18n key (#2542)
tasks-skip-for-now = Pular por enquanto

# semantic i18n key (#2542)
tasks-confirm-decision = Confirmar decisão

# semantic i18n key (#2542)
tasks-due-soon = Vence em breve

# semantic i18n key (#2542)
tasks-autotask-items = // ITENS DE AUTOTAREFA

# semantic i18n key (#2542)
tasks-edit-task = Editar tarefa

# semantic i18n key (#2542)
tasks-assignee-user-id = Responsável (ID do usuário)

# semantic i18n key (#2542)
tasks-parent-task-subtask-of = Tarefa pai (subtarefa de…)

# semantic i18n key (#2542)
tasks-reopen-action = ↩ Reabrir

# semantic i18n key (#2542)
tasks-complete-action = ✓ Concluir

# semantic i18n key (#2542)
tasks-delete-action = 🗑 Excluir
