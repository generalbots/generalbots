# =============================================================================
# General Bots — de — Attendant (support queue)
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
attendant-crm-disabled = CRM-Funktionen nicht aktiviert
attendant-crm-disabled-desc = Für die Attendant Console müssen CRM-Funktionen für diesen Bot aktiviert sein. Dadurch können menschliche Agenten vom Bot übertragene Gespräche empfangen und darauf reagieren.
attendant-crm-enable-instruction = Um CRM-Funktionen zu aktivieren, fügen Sie diese Zeile zu Ihren Bots hinzu
attendant-crm-config-file = config.csv
attendant-crm-create-attendant = Dann erstellen Sie eine
attendant-crm-attendant-file = attendant.csv
attendant-crm-configure-team = Datei zum Konfigurieren Ihres Teams

# -----------------------------------------------------------------------------
# Queue Sidebar
# -----------------------------------------------------------------------------
attendant-title = Vermittlungskonsole
attendant-status-online = Online
attendant-status-busy = Beschäftigt
attendant-status-away = Weg
attendant-status-offline = Offline
attendant-status-ready = Online – Bereit für Gespräche
attendant-status-busy-msg = Beschäftigt – Gespräche führen
attendant-status-away-msg = Weg – Wir kommen bald wieder
attendant-status-offline-msg = Offline – Nicht verfügbar

# -----------------------------------------------------------------------------
# Queue Stats
# -----------------------------------------------------------------------------
attendant-waiting = Warten
attendant-active = Aktiv
attendant-resolved = Gelöst
attendant-mine = Meins

# -----------------------------------------------------------------------------
# Queue Filters
# -----------------------------------------------------------------------------
attendant-filter-all = Alle
attendant-filter-waiting = Warten
attendant-filter-mine = Meins
attendant-filter-priority = Priorität

# -----------------------------------------------------------------------------
# Conversation List
# -----------------------------------------------------------------------------
attendant-no-conversations = Keine Gespräche in der Warteschlange
attendant-new-conversations-appear = Neue Konversationen werden hier angezeigt
attendant-unread = Ungelesen
attendant-typing = tippen...
attendant-select-conversation = Wählen Sie eine Konversation aus
attendant-select-conversation-desc = Wählen Sie eine Konversation aus der Warteschlange aus, um mit der Beantwortung zu beginnen

# -----------------------------------------------------------------------------
# Channel Tags
# -----------------------------------------------------------------------------
attendant-channel-whatsapp = WhatsApp
attendant-channel-teams = Mannschaften
attendant-channel-instagram = Instagram
attendant-channel-web = Web
attendant-channel-telegram = Telegramm
attendant-channel-email = E-Mail

# -----------------------------------------------------------------------------
# Priority Tags
# -----------------------------------------------------------------------------
attendant-priority-urgent = Dringend
attendant-priority-high = Hoch
attendant-priority-normal = Normal

# -----------------------------------------------------------------------------
# Chat Area
# -----------------------------------------------------------------------------
attendant-message-placeholder = Geben Sie Ihre Nachricht ein...
attendant-send = Senden
attendant-attach-file = Datei anhängen
attendant-insert-emoji = Emoji einfügen
attendant-quick-responses = Schnelle Antworten
attendant-transfer = Übertragen
attendant-resolve = Lösen
attendant-more-actions = Weitere Aktionen

# -----------------------------------------------------------------------------
# Quick Responses
# -----------------------------------------------------------------------------
attendant-quick-greeting = Hallo! Wie kann ich Ihnen heute helfen?
attendant-quick-thanks = Vielen Dank für Ihre Geduld.
attendant-quick-checking = Lassen Sie mich das für Sie überprüfen.
attendant-quick-moment = Einen Moment bitte.

# -----------------------------------------------------------------------------
# Transfer Modal
# -----------------------------------------------------------------------------
attendant-transfer-title = Gespräch übertragen
attendant-transfer-to = Übertragen an
attendant-transfer-reason = Grund (optional)
attendant-transfer-reason-placeholder = Warum übertragen Sie dieses Gespräch?
attendant-transfer-cancel = Abbrechen
attendant-transfer-confirm = Übertragen

# -----------------------------------------------------------------------------
# AI Insights Sidebar
# -----------------------------------------------------------------------------
attendant-ai-insights = KI-Einblicke
attendant-ai-summary = Zusammenfassung des Gesprächs
attendant-ai-sentiment = Kundenstimmung
attendant-sentiment-positive = Positiv
attendant-sentiment-neutral = Neutral
attendant-sentiment-negative = Negativ
attendant-smart-replies = Intelligente Antworten
attendant-confidence = Vertrauen
attendant-source = Quelle

# -----------------------------------------------------------------------------
# Customer Details
# -----------------------------------------------------------------------------
attendant-customer-details = Kundendaten
attendant-customer-name = Name
attendant-customer-email = E-Mail
attendant-customer-phone = Telefon
attendant-customer-location = Standort
attendant-customer-tags = Schlagworte

# -----------------------------------------------------------------------------
# Conversation History
# -----------------------------------------------------------------------------
attendant-history = Geschichte
attendant-history-resolved = Gelöst
attendant-history-transferred = Übertragen
attendant-history-abandoned = Verlassen
attendant-view-history = Vollständigen Verlauf anzeigen

# -----------------------------------------------------------------------------
# Toast Messages
# -----------------------------------------------------------------------------
attendant-toast-transferred = Konversation erfolgreich übertragen
attendant-toast-resolved = Konversation als gelöst markiert
attendant-toast-assigned = Ihnen zugewiesenes Gespräch
attendant-toast-error = Es ist ein Fehler aufgetreten
attendant-toast-connection-lost = Verbindung verloren. Verbindung wird wieder hergestellt...
attendant-toast-connection-restored = Verbindung wiederhergestellt
