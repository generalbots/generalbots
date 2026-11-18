# =============================================================================
# General Bots — es — Admin, monitoring and system apps
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Admin Panel (additional UI keys)
# -----------------------------------------------------------------------------
admin-panel-title = Panel de administración

# =============================================================================
# MONITORING SCREENS
# =============================================================================
# -----------------------------------------------------------------------------
# Monitoring Dashboard
# -----------------------------------------------------------------------------
monitoring-title = Panel de monitoreo
monitoring-toggle-view = Alternar vista
monitoring-last-updated = Última actualización
monitoring-live-view = Vista en vivo
monitoring-grid-view = Vista de cuadrícula

# -----------------------------------------------------------------------------
# Monitoring Panels
# -----------------------------------------------------------------------------
monitoring-sessions = Sesiones
monitoring-messages = Mensajes
monitoring-resources = Recursos
monitoring-services = Servicios
monitoring-active-bots = Bots activos
monitoring-loading = Cargando...

# -----------------------------------------------------------------------------
# Service Status
# -----------------------------------------------------------------------------
monitoring-status-running = corriendo
monitoring-status-warning = Advertencia
monitoring-status-stopped = Detenido
monitoring-status-healthy = saludable
monitoring-status-degraded = degradado
monitoring-status-down = abajo

# -----------------------------------------------------------------------------
# Resource Metrics
# -----------------------------------------------------------------------------
monitoring-cpu = CPU
monitoring-memory = Memoria
monitoring-disk = disco
monitoring-network = Red
monitoring-requests-per-sec = Solicitudes/seg
monitoring-active-connections = Conexiones activas
monitoring-uptime = tiempo de actividad

# -----------------------------------------------------------------------------
# Logs
# -----------------------------------------------------------------------------
monitoring-logs-title = Registros del sistema
monitoring-logs-filter = Filtrar registros
monitoring-logs-level = Nivel de registro
monitoring-logs-all = Todos los niveles
monitoring-logs-debug = Depurar
monitoring-logs-info = Información
monitoring-logs-warning = Advertencia
monitoring-logs-error = error
monitoring-logs-critical = Crítico
monitoring-logs-search = Buscar registros...
monitoring-logs-no-results = No se encontraron registros

# -----------------------------------------------------------------------------
# Health
# -----------------------------------------------------------------------------
monitoring-health-title = Estado del sistema
monitoring-health-status = Estado de salud
monitoring-health-services = Estado del servicio
monitoring-health-database = Base de datos
monitoring-health-cache = caché
monitoring-health-queue = Cola de mensajes
monitoring-health-storage = Almacenamiento
monitoring-health-external = Servicios externos

# -----------------------------------------------------------------------------
# Metrics
# -----------------------------------------------------------------------------
monitoring-metrics-title = Métricas de rendimiento
monitoring-metrics-response-time = Tiempo de respuesta
monitoring-metrics-throughput = Rendimiento
monitoring-metrics-error-rate = Tasa de errores
monitoring-metrics-latency = Latencia

# -----------------------------------------------------------------------------
# Alerts
# -----------------------------------------------------------------------------
monitoring-alerts-title = Alertas del sistema
monitoring-alerts-active = Alertas activas
monitoring-alerts-resolved = Resuelto
monitoring-alerts-all = Todas las alertas
monitoring-alert-severity = Gravedad
monitoring-alert-critical = Crítico
monitoring-alert-high = Alto
monitoring-alert-medium = Medio
monitoring-alert-low = Bajo
monitoring-alert-info = Información
monitoring-alert-acknowledge = Reconocer
monitoring-alert-resolve = resolver
monitoring-no-alerts = No hay alertas activas

# -----------------------------------------------------------------------------
# Security Module
# -----------------------------------------------------------------------------
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

# semantic i18n key (#2542)
admin-ops-dashboard-title = Panel de operaciones

# semantic i18n key (#2542)
admin-ops-dashboard-subtitle = Métricas del sistema en tiempo real, rastreo distribuido y supervisión del rendimiento

# semantic i18n key (#2542)
admin-ops-auto-refresh-on = Actualización automática: ACTIVADA

# semantic i18n key (#2542)
admin-ops-avg-response = Respuesta promedio

# semantic i18n key (#2542)
admin-ops-requests-min = Solicitudes/min

# semantic i18n key (#2542)
admin-ops-disk-io = E/S de disco

# semantic i18n key (#2542)
admin-ops-iops-read-write = IOPS (lectura/escritura)

# semantic i18n key (#2542)
admin-ops-in-95-out-50 = Entrada: 95 / Salida: 50

# semantic i18n key (#2542)
admin-ops-stable = Estable

# semantic i18n key (#2542)
admin-ops-request-metrics = Métricas de solicitudes

# semantic i18n key (#2542)
admin-ops-errors-label = Errores

# semantic i18n key (#2542)
admin-ops-distributed-tracing = Rastreo distribuido

# semantic i18n key (#2542)
admin-ops-all-traces = Todos los rastros

# semantic i18n key (#2542)
admin-ops-errors-only = Solo errores

# semantic i18n key (#2542)
admin-ops-api-requests = Solicitudes de API

# semantic i18n key (#2542)
admin-ops-post-api-chat-message = POST /api/chat/message

# semantic i18n key (#2542)
admin-ops-trace-a1b2c3d4e5f6 = trace_a1b2c3d4e5f6

# semantic i18n key (#2542)
admin-ops-get-api-kb-search = GET /api/kb/search

# semantic i18n key (#2542)
admin-ops-trace-b2c3d4e5f6g7 = trace_b2c3d4e5f6g7

# semantic i18n key (#2542)
admin-ops-post-api-billing-checkout = POST /api/billing/checkout

# semantic i18n key (#2542)
admin-ops-trace-c3d4e5f6g7h8 = trace_c3d4e5f6g7h8

# semantic i18n key (#2542)
admin-ops-get-api-user-profile = GET /api/user/profile

# semantic i18n key (#2542)
admin-ops-trace-d4e5f6g7h8i9 = trace_d4e5f6g7h8i9

# semantic i18n key (#2542)
admin-ops-api-gateway = Puerta de enlace de API

# semantic i18n key (#2542)
admin-ops-primary-2-replicas = Primario + 2 réplicas

# semantic i18n key (#2542)
admin-ops-qdrant-vector-db = Qdrant (BD vectorial)

# semantic i18n key (#2542)
admin-ops-redis-cache = Caché Redis

# semantic i18n key (#2542)
admin-ops-cluster-mode-3-shards = Modo clúster | 3 fragmentos

# semantic i18n key (#2542)
admin-ops-llm-service = Servicio de LLM

# semantic i18n key (#2542)
admin-ops-openai-gpt-4-high-latency = OpenAI GPT-4 | Alta latencia

# semantic i18n key (#2542)
admin-ops-object-storage = Almacenamiento de objetos

# semantic i18n key (#2542)
admin-ops-s3-compatible = Compatible con S3

# semantic i18n key (#2542)
admin-ops-error-tracking = Seguimiento de errores

# semantic i18n key (#2542)
admin-ops-view-all-errors = Ver todos los errores

# semantic i18n key (#2542)
admin-ops-warnings = Advertencias

# semantic i18n key (#2542)
admin-ops-payment-processing-failed-stripe-timeout = Error al procesar el pago: tiempo de espera de la API de Stripe agotado

# semantic i18n key (#2542)
admin-ops-billing-checkout-rs-145 = billing/checkout.rs:145

# semantic i18n key (#2542)
admin-ops-rate-limit-approaching-openai = El límite de velocidad se acerca para la API de OpenAI

# semantic i18n key (#2542)
admin-ops-llm-openai-rs-89 = llm/openai.rs:89

# semantic i18n key (#2542)
admin-ops-core-analytics-rs-234 = core/analytics.rs:234

# semantic i18n key (#2542)
admin-ops-cache-miss-rate-above-threshold = Tasa de fallos de caché por encima del umbral

# semantic i18n key (#2542)
admin-ops-cache-redis-rs-67 = cache/redis.rs:67

# semantic i18n key (#2542)
admin-ops-endpoint-performance = Rendimiento de endpoints

# semantic i18n key (#2542)
admin-ops-by-requests = Por solicitudes

# semantic i18n key (#2542)
admin-ops-by-latency = Por latencia

# semantic i18n key (#2542)
admin-ops-by-error-rate = Por tasa de error

# semantic i18n key (#2542)
admin-ops-post = POST

# semantic i18n key (#2542)
admin-ops-api-chat-message = /api/chat/message

# semantic i18n key (#2542)
admin-ops-req-min = req/min

# semantic i18n key (#2542)
admin-ops-errors-metric = errores

# semantic i18n key (#2542)
admin-ops-api-kb-search = /api/kb/search

# semantic i18n key (#2542)
admin-roles-title = Gestión de roles

# semantic i18n key (#2542)
admin-roles-subtitle = Configura roles y asigna permisos para controlar el acceso en toda la suite

# semantic i18n key (#2542)
admin-roles-sidebar-title = Roles

# semantic i18n key (#2542)
admin-roles-system-filter = Roles del sistema

# semantic i18n key (#2542)
admin-roles-custom-filter = Roles personalizados

# semantic i18n key (#2542)
admin-roles-select-title = Selecciona un rol

# semantic i18n key (#2542)
admin-roles-select-hint = Elige un rol de la lista para ver y administrar sus permisos

# semantic i18n key (#2542)
admin-roles-name-label = Nombre del rol

# semantic i18n key (#2542)
admin-roles-name-badge-placeholder = role_name

# semantic i18n key (#2542)
admin-roles-description-placeholder = La descripción del rol va aquí

# semantic i18n key (#2542)
admin-roles-permissions-label = Permisos

# semantic i18n key (#2542)
admin-roles-assigned-users-tab = Usuarios asignados

# semantic i18n key (#2542)
admin-roles-assigned-groups-tab = Grupos asignados

# semantic i18n key (#2542)
admin-roles-permission-assignment = Asignación de permisos

# semantic i18n key (#2542)
admin-roles-permission-assignment-help = Usa la lista dual para asignar permisos a este rol

# semantic i18n key (#2542)
admin-roles-available-permissions = Permisos disponibles

# semantic i18n key (#2542)
admin-roles-category-documents = Documentos

# semantic i18n key (#2542)
admin-roles-category-spreadsheets = Hojas de cálculo

# semantic i18n key (#2542)
admin-roles-category-presentations = Presentaciones

# semantic i18n key (#2542)
admin-roles-category-meetings = Reuniones

# semantic i18n key (#2542)
admin-roles-category-ai-bots = IA y bots

# semantic i18n key (#2542)
admin-roles-category-automation = Automatización

# semantic i18n key (#2542)
admin-roles-assigned-permissions = Permisos asignados

# semantic i18n key (#2542)
admin-roles-no-permissions-assigned = No hay permisos asignados

# semantic i18n key (#2542)
admin-roles-reset-to-default = Restablecer a valores predeterminados

# semantic i18n key (#2542)
admin-roles-users-with-role = Usuarios con este rol

# semantic i18n key (#2542)
admin-roles-groups-with-role = Grupos con este rol

# semantic i18n key (#2542)
admin-roles-change-history = Historial de cambios de rol

# semantic i18n key (#2542)
admin-roles-create-new-role = Crear nuevo rol

# semantic i18n key (#2542)
admin-roles-name-help = Minúsculas, sin espacios (usa guiones bajos)

# semantic i18n key (#2542)
admin-roles-copy-permissions-from = Copiar permisos de

# semantic i18n key (#2542)
admin-roles-start-with-no-permissions = Comenzar sin permisos

# semantic i18n key (#2542)
admin-roles-standard-user = Usuario estándar

# semantic i18n key (#2542)
admin-roles-power-user = Usuario avanzado

# semantic i18n key (#2542)
admin-roles-viewer = Visualizador

# semantic i18n key (#2542)
admin-roles-guest-user = Usuario invitado

# semantic i18n key (#2542)
admin-roles-create-role-btn = Crear rol

# semantic i18n key (#2542)
admin-roles-delete-role-title = Eliminar rol

# semantic i18n key (#2542)
admin-roles-delete-warning = Esto eliminará el rol de todos los usuarios y grupos. Esta acción no se puede deshacer.

# semantic i18n key (#2542)
admin-roles-assign-to-users = Asignar rol a usuarios

# semantic i18n key (#2542)
admin-roles-search-users-label = Buscar usuarios

# semantic i18n key (#2542)
admin-roles-type-to-search-users = Escribe para buscar usuarios

# semantic i18n key (#2542)
admin-roles-expiration-optional = Vencimiento (opcional)

# semantic i18n key (#2542)
admin-roles-expiration-help = Déjalo vacío para una asignación permanente

# semantic i18n key (#2542)
admin-roles-assign-to-groups = Asignar rol a grupos

# semantic i18n key (#2542)
admin-roles-search-groups-label = Buscar grupos

# semantic i18n key (#2542)
admin-roles-type-to-search-groups = Escribe para buscar grupos

# semantic i18n key (#2542)
admin-roles-assign-role-btn = Asignar rol

# semantic i18n key (#2542)
admin-onboarding-step-welcome = Bienvenido

# semantic i18n key (#2542)
admin-onboarding-step-welcome-desc = Cuéntanos sobre ti

# semantic i18n key (#2542)
admin-onboarding-step-org-desc = Crea tu espacio de trabajo

# semantic i18n key (#2542)
admin-onboarding-step-plan = Elegir plan

# semantic i18n key (#2542)
admin-onboarding-step-plan-desc = Selecciona tu suscripción

# semantic i18n key (#2542)
admin-onboarding-step-payment-desc = Agregar método de pago

# semantic i18n key (#2542)
admin-onboarding-step-start = Comenzar

# semantic i18n key (#2542)
admin-onboarding-step-start-desc = Crea tu primer bot

# semantic i18n key (#2542)
admin-onboarding-view-guide = Ver guía

# semantic i18n key (#2542)
admin-onboarding-welcome-title = ¡Bienvenido a General Bots!

# semantic i18n key (#2542)
admin-onboarding-welcome-subtitle = Vamos a configurarlo en solo unos minutos. Primero, cuéntanos un poco sobre ti.

# semantic i18n key (#2542)
admin-onboarding-full-name = Nombre completo

# semantic i18n key (#2542)
admin-onboarding-your-role = Tu rol

# semantic i18n key (#2542)
admin-onboarding-role-placeholder = Selecciona tu rol...

# semantic i18n key (#2542)
admin-onboarding-role-developer = Desarrollador / Ingeniero

# semantic i18n key (#2542)
admin-onboarding-role-product-manager = Gerente de producto

# semantic i18n key (#2542)
admin-onboarding-role-customer-support = Atención al cliente

# semantic i18n key (#2542)
admin-onboarding-role-executive = Ejecutivo / Fundador

# semantic i18n key (#2542)
admin-onboarding-use-case-label = ¿Qué quieres construir?

# semantic i18n key (#2542)
admin-onboarding-use-case-placeholder = Selecciona el caso de uso principal...

# semantic i18n key (#2542)
admin-onboarding-usecase-support-bot = Bot de atención al cliente

# semantic i18n key (#2542)
admin-onboarding-usecase-sales-assistant = Asistente de ventas

# semantic i18n key (#2542)
admin-onboarding-usecase-internal-kb = Base de conocimiento interna

# semantic i18n key (#2542)
admin-onboarding-usecase-lead-generation = Generación de leads

# semantic i18n key (#2542)
admin-onboarding-usecase-employee-onboarding = Incorporación de empleados

# semantic i18n key (#2542)
admin-onboarding-usecase-ecommerce = Asistente de comercio electrónico

# semantic i18n key (#2542)
admin-onboarding-usecase-other = Otra cosa

# semantic i18n key (#2542)
admin-onboarding-hear-about-label = ¿Cómo te enteraste de nosotros?

# semantic i18n key (#2542)
admin-onboarding-source-search = Motor de búsqueda

# semantic i18n key (#2542)
admin-onboarding-source-social = Redes sociales

# semantic i18n key (#2542)
admin-onboarding-source-friend = Amigo / Colega

# semantic i18n key (#2542)
admin-onboarding-create-org-title = Crea tu organización

# semantic i18n key (#2542)
admin-onboarding-create-org-subtitle = Tu organización es donde administrarás bots, miembros del equipo y bases de conocimiento.

# semantic i18n key (#2542)
admin-onboarding-org-name = Nombre de la organización

# semantic i18n key (#2542)
admin-onboarding-org-name-hint = Esto será visible para los miembros de tu equipo

# semantic i18n key (#2542)
admin-onboarding-org-url = URL de la organización

# semantic i18n key (#2542)
admin-onboarding-org-url-prefix = app.generalbots.com/

# semantic i18n key (#2542)
admin-onboarding-org-url-hint = Solo letras minúsculas, números y guiones

# semantic i18n key (#2542)
admin-onboarding-team-size = Tamaño del equipo

# semantic i18n key (#2542)
admin-onboarding-team-size-placeholder = ¿Cuántas personas usarán esto?

# semantic i18n key (#2542)
admin-onboarding-team-just-me = Solo yo

# semantic i18n key (#2542)
admin-onboarding-industry-placeholder = Selecciona tu industria (opcional)

# semantic i18n key (#2542)
admin-onboarding-industry-technology = Tecnología

# semantic i18n key (#2542)
admin-onboarding-industry-healthcare = Salud

# semantic i18n key (#2542)
admin-onboarding-industry-finance = Finanzas / Banca

# semantic i18n key (#2542)
admin-onboarding-industry-retail = Comercio minorista / Comercio electrónico

# semantic i18n key (#2542)
admin-onboarding-industry-education = Educación

# semantic i18n key (#2542)
admin-onboarding-industry-manufacturing = Manufactura

# semantic i18n key (#2542)
admin-onboarding-industry-consulting = Consultoría / Servicios

# semantic i18n key (#2542)
admin-onboarding-industry-government = Gobierno

# semantic i18n key (#2542)
admin-onboarding-industry-nonprofit = Sin fines de lucro

# semantic i18n key (#2542)
admin-onboarding-choose-plan-title = Elige tu plan

# semantic i18n key (#2542)
admin-onboarding-choose-plan-subtitle = Selecciona el plan que mejor se adapte a tus necesidades. Puedes cambiar de plan en cualquier momento.

# semantic i18n key (#2542)
admin-onboarding-plan-free = Gratis

# semantic i18n key (#2542)
admin-onboarding-plan-free-desc = Para individuos y proyectos pequeños

# semantic i18n key (#2542)
admin-onboarding-per-month = /mes

# semantic i18n key (#2542)
admin-onboarding-select-free = Seleccionar gratis

# semantic i18n key (#2542)
admin-onboarding-most-popular = Más popular

# semantic i18n key (#2542)
admin-onboarding-plan-pro-desc = Para equipos en crecimiento

# semantic i18n key (#2542)
admin-onboarding-select-pro = Seleccionar Pro

# semantic i18n key (#2542)
admin-dashboard-organization-title = Panel de la organización

# semantic i18n key (#2542)
admin-dashboard-organization-subtitle = Resumen de la actividad, los miembros y el estado del sistema de tu organización

# semantic i18n key (#2542)
admin-dashboard-team-members = Miembros del equipo

# semantic i18n key (#2542)
admin-dashboard-system-uptime = Tiempo de actividad del sistema

# semantic i18n key (#2542)
admin-dashboard-api-server = Servidor de API

# semantic i18n key (#2542)
admin-dashboard-operational = Operativo

# semantic i18n key (#2542)
admin-dashboard-latency-45ms = Latencia: 45ms

# semantic i18n key (#2542)
admin-dashboard-requests-1-2k-min = Solicitudes: 1,2K/min

# semantic i18n key (#2542)
admin-dashboard-connections-45-100 = Conexiones: 45/100

# semantic i18n key (#2542)
admin-dashboard-query-time-12ms = Tiempo de consulta: 12ms

# semantic i18n key (#2542)
admin-dashboard-vector-database = Base de datos vectorial

# semantic i18n key (#2542)
admin-dashboard-vectors-2-4m = Vectores: 2,4M

# semantic i18n key (#2542)
admin-dashboard-search-8ms = Búsqueda: 8ms

# semantic i18n key (#2542)
admin-dashboard-tokens-125k-min = Tokens: 125K/min

# semantic i18n key (#2542)
admin-dashboard-latency-850ms = Latencia: 850ms

# semantic i18n key (#2542)
admin-dashboard-high-usage = Uso alto

# semantic i18n key (#2542)
admin-dashboard-used-34-5gb = Usado: 34,5 GB

# semantic i18n key (#2542)
admin-dashboard-iops-2-1k = IOPS: 2,1K

# semantic i18n key (#2542)
admin-dashboard-members-filter = Miembros

# semantic i18n key (#2542)
admin-dashboard-customer-support-bot = Bot de atención al cliente

# semantic i18n key (#2542)
admin-dashboard-security-alert = Alerta de seguridad:

# semantic i18n key (#2542)
admin-dashboard-customer-feedback-survey = Encuesta de comentarios de clientes

# semantic i18n key (#2542)
admin-dashboard-sales-assistant = Asistente de ventas

# semantic i18n key (#2542)
admin-dashboard-load-more-activity = Cargar más actividad

# semantic i18n key (#2542)
admin-dashboard-john-smith = John Smith

# semantic i18n key (#2542)
admin-dashboard-role-distribution = Distribución de roles

# semantic i18n key (#2542)
admin-dashboard-manage-roles = Administrar roles

# semantic i18n key (#2542)
admin-dashboard-bot-status = Estado del bot

# semantic i18n key (#2542)
admin-dashboard-manage-bots = Administrar bots

# semantic i18n key (#2542)
admin-dashboard-handles-customer-inquiries = Gestiona consultas de clientes

# semantic i18n key (#2542)
admin-dashboard-product-recommendations = Recomendaciones de productos

# semantic i18n key (#2542)
admin-dashboard-hr-assistant = Asistente de RR. HH.

# semantic i18n key (#2542)
admin-dashboard-employee-questions = Preguntas de empleados

# semantic i18n key (#2542)
admin-dashboard-it-helpdesk = Mesa de ayuda de TI

# semantic i18n key (#2542)
admin-dashboard-technical-support = Soporte técnico

# semantic i18n key (#2542)
admin-dashboard-pending-invitations = Invitaciones pendientes

# semantic i18n key (#2542)
admin-dashboard-loading-invitations = Cargando invitaciones…

# semantic i18n key (#2542)
admin-dashboard-invite-member = Invitar miembro

# semantic i18n key (#2542)
admin-dashboard-bulk-invite = Invitación masiva

# semantic i18n key (#2542)
admin-dashboard-emails-label = Correos electrónicos (uno por línea, separados por coma o punto y coma)

# semantic i18n key (#2542)
admin-dashboard-manage-users = Administrar usuarios

# semantic i18n key (#2542)
admin-dashboard-roles-permissions = Roles y permisos

# semantic i18n key (#2542)
workspace-placeholder-untitled = Sin título

# semantic i18n key (#2542)
admin-dashboard-placeholder-invite-email = colega@ejemplo.com
admin-dashboard-placeholder-bulk-emails = a@ejemplo.com
b@ejemplo.com
