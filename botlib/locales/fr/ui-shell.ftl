# =============================================================================
# General Bots — fr — Desktop shell, navigation, controls
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# General Bots - English UI Translations
# =============================================================================
# -----------------------------------------------------------------------------
# Navigation
# -----------------------------------------------------------------------------
nav-home = Accueil
nav-chat = Discuter
nav-drive = Conduire
nav-tasks = Tâches
nav-mail = Courrier
nav-calendar = Calendrier
nav-meet = Rencontre
nav-paper = Papier
nav-video = Vidéo
nav-research = Recherche
nav-analytics = Analyse
nav-settings = Paramètres
nav-admin = Administrateur
nav-monitoring = Surveillance
nav-sources = Sources
nav-tools = Outils
nav-attendant = Préposé
nav-learn = Apprendre
nav-crm = GRC
nav-billing = Facturation
nav-products = Produits
nav-tickets = Billets
nav-docs = Documents
nav-sheet = Feuilles
nav-slides = Diapositives
nav-social = Social
nav-all-apps = Toutes les candidatures
nav-people = Les gens
nav-editor = Éditeur
nav-dashboards = Tableaux de bord
nav-security = Sécurité
nav-designer = Concepteur
nav-project = Projet
nav-canvas = Toile
nav-goals = Objectifs
nav-player = Joueur
nav-workspace = Espace de travail

# -----------------------------------------------------------------------------
# Dashboard
# -----------------------------------------------------------------------------
dashboard-title = Tableau de bord
dashboard-welcome = Bon retour, { $name } !
dashboard-quick-actions = Actions rapides
dashboard-recent-activity = Activité récente
dashboard-no-activity = Aucune activité récente pour l'instant. Commencez à explorer !
dashboard-analytics = Analyse

# -----------------------------------------------------------------------------
# Quick Actions
# -----------------------------------------------------------------------------
quick-start-chat = Démarrer le chat
quick-upload-files = Télécharger des fichiers
quick-new-task = Nouvelle tâche
quick-compose-email = Composer un e-mail
quick-start-meeting = Commencer la réunion
quick-new-event = Nouvel événement

# -----------------------------------------------------------------------------
# Application Cards
# -----------------------------------------------------------------------------
app-chat-name = Discuter
app-chat-desc = Conversations basées sur l'IA. Posez des questions, obtenez de l'aide et automatisez les tâches.
app-drive-name = Conduire
app-drive-desc = Stockage cloud pour tous vos fichiers. Téléchargez, organisez et partagez.
app-tasks-name = Tâches
app-tasks-desc = Restez organisé avec des listes de tâches, des priorités et des dates d'échéance.
app-mail-name = Courrier
app-mail-desc = Client de messagerie avec écriture assistée par l'IA et organisation intelligente.
app-calendar-name = Calendrier
app-calendar-desc = Planifiez des réunions, des événements et gérez votre temps efficacement.
app-meet-name = Rencontre
app-meet-desc = Vidéoconférence avec partage d'écran et transcription en direct.
app-paper-name = Papier
app-paper-desc = Rédigez des documents avec l’aide de l’IA. Notes, rapports et bien plus encore.
app-research-name = Recherche
app-research-desc = Recherche et découverte basées sur l'IA sur toutes vos sources.
app-analytics-name = Analyse
app-analytics-desc = Tableaux de bord et rapports pour suivre l'utilisation et les informations.

# -----------------------------------------------------------------------------
# Suite Header
# -----------------------------------------------------------------------------
suite-title = Suite générale de robots
suite-tagline = Votre espace de travail de productivité alimenté par l'IA. Discutez, collaborez et créez.
suite-new-intent = Nouvelle intention

# -----------------------------------------------------------------------------
# AI Panel
# -----------------------------------------------------------------------------
ai-developer = Développeur IA
ai-developing = En développement : { $project }
ai-quick-actions = Actions rapides
ai-add-field = Ajouter un champ
ai-change-color = Changer de couleur
ai-add-validation = Ajouter une validation
ai-export-data = Exporter des données
ai-placeholder = Tapez vos modifications...
ai-thinking = L'IA pense...
ai-status-online = En ligne
ai-status-offline = Hors ligne

# -----------------------------------------------------------------------------
# Search
# -----------------------------------------------------------------------------
search-placeholder = Rechercher...
search-no-results = Aucun résultat trouvé
search-results = { $compte ->
    [one] { $count } result
   *[other] { $count } results
}
search-in-progress = Recherche...
search-advanced = Recherche avancée
search-filters = Filtres
search-clear-filters = Effacer les filtres

# -----------------------------------------------------------------------------
# Pagination
# -----------------------------------------------------------------------------
# -----------------------------------------------------------------------------
# Tables
# -----------------------------------------------------------------------------
table-no-data = Aucune donnée disponible
table-loading = Chargement des données...
table-actions = Actions
table-select-all = Sélectionner tout
table-deselect-all = Désélectionner tout
table-export = Exporter
table-import = Importer

# -----------------------------------------------------------------------------
# Forms
# -----------------------------------------------------------------------------
form-required = Obligatoire
form-optional = Facultatif
form-submit = Soumettre
form-reset = Réinitialiser
form-clear = Effacer
form-uploading = Téléchargement...
form-processing = Traitement...

# -----------------------------------------------------------------------------
# Modals / Dialogs
# -----------------------------------------------------------------------------
modal-confirm-title = Confirmer l'action
modal-confirm-message = Êtes-vous sûr de vouloir continuer ?
modal-delete-title = Confirmation de suppression
modal-delete-message = Cette action ne peut pas être annulée. Es-tu sûr?

# -----------------------------------------------------------------------------
# Tooltips
# -----------------------------------------------------------------------------
tooltip-copy = Copier dans le presse-papier
tooltip-copied = Copié!
tooltip-expand = Développer
tooltip-collapse = Réduire
tooltip-refresh = Actualiser
tooltip-download = Télécharger
tooltip-upload = Télécharger
tooltip-print = Imprimer
tooltip-fullscreen = Plein écran
tooltip-exit-fullscreen = Quitter le plein écran

# -----------------------------------------------------------------------------
# Common Labels (additional)
# -----------------------------------------------------------------------------
label-username = Nom d'utilisateur
label-email = Courriel
label-display-name = Nom d'affichage
label-password = Mot de passe
label-role = Rôle
label-group-name = Nom du groupe
label-hostname = Nom d'hôte
label-record-type = Type d'enregistrement
label-target = Cible
label-your-name = Votre nom

# -----------------------------------------------------------------------------
# Actions (additional)
# -----------------------------------------------------------------------------
action-register = S'inscrire

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
notifications-title = Notifications
notifications-clear = Tout effacer
notifications-empty = Aucune notification

# #1853 — desktop chrome, launchers and notification settings.
window-ready = Prêt
window-loading = Chargement de l’application…
window-minimize = Réduire
window-maximize = Agrandir
window-fullscreen = Plein écran
window-exit-fullscreen = Quitter le plein écran
window-close = Fermer
app-loading = Chargement de l’application…
start-menu-search-placeholder = Tapez pour trouver une application…
start-menu-recent = Récents
start-menu-no-apps = Aucune application trouvée
start-menu-user = Utilisateur
category-ai = IA
category-business = Entreprise
category-office = Bureau
category-dev = Développement
category-system = Système
category-tools = Outils
sidebar-apps = Applications
sidebar-apps-toggle = Afficher/masquer les applications
sidebar-apps-toggle-aria = Basculer le panneau des applications
sidebar-apps-filter = Filtrer les applications…
sidebar-apps-filter-aria = Filtrer les applications
sidebar-preview = Mode aperçu
sidebar-preview-hint = Afficher les applications encore en aperçu
sidebar-preview-toggle = Basculer le mode aperçu
sidebar-app-preview-title = {app} (aperçu)
sidebar-apps-count = {count} applications
sidebar-apps-count-preview = {count} (+ {preview} en aperçu)
spotlight-placeholder = Recherchez des applications, fichiers… ou saisissez un objectif pour le Concierge
spotlight-hint = ↑↓ naviguer · Entrée exécuter · Ctrl+Space ou Échap fermer
spotlight-empty = Aucun résultat
spotlight-goal = Exécuter l’objectif : “{goal}”
spotlight-goal-subtitle = Concierge · planifier et exécuter dans les applications
palette-placeholder = Recherchez des applications et commandes. Ou tapez @ <commande> pour envoyer à l’assistant...
palette-empty = Aucune application ou commande correspondante
palette-kind-app = Application
palette-hint = ↑↓ naviguer · Entrée exécuter · Échap fermer · Ctrl+K basculer
palette-open-app = Ouvrir {app}
palette-value-prompt = {name} : {summary} — valeur pour {key} :
palette-execute = Exécuter {method} {path} et résumer le résultat.
palette-use-command = Utilisez la commande {command} pour {summary}.
toast-dismiss = Ignorer
settings-language-display = Langue d’affichage
settings-language-display-desc = Choisissez la langue de l’interface de l’application
settings-language-regional = Paramètres régionaux
settings-language-regional-desc = Configurez les formats de date, d’heure et de nombre
settings-language-date-format = Format de date
settings-language-time-format = Format d’heure
settings-language-timezone = Fuseau horaire
settings-language-first-day = Premier jour de la semaine
settings-language-time-12h = 12 heures (1:30 PM)
settings-language-time-24h = 24 heures (13:30)
settings-language-sunday = Dimanche
settings-language-monday = Lundi
settings-email-notifications-desc = Envoyées par l’expéditeur de notifications de l’espace de travail, jamais par un compte Gmail/Outlook connecté.
settings-push-enable = Activer les notifications push
settings-push-enable-desc = Recevez des notifications push du navigateur
settings-push-sound = Son
settings-push-sound-desc = Jouer un son pour les notifications
settings-desktop-notifications = Notifications du bureau
settings-desktop-notifications-desc = Afficher les notifications sur le bureau
settings-badge-count = Compteur de badge
settings-badge-count-desc = Afficher le badge de non-lus
settings-save-preferences = Enregistrer les préférences
settings-notifications-saved = Préférences de notification enregistrées
settings-notification-recipient = Boîte de réception des notifications
settings-notification-recipient-desc = Laissez vide pour utiliser votre compte Mail principal ou l’e-mail du profil.
language-search-placeholder = Rechercher des langues…
settings-profile-title = Paramètres du profil
settings-security-title = Paramètres de sécurité
