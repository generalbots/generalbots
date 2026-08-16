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
