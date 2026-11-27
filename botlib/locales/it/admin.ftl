# =============================================================================
# General Bots - Admin Translations (English)
# =============================================================================
# Administrative interface translations for the GB Admin Panel
# =============================================================================

# -----------------------------------------------------------------------------
# Admin Navigation & Dashboard
# -----------------------------------------------------------------------------
admin-title = Amministrazione
admin-dashboard = Dashboard di amministrazione
admin-overview = Introduzione
admin-welcome = Benvenuti nel Pannello di amministrazione

admin-nav-dashboard = Cruscotto
admin-nav-users = Utilizzatori
admin-nav-bots = Bot
admin-nav-tenants = Inquilini
admin-nav-settings = Impostazioni
admin-nav-logs = Log
admin-nav-analytics = Analisi
admin-nav-security = Sicurezza
admin-nav-integrations = Integrazione
admin-nav-billing = Fatturazione
admin-nav-support = Supporto
admin-nav-groups = Gruppi
admin-nav-dns = DNS
admin-nav-system = Sistema

# -----------------------------------------------------------------------------
# Admin Quick Actions
# -----------------------------------------------------------------------------
admin-quick-actions = Azioni rapide
admin-create-user = Crei un utente
admin-create-group = Crea gruppo
admin-register-dns = Registro DNS
admin-recent-activity = Attività recente
admin-system-health = Salute del sistema

# -----------------------------------------------------------------------------
# User Management
# -----------------------------------------------------------------------------
admin-users-title = Gestione utenti
admin-users-list = Elenco utenti
admin-users-add = Aggiungi utente
admin-users-edit = Modifica utente
admin-users-delete = Elimina utente
admin-users-search = Cerca utenti...
admin-users-filter = Utenti del filtro
admin-users-export = Esporta utenti
admin-users-import = Importa utenti
admin-users-total = Totale utenti
admin-users-active = Utenti attivi
admin-users-inactive = Utenti inattivi
admin-users-suspended = Utenti sospesi
admin-users-pending = Verifica in attesa
admin-users-last-login = Ultimo accesso
admin-users-created = Creato
admin-users-role = Ruolo
admin-users-status = Stato
admin-users-actions = Azioni
admin-users-no-users = Nessun utente trovato
admin-users-confirm-delete = È sicuro di voler eliminare questo utente?
admin-users-deleted = L'utente è stato eliminato con successo
admin-users-saved = L'utente ha salvato con successo
admin-users-invite = Invita utente
admin-users-invite-sent = Invito inviato con successo
admin-users-bulk-actions = Azioni in blocco
admin-users-select-all = Seleziona tutto
admin-users-deselect-all = Deseleziona tutto

# User Details
admin-user-details = Dettagli utente
admin-user-profile = Profilo
admin-user-email = Email
admin-user-name = Nome
admin-user-phone = Telefono
admin-user-avatar = AvatarCity name (optional, probably does not need a translation)
admin-user-timezone = Fuso orario
admin-user-language = Lingua
admin-user-role-admin = Amministratore
admin-user-role-manager = Gestore
admin-user-role-user = Utente
admin-user-role-viewer = Visualizzatore
admin-user-status-active = Attivo
admin-user-status-inactive = Inattivo
admin-user-status-suspended = Sospeso
admin-user-status-pending = In attesa
admin-user-permissions = Permessi
admin-user-activity = Registro attività
admin-user-sessions = Sessioni attive
admin-user-terminate-session = Termina sessione
admin-user-terminate-all = Termina tutte le sessioni
admin-user-reset-password = Reimposta password
admin-user-force-logout = Forza logout
admin-user-enable-2fa = Abilita 2FA
admin-user-disable-2fa = Disabilita 2FA

# -----------------------------------------------------------------------------
# Group Management
# -----------------------------------------------------------------------------
admin-groups-title = Gestione del gruppo
admin-groups-subtitle = Gestire gruppi, membri e autorizzazioni
admin-groups-list = Elenco dei gruppi
admin-groups-add = Aggiungi gruppo
admin-groups-create = Crea gruppo
admin-groups-edit = Modifica gruppo
admin-groups-delete = Elimina gruppo
admin-groups-search = Gruppi di ricerca...
admin-groups-filter = Gruppi di filtri
admin-groups-total = Totale gruppi
admin-groups-active = Gruppi attivi
admin-groups-no-groups = Non sono stati trovati gruppi
admin-groups-confirm-delete = È sicuro di voler eliminare questo gruppo?
admin-groups-deleted = Gruppo eliminato con successo
admin-groups-saved = Gruppo salvato con successo
admin-groups-created = Gruppo creato con successo
admin-groups-loading = Caricamento gruppi...

# Group Details
admin-group-details = Dettagli gruppo
admin-group-name = Nome gruppo
admin-group-description = Descrizione
admin-group-visibility = Visibilità
admin-group-visibility-public = Pubblico
admin-group-visibility-private = Privato
admin-group-visibility-hidden = Nascosto
admin-group-join-policy = Entra nella politica
admin-group-join-invite = Invita solo
admin-group-join-request = Richiesta di adesione
admin-group-join-open = Apri
admin-group-members = Membri
admin-group-member-count = { $count ->
    [one] Membro { $count }
    *[other] Membri { $count }
    }
admin-group-add-member = Aggiungi membro
admin-group-remove-member = Rimuovi membro
admin-group-permissions = Permessi
admin-group-settings = Impostazioni
admin-group-analytics = Analisi
admin-group-overview = Introduzione

# Group View Modes
admin-groups-view-grid = Vista griglia
admin-groups-view-list = Vista elenco
admin-groups-all-visibility = Tutta la visibilità

# -----------------------------------------------------------------------------
# DNS Management
# -----------------------------------------------------------------------------
admin-dns-title = Gestione DNS
admin-dns-subtitle = Registrati e gestisci i nomi host DNS per i tuoi bot
admin-dns-register = Nome host del registro
admin-dns-registered = Nome host registrato
admin-dns-search = Cerca nomi host...
admin-dns-refresh = Aggiorna
admin-dns-loading = Caricamento dei record DNS...
admin-dns-no-records = Non sono stati trovati risultati DNS
admin-dns-confirm-delete = È sicuro di voler rimuovere questo hostname?
admin-dns-deleted = Nome host rimosso con successo
admin-dns-saved = Record DNS salvato correttamente
admin-dns-created = Nome host registrato con successo

# DNS Form Fields
admin-dns-hostname = Nome host
admin-dns-hostname-placeholder = mybot.example.com
admin-dns-hostname-help = Inserisca il nome di dominio completo che desidera registrare
admin-dns-record-type = Tipo di registrazione
admin-dns-record-type-a = A (IPv4)
admin-dns-record-type-aaaa = AAAA (IPv6)
admin-dns-record-type-cname = CNAME
admin-dns-ttl = TTL (secondi)
admin-dns-ttl-5min = 5 minuti (300)
admin-dns-ttl-1hour = 1 ora (3600)
admin-dns-ttl-1day = 1 giorno (86400)
admin-dns-target = Indirizzo target/IP
admin-dns-target-placeholder-ipv4 = 192.168.1.1
admin-dns-target-placeholder-ipv6 = 2001:db8::1
admin-dns-target-placeholder-cname = target.example.com
admin-dns-target-help-a = Inserisca l'indirizzo IPv4 a cui puntare
admin-dns-target-help-aaaa = Inserisca l'indirizzo IPv6 a cui puntare
admin-dns-target-help-cname = Inserisca il nome di dominio di destinazione
admin-dns-auto-ssl = Effettua automaticamente il provisioning del certificato SSL

# DNS Table Headers
admin-dns-col-hostname = Nome host
admin-dns-col-type = Tipo
admin-dns-col-target = Obiettivo
admin-dns-col-ttl = TTL
admin-dns-col-ssl = SSL
admin-dns-col-status = Stato
admin-dns-col-actions = Azioni

# DNS Status
admin-dns-status-active = Attivo
admin-dns-status-pending = In attesa
admin-dns-status-error = Errore
admin-dns-ssl-enabled = Abilitato SSL
admin-dns-ssl-disabled = Nessun SSL
admin-dns-ssl-pending = SSL in attesa

# DNS Info Cards
admin-dns-help-title = Guida alla configurazione DNS
admin-dns-help-a-record = Record A
admin-dns-help-a-record-desc = Mappe di un nome di dominio a un indirizzo IPv4. Usa questo per puntare il tuo nome host direttamente a un IP del server.
admin-dns-help-aaaa-record = Record AAAA
admin-dns-help-aaaa-record-desc = Mappe di un nome di dominio a un indirizzo IPv6. Simile a un record ma per la connettività IPv6.
admin-dns-help-cname-record = Record CNAME
admin-dns-help-cname-record-desc = Crea un alias da un dominio a un altro. Utile per indirizzare i sottodomini al dominio principale.
admin-dns-help-ssl = SSL/TLS
admin-dns-help-ssl-desc = Effettua automaticamente il provisioning dei certificati Let's Encrypt per connessioni HTTPS sicure.

# DNS Edit/Remove Modals
admin-dns-edit-title = Modifica record DNS
admin-dns-remove-title = Rimuovi nome host
admin-dns-remove-warning = Questo elimina il record DNS e tutti i certificati SSL associati. L'hostname non risolverà più.

# -----------------------------------------------------------------------------
# Bot Management
# -----------------------------------------------------------------------------
admin-bots-title = Gestione del Bot
admin-bots-list = Elenco dei Bot
admin-bots-add = Aggiungi bot
admin-bots-edit = Modifica Bot
admin-bots-delete = Elimina Bot
admin-bots-search = Cerca bot...
admin-bots-filter = Bots filtro
admin-bots-total = Totale bot
admin-bots-active = Bot attivi
admin-bots-inactive = Bots inattivi
admin-bots-draft = Bot in bozza
admin-bots-published = Bots pubblicati
admin-bots-no-bots = Nessun bot trovato
admin-bots-confirm-delete = È sicuro di voler eliminare questo bot?
admin-bots-deleted = Bot eliminato con successo
admin-bots-saved = Bot salvato con successo
admin-bots-duplicate = Duplica Bot
admin-bots-export = Esporta bot
admin-bots-import = Importa bot
admin-bots-publish = Pubblica
admin-bots-unpublish = Non pubblicare
admin-bots-test = Prova di Bot
admin-bots-logs = Registri del Bot
admin-bots-analytics = Analisi Bot
admin-bots-conversations = Conversazioni
admin-bots-templates = Modelli
admin-bots-dialogs = Finestre di dialogo
admin-bots-knowledge-base = Base di conoscenza

# Bot Details
admin-bot-details = Dettagli del Bot
admin-bot-name = Nome del Bot
admin-bot-description = Descrizione
admin-bot-avatar = Avatar del Bot
admin-bot-language = Lingua
admin-bot-timezone = Fuso orario
admin-bot-greeting = Messaggio di saluto
admin-bot-fallback = Messaggio di fallback
admin-bot-channels = Canali
admin-bot-channel-web = Web Chat
admin-bot-channel-whatsapp = WhatsApp
admin-bot-channel-telegram = Telegramma
admin-bot-channel-slack = Slack
admin-bot-channel-teams = Microsoft Team
admin-bot-channel-email = Email
admin-bot-model = Modello IA
admin-bot-temperature = Temperatura
admin-bot-max-tokens = Max Tokens
admin-bot-system-prompt = Prompt di sistema

# -----------------------------------------------------------------------------
# Tenant Management
# -----------------------------------------------------------------------------
admin-tenants-title = Gestione degli inquilini
admin-tenants-list = Elenco degli inquilini
admin-tenants-add = Aggiungi tenant
admin-tenants-edit = Modifica tenant
admin-tenants-delete = Elimina tenant
admin-tenants-search = Cerca inquilini...
admin-tenants-total = Totale inquilini
admin-tenants-active = Tenant attivi
admin-tenants-suspended = Sospeso inquilini
admin-tenants-trial = Inquilini di prova
admin-tenants-no-tenants = Non sono stati trovati inquilini.
admin-tenants-confirm-delete = È sicuro di voler eliminare questo tenant?
admin-tenants-deleted = Tenant eliminato con successo
admin-tenants-saved = Tenant salvato con successo

# Tenant Details
admin-tenant-details = Dettagli dell'affittuario
admin-tenant-name = Nome dell'affittuario
admin-tenant-domain = Dominio
admin-tenant-plan = Piano
admin-tenant-plan-free = Libero
admin-tenant-plan-starter = Avviatore
admin-tenant-plan-professional = Professionista
admin-tenant-plan-enterprise = Impresa
admin-tenant-users = Utilizzatori
admin-tenant-bots = Bot
admin-tenant-storage = Conservazione usata
admin-tenant-api-calls = Chiamate API
admin-tenant-limits = Limiti di utilizzo
admin-tenant-billing = Informazioni di fatturazione

# -----------------------------------------------------------------------------
# System Settings
# -----------------------------------------------------------------------------
admin-settings-title = Impostazioni di sistema
admin-settings-general = Impostazioni generali
admin-settings-security = Impostazioni di sicurezza
admin-settings-email = Impostazioni email
admin-settings-storage = Impostazioni di archiviazione
admin-settings-integrations = Integrazione
admin-settings-api = Impostazioni API
admin-settings-appearance = Aspetto
admin-settings-localization = Localizzazione
admin-settings-notifications = Notifiche
admin-settings-backup = Backup e ripristino
admin-settings-maintenance = Modalità manutenzione
admin-settings-saved = Impostazioni salvate con successo
admin-settings-reset = Reimposta a predefiniti
admin-settings-confirm-reset = È sicuro di voler reimpostare tutte le impostazioni ai valori predefiniti?

# General Settings
admin-settings-site-name = Nome del sito
admin-settings-site-url = URL del sito
admin-settings-admin-email = Email dell'amministratore
admin-settings-support-email = Email di supporto
admin-settings-default-language = Lingua predefinita
admin-settings-default-timezone = Fuso orario predefinito
admin-settings-date-format = Formato data
admin-settings-time-format = Formato orario
admin-settings-currency = Valuta

# Email Settings
admin-settings-smtp-host = Host SMTP
admin-settings-smtp-port = Porta SMTP
admin-settings-smtp-user = Nome utente SMTP
admin-settings-smtp-password = Password SMTP
admin-settings-smtp-encryption = Crittografia
admin-settings-smtp-from-name = Dal nome
admin-settings-smtp-from-email = Dalla posta elettronica
admin-settings-smtp-test = Invia email di prova
admin-settings-smtp-test-success = Email di prova inviata con successo
admin-settings-smtp-test-failed = Impossibile inviare l'email di prova

# Storage Settings
admin-settings-storage-provider = Fornitore di storage
admin-settings-storage-local = Deposito locale
admin-settings-storage-s3 = Amazon S3City name (optional, probably does not need a translation)
admin-settings-storage-minio = MinIO
admin-settings-storage-gcs = Google Cloud Storage
admin-settings-storage-azure = Archiviazione Azure Blob
admin-settings-storage-bucket = Nome bucket
admin-settings-storage-region = Regione
admin-settings-storage-access-key = Chiave di accesso
admin-settings-storage-secret-key = Chiave segreta
admin-settings-storage-endpoint = URL endpoint

# -----------------------------------------------------------------------------
# System Logs
# -----------------------------------------------------------------------------
admin-logs-title = Log di sistema
admin-logs-search = Diario di ricerca...
admin-logs-filter-level = Filtra per livello
admin-logs-filter-source = Filtra per sorgente
admin-logs-filter-date = Filtra per data
admin-logs-level-all = Tutti i livelli
admin-logs-level-debug = Debug
admin-logs-level-info = Informazioni
admin-logs-level-warning = Attenzione
admin-logs-level-error = Errore
admin-logs-level-critical = Critico
admin-logs-export = Esporta log
admin-logs-clear = Cancellare log
admin-logs-confirm-clear = È sicuro di voler cancellare tutti i log?
admin-logs-cleared = Log cancellati con successo
admin-logs-no-logs = Nessun log trovato
admin-logs-refresh = Aggiorna
admin-logs-auto-refresh = Aggiornamento automatico
admin-logs-timestamp = Orario
admin-logs-level = Livello
admin-logs-source = Fonte
admin-logs-message = Messaggio
admin-logs-details = Dettagli

# -----------------------------------------------------------------------------
# Analytics
# -----------------------------------------------------------------------------
admin-analytics-title = Analisi
admin-analytics-overview = Introduzione
admin-analytics-users = Analisi utente
admin-analytics-bots = Analisi Bot
admin-analytics-conversations = Analisi delle conversazioni
admin-analytics-performance = Prestazioni
admin-analytics-period = Periodo
admin-analytics-period-today = Oggi
admin-analytics-period-week = Questa settimana
admin-analytics-period-month = Questo mese
admin-analytics-period-quarter = Questo quarto
admin-analytics-period-year = Quest'anno
admin-analytics-period-custom = Intervallo personalizzato
admin-analytics-export = Esporta rapporto
admin-analytics-total-users = Totale utenti
admin-analytics-new-users = Nuovi utenti
admin-analytics-active-users = Utenti attivi
admin-analytics-total-bots = Totale bot
admin-analytics-active-bots = Bot attivi
admin-analytics-total-conversations = Conversazioni totali
admin-analytics-avg-response-time = Tempo medio di risposta
admin-analytics-satisfaction-rate = Tasso di soddisfazione
admin-analytics-resolution-rate = Tasso di risoluzione

# -----------------------------------------------------------------------------
# Security
# -----------------------------------------------------------------------------
admin-security-title = Sicurezza
admin-security-overview = Panoramica sulla sicurezza
admin-security-audit-log = Registro di controllo
admin-security-login-attempts = Tentativi di login
admin-security-blocked-ips = IP bloccati
admin-security-api-keys = Chiavi API
admin-security-webhooks = Agganci web
admin-security-cors = Impostazioni CORS
admin-security-rate-limiting = Limitazioni delle aliquote
admin-security-encryption = Crittografia
admin-security-2fa = Autenticazione a due fattori
admin-security-sso = Firma singola
admin-security-password-policy = Politica della password

# API Keys
admin-api-keys-title = Chiavi API
admin-api-keys-add = Crea chiave API
admin-api-keys-name = Nome chiave
admin-api-keys-key = Chiave API
admin-api-keys-secret = Chiave segreta
admin-api-keys-created = Creato
admin-api-keys-last-used = Ultima utilizzazione
admin-api-keys-expires = Scade
admin-api-keys-never = Mai
admin-api-keys-revoke = Revoke
admin-api-keys-confirm-revoke = È sicuro di voler revocare questa chiave API?
admin-api-keys-revoked = Chiave API revocata con successo
admin-api-keys-created-success = Chiave API creata con successo
admin-api-keys-copy = Copi negli Appunti
admin-api-keys-copied = Copiato!
admin-api-keys-warning = Assicurati di copiare la tua chiave API ora. Non sarà in grado di vederlo di nuovo!

# -----------------------------------------------------------------------------
# Billing
# -----------------------------------------------------------------------------
admin-billing-title = Fatturazione
admin-billing-overview = Panoramica fatturazione
admin-billing-current-plan = Piano corrente
admin-billing-usage = Uso
admin-billing-invoices = Fatture
admin-billing-payment-methods = Metodi di pagamento
admin-billing-upgrade = Piano di aggiornamento
admin-billing-downgrade = Esegui downgrade del piano
admin-billing-cancel = Annulla abbonamento
admin-billing-invoice-date = Data fattura
admin-billing-invoice-amount = Importo
admin-billing-invoice-status = Stato
admin-billing-invoice-paid = Pagato
admin-billing-invoice-pending = In attesa
admin-billing-invoice-overdue = Scaduto
admin-billing-invoice-download = Scarica fattura

# -----------------------------------------------------------------------------
# Backup & Restore
# -----------------------------------------------------------------------------
admin-backup-title = Backup e ripristino
admin-backup-create = Crea backup
admin-backup-restore = Ripristina backup
admin-backup-schedule = Programmare i backup
admin-backup-list = Cronologia backup
admin-backup-name = Nome backup
admin-backup-size = Dimensione
admin-backup-created = Creato
admin-backup-download = Scarica
admin-backup-delete = Elimina
admin-backup-confirm-restore = È sicuro di voler ripristinare questo backup? Questa operazione sovrascriverà i dati correnti.
admin-backup-confirm-delete = È sicuro di voler eliminare questo backup?
admin-backup-in-progress = Backup in corso...
admin-backup-completed = Backup completato con successo
admin-backup-failed = Backup non riuscito
admin-backup-restore-in-progress = Ripristina in corso...
admin-backup-restore-completed = Ripristina completato con successo
admin-backup-restore-failed = Ripristinare non riuscito

# -----------------------------------------------------------------------------
# Maintenance Mode
# -----------------------------------------------------------------------------
admin-maintenance-title = Modalità manutenzione
admin-maintenance-enable = Abilita modalità manutenzione
admin-maintenance-disable = Disabilita modalità manutenzione
admin-maintenance-status = Stato corrente
admin-maintenance-active = La modalità di manutenzione è attiva
admin-maintenance-inactive = La modalità di manutenzione è inattiva
admin-maintenance-message = Messaggio di manutenzione
admin-maintenance-default-message = Attualmente stiamo eseguendo la manutenzione programmata. Si prega di controllare indietro presto.
admin-maintenance-allowed-ips = Indirizzi IP consentiti
admin-maintenance-confirm-enable = È sicuro di voler abilitare la modalità manutenzione? Gli utenti non potranno accedere al sistema.

# -----------------------------------------------------------------------------
# Common Admin UI Elements
# -----------------------------------------------------------------------------
admin-required = Richiesto
admin-optional = Facoltativo
admin-loading = Caricamento in corso...
admin-saving = Salvataggio...
admin-deleting = Eliminazione in corso...
admin-confirm = Confermare
admin-cancel = Annulla
admin-save = Salva
admin-create = Crea
admin-update = Aggiorna
admin-delete = Elimina
admin-edit = Modifica
admin-view = Vista
admin-close = Chiudi
admin-back = Indietro
admin-next = Avanti
admin-previous = Precedente
admin-refresh = Aggiorna
admin-export = Esporta
admin-import = Importa
admin-search = Cerca
admin-filter = Filtro
admin-clear = Cancellare
admin-select = Seleziona
admin-select-all = Seleziona tutto
admin-deselect-all = Deseleziona tutto
admin-actions = Azioni
admin-more-actions = Altre azioni
admin-no-data = Nessun dato disponibile
admin-error = Si è verificato un errore
admin-success = Successo
admin-warning = Attenzione
admin-info = Informazioni

# Table Pagination
admin-showing = Visualizzazione di { $from } a { $to } di risultati { $total }
admin-page = Pagina { $current } di { $total }
admin-items-per-page = Elementi per pagina
admin-go-to-page = Va alla pagina

# Bulk Actions
admin-bulk-delete = Elimina selezionati
admin-bulk-export = Esporta selezionati
admin-bulk-activate = Attiva selezionati
admin-bulk-deactivate = Disattivi selezionati
admin-selected-count = { $count ->
    [one] Elemento { $count } selezionato
    *[other] Elementi { $count } selezionati
    }


# ── Settings navigation (issue #1304) ──────────────────────────────
# The settings sidebar referenced these ids directly; without them the menu
# rendered the raw key instead of the label ("[id] instead of real text").
admin-audit = Registro di controllo
admin-audit-subtitle = Traccia di attività del sistema
admin-users = Utilizzatori
admin-users-subtitle = Persone con accesso a questo spazio di lavoro
admin-groups = Gruppi
admin-bots = Bot
admin-bots-subtitle = Gestisci i bot assistente
admin-dns = DNS


# ── #2307 Users & Groups dialogs / status regions ──────────────────
admin-add-user = Aggiungi utente
admin-add-group = Aggiungi gruppo
admin-search-users = Cerca utenti...
admin-search-groups = Gruppi di ricerca...
admin-confirm-title = Conferma azione
admin-confirm-accept = Confermare
admin-col-user = Utente
admin-col-email = Email
admin-col-organization = Organizzazione
admin-col-roles = Ruoli
admin-col-status = Stato
admin-col-actions = Azioni
admin-status-active = Attivo
admin-status-inactive = Inattivo
admin-action-edit = Modifica
admin-action-delete = Elimina
admin-view-members = Visualizza membri
admin-members-suffix = membri
admin-creating = Creazione in corso…
admin-user-created = L'utente creato e aggiunto all'organizzazione
admin-group-created = Gruppo creato con successo
admin-user-deleted = Utente eliminato
admin-group-deleted = Gruppo soppresso
admin-member-removed = Membro rimosso
admin-confirm-delete-user = Eliminare questo utente?
admin-confirm-delete-group = Eliminare questo gruppo?
admin-confirm-remove-member = Rimuovi membro?
admin-failed-load-users = Impossibile caricare gli utenti.
admin-failed-load-groups = Impossibile caricare i gruppi.
admin-failed-create-user = Impossibile creare l'utente.
admin-failed-create-group = Impossibile creare il gruppo.
admin-no-users = Nessun utente trovato
admin-no-groups = Non sono stati trovati gruppi

# semantic i18n key (#2542)
admin-org-switch-title = Organizzazione di commutazione

# semantic i18n key (#2542)
admin-org-create-new-title = Crea nuova organizzazione

# semantic i18n key (#2542)
admin-org-name-label = Nome dell'organizzazione

# semantic i18n key (#2542)
admin-org-url-slug-label = Slug URL

# semantic i18n key (#2542)
admin-org-slug-preview = organizzazione-slug

# semantic i18n key (#2542)
admin-org-plan-label = Piano

# semantic i18n key (#2542)
admin-org-plan-free = Libero

# semantic i18n key (#2542)
admin-org-plan-starter = Avviatore

# semantic i18n key (#2542)
admin-org-create-btn = Crei un'organizzazione

# semantic i18n key (#2542)
admin-filter-all-roles = Tutti i ruoli

# semantic i18n key (#2542)
admin-user-role-moderator = Moderatore

# semantic i18n key (#2542)
admin-filter-all-status = Tutti gli stati

# semantic i18n key (#2542)
admin-users-last-active = Ultima attività

# semantic i18n key (#2542)
admin-users-loading = Caricamento utenti...

# semantic i18n key (#2542)
admin-users-create-new = Crei un nuovo utente

# semantic i18n key (#2542)
admin-users-send-welcome-email = Invia email di benvenuto

# semantic i18n key (#2542)
admin-users-require-password-change = Richiede il cambio di password al primo login

# semantic i18n key (#2542)
admin-users-delete-warning = Questa azione non può essere annullata. Tutti i dati dell'utente saranno rimossi definitivamente.

# semantic i18n key (#2542)
accounts-subtitle = Gestisci le integrazioni dei tuoi social media e dei canali di messaggistica

# semantic i18n key (#2542)
accounts-channel-facebook = Facebook

# semantic i18n key (#2542)
accounts-channel-twitter-x = Twitter / X

# semantic i18n key (#2542)
accounts-channel-bluesky = BlueskyCity name (optional, probably does not need a translation)

# semantic i18n key (#2542)
accounts-channel-threads = Filettatura

# semantic i18n key (#2542)
accounts-channel-tiktok = TikTok

# semantic i18n key (#2542)
accounts-channel-youtube = YouTube

# semantic i18n key (#2542)
accounts-channel-pinterest = Pinterest

# semantic i18n key (#2542)
accounts-channel-reddit = Reddit

# semantic i18n key (#2542)
accounts-channel-snapchat = Snapchat

# semantic i18n key (#2542)
accounts-messaging = Messaggi

# semantic i18n key (#2542)
accounts-channel-whatsapp-business = WhatsApp Business

# semantic i18n key (#2542)
accounts-channel-twilio-sms = SMS Twilio

# semantic i18n key (#2542)
accounts-channel-wechat = WeChat

# semantic i18n key (#2542)
accounts-channel-gmail-google-workspace = Gmail / Google Workspace

# semantic i18n key (#2542)
accounts-channel-outlook-office-365 = Prospettive / Ufficio 365

# semantic i18n key (#2542)
accounts-channel-custom-smtp = SMTP personalizzato

# semantic i18n key (#2542)
accounts-redirecting-authorization = Riindirizzamento all'autorizzazione...

# semantic i18n key (#2542)
accounts-configure-smtp-server = Configura server SMTP

# semantic i18n key (#2542)
accounts-save-configuration = Salva configurazione

# semantic i18n key (#2542)
accounts-account-details = Dettagli account

# semantic i18n key (#2542)
accounts-connection-status = Stato connessione

# semantic i18n key (#2542)
accounts-sample-username = @username

# semantic i18n key (#2542)
accounts-sample-connected-since = 15 gennaio 2025

# semantic i18n key (#2542)
accounts-permissions = Permessi

# semantic i18n key (#2542)
admin-groups-hidden = Nascosto

# semantic i18n key (#2542)
admin-groups-detail-title = Dettagli gruppo

# semantic i18n key (#2542)
admin-groups-tab-overview = Introduzione

# semantic i18n key (#2542)
admin-groups-tab-members = Membri

# semantic i18n key (#2542)
admin-groups-tab-permissions = Permessi

# semantic i18n key (#2542)
admin-groups-tab-kbs = Base di conoscenza

# semantic i18n key (#2542)
admin-groups-create-title = Crea nuovo gruppo

# semantic i18n key (#2542)
admin-groups-visibility = Visibilità

# semantic i18n key (#2542)
admin-groups-join-policy = Entra nella politica

# semantic i18n key (#2542)
admin-groups-invite-only = Invita solo

# semantic i18n key (#2542)
admin-groups-request-join = Richiesta di adesione

# semantic i18n key (#2542)
admin-groups-allow-member-invite = Consenta ai membri di invitare altri

# semantic i18n key (#2542)
admin-groups-enable-chat = Abilita chat di gruppo

# semantic i18n key (#2542)
admin-groups-add-members = Aggiungi membri

# semantic i18n key (#2542)
admin-groups-search-users = Cerca utenti

# semantic i18n key (#2542)
admin-groups-selected-users = Utenti selezionati

# semantic i18n key (#2542)
admin-groups-no-users-selected = Nessun utente selezionato

# semantic i18n key (#2542)
admin-groups-role-member = Stato

# semantic i18n key (#2542)
admin-groups-role-moderator = Moderatore

# semantic i18n key (#2542)
admin-groups-send-invites = Invia inviti

# semantic i18n key (#2542)
admin-groups-email-addresses = Indirizzi email

# semantic i18n key (#2542)
admin-groups-emails-help = Inserisca un indirizzo email per riga

# semantic i18n key (#2542)
admin-groups-personal-message-optional = Messaggio personale (facoltativo)

# semantic i18n key (#2542)
admin-groups-delete-warning = Questo rimuoverà tutti i membri e cancellerà tutti i dati di gruppo. Questa azione non può essere annullata.

# semantic i18n key (#2542)
admin-billing-dashboard-title = Dashboard di fatturazione

# semantic i18n key (#2542)
admin-billing-dashboard-subtitle = Analisi complete e informazioni dettagliate per la fatturazione e l'utilizzo dell'organizzazione

# semantic i18n key (#2542)
admin-billing-dashboard-current-period = Periodo corrente

# semantic i18n key (#2542)
admin-billing-dashboard-year-to-date = Anno fino a data

# semantic i18n key (#2542)
admin-billing-dashboard-current-spending = Spesa corrente

# semantic i18n key (#2542)
admin-billing-dashboard-quota-utilization = Utilizzazione dei contingenti

# semantic i18n key (#2542)
admin-billing-dashboard-average-quotas = Media tra tutte le quote

# semantic i18n key (#2542)
admin-billing-dashboard-api-calls = Chiamate API

# semantic i18n key (#2542)
admin-billing-dashboard-storage-used = Conservazione usata

# semantic i18n key (#2542)
admin-billing-dashboard-spending-trends = Tendenze di spesa

# semantic i18n key (#2542)
admin-billing-dashboard-daily = Giornaliero

# semantic i18n key (#2542)
admin-billing-dashboard-weekly = Settimanale

# semantic i18n key (#2542)
admin-billing-dashboard-cost-breakdown = Dettaglio costi per servizio

# semantic i18n key (#2542)
admin-billing-dashboard-quota-management = Gestione delle quote

# semantic i18n key (#2542)
admin-billing-dashboard-team-members = Membri del gruppo

# semantic i18n key (#2542)
admin-billing-dashboard-no-alerts = Nessun avviso configurato

# semantic i18n key (#2542)
admin-billing-dashboard-usage-by-bot = Uso da Bot

# semantic i18n key (#2542)
admin-billing-dashboard-bot-customer-support = Bot di assistenza clienti

# semantic i18n key (#2542)
admin-billing-dashboard-bot-cs-01 = bot_cs_01

# semantic i18n key (#2542)
admin-billing-dashboard-bot-sales-assistant = Assistente vendite

# semantic i18n key (#2542)
admin-billing-dashboard-bot-sa-01 = bot_sa_01

# semantic i18n key (#2542)
admin-billing-dashboard-bot-hr-assistant = Assistente HR

# semantic i18n key (#2542)
admin-billing-dashboard-bot-hr-01 = bot_hr_01

# semantic i18n key (#2542)
admin-billing-dashboard-bot-it-helpdesk = IT Helpdesk

# semantic i18n key (#2542)
admin-billing-dashboard-bot-it-01 = bot_it_01

# semantic i18n key (#2542)
admin-billing-dashboard-recent-invoices = Fatture recenti

# semantic i18n key (#2542)
admin-billing-dashboard-alerts = Avvisi di fatturazione

# semantic i18n key (#2542)
admin-billing-dashboard-invoice-generated-january = Fattura generata per gennaio

# semantic i18n key (#2542)
admin-billing-dashboard-quota-settings = Impostazioni quote

# semantic i18n key (#2542)
admin-billing-dashboard-invoices-limit = Limite delle fatture

# semantic i18n key (#2542)
admin-billing-dashboard-team-members-limit = Limite dei membri della squadra

# semantic i18n key (#2542)
admin-billing-dashboard-bots-limit = Limite dei bot

# semantic i18n key (#2542)
admin-billing-dashboard-alert-thresholds = Soglie di avviso di fatturazione

# semantic i18n key (#2542)
admin-billing-subscription-title = Fatturazione e abbonamento

# semantic i18n key (#2542)
admin-billing-manage-subscription-desc = Gestisci l'abbonamento, i metodi di pagamento e la cronologia delle fatture della tua organizzazione

# semantic i18n key (#2542)
admin-billing-per-month = /mese

# semantic i18n key (#2542)
admin-billing-next-billing-date = Data di fatturazione successiva:

# semantic i18n key (#2542)
admin-billing-next-billing-date-value = 21 febbraio 2025

# semantic i18n key (#2542)
admin-billing-usage-this-period = Uso di questo periodo

# semantic i18n key (#2542)
admin-billing-usage-period-value = 21 Gen - 21 Feb, 2025

# semantic i18n key (#2542)
admin-billing-payment-method = Metodo di pagamento

# semantic i18n key (#2542)
admin-billing-card-expiry-value = Scade 12/2026

# semantic i18n key (#2542)
admin-billing-billing-address = Indirizzo di fatturazione

# semantic i18n key (#2542)
admin-billing-address-suite = Suite 456

# semantic i18n key (#2542)
admin-billing-address-city = San Francisco, CA 94102

# semantic i18n key (#2542)
admin-billing-address-country = Stati Uniti

# semantic i18n key (#2542)
admin-billing-upgrade-your-plan = Aggiorna il tuo piano

# semantic i18n key (#2542)
admin-billing-recommended = Raccomandato

# semantic i18n key (#2542)
admin-billing-priority-support = Sostegno prioritario

# semantic i18n key (#2542)
admin-billing-upgrade-to-business = Aggiornamento alle imprese

# semantic i18n key (#2542)
admin-billing-enterprise = Impresa

# semantic i18n key (#2542)
admin-billing-unlimited-everything = Illimitati tutto

# semantic i18n key (#2542)
admin-billing-dedicated-support = supporto dedicato

# semantic i18n key (#2542)
admin-billing-custom-integrations = Integrazioni personalizzate

# semantic i18n key (#2542)
admin-billing-sla-guarantee = Garanzia SLA

# semantic i18n key (#2542)
admin-billing-contact-sales = Contatti Vendite

# semantic i18n key (#2542)
admin-billing-prorated-notice = La tua prossima fattura sarà prorata in base al periodo di fatturazione rimanente.

# semantic i18n key (#2542)
admin-billing-sorry-to-see-you-go = Ci dispiace che tu te ne vada!

# semantic i18n key (#2542)
admin-billing-cancel-consider-options = Prima di annullare, desidera valutare una di queste opzioni?

# semantic i18n key (#2542)
admin-billing-stay-reduced-rate = Resta con noi ad un tasso ridotto

# semantic i18n key (#2542)
admin-billing-accept-offer = Accetta offerta

# semantic i18n key (#2542)
admin-billing-pause-your-subscription = Metti in pausa l'abbonamento

# semantic i18n key (#2542)
admin-billing-pause-up-to-3-months = Prendere una pausa per un massimo di 3 mesi

# semantic i18n key (#2542)
admin-billing-pause-instead = Pausa invece

# semantic i18n key (#2542)
admin-billing-downgrade-smaller-plan = Esegui il downgrade a un piano più piccolo

# semantic i18n key (#2542)
admin-billing-keep-essential-features = Mantenere caratteristiche essenziali a un costo inferiore

# semantic i18n key (#2542)
admin-billing-view-plans = Visualizza Piani

# semantic i18n key (#2542)
admin-billing-why-cancelling = Perche' stai cancellando?

# semantic i18n key (#2542)
admin-billing-select-reason = Seleziona un motivo...

# semantic i18n key (#2542)
admin-billing-reason-too-expensive = Troppo costoso

# semantic i18n key (#2542)
admin-billing-reason-missing-features = Caratteristiche mancanti di cui ho bisogno

# semantic i18n key (#2542)
admin-billing-reason-not-using = Non lo uso abbastanza

# semantic i18n key (#2542)
admin-billing-reason-switching = Passaggio ad un altro prodotto

# semantic i18n key (#2542)
admin-billing-reason-temporary = Temporaneo - tornerà

# semantic i18n key (#2542)
admin-billing-additional-feedback = Feedback aggiuntivo (opzionale)

# semantic i18n key (#2542)
admin-billing-keep-subscription = Mantieni l'abbonamento

# semantic i18n key (#2542)
admin-groups-placeholder-name = Team di ingegneria
admin-groups-placeholder-description = Descriva lo scopo di questo gruppo...
admin-groups-placeholder-search-members = Cerca per nome o email...
admin-groups-placeholder-invite-emails = Inserisca gli indirizzi email, uno per riga...
admin-groups-placeholder-invite-message = Aggiungi un messaggio personale all'invito...

# semantic i18n key (#2542)
admin-org-placeholder-search = Organizzazioni di ricerca...

# semantic i18n key (#2542)
accounts-placeholder-smtp-host = smtp.example.com
accounts-placeholder-smtp-username = your@email.com
accounts-placeholder-smtp-password = ••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••
accounts-placeholder-smtp-from-address = noreply@example.com
accounts-placeholder-smtp-from-name = La tua azienda

# semantic i18n key (#2542)
admin-placeholder-username = nome utente
admin-placeholder-email = user@example.com
admin-placeholder-display-name = John Doe
admin-placeholder-password = ••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••
admin-placeholder-group-name = Team di ingegneria
admin-placeholder-group-description = Descrizione del gruppo...

# semantic i18n key (#2542)
admin-org-placeholder-name = Inserisca il nome dell'organizzazione
admin-org-placeholder-slug = organizzazione-slug
admin-billing-placeholder-feedback = Aiutaci a migliorare...
admin-billing-dashboard-placeholder-invoices-limit = 100
admin-billing-dashboard-placeholder-team-limit = 50
admin-billing-dashboard-placeholder-bots-limit = 10
admin-billing-dashboard-warning-threshold = Soglia di avvertimento (%)
admin-billing-dashboard-critical-threshold = Soglia critica (%)
admin-billing-dashboard-exceeded-threshold = Soglia superata (%)
admin-billing-dashboard-placeholder-warning = 80
admin-billing-dashboard-placeholder-critical = 90
admin-billing-dashboard-placeholder-exceeded = 100
# semantic i18n key (#2542)
admin-billing-dashboard-toggle-breakdown-view = Commuta la vista di ripartizione
