# General Bots - Mensajes de Error (Español)
# Este archivo contiene todas las traducciones de mensajes de error

# =============================================================================
# Errores HTTP
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
# Errores de Validación
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
# Errores de Autenticación
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
# Errores de Configuración
# =============================================================================

error-config = Error de configuración: { $message }
error-config-missing = Configuración faltante: { $key }
error-config-invalid = Valor de configuración inválido para { $key }: { $reason }
error-config-file-not-found = Archivo de configuración no encontrado: { $path }
error-config-parse = Error al analizar configuración: { $message }

# =============================================================================
# Errores de Base de Datos
# =============================================================================

error-database = Error de base de datos: { $message }
error-database-connection = Error al conectar a la base de datos.
error-database-timeout = La operación de base de datos expiró.
error-database-constraint = Violación de restricción de base de datos: { $constraint }
error-database-duplicate = Ya existe un registro con este { $field }.
error-database-migration = La migración de base de datos falló: { $message }

# =============================================================================
# Errores de Archivos y Almacenamiento
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
# Errores de Red y Servicios Externos
# =============================================================================

error-network = Error de red: { $message }
error-network-timeout = La conexión expiró.
error-network-unreachable = El servidor no es alcanzable.
error-service-unavailable = Servicio no disponible: { $service }
error-external-api = Error de API externa: { $message }
error-rate-limit = Límite de tasa alcanzado. Reintentar después de { $seconds }s.

# =============================================================================
# Errores de Bot y Diálogos
# =============================================================================

error-bot-not-found = Bot no encontrado: { $botId }
error-bot-disabled = Este bot está actualmente deshabilitado.
error-bot-script-error = Error de script en línea { $line }: { $message }
error-bot-timeout = La respuesta del bot expiró.
error-bot-quota-exceeded = Cuota de uso del bot excedida.
error-dialog-not-found = Diálogo no encontrado: { $dialogId }
error-dialog-invalid = Configuración de diálogo inválida: { $message }

# =============================================================================
# Errores de LLM e IA
# =============================================================================

error-llm-unavailable = El servicio de IA no está disponible actualmente.
error-llm-timeout = La solicitud de IA expiró.
error-llm-rate-limit = Límite de tasa de IA excedido. Por favor espera antes de intentar nuevamente.
error-llm-content-filter = El contenido fue filtrado por las pautas de seguridad.
error-llm-context-length = La entrada es muy larga. Por favor acorta tu mensaje.
error-llm-invalid-response = Se recibió una respuesta inválida del servicio de IA.

# =============================================================================
# Errores de Correo Electrónico
# =============================================================================

error-email-send-failed = Error al enviar correo: { $message }
error-email-invalid-recipient = Dirección de correo del destinatario inválida: { $email }
error-email-attachment-failed = Error al adjuntar archivo: { $filename }
error-email-template-not-found = Plantilla de correo no encontrada: { $template }

# =============================================================================
# Errores de Calendario y Programación
# =============================================================================

error-calendar-conflict = El horario conflictúa con un evento existente.
error-calendar-past-date = No se pueden programar eventos en el pasado.
error-calendar-invalid-recurrence = Patrón de recurrencia inválido.
error-calendar-event-not-found = Evento no encontrado: { $eventId }

# =============================================================================
# Errores de Tareas
# =============================================================================

error-task-not-found = Tarea no encontrada: { $taskId }
error-task-already-completed = La tarea ya ha sido completada.
error-task-circular-dependency = Se detectó una dependencia circular en las tareas.
error-task-invalid-status = Transición de estado de tarea inválida.

# =============================================================================
# Errores de Permisos
# =============================================================================

error-permission-denied = No tienes permiso para realizar esta acción.
error-permission-resource = No tienes acceso a este { $resource }.
error-permission-action = No puedes { $action } este { $resource }.
error-permission-owner-only = Solo el propietario puede realizar esta acción.

# =============================================================================
# Errores Genéricos
# =============================================================================

error-internal = Error interno: { $message }
error-unexpected = Ocurrió un error inesperado. Por favor intenta nuevamente.
error-not-implemented = Esta función aún no está implementada.
error-maintenance = El sistema está en mantenimiento. Por favor intenta más tarde.
error-unknown = Ocurrió un error desconocido.
