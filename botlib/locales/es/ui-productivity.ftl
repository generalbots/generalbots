# =============================================================================
# General Bots — es — Productivity apps (tasks, calendar, meet, mail, research, paper)
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Tasks
# -----------------------------------------------------------------------------
tasks-title = Tareas
tasks-new = Nueva Tarea
tasks-due-today = Vence Hoy
tasks-overdue = Vencidas
tasks-completed = Completadas
tasks-all = Todas las Tareas
tasks-priority-high = Prioridad Alta
tasks-priority-medium = Prioridad Media
tasks-priority-low = Prioridad Baja
tasks-no-due-date = Sin fecha límite
tasks-add-subtask = Agregar subtarea
tasks-mark-complete = Marcar como completada
tasks-mark-incomplete = Marcar como incompleta
tasks-delete-confirm = ¿Estás seguro de que deseas eliminar esta tarea?
tasks-count = { $count ->
    [zero] No tasks
    [one] { $count } task
   *[other] { $count } tasks
}

# -----------------------------------------------------------------------------
# Calendar
# -----------------------------------------------------------------------------
calendar-title = Calendario
calendar-today = Hoy
calendar-new-event = Nuevo Evento
calendar-all-day = Todo el día
calendar-repeat = Repetir
calendar-reminder = Recordatorio
calendar-view-day = Día
calendar-view-week = Semana
calendar-view-month = Mes
calendar-view-year = Año
calendar-no-events = Sin eventos programados
calendar-event-title = Título del evento
calendar-event-location = Ubicación
calendar-event-description = Descripción
calendar-event-attendees = Asistentes

# -----------------------------------------------------------------------------
# Meet / Video Conferencing
# -----------------------------------------------------------------------------
meet-title = Reuniones
meet-join = Unirse a Reunión
meet-start = Iniciar Reunión
meet-mute = Silenciar
meet-unmute = Activar Micrófono
meet-video-on = Cámara Encendida
meet-video-off = Cámara Apagada
meet-share-screen = Compartir Pantalla
meet-stop-sharing = Dejar de Compartir
meet-end-call = Finalizar Llamada
meet-leave = Salir de la Reunión
meet-participants = { $count ->
    [one] { $count } participant
   *[other] { $count } participants
}
meet-waiting-room = Sala de Espera
meet-admit = Admitir
meet-remove = Eliminar
meet-chat = Chat de Reunión
meet-raise-hand = Levantar Mano
meet-lower-hand = Bajar Mano
meet-recording = Grabando
meet-start-recording = Iniciar Grabación
meet-stop-recording = Detener Grabación

# -----------------------------------------------------------------------------
# Mail / Email
# -----------------------------------------------------------------------------
mail-title = Correo
mail-compose = Redactar
mail-inbox = Bandeja de Entrada
mail-sent = Enviados
mail-drafts = Borradores
mail-trash = Papelera
mail-spam = Spam
mail-starred = Destacados
mail-archive = Archivo
mail-to = Para
mail-cc = CC
mail-bcc = CCO
mail-subject = Asunto
mail-body = Mensaje
mail-reply = Responder
mail-reply-all = Responder a Todos
mail-forward = Reenviar
mail-send = Enviar
mail-discard = Descartar
mail-save-draft = Guardar Borrador
mail-attach = Adjuntar Archivos
mail-unread = { $count ->
    [one] { $count } unread
   *[other] { $count } unread
}
mail-empty-inbox = Tu bandeja de entrada está vacía
mail-no-subject = (Sin asunto)

# -----------------------------------------------------------------------------
# Paper (Document Editor)
# -----------------------------------------------------------------------------
paper-title = Documentos
paper-new-note = Nueva Nota
paper-search-notes = Buscar notas...
paper-quick-start = Inicio Rápido
paper-template-blank = En Blanco
paper-template-meeting = Reunión
paper-template-todo = Lista de Tareas
paper-template-research = Investigación
paper-untitled = Sin Título
paper-placeholder = Comienza a escribir, o escribe / para comandos...
paper-commands = Comandos
paper-heading1 = Título 1
paper-heading1-desc = Título de sección grande
paper-heading2 = Título 2
paper-heading2-desc = Título de sección mediano
paper-heading3 = Título 3
paper-heading3-desc = Título de sección pequeño
paper-paragraph = Párrafo
paper-paragraph-desc = Texto plano
paper-bullet-list = Lista con Viñetas
paper-bullet-list-desc = Lista sin orden
paper-numbered-list = Lista Numerada
paper-numbered-list-desc = Lista ordenada
paper-todo-list = Lista de Tareas
paper-todo-list-desc = Lista de tareas marcables
paper-quote = Cita
paper-quote-desc = Bloque de cita para referencias
paper-divider = Divisor
paper-divider-desc = línea horizontal
paper-code-block = Bloque de código
paper-code-block-desc = código formateado
paper-table = mesa
paper-table-desc = Insertar tabla
paper-image = Imagen
paper-image-desc = Insertar imagen desde URL
paper-callout = Llamada
paper-callout-desc = Cuadro de información resaltado
paper-ai-write = Escritura IA
paper-ai-write-desc = Generar texto con IA
paper-ai-summarize = Resumen de IA
paper-ai-summarize-desc = Resumir el texto seleccionado
paper-ai-expand = IA Ampliar
paper-ai-expand-desc = Ampliar el texto seleccionado
paper-ai-improve = Mejora de la IA
paper-ai-improve-desc = Mejorar la calidad de la escritura
paper-ai-translate = Traductor AI
paper-ai-translate-desc = Traducir a otro idioma
paper-ai-assistant = Asistente de IA
paper-ai-quick-actions = Acciones rápidas
paper-ai-rewrite = Reescribir
paper-ai-make-shorter = Hacer más corto
paper-ai-make-longer = Hacer más largo
paper-ai-fix-grammar = Arreglar gramática
paper-ai-tone = tono
paper-ai-tone-professional = Profesional
paper-ai-tone-casual = Informal
paper-ai-tone-friendly = amigable
paper-ai-tone-formal = formales
paper-ai-translate-to = Traducir a
paper-ai-custom-prompt = Mensaje personalizado
paper-ai-custom-placeholder = Describe lo que quieres...
paper-ai-generate = generar
paper-ai-response = Respuesta de la IA
paper-ai-apply = Aplicar
paper-ai-regenerate = regenerar
paper-ai-copy = Copiar
paper-word-count = { $count } palabras
paper-char-count = { $count } caracteres
paper-saved = Guardado
paper-saving = Guardando...
paper-last-edited = Última edición: { $time }
paper-last-edited-now = Última edición: Justo ahora
paper-export = Exportar documento
paper-export-pdf = PDF
paper-export-docx = Palabra (.docx)
paper-export-markdown = Rebaja
paper-export-html = HTML
paper-export-txt = Texto sin formato

# Tasks translations
tasks-active = Intenciones activas
tasks-awaiting = En espera de decisión
tasks-paused = En pausa
tasks-blocked = Bloqueado/Problemas
tasks-time-saved = Tiempo activo ahorrado:
tasks-input-placeholder = ¿Qué te gustaría hacer? por ejemplo, "crear una aplicación CRM" o "recordarme que llame a John mañana"

# Calendar additional translations
calendar-my-calendars = Mis calendarios

# Email additional translations
email-scheduled = Programado
email-tracking = Seguimiento

# Email folder translations
email-inbox = Bandeja de entrada
email-starred = Destacado
email-sent = Enviado
email-drafts = Borradores
email-spam = spam
email-trash = basura
email-compose = componer

# -----------------------------------------------------------------------------
# Research
# -----------------------------------------------------------------------------
research-title = Investigación
research-search-placeholder = Pregunta cualquier cosa...
research-collections = Colecciones
research-new-collection = Nueva colección
research-recent = Reciente
research-academic = Académico
research-code = Código
research-internal = Interno
research-search-all = busca todo
research-academic-papers = Trabajos academicos
research-code-docs = Código y documentación
research-internal-kb = Base de conocimiento interna
research-sources = Fuentes
research-trending = Tendencia
research-pro-search = Búsqueda profesional
research-include-images = Incluir imágenes
research-try-asking = Intenta preguntar sobre
research-related = Preguntas relacionadas
research-view-all-sources = Ver todas las fuentes
research-export-citations = Exportar citas
research-save-to-collection = Guardar en colección

# -----------------------------------------------------------------------------
# Meet (additional keys)
# -----------------------------------------------------------------------------
meet-new-meeting = Nueva reunión
meet-join-meeting = Unirse a la reunión
meet-active-rooms = Salas activas
meet-room-title = Sala de Reuniones
meet-record = Grabar
meet-camera = cámara
meet-share = Compartir
meet-info = Información
meet-more = Más
meet-share-meeting = Compartir reunión
meet-meeting-title = Título de la reunión
meet-meeting-code = Código de reunión
meet-meeting-link = Enlace de reunión
meet-send-invite = Enviar invitación
