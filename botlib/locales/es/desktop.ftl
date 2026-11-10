# General Bots — shell del escritorio y asistente de configuración (#2089)

desktop-document-title = General Bots Desktop
sidebar-toggle = Alternar barra lateral
sidebar-collapsed-chat = Chat
sidebar-collapsed-terminal = Terminal
sidebar-collapsed-drive = Archivos
sidebar-collapsed-settings = Ajustes
sidebar-collapsed-account = Cuenta
setup-welcome-title = Bienvenido a General Bots
setup-welcome-subtitle = Vamos a configurarlo en unos pasos rápidos
setup-welcome-quick-subtitle = Dos pasos rápidos para empezar
setup-step-llm = Proveedor de LLM
setup-step-profile = Perfil de usuario
setup-step-bot = Configuración del bot
setup-step-training = Entrenamiento
setup-step-ai-model = Modelo de IA
setup-step-your-name = Tu nombre
setup-choose-provider-title = Elige tu proveedor de LLM
setup-choose-provider-subtitle = Selecciona el proveedor de modelo de IA para tu bot
setup-provider-openai-desc = Modelos GPT-4, GPT-3.5
setup-provider-nvidia-desc = Inferencia NVIDIA NIM
setup-provider-anthropic-desc = Modelos Claude 3, Claude 3.5
setup-provider-local-desc = llama.cpp, Ollama, etc.
setup-profile-title = ¿Qué te describe mejor?
setup-profile-subtitle = Selecciona tu tipo de perfil para valores personalizados
setup-profile-developer = Desarrollador
setup-profile-developer-desc = Crea y despliega bots, escribe scripts BASIC
setup-profile-business = Negocios
setup-profile-business-desc = Automatiza flujos de trabajo, gestiona el CRM
setup-profile-support = Soporte
setup-profile-support-desc = Atención al cliente, gestión de tickets
setup-bot-title = Configura tu bot
setup-bot-subtitle = Dale a tu bot un nombre, un propósito y una plantilla opcional
setup-bot-name = Nombre del bot
setup-bot-name-placeholder = Mi asistente
setup-bot-purpose = Propósito del bot
setup-bot-purpose-placeholder = Describe qué debe hacer tu bot...
setup-bot-template = Plantilla de bot
setup-optional = (opcional)
setup-template-scratch = — Empezar desde cero —
setup-name-title = ¿Cómo te llamamos?
setup-name-subtitle = Este nombre aparece en tus conversaciones
setup-name-label = Tu nombre
setup-name-placeholder = p. ej. María
setup-training-title = Entrena tu bot
setup-training-subtitle = Arrastra archivos .pdf, .xls, .doc para el entrenamiento inicial. Tendrás acceso a Drive y podrás cambiarlo después.
setup-drop-text = Suelta archivos aquí o haz clic para seleccionar
setup-drop-hint = .pdf, .xls, .doc — hasta 10MB cada uno
setup-select-files = Seleccionar archivos
setup-file-status-pending = Pendiente
setup-file-status-uploading = Subiendo
setup-file-status-done = Listo
setup-file-status-error = Error
setup-saving = Guardando...
setup-complete-message = ¡Configuración completa! Bienvenido a General Bots.
setup-failed = Error de configuración: {error}
setup-unknown-error = Error desconocido
setup-back = Atrás
setup-next = Siguiente
setup-finish = Finalizar
setup-skip = Omitir configuración
agent-all-apps = Todas las apps

# --- Paneles del shell del escritorio: versiones, accesibilidad, barra
# --- lateral, colaboración, actualizaciones, modales y uso — #2095

versions-component = Componente
versions-version = Versión
versions-status = Estado
versions-action = Acción
versions-up-to-date = Actualizado
versions-unknown = Desconocido
versions-check-update = Buscar actualización
versions-checking = Comprobando...
versions-checked = Comprobado
versions-error = Error
versions-update-available = Actualizar: { $version }
versions-could-not-load = No se pudo cargar la información de versiones.
versions-ensure-backend = Asegúrate de que el servidor backend esté en ejecución.
versions-retry = Reintentar

a11y-chat-conversation = Conversación del chat
a11y-chat-messages = Mensajes del chat
a11y-send-message = Enviar mensaje
a11y-type-message = Escribe tu mensaje
a11y-keyboard-shortcuts = Atajos de teclado
a11y-shortcut-focus-chat = Enfocar el campo del chat
a11y-shortcut-show-help = Mostrar esta ayuda
a11y-shortcut-escape = Cerrar modal / limpiar selección
a11y-shortcut-command-palette = Paleta de comandos
a11y-shortcut-privacy = Ajustes de privacidad
a11y-shortcut-scroll-history = Desplazar el historial del chat
a11y-shortcut-navigate = Navegar entre elementos
a11y-shortcut-activate = Activar el elemento enfocado
a11y-shortcut-delete-files = Eliminar los archivos seleccionados (Drive)
a11y-shortcut-select-all = Seleccionar todos los archivos (Drive)
a11y-shortcut-clipboard = Copiar / Cortar / Pegar archivos
a11y-shortcut-navigate-up = Subir un nivel en Drive
a11y-shortcut-rename = Renombrar el archivo seleccionado

sidebar-locked-conversations = Conversaciones bloqueadas
sidebar-no-conversations = Aún no hay conversaciones
sidebar-sign-in-conversations = Inicia sesión para ver tus conversaciones
sidebar-history-unavailable = Historial no disponible
sidebar-back-to-conversations = ← Volver a las conversaciones
sidebar-no-locked-conversations = No hay conversaciones bloqueadas
sidebar-locked-conversation = Conversación bloqueada
sidebar-sign-in = Iniciar sesión
sidebar-add-pin = Añadir fijado

collab-collaborators = Colaboradores ({ $count })
collab-no-one-else = No hay nadie más aquí

update-available-title = Actualización disponible — v{ $version }
update-now = Actualizar ahora
update-later = Más tarde
update-skip-version = Omitir esta versión
update-platform = Plataforma: { $name }
update-asset = Archivo: { $name }
update-check-failed = Falló la comprobación de actualización: { $error }
update-view-release-notes = Ver notas de la versión
update-release-notes-title = Notas de la versión — v{ $version }
update-no-release-notes = No hay notas de la versión disponibles.

modal-alert-title = Aviso
modal-input-title = Entrada

external-save-to = Guardar en { $target }

usage-title = Uso y coste
usage-last-days = Últimos { $days } días
usage-loading = Cargando uso…
usage-per-model = Por modelo
usage-per-day = Por día
usage-model = Modelo
usage-tokens = Tokens
usage-messages = Mensajes
usage-cost = Coste
usage-day = Día
usage-days-7 = 7 días
usage-days-30 = 30 días
usage-days-90 = 90 días

pin-close = Cerrar
pin-length-hint = 4–8 dígitos
pin-cancel = Cancelar
pin-ok = Aceptar

# --- Superficies de colaboración compartidas por todas las apps del suite:
# --- diálogo de compartir, historial de actividad, aviso de conflicto e
# --- historial de versiones (suite/js/collab-*.js) — #2095

role-owner = propietario
role-editor = Editor
role-commenter = Comentarista
role-viewer = Lector

share-request-failed = Error en la solicitud ({ $status })
share-warning-title = Los enlaces compartidos aún no son accesibles
share-warning-body = Esta instalación solo es accesible en localhost, así que un enlace aquí no abre en ningún lado. Configure una de estas opciones primero: active P2P en Configuración → Red, o apunte Enlaces Compartidos a su propio dominio.
share-open-shared-links = Abrir Enlaces Compartidos
share-open-network = Abrir Red
share-invite-people = Invitar personas
share-anyone-with-link = Cualquier persona con el enlace
share-create-link = Crear enlace
share-transfer-ownership = Transferir propiedad
share-transfer = Transferir
share-new-owner-placeholder = correo del nuevo propietario
share-owner-suffix = · propietario
share-no-people = Aún no hay personas añadidas.
share-no-links = Aún no hay enlace público.
share-copied = Copiado
share-revoke = Revocar
share-enter-email = Introduzca un correo
share-enter-new-owner = Introduzca el correo del nuevo propietario
share-needs-base = Configure P2P o su propio dominio antes de crear un enlace compartido.

activity-title = Actividad
activity-request-failed = Error en la solicitud ({ $status })
activity-load-more = Cargar más
activity-loading = Cargando…
activity-empty = Aún no hay actividad
activity-action-create = creó
activity-action-edit = editó
activity-action-comment = comentó en
activity-action-delete = eliminó
activity-action-resolve = resolvió un comentario en
activity-action-reopen = reabrió un comentario en
activity-action-reaction = reaccionó a
activity-action-share = compartió
activity-action-restore = restauró
activity-action-transfer = transfirió

conflict-title = Ediciones en conflicto
conflict-dismiss = Descartar
conflict-edited-concurrently = editó simultáneamente ({ $op })
conflict-keep-server = Mantener versión del servidor
conflict-keep-mine = Mantener mi cambio
conflict-not-started = La vigilancia de conflictos no está iniciada
conflict-request-failed = Error en la solicitud ({ $status })

versions-title = Historial de versiones
versions-loading = Cargando…
versions-empty = Aún no hay versiones — el primer guardado crea una.
versions-version-n = Versión { $n }
versions-current-tag = (actual)
versions-name = Nombrar…
versions-restore-failed = Error al restaurar: { $error }
versions-rename-failed = Error al renombrar: { $error }
versions-restore-dialog-title = Restaurar versión
versions-restore-confirm = ¿Restaurar esta versión? Se creará una nueva versión actual — no se pierde nada.
versions-name-dialog-title = Nombrar versión
versions-name-prompt = Nombre de esta versión (ej.: "v2 — aprobada"):
versions-request-failed = Error en la solicitud ({ $status })

# --- Shell del escritorio: selector de acople, pestañas de espacio de
# --- trabajo, escritorios virtuales, menú contextual, panel git, panel de
# --- navegador, widgets y exportación de datos (suite/js/*.js)

snap-title = Acoplar "{ $app }"
snap-layout-top-left = Esquina superior izquierda
snap-layout-top = Mitad superior
snap-layout-top-right = Esquina superior derecha
snap-layout-left = Mitad izquierda
snap-layout-full = Maximizar
snap-layout-right = Mitad derecha
snap-layout-bottom-left = Esquina inferior izquierda
snap-layout-bottom = Mitad inferior
snap-layout-bottom-right = Esquina inferior derecha
snap-layout-third-left = Tercio izquierdo
snap-layout-two-thirds = Dos tercios
snap-layout-third-right = Tercio derecho

wstabs-close-tab = Cerrar pestaña
wstabs-new-tab = Nueva pestaña de espacio de trabajo
wstabs-default-name = Espacio de trabajo { $n }
wstabs-default-main = Principal
wstabs-rename-title = Renombrar espacio de trabajo
wstabs-name-prompt = Nombre del espacio de trabajo:

vdesk-default-name = Escritorio { $n }
vdesk-title = Escritorios Virtuales
vdesk-new = Nuevo escritorio
vdesk-rename-title = Renombrar escritorio
vdesk-name-prompt = Nombre del escritorio:
vdesk-restore-title = Restaurar sesión
vdesk-restore-body = ¿Reabrir las { $count } ventana(s) de su última sesión en este escritorio?
vdesk-restore-body-short = ¿Restaurar su última sesión ({ $count } ventana(s))?

ctxmenu-search = Buscar en DuckDuckGo
ctxmenu-search-prompt = Buscar en DuckDuckGo:
ctxmenu-wikipedia = Definición en Wikipedia
ctxmenu-wikipedia-prompt = Definición en Wikipedia:

git-loading-status = Cargando estado...
git-no-changes = Sin cambios
git-new-branch-prompt = Nombre de la nueva rama:
git-loading-diff = Cargando diff...
git-diff-failed = Error al cargar el diff
git-commit-message-required = Introduzca un mensaje de commit
git-pushing = Enviando...
git-committed = Commit y push realizados correctamente
git-failed = Error en la operación de Git

browser-session-active = Sesión de navegador { $id } activa.
browser-init-failed = Error al inicializar el navegador
browser-init-first = Inicialice el navegador primero
browser-capture = Captura
browser-no-session = Ninguna sesión activa
browser-export-failed = Error en la exportación
browser-no-actions = Ninguna acción registrada

widget-load-failed = Error al cargar el widget
widget-no-source = Ninguna fuente de widget configurada
widget-open-windowed = Abrir en ventana
widget-pin-launcher = Fijar en el lanzador

dataexport-title = Exportación y Copia de Seguridad de Datos
dataexport-desc = Exporte todos sus datos, incluido el historial de chat, la memoria, los metadatos del drive y la configuración, como un archivo JSON de copia de seguridad.
dataexport-export-all = Exportar Todos los Datos
dataexport-import = Importar Copia
dataexport-exported = Copia de seguridad exportada correctamente
dataexport-too-large = Copia rechazada: el archivo supera los 4 MB
dataexport-invalid = Archivo de copia inválido: { $error }
dataexport-nothing = Esta copia no contiene ajustes restaurables
dataexport-unknown-date = fecha desconocida
dataexport-restore-confirm = Esto restaurará { $count } ajuste(s) de la copia del { $date }. Las credenciales y los tokens de sesión nunca se importan. ¿Continuar?
dataexport-restored = Copia restaurada ({ $count } elementos).
dataexport-rejected = { $count } entrada(s) rechazada(s) por la lista de permitidos.
dataexport-reloading = Recargando...

comments-title = Comentarios
comments-request-failed = Error en la solicitud ({ $status })
comments-loading = Cargando comentarios…
comments-load-failed = Error al cargar comentarios: { $error }
comments-empty-open = No hay comentarios abiertos. Use @nombre para mencionar a alguien.
comments-empty-resolved = No hay comentarios resueltos. Use @nombre para mencionar a alguien.
comments-resolved-chip = Resuelto
comments-resolve = Resolver
comments-reopen = Reabrir
comments-resolved-announce = Comentario resuelto
comments-reopened-announce = Comentario reabierto
comments-resolve-failed = Error al resolver: { $error }
comments-added = Comentario añadido
comments-post = Publicar
comments-post-failed = Error al comentar: { $error }
comments-reaction-failed = Error en la reacción: { $error }
comments-delete-failed = Error al eliminar: { $error }
comments-reply-failed = Error al responder: { $error }
comments-delete-title = Eliminar comentario
comments-delete-confirm = ¿Eliminar este comentario?
comments-reply-title = Responder
comments-reply-prompt = Responder al comentario:
comments-filter-open = Abiertos
comments-filter-all = Todos
comments-filter-resolved = Resueltos
comments-input-placeholder = Añada un comentario… use @nombre para mencionar
comments-someone = alguien
comments-mention-announce = Le mencionaron en un comentario de { $author }
