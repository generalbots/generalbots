# =============================================================================
# General Bots — en — Admin, monitoring and system apps
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Admin Panel (additional UI keys)
# -----------------------------------------------------------------------------
admin-panel-title = Pannello di amministrazione

# =============================================================================
# MONITORING SCREENS
# =============================================================================
# -----------------------------------------------------------------------------
# Monitoring Dashboard
# -----------------------------------------------------------------------------
monitoring-title = Monitoraggio dashboard
monitoring-toggle-view = Commuta vista
monitoring-last-updated = Ultimo aggiornamento
monitoring-live-view = Vista dal vivo
monitoring-grid-view = Vista griglia

# -----------------------------------------------------------------------------
# Monitoring Panels
# -----------------------------------------------------------------------------
monitoring-sessions = Sessioni
monitoring-messages = Messaggi
monitoring-resources = Risorse
monitoring-services = Servizi
monitoring-active-bots = Bot attivi
monitoring-loading = Caricamento in corso...

# -----------------------------------------------------------------------------
# Service Status
# -----------------------------------------------------------------------------
monitoring-status-running = Esecuzione
monitoring-status-warning = Attenzione
monitoring-status-stopped = Fermato
monitoring-status-healthy = Sano
monitoring-status-degraded = Degradato
monitoring-status-down = Giù

# -----------------------------------------------------------------------------
# Resource Metrics
# -----------------------------------------------------------------------------
monitoring-cpu = CPU
monitoring-memory = Memoria
monitoring-disk = Disco
monitoring-network = Rete
monitoring-requests-per-sec = Richieste/sez
monitoring-active-connections = Connessioni attive
monitoring-uptime = Tempo di uptime

# -----------------------------------------------------------------------------
# Logs
# -----------------------------------------------------------------------------
monitoring-logs-title = Log di sistema
monitoring-logs-filter = Log del filtro
monitoring-logs-level = Livello registro
monitoring-logs-all = Tutti i livelli
monitoring-logs-debug = Debug
monitoring-logs-info = Informazioni
monitoring-logs-warning = Attenzione
monitoring-logs-error = Errore
monitoring-logs-critical = Critico
monitoring-logs-search = Diario di ricerca...
monitoring-logs-no-results = Nessun log trovato

# -----------------------------------------------------------------------------
# Health
# -----------------------------------------------------------------------------
monitoring-health-title = Salute del sistema
monitoring-health-status = Stato di salute
monitoring-health-services = Servizio Salute
monitoring-health-database = Banca dati
monitoring-health-cache = Cache
monitoring-health-queue = Coda messaggio
monitoring-health-storage = Conservazione
monitoring-health-external = Servizi esterni

# -----------------------------------------------------------------------------
# Metrics
# -----------------------------------------------------------------------------
monitoring-metrics-title = Metrics prestazioni
monitoring-metrics-response-time = Tempo di risposta
monitoring-metrics-throughput = Flusso di cassa
monitoring-metrics-error-rate = Tasso di errore
monitoring-metrics-latency = Latenza

# -----------------------------------------------------------------------------
# Alerts
# -----------------------------------------------------------------------------
monitoring-alerts-title = Avvisi di sistema
monitoring-alerts-active = Avvisi attivi
monitoring-alerts-resolved = Risolto
monitoring-alerts-all = Tutti gli avvisi
monitoring-alert-severity = Severità
monitoring-alert-critical = Critico
monitoring-alert-high = Alto
monitoring-alert-medium = Media
monitoring-alert-low = Bassa
monitoring-alert-info = Informazioni
monitoring-alert-acknowledge = Conferma
monitoring-alert-resolve = Risolvi
monitoring-no-alerts = Nessuna segnalazione attiva

# -----------------------------------------------------------------------------
# Security Module
# -----------------------------------------------------------------------------
security-tab-compliance = Report di conformità API
security-tab-protection = Protezione
security-export-report = Esporta rapporto
security-run-scan = Esegui scansione della conformità
security-critical = Critico
security-critical-desc = Richiesta di intervento immediato
security-high = Alto
security-high-desc = Rischio di sicurezza
security-medium = Media
security-medium-desc = Dovresti essere trattato
security-low = Bassa
security-low-desc = Migliore pratica
security-info = Informazioni
security-info-desc = Informazioni
security-filter-severity = Severità:
security-filter-all-severities = Tutte le gravità
security-filter-type = Tipo:
security-filter-all-types = Tutti i tipi
security-type-password = Password in Config
security-type-hardcoded = Segreti codificati
security-type-deprecated = Parole chiave deprecate
security-type-fragile = Codice fragile
security-type-config = Problemi di configurazione
security-results = Problemi di conformità
security-col-severity = Severità
security-col-issue = Tipo di emissione
security-col-location = Ubicazione
security-col-details = Descrizione
security-col-action = Azione

# -----------------------------------------------------------------------------
# Learn Module
# -----------------------------------------------------------------------------
learn-title = Imparare
learn-my-progress = Il mio progresso
learn-completed = Completato
learn-in-progress = In corso
learn-certificates = Certificati
learn-time-spent = Tempo trascorso
learn-categories = Categorie
learn-all-courses = Tutti i corsi
learn-mandatory = Obbligatorio
learn-compliance = Conformità
learn-security = Sicurezza
learn-skills = Competenze
learn-onboarding = A bordo
learn-difficulty = Difficoltà
learn-my-certificates = I miei certificati
learn-view-all = Visualizza tutti

# -----------------------------------------------------------------------------
# Workspace Module
# -----------------------------------------------------------------------------
workspace-title = Spazio di lavoro
workspace-search-pages = Pagine di ricerca...
workspace-recent = Recenti
workspace-favorites = Preferiti
workspace-pages = Pagine
workspace-templates = Modelli
workspace-trash = Cestino
workspace-settings = Impostazioni

# -----------------------------------------------------------------------------
# Player Module
# -----------------------------------------------------------------------------
player-title = Riproduttore multimediale
player-no-file = Nessun file selezionato
player-search = Cerca nei file...
player-recent = Recenti
player-files = File

# -----------------------------------------------------------------------------
# Goals Module
# -----------------------------------------------------------------------------
goals-title = Obiettivi e OKR
goals-dashboard = Cruscotto
goals-objectives = Obiettivi
goals-alignment = Allineamento
goals-ai-suggestions = Suggerimenti IA
goals-enter-objective-title = Inserisca il titolo dell'obiettivo:
goals-edit-objective-title = Modifica titolo obiettivo:
goals-delete-objective-confirm = Eliminare questo obiettivo? Questa operazione non può essere annullata.
goals-enter-key-result-title = Inserisca il titolo del risultato chiave:
goals-enter-target-value = Inserisca il valore target:

# semantic i18n key (#2542)
admin-ops-dashboard-title = Cruscotto delle operazioni

# semantic i18n key (#2542)
admin-ops-dashboard-subtitle = metriche di sistema in tempo reale, tracciamento distribuito e monitoraggio delle prestazioni

# semantic i18n key (#2542)
admin-ops-auto-refresh-on = Aggiornamento automatico: ATTIVO

# semantic i18n key (#2542)
admin-ops-avg-response = Risposta media

# semantic i18n key (#2542)
admin-ops-requests-min = Richieste/min

# semantic i18n key (#2542)
admin-ops-disk-io = I/O disco

# semantic i18n key (#2542)
admin-ops-iops-read-write = IOPS (leggere/scrivere)

# semantic i18n key (#2542)
admin-ops-in-95-out-50 = In: 95 / Fuori: 50

# semantic i18n key (#2542)
admin-ops-stable = Stabile

# semantic i18n key (#2542)
admin-ops-request-metrics = Richiesta Metrics

# semantic i18n key (#2542)
admin-ops-errors-label = Errori

# semantic i18n key (#2542)
admin-ops-distributed-tracing = Tracciamento distribuito

# semantic i18n key (#2542)
admin-ops-all-traces = Tutte le tracce

# semantic i18n key (#2542)
admin-ops-errors-only = Solo errori

# semantic i18n key (#2542)
admin-ops-api-requests = Richieste API

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
admin-ops-api-gateway = Gateway API

# semantic i18n key (#2542)
admin-ops-primary-2-replicas = Primario + 2 repliche

# semantic i18n key (#2542)
admin-ops-qdrant-vector-db = Qdrant (Vettore DB)

# semantic i18n key (#2542)
admin-ops-redis-cache = Cache Redis

# semantic i18n key (#2542)
admin-ops-cluster-mode-3-shards = Modalità cluster | 3 shards

# semantic i18n key (#2542)
admin-ops-llm-service = Servizio LLM

# semantic i18n key (#2542)
admin-ops-openai-gpt-4-high-latency = GPT-4 OpenAI | Alta latenza

# semantic i18n key (#2542)
admin-ops-object-storage = Memorizzazione oggetto

# semantic i18n key (#2542)
admin-ops-s3-compatible = S3 Compatibile

# semantic i18n key (#2542)
admin-ops-error-tracking = Tracciamento errori

# semantic i18n key (#2542)
admin-ops-view-all-errors = Visualizza tutti gli errori

# semantic i18n key (#2542)
admin-ops-warnings = Avvertenze

# semantic i18n key (#2542)
admin-ops-payment-processing-failed-stripe-timeout = Elaborazione del pagamento non riuscita: Timeout API Stripe

# semantic i18n key (#2542)
admin-ops-billing-checkout-rs-145 = fatturazione/checkout.rs:145

# semantic i18n key (#2542)
admin-ops-rate-limit-approaching-openai = Limite di tariffa in avvicinamento per le API OpenAI

# semantic i18n key (#2542)
admin-ops-llm-openai-rs-89 = LM/Openai.rs:89

# semantic i18n key (#2542)
admin-ops-core-analytics-rs-234 = core/analytics.rs:234

# semantic i18n key (#2542)
admin-ops-cache-miss-rate-above-threshold = Tasso di miss della cache superiore alla soglia

# semantic i18n key (#2542)
admin-ops-cache-redis-rs-67 = cache/redis.rs:67

# semantic i18n key (#2542)
admin-ops-endpoint-performance = Prestazioni endpoint

# semantic i18n key (#2542)
admin-ops-by-requests = Per richieste

# semantic i18n key (#2542)
admin-ops-by-latency = Per latenza

# semantic i18n key (#2542)
admin-ops-by-error-rate = Per frequenza di errore

# semantic i18n key (#2542)
admin-ops-post = POST

# semantic i18n key (#2542)
admin-ops-api-chat-message = /api/chat/message

# semantic i18n key (#2542)
admin-ops-req-min = req/min

# semantic i18n key (#2542)
admin-ops-errors-metric = errori

# semantic i18n key (#2542)
admin-ops-api-kb-search = /api/kb/search

# semantic i18n key (#2542)
admin-roles-title = Gestione dei ruoli

# semantic i18n key (#2542)
admin-roles-subtitle = Configurare ruoli e assegnare permessi per controllare l'accesso nell'intera suite

# semantic i18n key (#2542)
admin-roles-sidebar-title = Ruoli

# semantic i18n key (#2542)
admin-roles-system-filter = Ruoli di sistema

# semantic i18n key (#2542)
admin-roles-custom-filter = Ruoli personalizzati

# semantic i18n key (#2542)
admin-roles-select-title = Seleziona un ruolo

# semantic i18n key (#2542)
admin-roles-select-hint = Scelga un ruolo dall'elenco per visualizzare e gestire le relative autorizzazioni

# semantic i18n key (#2542)
admin-roles-name-label = Nome ruolo

# semantic i18n key (#2542)
admin-roles-name-badge-placeholder = nome_ruolo

# semantic i18n key (#2542)
admin-roles-description-placeholder = Descrizione del ruolo va qui

# semantic i18n key (#2542)
admin-roles-permissions-label = Permessi

# semantic i18n key (#2542)
admin-roles-assigned-users-tab = Utenti assegnati

# semantic i18n key (#2542)
admin-roles-assigned-groups-tab = Gruppi assegnati

# semantic i18n key (#2542)
admin-roles-permission-assignment = Assegnazione del permesso

# semantic i18n key (#2542)
admin-roles-permission-assignment-help = Usa la lista dual per assegnare i permessi a questo ruolo

# semantic i18n key (#2542)
admin-roles-available-permissions = Autorizzazioni disponibili

# semantic i18n key (#2542)
admin-roles-category-documents = Documenti

# semantic i18n key (#2542)
admin-roles-category-spreadsheets = Fogli di calcolo

# semantic i18n key (#2542)
admin-roles-category-presentations = Presentazioni

# semantic i18n key (#2542)
admin-roles-category-meetings = Riunioni

# semantic i18n key (#2542)
admin-roles-category-ai-bots = IA e bot

# semantic i18n key (#2542)
admin-roles-category-automation = Automazione

# semantic i18n key (#2542)
admin-roles-assigned-permissions = Autorizzazioni assegnate

# semantic i18n key (#2542)
admin-roles-no-permissions-assigned = Nessun permesso assegnato

# semantic i18n key (#2542)
admin-roles-reset-to-default = Reimposta a predefinito

# semantic i18n key (#2542)
admin-roles-users-with-role = Utenti con questo ruolo

# semantic i18n key (#2542)
admin-roles-groups-with-role = Gruppi con questo ruolo

# semantic i18n key (#2542)
admin-roles-change-history = Cronologia dei cambiamenti di ruolo

# semantic i18n key (#2542)
admin-roles-create-new-role = Crea nuovo ruolo

# semantic i18n key (#2542)
admin-roles-name-help = In minuscolo, senza spazi (usare sottolineature)

# semantic i18n key (#2542)
admin-roles-copy-permissions-from = Copi autorizzazioni da

# semantic i18n key (#2542)
admin-roles-start-with-no-permissions = Inizia senza autorizzazioni

# semantic i18n key (#2542)
admin-roles-standard-user = Utente standard

# semantic i18n key (#2542)
admin-roles-power-user = Utente di alimentazione

# semantic i18n key (#2542)
admin-roles-viewer = Visualizzatore

# semantic i18n key (#2542)
admin-roles-guest-user = Utente ospite

# semantic i18n key (#2542)
admin-roles-create-role-btn = Crei un ruolo

# semantic i18n key (#2542)
admin-roles-delete-role-title = Elimina ruolo

# semantic i18n key (#2542)
admin-roles-delete-warning = Questo rimuoverà il ruolo da tutti gli utenti e gruppi. Questa azione non può essere annullata.

# semantic i18n key (#2542)
admin-roles-assign-to-users = Assegni ruolo agli utenti

# semantic i18n key (#2542)
admin-roles-search-users-label = Cerca utenti

# semantic i18n key (#2542)
admin-roles-type-to-search-users = Digita per cercare gli utenti

# semantic i18n key (#2542)
admin-roles-expiration-optional = Scadenza (opzionale)

# semantic i18n key (#2542)
admin-roles-expiration-help = Lascia vuoto per l'assegnazione permanente

# semantic i18n key (#2542)
admin-roles-assign-to-groups = Assegni ruolo ai gruppi

# semantic i18n key (#2542)
admin-roles-search-groups-label = Gruppi di ricerca

# semantic i18n key (#2542)
admin-roles-type-to-search-groups = Tipo di ricerca per gruppi

# semantic i18n key (#2542)
admin-roles-assign-role-btn = Assegni ruolo

# semantic i18n key (#2542)
admin-onboarding-step-welcome = Benvenuto

# semantic i18n key (#2542)
admin-onboarding-step-welcome-desc = Raccontaci di te

# semantic i18n key (#2542)
admin-onboarding-step-org-desc = Crei il tuo spazio di lavoro

# semantic i18n key (#2542)
admin-onboarding-step-plan = Scelga il piano

# semantic i18n key (#2542)
admin-onboarding-step-plan-desc = Seleziona il tuo abbonamento

# semantic i18n key (#2542)
admin-onboarding-step-payment-desc = Aggiungi metodo di pagamento

# semantic i18n key (#2542)
admin-onboarding-step-start = Inizia

# semantic i18n key (#2542)
admin-onboarding-step-start-desc = Crei il tuo primo bot

# semantic i18n key (#2542)
admin-onboarding-view-guide = Visualizza guida

# semantic i18n key (#2542)
admin-onboarding-welcome-title = Benvenuti al generale Bots!

# semantic i18n key (#2542)
admin-onboarding-welcome-subtitle = Ti organizziamo tra pochi minuti, prima dimmi qualcosa di te.

# semantic i18n key (#2542)
admin-onboarding-full-name = Nome completo

# semantic i18n key (#2542)
admin-onboarding-your-role = Il tuo ruolo

# semantic i18n key (#2542)
admin-onboarding-role-placeholder = Seleziona il tuo ruolo...

# semantic i18n key (#2542)
admin-onboarding-role-developer = Sviluppatore / Ingegnere

# semantic i18n key (#2542)
admin-onboarding-role-product-manager = Gestore prodotti

# semantic i18n key (#2542)
admin-onboarding-role-customer-support = Assistenza clienti

# semantic i18n key (#2542)
admin-onboarding-role-executive = Dirigente / Fondatore

# semantic i18n key (#2542)
admin-onboarding-use-case-label = Cosa vuoi costruire?

# semantic i18n key (#2542)
admin-onboarding-use-case-placeholder = Seleziona il caso di utilizzo primario...

# semantic i18n key (#2542)
admin-onboarding-usecase-support-bot = Bot di assistenza clienti

# semantic i18n key (#2542)
admin-onboarding-usecase-sales-assistant = Assistente vendite

# semantic i18n key (#2542)
admin-onboarding-usecase-internal-kb = Base di conoscenza interna

# semantic i18n key (#2542)
admin-onboarding-usecase-lead-generation = Generazione del piombo

# semantic i18n key (#2542)
admin-onboarding-usecase-employee-onboarding = Inserimento dipendenti

# semantic i18n key (#2542)
admin-onboarding-usecase-ecommerce = Assistente E-commerce

# semantic i18n key (#2542)
admin-onboarding-usecase-other = Qualcos'altro.

# semantic i18n key (#2542)
admin-onboarding-hear-about-label = Come hai saputo di noi?

# semantic i18n key (#2542)
admin-onboarding-source-search = Motore di ricerca

# semantic i18n key (#2542)
admin-onboarding-source-social = Social Media

# semantic i18n key (#2542)
admin-onboarding-source-friend = Amico / Collego

# semantic i18n key (#2542)
admin-onboarding-create-org-title = Crei la Sua organizzazione

# semantic i18n key (#2542)
admin-onboarding-create-org-subtitle = La vostra organizzazione è dove potrete gestire bot, membri del team e basi di conoscenza.

# semantic i18n key (#2542)
admin-onboarding-org-name = Nome dell'organizzazione

# semantic i18n key (#2542)
admin-onboarding-org-name-hint = Questo sarà visibile ai membri del tuo team

# semantic i18n key (#2542)
admin-onboarding-org-url = URL dell'organizzazione

# semantic i18n key (#2542)
admin-onboarding-org-url-prefix = app.generalbots.com/

# semantic i18n key (#2542)
admin-onboarding-org-url-hint = Solo lettere minuscole, numeri e trattini

# semantic i18n key (#2542)
admin-onboarding-team-size = Dimensione squadra

# semantic i18n key (#2542)
admin-onboarding-team-size-placeholder = Quante persone useranno questo?

# semantic i18n key (#2542)
admin-onboarding-team-just-me = Solo io.

# semantic i18n key (#2542)
admin-onboarding-industry-placeholder = Seleziona il tuo settore (opzionale)

# semantic i18n key (#2542)
admin-onboarding-industry-technology = Tecnologia

# semantic i18n key (#2542)
admin-onboarding-industry-healthcare = Sanità

# semantic i18n key (#2542)
admin-onboarding-industry-finance = Finanza / Banche

# semantic i18n key (#2542)
admin-onboarding-industry-retail = Commercio al dettaglio / commercio elettronico

# semantic i18n key (#2542)
admin-onboarding-industry-education = Istruzione

# semantic i18n key (#2542)
admin-onboarding-industry-manufacturing = Produzione

# semantic i18n key (#2542)
admin-onboarding-industry-consulting = Consulenza / Servizi

# semantic i18n key (#2542)
admin-onboarding-industry-government = Governo

# semantic i18n key (#2542)
admin-onboarding-industry-nonprofit = Non profit

# semantic i18n key (#2542)
admin-onboarding-choose-plan-title = Scelga il suo piano

# semantic i18n key (#2542)
admin-onboarding-choose-plan-subtitle = Selezionare il piano che meglio si adatta alle vostre esigenze. È possibile aggiornare o declassare in qualsiasi momento.

# semantic i18n key (#2542)
admin-onboarding-plan-free = Libero

# semantic i18n key (#2542)
admin-onboarding-plan-free-desc = Per privati e piccoli progetti

# semantic i18n key (#2542)
admin-onboarding-per-month = /mese

# semantic i18n key (#2542)
admin-onboarding-select-free = Seleziona libero

# semantic i18n key (#2542)
admin-onboarding-most-popular = Più popolare

# semantic i18n key (#2542)
admin-onboarding-plan-pro-desc = Per team in crescita

# semantic i18n key (#2542)
admin-onboarding-select-pro = Seleziona Pro

# semantic i18n key (#2542)
admin-dashboard-organization-title = Cruscotto dell'organizzazione

# semantic i18n key (#2542)
admin-dashboard-organization-subtitle = Panoramica dell'attività, dei membri e della salute del sistema della vostra organizzazione

# semantic i18n key (#2542)
admin-dashboard-team-members = Membri del gruppo

# semantic i18n key (#2542)
admin-dashboard-system-uptime = Uptime di sistema

# semantic i18n key (#2542)
admin-dashboard-api-server = Server API

# semantic i18n key (#2542)
admin-dashboard-operational = Funzionamento

# semantic i18n key (#2542)
admin-dashboard-latency-45ms = Latenza: 45 m

# semantic i18n key (#2542)
admin-dashboard-requests-1-2k-min = Richieste: 1.2K/min

# semantic i18n key (#2542)
admin-dashboard-connections-45-100 = Connessioni: 45/100

# semantic i18n key (#2542)
admin-dashboard-query-time-12ms = Tempo di interrogazione: 12ms

# semantic i18n key (#2542)
admin-dashboard-vector-database = Banca dati vettoriale

# semantic i18n key (#2542)
admin-dashboard-vectors-2-4m = vettori: 2.4M

# semantic i18n key (#2542)
admin-dashboard-search-8ms = Ricerca: 8ms

# semantic i18n key (#2542)
admin-dashboard-tokens-125k-min = Token: 125K/min

# semantic i18n key (#2542)
admin-dashboard-latency-850ms = Latenza: 850m

# semantic i18n key (#2542)
admin-dashboard-high-usage = Alto utilizzo

# semantic i18n key (#2542)
admin-dashboard-used-34-5gb = Usato: 34.5GB

# semantic i18n key (#2542)
admin-dashboard-iops-2-1k = IOPS: 2.1K

# semantic i18n key (#2542)
admin-dashboard-members-filter = Membri

# semantic i18n key (#2542)
admin-dashboard-customer-support-bot = Bot di assistenza clienti

# semantic i18n key (#2542)
admin-dashboard-security-alert = Allarme di sicurezza:

# semantic i18n key (#2542)
admin-dashboard-customer-feedback-survey = Questionario di feedback del cliente

# semantic i18n key (#2542)
admin-dashboard-sales-assistant = Assistente vendite

# semantic i18n key (#2542)
admin-dashboard-load-more-activity = Carica più attività

# semantic i18n key (#2542)
admin-dashboard-john-smith = John Smith

# semantic i18n key (#2542)
admin-dashboard-role-distribution = Distribuzione dei ruoli

# semantic i18n key (#2542)
admin-dashboard-manage-roles = Gestisci ruoli

# semantic i18n key (#2542)
admin-dashboard-bot-status = Stato del Bot

# semantic i18n key (#2542)
admin-dashboard-manage-bots = Gestisci bot

# semantic i18n key (#2542)
admin-dashboard-handles-customer-inquiries = Gestisce le richieste dei clienti

# semantic i18n key (#2542)
admin-dashboard-product-recommendations = Raccomandazioni per il prodotto

# semantic i18n key (#2542)
admin-dashboard-hr-assistant = Assistente HR

# semantic i18n key (#2542)
admin-dashboard-employee-questions = Domande dei dipendenti

# semantic i18n key (#2542)
admin-dashboard-it-helpdesk = IT Helpdesk

# semantic i18n key (#2542)
admin-dashboard-technical-support = Assistenza tecnica

# semantic i18n key (#2542)
admin-dashboard-pending-invitations = Inviti in attesa

# semantic i18n key (#2542)
admin-dashboard-loading-invitations = Caricamento inviti...

# semantic i18n key (#2542)
admin-dashboard-invite-member = Invita membro

# semantic i18n key (#2542)
admin-dashboard-bulk-invite = Invito in blocco

# semantic i18n key (#2542)
admin-dashboard-emails-label = Email (una per riga, separate da virgola o punto e virgola)

# semantic i18n key (#2542)
admin-dashboard-manage-users = Gestisci gli utenti

# semantic i18n key (#2542)
admin-dashboard-roles-permissions = Ruoli e permessi

# semantic i18n key (#2542)
workspace-placeholder-untitled = Senza titolo

# semantic i18n key (#2542)
admin-dashboard-placeholder-invite-email = teammate@example.com
admin-dashboard-placeholder-bulk-emails = a@example.com
b@example.com

# semantic i18n key (#2542)
admin-roles-placeholder-search = Cerca ruoli...
admin-roles-placeholder-filter-available = Filtro disponibile...
admin-roles-placeholder-filter-assigned = Filtro assegnato...
admin-roles-placeholder-name-example = Ad esempio, editor_contenuti
admin-roles-placeholder-display-name-example = Ad esempio, Editor contenuti
admin-roles-placeholder-description = Descriva a cosa serve questo ruolo...
admin-roles-placeholder-search-users = Digita per cercare gli utenti...
admin-roles-placeholder-search-groups = Digita per cercare gruppi...

# semantic i18n key (#2542)
admin-onboarding-placeholder-full-name = John Smith
admin-onboarding-placeholder-org-name = Acme Corporation
admin-onboarding-placeholder-org-slug = acme
admin-onboarding-team-2-5 = 2-5 persone
admin-onboarding-team-6-20 = 6-20 persone
admin-onboarding-team-21-50 = 21-50 persone
admin-onboarding-team-51-plus = 50+ persone
# semantic i18n key (#2542)
admin-ops-configure-alerts = Configura avvisi
admin-ops-trace-search-placeholder = Ricerca trace ID...
admin-ops-traces-slow = Lento (>500m)

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
workspace-state-empty-recent-title = Non si è ancora aperto nulla.
workspace-state-empty-recent-desc = Le pagine che apri vengono qui, cosi' puoi riprendere da dove hai lasciato.
workspace-state-empty-favorites-title = Ancora nessun favorito
workspace-state-empty-favorites-desc = Segna una pagina come preferita per tenerlo in cima alla barra laterale.
workspace-state-empty-pages-title = Nessuna pagina ancora
workspace-state-empty-pages-desc = Le pagine sono le note e i documenti di questo spazio di lavoro. Crea il primo.

# -----------------------------------------------------------------------------
# #2604 — Protection panel (status comes from /api/security/protection)
# -----------------------------------------------------------------------------
protection-title = Protezione
protection-subtitle = Strumenti di sicurezza dell'host segnalati dal server non sono mai stati "abilitati" in codice rigido.
protection-scanner-status = Stato dello scanner
protection-loading = Caricamento stato scanner...
protection-unknown = sconosciuto
protection-days-old = {days} giorni di età
protection-unknown-tool = strumento sconosciuto
protection-active = Attivo
protection-not-scanning = NON SCANNANTE
protection-version = Versione
protection-signature-age = Età della firma
protection-last-scan = Ultima scansione
protection-never = mai
protection-run-scan = Esegui scansione
protection-opt-out = Gli upload NON sono scansionati: le importazioni non scansionate sono consentite.
protection-no-scanner = Non c'e' nessun scanner registrato.
protection-quarantine = Quarantena
protection-tool = Strumento
protection-list = Elenca i file in quarantena
protection-quarantine-empty = Nessun file in quarantena.
protection-restore = Ripristina
protection-action-failed = L'azione è fallita.
protection-load-error = Lo stato dello scanner non è disponibile.
protection-clamav = ClamAVCity name (optional, probably does not need a translation)
protection-lmd = LMD
protection-quarantined-count = {count} in quarantena
