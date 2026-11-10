# Transactional email text (#2094). Subject and body only; MIME and envelope stay untouched.

email-greeting = Hallo { $name },
email-signoff = Mit freundlichen Grüßen,
email-team = Ihr General Bots Team
email-welcome-subject = Willkommen bei General Bots Cloud — Tarif { $plan }
email-welcome-body = Willkommen bei General Bots Cloud! Ihr Tarif { $plan } ist jetzt aktiv. Sie können jetzt Bots erstellen und Ihren Arbeitsbereich unter { $url } verwalten.
email-invoice-subject = Rechnung #{ $invoice } — General Bots Cloud
email-invoice-body = Eine neue Rechnung wurde erstellt. Rechnung: { $invoice }. Betrag: { $amount } { $currency }. Details finden Sie unter { $url }
email-payment-success-subject = Zahlung bestätigt — General Bots Cloud
email-payment-success-body = Ihre Zahlung über { $amount } { $currency } wurde bestätigt. Tarif: { $plan }. Vielen Dank für Ihr Abonnement!
email-subscription-activated-subject = Abonnement aktiviert — General Bots Cloud
email-subscription-activated-body = Ihr Abonnement { $plan } ist jetzt aktiv. Sie können es unter { $url } verwalten.
email-recurring-charge-subject = Wiederkehrende Abbuchung — Rechnung #{ $invoice }
email-recurring-charge-body = Ihr wiederkehrendes Abonnement wurde belastet. Rechnung: { $invoice }. Betrag: { $amount } { $currency }. Details finden Sie unter { $url }
email-payment-failed-subject = Zahlung fehlgeschlagen — Handlungsbedarf
email-payment-failed-body = Ihre Zahlung über { $amount } { $currency } ist fehlgeschlagen. Rechnung: { $invoice }. Bitte aktualisieren Sie Ihre Zahlungsmethode unter { $url }
email-subscription-cancelled-subject = Abonnement gekündigt — General Bots Cloud
email-subscription-cancelled-body = Ihr Abonnement { $plan } wurde gekündigt. Wenn Sie es reaktivieren möchten, besuchen Sie { $url }
email-grace-warning-subject = Zahlung überfällig — Warnung vor Dienstsperrung
email-grace-warning-body = Ihre Zahlung ist überfällig. Ihr Dienst wird in { $days } Tagen gesperrt. Bitte aktualisieren Sie Ihre Zahlungsmethode umgehend unter { $url }
email-invitation-subject = Einladung zu { $org }
email-invitation-body = Sie wurden eingeladen, { $org } beizutreten. Nehmen Sie Ihre Einladung unter { $url } an. Diese Einladung läuft am { $expires } ab.
email-quota-limit-reached = Sie haben Ihr Kontingent für { $metric } erreicht.
email-quota-threshold = Sie haben { $current }% Ihres Kontingents für { $metric } verbraucht (Schwelle: { $threshold }%).
email-metric-messages = Nachrichten
email-metric-storage = Speicher
email-metric-api-calls = API-Aufrufe
email-metric-bots = Bots
email-metric-users = Benutzer
email-metric-kb-documents = Wissensdatenbank-Dokumente
email-metric-apps = Apps
email-metric-emails-per-day = E-Mails pro Tag
email-metric-builds-per-day = Builds pro Tag
email-metric-uploads-per-hour = Uploads pro Stunde

# Sharing and mention notifications (#2094)
email-share-subject = { $sender } hat "{ $file }" mit Ihnen geteilt
email-share-download = { $file } herunterladen
email-share-link-expires = Dieser Link läuft am { $date } ab. Sie können ihn jederzeit widerrufen.
email-share-link-no-expiry = Dieser Link läuft nicht ab. Sie können ihn jederzeit widerrufen.
email-mention-subject = { $author } hat Sie in einem Kommentar erwähnt
email-mention-body = { $author } hat Sie in { $resource_type } ({ $resource_id }) erwähnt. "{ $preview }" Öffnen Sie die App, um den Beitrag zu sehen und zu antworten.
