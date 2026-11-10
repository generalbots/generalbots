# General Bots — desktop shell and setup wizard (#2089)

desktop-document-title = General Bots Desktop
sidebar-toggle = Afficher ou masquer la barre latérale
sidebar-collapsed-chat = Chat
sidebar-collapsed-terminal = Terminal
sidebar-collapsed-drive = Drive
sidebar-collapsed-settings = Paramètres
sidebar-collapsed-account = Compte
setup-welcome-title = Bienvenue dans General Bots
setup-welcome-subtitle = Configurons tout en quelques étapes rapides
setup-welcome-quick-subtitle = Deux étapes rapides pour commencer
setup-step-llm = Fournisseur LLM
setup-step-profile = Profil utilisateur
setup-step-bot = Configuration du bot
setup-step-training = Entraînement
setup-step-ai-model = Modèle IA
setup-step-your-name = Votre nom
setup-choose-provider-title = Choisissez votre fournisseur LLM
setup-choose-provider-subtitle = Sélectionnez le fournisseur de modèle IA de votre bot
setup-provider-openai-desc = Modèles GPT-4 et GPT-3.5
setup-provider-nvidia-desc = Inférence NVIDIA NIM
setup-provider-anthropic-desc = Modèles Claude 3 et Claude 3.5
setup-provider-local-desc = llama.cpp, Ollama, etc.
setup-profile-title = Qu'est-ce qui vous décrit le mieux ?
setup-profile-subtitle = Sélectionnez votre type de profil pour des valeurs par défaut personnalisées
setup-profile-developer = Développeur
setup-profile-developer-desc = Créer et déployer des bots, écrire des scripts BASIC
setup-profile-business = Entreprise
setup-profile-business-desc = Automatiser les workflows, gérer le CRM
setup-profile-support = Support
setup-profile-support-desc = Service client, gestion des tickets
setup-bot-title = Configurez votre bot
setup-bot-subtitle = Donnez à votre bot un nom, un objectif et éventuellement un modèle
setup-bot-name = Nom du bot
setup-bot-name-placeholder = Mon assistant
setup-bot-purpose = Objectif du bot
setup-bot-purpose-placeholder = Décrivez ce que votre bot doit faire...
setup-bot-template = Modèle de bot
setup-optional = (facultatif)
setup-template-scratch = — Partir de zéro —
setup-name-title = Comment devons-nous vous appeler ?
setup-name-subtitle = Ce nom apparaît dans vos conversations
setup-name-label = Votre nom
setup-name-placeholder = ex. Maria
setup-training-title = Entraînez votre bot
setup-training-subtitle = Déposez des fichiers .pdf, .xls et .doc pour l'entraînement initial. Vous aurez accès à Drive et pourrez le modifier plus tard.
setup-drop-text = Déposez des fichiers ici ou cliquez pour sélectionner
setup-drop-hint = .pdf, .xls, .doc — jusqu'à 10 Mo chacun
setup-select-files = Sélectionner des fichiers
setup-file-status-pending = En attente
setup-file-status-uploading = Téléversement
setup-file-status-done = Terminé
setup-file-status-error = Erreur
setup-saving = Enregistrement...
setup-complete-message = Configuration terminée ! Bienvenue dans General Bots.
setup-failed = Échec de la configuration : {error}
setup-unknown-error = Erreur inconnue
setup-back = Retour
setup-next = Suivant
setup-finish = Terminer
setup-skip = Ignorer la configuration
agent-all-apps = Toutes les applications

# --- Desktop shell panels: versions, accessibility, sidebar, collaboration,
# --- updates, modals and usage (suite/js/*.js) — #2095

versions-component = Composant
versions-version = Version
versions-status = État
versions-action = Action
versions-up-to-date = À jour
versions-unknown = Inconnu
versions-check-update = Vérifier les mises à jour
versions-checking = Vérification...
versions-checked = Vérifié
versions-error = Erreur
versions-update-available = Mise à jour : { $version }
versions-could-not-load = Impossible de charger les informations de version.
versions-ensure-backend = Assurez-vous que le serveur backend est en cours d'exécution.
versions-retry = Réessayer

a11y-chat-conversation = Conversation de chat
a11y-chat-messages = Messages du chat
a11y-send-message = Envoyer un message
a11y-type-message = Saisissez votre message
a11y-keyboard-shortcuts = Raccourcis clavier
a11y-shortcut-focus-chat = Mettre le focus sur la saisie du chat
a11y-shortcut-show-help = Afficher cette aide
a11y-shortcut-escape = Fermer la fenêtre / effacer la sélection
a11y-shortcut-command-palette = Palette de commandes
a11y-shortcut-privacy = Paramètres de confidentialité
a11y-shortcut-scroll-history = Faire défiler l'historique du chat
a11y-shortcut-navigate = Naviguer entre les éléments
a11y-shortcut-activate = Activer l'élément sélectionné
a11y-shortcut-delete-files = Supprimer les fichiers sélectionnés (Drive)
a11y-shortcut-select-all = Tout sélectionner (Drive)
a11y-shortcut-clipboard = Copier / couper / coller des fichiers
a11y-shortcut-navigate-up = Remonter dans Drive
a11y-shortcut-rename = Renommer le fichier sélectionné

sidebar-locked-conversations = Conversations verrouillées
sidebar-no-conversations = Aucune conversation pour l'instant
sidebar-sign-in-conversations = Connectez-vous pour voir vos conversations
sidebar-history-unavailable = Historique indisponible
sidebar-back-to-conversations = ← Retour aux conversations
sidebar-no-locked-conversations = Aucune conversation verrouillée
sidebar-locked-conversation = Conversation verrouillée
sidebar-sign-in = Se connecter
sidebar-add-pin = Ajouter un code PIN

collab-collaborators = Collaborateurs ({ $count })
collab-no-one-else = Personne d'autre n'est ici

update-available-title = Mise à jour disponible — v{ $version }
update-now = Mettre à jour maintenant
update-later = Plus tard
update-skip-version = Ignorer cette version
update-platform = Plateforme : { $name }
update-asset = Ressource : { $name }
update-check-failed = Échec de la vérification des mises à jour : { $error }
update-view-release-notes = Voir les notes de version
update-release-notes-title = Notes de version — v{ $version }
update-no-release-notes = Aucune note de version disponible.

modal-alert-title = Alerte
modal-input-title = Saisie

external-save-to = Enregistrer dans { $target }

usage-title = Utilisation et coût
usage-last-days = { $days } derniers jours
usage-loading = Chargement de l'utilisation…
usage-per-model = Par modèle
usage-per-day = Par jour
usage-model = Modèle
usage-tokens = Jetons
usage-messages = Messages
usage-cost = Coût
usage-day = Jour
usage-days-7 = 7 jours
usage-days-30 = 30 jours
usage-days-90 = 90 jours

pin-close = Fermer
pin-length-hint = 4 à 8 chiffres
pin-cancel = Annuler
pin-ok = OK

# --- Collaboration surfaces shared by every suite app: share dialog, activity
# --- trail, conflict banner and version history (suite/js/collab-*.js) — #2095

role-owner = propriétaire
role-editor = Éditeur
role-commenter = Commentateur
role-viewer = Lecteur

share-request-failed = Échec de la requête ({ $status })
share-warning-title = Les liens partagés ne sont pas encore accessibles
share-warning-body = Cette installation n'est accessible que sur localhost, un lien d'ici ne mène donc nulle part. Configurez d'abord l'une de ces options : activez P2P dans Paramètres → Réseau, ou pointez les liens partagés vers votre propre domaine.
share-open-shared-links = Ouvrir les liens partagés
share-open-network = Ouvrir le réseau
share-invite-people = Inviter des personnes
share-anyone-with-link = Toute personne disposant du lien
share-create-link = Créer un lien
share-transfer-ownership = Transférer la propriété
share-transfer = Transférer
share-new-owner-placeholder = e-mail du nouveau propriétaire
share-owner-suffix = · propriétaire
share-no-people = Aucune personne ajoutée pour l'instant.
share-no-links = Aucun lien public pour l'instant.
share-copied = Copié
share-revoke = Révoquer
share-enter-email = Saisissez un e-mail
share-enter-new-owner = Saisissez l'e-mail du nouveau propriétaire
share-needs-base = Configurez P2P ou votre propre domaine avant de créer un lien partagé.

activity-title = Activité
activity-request-failed = Échec de la requête ({ $status })
activity-load-more = Charger plus
activity-loading = Chargement…
activity-empty = Aucune activité pour l'instant
activity-action-create = a créé
activity-action-edit = a modifié
activity-action-comment = a commenté
activity-action-delete = a supprimé
activity-action-resolve = a résolu un commentaire sur
activity-action-reopen = a rouvert un commentaire sur
activity-action-reaction = a réagi à
activity-action-share = a partagé
activity-action-restore = a restauré
activity-action-transfer = a transféré

conflict-title = Modifications conflictuelles
conflict-dismiss = Ignorer
conflict-edited-concurrently = modifié simultanément ({ $op })
conflict-keep-server = Conserver la version du serveur
conflict-keep-mine = Conserver ma modification
conflict-not-started = Surveillance des conflits non démarrée
conflict-request-failed = Échec de la requête ({ $status })

versions-title = Historique des versions
versions-loading = Chargement…
versions-empty = Aucune version pour l'instant — le premier enregistrement en crée une.
versions-version-n = Version { $n }
versions-current-tag = (actuelle)
versions-name = Nom…
versions-restore-failed = Échec de la restauration : { $error }
versions-rename-failed = Échec du renommage : { $error }
versions-restore-dialog-title = Restaurer la version
versions-restore-confirm = Restaurer cette version ? Une nouvelle version actuelle sera créée — rien n'est perdu.
versions-name-dialog-title = Nommer la version
versions-name-prompt = Nommez cette version (ex. « v2 — approuvée ») :
versions-request-failed = Échec de la requête ({ $status })

# --- Desktop shell: snap picker, workspace tabs, virtual desktops, context
# --- menu, git panel, browser panel, widgets and data export (suite/js/*.js)

snap-title = Ancrer « { $app } »
snap-layout-top-left = En haut à gauche
snap-layout-top = Moitié supérieure
snap-layout-top-right = En haut à droite
snap-layout-left = Moitié gauche
snap-layout-full = Agrandir
snap-layout-right = Moitié droite
snap-layout-bottom-left = En bas à gauche
snap-layout-bottom = Moitié inférieure
snap-layout-bottom-right = En bas à droite
snap-layout-third-left = Tiers gauche
snap-layout-two-thirds = Deux tiers
snap-layout-third-right = Tiers droit

wstabs-close-tab = Fermer l'onglet
wstabs-new-tab = Nouvel onglet d'espace de travail
wstabs-default-name = Espace de travail { $n }
wstabs-default-main = Principal
wstabs-rename-title = Renommer l'espace de travail
wstabs-name-prompt = Nom de l'espace de travail :

vdesk-default-name = Bureau { $n }
vdesk-title = Bureaux virtuels
vdesk-new = Nouveau bureau
vdesk-rename-title = Renommer le bureau
vdesk-name-prompt = Nom du bureau :
vdesk-restore-title = Restaurer la session
vdesk-restore-body = Rouvrir les { $count } fenêtre(s) de votre dernière session sur ce bureau ?
vdesk-restore-body-short = Restaurer votre dernière session ({ $count } fenêtre(s)) ?

ctxmenu-search = Rechercher sur DuckDuckGo
ctxmenu-search-prompt = Rechercher sur DuckDuckGo :
ctxmenu-wikipedia = Définition Wikipédia
ctxmenu-wikipedia-prompt = Définition Wikipédia :

git-loading-status = Chargement de l'état...
git-no-changes = Aucune modification
git-new-branch-prompt = Nom de la nouvelle branche :
git-loading-diff = Chargement du diff...
git-diff-failed = Échec du chargement du diff
git-commit-message-required = Veuillez saisir un message de commit
git-pushing = Push en cours...
git-committed = Validé et poussé avec succès
git-failed = Échec de l'opération Git

browser-session-active = Session de navigateur { $id } active.
browser-init-failed = Échec de l'initialisation du navigateur
browser-init-first = Veuillez d'abord initialiser le navigateur
browser-capture = Capturer
browser-no-session = Aucune session active
browser-export-failed = Échec de l'export
browser-no-actions = Aucune action enregistrée

widget-load-failed = Échec du chargement du widget
widget-no-source = Aucune source de widget configurée
widget-open-windowed = Ouvrir dans une fenêtre
widget-pin-launcher = Épingler au lanceur

dataexport-title = Exportation et sauvegarde des données
dataexport-desc = Exportez toutes vos données, y compris l'historique du chat, la mémoire, les métadonnées de Drive et les paramètres, sous forme de fichier de sauvegarde JSON.
dataexport-export-all = Exporter toutes les données
dataexport-import = Importer une sauvegarde
dataexport-exported = Sauvegarde exportée avec succès
dataexport-too-large = Sauvegarde refusée : le fichier dépasse 4 Mo
dataexport-invalid = Fichier de sauvegarde non valide : { $error }
dataexport-nothing = Cette sauvegarde ne contient aucun paramètre restaurable
dataexport-unknown-date = date inconnue
dataexport-restore-confirm = Cela restaurera { $count } paramètre(s) de la sauvegarde du { $date }. Les identifiants et jetons de session ne sont jamais importés. Continuer ?
dataexport-restored = Sauvegarde restaurée ({ $count } éléments).
dataexport-rejected = { $count } entrée(s) refusée(s) par la liste d'autorisation.
dataexport-reloading = Rechargement...

comments-title = Commentaires
comments-request-failed = Échec de la requête ({ $status })
comments-loading = Chargement des commentaires…
comments-load-failed = Échec du chargement des commentaires : { $error }
comments-empty-open = Aucun commentaire ouvert. Utilisez @nom pour mentionner quelqu'un.
comments-empty-resolved = Aucun commentaire résolu. Utilisez @nom pour mentionner quelqu'un.
comments-resolved-chip = Résolu
comments-resolve = Résoudre
comments-reopen = Rouvrir
comments-resolved-announce = Commentaire résolu
comments-reopened-announce = Commentaire rouvert
comments-resolve-failed = Échec de la résolution : { $error }
comments-added = Commentaire ajouté
comments-post = Publier
comments-post-failed = Échec du commentaire : { $error }
comments-reaction-failed = Échec de la réaction : { $error }
comments-delete-failed = Échec de la suppression : { $error }
comments-reply-failed = Échec de la réponse : { $error }
comments-delete-title = Supprimer le commentaire
comments-delete-confirm = Supprimer ce commentaire ?
comments-reply-title = Répondre
comments-reply-prompt = Répondre au commentaire :
comments-filter-open = Ouverts
comments-filter-all = Tous
comments-filter-resolved = Résolus
comments-input-placeholder = Ajouter un commentaire… utilisez @nom pour mentionner
comments-someone = quelqu'un
comments-mention-announce = Vous avez été mentionné dans un commentaire par { $author }
