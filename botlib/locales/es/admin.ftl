# =============================================================================
# General Bots - Traducciones de Administración (Español)
# =============================================================================
# Traducciones de la interfaz administrativa para el Panel de Admin de GB
# =============================================================================

# -----------------------------------------------------------------------------
# Navegación y Panel de Administración
# -----------------------------------------------------------------------------
admin-title = Administración
admin-dashboard = Panel de Administración
admin-overview = Resumen
admin-welcome = Bienvenido al Panel de Administración

admin-nav-dashboard = Panel
admin-nav-users = Usuarios
admin-nav-bots = Bots
admin-nav-tenants = Inquilinos
admin-nav-settings = Configuración
admin-nav-logs = Registros
admin-nav-analytics = Analíticas
admin-nav-security = Seguridad
admin-nav-integrations = Integraciones
admin-nav-billing = Facturación
admin-nav-support = Soporte
admin-nav-groups = Grupos
admin-nav-dns = DNS
admin-nav-system = Sistema

# -----------------------------------------------------------------------------
# Acciones Rápidas de Admin
# -----------------------------------------------------------------------------
admin-quick-actions = Acciones Rápidas
admin-create-user = Crear Usuario
admin-create-group = Crear Grupo
admin-register-dns = Registrar DNS
admin-recent-activity = Actividad Reciente
admin-system-health = Salud del Sistema

# -----------------------------------------------------------------------------
# Gestión de Usuarios
# -----------------------------------------------------------------------------
admin-users-title = Gestión de Usuarios
admin-users-list = Lista de Usuarios
admin-users-add = Agregar Usuario
admin-users-edit = Editar Usuario
admin-users-delete = Eliminar Usuario
admin-users-search = Buscar usuarios...
admin-users-filter = Filtrar Usuarios
admin-users-export = Exportar Usuarios
admin-users-import = Importar Usuarios
admin-users-total = Total de Usuarios
admin-users-active = Usuarios Activos
admin-users-inactive = Usuarios Inactivos
admin-users-suspended = Usuarios Suspendidos
admin-users-pending = Verificación Pendiente
admin-users-last-login = Último Acceso
admin-users-created = Creado
admin-users-role = Rol
admin-users-status = Estado
admin-users-actions = Acciones
admin-users-no-users = No se encontraron usuarios
admin-users-confirm-delete = ¿Estás seguro de que deseas eliminar este usuario?
admin-users-deleted = Usuario eliminado exitosamente
admin-users-saved = Usuario guardado exitosamente
admin-users-invite = Invitar Usuario
admin-users-invite-sent = Invitación enviada exitosamente
admin-users-bulk-actions = Acciones Masivas
admin-users-select-all = Seleccionar Todo
admin-users-deselect-all = Deseleccionar Todo

# Detalles de Usuario
admin-user-details = Detalles del Usuario
admin-user-profile = Perfil
admin-user-email = Correo Electrónico
admin-user-name = Nombre
admin-user-phone = Teléfono
admin-user-avatar = Avatar
admin-user-timezone = Zona Horaria
admin-user-language = Idioma
admin-user-role-admin = Administrador
admin-user-role-manager = Gerente
admin-user-role-user = Usuario
admin-user-role-viewer = Visualizador
admin-user-status-active = Activo
admin-user-status-inactive = Inactivo
admin-user-status-suspended = Suspendido
admin-user-status-pending = Pendiente
admin-user-permissions = Permisos
admin-user-activity = Registro de Actividad
admin-user-sessions = Sesiones Activas
admin-user-terminate-session = Terminar Sesión
admin-user-terminate-all = Terminar Todas las Sesiones
admin-user-reset-password = Restablecer Contraseña
admin-user-force-logout = Forzar Cierre de Sesión
admin-user-enable-2fa = Habilitar 2FA
admin-user-disable-2fa = Deshabilitar 2FA

# -----------------------------------------------------------------------------
# Gestión de Grupos
# -----------------------------------------------------------------------------
admin-groups-title = Gestión de Grupos
admin-groups-subtitle = Administra grupos, miembros y permisos
admin-groups-list = Lista de Grupos
admin-groups-add = Agregar Grupo
admin-groups-create = Crear Grupo
admin-groups-edit = Editar Grupo
admin-groups-delete = Eliminar Grupo
admin-groups-search = Buscar grupos...
admin-groups-filter = Filtrar Grupos
admin-groups-total = Total de Grupos
admin-groups-active = Grupos Activos
admin-groups-no-groups = No se encontraron grupos
admin-groups-confirm-delete = ¿Estás seguro de que deseas eliminar este grupo?
admin-groups-deleted = Grupo eliminado exitosamente
admin-groups-saved = Grupo guardado exitosamente
admin-groups-created = Grupo creado exitosamente
admin-groups-loading = Cargando grupos...

# Detalles de Grupo
admin-group-details = Detalles del Grupo
admin-group-name = Nombre del Grupo
admin-group-description = Descripción
admin-group-visibility = Visibilidad
admin-group-visibility-public = Público
admin-group-visibility-private = Privado
admin-group-visibility-hidden = Oculto
admin-group-join-policy = Política de Unión
admin-group-join-invite = Solo por Invitación
admin-group-join-request = Solicitar Unirse
admin-group-join-open = Abierto
admin-group-members = Miembros
admin-group-member-count = { $count ->
    [one] { $count } miembro
   *[other] { $count } miembros
}
admin-group-add-member = Agregar Miembro
admin-group-remove-member = Eliminar Miembro
admin-group-permissions = Permisos
admin-group-settings = Configuración
admin-group-analytics = Analíticas
admin-group-overview = Resumen

# Modos de Vista de Grupos
admin-groups-view-grid = Vista de Cuadrícula
admin-groups-view-list = Vista de Lista
admin-groups-all-visibility = Toda Visibilidad

# -----------------------------------------------------------------------------
# Gestión de DNS
# -----------------------------------------------------------------------------
admin-dns-title = Gestión de DNS
admin-dns-subtitle = Registra y administra nombres de host DNS para tus bots
admin-dns-register = Registrar Nombre de Host
admin-dns-registered = Nombres de Host Registrados
admin-dns-search = Buscar nombres de host...
admin-dns-refresh = Actualizar
admin-dns-loading = Cargando registros DNS...
admin-dns-no-records = No se encontraron registros DNS
admin-dns-confirm-delete = ¿Estás seguro de que deseas eliminar este nombre de host?
admin-dns-deleted = Nombre de host eliminado exitosamente
admin-dns-saved = Registro DNS guardado exitosamente
admin-dns-created = Nombre de host registrado exitosamente

# Campos del Formulario DNS
admin-dns-hostname = Nombre de Host
admin-dns-hostname-placeholder = mibot.ejemplo.com
admin-dns-hostname-help = Ingresa el nombre de dominio completo que deseas registrar
admin-dns-record-type = Tipo de Registro
admin-dns-record-type-a = A (IPv4)
admin-dns-record-type-aaaa = AAAA (IPv6)
admin-dns-record-type-cname = CNAME
admin-dns-ttl = TTL (segundos)
admin-dns-ttl-5min = 5 minutos (300)
admin-dns-ttl-1hour = 1 hora (3600)
admin-dns-ttl-1day = 1 día (86400)
admin-dns-target = Destino/Dirección IP
admin-dns-target-placeholder-ipv4 = 192.168.1.1
admin-dns-target-placeholder-ipv6 = 2001:db8::1
admin-dns-target-placeholder-cname = destino.ejemplo.com
admin-dns-target-help-a = Ingresa la dirección IPv4 a la que apuntar
admin-dns-target-help-aaaa = Ingresa la dirección IPv6 a la que apuntar
admin-dns-target-help-cname = Ingresa el nombre de dominio destino
admin-dns-auto-ssl = Aprovisionar certificado SSL automáticamente

# Encabezados de Tabla DNS
admin-dns-col-hostname = Nombre de Host
admin-dns-col-type = Tipo
admin-dns-col-target = Destino
admin-dns-col-ttl = TTL
admin-dns-col-ssl = SSL
admin-dns-col-status = Estado
admin-dns-col-actions = Acciones

# Estado DNS
admin-dns-status-active = Activo
admin-dns-status-pending = Pendiente
admin-dns-status-error = Error
admin-dns-ssl-enabled = SSL Habilitado
admin-dns-ssl-disabled = Sin SSL
admin-dns-ssl-pending = SSL Pendiente

# Tarjetas de Ayuda DNS
admin-dns-help-title = Ayuda de Configuración DNS
admin-dns-help-a-record = Registro A
admin-dns-help-a-record-desc = Mapea un nombre de dominio a una dirección IPv4. Úsalo para apuntar tu nombre de host directamente a una IP de servidor.
admin-dns-help-aaaa-record = Registro AAAA
admin-dns-help-aaaa-record-desc = Mapea un nombre de dominio a una dirección IPv6. Similar al registro A pero para conectividad IPv6.
admin-dns-help-cname-record = Registro CNAME
admin-dns-help-cname-record-desc = Crea un alias de un dominio a otro. Útil para apuntar subdominios a tu dominio principal.
admin-dns-help-ssl = SSL/TLS
admin-dns-help-ssl-desc = Aprovisiona automáticamente certificados Let's Encrypt para conexiones HTTPS seguras.

# Modales de Editar/Eliminar DNS
admin-dns-edit-title = Editar Registro DNS
admin-dns-remove-title = Eliminar Nombre de Host
admin-dns-remove-warning = Esto eliminará el registro DNS y cualquier certificado SSL asociado. El nombre de host ya no resolverá.

# -----------------------------------------------------------------------------
# Gestión de Bots
# -----------------------------------------------------------------------------
admin-bots-title = Gestión de Bots
admin-bots-list = Lista de Bots
admin-bots-add = Agregar Bot
admin-bots-edit = Editar Bot
admin-bots-delete = Eliminar Bot
admin-bots-search = Buscar bots...
admin-bots-filter = Filtrar Bots
admin-bots-total = Total de Bots
admin-bots-active = Bots Activos
admin-bots-inactive = Bots Inactivos
admin-bots-draft = Bots en Borrador
admin-bots-published = Bots Publicados
admin-bots-no-bots = No se encontraron bots
admin-bots-confirm-delete = ¿Estás seguro de que deseas eliminar este bot?
admin-bots-deleted = Bot eliminado exitosamente
admin-bots-saved = Bot guardado exitosamente
admin-bots-duplicate = Duplicar Bot
admin-bots-export = Exportar Bot
admin-bots-import = Importar Bot
admin-bots-publish = Publicar
admin-bots-unpublish = Despublicar
admin-bots-test = Probar Bot
admin-bots-logs = Registros del Bot
admin-bots-analytics = Analíticas del Bot
admin-bots-conversations = Conversaciones
admin-bots-templates = Plantillas
admin-bots-dialogs = Diálogos
admin-bots-knowledge-base = Base de Conocimiento

# Detalles del Bot
admin-bot-details = Detalles del Bot
admin-bot-name = Nombre del Bot
admin-bot-description = Descripción
admin-bot-avatar = Avatar
admin-bot-status = Estado
admin-bot-status-active = Activo
admin-bot-status-inactive = Inactivo
admin-bot-status-draft = Borrador
admin-bot-status-published = Publicado
admin-bot-language = Idioma
admin-bot-timezone = Zona Horaria
admin-bot-welcome-message = Mensaje de Bienvenida
admin-bot-fallback-message = Mensaje de Respaldo
admin-bot-channels = Canales
admin-bot-integrations = Integraciones
admin-bot-settings = Configuración
admin-bot-permissions = Permisos
admin-bot-analytics = Analíticas
admin-bot-usage = Uso
admin-bot-conversations-count = { $count ->
    [one] { $count } conversación
   *[other] { $count } conversaciones
}

# -----------------------------------------------------------------------------
# Gestión de Inquilinos
# -----------------------------------------------------------------------------
admin-tenants-title = Gestión de Inquilinos
admin-tenants-list = Lista de Inquilinos
admin-tenants-add = Agregar Inquilino
admin-tenants-edit = Editar Inquilino
admin-tenants-delete = Eliminar Inquilino
admin-tenants-search = Buscar inquilinos...
admin-tenants-no-tenants = No se encontraron inquilinos
admin-tenants-confirm-delete = ¿Estás seguro de que deseas eliminar este inquilino?
admin-tenants-deleted = Inquilino eliminado exitosamente
admin-tenants-saved = Inquilino guardado exitosamente

# -----------------------------------------------------------------------------
# Configuración del Sistema
# -----------------------------------------------------------------------------
admin-settings-title = Configuración del Sistema
admin-settings-general = General
admin-settings-security = Seguridad
admin-settings-email = Correo Electrónico
admin-settings-storage = Almacenamiento
admin-settings-integrations = Integraciones
admin-settings-api = API
admin-settings-webhooks = Webhooks
admin-settings-branding = Marca
admin-settings-saved = Configuración guardada exitosamente

# -----------------------------------------------------------------------------
# Registros y Monitoreo
# -----------------------------------------------------------------------------
admin-logs-title = Registros del Sistema
admin-logs-filter = Filtrar Registros
admin-logs-level = Nivel
admin-logs-level-all = Todos
admin-logs-level-debug = Depuración
admin-logs-level-info = Información
admin-logs-level-warn = Advertencia
admin-logs-level-error = Error
admin-logs-search = Buscar registros...
admin-logs-refresh = Actualizar
admin-logs-export = Exportar Registros
admin-logs-clear = Limpiar Registros
admin-logs-no-logs = No se encontraron registros
admin-logs-timestamp = Marca de Tiempo
admin-logs-message = Mensaje
admin-logs-source = Fuente

# -----------------------------------------------------------------------------
# Seguridad
# -----------------------------------------------------------------------------
admin-security-title = Configuración de Seguridad
admin-security-2fa = Autenticación de Dos Factores
admin-security-2fa-required = Requerir 2FA para todos los usuarios
admin-security-password-policy = Política de Contraseñas
admin-security-password-min-length = Longitud Mínima
admin-security-password-require-uppercase = Requerir Mayúsculas
admin-security-password-require-lowercase = Requerir Minúsculas
admin-security-password-require-numbers = Requerir Números
admin-security-password-require-symbols = Requerir Símbolos
admin-security-session-timeout = Tiempo de Espera de Sesión
admin-security-ip-whitelist = Lista Blanca de IP
admin-security-audit-log = Registro de Auditoría
admin-security-api-keys = Claves de API

# -----------------------------------------------------------------------------
# Integraciones
# -----------------------------------------------------------------------------
admin-integrations-title = Integraciones
admin-integrations-available = Integraciones Disponibles
admin-integrations-connected = Integraciones Conectadas
admin-integrations-connect = Conectar
admin-integrations-disconnect = Desconectar
admin-integrations-configure = Configurar
admin-integrations-status-connected = Conectado
admin-integrations-status-disconnected = Desconectado
admin-integrations-status-error = Error
admin-integrations-no-integrations = No hay integraciones configuradas

# -----------------------------------------------------------------------------
# Facturación
# -----------------------------------------------------------------------------
admin-billing-title = Facturación
admin-billing-current-plan = Plan Actual
admin-billing-usage = Uso
admin-billing-invoices = Facturas
admin-billing-payment-methods = Métodos de Pago
admin-billing-upgrade = Mejorar Plan
admin-billing-downgrade = Reducir Plan
admin-billing-cancel = Cancelar Suscripción
