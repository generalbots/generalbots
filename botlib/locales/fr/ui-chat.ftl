# =============================================================================
# General Bots — fr — Chat
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

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

# Additional Chat translations
chat-camera = Caméra
chat-camera-title = Prendre une photo
chat-camera-capture = Prendre la photo
chat-camera-cancel = Annuler
chat-camera-sent = Photo envoyée.
chat-camera-denied = L'autorisation de la caméra a été refusée. Autorisez l'accès à la caméra pour l'application de bureau dans les paramètres système pour prendre des photos.
chat-camera-missing = Aucune caméra n'a été trouvée sur cet appareil.
chat-camera-error = Impossible de démarrer la caméra.
chat-camera-unsupported = Cette application de bureau ne fournit pas l'accès à la caméra.
voice-mode-title = Conversation vocale
voice-mode-listening = Écoute… parlez naturellement
voice-mode-thinking = Réflexion…
voice-mode-speaking = Parle…
voice-mode-idle = Conversation vocale
voice-mode-end = Terminer la conversation vocale
chat-message-placeholder = Message...

# Chat message controls and notices
chat-copy-message = Copier le message
chat-copied = Copié
chat-message-copied = Message copié
chat-copy-failed = Échec de la copie
chat-listen-tts = Écouter (TTS)
chat-listen-message = Écouter ce message
chat-image-unavailable = image indisponible
chat-open = Ouvrir
chat-expand = Développer
chat-collapse = Réduire
chat-preview = Aperçu
chat-consent-affected-count = Enregistrements concernés
chat-close-preview = Fermer l'aperçu
chat-preview-load-failed = Impossible de charger l'aperçu.
chat-show-more = Afficher plus
chat-show-less = Afficher moins
chat-tool = outil
chat-param = paramètre
chat-params = paramètres
chat-tool-result = Résultat de l'outil
chat-retry = Réessayer
chat-open-as-document = Ouvrir comme document
chat-show-more-history = afficher plus…
chat-disconnected-reconnecting = Discussion déconnectée. Reconnexion...
chat-connect-retrying = Impossible d'atteindre le serveur de discussion. Nouvelle tentative...
chat-connection-lost = Connexion perdue. Tentative de reconnexion...
chat-attachment-sent = Pièce jointe envoyée : 
chat-attachments-sent = Pièces jointes envoyées : 
chat-message-not-delivered = Message non livré — reconnexion
chat-attachment-read-failed = Impossible de lire une pièce jointe. Joignez-la de nouveau.
chat-attachment-queued = La connexion revient bientôt — pièce jointe en file d'attente
chat-reconnecting-queued = Reconnexion — votre message sera envoyé lorsque la connexion sera rétablie.
chat-bot-private-signin = Ce bot est privé — connectez-vous pour continuer
chat-approved-plan = Plan approuvé : 
chat-retrying-attachment = Nouvelle tentative de pièce jointe
chat-expanded-preview = Aperçu agrandi

# #2362 — chat code/artifact control labels
chat-code-show-all = Show all
chat-code-lines = lines
chat-artifact-save-drive = Save to Drive
chat-artifact-download-svg = Download SVG
chat-artifact-download-png = Download PNG
chat-artifact-open-canvas = Open in Canvas
chat-artifact-folder = Folder (optional)
chat-artifact-kind-markdown = Markdown document
chat-artifact-kind-html = HTML page
chat-artifact-kind-code = Code
chat-artifact-words = words

# #2362 — attachment action label
chat-open-in-app = Open in app
# Assistant-look chat: transcript export and answer feedback
chat-export-actions = Actions de la conversation
chat-export-md = Exporter la conversation en Markdown
chat-export-print = Imprimer ou enregistrer en PDF
chat-export-done = Conversation exportée
chat-export-empty = Rien à exporter pour l'instant
chat-export-untitled = Conversation
chat-export-user = Vous
chat-export-assistant = Assistant
chat-feedback-label = Avis sur la réponse
chat-feedback-good = Bonne réponse
chat-feedback-bad = Mauvaise réponse
chat-feedback-comment = Que pourrait-on améliorer ?
chat-feedback-send = Envoyer
chat-staged-files = Fichiers joints
chat-staged-close = Fermer l'aperçu
chat-camera-waiting = Waiting for the camera to start…
chat-camera-waiting-frames = Permission granted; waiting for video frames
chat-camera-ready = Camera ready. Take the photo or cancel.
chat-camera-started = Camera started — tap Take photo.
chat-camera-no-device = no video device
chat-camera-device = video device
chat-camera-no-frames = The camera preview did not start. Choose a photo from the system picker instead.
chat-camera-timeout = The camera did not answer. Choose a photo from the system picker instead.

# Chat engineer status strip (#2342)
chat-status-bot = Bot
chat-status-session = Session
chat-status-model = Modèle
chat-status-state = État
chat-status-no-session = —
chat-status-copy-session = Copier l’ID de session
chat-status-copied = Copié
chat-status-idle = Inactif
chat-status-streaming = Diffusion
chat-status-thinking = Réflexion
chat-status-error = Erreur
chat-status-stopped = Arrêté

chat-response-ready = Réponse prête
chat-response-started = L'assistant répond
chat-tool-result-ready = Résultat de l'outil disponible

# semantic i18n key (#2542)
chat-no-suggestions-available = Aucune suggestion disponible

# semantic i18n key (#2542)
chat-start-conversation = Démarrer une conversation

# semantic i18n key (#2542)
chat-empty-hint = Saisissez un message ci-dessous ou utilisez la saisie vocale

# semantic i18n key (#2542)
chat-no-conversations-yet = Aucune conversation pour l'instant

# semantic i18n key (#2542)
chat-context-label = Contexte :

# semantic i18n key (#2542)
chat-select-context = Sélectionner un contexte...

# semantic i18n key (#2542)
chat-no-contexts-configured = Aucun contexte configuré

# semantic i18n key (#2542)
projector-title = Visionneuse de contenu

# semantic i18n key (#2542)
projector-loading-content = Chargement du contenu...

# semantic i18n key (#2542)
projector-slide-info = Diapositive 1 sur 1

# semantic i18n key (#2542)
projector-code-info = code.rs

# Projector viewer control (#2542)
chat-projector-rotate = Tourner

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
chat-empty-title = Commencez la conversation
chat-empty-desc = Saisissez un message ci-dessous ou choisissez une suggestion proposée par le bot.
chat-empty-action = Écrire le premier message

# -----------------------------------------------------------------------------
# #2342 — chat engineer UX: slash commands, help, inspect/copy-markdown
# -----------------------------------------------------------------------------
chat-cmd-menu-title = Commands
chat-cmd-help = Show keyboard shortcuts
chat-cmd-new = Start a new conversation
chat-cmd-clear = Clear this conversation
chat-cmd-stop = Stop generating
chat-stop = Arrêter
chat-cmd-failed = Command failed
chat-help = Keyboard shortcuts
chat-inspect = Inspect
chat-inspect-title = Message details
chat-inspect-sender = Sender
chat-inspect-id = Message id
chat-inspect-no-id = none
chat-inspect-length = Characters
chat-copy-markdown = Copy as Markdown
chat-copied-markdown = Copied as Markdown

# Model attribution on each bot answer
chat-model-written-by = Réponse écrite par

# Assistant parity in chat
assistant-parity-block-allow = Autoriser
assistant-parity-block-allowed = Autorisé
assistant-parity-block-api-command = Commande API
assistant-parity-block-attachment-read = Pièce jointe lue
assistant-parity-block-chars = caractères
assistant-parity-block-command = Commande
assistant-parity-block-decision-failed = Impossible d'enregistrer la décision
assistant-parity-block-denied = Refusé
assistant-parity-block-deny = Refuser
assistant-parity-block-detail = Détail
assistant-parity-block-drive-list = Listage du Drive
assistant-parity-block-drive-read = Fichier Drive lu
assistant-parity-block-drive-search = Recherche Drive
assistant-parity-block-elapsed = Écoulé
assistant-parity-block-error = Erreur
assistant-parity-block-file = Fichier
assistant-parity-block-kb-search = Recherche dans la base de connaissances
assistant-parity-block-notice = Avis
assistant-parity-block-path = Chemin
assistant-parity-block-permission = Autorisation demandée
assistant-parity-block-query = Requête
assistant-parity-block-read = Lire
assistant-parity-block-reason = Raison
assistant-parity-block-result = Résultat
assistant-parity-block-retry = Réessayer
assistant-parity-block-seconds = s
assistant-parity-block-size = Taille
assistant-parity-block-status-done = Terminé
assistant-parity-block-status-failed = Échoué
assistant-parity-block-status-needs-you = Nécessite votre intervention
assistant-parity-block-status-running = En cours
assistant-parity-block-step = Étape
assistant-parity-block-thinking = Réflexion
assistant-parity-block-tool = Outil
assistant-parity-block-tool-run = Exécution d'outil
assistant-parity-block-total = Total
assistant-parity-block-truncated = tronqué
assistant-parity-block-type = Type
assistant-parity-block-yes = Oui
chat-parity-attach-title = Aperçu de la pièce jointe
chat-parity-attach-unavailable = Contenu indisponible - utilisez Télécharger pour ouvrir ce fichier.
chat-parity-attach-download-failed = Impossible de télécharger la pièce jointe.
chat-parity-model-tiers = Auto et niveaux
chat-parity-model-accounts = Comptes des fournisseurs
chat-parity-model-local = Local et gratuit
chat-drive-parity-attaching = Ajout...
chat-drive-parity-cancel = Annuler
chat-drive-parity-delete = Supprimer
chat-drive-parity-deleted = Supprimé
chat-drive-parity-delete-failed = Impossible de supprimer le fichier.
chat-drive-parity-delete-question = Supprimer ce fichier ?
chat-drive-parity-delete-title = Supprimer le fichier
chat-drive-parity-deleting = Suppression...
chat-drive-parity-download = Télécharger
chat-drive-parity-download-failed = Impossible de télécharger le fichier.
chat-drive-parity-downloading = Téléchargement...
chat-drive-parity-more = Plus d'actions
chat-drive-parity-new-name = Nouveau nom
chat-drive-parity-preview = Aperçu
chat-drive-parity-preview-failed = Impossible d'afficher l'aperçu du fichier.
chat-drive-parity-previewing = Ouverture de l'aperçu...
chat-drive-parity-rename = Renommer
chat-drive-parity-renamed = Renommé
chat-drive-parity-rename-failed = Impossible de renommer le fichier.
chat-drive-parity-rename-title = Renommer le fichier
chat-drive-parity-renaming = Renommage...
chat-drive-parity-unavailable = Cette action n'est pas disponible dans ce déploiement.
chat-drive-parity-use-in-chat = Utiliser dans la conversation
chat-parity-agent-does = L'agent s'en charge
chat-parity-back = Retour
chat-parity-checked-now = Vérifié à l'instant
chat-parity-close = Fermer
chat-parity-coming-soon = Bientôt disponible
chat-parity-cost-contract = Selon votre contrat
chat-parity-cost-free = Gratuit
chat-parity-cost-free-limit = Gratuit, avec limites d'usage
chat-parity-cost-sub = Votre abonnement
chat-parity-cuda-no = Aucun GPU adapté détecté
chat-parity-cuda-yes = Carte graphique détectée
chat-parity-depends-agent = Dépend de l'agent
chat-parity-driver = Pilote
chat-parity-engine = Moteur utilisé
chat-parity-feature-docs = Documents
chat-parity-feature-ocr = Texte dans les images
chat-parity-feature-photos = Photos et graphiques
chat-parity-feature-web = Recherche web
chat-parity-features = Fonctionnalités actives
chat-parity-gains = Vos avantages
chat-parity-go-agent = Va chez le fournisseur de l'agent
chat-parity-go-ext = Va à un fournisseur externe
chat-parity-go-net = Reste dans le réseau de votre organisation
chat-parity-go-server = Va vers le serveur que vous avez défini
chat-parity-guide = Vous ne savez pas lequel choisir ?
chat-parity-guide-sub = Répondez à deux questions.
chat-parity-in-use = En cours d'utilisation
chat-parity-load-failed = Impossible de charger ces informations.
chat-parity-loading = Chargement...
chat-parity-machine = Cet ordinateur
chat-parity-machine-ok = Cet ordinateur dispose de : %d, %v Go
chat-parity-machine-slow = Cet ordinateur n'a pas de carte graphique adaptée : ce serait lent
chat-parity-need-acp = Installez l'agent et connectez-vous à votre compte
chat-parity-need-free = Rien
chat-parity-need-local = Une carte graphique de 6 Go ou plus et un téléchargement de 4 à 6 Go
chat-parity-need-own = L'adresse du serveur et une clé
chat-parity-no-read = Ne lit pas
chat-parity-no-work = Ne fonctionne pas
chat-parity-none-found = Aucun trouvé
chat-parity-not-yet = Pas encore
chat-parity-only-net = Uniquement avec un serveur sur votre réseau
chat-parity-optional = Optionnel, vous l'activez
chat-parity-pick = Choisissez une option
chat-parity-priv-ext = Vos conversations sont envoyées à un hôte externe (%h).
chat-parity-priv-local = Vos conversations restent sur cet ordinateur.
chat-parity-priv-net = Vos conversations sont envoyées au serveur de votre organisation (%h).
chat-parity-privacy = Confidentialité
chat-parity-q1 = Ce que vous écrivez peut-il quitter cet ordinateur ?
chat-parity-q1-help = Pensez aux informations confidentielles ou personnelles.
chat-parity-q1-no = Non, doit rester ici
chat-parity-q1-no-desc = Seul le traitement local est acceptable.
chat-parity-q1-yes = Oui, peut quitter
chat-parity-q1-yes-desc = Un service externe peut la traiter.
chat-parity-q2-no = Non, ou je ne sais pas
chat-parity-q2-no-desc = Pas de serveur ni de clé d'entreprise.
chat-parity-q2-open = Votre organisation propose-t-elle un serveur ou une clé IA ?
chat-parity-q2-open-help = En cas de doute, renseignez-vous auprès de l'IT.
chat-parity-q2-secret = Votre organisation autorise-t-elle l'IA externe ?
chat-parity-q2-secret-help = Certaines politiques interdisent les services externes.
chat-parity-q2-secret-no-desc = L'IA externe n'est pas autorisée.
chat-parity-q2-secret-yes-desc = L'IA externe est autorisée.
chat-parity-q2-yes = Oui
chat-parity-q2-yes-desc = Un serveur ou une clé d'entreprise est disponible.
chat-parity-reads = Lit
chat-parity-ready = Prêt
chat-parity-rec-free = Commencez par le modèle gratuit : aucune configuration. Pour les sujets confidentiels, préférez cet ordinateur.
chat-parity-rec-local = Utilisez cet ordinateur : rien ne sort et la carte graphique est adaptée.
chat-parity-rec-local-slow = Cet ordinateur n'a pas de carte graphique adaptée à un modèle local. Utilisez le serveur de votre organisation sur son réseau.
chat-parity-rec-local-tight = Un modèle local fonctionnera, mais pourrait être lent avec cette carte graphique.
chat-parity-rec-own = Utilisez le serveur ou la clé de votre organisation : le contrat définit le coût et les limites.
chat-parity-rec-own-net = Utilisez le serveur de votre organisation sur son réseau pour garder les données privées.
chat-parity-restart = Recommencer
chat-parity-retry = Réessayer
chat-parity-row-cost = Coût
chat-parity-row-docs = Documents
chat-parity-row-needs = Vous avez besoin de
chat-parity-row-ocr = Texte dans les images
chat-parity-row-offline = Sans internet
chat-parity-row-photos = Photos et graphiques
chat-parity-row-privacy = Vos textes
chat-parity-row-web = Recherche web
chat-parity-rule = Règle
chat-parity-show-me = Voir comment l'activer
chat-parity-stay-here = Reste sur cet ordinateur
chat-parity-step = Question %n sur 2
chat-parity-sub-acp = Votre abonnement d'agent
chat-parity-sub-free = Prêt à l'emploi
chat-parity-sub-local = Privé et hors ligne
chat-parity-sub-own = De votre organisation
chat-parity-suggested = Suggéré
chat-parity-system-memory = Mémoire système
chat-parity-verdict-bad-gpu = Aucune carte graphique adaptée
chat-parity-verdict-bad-gpu-long = Aucune carte NVIDIA avec CUDA n'a été trouvée. Les modèles locaux tourneraient sur le processeur, beaucoup plus lentement : une réponse peut prendre des minutes. Pour garder vos textes dans le réseau, préférez le serveur de votre organisation.
chat-parity-verdict-capacity = Cette carte exécute des modèles nécessitant jusqu'à %v de mémoire vidéo.
chat-parity-verdict-good = Confortable
chat-parity-verdict-low-vram = Carte avec trop peu de mémoire (%v)
chat-parity-verdict-low-vram-long = La carte dispose de %v de mémoire vidéo ; le plus petit modèle local demande %m.
chat-parity-verdict-mid = Juste mais utilisable
chat-parity-verdict-none = Cet ordinateur n'a pas pu être examiné
chat-parity-video-memory = Mémoire vidéo
chat-parity-where = Choisissez le moteur
chat-parity-where-acp = Agent (ACP)
chat-parity-where-free = Modèle gratuit
chat-parity-where-local = Sur cet ordinateur
chat-parity-where-own = Serveur ou clé propre
chat-parity-where-sub = Comparez les quatre façons d'utiliser l'assistant. Cliquez sur l'une pour voir les détails et l'activer.
chat-parity-works = Fonctionne
chat-parity-workshop = Atelier de l'assistant
chat-parity-workshop-lead = Réglez où et comment votre assistant réfléchit.
chat-parity-workshop-title = Atelier de l'assistant
chat-parity-graphics-card = Carte graphique

# --- #2921 missing keys added to every locale catalogue ---
chat-confirm-title = Confirmer l'action
chat-confirm-unavailable = Confirmation indisponible
chat-copy = Copier
chat-delete = Supprimer
chat-delete-message-confirm = Supprimer ce message de la conversation ?
chat-delete-title = Supprimer le message
chat-edit = Modifier
chat-edit-unavailable = Éditeur indisponible
chat-message-actions = Actions du message
chat-regenerate = Régénérer
chat-regenerate-no-source = Aucun message utilisateur précédent à régénérer
chat-regenerate-unavailable = Impossible de régénérer ce tour
chat-voice-capture-silent = Aucun audio capturé
chat-voice-duplex-audio = Audio
chat-voice-duplex-closed = Session vocale fermée
chat-voice-duplex-connect = Connecter la voix
chat-voice-duplex-connected = Voix connectée
chat-voice-duplex-disabled = Voix désactivée
chat-voice-duplex-error = Erreur vocale
chat-voice-duplex-fallback = Retour au texte
chat-voice-duplex-live = Voix en direct
chat-voice-duplex-lost = Connexion perdue
chat-voice-duplex-timeout = La session vocale a expiré
chat-voice-duplex-unavailable = Voix indisponible
chat-voice-mic-failed = Microphone indisponible
chat-web-title = Recherche web
voice-mode-connecting = Connexion...
voice-mode-fallback-idle = Inactif
voice-mode-fallback-listening = Écoute
voice-mode-fallback-speaking = Parole
voice-mode-fallback-thinking = Réflexion
voice-mode-live-listening = Écoute
voice-mode-live-ready = Prêt
voice-mode-live-speaking = Parole

# gbgov assistant parity: busy-send and actionable engine refusals (#2563)
chat-busy-send-head = La réponse est encore en cours de rédaction
chat-busy-send-text = Attendez la fin de la réponse, ou arrêtez-la et envoyez votre message maintenant.
chat-busy-send-wait = Continuer à attendre
chat-busy-send-stop = Arrêter et envoyer
chat-engine-error-not-linked = Votre compte %label n'est pas connecté. Choisissez un autre modèle ou connectez le compte dans les paramètres.
chat-engine-error-switch-off = %label nécessite l'interrupteur Agent activé dans cette conversation.
chat-engine-error-unavailable = Les agents par abonnement ne sont pas disponibles sur cet ordinateur. Choisissez un autre modèle.
chat-engine-error-model-unavailable = Le modèle %model n'est pas disponible sur %label. Choisissez un autre modèle.
chat-engine-error-workspace-failed = La machine virtuelle de l'agent n'a pas pu démarrer. Vérifiez son état et réessayez.
chat-engine-error-account-model-off = Le modèle %model n'est pas activé pour %label. Choisissez un autre modèle.
chat-engine-error-free-unavailable = Le modèle gratuit est indisponible pour le moment. Réessayez plus tard ou choisissez un autre modèle.
chat-engine-error-limit-reached = Vous avez atteint la limite d'utilisation de votre forfait sur %label. %when Choisissez un autre modèle.
chat-engine-error-unsafe-version = Cette version de %label (%version) a été bloquée car elle n'a pas demandé d'autorisation avant d'agir. Choisissez un autre modèle.
chat-engine-error-choose-model = Choisir un autre modèle
chat-engine-error-back-at = Revient %when.
