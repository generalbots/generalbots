# =============================================================================
# General Bots — es — Attendant (support queue)
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# ATTENDANT / CRM SCREENS
# =============================================================================
# -----------------------------------------------------------------------------
# CRM Disabled State
# -----------------------------------------------------------------------------
attendant-crm-disabled = Funciones de CRM no habilitadas
attendant-crm-disabled-desc = La Attendant Console requiere que las funciones de CRM estén habilitadas para este bot. Esto permite a los agentes humanos recibir y responder a conversaciones transferidas desde el bot.
attendant-crm-enable-instruction = Para habilitar las funciones de CRM, agregue esta línea a la lista de su bot
attendant-crm-config-file = configuración.csv
attendant-crm-create-attendant = Luego crea un
attendant-crm-attendant-file = asistente.csv
attendant-crm-configure-team = archivo para configurar tu equipo

# -----------------------------------------------------------------------------
# Queue Sidebar
# -----------------------------------------------------------------------------
attendant-title = Consola de operadora
attendant-status-online = En línea
attendant-status-busy = Ocupado
attendant-status-away = lejos
attendant-status-offline = Sin conexión
attendant-status-ready = En línea: listo para conversaciones
attendant-status-busy-msg = Ocupado: manejo de conversaciones
attendant-status-away-msg = Fuera - Volveremos pronto
attendant-status-offline-msg = Sin conexión: no disponible

# -----------------------------------------------------------------------------
# Queue Stats
# -----------------------------------------------------------------------------
attendant-waiting = esperando
attendant-active = Activo
attendant-resolved = Resuelto
attendant-mine = mio

# -----------------------------------------------------------------------------
# Queue Filters
# -----------------------------------------------------------------------------
attendant-filter-all = Todos
attendant-filter-waiting = esperando
attendant-filter-mine = mio
attendant-filter-priority = Prioridad

# -----------------------------------------------------------------------------
# Conversation List
# -----------------------------------------------------------------------------
attendant-no-conversations = No hay conversaciones en cola
attendant-new-conversations-appear = Nuevas conversaciones aparecerán aquí.
attendant-unread = No leído
attendant-typing = escribiendo...
attendant-select-conversation = Seleccione una conversación
attendant-select-conversation-desc = Elija una conversación de la cola para comenzar a responder

# -----------------------------------------------------------------------------
# Channel Tags
# -----------------------------------------------------------------------------
attendant-channel-whatsapp = WhatsApp
attendant-channel-teams = equipos
attendant-channel-instagram = Instagram
attendant-channel-web = Web
attendant-channel-telegram = telegrama
attendant-channel-email = Correo electrónico

# -----------------------------------------------------------------------------
# Priority Tags
# -----------------------------------------------------------------------------
attendant-priority-urgent = Urgente
attendant-priority-high = Alto
attendant-priority-normal = normales

# -----------------------------------------------------------------------------
# Chat Area
# -----------------------------------------------------------------------------
attendant-message-placeholder = Escribe tu mensaje...
attendant-send = enviar
attendant-attach-file = Adjuntar archivo
attendant-insert-emoji = Insertar emojis
attendant-quick-responses = Respuestas Rápidas
attendant-transfer = Transferir
attendant-resolve = resolver
attendant-more-actions = Más acciones

# -----------------------------------------------------------------------------
# Quick Responses
# -----------------------------------------------------------------------------
attendant-quick-greeting = ¡Hola! ¿Cómo puedo ayudarte hoy?
attendant-quick-thanks = Gracias por tu paciencia.
attendant-quick-checking = Déjame comprobarlo por ti.
attendant-quick-moment = Un momento por favor.

# -----------------------------------------------------------------------------
# Transfer Modal
# -----------------------------------------------------------------------------
attendant-transfer-title = Transferir conversación
attendant-transfer-to = Transferir a
attendant-transfer-reason = Razón (opcional)
attendant-transfer-reason-placeholder = ¿Por qué estás transfiriendo esta conversación?
attendant-transfer-cancel = Cancelar
attendant-transfer-confirm = Transferir

# -----------------------------------------------------------------------------
# AI Insights Sidebar
# -----------------------------------------------------------------------------
attendant-ai-insights = Perspectivas de IA
attendant-ai-summary = Resumen de la conversación
attendant-ai-sentiment = Sentimiento del cliente
attendant-sentiment-positive = Positivo
attendant-sentiment-neutral = Neutro
attendant-sentiment-negative = Negativo
attendant-smart-replies = Respuestas inteligentes
attendant-confidence = confianza
attendant-source = Fuente

# -----------------------------------------------------------------------------
# Customer Details
# -----------------------------------------------------------------------------
attendant-customer-details = Detalles del cliente
attendant-customer-name = Nombre
attendant-customer-email = Correo electrónico
attendant-customer-phone = Teléfono
attendant-customer-location = Ubicación
attendant-customer-tags = Etiquetas

# -----------------------------------------------------------------------------
# Conversation History
# -----------------------------------------------------------------------------
attendant-history = Historia
attendant-history-resolved = Resuelto
attendant-history-transferred = Transferido
attendant-history-abandoned = abandonado
attendant-view-history = Ver historial completo

# -----------------------------------------------------------------------------
# Toast Messages
# -----------------------------------------------------------------------------
attendant-toast-transferred = Conversación transferida exitosamente
attendant-toast-resolved = Conversación marcada como resuelta
attendant-toast-assigned = Conversación asignada a ti
attendant-toast-error = Se produjo un error
attendant-toast-connection-lost = Conexión perdida. Reconectando...
attendant-toast-connection-restored = Conexión restaurada
