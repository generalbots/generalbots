# General Bots - Error Messages (English)
# This file contains all error message translations

# =============================================================================
# HTTP Errors
# =============================================================================

error-http-400 = Mauvaise demande. Veuillez vérifier votre saisie.
error-http-401 = Authentification requise. Veuillez vous connecter.
error-http-403 = Vous n'êtes pas autorisé à accéder à cette ressource.
error-http-404 = { $entity } introuvable.
error-http-409 = Conflit : { $message }
error-http-429 = Trop de demandes. Veuillez patienter { $seconds } secondes.
error-http-500 = Erreur interne du serveur. Veuillez réessayer plus tard.
error-http-502 = Mauvaise passerelle. Le serveur a reçu une réponse non valide.
error-http-503 = Service temporairement indisponible. Veuillez réessayer plus tard.
error-http-504 = La demande a expiré après { $milliseconds }ms.

# =============================================================================
# Validation Errors
# =============================================================================

error-validation-required = { $field } est requis.
error-validation-email = S'il vous plaît, mettez une adresse email valide.
error-validation-url = Veuillez saisir une URL valide.
error-validation-phone = Veuillez entrer un numéro de téléphone valide.
error-validation-min-length = { $field } doit contenir au moins { $min } caractères.
error-validation-max-length = { $field } ne doit pas contenir plus de { $max } caractères.
error-validation-min-value = { $field } doit être au moins { $min }.
error-validation-max-value = { $field } ne doit pas être supérieur à { $max }.
error-validation-pattern = Le format { $field } n'est pas valide.
error-validation-unique = { $field } existe déjà.
error-validation-mismatch = { $field } ne correspond pas à { $other }.
error-validation-date-format = Veuillez saisir une date valide au format { $format }.
error-validation-date-past = Le { $field } doit être du passé.
error-validation-date-future = Le { $field } doit être dans le futur.

# =============================================================================
# Authentication Errors
# =============================================================================

error-auth-invalid-credentials = Email ou mot de passe invalide.
error-auth-account-locked = Votre compte a été verrouillé. Veuillez contacter l'assistance.
error-auth-account-disabled = Votre compte a été désactivé.
error-auth-session-expired = Votre session a expiré. Veuillez vous reconnecter.
error-auth-token-invalid = Jeton invalide ou expiré.
error-auth-token-missing = Un jeton d'authentification est requis.
error-auth-mfa-required = Une authentification multifacteur est requise.
error-auth-mfa-invalid = Code de vérification invalide.
error-auth-password-weak = Le mot de passe est trop faible. Veuillez utiliser un mot de passe plus fort.
error-auth-password-expired = Votre mot de passe a expiré. Veuillez le réinitialiser.

# =============================================================================
# Configuration Errors
# =============================================================================

error-config = Erreur de configuration : { $message }
error-config-missing = Configuration manquante : { $key }
error-config-invalid = Valeur de configuration invalide pour { $key } : { $reason }
error-config-file-not-found = Fichier de configuration introuvable : { $path }
error-config-parse = Échec de l'analyse de la configuration : { $message }

# =============================================================================
# Database Errors
# =============================================================================

error-database = Erreur de base de données : { $message }
error-database-connection = Échec de la connexion à la base de données.
error-database-timeout = L’opération de la base de données a expiré.
error-database-constraint = Violation des contraintes de base de données : { $constraint }
error-database-duplicate = Un enregistrement avec ce { $field } existe déjà.
error-database-migration = Échec de la migration de la base de données : { $message }

# =============================================================================
# File & Storage Errors
# =============================================================================

error-file-not-found = Fichier introuvable : { $filename }
error-file-too-large = Le fichier est trop volumineux. La taille maximale est de { $maxSize }.
error-file-type-not-allowed = Type de fichier non autorisé. Types autorisés : { $allowedTypes }.
error-file-upload-failed = Échec du téléchargement du fichier : { $message }
error-file-read = Échec de la lecture du fichier : { $message }
error-file-write = Échec de l'écriture du fichier : { $message }
error-storage-full = Quota de stockage dépassé.
error-storage-unavailable = Le service de stockage n'est pas disponible.

# =============================================================================
# Network & External Service Errors
# =============================================================================

error-network = Erreur réseau : { $message }
error-network-timeout = La connexion a expiré.
error-network-unreachable = Le serveur est inaccessible.
error-service-unavailable = Service indisponible : { $service }
error-external-api = Erreur API externe : { $message }
error-rate-limit = Tarif limité. Réessayez après { $seconds }s.

# =============================================================================
# Bot & Dialog Errors
# =============================================================================

error-bot-not-found = Bot introuvable : { $botId }
error-bot-disabled = Ce bot est actuellement désactivé.
error-bot-script-error = Erreur de script à la ligne { $line } : { $message }
error-bot-timeout = La réponse du robot a expiré.
error-bot-quota-exceeded = Quota d'utilisation du robot dépassé.
error-dialog-not-found = Boîte de dialogue introuvable : { $dialogId }
error-dialog-invalid = Configuration de boîte de dialogue invalide : { $message }

# =============================================================================
# LLM & AI Errors
# =============================================================================

error-llm-unavailable = Le service IA est actuellement indisponible.
error-llm-timeout = La requête AI a expiré.
error-llm-rate-limit = Limite de débit IA dépassée. Veuillez patienter avant de réessayer.
error-llm-content-filter = Le contenu a été filtré selon les consignes de sécurité.
error-llm-context-length = La saisie est trop longue. Veuillez raccourcir votre message.
error-llm-invalid-response = Réponse non valide reçue du service AI.
error-llm-empty-response = Désolé, je n'ai pas pu traiter votre message pour le moment. Veuillez réessayer dans quelques secondes.

# =============================================================================
# Email Errors
# =============================================================================

error-email-send-failed = Échec de l'envoi de l'e-mail : { $message }
error-email-invalid-recipient = Adresse e-mail du destinataire invalide : { $email }
error-email-attachment-failed = Échec de la pièce jointe : { $filename }
error-email-template-not-found = Modèle d'e-mail introuvable : { $template }

# =============================================================================
# Calendar & Scheduling Errors
# =============================================================================

error-calendar-conflict = Le créneau horaire est en conflit avec l'événement existant.
error-calendar-past-date = Impossible de planifier des événements dans le passé.
error-calendar-invalid-recurrence = Modèle de récurrence non valide.
error-calendar-event-not-found = Événement introuvable : { $eventId }

# =============================================================================
# Task Errors
# =============================================================================

error-task-not-found = Tâche introuvable : { $taskId }
error-task-already-completed = La tâche est déjà terminée.
error-task-circular-dependency = Dépendance circulaire détectée dans les tâches.
error-task-invalid-status = Transition de statut de tâche non valide.

# =============================================================================
# Permission Errors
# =============================================================================

error-permission-denied = Vous n'êtes pas autorisé à effectuer cette action.
error-permission-resource = Vous n'avez pas accès à ce { $resource }.
error-permission-action = Vous ne pouvez pas { $action } ce { $resource }.
error-permission-owner-only = Seul le propriétaire peut effectuer cette action.

# =============================================================================
# Generic Errors
# =============================================================================

error-internal = Erreur interne : { $message }
error-unexpected = Une erreur inattendue s'est produite. Veuillez réessayer.
error-not-implemented = Cette fonctionnalité n'est pas encore implémentée.
error-maintenance = Le système est en maintenance. Veuillez réessayer plus tard.
error-unknown = Une erreur inconnue s'est produite.

error-locale-invalid = La langue demandée n'est pas valide.
error-locale-unsupported = Cette langue n'est pas disponible.
error-locale-persist-failed = Impossible d'enregistrer votre préférence de langue.
error-bot-id-required = L'identifiant du bot est requis.
error-bot-id-invalid = L'identifiant du bot n'est pas valide.
error-bot-name-required = Le nom du bot est requis.
error-token-invalid-format = Le format du jeton n'est pas valide.
error-token-signing-failed = Impossible de signer le jeton.
error-token-signature-invalid = La signature du jeton n'est pas valide.
error-token-payload-invalid = La charge utile du jeton n'est pas valide.
error-token-payload-json-invalid = La charge utile du jeton n'est pas un JSON valide.
error-cloud-token-required = Fournissez un cloud_token.
error-llm-subscription-token-missing = Jeton d'abonnement manquant.
error-llm-token-store-unavailable = Le stockage de jetons est indisponible.
error-llm-subscription-token-expired = Le jeton d'abonnement a expiré ; envoyez à nouveau le message.
error-llm-prompt-missing = La requête ne contient aucun prompt.
error-llm-account-unlinked = Connectez d'abord votre compte Claude ou ChatGPT dans Paramètres → Comptes IA.
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
error-autotask-schedule-invalid = The schedule must be a valid five-field cron expression.
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
