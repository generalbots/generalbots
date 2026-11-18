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

# Settings → Model (#2568)
mdl-title = Modèle
mdl-subtitle = Où l'assistant réfléchit. Choisissez un modèle exécuté sur cet ordinateur, l'endpoint de votre fournisseur, ou le modèle hébergé gratuit.
mdl-current = Modèle actuel
mdl-where = Où l'assistant réfléchit
mdl-local = Sur cet ordinateur
mdl-own = Le serveur de votre organisation ou votre clé
mdl-free = Modèle hébergé gratuit
mdl-active = Utilisé
mdl-endpoint = URL de l'endpoint
mdl-model = Modèle
mdl-key = Clé d'API
mdl-key-stored = clé enregistrée
mdl-test = Tester
mdl-save = Enregistrer
mdl-test-ok = Accessible, latence %ms ms
mdl-test-fail = Échec : %e
mdl-testing = Test en cours…
mdl-saving = Enregistrement…
mdl-saved = Enregistré. Prend effet au prochain message.
mdl-swapped = Prêt. Le nouveau modèle est actif.
mdl-restart = Enregistré. Redémarrez l'application pour utiliser le nouveau modèle.
mdl-admin-only = Seul l'administrateur peut modifier le modèle.
mdl-loading = Chargement…
mdl-local-active = Modèle local actif : %m
mdl-local-hint = Choisissez un modèle dans la page On-device AI pour l'utiliser ici.
mdl-local-open = Ouvrir On-device AI
mdl-free-note = Tout ce que vous écrivez et joignez est envoyé à %h.
mdl-inconsistent = La configuration du modèle est incohérente.
mdl-restore-free = Rétablir le modèle gratuit
mdl-confirm-external = Ce que vous écrivez et joignez sera envoyé à %h, hors de cet ordinateur. Continuer ?
mdl-scope-machine = Sur cet ordinateur
mdl-scope-network = Le serveur de votre organisation (%h)
mdl-scope-external = Un hôte externe (%h)
mdl-unavailable = Ce serveur n'expose pas l'API des modèles.

# Settings → Instructions, Shortcuts, Text size (#2568)
instr-title = Instructions personnalisées
instr-subtitle = Décrivez-vous et indiquez comment l'assistant doit répondre.
instr-enabled = Activer les instructions personnalisées
instr-enabled-desc = Désactivé, l'assistant répond sans ces instructions.
instr-about = Que doit savoir l'assistant à votre sujet ?
instr-about-placeholder = Rôle, langue, contexte, préférences…
instr-response = Comment l'assistant doit-il répondre ?
instr-response-placeholder = Ton, niveau de détail, format…
instr-save = Enregistrer
instr-saving = Enregistrement…
instr-saved = Enregistré.
instr-save-failed = Impossible d'enregistrer : %e
instr-loading = Chargement…
instr-load-failed = Impossible de charger : %e
instr-unavailable = Ce serveur n'expose pas les instructions personnalisées.
sc-subtitle = Appuyez sur Shift + ? n'importe où dans la suite pour ouvrir cette liste.
sc-group-general = Général
sc-group-drive = Drive
sc-note = Les raccourcis sont ignorés pendant la saisie dans un champ, pour ne jamais interrompre un message ou un formulaire.
ts-title = Taille du texte
ts-desc = Choisissez la taille du texte dans toute la suite
ts-small = Petit
ts-default = Par défaut
ts-large = Grand
ts-xlarge = Très grand
# Settings → Appearance: chat background pattern
chatbg-title = Fond du chat
chatbg-desc = Choisissez le motif derrière vos conversations
chatbg-none = Aucun
chatbg-image = Papier
chatbg-dots = Points
chatbg-microdots = Micro-points
chatbg-grid = Grille
chatbg-blueprint = Plan
chatbg-weave = Tissage
chatbg-mosaic = Mosaïque

# semantic i18n key (#2542)
settings-network-participation-desc = Contrôlez si cette installation participe au réseau pair à pair

# semantic i18n key (#2542)
settings-p2p-network = Réseau P2P

# semantic i18n key (#2542)
settings-network-mode-label = Mode réseau

# semantic i18n key (#2542)
settings-network-mode-p2p = P2P (recommandé)
settings-network-p2p-description = Le mode hors ligne maintient chaque connexion sur <code>127.0.0.1</code> et les liens partagés ne fonctionnent que sur votre réseau local.<br />Le P2P active de futures fonctionnalités telles que le relais gratuit des liens partagés et la découverte de pairs. Aucune donnée ne quitte cette machine tant que vous ne partagez pas explicitement quelque chose.
settings-network-p2p-restart-hint = Cette préférence est enregistrée localement et prend effet après le redémarrage de l'application. Le point de terminaison du relais est fixe (<code id="network-relay-host">peer.generalbots.org</code>) ; la résolution des pairs et du site d'accueil l'utilise toujours, aucune configuration manuelle n'est donc nécessaire.

# semantic i18n key (#2542)
settings-role-viewer = Lecteur

# semantic i18n key (#2542)
settings-register-dns-hostname = Enregistrer le nom d'hôte DNS

# semantic i18n key (#2542)
settings-dns-aaaa-ipv6 = AAAA (IPv6)

# semantic i18n key (#2542)
settings-dns-cname = CNAME

# semantic i18n key (#2542)
settings-create-token = Créer un jeton

# semantic i18n key (#2542)
settings-system-preview-desc = Mode aperçu et informations de version pour ce déploiement

# semantic i18n key (#2542)
settings-system-preview-surface-desc = Contrôle la quantité d'interface révélée, pas ce qui est installé.

# semantic i18n key (#2542)
settings-system-preview-toggle-desc = Révèle les applications en aperçu et l'arborescence complète des paramètres. Désactivé conserve uniquement les paramètres essentiels de cette édition.

# semantic i18n key (#2542)
settings-component-versions = Versions des composants

# semantic i18n key (#2542)
settings-component-versions-desc = Versions de BotServer, BotUI et des services pour cette installation

# semantic i18n key (#2542)
settings-open-versions = Ouvrir les versions

# semantic i18n key (#2542)
settings-on-device-ai = IA sur l'appareil

# semantic i18n key (#2542)
settings-operating-system = Système d'exploitation

# semantic i18n key (#2542)
settings-ledger = Grand livre

# semantic i18n key (#2542)
settings-updates = Mises à jour

# semantic i18n key (#2542)
settings-this-workspace = Cet espace de travail

# semantic i18n key (#2542)
settings-loading-identity = Chargement de l'identité…

# semantic i18n key (#2542)
settings-plan-placeholder = Forfait : —

# semantic i18n key (#2542)
settings-preview-placeholder = Aperçu : —

# semantic i18n key (#2542)
settings-oda-subtitle = Modèles qui s'exécutent sur cette machine. Téléchargez un modèle adapté au matériel, activez-le et le moteur local l'utilisera à son prochain démarrage. Aucune clé API n'est utilisée ici.

# semantic i18n key (#2542)
settings-oda-detecting-hardware = Détection du matériel…

# semantic i18n key (#2542)
settings-oda-recommended-for-machine = Recommandé pour cette machine

# semantic i18n key (#2542)
settings-oda-activate = Activer

# semantic i18n key (#2542)
settings-oda-models = Modèles

# semantic i18n key (#2542)
settings-oda-model = Modèle

# semantic i18n key (#2542)
settings-oda-params = Paramètres

# semantic i18n key (#2542)
settings-oda-quant = Quant.

# semantic i18n key (#2542)
settings-oda-min-vram = VRAM min.

# semantic i18n key (#2542)
settings-oda-loading-models = Chargement des modèles…

# semantic i18n key (#2542)
settings-date-format-mm-dd-yyyy-us = MM/JJ/AAAA (États-Unis)

# semantic i18n key (#2542)
settings-date-format-dd-mm-yyyy-eu-br = JJ/MM/AAAA (UE/BR)

# semantic i18n key (#2542)
settings-date-format-yyyy-mm-dd-iso = AAAA-MM-JJ (ISO)

# semantic i18n key (#2542)
settings-timezone-america-new-york = New York (GMT-5)

# semantic i18n key (#2542)
settings-timezone-america-los-angeles = Los Angeles (GMT-8)

# semantic i18n key (#2542)
settings-timezone-europe-london = Londres (GMT+0)

# semantic i18n key (#2542)
settings-timezone-europe-paris = Paris (GMT+1)

# semantic i18n key (#2542)
settings-timezone-asia-tokyo = Tokyo (GMT+9)

# semantic i18n key (#2542)
settings-desktop-wallpaper = Fond d'écran du bureau

# semantic i18n key (#2542)
settings-desktop-wallpaper-desc = Choisissez l'arrière-plan affiché sur votre bureau

# semantic i18n key (#2542)
settings-organizations-bots = Organisations et bots

# semantic i18n key (#2542)
settings-organizations-bots-desc = Gérez vos organisations et la hiérarchie des bots

# semantic i18n key (#2542)
settings-current-organization = Organisation actuelle

# semantic i18n key (#2542)
settings-current-organization-desc = Sélectionnez l'organisation que vous souhaitez gérer

# semantic i18n key (#2542)
settings-loading-organizations = Chargement des organisations...

# semantic i18n key (#2542)
settings-bot-tree = Arborescence des bots

# semantic i18n key (#2542)
settings-bot-tree-desc = Bots appartenant à l'organisation sélectionnée

# semantic i18n key (#2542)
settings-loading-bots = Chargement des bots...

# semantic i18n key (#2542)
settings-storage-sync-desc = Gérez vos paramètres de stockage et de synchronisation

# semantic i18n key (#2542)
settings-knowledge = Connaissances

# semantic i18n key (#2542)
settings-drive-files-as-context = Utiliser tous les fichiers Drive comme contexte de connaissances

# semantic i18n key (#2542)
settings-drive-files-as-context-desc = Lorsqu'elle est activée (par défaut), les noms de vos fichiers Drive récents sont envoyés à chaque tour de discussion afin que l'assistant puisse les utiliser comme contexte. Désactivez-la pour exclure Drive du prompt.

# semantic i18n key (#2542)
settings-loading-storage-information = Chargement des informations de stockage...

# semantic i18n key (#2542)
settings-connected-cloud-storage = Stockage cloud connecté

# semantic i18n key (#2542)
settings-loading-storage-connections = Chargement des connexions de stockage...

# semantic i18n key (#2542)
settings-os-subtitle = Paramètres et outils hôte de bas niveau pour cette installation du shell

# semantic i18n key (#2542)
settings-os-host-identity-desc = Identité de l'hôte, noyau et fuseau horaire

# semantic i18n key (#2542)
settings-os-kernel = Noyau

# semantic i18n key (#2542)
settings-os-architecture = Architecture

# semantic i18n key (#2542)
settings-os-host-readonly-hint = Le nom d'hôte et le fuseau horaire sont en lecture seule à cette étape. Utilisez l'application Terminal ou les outils hôte de l'image kiosque pour les modifier.

# semantic i18n key (#2542)
settings-os-services-desc = Démarrer, arrêter et redémarrer les services hôte

# semantic i18n key (#2542)
settings-os-load-services-hint = Ouvrez le panneau pour charger les services.

# semantic i18n key (#2542)
settings-os-disk-usage-desc = Utilisation du disque des systèmes de fichiers montés

# semantic i18n key (#2542)
settings-os-load-storage-hint = Ouvrez le panneau pour charger le stockage.

# semantic i18n key (#2542)
settings-os-log-files = Fichiers journaux

# semantic i18n key (#2542)
settings-os-log-files-desc = Fichiers écrits par cette installation, avec emplacement et taille

# semantic i18n key (#2542)
settings-os-load-log-files-hint = Ouvrez le panneau pour charger les fichiers journaux.

# semantic i18n key (#2542)
settings-os-tail-log-desc = Afficher les 200 dernières lignes d'un journal de service

# semantic i18n key (#2542)
settings-os-reload = Recharger

# semantic i18n key (#2542)
settings-os-restart-service-title = Redémarrer le service

# semantic i18n key (#2542)
settings-os-the-service = le service

# semantic i18n key (#2542)
settings-os-service-name = Nom du service

# semantic i18n key (#2542)
settings-os-restart-action = Redémarrer

# semantic i18n key (#2542)
settings-os-restart-confirm-message = Redémarrer <strong id="os-restart-service-name" data-i18n="settings-os-the-service">le service</strong> ? Saisissez le nom du service pour confirmer ; le redémarrage sera consigné dans le journal d'audit.

# semantic i18n key (#2542)
settings-profile-directory-managed = Géré par l'annuaire Zitadel

# semantic i18n key (#2542)
settings-profile-open-directory-profile = Ouvrir le profil de l'annuaire

# semantic i18n key (#2542)
settings-profile-public-title = Profil public

# semantic i18n key (#2542)
settings-profile-public-desc = Comment vous apparaissez sur votre page d'accueil General Bots

# semantic i18n key (#2542)
settings-profile-public-display-name = Nom d'affichage public

# semantic i18n key (#2542)
settings-profile-public-url-label = L'URL de votre profil sera

# semantic i18n key (#2542)
settings-profile-coming-soon = Bientôt disponible

# semantic i18n key (#2542)
settings-profile-public-bio = Biographie publique

# semantic i18n key (#2542)
settings-profile-public-url-placeholder = votre-nom.generalbots.org

# semantic i18n key (#2542)
settings-profile-language-region = Langue et région

# semantic i18n key (#2542)
settings-profile-public-url-hint-html = La page sera disponible à l'adresse <strong><span id="public-profile-url-hint" data-i18n="settings-profile-public-url-placeholder">votre-nom.generalbots.org</span></strong> lorsque le relais du site d'accueil General Bots sera livré (#1867).

# semantic i18n key (#2542)
settings-security-directory-note = L'identité, les sessions et la 2FA résident dans l'annuaire Zitadel

# semantic i18n key (#2542)
settings-security-open-directory-security = Ouvrir la sécurité de l'annuaire

# semantic i18n key (#2542)
settings-security-connected-devices = Appareils connectés

# semantic i18n key (#2542)
settings-security-connected-devices-desc = Appareils ayant accédé à votre compte

# semantic i18n key (#2542)
settings-security-local-data = Données locales

# semantic i18n key (#2542)
settings-security-local-data-desc = Cette installation conserve tout sur cet appareil.

# semantic i18n key (#2542)
settings-security-export-local-data = Exporter les données locales

# semantic i18n key (#2542)
settings-security-export-local-data-desc = Télécharger les données stockées sur cet appareil

# semantic i18n key (#2542)
settings-security-clear-local-cache = Vider le cache local

# semantic i18n key (#2542)
settings-security-clear-local-cache-desc = Supprime les paramètres et brouillons mis en cache dans ce profil de navigateur

# semantic i18n key (#2542)
settings-security-clear-cache-action = Vider le cache

# semantic i18n key (#2542)
settings-shared-links-subtitle = Choisissez comment un lien public de fichier parvient à la personne à qui vous l'envoyez

# semantic i18n key (#2542)
settings-shared-links-loopback-warning-title = Seule cette machine est accessible pour le moment.

# semantic i18n key (#2542)
settings-shared-links-loopback-warning-body-html = Un lien de bouclage copié (<code>127.0.0.1</code>) s'ouvre sur l'ordinateur du destinataire, pas sur le vôtre. Utilisez une adresse de réseau local, définissez un domaine ou attendez l'adresse General Bots gratuite avant de partager un lien en dehors de cette machine.

# semantic i18n key (#2542)
settings-shared-links-lan-title = Ce réseau

# semantic i18n key (#2542)
settings-shared-links-lan-desc = Toute personne sur votre réseau Wi-Fi ou filaire

# semantic i18n key (#2542)
settings-shared-links-on-this-computer = Sur cet ordinateur

# semantic i18n key (#2542)
settings-shared-links-on-this-network = Sur ce réseau

# semantic i18n key (#2542)
settings-shared-links-lan-detail = Fonctionne pour toute personne sur le même réseau. Rien ne quitte cette machine.

# semantic i18n key (#2542)
settings-shared-links-lan-config-title = Configuration

# semantic i18n key (#2542)
settings-shared-links-lan-config-body-html = Ce niveau est activé par défaut : le serveur local écoute sur toutes les interfaces. Ouvrez l'adresse réseau depuis un autre appareil du même réseau. Si le trafic entre appareils est bloqué, autorisez le port TCP entrant <code id="shared-links-port">8080</code> sur cet ordinateur.

# semantic i18n key (#2542)
settings-shared-links-lan-config-local-html = Si le nom <code>.local</code> ne se résout pas, utilisez plutôt l'adresse IPv4 de cet ordinateur (par exemple <code>http://192.168.1.20:8080</code>).

# semantic i18n key (#2542)
settings-shared-links-domain-title = Mon propre domaine

# semantic i18n key (#2542)
settings-shared-links-domain-desc = Un nom que vous contrôlez, accessible depuis Internet

# semantic i18n key (#2542)
settings-shared-links-public-link = Lien public

# semantic i18n key (#2542)
settings-shared-links-domain-not-configured = Non configuré — les nouveaux liens utilisent l'adresse locale ci-dessus.

# semantic i18n key (#2542)
settings-shared-links-domain-config-title = Comment configurer

# semantic i18n key (#2542)
settings-shared-links-domain-config-dns-html = Faites pointer un enregistrement DNS (par exemple <code>files.example.com</code>) vers l'adresse publique de cet ordinateur.

# semantic i18n key (#2542)
settings-shared-links-domain-config-url-html = Définissez l'URL de base publique de cette installation : <pre><code>PUBLIC_BASE_URL=https://files.example.com</code></pre> ou, dans le fichier produit : <pre><code>public_base_url=https://files.example.com</code></pre>

# semantic i18n key (#2542)
settings-shared-links-domain-config-restart = Redémarrez l'application de bureau pour que les nouveaux liens de partage utilisent le domaine ; les liens existants conservent leurs jetons.

# semantic i18n key (#2542)
settings-shared-links-domain-config-hint = Le jeton du registraire reste dans le coffre local et n'est jamais écrit dans les journaux. L'actualisation automatique du DNS et les certificats TLS font partie du travail du fournisseur de domaine suivi dans ce ticket ; en attendant, maintenez l'enregistrement à jour chez votre registraire.

# semantic i18n key (#2542)
settings-shared-links-free-title = Adresse General Bots gratuite

# semantic i18n key (#2542)
settings-shared-links-free-desc = Un nom fourni par General Bots, aucun domaine requis

# semantic i18n key (#2542)
settings-shared-links-free-unavailable-html = Pas encore disponible. Le bureau fonctionne hors ligne, ce niveau nécessite donc une zone DNS gratuite et un relais pour les machines injoignables directement. Il est livré désactivé avec le travail <code>botpublic</code> suivi dans #1864.

# semantic i18n key (#2542)
settings-shared-links-free-what-title = Ce que cela fera

# semantic i18n key (#2542)
settings-shared-links-free-what-body-html = Le compte gratuit détiendra un nom — pas vos fichiers. Lorsqu'une machine ne peut pas être atteinte directement, le relais géré (<code id="shared-links-relay-host">peer.generalbots.org</code>) ne conserve qu'une copie chiffrée du fichier partagé et cette copie expire. Aucune configuration manuelle du relais n'est requise.

# semantic i18n key (#2542)
settings-shared-links-peer-title = Interrupteur maître pair à pair

# semantic i18n key (#2542)
settings-shared-links-peer-desc-html = Arrête tout le trafic P2P pour cette installation. Le mode hors ligne maintient chaque connexion sur <code>127.0.0.1</code>.

# semantic i18n key (#2542)
settings-shared-links-offline-default = Hors ligne (par défaut)

# semantic i18n key (#2542)
settings-shared-links-p2p-opt-in = P2P (optionnel)

# semantic i18n key (#2542)
settings-shared-links-relay-hint-html = Le point de terminaison du relais est fixe (<code>peer.generalbots.org</code>). La modification de ce paramètre peut nécessiter le redémarrage de l'application pour prendre pleinement effet.

# semantic i18n key (#2542)
settings-shared-links-check-again = Vérifier à nouveau

# semantic i18n key (#2542)
settings-admin-bots-loading = Chargement des bots...

# semantic i18n key (#2542)
settings-admin-domain-configuration = Configuration du domaine

# semantic i18n key (#2542)
settings-admin-audit-loading = Chargement du journal d'audit...

# semantic i18n key (#2542)
settings-updates-subtitle = Consultez la dernière version sur GitHub et accédez au téléchargement officiel

# semantic i18n key (#2542)
settings-updates-desktop-title = Mises à jour du bureau

# semantic i18n key (#2542)
settings-updates-desktop-checks-html = L'application vérifie <code>github.com/generalbots/generalbots/releases/latest</code>. L'installation est toujours facultative.

# semantic i18n key (#2542)
settings-updates-check-startup = Rechercher des mises à jour au démarrage

# semantic i18n key (#2542)
settings-updates-check-startup-desc = S'exécute au maximum une fois par jour et affiche uniquement une notification. Vous pouvez toujours utiliser « Vérifier maintenant » lorsque cette option est désactivée.

# semantic i18n key (#2542)
settings-updates-current-version = Version actuelle

# semantic i18n key (#2542)
settings-updates-never = Jamais

# semantic i18n key (#2542)
settings-updates-check-now = Vérifier maintenant

# semantic i18n key (#2542)
settings-updates-view-release-notes = Voir les notes de version

# semantic i18n key (#2542)
settings-updates-not-checked-yet = Pas encore vérifié.

# semantic i18n key (#2542)
settings-versions-subtitle = Versions des composants et disponibilité des mises à jour

# semantic i18n key (#2542)
settings-versions-loading = Chargement des informations de version...

# semantic i18n key (#2542)
settings-ledger-description = Chaque action du terminal et du navigateur, avec la décision de consentement, le résultat et le chemin de la capture d'écran.

# semantic i18n key (#2542)
settings-ledger-actions-title = Journal des actions

# semantic i18n key (#2542)
settings-ledger-tab-terminal = Terminal

# semantic i18n key (#2542)
settings-ledger-tab-browser = Navigateur

# semantic i18n key (#2542)
settings-ledger-tab-budgets = Budgets

# semantic i18n key (#2542)
settings-ledger-tab-ttft = TTFT

# semantic i18n key (#2542)
settings-integrations-subtitle = Connectez des services externes et gérez l'accès à l'API

# semantic i18n key (#2542)
settings-open-integrations = Ouvrir les intégrations

# semantic i18n key (#2542)
settings-personal-access-tokens = Jetons d'accès personnels

# semantic i18n key (#2542)
settings-personal-tokens-desc = Jetons que vous créez pour appeler l'API en votre nom (<code>gb_…</code>). Les clés des fournisseurs de modèles se trouvent dans l'application Intégrations.

# semantic i18n key (#2542)
settings-no-api-keys = Aucune clé API créée pour le moment

# semantic i18n key (#2542)
settings-webhooks-title = Webhooks

# semantic i18n key (#2542)
settings-no-webhooks = Aucun webhook configuré

# semantic i18n key (#2542)
settings-full-catalog = Catalogue complet

# semantic i18n key (#2542)
settings-loading-mcp-servers = Chargement des serveurs MCP

# semantic i18n key (#2542)
settings-agent-skills = Compétences de l'agent

# semantic i18n key (#2542)
settings-import-skill = Importer une compétence

# semantic i18n key (#2542)
settings-loading-skills = Chargement des compétences

# semantic i18n key (#2542)
settings-ai-accounts-subtitle = Les abonnements aux forfaits Claude Code et ChatGPT sont par utilisateur. Les identifiants restent dans la VM de votre agent ou dans le coffre local ; seuls le statut et les métadonnées d'affichage sont stockés ici.

# semantic i18n key (#2542)
settings-subscription-accounts = Comptes d'abonnement

# semantic i18n key (#2542)
settings-ai-accounts-desc = Connectez un compte, puis attribuez-le à un type d'agent Vibe

# semantic i18n key (#2542)
settings-loading-ai-accounts = Chargement des comptes IA…

# semantic i18n key (#2542)
settings-vibe-subtitle = Attribuez un fournisseur LLM à chaque type d'agent Vibe. Les fournisseurs sont définis dans Intégrations → Fournisseurs de modèles (BYOK) ou dans la boutique Cloud.

# semantic i18n key (#2542)
settings-agent-llm-providers = Fournisseurs LLM de l'agent

# semantic i18n key (#2542)
settings-agent-llm-providers-desc = Chaque type d'agent achemine ses exécutions vers le fournisseur sélectionné ici pour cette branche

# semantic i18n key (#2542)
settings-loading-agent-providers = Chargement des fournisseurs d'agents…

# semantic i18n key (#2542)
settings-privacy-subtitle = Gérez vos données et vos paramètres de confidentialité

# semantic i18n key (#2542)
settings-privacy-preferences = Préférences de confidentialité

# semantic i18n key (#2542)
settings-profile-visibility = Visibilité du profil

# semantic i18n key (#2542)
settings-profile-visibility-desc = Qui peut voir votre profil

# semantic i18n key (#2542)
settings-contacts-only = Contacts uniquement

# semantic i18n key (#2542)
settings-online-status = Statut en ligne

# semantic i18n key (#2542)
settings-online-status-desc = Afficher quand vous êtes en ligne

# semantic i18n key (#2542)
settings-read-receipts = Accusés de lecture

# semantic i18n key (#2542)
settings-read-receipts-desc = Permettre aux autres de savoir quand vous avez lu les messages

# semantic i18n key (#2542)
settings-your-data = Vos données

# semantic i18n key (#2542)
settings-export-data = Exporter les données

# semantic i18n key (#2542)
settings-export-data-desc = Téléchargez toutes vos données dans un format portable

# semantic i18n key (#2542)
settings-delete-account-title = Supprimer définitivement votre compte et toutes les données associées

# semantic i18n key (#2542)
settings-delete-my-account = Supprimer mon compte

# semantic i18n key (#2542)
settings-memory-subtitle = Ce que l'assistant retient de vous entre les conversations

# semantic i18n key (#2542)
settings-remembered-facts = Faits mémorisés

# semantic i18n key (#2542)
settings-memory-facts-desc = Ces faits sont envoyés avec chaque message afin que les réponses restent cohérentes entre les conversations. Supprimez tout ce que vous ne souhaitez plus que l'assistant utilise.

# semantic i18n key (#2542)
settings-memory-all-scopes = Toutes les portées

# semantic i18n key (#2542)
settings-memory-global-all-bots = Global (tous les bots)

# semantic i18n key (#2542)
settings-memory-bot-specific = Spécifique au bot

# semantic i18n key (#2542)
settings-default-scope = par défaut

# semantic i18n key (#2542)
settings-manage-memories = Gérer les souvenirs

# semantic i18n key (#2542)
settings-manage-memories-desc = Les souvenirs sont créés par l'assistant. Modifiez-les ou supprimez-les ici.

# semantic i18n key (#2542)
settings-no-plan-info = Aucune information de forfait n'est gérée par ce déploiement.

# semantic i18n key (#2542)
settings-manage-plan-on-cloud = Gérer le forfait sur le cloud

# semantic i18n key (#2542)
settings-add-method = Ajouter un moyen

# semantic i18n key (#2542)
settings-no-payment-method = Aucun moyen de paiement enregistré.

# semantic i18n key (#2542)
settings-add-method-on-cloud = Ajouter un moyen sur le cloud

# semantic i18n key (#2542)
settings-no-invoices = Aucune facture pour ce compte.

# semantic i18n key (#2542)
settings-view-invoices-on-cloud = Voir les factures sur le cloud
