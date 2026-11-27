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

# semantic i18n key (#2542)
apps-menu-placeholder-search = Buscar aplicaciones...
apps-menu-aria-search = Buscar aplicaciones
apps-menu-aria-close = Cerrar menú de aplicaciones

# --- #2921 missing keys added to every locale catalogue ---
palette-kind-tour = Recorrido
palette-show-tour = Mostrar recorrido
palette-show-tour-hint = Mostrar el recorrido del producto
sidebar-app-hide = Ocultar { $app } de esta barra
sidebar-apps-count-off = { $count } ocultos
sidebar-apps-empty = No hay aplicaciones para mostrar
sidebar-apps-manage = Gestionar aplicaciones
sidebar-apps-manage-all = Mostrar todos
sidebar-apps-manage-close = Cerrar
sidebar-apps-manage-empty = No hay aplicaciones disponibles
sidebar-apps-manage-summary = { $on } de { $total } mostrados
sidebar-apps-none = Ninguna aplicación
toast-open = Abrir

# Assistant shell — knowledge-base source blocks
trecho = fragmento
trechos = fragmentos

# Assistant/Drive shell keys integrated from the gbgov edition
arquivo = file
arquivos = files
assistant-agent-backend-off = Agent backend is off
assistant-requires-agent-mode = Requiere el modo agente
assistant-agent-mode-disabled = Debe instalar el modo agente para usar agentes de suscripción.
assistant-agent-locked-hint = Agent is locked for this model
assistant-agent-locked-tip = Choose another model to use the agent
assistant-regenerate-older = Regenerating an older answer starts a new turn
caracteres = characters
corrigido = fixed
de = of
drive-actions = Actions
drive-folders = Folders
drive-usage-hint = Storage used
drive-used = Used
item = item
itens = items
linha = line
linhas = lines
resultado = result
resultados = results
shell-close = Close
shell-conversations = Conversations
shell-machine = Machine

# Assistant shell — literal keys
assistant-vm-wants = The agent wants to perform an action
assistant-vm-wants-command = The agent wants to run a command on the machine
drive-label = Drive

# Chat / assistant parity — VM consent and subscription-agent UI keys
chat-vm-consent-assistant = Asistente
chat-vm-consent-subject = Agente
chat-vm-consent-wants = El agente quiere realizar una acción
chat-vm-consent-wants-command = El agente quiere ejecutar un comando
chat-vm-consent-kicker = Necesita tu permiso
chat-vm-consent-command-label = Comando
chat-vm-consent-where = Dónde
chat-vm-consent-where-vm = Máquina virtual
chat-vm-consent-where-drive = tu Drive
chat-vm-consent-destination = Destino
chat-vm-consent-language = Lenguaje
chat-vm-consent-packages = Instala desde internet
chat-vm-consent-from = Origen
chat-vm-consent-background = Se ejecuta en segundo plano
chat-vm-consent-repair = Versión corregida del programa
chat-vm-consent-protected = Esta solicitud implica las credenciales del agente; solo se puede permitir una vez.
chat-vm-consent-all-note = Permitir todo cubre todas las acciones de este agente en esta conversación, solo dentro de la máquina virtual.
chat-vm-consent-outside-note = Esta acción sale de la máquina virtual: permitir todo no la cubre.
chat-vm-consent-allow-once = Permitir una vez
chat-vm-consent-allow-conversation = Permitir en esta conversación
chat-vm-consent-allow-all = Permitir todo
chat-vm-consent-deny = Denegar
chat-vm-consent-outcome-all = Permiso total en esta conversación
chat-vm-consent-outcome-conversation = Permitido en esta conversación
chat-vm-consent-outcome-once = Permitido una vez
chat-vm-consent-outcome-denied = Denegado por ti: no se ejecutó nada
chat-vm-consent-outcome-failed = No se pudo registrar tu decisión
chat-vm-consent-forbidden = Esta decisión no se pudo registrar aquí. La solicitud sigue abierta.
chat-vm-consent-announce = Solicitud de permiso abierta. Pulsa Alt+P para responder.
chat-vm-consent-still-open = Hay una solicitud de permiso abierta. Elige una opción; Denegar no ejecuta nada.
chat-vm-consent-waiting = Esperando tu permiso
chat-vm-all-chip = Permiso total
chat-vm-all-scope = solo dentro de la máquina virtual
chat-vm-all-revoke = Revocar
chat-vm-all-revoked = Permiso total revocado
chat-vm-send-head = Confirmación
chat-vm-send-text = Enviar ahora cancela la solicitud de permiso abierta. ¿Enviar de todos modos?
chat-vm-send-back = Volver a la solicitud
chat-vm-send-go = Enviar
chat-vm-engine-local-short = Local
chat-vm-engine-local-group = Modelos locales
chat-vm-engine-local-detail = En este equipo
chat-vm-engine-free-short = Gratis
chat-vm-engine-free-group = Modelo gratuito
chat-vm-engine-free-detail = Modelo gratuito
chat-vm-engine-own-short = Clave propia
chat-vm-engine-own-group = Claves propias
chat-vm-engine-own-detail = Servidor o clave propia
chat-vm-engine-sub-short = Suscripción
chat-vm-engine-sub-group = Suscripciones
chat-vm-engine-sub-detail = Agente por suscripción
chat-vm-agent-locked-tip = El agente seleccionado se ejecuta en la máquina virtual; elige otro modelo para desactivarlo
chat-vm-agent-locked-note = se ejecuta en la máquina virtual, por lo que el interruptor del Agente permanece activado mientras esté seleccionado. Elige otro modelo para desactivarlo.
assistant-no-update = No hay actualizaciones disponibles
