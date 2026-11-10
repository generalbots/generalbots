# =============================================================================
# General Bots — es — Desktop shell, navigation, controls
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# General Bots - English UI Translations
# =============================================================================
# -----------------------------------------------------------------------------
# Navigation
# -----------------------------------------------------------------------------
nav-home = Inicio
nav-chat = Chat
nav-drive = Archivos
nav-tasks = Tareas
nav-mail = Correo
nav-calendar = Calendario
nav-meet = Reuniones
nav-paper = Documentos
nav-video = Video
nav-research = Investigación
nav-analytics = Analíticas
nav-settings = Configuración
nav-admin = Administración
nav-monitoring = Monitoreo
nav-sources = Fuentes
nav-tools = Herramientas
nav-attendant = Asistente
nav-learn = Aprender
nav-crm = CRM
nav-billing = Facturación
nav-products = Productos
nav-tickets = Tickets
nav-docs = Documentos
nav-sheet = Hojas de Cálculo
nav-slides = Presentaciones
nav-social = Social
nav-all-apps = Todas las Apps
nav-people = Personas
nav-editor = Editor
nav-dashboards = Paneles
nav-security = Seguridad
nav-designer = Diseñador
nav-project = Proyecto
nav-canvas = Canvas
nav-goals = Metas
nav-player = Reproductor
nav-workspace = Espacio de Trabajo

# -----------------------------------------------------------------------------
# Dashboard
# -----------------------------------------------------------------------------
dashboard-title = Panel de Control
dashboard-welcome = ¡Bienvenido de nuevo, { $name }!
dashboard-quick-actions = Acciones Rápidas
dashboard-recent-activity = Actividad Reciente
dashboard-no-activity = Sin actividad reciente. ¡Comienza a explorar!
dashboard-analytics = Analíticas

# -----------------------------------------------------------------------------
# Quick Actions
# -----------------------------------------------------------------------------
quick-start-chat = Iniciar Chat
quick-upload-files = Subir Archivos
quick-new-task = Nueva Tarea
quick-compose-email = Redactar Correo
quick-start-meeting = Iniciar Reunión
quick-new-event = Nuevo Evento

# -----------------------------------------------------------------------------
# Application Cards
# -----------------------------------------------------------------------------
app-chat-name = Chat
app-chat-desc = Conversaciones impulsadas por IA. Haz preguntas, obtén ayuda y automatiza tareas.
app-drive-name = Archivos
app-drive-desc = Almacenamiento en la nube para todos tus archivos. Sube, organiza y comparte.
app-tasks-name = Tareas
app-tasks-desc = Mantente organizado con listas de tareas, prioridades y fechas límite.
app-mail-name = Correo
app-mail-desc = Cliente de correo con escritura asistida por IA y organización inteligente.
app-calendar-name = Calendario
app-calendar-desc = Programa reuniones, eventos y administra tu tiempo efectivamente.
app-meet-name = Reuniones
app-meet-desc = Videoconferencias con pantalla compartida y transcripción en vivo.
app-paper-name = Documentos
app-paper-desc = Escribe documentos con asistencia de IA. Notas, informes y más.
app-research-name = Investigación
app-research-desc = Búsqueda y descubrimiento impulsado por IA en todas tus fuentes.
app-analytics-name = Analíticas
app-analytics-desc = Paneles e informes para seguir el uso y obtener insights.

# -----------------------------------------------------------------------------
# Suite Header
# -----------------------------------------------------------------------------
suite-title = Suite General Bots
suite-tagline = Tu espacio de trabajo de productividad impulsado por IA. Chatea, colabora y crea.
suite-new-intent = Nueva Intención

# -----------------------------------------------------------------------------
# AI Panel
# -----------------------------------------------------------------------------
ai-developer = Desarrollador IA
ai-developing = Desarrollando: { $project }
ai-quick-actions = Acciones Rápidas
ai-add-field = Agregar campo
ai-change-color = Cambiar color
ai-add-validation = Agregar validación
ai-export-data = Exportar datos
ai-placeholder = Escribe tus modificaciones...
ai-thinking = La IA está pensando...
ai-status-online = En línea
ai-status-offline = Desconectado

# -----------------------------------------------------------------------------
# Search
# -----------------------------------------------------------------------------
search-placeholder = Buscar...
search-no-results = No se encontraron resultados
search-results = { $count ->
    [one] { $count } result
   *[other] { $count } results
}
search-in-progress = Buscando...
search-advanced = Búsqueda Avanzada
search-filters = Filtros
search-clear-filters = Limpiar Filtros

# -----------------------------------------------------------------------------
# Pagination
# -----------------------------------------------------------------------------
# -----------------------------------------------------------------------------
# Tables
# -----------------------------------------------------------------------------
table-no-data = No hay datos disponibles
table-loading = Cargando datos...
table-actions = Acciones
table-select-all = Seleccionar Todo
table-deselect-all = Deseleccionar Todo
table-export = Exportar
table-import = Importar

# -----------------------------------------------------------------------------
# Forms
# -----------------------------------------------------------------------------
form-required = Requerido
form-optional = Opcional
form-submit = Enviar
form-reset = Restablecer
form-clear = Limpiar
form-uploading = Subiendo...
form-processing = Procesando...

# -----------------------------------------------------------------------------
# Modals / Dialogs
# -----------------------------------------------------------------------------
modal-confirm-title = Confirmar Acción
modal-confirm-message = ¿Estás seguro de que deseas continuar?
modal-delete-title = Confirmación de Eliminación
modal-delete-message = Esta acción no se puede deshacer. ¿Estás seguro?

# -----------------------------------------------------------------------------
# Tooltips
# -----------------------------------------------------------------------------
tooltip-copy = Copiar al portapapeles
tooltip-copied = ¡Copiado!
tooltip-expand = Expandir
tooltip-collapse = Contraer
tooltip-refresh = Actualizar
tooltip-download = Descargar
tooltip-upload = Subir
tooltip-print = Imprimir
tooltip-fullscreen = Pantalla Completa
tooltip-exit-fullscreen = Salir de Pantalla Completa

# -----------------------------------------------------------------------------
# Common Labels (additional)
# -----------------------------------------------------------------------------
label-username = Nombre de usuario
label-email = Correo electrónico
label-display-name = Nombre para mostrar
label-password = Contraseña
label-role = Rol
label-group-name = Nombre del grupo
label-hostname = Nombre de host
label-record-type = Tipo de registro
label-target = Objetivo
label-your-name = Tu nombre

# -----------------------------------------------------------------------------
# Actions (additional)
# -----------------------------------------------------------------------------
action-register = Registrarse

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
notifications-title = Notificaciones
notifications-clear = Borrar todo
notifications-empty = Sin notificaciones

# #1853 — desktop chrome, launchers and notification settings.
window-ready = Listo
window-loading = Cargando aplicación…
window-minimize = Minimizar
window-maximize = Maximizar
window-fullscreen = Pantalla completa
window-exit-fullscreen = Salir de pantalla completa
window-close = Cerrar
app-loading = Cargando aplicación…
start-menu-search-placeholder = Escribe para encontrar una aplicación…
start-menu-recent = Recientes
start-menu-no-apps = No se encontraron aplicaciones
start-menu-user = Usuario
category-ai = IA
category-business = Negocios
category-office = Oficina
category-dev = Desarrollo
category-system = Sistema
category-tools = Herramientas
sidebar-apps = Aplicaciones
sidebar-apps-toggle = Mostrar/ocultar aplicaciones
sidebar-apps-toggle-aria = Alternar panel de aplicaciones
sidebar-apps-filter = Filtrar aplicaciones…
sidebar-apps-filter-aria = Filtrar aplicaciones
sidebar-preview = Modo de vista previa
sidebar-preview-hint = Mostrar las aplicaciones que aún están en vista previa
sidebar-preview-toggle = Alternar modo de vista previa
sidebar-app-preview-title = {app} (vista previa)
sidebar-apps-count = {count} aplicaciones
sidebar-apps-count-preview = {count} (+ {preview} en vista previa)
spotlight-placeholder = Busca aplicaciones, archivos… o escribe un objetivo para el Concierge
spotlight-hint = ↑↓ navegar · Enter ejecutar · Ctrl+Space o Esc cerrar
spotlight-empty = Sin resultados
spotlight-goal = Ejecutar objetivo: “{goal}”
spotlight-goal-subtitle = Concierge · planificar y ejecutar entre aplicaciones
palette-placeholder = Busca aplicaciones y comandos. O escribe @ <comando> para enviar al asistente...
palette-empty = No hay aplicaciones ni comandos coincidentes
palette-kind-app = Aplicación
palette-hint = ↑↓ navegar · Enter ejecutar · Esc cerrar · Ctrl+K alternar
palette-open-app = Abrir {app}
palette-value-prompt = {name}: {summary} — valor para {key}:
palette-execute = Ejecutar {method} {path} y resumir el resultado.
palette-use-command = Usa el comando {command} para {summary}.
toast-dismiss = Descartar
settings-language-display = Idioma de visualización
settings-language-display-desc = Elige el idioma de la interfaz de la aplicación
settings-language-regional = Configuración regional
settings-language-regional-desc = Configura los formatos de fecha, hora y números
settings-language-date-format = Formato de fecha
settings-language-time-format = Formato de hora
settings-language-timezone = Zona horaria
settings-language-first-day = Primer día de la semana
settings-language-time-12h = 12 horas (1:30 PM)
settings-language-time-24h = 24 horas (13:30)
settings-language-sunday = Domingo
settings-language-monday = Lunes
settings-email-notifications-desc = Enviadas por el remitente de notificaciones del espacio de trabajo, nunca por una cuenta de Gmail/Outlook conectada.
settings-push-enable = Activar notificaciones push
settings-push-enable-desc = Recibe notificaciones push del navegador
settings-push-sound = Sonido
settings-push-sound-desc = Reproducir sonido para las notificaciones
settings-desktop-notifications = Notificaciones de escritorio
settings-desktop-notifications-desc = Mostrar notificaciones en el escritorio
settings-badge-count = Contador de insignia
settings-badge-count-desc = Mostrar la insignia de no leídos
settings-save-preferences = Guardar preferencias
settings-notifications-saved = Preferencias de notificación guardadas
settings-notification-recipient = Buzón de entrega
settings-notification-recipient-desc = Déjalo en blanco para usar tu cuenta principal de Mail o el correo del perfil.
language-search-placeholder = Buscar idiomas…
settings-profile-title = Configuración de Perfil
settings-security-title = Configuración de Seguridad
