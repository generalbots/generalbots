# =============================================================================
# General Bots — pt-BR — Attendant (support queue)
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# TELAS DE ATENDENTE / CRM
# =============================================================================
# -----------------------------------------------------------------------------
# Estado CRM Desabilitado
# -----------------------------------------------------------------------------
attendant-crm-disabled = Recursos de CRM Não Habilitados
attendant-crm-disabled-desc = O Console do Atendente requer que os recursos de CRM estejam habilitados para este bot. Isso permite que agentes humanos recebam e respondam conversas transferidas do bot.
attendant-crm-enable-instruction = Para habilitar os recursos de CRM, adicione esta linha ao
attendant-crm-config-file = config.csv do seu bot
attendant-crm-create-attendant = Depois crie um arquivo
attendant-crm-attendant-file = attendant.csv
attendant-crm-configure-team = para configurar sua equipe

# -----------------------------------------------------------------------------
# Barra Lateral da Fila
# -----------------------------------------------------------------------------
attendant-title = Console do Atendente
attendant-status-online = Online
attendant-status-busy = Ocupado
attendant-status-away = Ausente
attendant-status-offline = Offline
attendant-status-ready = Online - Pronto para conversas
attendant-status-busy-msg = Ocupado - Atendendo conversas
attendant-status-away-msg = Ausente - Volto em breve
attendant-status-offline-msg = Offline - Não disponível

# -----------------------------------------------------------------------------
# Estatísticas da Fila
# -----------------------------------------------------------------------------
attendant-waiting = Aguardando
attendant-active = Ativas
attendant-resolved = Resolvidas
attendant-mine = Minhas

# -----------------------------------------------------------------------------
# Filtros da Fila
# -----------------------------------------------------------------------------
attendant-filter-all = Todas
attendant-filter-waiting = Aguardando
attendant-filter-mine = Minhas
attendant-filter-priority = Prioridade

# -----------------------------------------------------------------------------
# Lista de Conversas
# -----------------------------------------------------------------------------
attendant-no-conversations = Nenhuma conversa na fila
attendant-new-conversations-appear = Novas conversas aparecerão aqui
attendant-unread = Não lida
attendant-typing = digitando...
attendant-select-conversation = Selecione uma conversa
attendant-select-conversation-desc = Escolha uma conversa da fila para começar a responder

# -----------------------------------------------------------------------------
# Tags de Canal
# -----------------------------------------------------------------------------
attendant-channel-whatsapp = WhatsApp
attendant-channel-teams = Teams
attendant-channel-instagram = Instagram
attendant-channel-web = Web
attendant-channel-telegram = Telegram
attendant-channel-email = E-mail

# -----------------------------------------------------------------------------
# Tags de Prioridade
# -----------------------------------------------------------------------------
attendant-priority-urgent = Urgente
attendant-priority-high = Alta
attendant-priority-normal = Normal

# -----------------------------------------------------------------------------
# Área de Chat
# -----------------------------------------------------------------------------
attendant-message-placeholder = Digite sua mensagem...
attendant-send = Enviar
attendant-attach-file = Anexar arquivo
attendant-insert-emoji = Inserir emoji
attendant-quick-responses = Respostas Rápidas
attendant-transfer = Transferir
attendant-resolve = Resolver
attendant-more-actions = Mais Ações

# -----------------------------------------------------------------------------
# Respostas Rápidas
# -----------------------------------------------------------------------------
attendant-quick-greeting = Olá! Como posso ajudá-lo hoje?
attendant-quick-thanks = Obrigado pela sua paciência.
attendant-quick-checking = Deixe-me verificar isso para você.
attendant-quick-moment = Um momento, por favor.

# -----------------------------------------------------------------------------
# Modal de Transferência
# -----------------------------------------------------------------------------
attendant-transfer-title = Transferir Conversa
attendant-transfer-to = Transferir para
attendant-transfer-reason = Motivo (opcional)
attendant-transfer-reason-placeholder = Por que você está transferindo esta conversa?
attendant-transfer-cancel = Cancelar
attendant-transfer-confirm = Transferir

# -----------------------------------------------------------------------------
# Barra Lateral de Insights de IA
# -----------------------------------------------------------------------------
attendant-ai-insights = Insights de IA
attendant-ai-summary = Resumo da Conversa
attendant-ai-sentiment = Sentimento do Cliente
attendant-sentiment-positive = Positivo
attendant-sentiment-neutral = Neutro
attendant-sentiment-negative = Negativo
attendant-smart-replies = Respostas Inteligentes
attendant-confidence = Confiança
attendant-source = Fonte

# -----------------------------------------------------------------------------
# Detalhes do Cliente
# -----------------------------------------------------------------------------
attendant-customer-details = Detalhes do Cliente
attendant-customer-name = Nome
attendant-customer-email = E-mail
attendant-customer-phone = Telefone
attendant-customer-location = Localização
attendant-customer-tags = Tags

# -----------------------------------------------------------------------------
# Histórico de Conversas
# -----------------------------------------------------------------------------
attendant-history = Histórico
attendant-history-resolved = Resolvida
attendant-history-transferred = Transferida
attendant-history-abandoned = Abandonada
attendant-view-history = Ver Histórico Completo

# -----------------------------------------------------------------------------
# Mensagens Toast
# -----------------------------------------------------------------------------
attendant-toast-transferred = Conversa transferida com sucesso
attendant-toast-resolved = Conversa marcada como resolvida
attendant-toast-assigned = Conversa atribuída a você
attendant-toast-error = Ocorreu um erro
attendant-toast-connection-lost = Conexão perdida. Reconectando...
attendant-toast-connection-restored = Conexão restaurada
