# =============================================================================
# General Bots — en — Attendant (support queue)
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
attendant-crm-disabled = Funzionalità CRM non abilitate
attendant-crm-disabled-desc = La Console partecipante richiede l'attivazione delle funzioni CRM per questo bot. Ciò consente agli agenti umani di ricevere e rispondere alle conversazioni trasferite dal bot.
attendant-crm-enable-instruction = Per abilitare le funzionalità CRM, aggiungi questa linea al tuo bot
attendant-crm-config-file = config.csv
attendant-crm-create-attendant = Poi crea un
attendant-crm-attendant-file = assistant.csv
attendant-crm-configure-team = file per configurare il team

# -----------------------------------------------------------------------------
# Queue Sidebar
# -----------------------------------------------------------------------------
attendant-title = Console dell'addetto
attendant-status-online = Online
attendant-status-busy = Occupato
attendant-status-away = Assente
attendant-status-offline = Fuori rete
attendant-status-ready = Online - Pronti per le conversazioni
attendant-status-busy-msg = Occupato - Gestione conversazioni
attendant-status-away-msg = Assente - Tornerò presto
attendant-status-offline-msg = Non in linea - Non disponibile

# -----------------------------------------------------------------------------
# Queue Stats
# -----------------------------------------------------------------------------
attendant-waiting = In attesa
attendant-active = Attivo
attendant-resolved = Risolto
attendant-mine = Mio

# -----------------------------------------------------------------------------
# Queue Filters
# -----------------------------------------------------------------------------
attendant-filter-all = Tutto
attendant-filter-waiting = In attesa
attendant-filter-mine = Mio
attendant-filter-priority = Priorità

# -----------------------------------------------------------------------------
# Conversation List
# -----------------------------------------------------------------------------
attendant-no-conversations = Nessuna conversazione in coda
attendant-new-conversations-appear = Nuove conversazioni appariranno qui
attendant-unread = Non letto
attendant-typing = Digitando...
attendant-select-conversation = Seleziona una conversazione
attendant-select-conversation-desc = Scelga una conversazione dalla coda per iniziare a rispondere

# -----------------------------------------------------------------------------
# Channel Tags
# -----------------------------------------------------------------------------
attendant-channel-whatsapp = WhatsApp
attendant-channel-teams = Squadre
attendant-channel-instagram = Instagram
attendant-channel-web = Web
attendant-channel-telegram = Telegramma
attendant-channel-email = Email

# -----------------------------------------------------------------------------
# Priority Tags
# -----------------------------------------------------------------------------
attendant-priority-urgent = Urgente
attendant-priority-high = Alto
attendant-priority-normal = Normale

# -----------------------------------------------------------------------------
# Chat Area
# -----------------------------------------------------------------------------
attendant-message-placeholder = Digita il tuo messaggio...
attendant-send = Invia
attendant-attach-file = Allega file
attendant-insert-emoji = Inserisci emoji
attendant-quick-responses = Risposte rapide
attendant-transfer = Trasferimento
attendant-resolve = Risolvi
attendant-more-actions = Altre azioni

# -----------------------------------------------------------------------------
# Quick Responses
# -----------------------------------------------------------------------------
attendant-quick-greeting = Salve, come posso aiutarla oggi?
attendant-quick-thanks = Grazie per la pazienza.
attendant-quick-checking = Lascia che ti controlli.
attendant-quick-moment = Un momento, prego.

# -----------------------------------------------------------------------------
# Transfer Modal
# -----------------------------------------------------------------------------
attendant-transfer-title = Conversazione di trasferimento
attendant-transfer-to = Trasferimento a
attendant-transfer-reason = Motivo (facoltativo)
attendant-transfer-reason-placeholder = Perche' stai trasferendo questa conversazione?
attendant-transfer-cancel = Annulla
attendant-transfer-confirm = Trasferimento

# -----------------------------------------------------------------------------
# AI Insights Sidebar
# -----------------------------------------------------------------------------
attendant-ai-insights = Analisi IA
attendant-ai-summary = Riepilogo conversazione
attendant-ai-sentiment = Sentimento del cliente
attendant-sentiment-positive = Positivo
attendant-sentiment-neutral = Neutrale
attendant-sentiment-negative = Negativo
attendant-smart-replies = Risposte intelligenti
attendant-confidence = Confidenza
attendant-source = Fonte

# -----------------------------------------------------------------------------
# Customer Details
# -----------------------------------------------------------------------------
attendant-customer-details = Dettagli cliente
attendant-customer-name = Nome
attendant-customer-email = Email
attendant-customer-phone = Telefono
attendant-customer-location = Ubicazione
attendant-customer-tags = Etichette

# -----------------------------------------------------------------------------
# Conversation History
# -----------------------------------------------------------------------------
attendant-history = Cronologia
attendant-history-resolved = Risolto
attendant-history-transferred = Trasferito
attendant-history-abandoned = Abbandonato
attendant-view-history = Visualizza cronologia completa

# -----------------------------------------------------------------------------
# Toast Messages
# -----------------------------------------------------------------------------
attendant-toast-transferred = Conversazione trasferita con successo
attendant-toast-resolved = Conversazione contrassegnata come risolta
attendant-toast-assigned = Conversazione assegnata a lei
attendant-toast-error = Si è verificato un errore
attendant-toast-connection-lost = Connessione persa. Riconnessione in corso...
attendant-toast-connection-restored = Connessione ripristinata

# semantic i18n key (#2542)
handoff-title = Trasferimento chatbot e analisi

# semantic i18n key (#2542)
handoff-tab-queue = Coda di eliminazione

# semantic i18n key (#2542)
handoff-tab-agents = Agenti

# semantic i18n key (#2542)
handoff-tab-channels = Canali

# semantic i18n key (#2542)
handoff-tab-csat = CSATCity name (optional, probably does not need a translation)

# semantic i18n key (#2542)
handoff-tab-transcripts = Trascrizione

# semantic i18n key (#2542)
handoff-tab-sla = Monitor SLA

# semantic i18n key (#2542)
handoff-transfer-title = Trasferire conversazione

# semantic i18n key (#2542)
handoff-transfer-reason-skill = Mancanza di competenza

# semantic i18n key (#2542)
handoff-transfer-reason-workload = Bilanciamento carico di lavoro

# semantic i18n key (#2542)
handoff-transfer-reason-escalation = Escalation del cliente

# semantic i18n key (#2542)
handoff-transfer-reason-vip = Trattamento VIP

# semantic i18n key (#2542)
handoff-placeholder-search-conversations = Cerca conversazioni...
handoff-placeholder-search-transcripts = Cerca trascrizioni per parola chiave o agente...
handoff-placeholder-transfer-note = Contesto facoltativo per l'agente ricevente

# semantic i18n key (#2542)
attendant-placeholder-transfer-reason = Ad esempio, problema tecnico, esigenze specialistiche
