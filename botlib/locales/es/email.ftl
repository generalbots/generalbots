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
