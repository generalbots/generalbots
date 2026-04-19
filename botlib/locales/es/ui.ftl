# =============================================================================
# General Bots - Traducciones de UI en Español
# =============================================================================

# -----------------------------------------------------------------------------
# Navegación
# -----------------------------------------------------------------------------
nav-home = Inicio
nav-chat = Chat
nav-drive = Archivos
nav-tasks = Tareas
nav-mail = Correo
nav-calendar = Calendario
nav-meet = Reuniones
nav-paper = Documentos
nav-research = Investigación
nav-analytics = Analíticas
nav-settings = Configuración
nav-admin = Administración
nav-monitoring = Monitoreo
nav-sources = Fuentes
nav-tools = Herramientas
nav-attendant = Asistente
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
nav-video = Video
nav-learn = Aprender

# -----------------------------------------------------------------------------
# Panel de Control
# -----------------------------------------------------------------------------
dashboard-title = Panel de Control
dashboard-welcome = ¡Bienvenido de nuevo, { $name }!
dashboard-quick-actions = Acciones Rápidas
dashboard-recent-activity = Actividad Reciente
dashboard-no-activity = Sin actividad reciente. ¡Comienza a explorar!
dashboard-analytics = Analíticas

# -----------------------------------------------------------------------------
# Acciones Rápidas
# -----------------------------------------------------------------------------
quick-start-chat = Iniciar Chat
quick-upload-files = Subir Archivos
quick-new-task = Nueva Tarea
quick-compose-email = Redactar Correo
quick-start-meeting = Iniciar Reunión
quick-new-event = Nuevo Evento

# -----------------------------------------------------------------------------
# Tarjetas de Aplicaciones
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
# Encabezado de Suite
# -----------------------------------------------------------------------------
suite-title = Suite General Bots
suite-tagline = Tu espacio de trabajo de productividad impulsado por IA. Chatea, colabora y crea.
suite-new-intent = Nueva Intención

# -----------------------------------------------------------------------------
# Panel de IA
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
# Chat
# -----------------------------------------------------------------------------
chat-title = Chat
chat-placeholder = Escribe tu mensaje...
chat-send = Enviar
chat-new-conversation = Nueva Conversación
chat-history = Historial de Chat
chat-clear = Limpiar Chat
chat-export = Exportar Chat
chat-typing = { $name } está escribiendo...
chat-online = En línea
chat-offline = Desconectado
chat-last-seen = Última vez { $time }
chat-mention-title = Referenciar Entidad
chat-mention-placeholder = Mensaje... (escribe @ para mencionar)
chat-mention-search = Buscar entidades...
chat-mention-no-results = No se encontraron resultados
chat-mention-type-hint = Escribe : para buscar

# -----------------------------------------------------------------------------
# Drive / Archivos
# -----------------------------------------------------------------------------
drive-title = Archivos
drive-upload = Subir
drive-new-folder = Nueva Carpeta
drive-empty = Sin archivos aún. ¡Sube algo!
drive-search = Buscar archivos...
drive-sort-name = Nombre
drive-sort-date = Fecha
drive-sort-size = Tamaño
drive-sort-type = Tipo
drive-view-grid = Vista de Cuadrícula
drive-view-list = Vista de Lista
drive-selected = { $count ->
    [one] { $count } elemento seleccionado
   *[other] { $count } elementos seleccionados
}
drive-file-size = { $size ->
    [bytes] { $value } B
    [kb] { $value } KB
    [mb] { $value } MB
    [gb] { $value } GB
   *[other] { $value } bytes
}
drive-drop-files = Arrastra archivos aquí para subir

# -----------------------------------------------------------------------------
# Tareas
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
    [zero] Sin tareas
    [one] { $count } tarea
   *[other] { $count } tareas
}

# -----------------------------------------------------------------------------
# Calendario
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
# Meet / Videoconferencias
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
    [one] { $count } participante
   *[other] { $count } participantes
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
# Correo / Email
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
    [one] { $count } sin leer
   *[other] { $count } sin leer
}
mail-empty-inbox = Tu bandeja de entrada está vacía
mail-no-subject = (Sin asunto)

# -----------------------------------------------------------------------------
# Configuración
# -----------------------------------------------------------------------------
settings-title = Configuración
settings-general = General
settings-account = Cuenta
settings-notifications = Notificaciones
settings-privacy = Privacidad
settings-security = Seguridad
settings-language = Idioma
settings-theme = Tema
settings-theme-light = Claro
settings-theme-dark = Oscuro
settings-theme-system = Sistema
settings-save = Guardar Cambios
settings-saved = Configuración guardada exitosamente
settings-timezone = Zona Horaria
settings-date-format = Formato de Fecha
settings-time-format = Formato de Hora

# -----------------------------------------------------------------------------
# Autenticación / Login
# -----------------------------------------------------------------------------
auth-login = Iniciar Sesión
auth-logout = Cerrar Sesión
auth-signup = Registrarse
auth-forgot-password = ¿Olvidaste tu Contraseña?
auth-reset-password = Restablecer Contraseña
auth-email = Correo Electrónico
auth-password = Contraseña
auth-confirm-password = Confirmar Contraseña
auth-remember-me = Recordarme
auth-login-success = Sesión iniciada exitosamente
auth-logout-success = Sesión cerrada exitosamente
auth-invalid-credentials = Correo o contraseña inválidos
auth-session-expired = Tu sesión ha expirado. Por favor inicia sesión nuevamente.

# -----------------------------------------------------------------------------
# Búsqueda
# -----------------------------------------------------------------------------
search-placeholder = Buscar...
search-no-results = No se encontraron resultados
search-results = { $count ->
    [one] { $count } resultado
   *[other] { $count } resultados
}
search-in-progress = Buscando...
search-advanced = Búsqueda Avanzada
search-filters = Filtros
search-clear-filters = Limpiar Filtros

# -----------------------------------------------------------------------------
# Paginación
# -----------------------------------------------------------------------------
pagination-previous = Anterior
pagination-next = Siguiente
pagination-first = Primera
pagination-last = Última
pagination-page = Página { $current } de { $total }
pagination-showing = Mostrando { $from } a { $to } de { $total }

# -----------------------------------------------------------------------------
# Tablas
# -----------------------------------------------------------------------------
table-no-data = No hay datos disponibles
table-loading = Cargando datos...
table-actions = Acciones
table-select-all = Seleccionar Todo
table-deselect-all = Deseleccionar Todo
table-export = Exportar
table-import = Importar

# -----------------------------------------------------------------------------
# Formularios
# -----------------------------------------------------------------------------
form-required = Requerido
form-optional = Opcional
form-submit = Enviar
form-reset = Restablecer
form-clear = Limpiar
form-uploading = Subiendo...
form-processing = Procesando...

# -----------------------------------------------------------------------------
# Modales / Diálogos
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
# Configuración - Idioma y Localización
# -----------------------------------------------------------------------------
settings-language = Idioma
settings-language-desc = Elige tu idioma preferido
settings-display-language = Idioma de Visualización
settings-language-affects = Afecta todo el texto en la aplicación
settings-date-format = Formato de Fecha
settings-date-format-desc = Cómo se muestran las fechas
settings-time-format = Formato de Hora
settings-time-format-desc = Reloj de 12 horas o 24 horas
settings-saved = Configuración guardada exitosamente
settings-language-changed = Idioma cambiado exitosamente
settings-reload-required = Se requiere recargar la página para aplicar cambios

# Configuración - Perfil
settings-profile = Configuración de Perfil
settings-profile-desc = Administra tu información personal y preferencias
settings-profile-photo = Foto de Perfil
settings-profile-photo-desc = Tu foto de perfil es visible para otros usuarios
settings-upload-photo = Subir Foto
settings-remove-photo = Eliminar
settings-basic-info = Información Básica
settings-display-name = Nombre para Mostrar
settings-username = Nombre de Usuario
settings-email-address = Correo Electrónico
settings-bio = Biografía
settings-bio-placeholder = Cuéntanos sobre ti...
settings-contact-info = Información de Contacto
settings-phone-number = Número de Teléfono
settings-location = Ubicación
settings-website = Sitio Web

# Configuración - Seguridad
settings-security = Configuración de Seguridad
settings-security-desc = Protege tu cuenta con seguridad mejorada
settings-change-password = Cambiar Contraseña
settings-change-password-desc = Actualiza tu contraseña regularmente para mejor seguridad
settings-current-password = Contraseña Actual
settings-new-password = Nueva Contraseña
settings-confirm-password = Confirmar Nueva Contraseña
settings-update-password = Actualizar Contraseña
settings-2fa = Autenticación de Dos Factores
settings-2fa-desc = Agrega una capa extra de seguridad a tu cuenta
settings-authenticator-app = Aplicación de Autenticación
settings-authenticator-desc = Usa una app de autenticación para códigos 2FA
settings-enable-2fa = Habilitar 2FA
settings-disable-2fa = Deshabilitar 2FA
settings-active-sessions = Sesiones Activas
settings-active-sessions-desc = Administra tus sesiones de inicio de sesión activas
settings-this-device = Este dispositivo
settings-terminate-session = Terminar
settings-terminate-all = Terminar Todas las Otras Sesiones

# Configuración - Apariencia
settings-appearance = Apariencia
settings-appearance-desc = Personaliza cómo se ve la aplicación
settings-theme-selection = Tema
settings-theme-selection-desc = Elige tu tema de color preferido
settings-theme-dark = Oscuro
settings-theme-light = Claro
settings-theme-blue = Azul
settings-theme-purple = Púrpura
settings-theme-green = Verde
settings-theme-orange = Naranja
settings-layout-preferences = Preferencias de Diseño
settings-compact-mode = Modo Compacto
settings-compact-mode-desc = Reduce el espaciado para más contenido
settings-show-sidebar = Mostrar Barra Lateral
settings-show-sidebar-desc = Siempre mostrar la barra de navegación
settings-animations = Animaciones
settings-animations-desc = Habilitar animaciones y transiciones de UI

# Configuración - Notificaciones
settings-notifications-title = Notificaciones
settings-notifications-desc = Controla cómo recibes notificaciones
settings-email-notifications = Notificaciones por Correo
settings-direct-messages = Mensajes Directos
settings-direct-messages-desc = Recibir correo para nuevos mensajes directos
settings-mentions = Menciones
settings-mentions-desc = Recibir correo cuando alguien te menciona
settings-weekly-digest = Resumen Semanal
settings-weekly-digest-desc = Obtén un resumen semanal de actividad
settings-marketing = Marketing
settings-marketing-desc = Recibir noticias y actualizaciones de productos
settings-push-notifications = Notificaciones Push
settings-enable-push = Habilitar Notificaciones Push
settings-enable-push-desc = Recibir notificaciones push del navegador
settings-notification-sound = Sonido
settings-notification-sound-desc = Reproducir sonido para notificaciones
settings-in-app-notifications = Notificaciones en la App

# Configuración - Almacenamiento
settings-storage = Almacenamiento
settings-storage-desc = Administra tu uso de almacenamiento
settings-storage-usage = Uso de Almacenamiento
settings-storage-used = { $used } de { $total } usado
settings-storage-upgrade = Mejorar Almacenamiento

# Configuración - Privacidad
settings-privacy-title = Privacidad
settings-privacy-desc = Controla tu configuración de privacidad
settings-data-collection = Recolección de Datos
settings-analytics = Analíticas
settings-analytics-desc = Ayúdanos a mejorar enviando datos de uso anónimos
settings-crash-reports = Reportes de Errores
settings-crash-reports-desc = Enviar reportes de errores automáticamente
settings-download-data = Descargar Tus Datos
settings-download-data-desc = Obtén una copia de todos tus datos
settings-delete-account = Eliminar Cuenta
settings-delete-account-desc = Eliminar permanentemente tu cuenta y todos los datos
settings-delete-account-warning = Esta acción no se puede deshacer

# Configuración - Facturación
settings-billing = Facturación
settings-billing-desc = Administra tu suscripción y métodos de pago
settings-current-plan = Plan Actual
settings-free-plan = Plan Gratuito
settings-pro-plan = Plan Pro
settings-enterprise-plan = Plan Empresarial
settings-upgrade-plan = Mejorar Plan
settings-payment-methods = Métodos de Pago
settings-add-payment = Agregar Método de Pago
settings-billing-history = Historial de Facturación

# -----------------------------------------------------------------------------
# Paper (Editor de Documentos)
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

# =============================================================================
# CRM
# =============================================================================

# -----------------------------------------------------------------------------
# CRM Navegación & General
# -----------------------------------------------------------------------------
crm-title = CRM
crm-pipeline = Pipeline
crm-leads = Leads
crm-opportunities = Oportunidades
crm-accounts = Cuentas
crm-contacts = Contactos
crm-activities = Actividades

# -----------------------------------------------------------------------------
# CRM Entidades
# -----------------------------------------------------------------------------
crm-lead = Lead
crm-lead-desc = Prospecto no calificado
crm-opportunity = Oportunidad
crm-opportunity-desc = Oportunidad de venta calificada
crm-account = Cuenta
crm-account-desc = Empresa u organización
crm-contact = Contacto
crm-contact-desc = Persona en una cuenta
crm-activity = Actividad
crm-activity-desc = Tarea, llamada o correo

# -----------------------------------------------------------------------------
# CRM Acciones
# -----------------------------------------------------------------------------
crm-qualify = Calificar
crm-convert = Convertir
crm-won = Ganado
crm-lost = Perdido
crm-new-lead = Nuevo Lead
crm-new-opportunity = Nueva Oportunidad
crm-new-account = Nueva Cuenta
crm-new-contact = Nuevo Contacto

# -----------------------------------------------------------------------------
# CRM Campos
# -----------------------------------------------------------------------------
crm-stage = Etapa
crm-value = Valor
crm-probability = Probabilidad
crm-close-date = Fecha de Cierre
crm-company = Empresa
crm-phone = Teléfono
crm-email = Correo
crm-source = Origen
crm-owner = Responsable

# -----------------------------------------------------------------------------
# CRM Etapas del Pipeline
# -----------------------------------------------------------------------------
crm-pipeline-new = Nuevo
crm-pipeline-contacted = Contactado
crm-pipeline-qualified = Calificado
crm-pipeline-proposal = Propuesta
crm-pipeline-negotiation = Negociación
crm-pipeline-closed-won = Cerrado Ganado
crm-pipeline-closed-lost = Cerrado Perdido

# -----------------------------------------------------------------------------
# CRM Estadísticas & Métricas
# -----------------------------------------------------------------------------
crm-subtitle = Gestionar leads, oportunidades y clientes
crm-stage-lead = Lead
crm-stage-qualified = Calificado
crm-stage-proposal = Propuesta
crm-stage-negotiation = Negociación
crm-stage-won = Ganado
crm-stage-lost = Perdido
crm-conversion-rate = Tasa de Conversión
crm-pipeline-value = Valor del Pipeline
crm-avg-deal = Valor Promedio
crm-won-month = Ganados Este Mes

# -----------------------------------------------------------------------------
# CRM Estados Vacíos
# -----------------------------------------------------------------------------
crm-no-leads = No se encontraron leads
crm-no-opportunities = No se encontraron oportunidades
crm-no-accounts = No se encontraron cuentas
crm-no-contacts = No se encontraron contactos
crm-drag-hint = Arrastra las tarjetas para cambiar la etapa

# =============================================================================
# Facturación
# =============================================================================

# -----------------------------------------------------------------------------
# Facturación Navegación & General
# -----------------------------------------------------------------------------
billing-title = Facturación
billing-invoices = Facturas
billing-payments = Pagos
billing-quotes = Cotizaciones
billing-dashboard = Panel

# -----------------------------------------------------------------------------
# Facturación Entidades
# -----------------------------------------------------------------------------
billing-invoice = Factura
billing-invoice-desc = Cobro al cliente
billing-payment = Pago
billing-payment-desc = Pago recibido
billing-quote = Cotización
billing-quote-desc = Cotización de precio

# -----------------------------------------------------------------------------
# Facturación Estado
# -----------------------------------------------------------------------------
billing-due-date = Fecha de Vencimiento
billing-overdue = Vencido
billing-paid = Pagado
billing-pending = Pendiente
billing-draft = Borrador
billing-sent = Enviado
billing-partial = Parcial
billing-cancelled = Cancelado

# -----------------------------------------------------------------------------
# Facturación Acciones
# -----------------------------------------------------------------------------
billing-new-invoice = Nueva Factura
billing-new-quote = Nueva Cotización
billing-new-payment = Nuevo Pago
billing-send-invoice = Enviar Factura
billing-record-payment = Registrar Pago
billing-mark-paid = Marcar como Pagado
billing-void = Anular

# -----------------------------------------------------------------------------
# Facturación Campos
# -----------------------------------------------------------------------------
billing-amount = Monto
billing-tax = Impuesto
billing-subtotal = Subtotal
billing-total = Total
billing-discount = Descuento
billing-line-items = Artículos
billing-add-item = Agregar Artículo
billing-remove-item = Eliminar Artículo
billing-customer = Cliente
billing-issue-date = Fecha de Emisión
billing-payment-terms = Términos de Pago
billing-notes = Notas
billing-invoice-number = Número de Factura
billing-quote-number = Número de Cotización

# -----------------------------------------------------------------------------
# Facturación Reportes
# -----------------------------------------------------------------------------
billing-revenue = Ingresos
billing-outstanding = Pendiente
billing-this-month = Este Mes
billing-last-month = Mes Pasado
billing-total-paid = Total Pagado
billing-total-overdue = Total Vencido
billing-subtitle = Facturas, pagos y cotizaciones
billing-revenue-month = Ingresos Este Mes
billing-total-revenue = Ingresos Totales
billing-paid-month = Pagado Este Mes

# -----------------------------------------------------------------------------
# Facturación Estados Vacíos
# -----------------------------------------------------------------------------
billing-no-invoices = No se encontraron facturas
billing-no-payments = No se encontraron pagos
billing-no-quotes = No se encontraron cotizaciones

# =============================================================================
# Productos
# =============================================================================

# -----------------------------------------------------------------------------
# Productos Navegación & General
# -----------------------------------------------------------------------------
products-title = Productos
products-catalog = Catálogo
products-services = Servicios
products-price-lists = Listas de Precios
products-inventory = Inventario

# -----------------------------------------------------------------------------
# Productos Entidades
# -----------------------------------------------------------------------------
products-product = Producto
products-product-desc = Producto físico o digital
products-service = Servicio
products-service-desc = Oferta de servicio
products-price-list = Lista de Precios
products-price-list-desc = Niveles de precios

# -----------------------------------------------------------------------------
# Productos Acciones
# -----------------------------------------------------------------------------
products-new-product = Nuevo Producto
products-new-service = Nuevo Servicio
products-new-price-list = Nueva Lista de Precios
products-new-pricelist = Nueva Lista de Precios
products-edit-product = Editar Producto
products-duplicate = Duplicar

# -----------------------------------------------------------------------------
# Productos Campos
# -----------------------------------------------------------------------------
products-sku = SKU
products-category = Categoría
products-price = Precio
products-unit = Unidad
products-stock = Stock
products-cost = Costo
products-margin = Margen
products-barcode = Código de Barras

# -----------------------------------------------------------------------------
# Productos Estado
# -----------------------------------------------------------------------------
products-in-stock = En Stock
products-out-of-stock = Sin Stock
products-low-stock = Stock Bajo
products-active = Activo
products-inactive = Inactivo
products-featured = Destacado
products-archived = Archivado

# -----------------------------------------------------------------------------
# Productos Estadísticas & Métricas
# -----------------------------------------------------------------------------
products-subtitle = Gestionar productos, servicios y precios
products-items = Productos
products-pricelists = Listas de Precios
products-total-products = Total de Productos
products-total-services = Total de Servicios

# -----------------------------------------------------------------------------
# Productos Estados Vacíos
# -----------------------------------------------------------------------------
products-no-products = No se encontraron productos
products-no-services = No se encontraron servicios
products-no-price-lists = No se encontraron listas de precios

# =============================================================================
# Tickets (Casos de Soporte)
# =============================================================================

# -----------------------------------------------------------------------------
# Tickets Navegación & General
# -----------------------------------------------------------------------------
tickets-title = Tickets
tickets-cases = Casos
tickets-open = Abiertos
tickets-closed = Cerrados
tickets-all = Todos los Tickets
tickets-my-tickets = Mis Tickets

# -----------------------------------------------------------------------------
# Tickets Entidades
# -----------------------------------------------------------------------------
tickets-case = Caso
tickets-case-desc = Ticket de soporte
tickets-resolution = Resolución
tickets-resolution-desc = Solución sugerida por IA

# -----------------------------------------------------------------------------
# Tickets Prioridad
# -----------------------------------------------------------------------------
tickets-priority = Prioridad
tickets-priority-low = Baja
tickets-priority-medium = Media
tickets-priority-high = Alta
tickets-priority-urgent = Urgente

# -----------------------------------------------------------------------------
# Tickets Estado
# -----------------------------------------------------------------------------
tickets-status = Estado
tickets-status-new = Nuevo
tickets-status-open = Abierto
tickets-status-pending = Pendiente
tickets-status-resolved = Resuelto
tickets-status-closed = Cerrado
tickets-status-on-hold = En Espera

# -----------------------------------------------------------------------------
# Tickets Acciones
# -----------------------------------------------------------------------------
tickets-new-ticket = Nuevo Ticket
tickets-assign = Asignar
tickets-reassign = Reasignar
tickets-escalate = Escalar
tickets-resolve = Resolver
tickets-reopen = Reabrir
tickets-close = Cerrar
tickets-merge = Fusionar

# -----------------------------------------------------------------------------
# Tickets Campos
# -----------------------------------------------------------------------------
tickets-subject = Asunto
tickets-description = Descripción
tickets-category = Categoría
tickets-assigned = Asignado a
tickets-unassigned = Sin Asignar
tickets-created = Creado
tickets-updated = Actualizado
tickets-response-time = Tiempo de Respuesta
tickets-resolution-time = Tiempo de Resolución
tickets-customer = Cliente
tickets-internal-notes = Notas Internas
tickets-attachments = Adjuntos

# -----------------------------------------------------------------------------
# Tickets Funciones de IA
# -----------------------------------------------------------------------------
tickets-ai-suggestion = Sugerencia de IA
tickets-apply-suggestion = Aplicar Sugerencia
tickets-ai-summary = Resumen de IA
tickets-similar-tickets = Tickets Similares
tickets-suggested-articles = Artículos Sugeridos

# -----------------------------------------------------------------------------
# Tickets Estados Vacíos
# -----------------------------------------------------------------------------
tickets-no-tickets = No se encontraron tickets
tickets-no-open = No hay tickets abiertos
tickets-no-closed = No hay tickets cerrados

# -----------------------------------------------------------------------------
# Security Module
# -----------------------------------------------------------------------------
security-title = Seguridad
security-subtitle = Herramientas de seguridad, escaneo de cumplimiento y protección del servidor
security-tab-compliance = Informe de Cumplimiento API
security-tab-protection = Protección
security-export-report = Exportar Informe
security-run-scan = Ejecutar Escaneo
security-critical = Crítico
security-critical-desc = Acción inmediata requerida
security-high = Alto
security-high-desc = Riesgo de seguridad
security-medium = Medio
security-medium-desc = Debe ser atendido
security-low = Bajo
security-low-desc = Mejores prácticas
security-info = Info
security-info-desc = Informativo
security-filter-severity = Severidad:
security-filter-all-severities = Todas las Severidades
security-filter-type = Tipo:
security-filter-all-types = Todos los Tipos
security-type-password = Contraseña en Config
security-type-hardcoded = Secretos Hardcodeados
security-type-deprecated = Palabras Clave Obsoletas
security-type-fragile = Código Frágil
security-type-config = Problemas de Configuración
security-results = Problemas de Cumplimiento
security-col-severity = Severidad
security-col-issue = Tipo de Problema
security-col-location = Ubicación
security-col-details = Descripción
security-col-action = Acción

# -----------------------------------------------------------------------------
# Learn Module
# -----------------------------------------------------------------------------
learn-title = Aprender
learn-my-progress = Mi Progreso
learn-completed = Completados
learn-in-progress = En Progreso
learn-certificates = Certificados
learn-time-spent = Tiempo Invertido
learn-categories = Categorías
learn-all-courses = Todos los Cursos
learn-mandatory = Obligatorio
learn-compliance = Cumplimiento
learn-security = Seguridad
learn-skills = Habilidades
learn-onboarding = Incorporación
learn-difficulty = Dificultad
learn-my-certificates = Mis Certificados
learn-view-all = Ver Todo

# -----------------------------------------------------------------------------
# Workspace Module
# -----------------------------------------------------------------------------
workspace-title = Espacio de Trabajo
workspace-search-pages = Buscar páginas...
workspace-recent = Recientes
workspace-favorites = Favoritos
workspace-pages = Páginas
workspace-templates = Plantillas
workspace-trash = Papelera
workspace-settings = Configuración

# -----------------------------------------------------------------------------
# Player Module
# -----------------------------------------------------------------------------
player-title = Reproductor de Medios
player-no-file = Ningún archivo seleccionado
player-search = Buscar archivos...
player-recent = Recientes
player-files = Archivos

# -----------------------------------------------------------------------------
# Goals Module
# -----------------------------------------------------------------------------
goals-title = Metas y OKRs
goals-dashboard = Panel
goals-objectives = Objetivos
goals-alignment = Alineación
goals-ai-suggestions = Sugerencias de IA

crm-email = Correo
crm-compose-email = Redactar Correo
crm-send-email = Enviar Correo
mail-snooze = Posponer
mail-snooze-later-today = Más tarde hoy (18:00)
mail-snooze-tomorrow = Mañana (08:00)
mail-snooze-next-week = Próxima semana (Lun 08:00)
mail-crm-log = Registrar en CRM
mail-crm-create-lead = Crear Lead
mail-add-to-list = Agregar a Lista
campaign-send-email = Enviar Correo
