# =============================================================================
# General Bots — es — Settings
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Settings
# -----------------------------------------------------------------------------
settings-title = Configuración
settings-general = General
settings-account = Cuenta
settings-notifications = Notificaciones
settings-privacy = Privacidad
settings-security = Configuración de Seguridad
settings-language = Idioma
settings-theme = Tema
settings-theme-light = Claro
settings-theme-dark = Oscuro
settings-theme-system = Sistema
settings-save = Guardar Cambios
settings-saved = Configuración guardada exitosamente
settings-create-api-key = Crear clave API
settings-api-key-name = Nombre clave
settings-api-key-scopes = Ámbitos (separados por comas)
settings-api-key-scopes-hint = Déjelo vacío para el ámbito de "usuario" predeterminado.
settings-api-key-expiry = Vence en (días)
settings-api-key-expiry-hint = Déjelo vacío sin vencimiento.
settings-add-webhook = Agregar webhook
settings-webhook-url = URL de webhook
settings-webhook-events = Eventos (separados por comas)
settings-webhook-events-hint = Déjelo vacío para recibir todos los eventos.
settings-timezone = Zona Horaria
settings-date-format = Formato de Fecha
settings-time-format = Formato de Hora

# -----------------------------------------------------------------------------
# Settings - Language & Localization
# -----------------------------------------------------------------------------
settings-language-desc = Elige tu idioma preferido
settings-display-language = Idioma de Visualización
settings-language-affects = Afecta todo el texto en la aplicación
settings-date-format-desc = Cómo se muestran las fechas
settings-time-format-desc = Reloj de 12 horas o 24 horas
settings-language-changed = Idioma cambiado exitosamente
settings-reload-required = Se requiere recargar la página para aplicar cambios

# Settings - Profile
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

# Settings - Security
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

# Settings - Appearance
settings-appearance = Apariencia
settings-appearance-desc = Personaliza cómo se ve la aplicación
settings-theme-selection = Tema
settings-theme-selection-desc = Elige tu tema de color preferido
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

# Settings - Notifications
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

# Settings - Storage
settings-storage = Almacenamiento
settings-storage-desc = Administra tu uso de almacenamiento
settings-storage-usage = Uso de Almacenamiento
settings-storage-used = { $used } de { $total } usado
settings-storage-upgrade = Mejorar Almacenamiento

# Settings - Privacy
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

# Settings - Billing
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
# OAuth Account Linking (Settings)
# -----------------------------------------------------------------------------
oauth-connected-accounts = Cuentas Conectadas
oauth-connect = Conectar
oauth-unlink = Desvincular
oauth-not-connected = No conectada
oauth-linked = Vinculada
oauth-no-accounts = No hay cuentas vinculadas todavía.
oauth-loading = Cargando cuentas vinculadas…

## Payment cards (Stripe SetupIntent)
cards-title = Pago y Tarjetas
cards-saved = Tarjetas guardadas
cards-hint = Las tarjetas se almacenan de forma segura en nuestro proveedor de pagos. Los números de tarjeta nunca llegan a nuestros servidores.
cards-add = Añadir tarjeta
cards-add-first = Añadir su primera tarjeta
cards-none = Aún no hay tarjetas guardadas
cards-empty-hint = Añada una tarjeta para habilitar la facturación automática y pagos más rápidos. Será redirigido a nuestro proveedor de pagos seguro para introducir los datos de su tarjeta.
cards-default = Predeterminada
cards-set-default = Establecer como predeterminada
cards-default-btn = Tarjeta predeterminada
cards-remove = Eliminar
cards-remove-confirm = ¿Eliminar esta tarjeta?
cards-expires = Vence
cards-load-error = No se pudieron cargar las tarjetas guardadas.
cards-add-error = No se pudo añadir una tarjeta
cards-default-error = No se pudo actualizar la tarjeta predeterminada
cards-remove-error = No se pudo eliminar la tarjeta
cards-default-updated = Tarjeta predeterminada actualizada
cards-removed = Tarjeta eliminada

# Settings → Model (#2568)
mdl-title = Modelo
mdl-subtitle = Dónde piensa el asistente. Elija un modelo que se ejecute en este equipo, el endpoint de su proveedor o el modelo gratuito alojado.
mdl-current = Modelo actual
mdl-where = Dónde piensa el asistente
mdl-local = En este equipo
mdl-own = El servidor de su organización o su clave
mdl-free = Modelo gratuito alojado
mdl-active = En uso
mdl-endpoint = URL del endpoint
mdl-model = Modelo
mdl-key = Clave de API
mdl-key-stored = clave guardada
mdl-test = Probar
mdl-save = Guardar
mdl-test-ok = Accesible, latencia %ms ms
mdl-test-fail = Error: %e
mdl-testing = Probando…
mdl-saving = Guardando…
mdl-saved = Guardado. Se aplica desde el próximo mensaje.
mdl-swapped = Listo. El nuevo modelo ya está activo.
mdl-restart = Guardado. Reinicie la aplicación para usar el nuevo modelo.
mdl-admin-only = Solo el administrador puede cambiar el modelo.
mdl-loading = Cargando…
mdl-local-active = Modelo local activo: %m
mdl-local-hint = Elija un modelo en la página On-device AI para usarlo aquí.
mdl-local-open = Abrir On-device AI
mdl-free-note = Todo lo que escriba y adjunte se envía a %h.
mdl-inconsistent = La configuración del modelo es inconsistente.
mdl-restore-free = Restaurar modelo gratuito
mdl-confirm-external = Lo que escriba y adjunte se enviará a %h, fuera de este equipo. ¿Continuar?
mdl-scope-machine = En este equipo
mdl-scope-network = El servidor de su organización (%h)
mdl-scope-external = Un host externo (%h)
mdl-unavailable = Este servidor no expone la API de modelos.

# Settings → Instructions, Shortcuts, Text size (#2568)
instr-title = Instrucciones personalizadas
instr-subtitle = Cuéntele al asistente sobre usted y cómo quiere que responda.
instr-enabled = Activar instrucciones personalizadas
instr-enabled-desc = Cuando está desactivado, el asistente responde sin estas instrucciones.
instr-about = ¿Qué debe saber el asistente sobre usted?
instr-about-placeholder = Rol, idioma, contexto, preferencias…
instr-response = ¿Cómo debe responder el asistente?
instr-response-placeholder = Tono, nivel de detalle, formato…
instr-save = Guardar
instr-saving = Guardando…
instr-saved = Guardado.
instr-save-failed = No se pudo guardar: %e
instr-loading = Cargando…
instr-load-failed = No se pudo cargar: %e
instr-unavailable = Este servidor no expone instrucciones personalizadas.
sc-subtitle = Pulse Shift + ? en cualquier parte del suite para abrir esta lista.
sc-group-general = General
sc-group-drive = Drive
sc-note = Los atajos se ignoran mientras escribe en un campo, para no interrumpir mensajes ni formularios.
ts-title = Tamaño del texto
ts-desc = Elija el tamaño del texto en todo el suite
ts-small = Pequeño
ts-default = Predeterminado
ts-large = Grande
ts-xlarge = Muy grande
# Settings → Appearance: chat background pattern
chatbg-title = Fondo del chat
chatbg-desc = Elige el patrón detrás de tus conversaciones
chatbg-none = Ninguno
chatbg-image = Papel
chatbg-dots = Puntos
chatbg-microdots = Micropuntos
chatbg-grid = Cuadrícula
chatbg-blueprint = Plano
chatbg-weave = Trama
chatbg-mosaic = Mosaico

# semantic i18n key (#2542)
settings-network-participation-desc = Controla si esta instalación participa en la red entre pares

# semantic i18n key (#2542)
settings-p2p-network = Red P2P

# semantic i18n key (#2542)
settings-network-mode-label = Modo de red

# semantic i18n key (#2542)
settings-network-mode-p2p = P2P (recomendado)

# semantic i18n key (#2542)
settings-role-viewer = Visualizador

# semantic i18n key (#2542)
settings-register-dns-hostname = Registrar nombre de host DNS

# semantic i18n key (#2542)
settings-dns-aaaa-ipv6 = AAAA (IPv6)

# semantic i18n key (#2542)
settings-dns-cname = CNAME

# semantic i18n key (#2542)
settings-create-token = Crear token
