# General Bots - Error Messages (English)
# This file contains all error message translations

# =============================================================================
# HTTP Errors
# =============================================================================

error-http-400 = Solicitud incorrecta. Por favor verifica tu entrada.
error-http-401 = Autenticación requerida. Por favor inicia sesión.
error-http-403 = No tienes permiso para acceder a este recurso.
error-http-404 = { $entity } no encontrado.
error-http-409 = Conflicto: { $message }
error-http-429 = Demasiadas solicitudes. Por favor espera { $seconds } segundos.
error-http-500 = Error interno del servidor. Por favor intenta más tarde.
error-http-502 = Puerta de enlace incorrecta. El servidor recibió una respuesta inválida.
error-http-503 = Servicio temporalmente no disponible. Por favor intenta más tarde.
error-http-504 = La solicitud expiró después de { $milliseconds }ms.

# =============================================================================
# Validation Errors
# =============================================================================

error-validation-required = { $field } es requerido.
error-validation-email = Por favor ingresa una dirección de correo válida.
error-validation-url = Por favor ingresa una URL válida.
error-validation-phone = Por favor ingresa un número de teléfono válido.
error-validation-min-length = { $field } debe tener al menos { $min } caracteres.
error-validation-max-length = { $field } no debe tener más de { $max } caracteres.
error-validation-min-value = { $field } debe ser al menos { $min }.
error-validation-max-value = { $field } no debe ser mayor que { $max }.
error-validation-pattern = El formato de { $field } es inválido.
error-validation-unique = { $field } ya existe.
error-validation-mismatch = { $field } no coincide con { $other }.
error-validation-date-format = Por favor ingresa una fecha válida en el formato { $format }.
error-validation-date-past = { $field } debe ser en el pasado.
error-validation-date-future = { $field } debe ser en el futuro.

# =============================================================================
# Authentication Errors
# =============================================================================

error-auth-invalid-credentials = Correo o contraseña inválidos.
error-auth-account-locked = Tu cuenta ha sido bloqueada. Por favor contacta a soporte.
error-auth-account-disabled = Tu cuenta ha sido deshabilitada.
error-auth-session-expired = Tu sesión ha expirado. Por favor inicia sesión nuevamente.
error-auth-token-invalid = Token inválido o expirado.
error-auth-token-missing = Se requiere token de autenticación.
error-auth-mfa-required = Se requiere autenticación de múltiples factores.
error-auth-mfa-invalid = Código de verificación inválido.
error-auth-password-weak = La contraseña es muy débil. Por favor usa una contraseña más fuerte.
error-auth-password-expired = Tu contraseña ha expirado. Por favor restablécela.

# =============================================================================
# Configuration Errors
# =============================================================================

error-config = Error de configuración: { $message }
error-config-missing = Configuración faltante: { $key }
error-config-invalid = Valor de configuración inválido para { $key }: { $reason }
error-config-file-not-found = Archivo de configuración no encontrado: { $path }
error-config-parse = Error al analizar configuración: { $message }

# =============================================================================
# Database Errors
# =============================================================================

error-database = Error de base de datos: { $message }
error-database-connection = Error al conectar a la base de datos.
error-database-timeout = La operación de base de datos expiró.
error-database-constraint = Violación de restricción de base de datos: { $constraint }
error-database-duplicate = Ya existe un registro con este { $field }.
error-database-migration = La migración de base de datos falló: { $message }

# =============================================================================
# File & Storage Errors
# =============================================================================

error-file-not-found = Archivo no encontrado: { $filename }
error-file-too-large = El archivo es muy grande. El tamaño máximo es { $maxSize }.
error-file-type-not-allowed = Tipo de archivo no permitido. Tipos permitidos: { $allowedTypes }.
error-file-upload-failed = La subida del archivo falló: { $message }
error-file-read = Error al leer archivo: { $message }
error-file-write = Error al escribir archivo: { $message }
error-storage-full = Cuota de almacenamiento excedida.
error-storage-unavailable = El servicio de almacenamiento no está disponible.

# =============================================================================
# Network & External Service Errors
# =============================================================================

error-network = Error de red: { $message }
error-network-timeout = La conexión expiró.
error-network-unreachable = El servidor no es alcanzable.
error-service-unavailable = Servicio no disponible: { $service }
error-external-api = Error de API externa: { $message }
error-rate-limit = Límite de tasa alcanzado. Reintentar después de { $seconds }s.

# =============================================================================
# Bot & Dialog Errors
# =============================================================================

error-bot-not-found = Bot no encontrado: { $botId }
error-bot-disabled = Este bot está actualmente deshabilitado.
error-bot-script-error = Error de script en línea { $line }: { $message }
error-bot-timeout = La respuesta del bot expiró.
error-bot-quota-exceeded = Cuota de uso del bot excedida.
error-dialog-not-found = Diálogo no encontrado: { $dialogId }
error-dialog-invalid = Configuración de diálogo inválida: { $message }

# =============================================================================
# LLM & AI Errors
# =============================================================================

error-llm-unavailable = El servicio de IA no está disponible actualmente.
error-llm-timeout = La solicitud de IA expiró.
error-llm-rate-limit = Límite de tasa de IA excedido. Por favor espera antes de intentar nuevamente.
error-llm-content-filter = El contenido fue filtrado por las pautas de seguridad.
error-llm-context-length = La entrada es muy larga. Por favor acorta tu mensaje.
error-llm-invalid-response = Se recibió una respuesta inválida del servicio de IA.
error-llm-empty-response = Lo siento, no pude procesar tu mensaje ahora. Intenta de nuevo en unos segundos.

# =============================================================================
# Email Errors
# =============================================================================

error-email-send-failed = Error al enviar correo: { $message }
error-email-invalid-recipient = Dirección de correo del destinatario inválida: { $email }
error-email-attachment-failed = Error al adjuntar archivo: { $filename }
error-email-template-not-found = Plantilla de correo no encontrada: { $template }

# =============================================================================
# Calendar & Scheduling Errors
# =============================================================================

error-calendar-conflict = El horario conflictúa con un evento existente.
error-calendar-past-date = No se pueden programar eventos en el pasado.
error-calendar-invalid-recurrence = Patrón de recurrencia inválido.
error-calendar-event-not-found = Evento no encontrado: { $eventId }

# =============================================================================
# Task Errors
# =============================================================================

error-task-not-found = Tarea no encontrada: { $taskId }
error-task-already-completed = La tarea ya ha sido completada.
error-task-circular-dependency = Se detectó una dependencia circular en las tareas.
error-task-invalid-status = Transición de estado de tarea inválida.

# =============================================================================
# Permission Errors
# =============================================================================

error-permission-denied = No tienes permiso para realizar esta acción.
error-permission-resource = No tienes acceso a este { $resource }.
error-permission-action = No puedes { $action } este { $resource }.
error-permission-owner-only = Solo el propietario puede realizar esta acción.

# =============================================================================
# Generic Errors
# =============================================================================

error-internal = Error interno: { $message }
error-unexpected = Ocurrió un error inesperado. Por favor intenta nuevamente.
error-not-implemented = Esta función aún no está implementada.
error-maintenance = El sistema está en mantenimiento. Por favor intenta más tarde.
error-unknown = Ocurrió un error desconocido.

# =============================================================================
# Contrato de API localizado (#2094) — códigos estables, mensajes localizados
# =============================================================================

error-locale-invalid = El idioma solicitado no es válido.
error-locale-unsupported = Este idioma no está disponible.
error-locale-persist-failed = No se pudo guardar tu preferencia de idioma.
error-bot-id-required = El id del bot es obligatorio.
error-bot-id-invalid = El id del bot no es válido.
error-bot-name-required = El nombre del bot es obligatorio.
error-token-invalid-format = El formato del token no es válido.
error-token-signing-failed = No se pudo firmar el token.
error-token-signature-invalid = La firma del token no es válida.
error-token-payload-invalid = El contenido del token no es válido.
error-token-payload-json-invalid = El contenido del token no es un JSON válido.
error-cloud-token-required = Proporciona un cloud_token.
error-llm-subscription-token-missing = Falta el token de suscripción.
error-llm-token-store-unavailable = El almacén de tokens no está disponible.
error-llm-subscription-token-expired = El token de suscripción expiró; envía el mensaje de nuevo.
error-llm-prompt-missing = No hay ningún prompt en la solicitud.
error-llm-account-unlinked = Conecta tu cuenta de Claude o ChatGPT en Ajustes → Cuentas de IA primero.

error-timeout = La solicitud ha caducado. Inténtalo de nuevo.

# =============================================================================
# Errores de API localizados — rutas, sincronización, pares, uso del equipo (#2094)
# =============================================================================

error-agent-drive-database-unavailable = El almacén de archivos no está disponible temporalmente.
error-agent-drive-lookup-failed = No se pudo leer la conversación. Inténtalo de nuevo.
error-agent-drive-conversation-not-found = Conversación no encontrada.
error-agent-drive-bad-owner = El registro del propietario de la conversación no es válido.
error-agent-drive-forbidden = Esta conversación pertenece a otro usuario.
error-agent-drive-bad-bot = El registro del bot de esta conversación no es válido.
error-agent-drive-workspace-stopped = El espacio de trabajo del agente no está en ejecución.
error-agent-drive-no-files = No hay archivos en la carpeta de salida del espacio de trabajo.
error-agent-drive-named-files-missing = Los archivos solicitados no están en la carpeta de salida del espacio de trabajo.
error-agent-drive-save-failed = No se pudo guardar ningún archivo en Drive.
error-agent-drive-file-missing = El archivo no está en la carpeta de salida del espacio de trabajo.
error-agent-drive-save-one-failed = No se pudo guardar el archivo en Drive.
error-agent-drive-bad-path = La ruta de Drive devuelta por el almacén no es válida.
error-agent-drive-link-failed = No se pudo crear el enlace para compartir.
error-subagent-no-llm = No hay ningún proveedor de LLM configurado para subagentes.
error-goal-run-not-running = Esta ejecución de meta no está en curso.
error-goal-run-not-paused = Esta ejecución de meta no está en pausa.
error-autotask-not-found = Tarea automática no encontrada.
error-autotask-intent-required = Describe qué debe hacer la tarea.
error-peer-network-disabled = La red de pares está desactivada en este nodo.
error-peer-network-unauthorized = Se requiere un token de par válido.
error-peer-network-unconfigured = Este nodo no tiene token de par configurado, así que los endpoints de par están cerrados.
error-peer-network-forbidden = Se requiere una sesión de administrador para este control de par.
error-peer-network-invalid = La solicitud de par no es válida: { $message }
error-peer-network-unknown-node = Nodo de par desconocido: { $message }
error-peer-network-registry-full = El registro de pares está lleno.
error-peer-network-payload-store-full = El almacén de cargas de par está lleno: { $message }
error-peer-network-recipient-required = El parámetro de consulta recipient es obligatorio.
error-drive-sync-field-required = { $field } es obligatorio.
error-drive-sync-field-absolute = { $field } debe ser una ruta absoluta.
error-drive-sync-field-parent = { $field } no puede contener '..'.
error-drive-sync-field-not-folder = { $field } no es una carpeta en esta máquina: { $path }
error-drive-sync-field-local-folder = La carpeta local
error-drive-sync-field-drive-folder = La carpeta de Drive
error-drive-sync-job-not-found = La tarea de sincronización { $id } no se encontró.
error-drive-sync-job-name-required = El nombre de la tarea de sincronización es obligatorio.
error-drive-sync-job-exists = Ya existe una tarea de sincronización llamada { $name }.
error-drive-sync-conflict-path-required = La ruta del conflicto es obligatoria.
error-drive-sync-conflict-path-escape = La ruta del conflicto se sale de su carpeta de sincronización.
error-drive-sync-task-failed = La tarea { $task } falló: { $error }
error-org-auth-service-unavailable = El servicio de directorio no está disponible.
error-org-default-delete = La organización predeterminada no se puede eliminar.
error-org-settings-save-failed = No se pudo guardar la configuración de la organización.
error-org-name-required = El nombre de la organización es obligatorio.
error-org-tenant-missing = Todavía no existe un tenant: ejecuta las migraciones o el arranque primero.
error-computer-use-session-required = Se requiere un id de sesión.
error-computer-use-kind-invalid = El tipo debe ser 'browser' o 'terminal'.
error-computer-use-action-not-found = Acción no encontrada.
error-computer-use-no-consent = No hay registro de consentimiento para esta acción; apruébala primero.
error-computer-use-browser-only = Esta acción debe ejecutarse en el panel del navegador.
error-computer-use-url-missing = La acción no tiene URL.
error-terminal-approval-required = La ejecución en terminal requiere la aprobación explícita del usuario.
error-terminal-action-id-required = La ejecución en terminal requiere un id de acción aprobado.
error-terminal-command-required = El comando es obligatorio.
error-terminal-command-too-long = El comando está limitado a 16000 caracteres.
error-terminal-blocked-pattern = El comando coincide con un patrón destructivo bloqueado.
error-terminal-workspace-stopped = El espacio de trabajo del agente no está en ejecución. Termina el aprovisionamiento (o reinicia la aplicación) e inténtalo de nuevo.
error-terminal-timeout = El comando agotó el tiempo de espera.

# =============================================================================
# Colaboración, grupos, autotareas y cuentas de IA (#2094)
# =============================================================================

error-db-unavailable = La base de datos no está disponible en este momento. Inténtalo de nuevo.
error-collab-invalid-resource = El tipo o el id del recurso no es válido.
error-collab-owner-only = Solo el propietario puede hacer esto.
error-collab-invalid-grantee-type = El tipo de destinatario debe ser 'user' o 'domain'.
error-collab-invalid-role = El rol no es válido.
error-collab-invalid-grantee-id = El id del destinatario no es válido.
error-collab-grant-not-found = No se encontró el permiso.
error-collab-invalid-new-owner = El id del nuevo propietario no es válido.
error-collab-link-not-found = No se encontró el enlace para compartir.
error-collab-comment-not-found = No se encontró el comentario.
error-collab-comment-resolve-forbidden = No se encontró el comentario, o no es tuyo para resolverlo.
error-collab-comment-body-required = El cuerpo del comentario es obligatorio.
error-collab-comment-too-long = El comentario está limitado a 8000 caracteres.
error-collab-invalid-emoji = El emoji no es válido.
error-collab-invalid-cursor = El cursor 'before' no es válido.
error-collab-invalid-action = La acción no es válida.
error-collab-invalid-name = El nombre no es válido.
error-collab-content-required = El contenido es obligatorio.
error-collab-content-too-large = El contenido es demasiado grande.
error-collab-version-not-found = No se encontró la versión.
error-collab-invalid-base-version = La versión base no es válida.
error-collab-invalid-parent = El id del padre no es válido.
error-collab-invalid-op-type = El tipo de operación no es válido.
error-collab-operation-not-found = No se encontró la operación.
error-collab-not-a-conflict = La operación no es un conflicto sin resolver.
error-collab-invalid-resolution = La resolución debe ser 'accept-server' o 'accept-client'.
error-group-budget-not-found = No se encontró el presupuesto del grupo.
error-group-budget-invalid-id = El id del presupuesto no es válido.
error-group-budget-amount-invalid = El importe debe ser cero o positivo.
error-group-budget-caps-invalid = Los límites deben ser cero o positivos (cero significa sin límite).
error-autotask-fields-required = El título, la programación y el comando son obligatorios.
error-autotask-kind-invalid = El tipo debe ser 'cron' o 'systemd'.
error-conversation-not-found = No se encontró la conversación.
error-conversation-session-invalid = La sesión de la conversación no es válida.
error-subagent-invalid-tasks = Las tareas del subagente no son válidas.
error-ai-account-unknown-provider = El proveedor de cuenta de IA es desconocido.
error-ai-account-workspace-unavailable = El espacio de trabajo del agente no está disponible en esta compilación.
error-ai-account-agent-invalid = El agente debe ser reasoning, agentic o fast.
error-ai-account-not-connected-vibe = Conecta la cuenta antes de asignarla a Vibe.
error-ai-account-not-connected-chat = Conecta la cuenta antes de seleccionarla para el chat.
error-computer-use-action-id-mismatch = El id de la acción no coincide con la ruta.
error-http-no-route = Ninguna ruta coincide con esta ruta.
error-goal-run-advance-refused = La ejecución no puede avanzar en su estado actual.

# =============================================================================
# People, attendance and products routes (#2094)
# =============================================================================

error-person-not-found = Persona no encontrada.
error-team-not-found = Equipo no encontrado.
error-time-off-not-found = Solicitud de ausencia no encontrada.
error-people-invalid-start-date = La fecha inicial no es válida. Usa el formato AAAA-MM-DD.
error-people-invalid-end-date = La fecha final no es válida. Usa el formato AAAA-MM-DD.
error-attendance-webhook-not-found = Webhook no encontrado.
error-attendance-session-id-invalid = El id de la sesión no es válido.
error-attendance-session-not-found = Sesión no encontrada.
error-attendance-attendant-id-required = El id del agente es obligatorio.
attendance-no-attendant-matching-skills = No se encontró ningún agente con las habilidades requeridas.
attendance-response-sent = Respuesta enviada.
error-product-not-found = Producto no encontrado.
error-service-not-found = Servicio no encontrado.
error-service-invalid-value = El valor del servicio no es válido.
error-service-price-required = Este servicio no tiene precio; informa un valor.

# Service-tax value validation (#2094)
error-invalid-value = No se pudo interpretar el valor indicado.
error-value-required = Es necesario indicar un valor para este servicio.
