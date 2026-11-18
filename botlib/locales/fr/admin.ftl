# =============================================================================
# General Bots - Admin Translations (English)
# =============================================================================
# Administrative interface translations for the GB Admin Panel
# =============================================================================

# -----------------------------------------------------------------------------
# Admin Navigation & Dashboard
# -----------------------------------------------------------------------------
admin-title = Administration
admin-dashboard = Tableau de bord d'administration
admin-overview = Aperçu
admin-welcome = Bienvenue dans le panneau d'administration

admin-nav-dashboard = Tableau de bord
admin-nav-users = Utilisateurs
admin-nav-bots = Bots
admin-nav-tenants = Locataires
admin-nav-settings = Paramètres
admin-nav-logs = Journaux
admin-nav-analytics = Analyse
admin-nav-security = Sécurité
admin-nav-integrations = Intégrations
admin-nav-billing = Facturation
admin-nav-support = Assistance
admin-nav-groups = Groupes
admin-nav-dns = DNS
admin-nav-system = Système

# -----------------------------------------------------------------------------
# Admin Quick Actions
# -----------------------------------------------------------------------------
admin-quick-actions = Actions rapides
admin-create-user = Créer un utilisateur
admin-create-group = Créer un groupe
admin-register-dns = Enregistrer le DNS
admin-recent-activity = Activité récente
admin-system-health = Santé du système

# -----------------------------------------------------------------------------
# User Management
# -----------------------------------------------------------------------------
admin-users-title = Gestion des utilisateurs
admin-users-list = Liste des utilisateurs
admin-users-add = Ajouter un utilisateur
admin-users-edit = Modifier l'utilisateur
admin-users-delete = Supprimer un utilisateur
admin-users-search = Rechercher des utilisateurs...
admin-users-filter = Filtrer les utilisateurs
admin-users-export = Exporter les utilisateurs
admin-users-import = Importer des utilisateurs
admin-users-total = Nombre total d'utilisateurs
admin-users-active = Utilisateurs actifs
admin-users-inactive = Utilisateurs inactifs
admin-users-suspended = Utilisateurs suspendus
admin-users-pending = En attente de vérification
admin-users-last-login = Dernière connexion
admin-users-created = Créé
admin-users-role = Rôle
admin-users-status = Statut
admin-users-actions = Actions
admin-users-no-users = Aucun utilisateur trouvé
admin-users-confirm-delete = Êtes-vous sûr de vouloir supprimer cet utilisateur ?
admin-users-deleted = Utilisateur supprimé avec succès
admin-users-saved = Utilisateur enregistré avec succès
admin-users-invite = Inviter un utilisateur
admin-users-invite-sent = Invitation envoyée avec succès
admin-users-bulk-actions = Actions groupées
admin-users-select-all = Sélectionner tout
admin-users-deselect-all = Désélectionner tout

# User Details
admin-user-details = Détails de l'utilisateur
admin-user-profile = Profil
admin-user-email = Courriel
admin-user-name = Nom
admin-user-phone = Téléphone
admin-user-avatar = avatar
admin-user-timezone = Fuseau horaire
admin-user-language = Langue
admin-user-role-admin = Administrateur
admin-user-role-manager = Gestionnaire
admin-user-role-user = Utilisateur
admin-user-role-viewer = Visionneuse
admin-user-status-active = Actif
admin-user-status-inactive = Inactif
admin-user-status-suspended = Suspendu
admin-user-status-pending = En attente
admin-user-permissions = Autorisations
admin-user-activity = Journal d'activité
admin-user-sessions = Séances actives
admin-user-terminate-session = Terminer la session
admin-user-terminate-all = Terminer toutes les sessions
admin-user-reset-password = Réinitialiser le mot de passe
admin-user-force-logout = Forcer la déconnexion
admin-user-enable-2fa = Activer 2FA
admin-user-disable-2fa = Désactiver 2FA

# -----------------------------------------------------------------------------
# Group Management
# -----------------------------------------------------------------------------
admin-groups-title = Gestion de groupe
admin-groups-subtitle = Gérer les groupes, les membres et les autorisations
admin-groups-list = Liste des groupes
admin-groups-add = Ajouter un groupe
admin-groups-create = Créer un groupe
admin-groups-edit = Modifier le groupe
admin-groups-delete = Supprimer le groupe
admin-groups-search = Rechercher des groupes...
admin-groups-filter = Filtrer les groupes
admin-groups-total = Total des groupes
admin-groups-active = Groupes actifs
admin-groups-no-groups = Aucun groupe trouvé
admin-groups-confirm-delete = Êtes-vous sûr de vouloir supprimer ce groupe ?
admin-groups-deleted = Groupe supprimé avec succès
admin-groups-saved = Groupe enregistré avec succès
admin-groups-created = Groupe créé avec succès
admin-groups-loading = Chargement des groupes...

# Group Details
admin-group-details = Détails du groupe
admin-group-name = Nom du groupe
admin-group-description = Descriptif
admin-group-visibility = Visibilité
admin-group-visibility-public = Publique
admin-group-visibility-private = Privé
admin-group-visibility-hidden = Caché
admin-group-join-policy = Politique d'adhésion
admin-group-join-invite = Sur invitation uniquement
admin-group-join-request = Demande d'adhésion
admin-group-join-open = Ouvert
admin-group-members = Membres
admin-group-member-count = { $compte ->
    [one] { $count } member
   *[other] { $count } members
}
admin-group-add-member = Ajouter un membre
admin-group-remove-member = Supprimer un membre
admin-group-permissions = Autorisations
admin-group-settings = Paramètres
admin-group-analytics = Analyse
admin-group-overview = Aperçu

# Group View Modes
admin-groups-view-grid = Vue Grille
admin-groups-view-list = Vue en liste
admin-groups-all-visibility = Toute visibilité

# -----------------------------------------------------------------------------
# DNS Management
# -----------------------------------------------------------------------------
admin-dns-title = Gestion DNS
admin-dns-subtitle = Enregistrez et gérez les noms d'hôte DNS pour vos robots
admin-dns-register = Enregistrer le nom d'hôte
admin-dns-registered = Noms d'hôtes enregistrés
admin-dns-search = Rechercher des noms d'hôtes...
admin-dns-refresh = Actualiser
admin-dns-loading = Chargement des enregistrements DNS...
admin-dns-no-records = Aucun enregistrement DNS trouvé
admin-dns-confirm-delete = Êtes-vous sûr de vouloir supprimer ce nom d'hôte ?
admin-dns-deleted = Nom d'hôte supprimé avec succès
admin-dns-saved = Enregistrement DNS enregistré avec succès
admin-dns-created = Nom d'hôte enregistré avec succès

# DNS Form Fields
admin-dns-hostname = Nom d'hôte
admin-dns-hostname-placeholder = monbot.exemple.com
admin-dns-hostname-help = Entrez le nom de domaine complet que vous souhaitez enregistrer
admin-dns-record-type = Type d'enregistrement
admin-dns-record-type-a = Un (IPv4)
admin-dns-record-type-aaaa = AAAA (IPv6)
admin-dns-record-type-cname = CNAME
admin-dns-ttl = TTL (secondes)
admin-dns-ttl-5min = 5 minutes (300)
admin-dns-ttl-1hour = 1 heure (3600)
admin-dns-ttl-1day = 1 jour (86400)
admin-dns-target = Cible/Adresse IP
admin-dns-target-placeholder-ipv4 = 192.168.1.1
admin-dns-target-placeholder-ipv6 = 2001:db8::1
admin-dns-target-placeholder-cname = cible.exemple.com
admin-dns-target-help-a = Saisissez l'adresse IPv4 vers laquelle pointer
admin-dns-target-help-aaaa = Saisissez l'adresse IPv6 vers laquelle pointer
admin-dns-target-help-cname = Entrez le nom de domaine cible
admin-dns-auto-ssl = Provisionner automatiquement le certificat SSL

# DNS Table Headers
admin-dns-col-hostname = Nom d'hôte
admin-dns-col-type = Tapez
admin-dns-col-target = Cible
admin-dns-col-ttl = Durée de vie
admin-dns-col-ssl = SSL
admin-dns-col-status = Statut
admin-dns-col-actions = Actions

# DNS Status
admin-dns-status-active = Actif
admin-dns-status-pending = En attente
admin-dns-status-error = Erreur
admin-dns-ssl-enabled = SSL activé
admin-dns-ssl-disabled = Pas de SSL
admin-dns-ssl-pending = SSL en attente

# DNS Info Cards
admin-dns-help-title = Aide à la configuration DNS
admin-dns-help-a-record = Un record
admin-dns-help-a-record-desc = Mappe un nom de domaine à une adresse IPv4. Utilisez-le pour pointer votre nom d'hôte directement vers une adresse IP de serveur.
admin-dns-help-aaaa-record = Enregistrement AAAA
admin-dns-help-aaaa-record-desc = Mappe un nom de domaine à une adresse IPv6. Similaire à l'enregistrement A mais pour la connectivité IPv6.
admin-dns-help-cname-record = Enregistrement CNAME
admin-dns-help-cname-record-desc = Crée un alias d'un domaine à un autre. Utile pour pointer des sous-domaines vers votre domaine principal.
admin-dns-help-ssl = SSL/TLS
admin-dns-help-ssl-desc = Provisionne automatiquement les certificats Let's Encrypt pour des connexions HTTPS sécurisées.

# DNS Edit/Remove Modals
admin-dns-edit-title = Modifier l'enregistrement DNS
admin-dns-remove-title = Supprimer le nom d'hôte
admin-dns-remove-warning = Cela supprimera l'enregistrement DNS et tous les certificats SSL associés. Le nom d'hôte ne sera plus résolu.

# -----------------------------------------------------------------------------
# Bot Management
# -----------------------------------------------------------------------------
admin-bots-title = Gestion des robots
admin-bots-list = Liste des robots
admin-bots-add = Ajouter un robot
admin-bots-edit = Modifier le robot
admin-bots-delete = Supprimer le robot
admin-bots-search = Rechercher des robots...
admin-bots-filter = Filtrer les robots
admin-bots-total = Nombre total de robots
admin-bots-active = Bots actifs
admin-bots-inactive = Bots inactifs
admin-bots-draft = Projet de robots
admin-bots-published = Bots publiés
admin-bots-no-bots = Aucun robot trouvé
admin-bots-confirm-delete = Etes-vous sûr de vouloir supprimer ce bot ?
admin-bots-deleted = Bot supprimé avec succès
admin-bots-saved = Bot enregistré avec succès
admin-bots-duplicate = Bot en double
admin-bots-export = Exporter le robot
admin-bots-import = Importer un robot
admin-bots-publish = Publier
admin-bots-unpublish = Annuler la publication
admin-bots-test = Testez le robot
admin-bots-logs = Journaux des robots
admin-bots-analytics = Analyse des robots
admin-bots-conversations = Conversations
admin-bots-templates = Modèles
admin-bots-dialogs = Boîtes de dialogue
admin-bots-knowledge-base = Base de connaissances

# Bot Details
admin-bot-details = Détails du robot
admin-bot-name = Nom du robot
admin-bot-description = Descriptif
admin-bot-avatar = Avatar de robot
admin-bot-language = Langue
admin-bot-timezone = Fuseau horaire
admin-bot-greeting = Message de bienvenue
admin-bot-fallback = Message de secours
admin-bot-channels = Canaux
admin-bot-channel-web = Discussion en ligne
admin-bot-channel-whatsapp = WhatsApp
admin-bot-channel-telegram = Télégramme
admin-bot-channel-slack = Mou
admin-bot-channel-teams = Équipes Microsoft
admin-bot-channel-email = Courriel
admin-bot-model = Modèle d'IA
admin-bot-temperature = Température
admin-bot-max-tokens = Nombre maximum de jetons
admin-bot-system-prompt = Invite système

# -----------------------------------------------------------------------------
# Tenant Management
# -----------------------------------------------------------------------------
admin-tenants-title = Gestion des locataires
admin-tenants-list = Liste des locataires
admin-tenants-add = Ajouter un locataire
admin-tenants-edit = Modifier le locataire
admin-tenants-delete = Supprimer le locataire
admin-tenants-search = Rechercher des locataires...
admin-tenants-total = Total des locataires
admin-tenants-active = Locataires actifs
admin-tenants-suspended = Locataires suspendus
admin-tenants-trial = Locataires d'essai
admin-tenants-no-tenants = Aucun locataire trouvé
admin-tenants-confirm-delete = Êtes-vous sûr de vouloir supprimer ce locataire ?
admin-tenants-deleted = Locataire supprimé avec succès
admin-tenants-saved = Locataire enregistré avec succès

# Tenant Details
admin-tenant-details = Détails du locataire
admin-tenant-name = Nom du locataire
admin-tenant-domain = Domaine
admin-tenant-plan = Planifier
admin-tenant-plan-free = Gratuit
admin-tenant-plan-starter = Démarreur
admin-tenant-plan-professional = Professionnel
admin-tenant-plan-enterprise = Entreprise
admin-tenant-users = Utilisateurs
admin-tenant-bots = Bots
admin-tenant-storage = Stockage utilisé
admin-tenant-api-calls = Appels API
admin-tenant-limits = Limites d'utilisation
admin-tenant-billing = Informations de facturation

# -----------------------------------------------------------------------------
# System Settings
# -----------------------------------------------------------------------------
admin-settings-title = Paramètres système
admin-settings-general = Paramètres généraux
admin-settings-security = Paramètres de sécurité
admin-settings-email = Paramètres de messagerie
admin-settings-storage = Paramètres de stockage
admin-settings-integrations = Intégrations
admin-settings-api = Paramètres de l'API
admin-settings-appearance = Apparence
admin-settings-localization = Localisation
admin-settings-notifications = Notifications
admin-settings-backup = Sauvegarde et restauration
admin-settings-maintenance = Mode d'entretien
admin-settings-saved = Paramètres enregistrés avec succès
admin-settings-reset = Réinitialiser aux valeurs par défaut
admin-settings-confirm-reset = Êtes-vous sûr de vouloir réinitialiser tous les paramètres par défaut ?

# General Settings
admin-settings-site-name = Nom du site
admin-settings-site-url = URL du site
admin-settings-admin-email = E-mail de l'administrateur
admin-settings-support-email = E-mail d'assistance
admin-settings-default-language = Langue par défaut
admin-settings-default-timezone = Fuseau horaire par défaut
admin-settings-date-format = Format des dates
admin-settings-time-format = Format de l'heure
admin-settings-currency = Devise

# Email Settings
admin-settings-smtp-host = Hôte SMTP
admin-settings-smtp-port = Port SMTP
admin-settings-smtp-user = Nom d'utilisateur SMTP
admin-settings-smtp-password = Mot de passe SMTP
admin-settings-smtp-encryption = Cryptage
admin-settings-smtp-from-name = Du nom
admin-settings-smtp-from-email = Depuis un e-mail
admin-settings-smtp-test = Envoyer un e-mail de test
admin-settings-smtp-test-success = E-mail de test envoyé avec succès
admin-settings-smtp-test-failed = Échec de l'envoi de l'e-mail de test

# Storage Settings
admin-settings-storage-provider = Fournisseur de stockage
admin-settings-storage-local = Stockage local
admin-settings-storage-s3 = Amazone S3
admin-settings-storage-minio = MinIO
admin-settings-storage-gcs = Stockage Google Cloud
admin-settings-storage-azure = Stockage Blob Azure
admin-settings-storage-bucket = Nom du compartiment
admin-settings-storage-region = Région
admin-settings-storage-access-key = Clé d'accès
admin-settings-storage-secret-key = Clé secrète
admin-settings-storage-endpoint = URL du point de terminaison

# -----------------------------------------------------------------------------
# System Logs
# -----------------------------------------------------------------------------
admin-logs-title = Journaux système
admin-logs-search = Rechercher des journaux...
admin-logs-filter-level = Filtrer par niveau
admin-logs-filter-source = Filtrer par source
admin-logs-filter-date = Filtrer par date
admin-logs-level-all = Tous les niveaux
admin-logs-level-debug = Débogage
admin-logs-level-info = Informations
admin-logs-level-warning = Avertissement
admin-logs-level-error = Erreur
admin-logs-level-critical = Critique
admin-logs-export = Exporter les journaux
admin-logs-clear = Effacer les journaux
admin-logs-confirm-clear = Êtes-vous sûr de vouloir effacer tous les journaux ?
admin-logs-cleared = Journaux effacés avec succès
admin-logs-no-logs = Aucun journal trouvé
admin-logs-refresh = Actualiser
admin-logs-auto-refresh = Actualisation automatique
admin-logs-timestamp = Horodatage
admin-logs-level = Niveau
admin-logs-source = Source
admin-logs-message = Message
admin-logs-details = Détails

# -----------------------------------------------------------------------------
# Analytics
# -----------------------------------------------------------------------------
admin-analytics-title = Analyse
admin-analytics-overview = Aperçu
admin-analytics-users = Analyse des utilisateurs
admin-analytics-bots = Analyse des robots
admin-analytics-conversations = Analyse des conversations
admin-analytics-performance = Performances
admin-analytics-period = Période
admin-analytics-period-today = Aujourd'hui
admin-analytics-period-week = Cette semaine
admin-analytics-period-month = Ce mois-ci
admin-analytics-period-quarter = Ce trimestre
admin-analytics-period-year = Cette année
admin-analytics-period-custom = Gamme personnalisée
admin-analytics-export = Exporter le rapport
admin-analytics-total-users = Nombre total d'utilisateurs
admin-analytics-new-users = Nouveaux utilisateurs
admin-analytics-active-users = Utilisateurs actifs
admin-analytics-total-bots = Nombre total de robots
admin-analytics-active-bots = Bots actifs
admin-analytics-total-conversations = Conversations totales
admin-analytics-avg-response-time = Temps de réponse moyen
admin-analytics-satisfaction-rate = Taux de satisfaction
admin-analytics-resolution-rate = Taux de résolution

# -----------------------------------------------------------------------------
# Security
# -----------------------------------------------------------------------------
admin-security-title = Sécurité
admin-security-overview = Présentation de la sécurité
admin-security-audit-log = Journal d'audit
admin-security-login-attempts = Tentatives de connexion
admin-security-blocked-ips = IP bloquées
admin-security-api-keys = Clés API
admin-security-webhooks = Webhooks
admin-security-cors = Paramètres CORS
admin-security-rate-limiting = Limitation du débit
admin-security-encryption = Cryptage
admin-security-2fa = Authentification à deux facteurs
admin-security-sso = Authentification unique
admin-security-password-policy = Politique de mot de passe

# API Keys
admin-api-keys-title = Clés API
admin-api-keys-add = Créer une clé API
admin-api-keys-name = Nom de la clé
admin-api-keys-key = Clé API
admin-api-keys-secret = Clé secrète
admin-api-keys-created = Créé
admin-api-keys-last-used = Dernière utilisation
admin-api-keys-expires = Expire
admin-api-keys-never = Jamais
admin-api-keys-revoke = Révoquer
admin-api-keys-confirm-revoke = Êtes-vous sûr de vouloir révoquer cette clé API ?
admin-api-keys-revoked = Clé API révoquée avec succès
admin-api-keys-created-success = Clé API créée avec succès
admin-api-keys-copy = Copier dans le Presse-papiers
admin-api-keys-copied = Copié!
admin-api-keys-warning = Assurez-vous de copier votre clé API maintenant. Vous ne pourrez plus le revoir !

# -----------------------------------------------------------------------------
# Billing
# -----------------------------------------------------------------------------
admin-billing-title = Facturation
admin-billing-overview = Aperçu de la facturation
admin-billing-current-plan = Forfait actuel
admin-billing-usage = Utilisation
admin-billing-invoices = Factures
admin-billing-payment-methods = Méthodes de paiement
admin-billing-upgrade = Plan de mise à niveau
admin-billing-downgrade = Plan de rétrogradation
admin-billing-cancel = Annuler l'abonnement
admin-billing-invoice-date = Date de facture
admin-billing-invoice-amount = Montant
admin-billing-invoice-status = Statut
admin-billing-invoice-paid = Payé
admin-billing-invoice-pending = En attente
admin-billing-invoice-overdue = En retard
admin-billing-invoice-download = Télécharger la facture

# -----------------------------------------------------------------------------
# Backup & Restore
# -----------------------------------------------------------------------------
admin-backup-title = Sauvegarde et restauration
admin-backup-create = Créer une sauvegarde
admin-backup-restore = Restaurer la sauvegarde
admin-backup-schedule = Planifier des sauvegardes
admin-backup-list = Historique de sauvegarde
admin-backup-name = Nom de la sauvegarde
admin-backup-size = Taille
admin-backup-created = Créé
admin-backup-download = Télécharger
admin-backup-delete = Supprimer
admin-backup-confirm-restore = Êtes-vous sûr de vouloir restaurer cette sauvegarde ? Cela écrasera les données actuelles.
admin-backup-confirm-delete = Êtes-vous sûr de vouloir supprimer cette sauvegarde ?
admin-backup-in-progress = Sauvegarde en cours...
admin-backup-completed = Sauvegarde terminée avec succès
admin-backup-failed = La sauvegarde a échoué
admin-backup-restore-in-progress = Restauration en cours...
admin-backup-restore-completed = Restauration terminée avec succès
admin-backup-restore-failed = La restauration a échoué

# -----------------------------------------------------------------------------
# Maintenance Mode
# -----------------------------------------------------------------------------
admin-maintenance-title = Mode d'entretien
admin-maintenance-enable = Activer le mode maintenance
admin-maintenance-disable = Désactiver le mode maintenance
admin-maintenance-status = Statut actuel
admin-maintenance-active = Le mode maintenance est actif
admin-maintenance-inactive = Le mode maintenance est inactif
admin-maintenance-message = Message d'entretien
admin-maintenance-default-message = Nous effectuons actuellement une maintenance programmée. Veuillez revenir bientôt.
admin-maintenance-allowed-ips = Adresses IP autorisées
admin-maintenance-confirm-enable = Êtes-vous sûr de vouloir activer le mode maintenance ? Les utilisateurs ne pourront pas accéder au système.

# -----------------------------------------------------------------------------
# Common Admin UI Elements
# -----------------------------------------------------------------------------
admin-required = Obligatoire
admin-optional = Facultatif
admin-loading = Chargement...
admin-saving = Sauvegarde...
admin-deleting = Suppression...
admin-confirm = Confirmer
admin-cancel = Annuler
admin-save = Enregistrer
admin-create = Créer
admin-update = Mise à jour
admin-delete = Supprimer
admin-edit = Modifier
admin-view = Voir
admin-close = Fermer
admin-back = Retour
admin-next = Suivant
admin-previous = Précédent
admin-refresh = Actualiser
admin-export = Exporter
admin-import = Importer
admin-search = Rechercher
admin-filter = Filtrer
admin-clear = Effacer
admin-select = Sélectionnez
admin-select-all = Sélectionner tout
admin-deselect-all = Désélectionner tout
admin-actions = Actions
admin-more-actions = Plus de mesures
admin-no-data = Aucune donnée disponible
admin-error = Une erreur s'est produite
admin-success = Succès
admin-warning = Avertissement
admin-info = Informations

# Table Pagination
admin-showing = Affichage de { $from } à { $to } sur { $total } résultats
admin-page = Page { $current } de { $total }
admin-items-per-page = Articles par page
admin-go-to-page = Aller à la page

# Bulk Actions
admin-bulk-delete = Supprimer la sélection
admin-bulk-export = Exporter la sélection
admin-bulk-activate = Activer la sélection
admin-bulk-deactivate = Désactiver la sélection
admin-selected-count = { $compte ->
    [one] { $count } item selected
   *[other] { $count } items selected
}

admin-audit-subtitle = Journal d'activité du système
admin-users-subtitle = Personnes ayant accès à cet espace de travail
admin-bots-subtitle = Gérer les bots assistants
admin-users = Utilisateurs
admin-groups = Groupes
admin-bots = Bots
admin-dns = DNS
admin-audit = Journal d'audit
nav-about = À propos
nav-organizations = Organisations


# ── #2307 Users & Groups dialogs / status regions ──────────────────
admin-add-user = Ajouter un utilisateur
admin-add-group = Ajouter un groupe
admin-search-users = Rechercher des utilisateurs...
admin-search-groups = Rechercher des groupes...
admin-confirm-title = Confirmer l'action
admin-confirm-accept = Confirmer
admin-col-user = Utilisateur
admin-col-email = E-mail
admin-col-organization = Organisation
admin-col-roles = Rôles
admin-col-status = Statut
admin-col-actions = Actions
admin-status-active = Actif
admin-status-inactive = Inactif
admin-action-edit = Modifier
admin-action-delete = Supprimer
admin-view-members = Voir les membres
admin-members-suffix = membres
admin-creating = Création…
admin-user-created = Utilisateur créé et ajouté à l'organisation
admin-group-created = Groupe créé avec succès
admin-user-deleted = Utilisateur supprimé
admin-group-deleted = Groupe supprimé
admin-member-removed = Membre supprimé
admin-confirm-delete-user = Supprimer cet utilisateur ?
admin-confirm-delete-group = Supprimer ce groupe ?
admin-confirm-remove-member = Supprimer le membre ?
admin-failed-load-users = Impossible de charger les utilisateurs.
admin-failed-load-groups = Impossible de charger les groupes.
admin-failed-create-user = Impossible de créer l'utilisateur.
admin-failed-create-group = Impossible de créer le groupe.
admin-no-users = Aucun utilisateur trouvé
admin-no-groups = Aucun groupe trouvé

# semantic i18n key (#2542)
admin-org-switch-title = Changer d'organisation

# semantic i18n key (#2542)
admin-org-create-new-title = Créer une nouvelle organisation

# semantic i18n key (#2542)
admin-org-name-label = Nom de l'organisation

# semantic i18n key (#2542)
admin-org-url-slug-label = Slug d'URL

# semantic i18n key (#2542)
admin-org-slug-preview = organization-slug

# semantic i18n key (#2542)
admin-org-plan-label = Planifier

# semantic i18n key (#2542)
admin-org-plan-free = Gratuit

# semantic i18n key (#2542)
admin-org-plan-starter = Démarreur

# semantic i18n key (#2542)
admin-org-create-btn = Créer l'organisation

# semantic i18n key (#2542)
admin-filter-all-roles = Tous les rôles

# semantic i18n key (#2542)
admin-user-role-moderator = Modérateur

# semantic i18n key (#2542)
admin-filter-all-status = Tous les statuts

# semantic i18n key (#2542)
admin-users-last-active = Dernière activité

# semantic i18n key (#2542)
admin-users-loading = Chargement des utilisateurs...

# semantic i18n key (#2542)
admin-users-create-new = Créer un nouvel utilisateur

# semantic i18n key (#2542)
admin-users-send-welcome-email = Envoyer un e-mail de bienvenue

# semantic i18n key (#2542)
admin-users-require-password-change = Exiger un changement de mot de passe lors de la première connexion

# semantic i18n key (#2542)
admin-users-delete-warning = Cette action est irréversible. Toutes les données de l'utilisateur seront définitivement supprimées.

# semantic i18n key (#2542)
accounts-subtitle = Gérez vos intégrations de canaux de réseaux sociaux et de messagerie

# semantic i18n key (#2542)
accounts-channel-facebook = Facebook

# semantic i18n key (#2542)
accounts-channel-twitter-x = Twitter / X

# semantic i18n key (#2542)
accounts-channel-bluesky = Bluesky

# semantic i18n key (#2542)
accounts-channel-threads = Threads

# semantic i18n key (#2542)
accounts-channel-tiktok = TikTok

# semantic i18n key (#2542)
accounts-channel-youtube = YouTube

# semantic i18n key (#2542)
accounts-channel-pinterest = Pinterest

# semantic i18n key (#2542)
accounts-channel-reddit = Reddit

# semantic i18n key (#2542)
accounts-channel-snapchat = Snapchat

# semantic i18n key (#2542)
accounts-messaging = Messagerie

# semantic i18n key (#2542)
accounts-channel-whatsapp-business = WhatsApp Business

# semantic i18n key (#2542)
accounts-channel-twilio-sms = Twilio SMS

# semantic i18n key (#2542)
accounts-channel-wechat = WeChat

# semantic i18n key (#2542)
accounts-channel-gmail-google-workspace = Gmail / Google Workspace

# semantic i18n key (#2542)
accounts-channel-outlook-office-365 = Outlook / Office 365

# semantic i18n key (#2542)
accounts-channel-custom-smtp = SMTP personnalisé

# semantic i18n key (#2542)
accounts-redirecting-authorization = Redirection vers l'autorisation...

# semantic i18n key (#2542)
accounts-configure-smtp-server = Configurer le serveur SMTP

# semantic i18n key (#2542)
accounts-save-configuration = Enregistrer la configuration

# semantic i18n key (#2542)
accounts-account-details = Détails du compte

# semantic i18n key (#2542)
accounts-connection-status = État de la connexion

# semantic i18n key (#2542)
accounts-sample-username = @utilisateur

# semantic i18n key (#2542)
accounts-sample-connected-since = 15 janvier 2025

# semantic i18n key (#2542)
accounts-permissions = Autorisations

# semantic i18n key (#2542)
admin-groups-hidden = Masqué

# semantic i18n key (#2542)
admin-groups-detail-title = Détails du groupe

# semantic i18n key (#2542)
admin-groups-tab-overview = Aperçu

# semantic i18n key (#2542)
admin-groups-tab-members = Membres

# semantic i18n key (#2542)
admin-groups-tab-permissions = Autorisations

# semantic i18n key (#2542)
admin-groups-tab-kbs = Bases de connaissances

# semantic i18n key (#2542)
admin-groups-create-title = Créer un nouveau groupe

# semantic i18n key (#2542)
admin-groups-visibility = Visibilité

# semantic i18n key (#2542)
admin-groups-join-policy = Politique d'adhésion

# semantic i18n key (#2542)
admin-groups-invite-only = Sur invitation uniquement

# semantic i18n key (#2542)
admin-groups-request-join = Demander à rejoindre

# semantic i18n key (#2542)
admin-groups-allow-member-invite = Autoriser les membres à inviter d'autres personnes

# semantic i18n key (#2542)
admin-groups-enable-chat = Activer le chat de groupe

# semantic i18n key (#2542)
admin-groups-add-members = Ajouter des membres

# semantic i18n key (#2542)
admin-groups-search-users = Rechercher des utilisateurs

# semantic i18n key (#2542)
admin-groups-selected-users = Utilisateurs sélectionnés

# semantic i18n key (#2542)
admin-groups-no-users-selected = Aucun utilisateur sélectionné

# semantic i18n key (#2542)
admin-groups-role-member = Membre

# semantic i18n key (#2542)
admin-groups-role-moderator = Modérateur

# semantic i18n key (#2542)
admin-groups-send-invites = Envoyer des invitations

# semantic i18n key (#2542)
admin-groups-email-addresses = Adresses e-mail

# semantic i18n key (#2542)
admin-groups-emails-help = Saisissez une adresse e-mail par ligne

# semantic i18n key (#2542)
admin-groups-personal-message-optional = Message personnel (facultatif)

# semantic i18n key (#2542)
admin-groups-delete-warning = Cela supprimera tous les membres et toutes les données du groupe. Cette action est irréversible.

# semantic i18n key (#2542)
admin-billing-dashboard-title = Tableau de bord de facturation

# semantic i18n key (#2542)
admin-billing-dashboard-subtitle = Analyses et informations complètes sur la facturation et l'utilisation de votre organisation

# semantic i18n key (#2542)
admin-billing-dashboard-current-period = Période actuelle

# semantic i18n key (#2542)
admin-billing-dashboard-year-to-date = Depuis le début de l'année

# semantic i18n key (#2542)
admin-billing-dashboard-current-spending = Dépenses actuelles

# semantic i18n key (#2542)
admin-billing-dashboard-quota-utilization = Utilisation du quota

# semantic i18n key (#2542)
admin-billing-dashboard-average-quotas = Moyenne sur tous les quotas

# semantic i18n key (#2542)
admin-billing-dashboard-api-calls = Appels API

# semantic i18n key (#2542)
admin-billing-dashboard-storage-used = Stockage utilisé

# semantic i18n key (#2542)
admin-billing-dashboard-spending-trends = Tendances des dépenses

# semantic i18n key (#2542)
admin-billing-dashboard-daily = Quotidien

# semantic i18n key (#2542)
admin-billing-dashboard-weekly = Hebdomadaire

# semantic i18n key (#2542)
admin-billing-dashboard-cost-breakdown = Répartition des coûts par service

# semantic i18n key (#2542)
admin-billing-dashboard-quota-management = Gestion des quotas

# semantic i18n key (#2542)
admin-billing-dashboard-team-members = Membres de l'équipe

# semantic i18n key (#2542)
admin-billing-dashboard-no-alerts = Aucune alerte configurée

# semantic i18n key (#2542)
admin-billing-dashboard-usage-by-bot = Utilisation par bot

# semantic i18n key (#2542)
admin-billing-dashboard-bot-customer-support = Bot de support client

# semantic i18n key (#2542)
admin-billing-dashboard-bot-cs-01 = bot_cs_01

# semantic i18n key (#2542)
admin-billing-dashboard-bot-sales-assistant = Assistant commercial

# semantic i18n key (#2542)
admin-billing-dashboard-bot-sa-01 = bot_sa_01

# semantic i18n key (#2542)
admin-billing-dashboard-bot-hr-assistant = Assistant RH

# semantic i18n key (#2542)
admin-billing-dashboard-bot-hr-01 = bot_hr_01

# semantic i18n key (#2542)
admin-billing-dashboard-bot-it-helpdesk = Service d'assistance informatique

# semantic i18n key (#2542)
admin-billing-dashboard-bot-it-01 = bot_it_01

# semantic i18n key (#2542)
admin-billing-dashboard-recent-invoices = Factures récentes

# semantic i18n key (#2542)
admin-billing-dashboard-alerts = Alertes de facturation

# semantic i18n key (#2542)
admin-billing-dashboard-invoice-generated-january = Facture générée pour janvier

# semantic i18n key (#2542)
admin-billing-dashboard-quota-settings = Paramètres de quota

# semantic i18n key (#2542)
admin-billing-dashboard-invoices-limit = Limite de factures

# semantic i18n key (#2542)
admin-billing-dashboard-team-members-limit = Limite de membres de l'équipe

# semantic i18n key (#2542)
admin-billing-dashboard-bots-limit = Limite de bots

# semantic i18n key (#2542)
admin-billing-dashboard-alert-thresholds = Seuils d'alerte de facturation

# semantic i18n key (#2542)
admin-billing-subscription-title = Facturation et abonnement

# semantic i18n key (#2542)
admin-billing-manage-subscription-desc = Gérez l'abonnement, les moyens de paiement et l'historique de facturation de votre organisation

# semantic i18n key (#2542)
admin-billing-per-month = /mois

# semantic i18n key (#2542)
admin-billing-next-billing-date = Prochaine date de facturation :

# semantic i18n key (#2542)
admin-billing-next-billing-date-value = 21 février 2025

# semantic i18n key (#2542)
admin-billing-usage-this-period = Utilisation sur cette période

# semantic i18n key (#2542)
admin-billing-usage-period-value = 21 janv. - 21 févr. 2025

# semantic i18n key (#2542)
admin-billing-payment-method = Moyen de paiement

# semantic i18n key (#2542)
admin-billing-card-expiry-value = Expire 12/2026

# semantic i18n key (#2542)
admin-billing-billing-address = Adresse de facturation

# semantic i18n key (#2542)
admin-billing-address-suite = Suite 456

# semantic i18n key (#2542)
admin-billing-address-city = San Francisco, CA 94102

# semantic i18n key (#2542)
admin-billing-address-country = États-Unis

# semantic i18n key (#2542)
admin-billing-upgrade-your-plan = Améliorez votre forfait

# semantic i18n key (#2542)
admin-billing-recommended = Recommandé

# semantic i18n key (#2542)
admin-billing-priority-support = Support prioritaire

# semantic i18n key (#2542)
admin-billing-upgrade-to-business = Passer à Business

# semantic i18n key (#2542)
admin-billing-enterprise = Entreprise

# semantic i18n key (#2542)
admin-billing-unlimited-everything = Tout illimité

# semantic i18n key (#2542)
admin-billing-dedicated-support = Support dédié

# semantic i18n key (#2542)
admin-billing-custom-integrations = Intégrations personnalisées

# semantic i18n key (#2542)
admin-billing-sla-guarantee = Garantie SLA

# semantic i18n key (#2542)
admin-billing-contact-sales = Contacter les ventes

# semantic i18n key (#2542)
admin-billing-prorated-notice = Votre prochaine facture sera calculée au prorata de la période de facturation restante.

# semantic i18n key (#2542)
admin-billing-sorry-to-see-you-go = Nous sommes désolés de vous voir partir !

# semantic i18n key (#2542)
admin-billing-cancel-consider-options = Avant d'annuler, souhaitez-vous envisager l'une de ces options ?

# semantic i18n key (#2542)
admin-billing-stay-reduced-rate = Restez avec nous à un tarif réduit

# semantic i18n key (#2542)
admin-billing-accept-offer = Accepter l'offre

# semantic i18n key (#2542)
admin-billing-pause-your-subscription = Mettre votre abonnement en pause

# semantic i18n key (#2542)
admin-billing-pause-up-to-3-months = Faites une pause jusqu'à 3 mois

# semantic i18n key (#2542)
admin-billing-pause-instead = Mettre en pause plutôt

# semantic i18n key (#2542)
admin-billing-downgrade-smaller-plan = Passer à un forfait inférieur

# semantic i18n key (#2542)
admin-billing-keep-essential-features = Conservez les fonctionnalités essentielles à moindre coût

# semantic i18n key (#2542)
admin-billing-view-plans = Voir les forfaits

# semantic i18n key (#2542)
admin-billing-why-cancelling = Pourquoi annulez-vous ?

# semantic i18n key (#2542)
admin-billing-select-reason = Sélectionnez une raison...

# semantic i18n key (#2542)
admin-billing-reason-too-expensive = Trop cher

# semantic i18n key (#2542)
admin-billing-reason-missing-features = Fonctionnalités manquantes dont j'ai besoin

# semantic i18n key (#2542)
admin-billing-reason-not-using = Je ne l'utilise pas assez

# semantic i18n key (#2542)
admin-billing-reason-switching = Passage à un autre produit

# semantic i18n key (#2542)
admin-billing-reason-temporary = Temporaire - je reviendrai

# semantic i18n key (#2542)
admin-billing-additional-feedback = Commentaires supplémentaires (facultatif)

# semantic i18n key (#2542)
admin-billing-keep-subscription = Conserver l'abonnement

# semantic i18n key (#2542)
admin-groups-placeholder-name = Équipe d'ingénierie
admin-groups-placeholder-description = Décrivez l’objectif de ce groupe...
admin-groups-placeholder-search-members = Rechercher par nom ou e-mail...
admin-groups-placeholder-invite-emails = Entrez les adresses e-mail, une par ligne...
admin-groups-placeholder-invite-message = Ajoutez un message personnel à l'invitation...

# semantic i18n key (#2542)
admin-org-placeholder-search = Rechercher des organisations...

# semantic i18n key (#2542)
accounts-placeholder-smtp-host = smtp.exemple.com
accounts-placeholder-smtp-username = votre@email.com
accounts-placeholder-smtp-password = ••••••••
accounts-placeholder-smtp-from-address = noreply@exemple.com
accounts-placeholder-smtp-from-name = Votre entreprise

# semantic i18n key (#2542)
admin-placeholder-username = nom d'utilisateur
admin-placeholder-email = utilisateur@exemple.com
admin-placeholder-display-name = Jean Dupont
admin-placeholder-password = ••••••••
admin-placeholder-group-name = Équipe d'ingénierie
admin-placeholder-group-description = Description du groupe...
