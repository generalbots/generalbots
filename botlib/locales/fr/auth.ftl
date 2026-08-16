# =============================================================================
# General Bots - Authentication Translations (English)
# =============================================================================
# Authentication, Passkey/WebAuthn, and security interface translations
# =============================================================================

# -----------------------------------------------------------------------------
# Authentication General
# -----------------------------------------------------------------------------
auth-title = Authentification
auth-login = Connectez-vous
auth-logout = Se déconnecter
auth-signup = S'inscrire
auth-welcome = Bienvenue
auth-welcome-back = Bon retour, { $name } !
auth-session-expired = Votre session a expiré
auth-session-timeout = Délai d'expiration de la session dans { $minutes } minutes

# -----------------------------------------------------------------------------
# Login Form
# -----------------------------------------------------------------------------
auth-login-title = Connectez-vous à votre compte
auth-login-subtitle = Entrez vos identifiants pour continuer
auth-login-email = Adresse e-mail
auth-login-username = Nom d'utilisateur
auth-login-password = Mot de passe
auth-login-remember = Souviens-toi de moi
auth-login-forgot = Mot de passe oublié ?
auth-login-submit = Connectez-vous
auth-login-loading = Connexion...
auth-login-or = ou continuez avec
auth-login-no-account = Vous n'avez pas de compte ?
auth-login-create-account = Créer un compte

# -----------------------------------------------------------------------------
# Passkey/WebAuthn
# -----------------------------------------------------------------------------
passkey-title = Mots-clés
passkey-subtitle = Authentification sécurisée et sans mot de passe
passkey-description = Les clés d'accès utilisent les données biométriques ou le code PIN de votre appareil pour une connexion sécurisée et résistante au phishing
passkey-what-is = Qu'est-ce qu'un mot de passe ?
passkey-benefits = Avantages des mots de passe
passkey-benefit-secure = Plus sécurisé que les mots de passe
passkey-benefit-easy = Facile à utiliser - aucun mot de passe à retenir
passkey-benefit-fast = Connexion rapide avec la biométrie
passkey-benefit-phishing = Résistant aux attaques de phishing

# -----------------------------------------------------------------------------
# Passkey Registration
# -----------------------------------------------------------------------------
passkey-register-title = Configurer la clé d'accès
passkey-register-subtitle = Créez un mot de passe pour une connexion plus rapide et plus sécurisée
passkey-register-description = Votre appareil vous demandera de vérifier votre identité à l'aide de votre empreinte digitale, de votre visage ou du verrouillage de l'écran.
passkey-register-button = Créer un mot de passe
passkey-register-name = Nom de la clé d'accès
passkey-register-name-placeholder = par exemple, MacBook Pro, iPhone
passkey-register-name-hint = Donnez un nom à votre mot de passe pour l'identifier plus tard
passkey-register-loading = Configuration du mot de passe...
passkey-register-verifying = Vérification avec votre appareil...
passkey-register-success = Clé d'accès créée avec succès
passkey-register-error = Échec de la création du mot de passe
passkey-register-cancelled = Configuration du mot de passe annulée
passkey-register-not-supported = Votre navigateur ne prend pas en charge les mots de passe

# -----------------------------------------------------------------------------
# Passkey Authentication
# -----------------------------------------------------------------------------
passkey-login-title = Connectez-vous avec le mot de passe
passkey-login-subtitle = Utilisez votre mot de passe pour une connexion sécurisée et sans mot de passe
passkey-login-button = Connectez-vous avec le mot de passe
passkey-login-loading = Authentification...
passkey-login-verifying = Vérification du mot de passe...
passkey-login-success = Connecté avec succès
passkey-login-error = L'authentification a échoué
passkey-login-cancelled = Authentification annulée
passkey-login-no-passkeys = Aucun mot de passe trouvé pour ce compte
passkey-login-try-another = Essayez une autre méthode

# -----------------------------------------------------------------------------
# Passkey Management
# -----------------------------------------------------------------------------
passkey-manage-title = Gérer les clés d'accès
passkey-manage-subtitle = Afficher et gérer vos mots de passe enregistrés
passkey-manage-count = { $compte ->
    [one] { $count } passkey registered
   *[other] { $count } passkeys registered
}
passkey-manage-add = Ajouter une nouvelle clé d'accès
passkey-manage-rename = Renommer
passkey-manage-delete = Supprimer
passkey-manage-created = Créé { $date }
passkey-manage-last-used = Dernière utilisation { $date }
passkey-manage-never-used = Jamais utilisé
passkey-manage-this-device = Cet appareil
passkey-manage-cross-platform = Multiplateforme
passkey-manage-platform = Authentificateur de plateforme
passkey-manage-security-key = Clé de sécurité
passkey-manage-empty = Aucun mot de passe enregistré
passkey-manage-empty-description = Ajoutez un mot de passe pour une connexion plus rapide et plus sécurisée

# -----------------------------------------------------------------------------
# Passkey Deletion
# -----------------------------------------------------------------------------
passkey-delete-title = Supprimer le mot de passe
passkey-delete-confirm = Êtes-vous sûr de vouloir supprimer ce mot de passe ?
passkey-delete-warning = Vous ne pourrez plus utiliser ce mot de passe pour vous connecter
passkey-delete-last-warning = C'est votre seul mot de passe. Vous devrez utiliser l'authentification par mot de passe après l'avoir supprimé.
passkey-delete-success = Clé d'accès supprimée avec succès
passkey-delete-error = Échec de la suppression du mot de passe

# -----------------------------------------------------------------------------
# Password Fallback
# -----------------------------------------------------------------------------
passkey-fallback-title = Utilisez plutôt un mot de passe
passkey-fallback-description = Si vous ne pouvez pas utiliser votre mot de passe, vous pouvez vous connecter avec votre mot de passe
passkey-fallback-button = Utiliser le mot de passe
passkey-fallback-or-passkey = Ou connectez-vous avec un mot de passe
passkey-fallback-setup-prompt = Configurez un mot de passe pour une connexion plus rapide la prochaine fois
passkey-fallback-setup-later = Peut-être plus tard
passkey-fallback-setup-now = Configurer maintenant
passkey-fallback-locked = Compte temporairement verrouillé
passkey-fallback-locked-description = Trop de tentatives infructueuses. Réessayez dans { $minutes } minutes.
passkey-fallback-attempts = { $remaining } tentatives restantes

# -----------------------------------------------------------------------------
# Multi-Factor Authentication
# -----------------------------------------------------------------------------
mfa-title = Authentification à deux facteurs
mfa-subtitle = Ajoutez une couche de sécurité supplémentaire à votre compte
mfa-enabled = L'authentification à deux facteurs est activée
mfa-disabled = L'authentification à deux facteurs est désactivée
mfa-enable = Activer 2FA
mfa-disable = Désactiver 2FA
mfa-setup = Configurer 2FA
mfa-verify = Vérifier le code
mfa-code = Code de vérification
mfa-code-placeholder = Entrez le code à 6 chiffres
mfa-code-sent = Code envoyé au { $destination }
mfa-code-expired = Le code a expiré
mfa-code-invalid = Code invalide
mfa-resend = Renvoyer le code
mfa-resend-in = Renvoyer dans { $seconds }s
mfa-methods = Méthodes d'authentification
mfa-method-app = Application d'authentification
mfa-method-sms = SMS
mfa-method-email = Courriel
mfa-method-passkey = Clé d'accès
mfa-backup-codes = Codes de sauvegarde
mfa-backup-codes-description = Conservez ces codes dans un endroit sûr. Chaque code ne peut être utilisé qu'une seule fois.
mfa-backup-codes-remaining = { $count } codes de sauvegarde restants
mfa-backup-codes-generate = Générer de nouveaux codes
mfa-backup-codes-download = Codes de téléchargement
mfa-backup-codes-copy = Copier les codes

# -----------------------------------------------------------------------------
# Password Management
# -----------------------------------------------------------------------------
password-title = Mot de passe
password-change = Changer le mot de passe
password-current = Mot de passe actuel
password-new = Nouveau mot de passe
password-confirm = Confirmer le nouveau mot de passe
password-requirements = Exigences de mot de passe
password-requirement-length = Au moins { $length } caractères
password-requirement-uppercase = Au moins une lettre majuscule
password-requirement-lowercase = Au moins une lettre minuscule
password-requirement-number = Au moins un numéro
password-requirement-special = Au moins un caractère spécial
password-strength = Force du mot de passe
password-strength-weak = Faible
password-strength-fair = Foire
password-strength-good = Bon
password-strength-strong = Fort
password-match = Les mots de passe correspondent
password-mismatch = Les mots de passe ne correspondent pas
password-changed = Mot de passe modifié avec succès
password-change-error = Échec de la modification du mot de passe

# -----------------------------------------------------------------------------
# Password Reset
# -----------------------------------------------------------------------------
password-reset-title = Réinitialiser le mot de passe
password-reset-subtitle = Entrez votre email pour recevoir un lien de réinitialisation
password-reset-email-sent = E-mail de réinitialisation du mot de passe envoyé
password-reset-email-sent-description = Vérifiez votre courrier électronique pour obtenir des instructions pour réinitialiser votre mot de passe
password-reset-invalid-token = Lien de réinitialisation invalide ou expiré
password-reset-success = Mot de passe réinitialisé avec succès
password-reset-error = Échec de la réinitialisation du mot de passe

# -----------------------------------------------------------------------------
# Session Management
# -----------------------------------------------------------------------------
session-title = Séances actives
session-subtitle = Gérez vos sessions actives sur tous les appareils
session-current = Session en cours
session-device = Appareil
session-location = Emplacement
session-last-active = Dernier actif
session-ip-address = Adresse IP
session-browser = Navigateur
session-os = Système d'exploitation
session-sign-out = Se déconnecter
session-sign-out-all = Se déconnecter de toutes les autres sessions
session-sign-out-confirm = Êtes-vous sûr de vouloir vous déconnecter de cette session ?
session-sign-out-all-confirm = Êtes-vous sûr de vouloir vous déconnecter de toutes les autres sessions ?

# -----------------------------------------------------------------------------
# Security Settings
# -----------------------------------------------------------------------------
security-title = Sécurité
security-subtitle = Gérer les paramètres de sécurité de votre compte
security-overview = Présentation de la sécurité
security-last-login = Dernière connexion
security-password-last-changed = Mot de passe modifié pour la dernière fois
security-security-checkup = Vérification de sécurité
security-checkup-description = Vérifiez vos paramètres de sécurité
security-recommendation = Recommandation
security-add-passkey = Ajoutez un mot de passe pour une connexion plus sécurisée
security-enable-mfa = Activer l'authentification à deux facteurs
security-update-password = Mettez régulièrement à jour votre mot de passe

# -----------------------------------------------------------------------------
# Error Messages
# -----------------------------------------------------------------------------
auth-error-invalid-credentials = Email ou mot de passe invalide
auth-error-account-locked = Le compte est verrouillé. Veuillez contacter l'assistance.
auth-error-account-disabled = Le compte a été désactivé
auth-error-email-not-verified = Veuillez vérifier votre adresse e-mail
auth-error-too-many-attempts = Trop de tentatives infructueuses. Veuillez réessayer plus tard.
auth-error-network = Erreur réseau. Veuillez vérifier votre connexion.
auth-error-server = Erreur de serveur. Veuillez réessayer plus tard.
auth-error-unknown = Une erreur inconnue s'est produite
auth-error-session-invalid = Séance invalide. Veuillez vous reconnecter.
auth-error-token-expired = Votre session a expiré. Veuillez vous reconnecter.
auth-error-unauthorized = Vous n'êtes pas autorisé à effectuer cette action

# -----------------------------------------------------------------------------
# Success Messages
# -----------------------------------------------------------------------------
auth-success-login = Connecté avec succès
auth-success-logout = Déconnexion réussie
auth-success-signup = Compte créé avec succès
auth-success-password-changed = Mot de passe modifié avec succès
auth-success-email-verified = E-mail vérifié avec succès
auth-success-mfa-enabled = Authentification à deux facteurs activée
auth-success-mfa-disabled = Authentification à deux facteurs désactivée
auth-success-session-terminated = Session terminée avec succès

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
auth-notify-new-login = Nouvelle connexion du { $device } au { $location }
auth-notify-password-changed = Votre mot de passe a été modifié
auth-notify-mfa-enabled = L'authentification à deux facteurs a été activée
auth-notify-passkey-added = Un nouveau mot de passe a été ajouté à votre compte
auth-notify-suspicious-activity = Activité suspecte détectée sur votre compte
