# Transactional email text (#2094). Subject and body only; MIME and envelope stay untouched.

email-greeting = Hola { $name },
email-signoff = Atentamente,
email-team = Equipo de General Bots
email-welcome-subject = Bienvenido a General Bots Cloud — plan { $plan }
email-welcome-body = ¡Bienvenido a General Bots Cloud! Su plan { $plan } ya está activo. Puede empezar a crear bots y gestionar su espacio de trabajo en { $url }
email-invoice-subject = Factura #{ $invoice } — General Bots Cloud
email-invoice-body = Se ha creado una nueva factura. Factura: { $invoice }. Importe: { $amount } { $currency }. Vea los detalles en { $url }
email-payment-success-subject = Pago confirmado — General Bots Cloud
email-payment-success-body = Su pago de { $amount } { $currency } ha sido confirmado. Plan: { $plan }. ¡Gracias por su suscripción!
email-subscription-activated-subject = Suscripción activada — General Bots Cloud
email-subscription-activated-body = Su suscripción { $plan } ya está activa. Puede gestionarla en { $url }
email-recurring-charge-subject = Cargo recurrente — factura #{ $invoice }
email-recurring-charge-body = Se ha cobrado su suscripción recurrente. Factura: { $invoice }. Importe: { $amount } { $currency }. Vea los detalles en { $url }
email-payment-failed-subject = Pago fallido — se requiere acción
email-payment-failed-body = Su pago de { $amount } { $currency } ha fallado. Factura: { $invoice }. Actualice su método de pago en { $url }
email-subscription-cancelled-subject = Suscripción cancelada — General Bots Cloud
email-subscription-cancelled-body = Su suscripción { $plan } ha sido cancelada. Si desea reactivarla, visite { $url }
email-grace-warning-subject = Pago vencido — aviso de suspensión del servicio
email-grace-warning-body = Su pago está vencido. El servicio se suspenderá en { $days } días. Actualice su método de pago de inmediato en { $url }
email-invitation-subject = Invitación para unirse a { $org }
email-invitation-body = Ha sido invitado a unirse a { $org }. Acepte su invitación en { $url }. Esta invitación caduca el { $expires }.
email-quota-limit-reached = Ha alcanzado el límite de su cuota de { $metric }.
email-quota-threshold = Ha usado { $current }% de su cuota de { $metric } (umbral: { $threshold }%).
email-metric-messages = mensajes
email-metric-storage = almacenamiento
email-metric-api-calls = llamadas de API
email-metric-bots = bots
email-metric-users = usuarios
email-metric-kb-documents = documentos de la base de conocimiento
email-metric-apps = aplicaciones
email-metric-emails-per-day = correos por día
email-metric-builds-per-day = compilaciones por día
email-metric-uploads-per-hour = cargas por hora

# Sharing and mention notifications (#2094)
email-share-subject = { $sender } compartió "{ $file }" con usted
email-share-download = Descargar { $file }
email-share-link-expires = Este enlace caduca el { $date }. Puede revocarlo en cualquier momento.
email-share-link-no-expiry = Este enlace no caduca. Puede revocarlo en cualquier momento.
email-mention-subject = { $author } le mencionó en un comentario
email-mention-body = { $author } le mencionó en { $resource_type } ({ $resource_id }). "{ $preview }" Abra la aplicación para ver y responder.

# semantic i18n key (#2542)
mail-new-message = Nuevo mensaje

# semantic i18n key (#2542)
mail-cc-bcc = Cc/Cco

# semantic i18n key (#2542)
mail-subject-label = Asunto:

# semantic i18n key (#2542)
mail-track-opens = Rastrear aperturas

# semantic i18n key (#2542)
mail-request-read-receipt = Solicitar confirmación de lectura

# semantic i18n key (#2542)
mail-priority-normal = Prioridad normal

# semantic i18n key (#2542)
mail-priority-high = Alta prioridad

# semantic i18n key (#2542)
mail-priority-low = Baja prioridad

# semantic i18n key (#2542)
mail-dialog-templates-title = Plantillas de correo

# semantic i18n key (#2542)
mail-dialog-new-template = + Nueva plantilla

# semantic i18n key (#2542)
mail-dialog-signatures-title = Firmas de correo

# semantic i18n key (#2542)
mail-dialog-new-signature = + Nueva firma

# semantic i18n key (#2542)
mail-dialog-rules-title = Reglas de correo

# semantic i18n key (#2542)
mail-dialog-new-rule = + Nueva regla

# semantic i18n key (#2542)
mail-dialog-automatic-replies-title = Respuestas automáticas

# semantic i18n key (#2542)
mail-dialog-send-automatic-replies = Enviar respuestas automáticas

# semantic i18n key (#2542)
mail-dialog-start-date = Fecha de inicio

# semantic i18n key (#2542)
mail-dialog-end-date = Fecha de finalización

# semantic i18n key (#2542)
mail-dialog-reply-only-org = Enviar solo a personas de mi organización

# semantic i18n key (#2542)
mail-dialog-add-account-title = Agregar cuenta de correo

# semantic i18n key (#2542)
mail-dialog-account-name = Nombre de la cuenta

# semantic i18n key (#2542)
mail-dialog-imap-server = Servidor IMAP

# semantic i18n key (#2542)
mail-dialog-imap-port = Puerto IMAP

# semantic i18n key (#2542)
mail-dialog-smtp-server = Servidor SMTP

# semantic i18n key (#2542)
mail-dialog-smtp-port = Puerto SMTP

# semantic i18n key (#2542)
mail-dialog-use-ssl-tls = Usar SSL/TLS

# semantic i18n key (#2542)
mail-dialog-add-account-action = Agregar cuenta

# semantic i18n key (#2542)
mail-dialog-schedule-send-title = Programar envío

# semantic i18n key (#2542)
mail-dialog-schedule-action = Programar

# semantic i18n key (#2542)
mail-manage-labels-title = Administrar etiquetas

# semantic i18n key (#2542)
mail-label-name = Nombre de la etiqueta

# semantic i18n key (#2542)
mail-label-color = Color

# semantic i18n key (#2542)
mail-new-signature-title = Nueva firma de correo electrónico

# semantic i18n key (#2542)
mail-signature-name = Nombre de la firma

# semantic i18n key (#2542)
mail-html-content = Contenido HTML

# semantic i18n key (#2542)
mail-new-rule-title = Nueva regla de correo

# semantic i18n key (#2542)
mail-rule-name = Nombre de la regla

# semantic i18n key (#2542)
mail-condition-sender-contains = Condición (el remitente contiene)

# semantic i18n key (#2542)
mail-action-star = Destacar

# semantic i18n key (#2542)
mail-new-template-title = Nueva plantilla de correo

# semantic i18n key (#2542)
mail-template-name = Nombre de la plantilla

# semantic i18n key (#2542)
mail-body-html = Cuerpo HTML
