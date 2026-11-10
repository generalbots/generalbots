# =============================================================================
# General Bots — fr — Productivity apps (tasks, calendar, meet, mail, research, paper)
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

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
