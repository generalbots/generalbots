# =============================================================================
# General Bots — de — Admin, monitoring and system apps
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Admin Panel (additional UI keys)
# -----------------------------------------------------------------------------
admin-panel-title = Admin-Panel

# =============================================================================
# MONITORING SCREENS
# =============================================================================
# -----------------------------------------------------------------------------
# Monitoring Dashboard
# -----------------------------------------------------------------------------
monitoring-title = Überwachungs-Dashboard
monitoring-toggle-view = Ansicht umschalten
monitoring-last-updated = Zuletzt aktualisiert
monitoring-live-view = Live-Ansicht
monitoring-grid-view = Rasteransicht

# -----------------------------------------------------------------------------
# Monitoring Panels
# -----------------------------------------------------------------------------
monitoring-sessions = Sitzungen
monitoring-messages = Nachrichten
monitoring-resources = Ressourcen
monitoring-services = Dienstleistungen
monitoring-active-bots = Aktive Bots
monitoring-loading = Laden...

# -----------------------------------------------------------------------------
# Service Status
# -----------------------------------------------------------------------------
monitoring-status-running = Laufen
monitoring-status-warning = Warnung
monitoring-status-stopped = Angehalten
monitoring-status-healthy = Gesund
monitoring-status-degraded = Degradiert
monitoring-status-down = Runter

# -----------------------------------------------------------------------------
# Resource Metrics
# -----------------------------------------------------------------------------
monitoring-cpu = CPU
monitoring-memory = Erinnerung
monitoring-disk = Diskette
monitoring-network = Netzwerk
monitoring-requests-per-sec = Anfragen/Sek
monitoring-active-connections = Aktive Verbindungen
monitoring-uptime = Betriebszeit

# -----------------------------------------------------------------------------
# Logs
# -----------------------------------------------------------------------------
monitoring-logs-title = Systemprotokolle
monitoring-logs-filter = Protokolle filtern
monitoring-logs-level = Protokollebene
monitoring-logs-all = Alle Ebenen
monitoring-logs-debug = Debuggen
monitoring-logs-info = Infos
monitoring-logs-warning = Warnung
monitoring-logs-error = Fehler
monitoring-logs-critical = Kritisch
monitoring-logs-search = Protokolle durchsuchen...
monitoring-logs-no-results = Keine Protokolle gefunden

# -----------------------------------------------------------------------------
# Health
# -----------------------------------------------------------------------------
monitoring-health-title = Systemgesundheit
monitoring-health-status = Gesundheitszustand
monitoring-health-services = Servicegesundheit
monitoring-health-database = Datenbank
monitoring-health-cache = Cache
monitoring-health-queue = Nachrichtenwarteschlange
monitoring-health-storage = Lagerung
monitoring-health-external = Externe Dienstleistungen

# -----------------------------------------------------------------------------
# Metrics
# -----------------------------------------------------------------------------
monitoring-metrics-title = Leistungskennzahlen
monitoring-metrics-response-time = Reaktionszeit
monitoring-metrics-throughput = Durchsatz
monitoring-metrics-error-rate = Fehlerrate
monitoring-metrics-latency = Latenz

# -----------------------------------------------------------------------------
# Alerts
# -----------------------------------------------------------------------------
monitoring-alerts-title = Systemwarnungen
monitoring-alerts-active = Aktive Warnungen
monitoring-alerts-resolved = Gelöst
monitoring-alerts-all = Alle Warnungen
monitoring-alert-severity = Schweregrad
monitoring-alert-critical = Kritisch
monitoring-alert-high = Hoch
monitoring-alert-medium = Mittel
monitoring-alert-low = Niedrig
monitoring-alert-info = Infos
monitoring-alert-acknowledge = Bestätigen
monitoring-alert-resolve = Lösen
monitoring-no-alerts = Keine aktiven Warnungen

# -----------------------------------------------------------------------------
# Security Module
# -----------------------------------------------------------------------------
security-tab-compliance = API-Konformitätsbericht
security-tab-protection = Schutz
security-export-report = Bericht exportieren
security-run-scan = Führen Sie den Compliance-Scan aus
security-critical = Kritisch
security-critical-desc = Sofortiges Handeln erforderlich
security-high = Hoch
security-high-desc = Sicherheitsrisiko
security-medium = Mittel
security-medium-desc = Sollte angegangen werden
security-low = Niedrig
security-low-desc = Best Practice
security-info = Infos
security-info-desc = Informativ
security-filter-severity = Schweregrad:
security-filter-all-severities = Alle Schweregrade
security-filter-type = Typ:
security-filter-all-types = Alle Typen
security-type-password = Passwort in der Konfiguration
security-type-hardcoded = Hartcodierte Geheimnisse
security-type-deprecated = Veraltete Schlüsselwörter
security-type-fragile = Fragiler Code
security-type-config = Konfigurationsprobleme
security-results = Compliance-Probleme
security-col-severity = Schweregrad
security-col-issue = Problemtyp
security-col-location = Standort
security-col-details = Beschreibung
security-col-action = Aktion

# -----------------------------------------------------------------------------
# Learn Module
# -----------------------------------------------------------------------------
learn-title = Lernen
learn-my-progress = Mein Fortschritt
learn-completed = Abgeschlossen
learn-in-progress = In Bearbeitung
learn-certificates = Zertifikate
learn-time-spent = Zeitaufwand
learn-categories = Kategorien
learn-all-courses = Alle Kurse
learn-mandatory = Obligatorisch
learn-compliance = Compliance
learn-security = Sicherheit
learn-skills = Fähigkeiten
learn-onboarding = Onboarding
learn-difficulty = Schwierigkeit
learn-my-certificates = Meine Zertifikate
learn-view-all = Alle anzeigen

# -----------------------------------------------------------------------------
# Workspace Module
# -----------------------------------------------------------------------------
workspace-title = Arbeitsbereich
workspace-search-pages = Seiten durchsuchen...
workspace-recent = Neu
workspace-favorites = Favoriten
workspace-pages = Seiten
workspace-templates = Vorlagen
workspace-trash = Müll
workspace-settings = Einstellungen

# -----------------------------------------------------------------------------
# Player Module
# -----------------------------------------------------------------------------
player-title = Mediaplayer
player-no-file = Keine Datei ausgewählt
player-search = Dateien durchsuchen...
player-recent = Neu
player-files = Dateien

# -----------------------------------------------------------------------------
# Goals Module
# -----------------------------------------------------------------------------
goals-title = Ziele und OKRs
goals-dashboard = Armaturenbrett
goals-objectives = Ziele
goals-alignment = Ausrichtung
goals-ai-suggestions = KI-Vorschläge

# semantic i18n key (#2542)
admin-ops-dashboard-title = Betriebs-Dashboard

# semantic i18n key (#2542)
admin-ops-dashboard-subtitle = Echtzeit-Systemmetriken, verteiltes Tracing und Leistungsüberwachung

# semantic i18n key (#2542)
admin-ops-auto-refresh-on = Automatische Aktualisierung: EIN

# semantic i18n key (#2542)
admin-ops-avg-response = Durchschn. Antwortzeit

# semantic i18n key (#2542)
admin-ops-requests-min = Anfragen/min

# semantic i18n key (#2542)
admin-ops-disk-io = Datenträger-E/A

# semantic i18n key (#2542)
admin-ops-iops-read-write = IOPS (Lesen/Schreiben)

# semantic i18n key (#2542)
admin-ops-in-95-out-50 = Eingang: 95 / Ausgang: 50

# semantic i18n key (#2542)
admin-ops-stable = Stabil

# semantic i18n key (#2542)
admin-ops-request-metrics = Anfragemetriken

# semantic i18n key (#2542)
admin-ops-errors-label = Fehler

# semantic i18n key (#2542)
admin-ops-distributed-tracing = Verteiltes Tracing

# semantic i18n key (#2542)
admin-ops-all-traces = Alle Traces

# semantic i18n key (#2542)
admin-ops-errors-only = Nur Fehler

# semantic i18n key (#2542)
admin-ops-api-requests = API-Anfragen

# semantic i18n key (#2542)
admin-ops-post-api-chat-message = POST /api/chat/message

# semantic i18n key (#2542)
admin-ops-trace-a1b2c3d4e5f6 = trace_a1b2c3d4e5f6

# semantic i18n key (#2542)
admin-ops-get-api-kb-search = GET /api/kb/search

# semantic i18n key (#2542)
admin-ops-trace-b2c3d4e5f6g7 = trace_b2c3d4e5f6g7

# semantic i18n key (#2542)
admin-ops-post-api-billing-checkout = POST /api/billing/checkout

# semantic i18n key (#2542)
admin-ops-trace-c3d4e5f6g7h8 = trace_c3d4e5f6g7h8

# semantic i18n key (#2542)
admin-ops-get-api-user-profile = GET /api/user/profile

# semantic i18n key (#2542)
admin-ops-trace-d4e5f6g7h8i9 = trace_d4e5f6g7h8i9

# semantic i18n key (#2542)
admin-ops-api-gateway = API-Gateway

# semantic i18n key (#2542)
admin-ops-primary-2-replicas = Primär + 2 Replikate

# semantic i18n key (#2542)
admin-ops-qdrant-vector-db = Qdrant (Vektor-DB)

# semantic i18n key (#2542)
admin-ops-redis-cache = Redis-Cache

# semantic i18n key (#2542)
admin-ops-cluster-mode-3-shards = Cluster-Modus | 3 Shards

# semantic i18n key (#2542)
admin-ops-llm-service = LLM-Dienst

# semantic i18n key (#2542)
admin-ops-openai-gpt-4-high-latency = OpenAI GPT-4 | Hohe Latenz

# semantic i18n key (#2542)
admin-ops-object-storage = Objektspeicher

# semantic i18n key (#2542)
admin-ops-s3-compatible = S3-kompatibel

# semantic i18n key (#2542)
admin-ops-error-tracking = Fehlerverfolgung

# semantic i18n key (#2542)
admin-ops-view-all-errors = Alle Fehler anzeigen

# semantic i18n key (#2542)
admin-ops-warnings = Warnungen

# semantic i18n key (#2542)
admin-ops-payment-processing-failed-stripe-timeout = Zahlungsverarbeitung fehlgeschlagen: Stripe-API-Zeitüberschreitung

# semantic i18n key (#2542)
admin-ops-billing-checkout-rs-145 = billing/checkout.rs:145

# semantic i18n key (#2542)
admin-ops-rate-limit-approaching-openai = Ratenlimit für die OpenAI-API wird bald erreicht

# semantic i18n key (#2542)
admin-ops-llm-openai-rs-89 = llm/openai.rs:89

# semantic i18n key (#2542)
admin-ops-core-analytics-rs-234 = core/analytics.rs:234

# semantic i18n key (#2542)
admin-ops-cache-miss-rate-above-threshold = Cache-Fehltrefferrate über dem Schwellenwert

# semantic i18n key (#2542)
admin-ops-cache-redis-rs-67 = cache/redis.rs:67

# semantic i18n key (#2542)
admin-ops-endpoint-performance = Endpunktleistung

# semantic i18n key (#2542)
admin-ops-by-requests = Nach Anfragen

# semantic i18n key (#2542)
admin-ops-by-latency = Nach Latenz

# semantic i18n key (#2542)
admin-ops-by-error-rate = Nach Fehlerrate

# semantic i18n key (#2542)
admin-ops-post = POST

# semantic i18n key (#2542)
admin-ops-api-chat-message = /api/chat/message

# semantic i18n key (#2542)
admin-ops-req-min = Anf./Min.

# semantic i18n key (#2542)
admin-ops-errors-metric = Fehler

# semantic i18n key (#2542)
admin-ops-api-kb-search = /api/kb/search
