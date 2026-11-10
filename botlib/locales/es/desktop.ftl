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
