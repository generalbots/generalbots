# General Bots - Error Messages (English)
# This file contains all error message translations

# =============================================================================
# HTTP Errors
# =============================================================================

error-http-400 = Ungültige Anforderung. Bitte überprüfen Sie Ihre Eingabe.
error-http-401 = Authentifizierung erforderlich. Bitte melden Sie sich an.
error-http-403 = Sie haben keine Berechtigung, auf diese Ressource zuzugreifen.
error-http-404 = { $entity } nicht gefunden.
error-http-409 = Konflikt: { $message }
error-http-429 = Zu viele Anfragen. Bitte warten Sie { $seconds } Sekunden.
error-http-500 = Interner Serverfehler. Bitte versuchen Sie es später noch einmal.
error-http-502 = Schlechtes Gateway. Der Server hat eine ungültige Antwort erhalten.
error-http-503 = Der Dienst ist vorübergehend nicht verfügbar. Bitte versuchen Sie es später noch einmal.
error-http-504 = Zeitüberschreitung der Anfrage nach { $milliseconds }ms.

# =============================================================================
# Validation Errors
# =============================================================================

error-validation-required = { $field } ist erforderlich.
error-validation-email = Bitte geben Sie eine gültige E-Mail-Adresse ein.
error-validation-url = Bitte geben Sie eine gültige URL ein.
error-validation-phone = Bitte geben Sie eine gültige Telefonnummer ein.
error-validation-min-length = { $field } muss mindestens { $min } Zeichen lang sein.
error-validation-max-length = { $field } darf nicht mehr als { $max } Zeichen lang sein.
error-validation-min-value = { $field } muss mindestens { $min } sein.
error-validation-max-value = { $field } darf nicht größer als { $max } sein.
error-validation-pattern = { $field } Format ist ungültig.
error-validation-unique = { $field } existiert bereits.
error-validation-mismatch = { $field } stimmt nicht mit { $other } überein.
error-validation-date-format = Bitte geben Sie ein gültiges Datum im Format { $format } ein.
error-validation-date-past = { $field } muss in der Vergangenheit liegen.
error-validation-date-future = { $field } muss in der Zukunft liegen.

# =============================================================================
# Authentication Errors
# =============================================================================

error-auth-invalid-credentials = Ungültige E-Mail-Adresse oder ungültiges Passwort.
error-auth-account-locked = Ihr Konto wurde gesperrt. Bitte wenden Sie sich an den Support.
error-auth-account-disabled = Ihr Konto wurde deaktiviert.
error-auth-session-expired = Ihre Sitzung ist abgelaufen. Bitte melden Sie sich erneut an.
error-auth-token-invalid = Ungültiges oder abgelaufenes Token.
error-auth-token-missing = Authentifizierungstoken ist erforderlich.
error-auth-mfa-required = Eine Multi-Faktor-Authentifizierung ist erforderlich.
error-auth-mfa-invalid = Ungültiger Bestätigungscode.
error-auth-password-weak = Das Passwort ist zu schwach. Bitte verwenden Sie ein stärkeres Passwort.
error-auth-password-expired = Ihr Passwort ist abgelaufen. Bitte setzen Sie es zurück.

# =============================================================================
# Configuration Errors
# =============================================================================

error-config = Konfigurationsfehler: { $message }
error-config-missing = Fehlende Konfiguration: { $key }
error-config-invalid = Ungültiger Konfigurationswert für { $key }: { $reason }
error-config-file-not-found = Konfigurationsdatei nicht gefunden: { $path }
error-config-parse = Konfiguration konnte nicht analysiert werden: { $message }

# =============================================================================
# Database Errors
# =============================================================================

error-database = Datenbankfehler: { $message }
error-database-connection = Verbindung zur Datenbank konnte nicht hergestellt werden.
error-database-timeout = Zeitüberschreitung beim Datenbankvorgang.
error-database-constraint = Verletzung der Datenbankeinschränkung: { $constraint }
error-database-duplicate = Ein Datensatz mit diesem { $field } existiert bereits.
error-database-migration = Datenbankmigration fehlgeschlagen: { $message }

# =============================================================================
# File & Storage Errors
# =============================================================================

error-file-not-found = Datei nicht gefunden: { $filename }
error-file-too-large = Die Datei ist zu groß. Die maximale Größe beträgt { $maxSize }.
error-file-type-not-allowed = Dateityp nicht zulässig. Zulässige Typen: { $allowedTypes }.
error-file-upload-failed = Datei-Upload fehlgeschlagen: { $message }
error-file-read = Datei konnte nicht gelesen werden: { $message }
error-file-write = Datei konnte nicht geschrieben werden: { $message }
error-storage-full = Speicherkontingent überschritten.
error-storage-unavailable = Der Speicherdienst ist nicht verfügbar.

# =============================================================================
# Network & External Service Errors
# =============================================================================

error-network = Netzwerkfehler: { $message }
error-network-timeout = Zeitüberschreitung bei der Verbindung.
error-network-unreachable = Der Server ist nicht erreichbar.
error-service-unavailable = Dienst nicht verfügbar: { $service }
error-external-api = Externer API-Fehler: { $message }
error-rate-limit = Preis begrenzt. Versuchen Sie es nach { $seconds }s erneut.

# =============================================================================
# Bot & Dialog Errors
# =============================================================================

error-bot-not-found = Bot nicht gefunden: { $botId }
error-bot-disabled = Dieser Bot ist derzeit deaktiviert.
error-bot-script-error = Skriptfehler in Zeile { $line }: { $message }
error-bot-timeout = Zeitüberschreitung bei der Bot-Antwort.
error-bot-quota-exceeded = Bot-Nutzungskontingent überschritten.
error-dialog-not-found = Dialog nicht gefunden: { $dialogId }
error-dialog-invalid = Ungültige Dialogkonfiguration: { $message }

# =============================================================================
# LLM & AI Errors
# =============================================================================

error-llm-unavailable = Der AI-Dienst ist derzeit nicht verfügbar.
error-llm-timeout = Zeitüberschreitung bei der AI-Anfrage.
error-llm-rate-limit = AI-Ratenlimit überschritten. Bitte warten Sie, bevor Sie es erneut versuchen.
error-llm-content-filter = Der Inhalt wurde nach Sicherheitsrichtlinien gefiltert.
error-llm-context-length = Die Eingabe ist zu lang. Bitte kürzen Sie Ihre Nachricht.
error-llm-invalid-response = Ungültige Antwort vom AI-Dienst erhalten.
error-llm-empty-response = Leider konnte ich Ihre Nachricht im Moment nicht verarbeiten. Bitte versuchen Sie es in ein paar Sekunden noch einmal.

# =============================================================================
# Email Errors
# =============================================================================

error-email-send-failed = E-Mail konnte nicht gesendet werden: { $message }
error-email-invalid-recipient = Ungültige E-Mail-Adresse des Empfängers: { $email }
error-email-attachment-failed = Datei konnte nicht angehängt werden: { $filename }
error-email-template-not-found = E-Mail-Vorlage nicht gefunden: { $template }

# =============================================================================
# Calendar & Scheduling Errors
# =============================================================================

error-calendar-conflict = Zeitfensterkonflikt mit bestehender Veranstaltung.
error-calendar-past-date = Es können keine Ereignisse in der Vergangenheit geplant werden.
error-calendar-invalid-recurrence = Ungültiges Wiederholungsmuster.
error-calendar-event-not-found = Ereignis nicht gefunden: { $eventId }

# =============================================================================
# Task Errors
# =============================================================================

error-task-not-found = Aufgabe nicht gefunden: { $taskId }
error-task-already-completed = Die Aufgabe wurde bereits erledigt.
error-task-circular-dependency = In Aufgaben wurde eine zirkuläre Abhängigkeit erkannt.
error-task-invalid-status = Ungültiger Aufgabenstatusübergang.

# =============================================================================
# Permission Errors
# =============================================================================

error-permission-denied = Sie haben keine Berechtigung, diese Aktion auszuführen.
error-permission-resource = Sie haben keinen Zugriff darauf { $resource }.
error-permission-action = Sie können dies nicht { $action } { $resource } tun.
error-permission-owner-only = Nur der Eigentümer kann diese Aktion ausführen.

# =============================================================================
# Generic Errors
# =============================================================================

error-internal = Interner Fehler: { $message }
error-unexpected = Es ist ein unerwarteter Fehler aufgetreten. Bitte versuchen Sie es erneut.
error-not-implemented = Diese Funktion ist noch nicht implementiert.
error-maintenance = Das System wird gewartet. Bitte versuchen Sie es später noch einmal.
error-unknown = Es ist ein unbekannter Fehler aufgetreten.
