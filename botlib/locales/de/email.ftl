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

# semantic i18n key (#2542)
mail-new-message = Neue Nachricht

# semantic i18n key (#2542)
mail-cc-bcc = Cc/Bcc

# semantic i18n key (#2542)
mail-subject-label = Betreff:

# semantic i18n key (#2542)
mail-track-opens = Öffnungen verfolgen

# semantic i18n key (#2542)
mail-request-read-receipt = Lesebestätigung anfordern

# semantic i18n key (#2542)
mail-priority-normal = Normale Priorität

# semantic i18n key (#2542)
mail-priority-high = Hohe Priorität

# semantic i18n key (#2542)
mail-priority-low = Niedrige Priorität

# semantic i18n key (#2542)
mail-dialog-templates-title = E-Mail-Vorlagen

# semantic i18n key (#2542)
mail-dialog-new-template = + Neue Vorlage

# semantic i18n key (#2542)
mail-dialog-signatures-title = E-Mail-Signaturen

# semantic i18n key (#2542)
mail-dialog-new-signature = + Neue Signatur

# semantic i18n key (#2542)
mail-dialog-rules-title = E-Mail-Regeln

# semantic i18n key (#2542)
mail-dialog-new-rule = + Neue Regel

# semantic i18n key (#2542)
mail-dialog-automatic-replies-title = Automatische Antworten

# semantic i18n key (#2542)
mail-dialog-send-automatic-replies = Automatische Antworten senden

# semantic i18n key (#2542)
mail-dialog-start-date = Startdatum

# semantic i18n key (#2542)
mail-dialog-end-date = Enddatum

# semantic i18n key (#2542)
mail-dialog-reply-only-org = Nur an Personen in meiner Organisation senden

# semantic i18n key (#2542)
mail-dialog-add-account-title = E-Mail-Konto hinzufügen

# semantic i18n key (#2542)
mail-dialog-account-name = Kontoname

# semantic i18n key (#2542)
mail-dialog-imap-server = IMAP-Server

# semantic i18n key (#2542)
mail-dialog-imap-port = IMAP-Port

# semantic i18n key (#2542)
mail-dialog-smtp-server = SMTP-Server

# semantic i18n key (#2542)
mail-dialog-smtp-port = SMTP-Port

# semantic i18n key (#2542)
mail-dialog-use-ssl-tls = SSL/TLS verwenden

# semantic i18n key (#2542)
mail-dialog-add-account-action = Konto hinzufügen

# semantic i18n key (#2542)
mail-dialog-schedule-send-title = Senden planen

# semantic i18n key (#2542)
mail-dialog-schedule-action = Planen

# semantic i18n key (#2542)
mail-manage-labels-title = Labels verwalten

# semantic i18n key (#2542)
mail-label-name = Labelname

# semantic i18n key (#2542)
mail-label-color = Farbe

# semantic i18n key (#2542)
mail-new-signature-title = Neue E-Mail-Signatur

# semantic i18n key (#2542)
mail-signature-name = Signaturname

# semantic i18n key (#2542)
mail-html-content = HTML-Inhalt

# semantic i18n key (#2542)
mail-new-rule-title = Neue E-Mail-Regel

# semantic i18n key (#2542)
mail-rule-name = Regelname

# semantic i18n key (#2542)
mail-condition-sender-contains = Bedingung (Absender enthält)

# semantic i18n key (#2542)
mail-action-star = Markieren

# semantic i18n key (#2542)
mail-new-template-title = Neue E-Mail-Vorlage

# semantic i18n key (#2542)
mail-template-name = Vorlagenname

# semantic i18n key (#2542)
mail-body-html = HTML-Textkörper

# semantic i18n key (#2542)
mail-placeholder-label-name = z. B. Arbeit
mail-placeholder-signature-name = z. B. Professionelle Signatur
mail-placeholder-signature-content = Mit freundlichen Grüßen,<br><b>Ihr Name</b>

# semantic i18n key (#2542)
mail-placeholder-rule-name = z. B. Newsletter archivieren
mail-placeholder-rule-condition = z. B. Newsletter
mail-placeholder-template-name = z. B. Willkommens-E-Mail
mail-placeholder-template-subject = Standard-Willkommensbetreff
mail-placeholder-template-body = Hallo {{name}}, willkommen auf unserer Plattform!
mail-placeholder-autoreply-subject = Betreff der Antwort
mail-placeholder-autoreply-message = Ihre automatische Antwortnachricht...
