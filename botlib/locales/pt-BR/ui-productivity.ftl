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
