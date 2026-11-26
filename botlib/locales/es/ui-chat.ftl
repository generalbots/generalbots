# =============================================================================
# General Bots — es — Chat
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

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

# Additional Chat translations
chat-camera = Cámara
chat-camera-title = Tomar una foto
chat-camera-capture = Tomar foto
chat-camera-cancel = Cancelar
chat-camera-sent = Foto enviada.
chat-camera-denied = Se denegó el permiso de la cámara. Autoriza el acceso a la cámara para la aplicación de escritorio en la configuración del sistema para tomar fotos.
chat-camera-missing = No se encontró ninguna cámara en este dispositivo.
chat-camera-error = No se pudo iniciar la cámara.
chat-camera-unsupported = Esta aplicación de escritorio no ofrece acceso a la cámara.
voice-mode-title = Conversación por voz
voice-mode-listening = Escuchando… habla con naturalidad
voice-mode-thinking = Pensando…
voice-mode-speaking = Hablando…
voice-mode-idle = Conversación por voz
voice-mode-end = Finalizar conversación por voz
chat-message-placeholder = Mensaje...

# Chat message controls and notices
chat-copy-message = Copiar mensaje
chat-copied = Copiado
chat-message-copied = Mensaje copiado
chat-copy-failed = Error al copiar
chat-listen-tts = Escuchar (TTS)
chat-listen-message = Escuchar este mensaje
chat-image-unavailable = imagen no disponible
chat-open = Abrir
chat-expand = Expandir
chat-collapse = Contraer
chat-preview = Vista previa
chat-consent-affected-count = Registros afectados
chat-close-preview = Cerrar vista previa
chat-preview-load-failed = No se pudo cargar la vista previa.
chat-show-more = Mostrar más
chat-show-less = Mostrar menos
chat-tool = herramienta
chat-param = parámetro
chat-params = parámetros
chat-tool-result = Resultado de la herramienta
chat-retry = Reintentar
chat-open-as-document = Abrir como documento
chat-show-more-history = mostrar más…
chat-disconnected-reconnecting = Chat desconectado. Reconectando...
chat-connect-retrying = No se pudo conectar con el servidor de chat. Reintentando...
chat-connection-lost = Conexión perdida. Intentando reconectar...
chat-attachment-sent = Archivo adjunto enviado: 
chat-attachments-sent = Archivos adjuntos enviados: 
chat-message-not-delivered = Mensaje no entregado — reconectando
chat-attachment-read-failed = No se pudo leer un archivo adjunto. Adjunte de nuevo.
chat-attachment-queued = La conexión volverá pronto — archivo adjunto en cola
chat-reconnecting-queued = Reconectando — su mensaje se enviará cuando vuelva la conexión.
chat-bot-private-signin = Este bot es privado — inicie sesión para continuar
chat-approved-plan = Plan aprobado: 
chat-retrying-attachment = Reintentando adjunto
chat-expanded-preview = Vista previa ampliada

# #2362 — chat code/artifact control labels
chat-code-show-all = Show all
chat-code-lines = lines
chat-artifact-save-drive = Save to Drive
chat-artifact-download-svg = Download SVG
chat-artifact-download-png = Download PNG
chat-artifact-open-canvas = Open in Canvas
chat-artifact-folder = Folder (optional)
chat-artifact-kind-markdown = Markdown document
chat-artifact-kind-html = HTML page
chat-artifact-kind-code = Code
chat-artifact-words = words

# #2362 — attachment action label
chat-open-in-app = Open in app
# Assistant-look chat: transcript export and answer feedback
chat-export-actions = Acciones de la conversación
chat-export-md = Exportar conversación en Markdown
chat-export-print = Imprimir o guardar como PDF
chat-export-done = Conversación exportada
chat-export-empty = Nada que exportar todavía
chat-export-untitled = Conversación
chat-export-user = Tú
chat-export-assistant = Asistente
chat-feedback-label = Valoración de la respuesta
chat-feedback-good = Buena respuesta
chat-feedback-bad = Mala respuesta
chat-feedback-comment = ¿Qué podría mejorar?
chat-feedback-send = Enviar
chat-staged-files = Archivos adjuntos
chat-staged-close = Cerrar vista previa
chat-camera-waiting = Waiting for the camera to start…
chat-camera-waiting-frames = Permission granted; waiting for video frames
chat-camera-ready = Camera ready. Take the photo or cancel.
chat-camera-started = Camera started — tap Take photo.
chat-camera-no-device = no video device
chat-camera-device = video device
chat-camera-no-frames = The camera preview did not start. Choose a photo from the system picker instead.
chat-camera-timeout = The camera did not answer. Choose a photo from the system picker instead.

# Chat engineer status strip (#2342)
chat-status-bot = Bot
chat-status-session = Sesión
chat-status-model = Modelo
chat-status-state = Estado
chat-status-no-session = —
chat-status-copy-session = Copiar ID de sesión
chat-status-copied = Copiado
chat-status-idle = Inactivo
chat-status-streaming = Transmitiendo
chat-status-thinking = Pensando
chat-status-error = Error
chat-status-stopped = Detenido

chat-response-ready = Respuesta lista
chat-response-started = El asistente está respondiendo
chat-tool-result-ready = Resultado de herramienta disponible

# semantic i18n key (#2542)
chat-no-suggestions-available = No hay sugerencias disponibles

# semantic i18n key (#2542)
chat-start-conversation = Iniciar una conversación

# semantic i18n key (#2542)
chat-empty-hint = Escribe un mensaje abajo o usa entrada de voz

# semantic i18n key (#2542)
chat-no-conversations-yet = Aún no hay conversaciones

# semantic i18n key (#2542)
chat-context-label = Contexto:

# semantic i18n key (#2542)
chat-select-context = Seleccionar contexto...

# semantic i18n key (#2542)
chat-no-contexts-configured = No hay contextos configurados

# semantic i18n key (#2542)
projector-title = Visor de contenido

# semantic i18n key (#2542)
projector-loading-content = Cargando contenido...

# semantic i18n key (#2542)
projector-slide-info = Diapositiva 1 de 1

# semantic i18n key (#2542)
projector-code-info = code.rs

# Projector viewer control (#2542)
chat-projector-rotate = Girar

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
chat-empty-title = Empieza la conversación
chat-empty-desc = Escribe un mensaje abajo o elige una sugerencia cuando el bot la ofrezca.
chat-empty-action = Escribir el primer mensaje

# -----------------------------------------------------------------------------
# #2342 — chat engineer UX: slash commands, help, inspect/copy-markdown
# -----------------------------------------------------------------------------
chat-cmd-menu-title = Commands
chat-cmd-help = Show keyboard shortcuts
chat-cmd-new = Start a new conversation
chat-cmd-clear = Clear this conversation
chat-cmd-stop = Stop generating
chat-stop = Detener
chat-cmd-failed = Command failed
chat-help = Keyboard shortcuts
chat-inspect = Inspect
chat-inspect-title = Message details
chat-inspect-sender = Sender
chat-inspect-id = Message id
chat-inspect-no-id = none
chat-inspect-length = Characters
chat-copy-markdown = Copy as Markdown
chat-copied-markdown = Copied as Markdown

# Model attribution on each bot answer
chat-model-written-by = Respuesta escrita por

# Assistant parity in chat
assistant-parity-block-allow = Permitir
assistant-parity-block-allowed = Permitido
assistant-parity-block-api-command = Comando de API
assistant-parity-block-attachment-read = Adjunto leído
assistant-parity-block-chars = caracteres
assistant-parity-block-command = Comando
assistant-parity-block-decision-failed = No se pudo registrar la decisión
assistant-parity-block-denied = Denegado
assistant-parity-block-deny = Denegar
assistant-parity-block-detail = Detalle
assistant-parity-block-drive-list = Listado de Drive
assistant-parity-block-drive-read = Archivo de Drive leído
assistant-parity-block-drive-search = Búsqueda en Drive
assistant-parity-block-elapsed = Transcurrido
assistant-parity-block-error = Error
assistant-parity-block-file = Archivo
assistant-parity-block-kb-search = Búsqueda en el conocimiento
assistant-parity-block-notice = Aviso
assistant-parity-block-path = Ruta
assistant-parity-block-permission = Permiso solicitado
assistant-parity-block-query = Consulta
assistant-parity-block-read = Leer
assistant-parity-block-reason = Motivo
assistant-parity-block-result = Resultado
assistant-parity-block-retry = Reintentar
assistant-parity-block-seconds = s
assistant-parity-block-size = Tamaño
assistant-parity-block-status-done = Completado
assistant-parity-block-status-failed = Falló
assistant-parity-block-status-needs-you = Necesita tu intervención
assistant-parity-block-status-running = En ejecución
assistant-parity-block-step = Paso
assistant-parity-block-thinking = Pensando
assistant-parity-block-tool = Herramienta
assistant-parity-block-tool-run = Ejecución de herramienta
assistant-parity-block-total = Total
assistant-parity-block-truncated = truncado
assistant-parity-block-type = Tipo
assistant-parity-block-yes = Sí
chat-parity-attach-title = Vista previa del adjunto
chat-parity-attach-unavailable = Contenido no disponible: use Descargar para abrir este archivo.
chat-parity-attach-download-failed = No se pudo descargar el adjunto.
chat-parity-model-tiers = Automático y niveles
chat-parity-model-accounts = Cuentas de proveedores
chat-parity-model-local = Local y gratuito
chat-drive-parity-attaching = Adjuntando...
chat-drive-parity-cancel = Cancelar
chat-drive-parity-delete = Eliminar
chat-drive-parity-deleted = Eliminado
chat-drive-parity-delete-failed = No se pudo eliminar el archivo.
chat-drive-parity-delete-question = ¿Eliminar este archivo?
chat-drive-parity-delete-title = Eliminar archivo
chat-drive-parity-deleting = Eliminando...
chat-drive-parity-download = Descargar
chat-drive-parity-download-failed = No se pudo descargar el archivo.
chat-drive-parity-downloading = Descargando...
chat-drive-parity-more = Más acciones
chat-drive-parity-new-name = Nuevo nombre
chat-drive-parity-preview = Vista previa
chat-drive-parity-preview-failed = No se pudo previsualizar el archivo.
chat-drive-parity-previewing = Abriendo vista previa...
chat-drive-parity-rename = Renombrar
chat-drive-parity-renamed = Renombrado
chat-drive-parity-rename-failed = No se pudo renombrar el archivo.
chat-drive-parity-rename-title = Renombrar archivo
chat-drive-parity-renaming = Renombrando...
chat-drive-parity-unavailable = Esta acción no está disponible en esta instalación.
chat-drive-parity-use-in-chat = Usar en la conversación
chat-parity-agent-does = El agente lo hace
chat-parity-back = Volver
chat-parity-checked-now = Verificado ahora
chat-parity-close = Cerrar
chat-parity-coming-soon = Próximamente
chat-parity-cost-contract = Según tu contrato
chat-parity-cost-free = Gratis
chat-parity-cost-free-limit = Gratis, con límites de uso
chat-parity-cost-sub = Tu suscripción
chat-parity-cuda-no = No se detectó una GPU adecuada
chat-parity-cuda-yes = Tarjeta gráfica detectada
chat-parity-depends-agent = Depende del agente
chat-parity-driver = Controlador
chat-parity-engine = Motor en uso
chat-parity-feature-docs = Documentos
chat-parity-feature-ocr = Texto en imágenes
chat-parity-feature-photos = Fotos y gráficos
chat-parity-feature-web = Búsqueda web
chat-parity-features = Funciones activadas
chat-parity-gains = Tus beneficios
chat-parity-go-agent = Va al proveedor del agente
chat-parity-go-ext = Va a un proveedor externo
chat-parity-go-net = Se queda en la red de tu organización
chat-parity-go-server = Va al servidor que configuraste
chat-parity-guide = ¿No sabes cuál elegir?
chat-parity-guide-sub = Responde dos preguntas.
chat-parity-in-use = En uso
chat-parity-load-failed = No se pudo cargar esta información.
chat-parity-loading = Cargando...
chat-parity-machine = Este equipo
chat-parity-machine-ok = Este equipo tiene: %d, %v GB
chat-parity-machine-slow = Este equipo no tiene tarjeta gráfica adecuada: sería lento
chat-parity-need-acp = Instala el agente e inicia sesión en tu cuenta
chat-parity-need-free = Nada
chat-parity-need-local = Una tarjeta gráfica de 6 GB o más y una descarga de 4 a 6 GB
chat-parity-need-own = La dirección del servidor y una clave
chat-parity-no-read = No lee
chat-parity-no-work = No funciona
chat-parity-none-found = Ninguno encontrado
chat-parity-not-yet = Todavía no
chat-parity-only-net = Solo con un servidor en tu red
chat-parity-optional = Opcional, tú lo activas
chat-parity-pick = Elige una opción
chat-parity-priv-ext = Tus conversaciones se envían a un host externo (%h).
chat-parity-priv-local = Tus conversaciones se quedan en este equipo.
chat-parity-priv-net = Tus conversaciones se envían al servidor de tu organización (%h).
chat-parity-privacy = Privacidad
chat-parity-q1 = ¿Lo que escribes puede salir de este equipo?
chat-parity-q1-help = Piensa en información confidencial o personal.
chat-parity-q1-no = No, debe quedarse aquí
chat-parity-q1-no-desc = Solo se acepta procesamiento local.
chat-parity-q1-yes = Sí, puede salir
chat-parity-q1-yes-desc = Un servicio externo puede procesarlo.
chat-parity-q2-no = No, o no sé
chat-parity-q2-no-desc = Sin servidor o clave de la empresa.
chat-parity-q2-open = ¿Tu organización ofrece un servidor o clave de IA?
chat-parity-q2-open-help = En caso de duda, consulta a TI.
chat-parity-q2-secret = ¿Tu organización permite IA externa?
chat-parity-q2-secret-help = Algunas políticas prohíben servicios externos.
chat-parity-q2-secret-no-desc = No se permite IA externa.
chat-parity-q2-secret-yes-desc = Se permite IA externa.
chat-parity-q2-yes = Sí
chat-parity-q2-yes-desc = Hay servidor o clave de la empresa.
chat-parity-reads = Lee
chat-parity-ready = Listo
chat-parity-rec-free = Empieza con el modelo gratuito: no hay que configurar nada. Para asuntos confidenciales, prefiere este equipo.
chat-parity-rec-local = Usa este equipo: nada sale de aquí y tiene tarjeta gráfica.
chat-parity-rec-local-slow = Este equipo no tiene tarjeta gráfica adecuada para un modelo local. Usa el servidor de tu organización en su red.
chat-parity-rec-local-tight = Un modelo local funciona, pero puede ser lento con esta tarjeta gráfica.
chat-parity-rec-own = Usa el servidor o clave de tu organización: el contrato define coste y límites.
chat-parity-rec-own-net = Usa el servidor de tu organización en su red para mantener los datos privados.
chat-parity-restart = Empezar de nuevo
chat-parity-retry = Reintentar
chat-parity-row-cost = Coste
chat-parity-row-docs = Documentos
chat-parity-row-needs = Necesitas
chat-parity-row-ocr = Texto en imágenes
chat-parity-row-offline = Sin internet
chat-parity-row-photos = Fotos y gráficos
chat-parity-row-privacy = Tus textos
chat-parity-row-web = Búsqueda web
chat-parity-rule = Regla
chat-parity-show-me = Ver cómo activarlo
chat-parity-stay-here = Se queda en este equipo
chat-parity-step = Pregunta %n de 2
chat-parity-sub-acp = Tu suscripción de agente
chat-parity-sub-free = Listo para usar
chat-parity-sub-local = Privado y sin internet
chat-parity-sub-own = De tu organización
chat-parity-suggested = Sugerido
chat-parity-system-memory = Memoria del sistema
chat-parity-verdict-bad-gpu = No hay tarjeta gráfica adecuada
chat-parity-verdict-bad-gpu-long = No se encontró ninguna tarjeta NVIDIA con CUDA. Los modelos locales se ejecutarían en el procesador, mucho más despacio: una respuesta puede tardar minutos. Para mantener tus textos en la red, prefiere el servidor de tu organización.
chat-parity-verdict-capacity = Esta tarjeta ejecuta modelos que necesitan hasta %v de memoria de vídeo.
chat-parity-verdict-good = Cómodo
chat-parity-verdict-low-vram = Tarjeta con memoria insuficiente (%v)
chat-parity-verdict-low-vram-long = La tarjeta tiene %v de memoria de vídeo; el modelo local más pequeño necesita %m.
chat-parity-verdict-mid = Justo pero usable
chat-parity-verdict-none = No se pudo examinar este equipo
chat-parity-video-memory = Memoria de vídeo
chat-parity-where = Elige el motor
chat-parity-where-acp = Agente (ACP)
chat-parity-where-free = Modelo gratuito
chat-parity-where-local = En este equipo
chat-parity-where-own = Servidor o clave propia
chat-parity-where-sub = Compara las cuatro formas de usar el asistente. Haz clic en una para ver detalles y activarla.
chat-parity-works = Funciona
chat-parity-workshop = Taller del asistente
chat-parity-workshop-lead = Ajusta dónde y cómo piensa tu asistente.
chat-parity-workshop-title = Taller del asistente
chat-parity-graphics-card = Tarjeta gráfica

# --- #2921 missing keys added to every locale catalogue ---
chat-confirm-title = Confirmar acción
chat-confirm-unavailable = Confirmación no disponible
chat-copy = Copiar
chat-delete = Eliminar
chat-delete-message-confirm = ¿Eliminar este mensaje de la conversación?
chat-delete-title = Eliminar mensaje
chat-edit = Editar
chat-edit-unavailable = Editor no disponible
chat-message-actions = Acciones del mensaje
chat-regenerate = Regenerar
chat-regenerate-no-source = No hay mensaje anterior del usuario para regenerar
chat-regenerate-unavailable = No se puede regenerar este turno
chat-voice-capture-silent = No se capturó audio
chat-voice-duplex-audio = Audio
chat-voice-duplex-closed = Sesión de voz cerrada
chat-voice-duplex-connect = Conectar voz
chat-voice-duplex-connected = Voz conectada
chat-voice-duplex-disabled = Voz desactivada
chat-voice-duplex-error = Error de voz
chat-voice-duplex-fallback = Volviendo al texto
chat-voice-duplex-live = Voz en vivo
chat-voice-duplex-lost = Conexión perdida
chat-voice-duplex-timeout = La sesión de voz agotó el tiempo
chat-voice-duplex-unavailable = Voz no disponible
chat-voice-mic-failed = Micrófono no disponible
chat-web-title = Búsqueda en la web
voice-mode-connecting = Conectando...
voice-mode-fallback-idle = Inactivo
voice-mode-fallback-listening = Escuchando
voice-mode-fallback-speaking = Hablando
voice-mode-fallback-thinking = Pensando
voice-mode-live-listening = Escuchando
voice-mode-live-ready = Listo
voice-mode-live-speaking = Hablando

# gbgov assistant parity: busy-send and actionable engine refusals (#2563)
chat-busy-send-head = La respuesta todavía se está escribiendo
chat-busy-send-text = Espera a que termine la respuesta, o detínala y envía tu mensaje ahora.
chat-busy-send-wait = Seguir esperando
chat-busy-send-stop = Detener y enviar
chat-engine-error-not-linked = Tu cuenta de %label no está conectada. Elige otro modelo o conecta la cuenta en Ajustes.
chat-engine-error-switch-off = %label necesita el interruptor Agente activado en esta conversa.
chat-engine-error-unavailable = Los agentes por suscripción no están disponibles en este equipo. Elige otro modelo.
chat-engine-error-model-unavailable = El modelo %model no está disponible en %label. Elige otro modelo.
chat-engine-error-workspace-failed = No se pudo iniciar la máquina virtual del agente. Comprueba su estado e inténtalo de nuevo.
chat-engine-error-account-model-off = El modelo %model no está activado para %label. Elige otro modelo.
chat-engine-error-free-unavailable = El modelo gratuito no está disponible ahora. Inténtalo más tarde o elige otro modelo.
chat-engine-error-limit-reached = Has alcanzado el límite de uso de tu plan en %label. %when Elige otro modelo.
chat-engine-error-unsafe-version = Esta versión de %label (%version) se bloqueó porque no pidió permiso antes de actuar. Elige otro modelo.
chat-engine-error-choose-model = Elegir otro modelo
chat-engine-error-back-at = Vuelve %when.
