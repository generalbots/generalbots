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
# Chat
# -----------------------------------------------------------------------------
chat-title = Discuter
chat-placeholder = Tapez votre message...
chat-send = Envoyer
chat-new-conversation = Nouvelle conversation
chat-history = Historique des discussions
chat-clear = Effacer le chat
chat-export = Exporter la discussion
chat-typing = { $name } est en train d'écrire...
chat-online = En ligne
chat-offline = Hors ligne
chat-last-seen = Vu pour la dernière fois { $time }
chat-mention-title = Entité de référence
chat-mention-placeholder = Message... (tapez @ pour mentionner)
chat-mention-search = Rechercher des entités...
chat-mention-no-results = Aucun résultat trouvé
chat-mention-type-hint = Tapez : pour rechercher

# -----------------------------------------------------------------------------
# Drive / Files
# -----------------------------------------------------------------------------
drive-title = Conduire
drive-upload = Télécharger
drive-new-folder = Nouveau dossier
drive-empty = Aucun fichier pour l'instant. Téléchargez quelque chose !
drive-search = Rechercher des fichiers...
drive-sort-name = Nom
drive-sort-date = Date
drive-sort-size = Taille
drive-sort-type = Tapez
drive-view-grid = Vue Grille
drive-view-list = Vue en liste
drive-selected = { $compte ->
    [one] { $count } item selected
   *[other] { $count } items selected
}
drive-file-size = { $taille ->
    [bytes] { $value } B
    [kb] { $value } KB
    [mb] { $value } MB
    [gb] { $value } GB
   *[other] { $value } bytes
}
drive-drop-files = Déposez les fichiers ici pour les télécharger

# -----------------------------------------------------------------------------
# Tasks
# -----------------------------------------------------------------------------
tasks-title = Tâches
tasks-new = Nouvelle tâche
tasks-due-today = À rendre aujourd'hui
tasks-overdue = En retard
tasks-completed = Terminé
tasks-all = Toutes les tâches
tasks-priority-high = Haute priorité
tasks-priority-medium = Priorité moyenne
tasks-priority-low = Faible priorité
tasks-no-due-date = Pas de date d'échéance
tasks-add-subtask = Ajouter une sous-tâche
tasks-mark-complete = Marquer comme terminé
tasks-mark-incomplete = Marquer comme incomplet
tasks-delete-confirm = Êtes-vous sûr de vouloir supprimer cette tâche ?
tasks-count = { $compte ->
    [zero] No tasks
    [one] { $count } task
   *[other] { $count } tasks
}

# -----------------------------------------------------------------------------
# Calendar
# -----------------------------------------------------------------------------
calendar-title = Calendrier
calendar-today = Aujourd'hui
calendar-new-event = Nouvel événement
calendar-all-day = Toute la journée
calendar-repeat = Répéter
calendar-reminder = Rappel
calendar-view-day = Jour
calendar-view-week = Semaine
calendar-view-month = Mois
calendar-view-year = Année
calendar-no-events = Aucun événement programmé
calendar-event-title = Titre de l'événement
calendar-event-location = Emplacement
calendar-event-description = Descriptif
calendar-event-attendees = Participants

# -----------------------------------------------------------------------------
# Meet / Video Conferencing
# -----------------------------------------------------------------------------
meet-title = Rencontre
meet-join = Rejoindre la réunion
meet-start = Commencer la réunion
meet-mute = Muet
meet-unmute = Activer le son
meet-video-on = Caméra allumée
meet-video-off = Caméra éteinte
meet-share-screen = Partager l'écran
meet-stop-sharing = Arrêter de partager
meet-end-call = Terminer l'appel
meet-leave = Quitter la réunion
meet-participants = { $compte ->
    [one] { $count } participant
   *[other] { $count } participants
}
meet-waiting-room = Salle d'attente
meet-admit = Admettre
meet-remove = Supprimer
meet-chat = Discussion en réunion
meet-raise-hand = Lever la main
meet-lower-hand = Baisser la main
meet-recording = Enregistrement
meet-start-recording = Commencer l'enregistrement
meet-stop-recording = Arrêter l'enregistrement

# -----------------------------------------------------------------------------
# Mail / Email
# -----------------------------------------------------------------------------
mail-title = Courrier
mail-compose = Composer
mail-inbox = Boîte de réception
mail-sent = Envoyé
mail-drafts = Brouillons
mail-trash = Corbeille
mail-spam = Pourriel
mail-starred = Favoris
mail-archive = Archiver
mail-to = À
mail-cc = CC
mail-bcc = Cci
mail-subject = Sujet
mail-body = Message
mail-reply = Répondre
mail-reply-all = Répondre à tous
mail-forward = En avant
mail-send = Envoyer
mail-discard = Jeter
mail-save-draft = Enregistrer le brouillon
mail-attach = Joindre des fichiers
mail-unread = { $compte ->
    [one] { $count } unread
   *[other] { $count } unread
}
mail-empty-inbox = Votre boîte de réception est vide
mail-no-subject = (Aucun sujet)

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
settings-timezone = Fuseau horaire
settings-date-format = Format des dates
settings-time-format = Format de l'heure

# -----------------------------------------------------------------------------
# Auth / Login
# -----------------------------------------------------------------------------
auth-login = Connectez-vous
auth-logout = Se déconnecter
auth-signup = S'inscrire
auth-forgot-password = Mot de passe oublié ?
auth-reset-password = Réinitialiser le mot de passe
auth-email = Courriel
auth-password = Mot de passe
auth-confirm-password = Confirmer le mot de passe
auth-remember-me = Souviens-toi de moi
auth-login-success = Connecté avec succès
auth-logout-success = Déconnecté avec succès
auth-invalid-credentials = Email ou mot de passe invalide
auth-session-expired = Votre session a expiré

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
pagination-previous = Précédent
pagination-next = Suivant
pagination-first = D'abord
pagination-last = Dernier
pagination-page = Page { $current } de { $total }
pagination-showing = Affichage de { $from } à { $to } sur { $total }

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
# Settings - Language & Localization
# -----------------------------------------------------------------------------
settings-language = Langue
settings-language-desc = Choisissez votre langue préférée
settings-display-language = Langue d'affichage
settings-language-affects = Affecte tout le texte de l'application
settings-date-format = Format des dates
settings-date-format-desc = Comment les dates sont affichées
settings-time-format = Format de l'heure
settings-time-format-desc = Horloge 12 heures ou 24 heures
settings-saved = Paramètres enregistrés avec succès
settings-language-changed = La langue a changé avec succès
settings-reload-required = Rechargement de la page requis pour appliquer les modifications

# Settings - Profile
settings-profile = Paramètres du profil
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
settings-security = Paramètres de sécurité
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
settings-theme-dark = Sombre
settings-theme-light = Lumière
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
# Paper (Document Editor)
# -----------------------------------------------------------------------------
paper-title = Papier
paper-new-note = Nouvelle remarque
paper-search-notes = Notes de recherche...
paper-quick-start = Démarrage rapide
paper-template-blank = Vide
paper-template-meeting = Réunion
paper-template-todo = À faire
paper-template-research = Recherche
paper-untitled = Sans titre
paper-placeholder = Commencez à écrire ou tapez / pour les commandes...
paper-commands = Commandes
paper-heading1 = Titre 1
paper-heading1-desc = Titre de grande section
paper-heading2 = Titre 2
paper-heading2-desc = Titre de la section moyenne
paper-heading3 = Titre 3
paper-heading3-desc = Titre de petite section
paper-paragraph = Paragraphe
paper-paragraph-desc = Texte brut
paper-bullet-list = Liste à puces
paper-bullet-list-desc = Liste non ordonnée
paper-numbered-list = Liste numérotée
paper-numbered-list-desc = Liste ordonnée
paper-todo-list = Liste de tâches
paper-todo-list-desc = Liste de tâches vérifiable
paper-quote = Citation
paper-quote-desc = Blockquote pour les citations
paper-divider = Diviseur
paper-divider-desc = Ligne horizontale
paper-code-block = Bloc de code
paper-code-block-desc = Code formaté
paper-table = Tableau
paper-table-desc = Insérer un tableau
paper-image = Images
paper-image-desc = Insérer une image à partir de l'URL
paper-callout = Légende
paper-callout-desc = Zone d'informations en surbrillance
paper-ai-write = Écriture IA
paper-ai-write-desc = Générer du texte avec l'IA
paper-ai-summarize = Résumé de l'IA
paper-ai-summarize-desc = Résumer le texte sélectionné
paper-ai-expand = IA Développer
paper-ai-expand-desc = Développer le texte sélectionné
paper-ai-improve = Amélioration de l'IA
paper-ai-improve-desc = Améliorer la qualité d'écriture
paper-ai-translate = Traduction par l'IA
paper-ai-translate-desc = Traduire dans une autre langue
paper-ai-assistant = Assistant IA
paper-ai-quick-actions = Actions rapides
paper-ai-rewrite = Réécrire
paper-ai-make-shorter = Rendre plus court
paper-ai-make-longer = Faire plus longtemps
paper-ai-fix-grammar = Corriger la grammaire
paper-ai-tone = Ton
paper-ai-tone-professional = Professionnel
paper-ai-tone-casual = Décontracté
paper-ai-tone-friendly = Amical
paper-ai-tone-formal = Formel
paper-ai-translate-to = Traduire en
paper-ai-custom-prompt = Invite personnalisée
paper-ai-custom-placeholder = Décrivez ce que vous voulez...
paper-ai-generate = Générer
paper-ai-response = Réponse de l'IA
paper-ai-apply = Postuler
paper-ai-regenerate = Régénérer
paper-ai-copy = Copier
paper-word-count = { $count } mots
paper-char-count = { $count } caractères
paper-saved = Enregistré
paper-saving = Sauvegarde...
paper-last-edited = Dernière édition : { $time }
paper-last-edited-now = Dernière modification : Tout à l'heure
paper-export = Exporter le document
paper-export-pdf = PDF
paper-export-docx = Mot (.docx)
paper-export-markdown = Démarquage
paper-export-html = HTML
paper-export-txt = Texte brut

# Additional Chat translations
chat-voice = Saisie vocale
chat-message-placeholder = Message...

# Drive translations
drive-my-drive = Mon disque
drive-shared = Partagé avec moi
drive-recent = Récent
drive-starred = Favoris
drive-trash = Corbeille
drive-loading-storage = Chargement du stockage...
drive-storage-used = { $used } sur { $total } utilisés
drive-empty-folder = Ce dossier est vide

# Tasks translations
tasks-active = Intentions actives
tasks-awaiting = En attente de décision
tasks-paused = En pause
tasks-blocked = Bloqué/Problèmes
tasks-time-saved = Temps actif économisé :
tasks-input-placeholder = Qu'aimeriez-vous faire ? par exemple, « créez une application CRM » ou « rappelez-moi d'appeler John demain »

# Calendar additional translations
calendar-my-calendars = Mes calendriers

# Email additional translations
email-scheduled = Programmé
email-tracking = Suivi

# Email folder translations
email-inbox = Boîte de réception
email-starred = Favoris
email-sent = Envoyé
email-drafts = Brouillons
email-spam = Pourriel
email-trash = Corbeille
email-compose = Composer

# -----------------------------------------------------------------------------
# Research
# -----------------------------------------------------------------------------
research-title = Recherche
research-search-placeholder = Demandez n'importe quoi...
research-collections = Collections
research-new-collection = Nouvelle collection
research-recent = Récent
research-academic = Académique
research-code = Coder
research-internal = Interne
research-search-all = Rechercher tout
research-academic-papers = Articles académiques
research-code-docs = Codes et documentation
research-internal-kb = Base de connaissances interne
research-sources = Sources
research-trending = Tendance
research-pro-search = Recherche professionnelle
research-include-images = Inclure des images
research-try-asking = Essayez de poser des questions sur
research-related = Questions connexes
research-view-all-sources = Afficher toutes les sources
research-export-citations = Citations d'exportation
research-save-to-collection = Enregistrer dans la collection

# -----------------------------------------------------------------------------
# Admin Panel (additional UI keys)
# -----------------------------------------------------------------------------
admin-panel-title = Panneau d'administration
admin-quick-actions = Actions rapides
admin-create-user = Créer un utilisateur
admin-create-group = Créer un groupe
admin-register-dns = Enregistrer le DNS
admin-recent-activity = Activité récente
admin-system-health = Santé du système

# -----------------------------------------------------------------------------
# Meet (additional keys)
# -----------------------------------------------------------------------------
meet-new-meeting = Nouvelle réunion
meet-join-meeting = Rejoindre la réunion
meet-active-rooms = Chambres actives
meet-room-title = Salle de réunion
meet-record = Enregistrer
meet-camera = Appareil photo
meet-share = Partager
meet-info = Informations
meet-more = Plus
meet-share-meeting = Partager la réunion
meet-meeting-title = Titre de la réunion
meet-meeting-code = Code de réunion
meet-meeting-link = Lien de réunion
meet-send-invite = Envoyer une invitation

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
# Analytics (additional UI keys)
# -----------------------------------------------------------------------------
analytics-dashboard-title = Tableau de bord d'analyse
analytics-last-hour = Dernière heure
analytics-last-6h = 6 dernières heures
analytics-last-24h = Dernières 24 heures
analytics-last-7d = 7 derniers jours
analytics-last-30d = 30 derniers jours

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
notifications-title = Notifications
notifications-clear = Tout effacer
notifications-empty = Aucune notification

# -----------------------------------------------------------------------------
# All Applications
# -----------------------------------------------------------------------------
nav-all-apps = Toutes les candidatures

# =============================================================================
# AUTH SCREENS - Complete translations for login, register, forgot/reset password
# =============================================================================

# -----------------------------------------------------------------------------
# Login Screen
# -----------------------------------------------------------------------------
auth-welcome-back = Bon retour, { $name } !
auth-sign-in-to-account = Connectez-vous à votre compte General Bots
auth-email-address = Adresse e-mail
auth-email-placeholder = you@example.com
auth-password-placeholder = ••••••••
auth-sign-in = Connectez-vous
auth-or-continue-with = ou continuez avec
auth-dont-have-account = Vous n'avez pas de compte ?
auth-create-account = Créer un compte
auth-google = Google
auth-microsoft = Microsoft
auth-github = GitHub
auth-apple = Pomme

# -----------------------------------------------------------------------------
# Two-Factor Authentication
# -----------------------------------------------------------------------------
auth-2fa-title = Authentification à deux facteurs
auth-2fa-subtitle = Saisissez le code à 6 chiffres de votre application d'authentification
auth-2fa-verify = Vérifier le code
auth-2fa-didnt-receive = Vous n'avez pas reçu de code ?
auth-2fa-resend = Renvoyer le code
auth-2fa-back-to-login = Retour à la connexion
auth-2fa-trust-device = Faites confiance à cet appareil
auth-2fa-trust-desc = Ne demandez pas 2FA sur cet appareil pendant 30 jours

# -----------------------------------------------------------------------------
# Register Screen
# -----------------------------------------------------------------------------
auth-create-your-account = Créez votre compte
auth-join-general-bots = Rejoignez General Bots et commencez à construire
auth-first-name = Prénom
auth-last-name = Nom de famille
auth-create-password = Créer un mot de passe
auth-confirm-your-password = Confirmer le mot de passe
auth-password-strength = Force du mot de passe
auth-password-weak = Faible
auth-password-fair = Foire
auth-password-good = Bon
auth-password-strong = Fort
auth-password-req-length = Au moins 8 caractères
auth-password-req-uppercase = Une lettre majuscule
auth-password-req-lowercase = Une lettre minuscule
auth-password-req-number = Un numéro
auth-password-req-special = Un personnage spécial
auth-passwords-match = Les mots de passe correspondent
auth-passwords-dont-match = Les mots de passe ne correspondent pas
auth-agree-terms = J'accepte le
auth-terms-of-service = Conditions d'utilisation
auth-and = et
auth-privacy-policy = Politique de confidentialité
auth-sign-up = S'inscrire
auth-already-have-account = Vous avez déjà un compte ?
auth-sign-in-link = Connectez-vous
auth-registration-success = Compte créé avec succès !
auth-check-email = Veuillez vérifier votre courrier électronique pour vérifier votre compte
auth-email-sent-to = Nous avons envoyé un lien de vérification à
auth-resend-verification = Renvoyer l'e-mail de vérification
auth-go-to-login = Allez dans Connexion

# -----------------------------------------------------------------------------
# Forgot Password Screen
# -----------------------------------------------------------------------------
auth-forgot-password-title = Mot de passe oublié ?
auth-forgot-password-subtitle = Pas de soucis! Entrez votre e-mail et nous vous enverrons des instructions de réinitialisation.
auth-send-reset-link = Envoyer le lien de réinitialisation
auth-back-to-login = Retour à la connexion
auth-reset-email-sent = Réinitialiser l'e-mail envoyé !
auth-reset-instructions = Nous avons envoyé des instructions de réinitialisation du mot de passe à
auth-check-inbox = Vérifiez votre boîte de réception
auth-check-spam = Vérifiez votre dossier spam si vous ne le voyez pas
auth-link-expires = Le lien expire dans 1 heure
auth-resend-email = Renvoyer l'e-mail
auth-didnt-receive-email = Vous n'avez pas reçu l'e-mail ?

# -----------------------------------------------------------------------------
# Reset Password Screen
# -----------------------------------------------------------------------------
auth-reset-password-title = Réinitialiser le mot de passe
auth-reset-password-subtitle = Créez un nouveau mot de passe sécurisé pour votre compte
auth-new-password = Nouveau mot de passe
auth-confirm-new-password = Confirmer le nouveau mot de passe
auth-reset-password-btn = Réinitialiser le mot de passe
auth-password-reset-success = Réinitialisation du mot de passe réussie !
auth-password-updated = Votre mot de passe a été mis à jour. Vous pouvez maintenant vous connecter avec votre nouveau mot de passe.
auth-invalid-token = Lien invalide ou expiré
auth-invalid-token-desc = Ce lien de réinitialisation de mot de passe n'est pas valide ou a expiré. Veuillez en demander un nouveau.
auth-request-new-link = Demander un nouveau lien

# =============================================================================
# MONITORING SCREENS
# =============================================================================

# -----------------------------------------------------------------------------
# Monitoring Dashboard
# -----------------------------------------------------------------------------
monitoring-title = Tableau de bord de surveillance
monitoring-toggle-view = Basculer la vue
monitoring-last-updated = Dernière mise à jour
monitoring-live-view = Affichage en direct
monitoring-grid-view = Vue Grille

# -----------------------------------------------------------------------------
# Monitoring Panels
# -----------------------------------------------------------------------------
monitoring-sessions = Séances
monitoring-messages = Messages
monitoring-resources = Ressources
monitoring-services = Prestations
monitoring-active-bots = Bots actifs
monitoring-loading = Chargement...

# -----------------------------------------------------------------------------
# Service Status
# -----------------------------------------------------------------------------
monitoring-status-running = Courir
monitoring-status-warning = Avertissement
monitoring-status-stopped = Arrêté
monitoring-status-healthy = Sain
monitoring-status-degraded = Dégradé
monitoring-status-down = Vers le bas

# -----------------------------------------------------------------------------
# Resource Metrics
# -----------------------------------------------------------------------------
monitoring-cpu = Processeur
monitoring-memory = Mémoire
monitoring-disk = Disque
monitoring-network = Réseau
monitoring-requests-per-sec = Requêtes/s
monitoring-active-connections = Connexions actives
monitoring-uptime = Temps de disponibilité

# -----------------------------------------------------------------------------
# Logs
# -----------------------------------------------------------------------------
monitoring-logs-title = Journaux système
monitoring-logs-filter = Filtrer les journaux
monitoring-logs-level = Niveau de journalisation
monitoring-logs-all = Tous les niveaux
monitoring-logs-debug = Débogage
monitoring-logs-info = Informations
monitoring-logs-warning = Avertissement
monitoring-logs-error = Erreur
monitoring-logs-critical = Critique
monitoring-logs-search = Rechercher des journaux...
monitoring-logs-no-results = Aucun journal trouvé

# -----------------------------------------------------------------------------
# Health
# -----------------------------------------------------------------------------
monitoring-health-title = Santé du système
monitoring-health-status = État de santé
monitoring-health-services = Santé des services
monitoring-health-database = Base de données
monitoring-health-cache = Cache
monitoring-health-queue = File d'attente des messages
monitoring-health-storage = Stockage
monitoring-health-external = Services externes

# -----------------------------------------------------------------------------
# Metrics
# -----------------------------------------------------------------------------
monitoring-metrics-title = Mesures de performances
monitoring-metrics-response-time = Temps de réponse
monitoring-metrics-throughput = Débit
monitoring-metrics-error-rate = Taux d'erreur
monitoring-metrics-latency = Latence

# -----------------------------------------------------------------------------
# Alerts
# -----------------------------------------------------------------------------
monitoring-alerts-title = Alertes système
monitoring-alerts-active = Alertes actives
monitoring-alerts-resolved = Résolu
monitoring-alerts-all = Toutes les alertes
monitoring-alert-severity = Gravité
monitoring-alert-critical = Critique
monitoring-alert-high = Élevé
monitoring-alert-medium = Moyen
monitoring-alert-low = Faible
monitoring-alert-info = Informations
monitoring-alert-acknowledge = Reconnaître
monitoring-alert-resolve = Résoudre
monitoring-no-alerts = Aucune alerte active

# =============================================================================
# SOURCES SCREENS
# =============================================================================

# -----------------------------------------------------------------------------
# Sources Main
# -----------------------------------------------------------------------------
sources-title = Sources
sources-subtitle = Dépôts, applications, invites, modèles et serveurs MCP
sources-search = Rechercher des sources...

# -----------------------------------------------------------------------------
# Sources Tabs
# -----------------------------------------------------------------------------
sources-repositories = Dépôts
sources-apps = Applications
sources-prompts = Invites
sources-templates = Modèles
sources-servers = Serveurs MCP
sources-models = Modèles d'IA
sources-news = Actualités

# -----------------------------------------------------------------------------
# Repository Cards
# -----------------------------------------------------------------------------
sources-repo-connect = Se connecter
sources-repo-disconnect = Déconnecter
sources-repo-browse = Parcourir
sources-repo-connected = Connecté
sources-repo-disconnected = Déconnecté
sources-repo-stars = Étoiles
sources-repo-forks = Fourchettes
sources-repo-last-updated = Dernière mise à jour

# -----------------------------------------------------------------------------
# Prompt Cards
# -----------------------------------------------------------------------------
sources-prompt-use = Utiliser
sources-prompt-copy = Copier
sources-prompt-edit = Modifier
sources-prompt-rating = Note
sources-prompt-uses = Utilisations

# -----------------------------------------------------------------------------
# Server Cards
# -----------------------------------------------------------------------------
sources-server-active = Actif
sources-server-inactive = Inactif
sources-server-connect = Se connecter
sources-server-configure = Configurer

# -----------------------------------------------------------------------------
# Model Cards
# -----------------------------------------------------------------------------
sources-model-active = Actif
sources-model-coming-soon = Bientôt disponible
sources-model-provider = Fournisseur
sources-model-context = Contexte
sources-model-tokens = jetons

# -----------------------------------------------------------------------------
# App Cards
# -----------------------------------------------------------------------------
sources-app-open = Ouvert
sources-app-edit = Modifier
sources-app-installed = Installé
sources-app-install = Installer

# -----------------------------------------------------------------------------
# Template Cards
# -----------------------------------------------------------------------------
sources-template-preview = Aperçu
sources-template-use = Utiliser le modèle
sources-template-components = composants

# -----------------------------------------------------------------------------
# Categories
# -----------------------------------------------------------------------------
sources-category-all = Tout
sources-category-development = Développement
sources-category-productivity = Productivité
sources-category-communication = Communications
sources-category-analytics = Analyse
sources-category-security = Sécurité
sources-category-other = Autre

# -----------------------------------------------------------------------------
# Empty States
# -----------------------------------------------------------------------------
sources-empty-repos = Aucun référentiel connecté
sources-empty-apps = Aucune application disponible
sources-empty-prompts = Aucune invite trouvée
sources-empty-templates = Aucun modèle disponible
sources-empty-servers = Aucun serveur MCP configuré
sources-empty-models = Aucun modèle disponible
sources-empty-results = Aucun résultat trouvé
sources-empty-results-desc = Essayez d'ajuster votre recherche ou vos filtres

# =============================================================================
# TOOLS / COMPLIANCE SCREENS
# =============================================================================

# -----------------------------------------------------------------------------
# Compliance Main
# -----------------------------------------------------------------------------
compliance-title = Rapport de conformité des API
compliance-subtitle = Analyse de sécurité pour tous les robots - Vérifiez les mots de passe, le code fragile et les erreurs de configuration
compliance-export-report = Exporter le rapport
compliance-run-scan = Exécuter une analyse de conformité
compliance-scanning = Numérisation...

# -----------------------------------------------------------------------------
# Bot Selector
# -----------------------------------------------------------------------------
compliance-all-bots = Tous les robots
compliance-select-bots = Sélectionnez les robots

# -----------------------------------------------------------------------------
# Stats Cards
# -----------------------------------------------------------------------------
compliance-critical = Critique
compliance-critical-desc = Nécessite une action immédiate
compliance-high = Élevé
compliance-high-desc = Risque de sécurité
compliance-medium = Moyen
compliance-medium-desc = Doit être abordé
compliance-low = Faible
compliance-low-desc = Meilleure pratique
compliance-info = Informations
compliance-info-desc = Informatif

# -----------------------------------------------------------------------------
# Filters
# -----------------------------------------------------------------------------
compliance-filter-severity = Gravité
compliance-filter-type = Tapez
compliance-filter-all-severities = Toutes les gravités
compliance-filter-all-types = Tous types
compliance-search-issues = Problèmes de recherche...

# -----------------------------------------------------------------------------
# Issue Types
# -----------------------------------------------------------------------------
compliance-type-password = Mot de passe dans la configuration
compliance-type-hardcoded = Secrets codés en dur
compliance-type-deprecated = Mots clés obsolètes
compliance-type-fragile = Codes fragiles
compliance-type-config = Problèmes de configuration

# -----------------------------------------------------------------------------
# Results Table
# -----------------------------------------------------------------------------
compliance-results = Résultats
compliance-results-count = { $compte ->
    [one] { $count } issue found
   *[other] { $count } issues found
}
compliance-col-severity = Gravité
compliance-col-issue = Problème
compliance-col-location = Emplacement
compliance-col-details = Détails
compliance-col-action = Action
compliance-view-details = Afficher les détails
compliance-fix-issue = Résoudre le problème
compliance-ignore = Ignorer
compliance-no-issues = Aucun problème trouvé
compliance-no-issues-desc = Génial ! Vos robots sont conformes.

# -----------------------------------------------------------------------------
# Scan Progress
# -----------------------------------------------------------------------------
compliance-scan-in-progress = Scan en cours...
compliance-scan-checking = Vérification { $item }...
compliance-scan-complete = Analyse terminée
compliance-scan-failed = Échec de l'analyse

# =============================================================================
# ATTENDANT / CRM SCREENS
# =============================================================================

# -----------------------------------------------------------------------------
# CRM Disabled State
# -----------------------------------------------------------------------------
attendant-crm-disabled = Fonctionnalités CRM non activées
attendant-crm-disabled-desc = La console Attendant nécessite que les fonctionnalités CRM soient activées pour ce bot. Cela permet aux agents humains de recevoir et de répondre aux conversations transférées depuis le bot.
attendant-crm-enable-instruction = Pour activer les fonctionnalités CRM, ajoutez cette ligne au nom de votre bot
attendant-crm-config-file = config.csv
attendant-crm-create-attendant = Créez ensuite un
attendant-crm-attendant-file = accompagnant.csv
attendant-crm-configure-team = fichier pour configurer votre équipe

# -----------------------------------------------------------------------------
# Queue Sidebar
# -----------------------------------------------------------------------------
attendant-title = Console opérateur
attendant-status-online = En ligne
attendant-status-busy = Occupé
attendant-status-away = Absent
attendant-status-offline = Hors ligne
attendant-status-ready = En ligne - Prêt pour les conversations
attendant-status-busy-msg = Occupé – Gérer les conversations
attendant-status-away-msg = Absent - Je reviendrai bientôt
attendant-status-offline-msg = Hors ligne - Non disponible

# -----------------------------------------------------------------------------
# Queue Stats
# -----------------------------------------------------------------------------
attendant-waiting = En attente
attendant-active = Actif
attendant-resolved = Résolu
attendant-mine = Le mien

# -----------------------------------------------------------------------------
# Queue Filters
# -----------------------------------------------------------------------------
attendant-filter-all = Tout
attendant-filter-waiting = En attente
attendant-filter-mine = Le mien
attendant-filter-priority = Priorité

# -----------------------------------------------------------------------------
# Conversation List
# -----------------------------------------------------------------------------
attendant-no-conversations = Aucune conversation en file d'attente
attendant-new-conversations-appear = De nouvelles conversations apparaîtront ici
attendant-unread = Non lu
attendant-typing = en tapant...
attendant-select-conversation = Sélectionnez une conversation
attendant-select-conversation-desc = Choisissez une conversation dans la file d'attente pour commencer à répondre

# -----------------------------------------------------------------------------
# Channel Tags
# -----------------------------------------------------------------------------
attendant-channel-whatsapp = WhatsApp
attendant-channel-teams = Équipes
attendant-channel-instagram = Instagram
attendant-channel-web = Web
attendant-channel-telegram = Télégramme
attendant-channel-email = Courriel

# -----------------------------------------------------------------------------
# Priority Tags
# -----------------------------------------------------------------------------
attendant-priority-urgent = Urgent
attendant-priority-high = Élevé
attendant-priority-normal = Normale

# -----------------------------------------------------------------------------
# Chat Area
# -----------------------------------------------------------------------------
attendant-message-placeholder = Tapez votre message...
attendant-send = Envoyer
attendant-attach-file = Joindre un fichier
attendant-insert-emoji = Insérer un emoji
attendant-quick-responses = Réponses rapides
attendant-transfer = Transfert
attendant-resolve = Résoudre
attendant-more-actions = Plus de mesures

# -----------------------------------------------------------------------------
# Quick Responses
# -----------------------------------------------------------------------------
attendant-quick-greeting = Bonjour ! Comment puis-je vous aider aujourd'hui ?
attendant-quick-thanks = Merci pour votre patience.
attendant-quick-checking = Laissez-moi vérifier cela pour vous.
attendant-quick-moment = Un instant s'il vous plaît.

# -----------------------------------------------------------------------------
# Transfer Modal
# -----------------------------------------------------------------------------
attendant-transfer-title = Conversation de transfert
attendant-transfer-to = Transférer à
attendant-transfer-reason = Raison (facultatif)
attendant-transfer-reason-placeholder = Pourquoi transférez-vous cette conversation ?
attendant-transfer-cancel = Annuler
attendant-transfer-confirm = Transfert

# -----------------------------------------------------------------------------
# AI Insights Sidebar
# -----------------------------------------------------------------------------
attendant-ai-insights = Informations sur l'IA
attendant-ai-summary = Résumé de la conversation
attendant-ai-sentiment = Sentiment des clients
attendant-sentiment-positive = Positif
attendant-sentiment-neutral = Neutre
attendant-sentiment-negative = Négatif
attendant-smart-replies = Réponses intelligentes
attendant-confidence = Confiance
attendant-source = Source

# -----------------------------------------------------------------------------
# Customer Details
# -----------------------------------------------------------------------------
attendant-customer-details = Détails du client
attendant-customer-name = Nom
attendant-customer-email = Courriel
attendant-customer-phone = Téléphone
attendant-customer-location = Emplacement
attendant-customer-tags = Balises

# -----------------------------------------------------------------------------
# Conversation History
# -----------------------------------------------------------------------------
attendant-history = Histoire
attendant-history-resolved = Résolu
attendant-history-transferred = Transféré
attendant-history-abandoned = Abandonné
attendant-view-history = Afficher l'historique complet

# -----------------------------------------------------------------------------
# Toast Messages
# -----------------------------------------------------------------------------
attendant-toast-transferred = Conversation transférée avec succès
attendant-toast-resolved = Conversation marquée comme résolue
attendant-toast-assigned = Conversation qui vous est attribuée
attendant-toast-error = Une erreur s'est produite
attendant-toast-connection-lost = Connexion perdue. Reconnexion...
attendant-toast-connection-restored = Connexion rétablie

# =============================================================================
# CRM
# =============================================================================

# -----------------------------------------------------------------------------
# CRM Navigation & General
# -----------------------------------------------------------------------------
crm-title = GRC
crm-pipeline = Pipeline
crm-leads = Pistes
crm-opportunities = Opportunités
crm-accounts = Comptes
crm-contacts = Contacts
crm-activities = Activités

# -----------------------------------------------------------------------------
# CRM Entities
# -----------------------------------------------------------------------------
crm-lead = Diriger
crm-lead-desc = Prospect non qualifié
crm-opportunity = Opportunité
crm-opportunity-desc = Opportunité de vente qualifiée
crm-account = Compte
crm-account-desc = Entreprise ou organisation
crm-contact = Contacter
crm-contact-desc = Personne sur un compte
crm-activity = Activité
crm-activity-desc = Tâche, appel ou e-mail

# -----------------------------------------------------------------------------
# CRM Actions
# -----------------------------------------------------------------------------
crm-qualify = Qualifier
crm-convert = Convertir
crm-won = Gagné
crm-lost = Perdu
crm-new-lead = Nouveau responsable
crm-new-opportunity = Nouvelle opportunité
crm-new-account = Nouveau compte
crm-new-contact = Nouveau contact

# -----------------------------------------------------------------------------
# CRM Fields
# -----------------------------------------------------------------------------
crm-stage = Scène
crm-value = Valeur
crm-probability = Probabilité
crm-close-date = Date de clôture
crm-company = Entreprise
crm-phone = Téléphone
crm-email = Courriel
crm-source = Source
crm-owner = Propriétaire

# -----------------------------------------------------------------------------
# CRM Pipeline Stages
# -----------------------------------------------------------------------------
crm-pipeline-new = Nouveau
crm-pipeline-contacted = Contacté
crm-pipeline-qualified = Qualifié
crm-pipeline-proposal = Proposition
crm-pipeline-negotiation = Négociation
crm-pipeline-closed-won = Fermé Gagné
crm-pipeline-closed-lost = Fermé Perdu

# -----------------------------------------------------------------------------
# CRM Stats & Metrics
# -----------------------------------------------------------------------------
crm-subtitle = Gérer les leads, les opportunités et les clients
crm-stage-lead = Diriger
crm-stage-qualified = Qualifié
crm-stage-proposal = Proposition
crm-stage-negotiation = Négociation
crm-stage-won = Gagné
crm-stage-lost = Perdu
crm-conversion-rate = Taux de conversion
crm-pipeline-value = Valeur du pipeline
crm-avg-deal = Taille moyenne de la transaction
crm-won-month = Gagné ce mois-ci

# -----------------------------------------------------------------------------
# CRM Empty States
# -----------------------------------------------------------------------------
crm-no-leads = Aucune piste trouvée
crm-no-opportunities = Aucune opportunité trouvée
crm-no-accounts = Aucun compte trouvé
crm-no-contacts = Aucun contact trouvé
crm-drag-hint = Faites glisser les cartes pour changer d'étape

# =============================================================================
# Billing
# =============================================================================

# -----------------------------------------------------------------------------
# Billing Navigation & General
# -----------------------------------------------------------------------------
billing-title = Facturation
billing-invoices = Factures
billing-payments = Paiements
billing-quotes = Citations
billing-dashboard = Tableau de bord

# -----------------------------------------------------------------------------
# Billing Entities
# -----------------------------------------------------------------------------
billing-invoice = Facture
billing-invoice-desc = Facture au client
billing-payment = Paiement
billing-payment-desc = Paiement reçu
billing-quote = Citation
billing-quote-desc = Devis

# -----------------------------------------------------------------------------
# Billing Status
# -----------------------------------------------------------------------------
billing-due-date = Date d'échéance
billing-overdue = En retard
billing-paid = Payé
billing-pending = En attente
billing-draft = Brouillon
billing-sent = Envoyé
billing-partial = Partielle
billing-cancelled = Annulé

# -----------------------------------------------------------------------------
# Billing Actions
# -----------------------------------------------------------------------------
billing-new-invoice = Nouvelle facture
billing-new-quote = Nouveau devis
billing-new-payment = Nouveau paiement
billing-send-invoice = Envoyer la facture
billing-record-payment = Enregistrer le paiement
billing-mark-paid = Marquer comme payé
billing-void = Vide

# -----------------------------------------------------------------------------
# Billing Fields
# -----------------------------------------------------------------------------
billing-amount = Montant
billing-tax = Taxe
billing-subtotal = Sous-total
billing-total = Total
billing-discount = Remise
billing-line-items = Éléments de campagne
billing-add-item = Ajouter un article
billing-remove-item = Supprimer l'élément
billing-customer = Client
billing-issue-date = Date d'émission
billing-payment-terms = Conditions de paiement
billing-notes = Remarques
billing-invoice-number = Numéro de facture
billing-quote-number = Numéro de devis

# -----------------------------------------------------------------------------
# Billing Reports
# -----------------------------------------------------------------------------
billing-revenue = Revenus
billing-outstanding = Exceptionnel
billing-this-month = Ce mois-ci
billing-last-month = Le mois dernier
billing-total-paid = Total payé
billing-total-overdue = Total en retard
billing-subtitle = Factures, paiements et devis
billing-revenue-month = Revenus ce mois-ci
billing-total-revenue = Revenu total
billing-paid-month = Payé ce mois-ci

# -----------------------------------------------------------------------------
# Billing Empty States
# -----------------------------------------------------------------------------
billing-no-invoices = Aucune facture trouvée
billing-no-payments = Aucun paiement trouvé
billing-no-quotes = Aucune citation trouvée

# =============================================================================
# Products
# =============================================================================

# -----------------------------------------------------------------------------
# Products Navigation & General
# -----------------------------------------------------------------------------
products-title = Produits
products-catalog = Catalogue
products-services = Prestations
products-price-lists = Listes de prix
products-inventory = Inventaire

# -----------------------------------------------------------------------------
# Products Entities
# -----------------------------------------------------------------------------
products-product = Produit
products-product-desc = Produit physique ou numérique
products-service = Service
products-service-desc = Offre de services
products-price-list = Liste de prix
products-price-list-desc = Niveaux de tarification

# -----------------------------------------------------------------------------
# Products Actions
# -----------------------------------------------------------------------------
products-new-product = Nouveau produit
products-new-service = Nouveau service
products-new-price-list = Nouvelle liste de prix
products-new-pricelist = Nouvelle liste de prix
products-edit-product = Modifier le produit
products-duplicate = Dupliquer

# -----------------------------------------------------------------------------
# Products Fields
# -----------------------------------------------------------------------------
products-sku = UGS
products-category = Catégorie
products-price = Prix
products-unit = Unité
products-stock = Actions
products-cost = Coût
products-margin = Marge
products-barcode = Code à barres

# -----------------------------------------------------------------------------
# Products Status
# -----------------------------------------------------------------------------
products-in-stock = En stock
products-out-of-stock = En rupture de stock
products-low-stock = Stock faible
products-active = Actif
products-inactive = Inactif
products-featured = En vedette
products-archived = Archivé

# -----------------------------------------------------------------------------
# Products Stats & Metrics
# -----------------------------------------------------------------------------
products-subtitle = Gérer les produits, les services et les prix
products-items = Produits
products-pricelists = Listes de prix
products-total-products = Produits totaux
products-total-services = Services totaux

# -----------------------------------------------------------------------------
# Products Empty States
# -----------------------------------------------------------------------------
products-no-products = Aucun produit trouvé
products-no-services = Aucun service trouvé
products-no-price-lists = Aucune liste de prix trouvée

# =============================================================================
# Tickets (Support Cases)
# =============================================================================

# -----------------------------------------------------------------------------
# Tickets Navigation & General
# -----------------------------------------------------------------------------
tickets-title = Billets
tickets-cases = Cas
tickets-open = Ouvert
tickets-closed = Fermé
tickets-all = Tous les billets
tickets-my-tickets = Mes billets

# -----------------------------------------------------------------------------
# Tickets Entities
# -----------------------------------------------------------------------------
tickets-case = Cas
tickets-case-desc = Billet d'assistance
tickets-resolution = Résolution
tickets-resolution-desc = Solution suggérée par l'IA

# -----------------------------------------------------------------------------
# Tickets Priority
# -----------------------------------------------------------------------------
tickets-priority = Priorité
tickets-priority-low = Faible
tickets-priority-medium = Moyen
tickets-priority-high = Élevé
tickets-priority-urgent = Urgent

# -----------------------------------------------------------------------------
# Tickets Status
# -----------------------------------------------------------------------------
tickets-status = Statut
tickets-status-new = Nouveau
tickets-status-open = Ouvert
tickets-status-pending = En attente
tickets-status-resolved = Résolu
tickets-status-closed = Fermé
tickets-status-on-hold = En attente

# -----------------------------------------------------------------------------
# Tickets Actions
# -----------------------------------------------------------------------------
tickets-new-ticket = Nouveau billet
tickets-assign = Attribuer
tickets-reassign = Réaffecter
tickets-escalate = Escalader
tickets-resolve = Résoudre
tickets-reopen = Réouvrir
tickets-close = Fermer
tickets-merge = Fusionner

# -----------------------------------------------------------------------------
# Tickets Fields
# -----------------------------------------------------------------------------
tickets-subject = Sujet
tickets-description = Descriptif
tickets-category = Catégorie
tickets-assigned = Attribué à
tickets-unassigned = Non attribué
tickets-created = Créé
tickets-updated = Mis à jour
tickets-response-time = Temps de réponse
tickets-resolution-time = Temps de résolution
tickets-customer = Client
tickets-internal-notes = Notes internes
tickets-attachments = Pièces jointes

# -----------------------------------------------------------------------------
# Tickets AI Features
# -----------------------------------------------------------------------------
tickets-ai-suggestion = Suggestions d'IA
tickets-apply-suggestion = Appliquer la suggestion
tickets-ai-summary = Résumé de l'IA
tickets-similar-tickets = Billets similaires
tickets-suggested-articles = Articles suggérés

# -----------------------------------------------------------------------------
# Tickets Empty States
# -----------------------------------------------------------------------------
tickets-no-tickets = Aucun billet trouvé
tickets-no-open = Pas de billets ouverts
tickets-no-closed = Pas de billets fermés

# -----------------------------------------------------------------------------
# Security Module
# -----------------------------------------------------------------------------
security-title = Sécurité
security-subtitle = Gérer les paramètres de sécurité de votre compte
security-tab-compliance = Rapport de conformité des API
security-tab-protection = Protection
security-export-report = Exporter le rapport
security-run-scan = Exécuter une analyse de conformité
security-critical = Critique
security-critical-desc = Action immédiate requise
security-high = Élevé
security-high-desc = Risque de sécurité
security-medium = Moyen
security-medium-desc = Doit être abordé
security-low = Faible
security-low-desc = Meilleure pratique
security-info = Informations
security-info-desc = Informatif
security-filter-severity = Gravité :
security-filter-all-severities = Toutes les gravités
security-filter-type = Tapez :
security-filter-all-types = Tous types
security-type-password = Mot de passe dans la configuration
security-type-hardcoded = Secrets codés en dur
security-type-deprecated = Mots clés obsolètes
security-type-fragile = Codes fragiles
security-type-config = Problèmes de configuration
security-results = Problèmes de conformité
security-col-severity = Gravité
security-col-issue = Type de problème
security-col-location = Emplacement
security-col-details = Descriptif
security-col-action = Action

# -----------------------------------------------------------------------------
# Learn Module
# -----------------------------------------------------------------------------
learn-title = Apprendre
learn-my-progress = Mes progrès
learn-completed = Terminé
learn-in-progress = En cours
learn-certificates = Certificats
learn-time-spent = Temps passé
learn-categories = Catégories
learn-all-courses = Tous les cours
learn-mandatory = Obligatoire
learn-compliance = Conformité
learn-security = Sécurité
learn-skills = Compétences
learn-onboarding = Intégration
learn-difficulty = Difficulté
learn-my-certificates = Mes certificats
learn-view-all = Tout afficher

# -----------------------------------------------------------------------------
# Workspace Module
# -----------------------------------------------------------------------------
workspace-title = Espace de travail
workspace-search-pages = Pages de recherche...
workspace-recent = Récent
workspace-favorites = Favoris
workspace-pages = Pages
workspace-templates = Modèles
workspace-trash = Corbeille
workspace-settings = Paramètres

# -----------------------------------------------------------------------------
# Player Module
# -----------------------------------------------------------------------------
player-title = Lecteur multimédia
player-no-file = Aucun fichier sélectionné
player-search = Rechercher des fichiers...
player-recent = Récent
player-files = Fichiers

# -----------------------------------------------------------------------------
# Goals Module
# -----------------------------------------------------------------------------
goals-title = Objectifs et OKR
goals-dashboard = Tableau de bord
goals-objectives = Objectifs
goals-alignment = Alignement
goals-ai-suggestions = Suggestions d'IA

# CRM / Mail / Campaigns integration keys
crm-email = Courriel
crm-compose-email = Composer un e-mail
crm-send-email = Envoyer un e-mail
mail-snooze = Répéter
mail-snooze-later-today = Plus tard dans la journée (18h00)
mail-snooze-tomorrow = Demain (8h00)
mail-snooze-next-week = La semaine prochaine (lundi 8h00)
mail-crm-log = Connectez-vous au CRM
mail-crm-create-lead = Créer un prospect
mail-add-to-list = Ajouter à la liste
campaign-send-email = Envoyer un e-mail

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

## Compliance frameworks (enterprise-grade release)
compliance-frameworks = Cadres
compliance-new-framework = Nouveau
compliance-framework-name = Nom
compliance-framework-version = Version
compliance-framework-description = Descriptif
compliance-create-framework = Créer un cadre
compliance-controls = Contrôles
compliance-add-control = Ajouter un contrôle
compliance-control-id = ID de contrôle
compliance-control-title = Titre
compliance-control-category = Catégorie
compliance-control-description = Descriptif
compliance-mandatory = Obligatoire
compliance-optional = Facultatif
compliance-evidence = Preuve
compliance-attach-evidence = Joindre des preuves
compliance-evidence-path = Chemin du fichier (artefact de lecteur)
compliance-evidence-type = Tapez
compliance-approve = Approuver
compliance-covered = Couvert
compliance-no-evidence = Aucune preuve
compliance-export-csv = Exporter au format CSV
compliance-archive = Archiver
compliance-total-controls = Contrôles totaux
compliance-coverage = Couverture
compliance-no-frameworks = Aucun framework n'est encore configuré.

## Sources connectors (enterprise-grade release)
sources-connectors = Connecteurs
sources-add-connector = Ajouter un connecteur
sources-connector-name = Nom
sources-connector-description = Descriptif
sources-connector-schedule = Calendrier de synchronisation (cron)
sources-connector-type = Tapez
sources-connector-host = Hôte
sources-connector-port = Port
sources-connector-database = Base de données
sources-connector-username = Nom d'utilisateur
sources-connector-password = Mot de passe
sources-connector-base-url = URL de base
sources-connector-api-key = Clé API
sources-connector-credentials-hint = Les informations d'identification sont stockées dans Vault et ne s'affichent plus jamais après l'enregistrement.
sources-create-connector = Créer un connecteur
sources-test-connector = Tester
sources-sync-now = Synchronisez maintenant
sources-remove-connector = Supprimer
sources-connector-health = Santé
sources-connector-last-sync = Dernière synchronisation
sources-no-connectors = Aucun connecteur configuré

# VDI (remote desktop)
vdi-title = Bureau virtuel
vdi-new-connection = Nouvelle connexion
vdi-connection-name = Nom de la connexion
vdi-host = Hôte
vdi-port = Port
vdi-protocol = Protocole
vdi-rdp-password = Mot de passe RDP
vdi-rdp-domain = Domaine RDP (facultatif)
vdi-save-connect = Enregistrer et se connecter
vdi-cancel = Annuler
vdi-connect = Se connecter
vdi-delete = Supprimer
vdi-no-connections = Aucune connexion pour l'instant
vdi-create-first = Créez une nouvelle connexion pour commencer
vdi-connecting = Connexion...
vdi-connected = Connecté
vdi-disconnected = Déconnecté
vdi-error = Erreur
vdi-clipboard-sent = Presse-papiers envoyé
vdi-ctrl-alt-del-sent = Ctrl+Alt+Suppr envoyé
vdi-rdp = RDP
vdi-vnc = VNC
attendant-attach = Joindre un fichier
attendant-emoji = Emoji
attendant-uploading = Téléversement...
attendant-attach-error = Échec du téléversement
attendant-emoji-search = Rechercher un emoji...
