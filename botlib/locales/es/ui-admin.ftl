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
