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

# semantic i18n key (#2542)
settings-system-preview-desc = Modo de vista previa e información de versión de esta implementación

# semantic i18n key (#2542)
settings-system-preview-surface-desc = Controla cuánta superficie se revela, no lo que está instalado.

# semantic i18n key (#2542)
settings-system-preview-toggle-desc = Muestra aplicaciones en vista previa y el árbol completo de configuración. Desactivado mantiene solo los ajustes esenciales de esta edición.

# semantic i18n key (#2542)
settings-component-versions = Versiones de componentes

# semantic i18n key (#2542)
settings-component-versions-desc = Versiones de BotServer, BotUI y servicios de esta instalación

# semantic i18n key (#2542)
settings-open-versions = Abrir versiones

# semantic i18n key (#2542)
settings-on-device-ai = IA en el dispositivo

# semantic i18n key (#2542)
settings-operating-system = Sistema operativo

# semantic i18n key (#2542)
settings-ledger = Libro mayor

# semantic i18n key (#2542)
settings-updates = Actualizaciones

# semantic i18n key (#2542)
settings-this-workspace = Este espacio de trabajo

# semantic i18n key (#2542)
settings-loading-identity = Cargando identidad…

# semantic i18n key (#2542)
settings-plan-placeholder = Plan: —

# semantic i18n key (#2542)
settings-preview-placeholder = Vista previa: —

# semantic i18n key (#2542)
settings-oda-subtitle = Modelos que se ejecutan en esta máquina. Descarga un modelo compatible con el hardware, actívalo y el motor local lo usará en su próximo inicio. No se utilizan claves de API aquí.

# semantic i18n key (#2542)
settings-oda-detecting-hardware = Detectando hardware…

# semantic i18n key (#2542)
settings-oda-recommended-for-machine = Recomendado para esta máquina

# semantic i18n key (#2542)
settings-oda-activate = Activar

# semantic i18n key (#2542)
settings-oda-models = Modelos

# semantic i18n key (#2542)
settings-oda-model = Modelo

# semantic i18n key (#2542)
settings-oda-params = Parámetros

# semantic i18n key (#2542)
settings-oda-quant = Cuant.

# semantic i18n key (#2542)
settings-oda-min-vram = VRAM mín.

# semantic i18n key (#2542)
settings-oda-loading-models = Cargando modelos…

# semantic i18n key (#2542)
settings-date-format-mm-dd-yyyy-us = MM/DD/AAAA (EE. UU.)

# semantic i18n key (#2542)
settings-date-format-dd-mm-yyyy-eu-br = DD/MM/AAAA (UE/BR)

# semantic i18n key (#2542)
settings-date-format-yyyy-mm-dd-iso = AAAA-MM-DD (ISO)

# semantic i18n key (#2542)
settings-timezone-america-new-york = Nueva York (GMT-5)

# semantic i18n key (#2542)
settings-timezone-america-los-angeles = Los Ángeles (GMT-8)

# semantic i18n key (#2542)
settings-timezone-europe-london = Londres (GMT+0)

# semantic i18n key (#2542)
settings-timezone-europe-paris = París (GMT+1)

# semantic i18n key (#2542)
settings-timezone-asia-tokyo = Tokio (GMT+9)

# semantic i18n key (#2542)
settings-desktop-wallpaper = Fondo de escritorio

# semantic i18n key (#2542)
settings-desktop-wallpaper-desc = Elige el fondo que se muestra en tu escritorio

# semantic i18n key (#2542)
settings-organizations-bots = Organizaciones y bots

# semantic i18n key (#2542)
settings-organizations-bots-desc = Administra tus organizaciones y la jerarquía de bots

# semantic i18n key (#2542)
settings-current-organization = Organización actual

# semantic i18n key (#2542)
settings-current-organization-desc = Selecciona la organización que deseas administrar

# semantic i18n key (#2542)
settings-loading-organizations = Cargando organizaciones...

# semantic i18n key (#2542)
settings-bot-tree = Árbol de bots

# semantic i18n key (#2542)
settings-bot-tree-desc = Bots que pertenecen a la organización seleccionada

# semantic i18n key (#2542)
settings-loading-bots = Cargando bots...

# semantic i18n key (#2542)
settings-storage-sync-desc = Administra tu configuración de almacenamiento y sincronización

# semantic i18n key (#2542)
settings-knowledge = Conocimiento

# semantic i18n key (#2542)
settings-drive-files-as-context = Usar todos los archivos de Drive como contexto de conocimiento

# semantic i18n key (#2542)
settings-drive-files-as-context-desc = Cuando está activado (predeterminado), los nombres de tus archivos recientes de Drive se envían en cada turno del chat para que el asistente pueda usarlos como contexto. Desactívalo para mantener Drive fuera del prompt.

# semantic i18n key (#2542)
settings-loading-storage-information = Cargando información de almacenamiento...

# semantic i18n key (#2542)
settings-connected-cloud-storage = Almacenamiento en la nube conectado

# semantic i18n key (#2542)
settings-loading-storage-connections = Cargando conexiones de almacenamiento...

# semantic i18n key (#2542)
settings-os-subtitle = Configuración y herramientas de bajo nivel del host para esta instalación del shell

# semantic i18n key (#2542)
settings-os-host-identity-desc = Identidad del host, kernel y zona horaria

# semantic i18n key (#2542)
settings-os-kernel = Kernel

# semantic i18n key (#2542)
settings-os-architecture = Arquitectura

# semantic i18n key (#2542)
settings-os-host-readonly-hint = El nombre de host y la zona horaria son de solo lectura en esta fase. Use la aplicación Terminal o las herramientas del host en la imagen de quiosco para cambiarlos.

# semantic i18n key (#2542)
settings-os-services-desc = Iniciar, detener y reiniciar servicios del host

# semantic i18n key (#2542)
settings-os-load-services-hint = Abra el panel para cargar los servicios.

# semantic i18n key (#2542)
settings-os-disk-usage-desc = Uso de disco de los sistemas de archivos montados

# semantic i18n key (#2542)
settings-os-load-storage-hint = Abra el panel para cargar el almacenamiento.

# semantic i18n key (#2542)
settings-os-log-files = Archivos de registro

# semantic i18n key (#2542)
settings-os-log-files-desc = Archivos que escribe esta instalación, con ubicación y tamaño

# semantic i18n key (#2542)
settings-os-load-log-files-hint = Abra el panel para cargar los archivos de registro.

# semantic i18n key (#2542)
settings-os-tail-log-desc = Mostrar las últimas 200 líneas de un registro de servicio

# semantic i18n key (#2542)
settings-os-reload = Recargar

# semantic i18n key (#2542)
settings-os-restart-service-title = Reiniciar servicio

# semantic i18n key (#2542)
settings-os-the-service = el servicio

# semantic i18n key (#2542)
settings-os-service-name = Nombre del servicio

# semantic i18n key (#2542)
settings-os-restart-action = Reiniciar

# semantic i18n key (#2542)
settings-os-restart-confirm-message = ¿Reiniciar <strong id="os-restart-service-name" data-i18n="settings-os-the-service">el servicio</strong>? Escriba el nombre del servicio para confirmar; el reinicio se registrará en el registro de auditoría.

# semantic i18n key (#2542)
settings-profile-directory-managed = Administrado por el directorio Zitadel

# semantic i18n key (#2542)
settings-profile-open-directory-profile = Abrir perfil del directorio

# semantic i18n key (#2542)
settings-profile-public-title = Perfil público

# semantic i18n key (#2542)
settings-profile-public-desc = Cómo apareces en tu página de inicio de General Bots

# semantic i18n key (#2542)
settings-profile-public-display-name = Nombre para mostrar público

# semantic i18n key (#2542)
settings-profile-public-url-label = La URL de tu perfil será

# semantic i18n key (#2542)
settings-profile-coming-soon = Próximamente

# semantic i18n key (#2542)
settings-profile-public-bio = Biografía pública

# semantic i18n key (#2542)
settings-profile-public-url-placeholder = tu-nombre.generalbots.org

# semantic i18n key (#2542)
settings-profile-language-region = Idioma y región

# semantic i18n key (#2542)
settings-profile-public-url-hint-html = La página estará disponible en <strong><span id="public-profile-url-hint" data-i18n="settings-profile-public-url-placeholder">tu-nombre.generalbots.org</span></strong> cuando se publique el relay del sitio principal de General Bots (#1867).

# semantic i18n key (#2542)
settings-security-directory-note = La identidad, las sesiones y 2FA residen en el directorio Zitadel

# semantic i18n key (#2542)
settings-security-open-directory-security = Abrir seguridad del directorio

# semantic i18n key (#2542)
settings-security-connected-devices = Dispositivos conectados

# semantic i18n key (#2542)
settings-security-connected-devices-desc = Dispositivos que han accedido a tu cuenta

# semantic i18n key (#2542)
settings-security-local-data = Datos locales

# semantic i18n key (#2542)
settings-security-local-data-desc = Esta instalación mantiene todo en este dispositivo.

# semantic i18n key (#2542)
settings-security-export-local-data = Exportar datos locales

# semantic i18n key (#2542)
settings-security-export-local-data-desc = Descargar los datos almacenados en este dispositivo

# semantic i18n key (#2542)
settings-security-clear-local-cache = Borrar caché local

# semantic i18n key (#2542)
settings-security-clear-local-cache-desc = Elimina la configuración y los borradores en caché almacenados en este perfil del navegador

# semantic i18n key (#2542)
settings-security-clear-cache-action = Borrar caché

# semantic i18n key (#2542)
settings-shared-links-subtitle = Elige cómo un enlace público de archivo llega a la persona a la que se lo envías

# semantic i18n key (#2542)
settings-shared-links-loopback-warning-title = Solo se puede acceder a este equipo en este momento.

# semantic i18n key (#2542)
settings-shared-links-loopback-warning-body-html = Un enlace de loopback copiado (<code>127.0.0.1</code>) se abre en el equipo del destinatario, no en el suyo. Use una dirección de red local, defina un dominio o espere la dirección gratuita de General Bots antes de compartir un enlace fuera de este equipo.

# semantic i18n key (#2542)
settings-shared-links-lan-title = Esta red

# semantic i18n key (#2542)
settings-shared-links-lan-desc = Cualquier persona en tu red Wi-Fi o cableada

# semantic i18n key (#2542)
settings-shared-links-on-this-computer = En este equipo

# semantic i18n key (#2542)
settings-shared-links-on-this-network = En esta red

# semantic i18n key (#2542)
settings-shared-links-lan-detail = Funciona para cualquier persona en la misma red. Nada sale de este equipo.

# semantic i18n key (#2542)
settings-shared-links-lan-config-title = Configuración

# semantic i18n key (#2542)
settings-shared-links-lan-config-body-html = Este nivel está activo por defecto: el servidor local escucha en todas las interfaces. Abra la dirección de red desde otro dispositivo de la misma red. Si el tráfico entre dispositivos está bloqueado, permita el puerto TCP de entrada <code id="shared-links-port">8080</code> en este equipo.

# semantic i18n key (#2542)
settings-shared-links-lan-config-local-html = Si el nombre <code>.local</code> no se resuelve, use la dirección IPv4 de este equipo (por ejemplo <code>http://192.168.1.20:8080</code>).

# semantic i18n key (#2542)
settings-shared-links-domain-title = Mi propio dominio

# semantic i18n key (#2542)
settings-shared-links-domain-desc = Un nombre que controlas, accesible desde internet

# semantic i18n key (#2542)
settings-shared-links-public-link = Enlace público

# semantic i18n key (#2542)
settings-shared-links-domain-not-configured = No configurado: los nuevos enlaces volverán a la dirección local anterior.

# semantic i18n key (#2542)
settings-shared-links-domain-config-title = Cómo configurar

# semantic i18n key (#2542)
settings-shared-links-domain-config-dns-html = Apunte un registro DNS (por ejemplo <code>files.example.com</code>) a la dirección pública de este equipo.

# semantic i18n key (#2542)
settings-shared-links-domain-config-url-html = Defina la URL base pública de esta instalación: <pre><code>PUBLIC_BASE_URL=https://files.example.com</code></pre> o, en el archivo de producto: <pre><code>public_base_url=https://files.example.com</code></pre>

# semantic i18n key (#2542)
settings-shared-links-domain-config-restart = Reinicie la aplicación de escritorio para que los nuevos enlaces compartidos se compongan con el dominio; los enlaces existentes conservan sus tokens.

# semantic i18n key (#2542)
settings-shared-links-domain-config-hint = El token del registrador permanece en la bóveda local y nunca se escribe en los registros. La actualización automática de DNS y los certificados TLS forman parte del trabajo del proveedor de dominios registrado en esta incidencia; hasta que eso llegue, mantenga el registro actualizado en su registrador.

# semantic i18n key (#2542)
settings-shared-links-free-title = Dirección gratuita de General Bots

# semantic i18n key (#2542)
settings-shared-links-free-desc = Un nombre proporcionado por General Bots, sin necesidad de dominio

# semantic i18n key (#2542)
settings-shared-links-free-unavailable-html = Aún no disponible. El escritorio funciona sin conexión, así que este nivel necesita una zona DNS gratuita y un relé para equipos que no pueden alcanzarse directamente. Se distribuye desactivado con el trabajo <code>botpublic</code> registrado en #1864.

# semantic i18n key (#2542)
settings-shared-links-free-what-title = Qué hará

# semantic i18n key (#2542)
settings-shared-links-free-what-body-html = La cuenta gratuita guardará un nombre, no sus archivos. Cuando un equipo no pueda alcanzarse directamente, el relé gestionado (<code id="shared-links-relay-host">peer.generalbots.org</code>) conserva solo una copia cifrada del archivo compartido y esa copia caduca. No se requiere configuración manual del relé.

# semantic i18n key (#2542)
settings-shared-links-peer-title = Interruptor maestro de pares

# semantic i18n key (#2542)
settings-shared-links-peer-desc-html = Detiene todo el tráfico P2P de esta instalación. El modo sin conexión mantiene todas las conexiones en <code>127.0.0.1</code>.

# semantic i18n key (#2542)
settings-shared-links-offline-default = Sin conexión (predeterminado)

# semantic i18n key (#2542)
settings-shared-links-p2p-opt-in = P2P (opcional)

# semantic i18n key (#2542)
settings-shared-links-relay-hint-html = El endpoint del relé es fijo (<code>peer.generalbots.org</code>). Cambiar esta configuración puede requerir reiniciar la aplicación para que surta efecto completo.

# semantic i18n key (#2542)
settings-shared-links-check-again = Comprobar de nuevo

# semantic i18n key (#2542)
settings-admin-bots-loading = Cargando bots...

# semantic i18n key (#2542)
settings-admin-domain-configuration = Configuración de dominio

# semantic i18n key (#2542)
settings-admin-audit-loading = Cargando registro de auditoría...

# semantic i18n key (#2542)
settings-updates-subtitle = Consulta la última versión en GitHub y ve a la descarga oficial

# semantic i18n key (#2542)
settings-updates-desktop-title = Actualizaciones de escritorio

# semantic i18n key (#2542)
settings-updates-desktop-checks-html = La aplicación consulta <code>github.com/generalbots/generalbots/releases/latest</code>. La instalación siempre es opcional.

# semantic i18n key (#2542)
settings-updates-check-startup = Buscar actualizaciones al iniciar

# semantic i18n key (#2542)
settings-updates-check-startup-desc = Se ejecuta como máximo una vez al día y solo muestra una notificación. Aun así puedes usar “Comprobar ahora” cuando esto esté desactivado.

# semantic i18n key (#2542)
settings-updates-current-version = Versión actual

# semantic i18n key (#2542)
settings-updates-never = Nunca

# semantic i18n key (#2542)
settings-updates-check-now = Comprobar ahora

# semantic i18n key (#2542)
settings-updates-view-release-notes = Ver notas de la versión

# semantic i18n key (#2542)
settings-updates-not-checked-yet = Aún no se ha comprobado.

# semantic i18n key (#2542)
settings-versions-subtitle = Versiones de componentes y disponibilidad de actualizaciones

# semantic i18n key (#2542)
settings-versions-loading = Cargando información de la versión...

# semantic i18n key (#2542)
settings-ledger-description = Cada acción del terminal y del navegador, con la decisión de consentimiento, el resultado y la ruta de la captura de pantalla.

# semantic i18n key (#2542)
settings-ledger-actions-title = Registro de acciones

# semantic i18n key (#2542)
settings-ledger-tab-terminal = Terminal

# semantic i18n key (#2542)
settings-ledger-tab-browser = Navegador

# semantic i18n key (#2542)
settings-ledger-tab-budgets = Presupuestos

# semantic i18n key (#2542)
settings-ledger-tab-ttft = TTFT
