# =============================================================================
# General Bots — fr — Attendant (support queue)
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

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
