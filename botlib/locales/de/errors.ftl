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

error-locale-invalid = Die angeforderte Sprache ist ungültig.
error-locale-unsupported = Diese Sprache ist nicht verfügbar.
error-locale-persist-failed = Ihre Sprachauswahl konnte nicht gespeichert werden.
error-bot-id-required = Die Bot-ID ist erforderlich.
error-bot-id-invalid = Die Bot-ID ist ungültig.
error-bot-name-required = Der Bot-Name ist erforderlich.
error-token-invalid-format = Das Token-Format ist ungültig.
error-token-signing-failed = Das Token konnte nicht signiert werden.
error-token-signature-invalid = Die Token-Signatur ist ungültig.
error-token-payload-invalid = Die Token-Nutzlast ist ungültig.
error-token-payload-json-invalid = Die Token-Nutzlast ist kein gültiges JSON.
error-cloud-token-required = Geben Sie ein cloud_token an.
error-llm-subscription-token-missing = Abonnement-Token fehlt.
error-llm-token-store-unavailable = Der Token-Speicher ist nicht verfügbar.
error-llm-subscription-token-expired = Das Abonnement-Token ist abgelaufen; senden Sie die Nachricht erneut.
error-llm-prompt-missing = Die Anfrage enthält keinen Prompt.
error-llm-account-unlinked = Verbinden Sie zuerst Ihr Claude- oder ChatGPT-Konto unter Einstellungen → KI-Konten.
attendance-no-attendant-matching-skills = No attendant was found with the required skills.
attendance-response-sent = Response sent.
error-agent-drive-bad-bot = The bot record for this conversation is not valid.
error-agent-drive-bad-owner = The conversation owner record is not valid.
error-agent-drive-bad-path = The Drive path returned by the store is not valid.
error-agent-drive-conversation-not-found = Conversation not found.
error-agent-drive-database-unavailable = The file store is temporarily unavailable.
error-agent-drive-file-missing = The file is not in the workspace output folder.
error-agent-drive-forbidden = This conversation belongs to another user.
error-agent-drive-link-failed = Could not create the share link.
error-agent-drive-lookup-failed = Could not read the conversation. Please try again.
error-agent-drive-named-files-missing = The requested files are not in the workspace output folder.
error-agent-drive-no-files = There are no files in the workspace output folder.
error-agent-drive-save-failed = Could not save any file to Drive.
error-agent-drive-save-one-failed = Could not save the file to Drive.
error-agent-drive-workspace-stopped = The agent workspace is not running.
error-agent-vm-disabled = Agent/VM is disabled for this conversation. Turn it on in the chat controls to use the workspace.
error-ai-account-agent-invalid = The agent must be reasoning, agentic or fast.
error-ai-account-not-connected-chat = Connect the account before selecting it for chat.
error-ai-account-not-connected-vibe = Connect the account before assigning it to Vibe.
error-ai-account-unknown-provider = The AI account provider is unknown.
error-ai-account-workspace-unavailable = The agent workspace is not available in this build.
error-attendance-attendant-id-required = An attendant id is required.
error-attendance-session-id-invalid = The session id is invalid.
error-attendance-session-not-found = Session not found.
error-attendance-webhook-not-found = Webhook not found.
error-autotask-fields-required = Title, schedule and command are required.
error-autotask-intent-required = Describe what the task should do.
error-autotask-kind-invalid = The kind must be 'cron' or 'systemd'.
error-autotask-not-found = Autotask not found.
error-collab-comment-body-required = A comment body is required.
error-collab-comment-not-found = The comment was not found.
error-collab-comment-resolve-forbidden = The comment was not found, or it is not yours to resolve.
error-collab-comment-too-long = The comment is limited to 8000 characters.
error-collab-content-required = Content is required.
error-collab-content-too-large = The content is too large.
error-collab-grant-not-found = The grant was not found.
error-collab-invalid-action = The action is invalid.
error-collab-invalid-base-version = The base version is invalid.
error-collab-invalid-cursor = The 'before' cursor is invalid.
error-collab-invalid-emoji = The emoji is invalid.
error-collab-invalid-grantee-id = The grantee id is invalid.
error-collab-invalid-grantee-type = The grantee type must be 'user' or 'domain'.
error-collab-invalid-name = The name is invalid.
error-collab-invalid-new-owner = The new owner id is invalid.
error-collab-invalid-op-type = The operation type is invalid.
error-collab-invalid-parent = The parent id is invalid.
error-collab-invalid-resolution = The resolution must be 'accept-server' or 'accept-client'.
error-collab-invalid-resource = The resource type or resource id is invalid.
error-collab-invalid-role = The role is invalid.
error-collab-link-not-found = The share link was not found.
error-collab-not-a-conflict = The operation is not an unresolved conflict.
error-collab-operation-not-found = The operation was not found.
error-collab-owner-only = Only the owner can do this.
error-collab-version-not-found = The version was not found.
error-computer-use-action-id-mismatch = The action id does not match the path.
error-computer-use-action-not-found = Action not found.
error-computer-use-browser-only = This action must run in the browser panel.
error-computer-use-kind-invalid = The kind must be 'browser' or 'terminal'.
error-computer-use-no-consent = There is no consent record for this action; approve it first.
error-computer-use-session-required = A session id is required.
error-computer-use-url-missing = The action has no URL.
error-conversation-not-found = The conversation was not found.
error-conversation-session-invalid = The conversation session is invalid.
error-db-unavailable = The database is unavailable right now. Please try again.
error-drive-sync-conflict-path-escape = The conflict path escapes its sync folder.
error-drive-sync-conflict-path-required = A conflict path is required.
error-drive-sync-field-absolute = { $field } must be an absolute path.
error-drive-sync-field-drive-folder = The Drive folder
error-drive-sync-field-local-folder = The local folder
error-drive-sync-field-not-folder = { $field } is not a folder on this machine: { $path }
error-drive-sync-field-parent = { $field } must not contain '..'.
error-drive-sync-field-required = { $field } is required.
error-drive-sync-job-exists = A sync job named { $name } already exists.
error-drive-sync-job-name-required = A sync job name is required.
error-drive-sync-job-not-found = Sync job { $id } was not found.
error-drive-sync-task-failed = The { $task } task failed: { $error }
error-file-archive-unsafe = The archive was refused: { $reason }.
error-file-content-mismatch = The file content does not match its type ({ $expected } expected, { $detected } found).
error-file-empty = The uploaded file is empty.
error-file-extension-blocked = Files with the .{ $extension } extension are not allowed.
error-file-extension-unknown = The .{ $extension } extension is not recognised.
error-file-name-invalid = The file name is not valid.
error-goal-run-advance-refused = The run cannot advance in its current state.
error-goal-run-not-paused = This goal run is not paused.
error-goal-run-not-running = This goal run is not running.
error-group-budget-amount-invalid = The amount must be zero or positive.
error-group-budget-caps-invalid = The caps must be zero or positive (zero means no limit).
error-group-budget-invalid-id = The budget id is invalid.
error-group-budget-not-found = The group budget was not found.
error-http-no-route = No route matches this path.
error-invalid-value = The supplied value could not be parsed.
error-org-auth-service-unavailable = The directory service is not available.
error-org-default-delete = The default organization cannot be deleted.
error-org-name-required = The organization name is required.
error-org-settings-save-failed = Could not save the organization settings.
error-org-tenant-missing = No tenant exists yet — run the migrations or the bootstrap first.
error-peer-network-disabled = The peer network is disabled on this node.
error-peer-network-forbidden = An administrator session is required for this peer control.
error-peer-network-invalid = The peer request is not valid: { $message }
error-peer-network-payload-store-full = The peer payload store is full: { $message }
error-peer-network-recipient-required = The recipient query parameter is required.
error-peer-network-registry-full = The peer registry is full.
error-peer-network-unauthorized = A valid peer bearer token is required.
error-peer-network-unconfigured = This node has no peer token configured, so peer endpoints are closed.
error-peer-network-unknown-node = Unknown peer node: { $message }
error-people-invalid-end-date = The end date is invalid. Use the YYYY-MM-DD format.
error-people-invalid-start-date = The start date is invalid. Use the YYYY-MM-DD format.
error-person-not-found = Person not found.
error-product-not-found = Product not found.
error-service-invalid-value = The service value is invalid.
error-service-not-found = Service not found.
error-service-price-required = This service has no price; provide a value.
error-subagent-invalid-tasks = The sub-agent tasks are invalid.
error-subagent-no-llm = No LLM provider is configured for sub-agents.
error-team-not-found = Team not found.
error-terminal-action-id-required = Terminal execution requires an approved action id.
error-terminal-approval-required = Terminal execution requires explicit user approval.
error-terminal-blocked-pattern = The command matches a blocked destructive pattern.
error-terminal-command-required = A command is required.
error-terminal-command-too-long = The command is limited to 16000 characters.
error-terminal-timeout = The command timed out.
error-terminal-workspace-stopped = The agent workspace is not running. Finish provisioning (or restart the desktop app), then retry.
error-time-off-not-found = Time-off request not found.
error-value-required = A value is required for this service.
