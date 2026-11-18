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

# ── #2307 Tasks goal/monitor detail panel ──────────────────────────
tasks-detail-untitled-goal = Objetivo sin título
tasks-detail-goal-progress = Progreso del objetivo
tasks-detail-current-value = Valor actual
tasks-detail-target-value = Valor objetivo
tasks-detail-percent = Progreso
tasks-detail-no-actions = Aún no hay acciones
tasks-detail-pending-info = Información pendiente
tasks-detail-scheduler-info = Información del programador
tasks-detail-monitor-info = Información del monitor
tasks-detail-unavailable = No disponible
tasks-detail-load-failed = No se pudieron cargar los detalles del objetivo


# ── #2307 Database insert/confirm dialogs ──────────────────────────
db-insert-row = Insertar fila
db-insert-into = Insertar en
db-provide-value = Proporcione al menos un valor
db-insert-failed = Error al insertar
db-confirm-title = Confirmar
db-confirm-accept = Confirmar
db-confirm-delete = ¿Eliminar esta fila?
db-confirm-drop = Eliminar tabla
db-delete-failed = Error al eliminar
db-drop-failed = Error al eliminar la tabla

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
tasks-load-error = No se pudieron cargar las tareas.
tasks-sign-in-to-load = Inicia sesión para cargar las tareas.
calendar-collapse-sidebar = Contraer barra lateral
calendar-prev-month = Mes anterior
calendar-next-month = Mes siguiente
calendar-prev-period = Período anterior
calendar-next-period = Período siguiente
calendar-add-calendar = Agregar calendario
paper-toggle-sidebar = Alternar barra lateral
paper-insert-ai-response = Insertar respuesta de IA
paper-replace-selection = Reemplazar selección
paper-highlight = Resaltar
paper-align-left = Alinear a la izquierda
paper-align-center = Centrar
paper-align-right = Alinear a la derecha
paper-insert-link = Insertar enlace
paper-insert-image = Insertar imagen
paper-insert-table = Insertar tabla
paper-bold = Negrita
paper-italic = Cursiva
paper-underline = Subrayado
paper-strikethrough = Tachado
products-view-grid = Vista de cuadrícula
products-view-list = Vista de lista
sheet-freeze-panes = Congelar paneles

# semantic i18n key (#2542)
db-schema-title = 🗄️ Esquema de la base de datos

# semantic i18n key (#2542)
db-loading-tables = Cargando tablas...

# semantic i18n key (#2542)
db-query-builder = Constructor de consultas

# semantic i18n key (#2542)
db-run-query = Ejecutar consulta

# semantic i18n key (#2542)
db-select-table-hint = Selecciona una tabla para ver los datos

# semantic i18n key (#2542)
db-prev-page = ◀ Anterior

# semantic i18n key (#2542)
db-page-info-default = Página 1 de 1

# semantic i18n key (#2542)
db-next-page = Siguiente ▶

# semantic i18n key (#2542)
research-searching-sources = Buscando fuentes...

# semantic i18n key (#2542)
research-suggestion-latest-ai = Últimos desarrollos en IA

# semantic i18n key (#2542)
research-suggestion-quantum = Computación cuántica explicada

# semantic i18n key (#2542)
research-suggestion-software-architecture = Arquitectura de software

# semantic i18n key (#2542)
research-suggestion-ai-productivity = Herramientas de productividad con IA

# semantic i18n key (#2542)
research-related-heading = Relacionado

# semantic i18n key (#2542)
research-loading = Cargando investigación…

# semantic i18n key (#2542)
research-service-unavailable = Servicio de investigación no disponible

# semantic i18n key (#2542)
calendar-mini-month-placeholder = Enero de 2025

# semantic i18n key (#2542)
calendar-upcoming = Próximos

# semantic i18n key (#2542)
calendar-import-export = Importar / Exportar

# semantic i18n key (#2542)
calendar-popup-event-title = Título del evento

# semantic i18n key (#2542)
calendar-popup-location-placeholder = Sala de conferencias A

# semantic i18n key (#2542)
calendar-popup-event-description = Descripción del evento

# semantic i18n key (#2542)
calendar-personal = Personal

# semantic i18n key (#2542)
calendar-import-ical-title = Importar archivo iCal

# semantic i18n key (#2542)
calendar-select-ics-file = Seleccionar archivo .ics

# semantic i18n key (#2542)
calendar-import-to-calendar-label = Importar al calendario

# semantic i18n key (#2542)
calendar-default-calendar = Calendario predeterminado

# semantic i18n key (#2542)
calendar-loading = Cargando calendario…

# semantic i18n key (#2542)
calendar-unavailable = Calendario no disponible

# semantic i18n key (#2542)
tasks-progress-status = ESTADO

# semantic i18n key (#2542)
tasks-progress-sample-title = Implementar sistema de autenticación de usuarios

# semantic i18n key (#2542)
tasks-progress-runtime = Tiempo de ejecución:

# semantic i18n key (#2542)
tasks-progress-sample-action = Elegir estrategia de expiración del token

# semantic i18n key (#2542)
tasks-progress-estimated = Estimado:

# semantic i18n key (#2542)
tasks-progress-sample-decision = Necesitará aprobación para la configuración de seguridad

# semantic i18n key (#2542)
tasks-progress-log = REGISTRO DE PROGRESO

# semantic i18n key (#2542)
tasks-progress-terminal-live = TERMINAL (ACTIVIDAD DEL AGENTE EN VIVO)

# semantic i18n key (#2542)
tasks-progress-processed = Procesado:

# semantic i18n key (#2542)
tasks-progress-data-points = puntos de datos

# semantic i18n key (#2542)
tasks-progress-sample-speed = ~8 fuentes/min

# semantic i18n key (#2542)
tasks-progress-sample-insight = Sintetizando información competitiva...

# semantic i18n key (#2542)
tasks-progress-sample-pricing = Cruzando datos de precios...

# semantic i18n key (#2542)
tasks-progress-view-details = Ver detalles ▸

# semantic i18n key (#2542)
tasks-create-run = Crear y ejecutar

# semantic i18n key (#2542)
tasks-loading-tasks = Cargando tareas...

# semantic i18n key (#2542)
tasks-select-task = Selecciona una tarea

# semantic i18n key (#2542)
tasks-select-task-desc = Haz clic en una tarea de la lista para ver los detalles

# semantic i18n key (#2542)
tasks-bot-database-label = Base de datos del bot:

# semantic i18n key (#2542)
tasks-shared-resources-label = Recursos compartidos:

# semantic i18n key (#2542)
tasks-starting = Iniciando...

# semantic i18n key (#2542)
tasks-create-new-intent = Crear nueva intención

# semantic i18n key (#2542)
tasks-intent-accomplish-label = ¿Qué te gustaría lograr?

# semantic i18n key (#2542)
tasks-execution-mode = Modo de ejecución

# semantic i18n key (#2542)
tasks-mode-automatic = Automático

# semantic i18n key (#2542)
tasks-mode-supervised = Supervisado

# semantic i18n key (#2542)
tasks-mode-manual-approval = Aprobación manual

# semantic i18n key (#2542)
tasks-make-decision = Tomar decisión

# semantic i18n key (#2542)
tasks-decision-required = Decisión requerida

# semantic i18n key (#2542)
tasks-loading-decision-details = Cargando detalles de la decisión...

# semantic i18n key (#2542)
tasks-skip-for-now = Omitir por ahora

# semantic i18n key (#2542)
tasks-confirm-decision = Confirmar decisión

# semantic i18n key (#2542)
tasks-due-soon = Vence pronto

# semantic i18n key (#2542)
tasks-autotask-items = // ELEMENTOS DE AUTOTAREA

# semantic i18n key (#2542)
tasks-edit-task = Editar tarea

# semantic i18n key (#2542)
tasks-assignee-user-id = Asignado (ID de usuario)

# semantic i18n key (#2542)
tasks-parent-task-subtask-of = Tarea principal (subtarea de…)

# semantic i18n key (#2542)
tasks-reopen-action = ↩ Reabrir

# semantic i18n key (#2542)
tasks-complete-action = ✓ Completar

# semantic i18n key (#2542)
tasks-delete-action = 🗑 Eliminar

# semantic i18n key (#2542)
db-schema-browser = Explorador de esquema

# semantic i18n key (#2542)
db-loading-schema = Cargando esquema...

# semantic i18n key (#2542)
db-new-table = + Nueva tabla

# semantic i18n key (#2542)
db-previous = ← Anterior

# semantic i18n key (#2542)
db-next = Siguiente →

# semantic i18n key (#2542)
db-sql-console = Consola SQL

# semantic i18n key (#2542)
db-execute-query = Ejecutar (Ctrl+Enter)

# semantic i18n key (#2542)
db-explain = EXPLAIN

# semantic i18n key (#2542)
db-create-new-table = Crear nueva tabla

# semantic i18n key (#2542)
db-table-name = Nombre de la tabla

# semantic i18n key (#2542)
db-columns = Columnas

# semantic i18n key (#2542)
db-not-null = NOT NULL

# semantic i18n key (#2542)
db-type-varchar-255 = VARCHAR(255)

# semantic i18n key (#2542)
db-type-text = TEXT

# semantic i18n key (#2542)
db-type-integer = INTEGER

# semantic i18n key (#2542)
db-type-bigint = BIGINT

# semantic i18n key (#2542)
db-type-decimal = DECIMAL

# semantic i18n key (#2542)
db-type-boolean = BOOLEAN

# semantic i18n key (#2542)
db-type-date = DATE

# semantic i18n key (#2542)
db-type-timestamptz = TIMESTAMPTZ

# semantic i18n key (#2542)
db-type-jsonb = JSONB

# semantic i18n key (#2542)
db-add-column = + Agregar columna

# semantic i18n key (#2542)
db-create-table = Crear tabla

# semantic i18n key (#2542)
db-edit-row = Editar fila

# semantic i18n key (#2542)
db-import-csv = Importar CSV

# semantic i18n key (#2542)
db-select-csv-file = Seleccionar archivo CSV

# semantic i18n key (#2542)
analytics-ask-placeholder = Pregunta sobre tus datos de análisis...
tasks-intent-description-placeholder = Describe lo que quieres crear o automatizar...
tasks-assignee-placeholder = UUID del responsable (opcional)
tasks-parent-placeholder = UUID de la tarea principal (opcional)

# semantic i18n key (#2542)
db-placeholder-search-tables = Buscar tablas...
db-aria-close-sql-console = Cerrar consola SQL
db-placeholder-sql-query = SELECT * FROM nombre_tabla WHERE condicion...
db-aria-sql-query = Consulta SQL
db-placeholder-table-name-example = mi_tabla
db-placeholder-column-id = id
db-placeholder-column-name-example = nombre_columna
