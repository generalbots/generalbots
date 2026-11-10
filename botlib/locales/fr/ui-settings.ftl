# =============================================================================
# General Bots — fr — Settings
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Settings
# -----------------------------------------------------------------------------
settings-title = Paramètres
settings-general = Général
settings-account = Compte
settings-notifications = Notifications
settings-privacy = Confidentialité
settings-security = Paramètres de sécurité
settings-language = Langue
settings-theme = Thème
settings-theme-light = Lumière
settings-theme-dark = Sombre
settings-theme-system = Système
settings-save = Enregistrer les modifications
settings-saved = Paramètres enregistrés avec succès
settings-create-api-key = Créer une clé API
settings-api-key-name = Nom de la clé
settings-api-key-scopes = Portées (séparées par des virgules)
settings-api-key-scopes-hint = Laissez vide pour la portée « utilisateur » par défaut.
settings-api-key-expiry = Expire dans (jours)
settings-api-key-expiry-hint = Laissez vide sans expiration.
settings-add-webhook = Ajouter un webhook
settings-webhook-url = URL du webhook
settings-webhook-events = Événements (séparés par des virgules)
settings-webhook-events-hint = Laissez vide pour recevoir tous les événements.
settings-timezone = Fuseau horaire
settings-date-format = Format des dates
settings-time-format = Format de l'heure

# -----------------------------------------------------------------------------
# Settings - Language & Localization
# -----------------------------------------------------------------------------
settings-language-desc = Choisissez votre langue préférée
settings-display-language = Langue d'affichage
settings-language-affects = Affecte tout le texte de l'application
settings-date-format-desc = Comment les dates sont affichées
settings-time-format-desc = Horloge 12 heures ou 24 heures
settings-language-changed = La langue a changé avec succès
settings-reload-required = Rechargement de la page requis pour appliquer les modifications

# Settings - Profile
settings-profile-desc = Gérer vos informations personnelles et vos préférences
settings-profile-photo = Photo de profil
settings-profile-photo-desc = Votre photo de profil est visible par les autres utilisateurs
settings-upload-photo = Télécharger une photo
settings-remove-photo = Supprimer
settings-basic-info = Informations de base
settings-display-name = Nom d'affichage
settings-username = Nom d'utilisateur
settings-email-address = Adresse e-mail
settings-bio = Biographie
settings-bio-placeholder = Parlez-nous de vous...
settings-contact-info = Coordonnées
settings-phone-number = Numéro de téléphone
settings-location = Emplacement
settings-website = Site Web

# Settings - Security
settings-security-desc = Protégez votre compte avec une sécurité renforcée
settings-change-password = Changer le mot de passe
settings-change-password-desc = Mettez régulièrement à jour votre mot de passe pour une meilleure sécurité
settings-current-password = Mot de passe actuel
settings-new-password = Nouveau mot de passe
settings-confirm-password = Confirmer le nouveau mot de passe
settings-update-password = Mettre à jour le mot de passe
settings-2fa = Authentification à deux facteurs
settings-2fa-desc = Ajoutez une couche de sécurité supplémentaire à votre compte
settings-authenticator-app = Application d'authentification
settings-authenticator-desc = Utilisez une application d'authentification pour les codes 2FA
settings-enable-2fa = Activer 2FA
settings-disable-2fa = Désactiver 2FA
settings-active-sessions = Séances actives
settings-active-sessions-desc = Gérez vos sessions de connexion actives
settings-this-device = Cet appareil
settings-terminate-session = Terminer
settings-terminate-all = Terminer toutes les autres sessions

# Settings - Appearance
settings-appearance = Apparence
settings-appearance-desc = Personnalisez l'apparence de l'application
settings-theme-selection = Thème
settings-theme-selection-desc = Choisissez votre thème de couleur préféré
settings-theme-blue = Bleu
settings-theme-purple = Violet
settings-theme-green = Vert
settings-theme-orange = Orange
settings-layout-preferences = Préférences de mise en page
settings-compact-mode = Mode compact
settings-compact-mode-desc = Réduisez l’espace pour plus de contenu
settings-show-sidebar = Afficher la barre latérale
settings-show-sidebar-desc = Toujours afficher la barre latérale de navigation
settings-animations = Animations
settings-animations-desc = Activer les animations et les transitions de l'interface utilisateur

# Settings - Notifications
settings-notifications-title = Notifications
settings-notifications-desc = Contrôlez la façon dont vous recevez des notifications
settings-email-notifications = Notifications par courrier électronique
settings-direct-messages = Messages directs
settings-direct-messages-desc = Recevoir un e-mail pour les nouveaux messages directs
settings-mentions = Mentionné
settings-mentions-desc = Recevez un e-mail lorsque quelqu'un vous mentionne
settings-weekly-digest = Résumé hebdomadaire
settings-weekly-digest-desc = Obtenez un résumé hebdomadaire de l'activité
settings-marketing = Commercialisation
settings-marketing-desc = Recevez des actualités et des mises à jour de produits
settings-push-notifications = Notifications poussées
settings-enable-push = Activer les notifications push
settings-enable-push-desc = Recevoir les notifications push du navigateur
settings-notification-sound = Son
settings-notification-sound-desc = Jouer du son pour les notifications
settings-in-app-notifications = Notifications dans l'application

# Settings - Storage
settings-storage = Stockage
settings-storage-desc = Gérez votre utilisation du stockage
settings-storage-usage = Utilisation du stockage
settings-storage-used = { $used } sur { $total } utilisés
settings-storage-upgrade = Mettre à niveau le stockage

# Settings - Privacy
settings-privacy-title = Confidentialité
settings-privacy-desc = Contrôlez vos paramètres de confidentialité
settings-data-collection = Collecte de données
settings-analytics = Analyse
settings-analytics-desc = Aidez-nous à nous améliorer en envoyant des données d'utilisation anonymes
settings-crash-reports = Rapports d'erreur
settings-crash-reports-desc = Envoyer automatiquement des rapports d'erreur
settings-download-data = Téléchargez vos données
settings-download-data-desc = Obtenez une copie de toutes vos données
settings-delete-account = Supprimer le compte
settings-delete-account-desc = Supprimez définitivement votre compte et toutes vos données
settings-delete-account-warning = Cette action ne peut pas être annulée

# Settings - Billing
settings-billing = Facturation
settings-billing-desc = Gérez votre abonnement et vos moyens de paiement
settings-current-plan = Forfait actuel
settings-free-plan = Forfait gratuit
settings-pro-plan = Forfait Pro
settings-enterprise-plan = Forfait Entreprise
settings-upgrade-plan = Plan de mise à niveau
settings-payment-methods = Méthodes de paiement
settings-add-payment = Ajouter un mode de paiement
settings-billing-history = Historique de facturation

# -----------------------------------------------------------------------------
# OAuth Account Linking (Settings)
# -----------------------------------------------------------------------------
oauth-connected-accounts = Comptes connectés
oauth-connect = Se connecter
oauth-unlink = Dissocier
oauth-not-connected = Non connecté
oauth-linked = Lié
oauth-no-accounts = Aucun compte associé pour l'instant.
oauth-loading = Chargement des comptes liés…

## Payment cards (Stripe SetupIntent)
cards-title = Paiement et cartes
cards-saved = Cartes enregistrées
cards-hint = Les cartes sont stockées en toute sécurité par notre fournisseur de paiement. Les numéros de carte n'atteignent jamais nos serveurs.
cards-add = Ajouter une carte
cards-add-first = Ajoutez votre première carte
cards-none = Aucune carte enregistrée pour l'instant
cards-empty-hint = Ajoutez une carte pour activer la facturation automatique et des paiements plus rapides. Vous serez redirigé vers notre fournisseur de paiement sécurisé pour saisir les détails de votre carte.
cards-default = Par défaut
cards-set-default = Définir par défaut
cards-default-btn = Carte par défaut
cards-remove = Supprimer
cards-remove-confirm = Supprimer cette carte ?
cards-expires = Expire
cards-load-error = Impossible de charger les cartes enregistrées.
cards-add-error = Impossible d'ajouter une carte
cards-default-error = Impossible de mettre à jour la valeur par défaut
cards-remove-error = Impossible de retirer la carte
cards-default-updated = Carte par défaut mise à jour
cards-removed = Carte supprimée
