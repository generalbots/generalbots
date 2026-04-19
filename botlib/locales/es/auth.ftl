# =============================================================================
# General Bots - Authentication Translations (Spanish)
# =============================================================================
# Traducciones de autenticación, Passkey/WebAuthn y seguridad
# =============================================================================

# -----------------------------------------------------------------------------
# Authentication General
# -----------------------------------------------------------------------------
auth-title = Autenticación
auth-login = Iniciar Sesión
auth-logout = Cerrar Sesión
auth-signup = Registrarse
auth-welcome = Bienvenido
auth-welcome-back = ¡Bienvenido de nuevo, { $name }!
auth-session-expired = Tu sesión ha expirado
auth-session-timeout = La sesión expira en { $minutes } minutos

# -----------------------------------------------------------------------------
# Login Form
# -----------------------------------------------------------------------------
auth-login-title = Inicia sesión en tu cuenta
auth-login-subtitle = Introduce tus credenciales para continuar
auth-login-email = Correo Electrónico
auth-login-username = Nombre de Usuario
auth-login-password = Contraseña
auth-login-remember = Recordarme
auth-login-forgot = ¿Olvidaste tu contraseña?
auth-login-submit = Iniciar Sesión
auth-login-loading = Iniciando sesión...
auth-login-or = o continúa con
auth-login-no-account = ¿No tienes una cuenta?
auth-login-create-account = Crear una cuenta

# -----------------------------------------------------------------------------
# Passkey/WebAuthn
# -----------------------------------------------------------------------------
passkey-title = Llaves de Acceso
passkey-subtitle = Autenticación segura sin contraseña
passkey-description = Las llaves de acceso utilizan la biometría o PIN de tu dispositivo para un inicio de sesión seguro y resistente al phishing
passkey-what-is = ¿Qué es una llave de acceso?
passkey-benefits = Beneficios de las llaves de acceso
passkey-benefit-secure = Más seguro que las contraseñas
passkey-benefit-easy = Fácil de usar - sin contraseñas que recordar
passkey-benefit-fast = Inicio de sesión rápido con biometría
passkey-benefit-phishing = Resistente a ataques de phishing

# -----------------------------------------------------------------------------
# Passkey Registration
# -----------------------------------------------------------------------------
passkey-register-title = Configurar Llave de Acceso
passkey-register-subtitle = Crea una llave de acceso para un inicio de sesión más rápido y seguro
passkey-register-description = Tu dispositivo te pedirá verificar tu identidad usando huella dactilar, rostro o bloqueo de pantalla
passkey-register-button = Crear Llave de Acceso
passkey-register-name = Nombre de la Llave de Acceso
passkey-register-name-placeholder = ej: MacBook Pro, iPhone
passkey-register-name-hint = Dale un nombre a tu llave de acceso para identificarla después
passkey-register-loading = Configurando llave de acceso...
passkey-register-verifying = Verificando con tu dispositivo...
passkey-register-success = Llave de acceso creada con éxito
passkey-register-error = Error al crear llave de acceso
passkey-register-cancelled = Configuración de llave de acceso cancelada
passkey-register-not-supported = Tu navegador no soporta llaves de acceso

# -----------------------------------------------------------------------------
# Passkey Authentication
# -----------------------------------------------------------------------------
passkey-login-title = Iniciar Sesión con Llave de Acceso
passkey-login-subtitle = Usa tu llave de acceso para un inicio de sesión seguro sin contraseña
passkey-login-button = Iniciar Sesión con Llave de Acceso
passkey-login-loading = Autenticando...
passkey-login-verifying = Verificando llave de acceso...
passkey-login-success = Sesión iniciada con éxito
passkey-login-error = Error en la autenticación
passkey-login-cancelled = Autenticación cancelada
passkey-login-no-passkeys = No se encontraron llaves de acceso para esta cuenta
passkey-login-try-another = Probar otro método

# -----------------------------------------------------------------------------
# Passkey Management
# -----------------------------------------------------------------------------
passkey-manage-title = Gestionar Llaves de Acceso
passkey-manage-subtitle = Ver y gestionar tus llaves de acceso registradas
passkey-manage-count = { $count ->
    [one] { $count } llave de acceso registrada
   *[other] { $count } llaves de acceso registradas
}
passkey-manage-add = Añadir Nueva Llave de Acceso
passkey-manage-rename = Renombrar
passkey-manage-delete = Eliminar
passkey-manage-created = Creada el { $date }
passkey-manage-last-used = Último uso el { $date }
passkey-manage-never-used = Nunca usada
passkey-manage-this-device = Este dispositivo
passkey-manage-cross-platform = Multiplataforma
passkey-manage-platform = Autenticador de plataforma
passkey-manage-security-key = Llave de seguridad
passkey-manage-empty = No hay llaves de acceso registradas
passkey-manage-empty-description = Añade una llave de acceso para un inicio de sesión más rápido y seguro

# -----------------------------------------------------------------------------
# Passkey Deletion
# -----------------------------------------------------------------------------
passkey-delete-title = Eliminar Llave de Acceso
passkey-delete-confirm = ¿Estás seguro de que quieres eliminar esta llave de acceso?
passkey-delete-warning = No podrás usar esta llave de acceso para iniciar sesión
passkey-delete-last-warning = Esta es tu única llave de acceso. Necesitarás usar autenticación con contraseña después de eliminarla.
passkey-delete-success = Llave de acceso eliminada con éxito
passkey-delete-error = Error al eliminar llave de acceso

# -----------------------------------------------------------------------------
# Password Fallback
# -----------------------------------------------------------------------------
passkey-fallback-title = Usar Contraseña
passkey-fallback-description = Si no puedes usar tu llave de acceso, puedes iniciar sesión con tu contraseña
passkey-fallback-button = Usar Contraseña
passkey-fallback-or-passkey = O inicia sesión con llave de acceso
passkey-fallback-setup-prompt = Configura una llave de acceso para un inicio de sesión más rápido la próxima vez
passkey-fallback-setup-later = Quizás después
passkey-fallback-setup-now = Configurar ahora
passkey-fallback-locked = Cuenta temporalmente bloqueada
passkey-fallback-locked-description = Demasiados intentos fallidos. Inténtalo de nuevo en { $minutes } minutos.
passkey-fallback-attempts = { $remaining } intentos restantes

# -----------------------------------------------------------------------------
# Multi-Factor Authentication
# -----------------------------------------------------------------------------
mfa-title = Autenticación de Dos Factores
mfa-subtitle = Añade una capa extra de seguridad a tu cuenta
mfa-enabled = La autenticación de dos factores está activada
mfa-disabled = La autenticación de dos factores está desactivada
mfa-enable = Activar 2FA
mfa-disable = Desactivar 2FA
mfa-setup = Configurar 2FA
mfa-verify = Verificar Código
mfa-code = Código de Verificación
mfa-code-placeholder = Introduce el código de 6 dígitos
mfa-code-sent = Código enviado a { $destination }
mfa-code-expired = El código ha expirado
mfa-code-invalid = Código inválido
mfa-resend = Reenviar código
mfa-resend-in = Reenviar en { $seconds }s
mfa-methods = Métodos de Autenticación
mfa-method-app = Aplicación Autenticadora
mfa-method-sms = SMS
mfa-method-email = Correo Electrónico
mfa-method-passkey = Llave de Acceso
mfa-backup-codes = Códigos de Respaldo
mfa-backup-codes-description = Guarda estos códigos en un lugar seguro. Cada código solo puede usarse una vez.
mfa-backup-codes-remaining = { $count } códigos de respaldo restantes
mfa-backup-codes-generate = Generar Nuevos Códigos
mfa-backup-codes-download = Descargar Códigos
mfa-backup-codes-copy = Copiar Códigos

# -----------------------------------------------------------------------------
# Password Management
# -----------------------------------------------------------------------------
password-title = Contraseña
password-change = Cambiar Contraseña
password-current = Contraseña Actual
password-new = Nueva Contraseña
password-confirm = Confirmar Nueva Contraseña
password-requirements = Requisitos de la Contraseña
password-requirement-length = Al menos { $length } caracteres
password-requirement-uppercase = Al menos una letra mayúscula
password-requirement-lowercase = Al menos una letra minúscula
password-requirement-number = Al menos un número
password-requirement-special = Al menos un carácter especial
password-strength = Fortaleza de la Contraseña
password-strength-weak = Débil
password-strength-fair = Aceptable
password-strength-good = Buena
password-strength-strong = Fuerte
password-match = Las contraseñas coinciden
password-mismatch = Las contraseñas no coinciden
password-changed = Contraseña cambiada con éxito
password-change-error = Error al cambiar contraseña

# -----------------------------------------------------------------------------
# Password Reset
# -----------------------------------------------------------------------------
password-reset-title = Restablecer Contraseña
password-reset-subtitle = Introduce tu correo para recibir un enlace de restablecimiento
password-reset-email-sent = Correo de restablecimiento de contraseña enviado
password-reset-email-sent-description = Revisa tu correo para instrucciones de restablecimiento de contraseña
password-reset-invalid-token = Enlace de restablecimiento inválido o expirado
password-reset-success = Contraseña restablecida con éxito
password-reset-error = Error al restablecer contraseña

# -----------------------------------------------------------------------------
# Session Management
# -----------------------------------------------------------------------------
session-title = Sesiones Activas
session-subtitle = Gestiona tus sesiones activas en diferentes dispositivos
session-current = Sesión Actual
session-device = Dispositivo
session-location = Ubicación
session-last-active = Última Actividad
session-ip-address = Dirección IP
session-browser = Navegador
session-os = Sistema Operativo
session-sign-out = Cerrar Sesión
session-sign-out-all = Cerrar Todas las Otras Sesiones
session-sign-out-confirm = ¿Estás seguro de que quieres cerrar esta sesión?
session-sign-out-all-confirm = ¿Estás seguro de que quieres cerrar todas las otras sesiones?

# -----------------------------------------------------------------------------
# Security Settings
# -----------------------------------------------------------------------------
security-title = Seguridad
security-subtitle = Gestiona la configuración de seguridad de tu cuenta
security-overview = Resumen de Seguridad
security-last-login = Último Inicio de Sesión
security-password-last-changed = Último Cambio de Contraseña
security-security-checkup = Revisión de Seguridad
security-checkup-description = Revisa tu configuración de seguridad
security-recommendation = Recomendación
security-add-passkey = Añade una llave de acceso para un inicio de sesión más seguro
security-enable-mfa = Activa la autenticación de dos factores
security-update-password = Actualiza tu contraseña regularmente

# -----------------------------------------------------------------------------
# Error Messages
# -----------------------------------------------------------------------------
auth-error-invalid-credentials = Correo o contraseña inválidos
auth-error-account-locked = Cuenta bloqueada. Por favor, contacta con soporte.
auth-error-account-disabled = La cuenta ha sido desactivada
auth-error-email-not-verified = Por favor, verifica tu dirección de correo
auth-error-too-many-attempts = Demasiados intentos fallidos. Por favor, inténtalo más tarde.
auth-error-network = Error de red. Por favor, comprueba tu conexión.
auth-error-server = Error del servidor. Por favor, inténtalo más tarde.
auth-error-unknown = Ha ocurrido un error desconocido
auth-error-session-invalid = Sesión inválida. Por favor, inicia sesión de nuevo.
auth-error-token-expired = Tu sesión ha expirado. Por favor, inicia sesión de nuevo.
auth-error-unauthorized = No estás autorizado para realizar esta acción

# -----------------------------------------------------------------------------
# Success Messages
# -----------------------------------------------------------------------------
auth-success-login = Sesión iniciada con éxito
auth-success-logout = Sesión cerrada con éxito
auth-success-signup = Cuenta creada con éxito
auth-success-password-changed = Contraseña cambiada con éxito
auth-success-email-verified = Correo verificado con éxito
auth-success-mfa-enabled = Autenticación de dos factores activada
auth-success-mfa-disabled = Autenticación de dos factores desactivada
auth-success-session-terminated = Sesión terminada con éxito

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
auth-notify-new-login = Nuevo inicio de sesión desde { $device } en { $location }
auth-notify-password-changed = Tu contraseña ha sido cambiada
auth-notify-mfa-enabled = La autenticación de dos factores ha sido activada
auth-notify-passkey-added = Nueva llave de acceso añadida a tu cuenta
auth-notify-suspicious-activity = Actividad sospechosa detectada en tu cuenta
