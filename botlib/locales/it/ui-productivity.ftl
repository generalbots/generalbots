# =============================================================================
# General Bots — en — Productivity apps (tasks, calendar, meet, mail, research, paper)
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Tasks
# -----------------------------------------------------------------------------
tasks-title = Compiti
tasks-new = Nuova attività
tasks-due-today = Scadenza oggi
tasks-overdue = Scaduto
tasks-completed = Completato
tasks-all = Tutte le attività
tasks-priority-high = Alta priorità
tasks-priority-medium = Media priorità
tasks-priority-low = Bassa priorità
tasks-no-due-date = Nessuna data di scadenza
tasks-add-subtask = Aggiungi sottotask
tasks-mark-complete = Contrassegna come completo
tasks-mark-incomplete = Contrassegna come incompleta
tasks-delete-confirm = È sicuro di voler eliminare questa attività?
tasks-count = { $count ->
    [zero] Nessuna attività
    [one] Attività { $count }
    *[other] Attività { $count }
    }

# -----------------------------------------------------------------------------
# Calendar
# -----------------------------------------------------------------------------
calendar-title = Calendario
calendar-today = Oggi
calendar-new-event = Nuovo evento
calendar-all-day = Tutto il giorno
calendar-repeat = Ripeti
calendar-reminder = Promemoria
calendar-view-day = Giorno
calendar-view-week = Settimana
calendar-view-month = Mese
calendar-view-year = Anno
calendar-no-events = Nessun evento programmato
calendar-event-title = Titolo evento
calendar-event-location = Ubicazione
calendar-event-description = Descrizione
calendar-event-attendees = Partecipanti

# -----------------------------------------------------------------------------
# Meet / Video Conferencing
# -----------------------------------------------------------------------------
meet-title = Incontra
meet-join = Iscriviti alla riunione
meet-start = Inizia riunione
meet-mute = Muto
meet-unmute = Non muto
meet-video-on = Fotocamera accesa
meet-video-off = Fotocamera spenta
meet-share-screen = Condividi schermo
meet-stop-sharing = Smettila di condividere
meet-end-call = Termina chiamata
meet-leave = Lascia riunione
meet-participants = { $count ->
    [one] Partecipante { $count }
    *[other] Partecipanti { $count }
    }
meet-waiting-room = Sala d'attesa
meet-admit = Ammetti
meet-remove = Rimuovi
meet-chat = Incontro chat
meet-raise-hand = Alza la mano
meet-lower-hand = Mano bassa
meet-recording = Registrazione
meet-start-recording = Inizia la registrazione
meet-stop-recording = Ferma registrazione

# -----------------------------------------------------------------------------
# Mail / Email
# -----------------------------------------------------------------------------
mail-title = Posta
mail-compose = Componi
mail-inbox = Posta in arrivo
mail-sent = Inviato
mail-drafts = Bozze
mail-trash = Cestino
mail-spam = Spam
mail-starred = Stellato
mail-archive = Archivio
mail-to = A
mail-cc = CC
mail-bcc = BCC
mail-subject = Oggetto
mail-body = Messaggio
mail-reply = Rispondi
mail-reply-all = Rispondi a tutti
mail-forward = Avanti
mail-send = Invia
mail-discard = Scarta
mail-save-draft = Salva bozza
mail-attach = Allega file
mail-unread = { $count ->
    [one] { $count } non letto
    *[other] { $count } non letto
    }
mail-empty-inbox = La tua casella di posta è vuota
mail-no-subject = (Senza oggetto)

# -----------------------------------------------------------------------------
# Paper (Document Editor)
# -----------------------------------------------------------------------------
paper-title = Carta
paper-new-note = Nuova nota
paper-search-notes = Cerca note...
paper-quick-start = Avvio rapido
paper-template-blank = Vuoto
paper-template-meeting = Riunione
paper-template-todo = Da fare
paper-template-research = Ricerca
paper-untitled = Senza titolo
paper-placeholder = Inizia a scrivere, o digita / per i comandi...
paper-commands = Comandi
paper-heading1 = Rubrica 1
paper-heading1-desc = Numero della sezione
paper-heading2 = Rubrica 2
paper-heading2-desc = Voce della sezione media
paper-heading3 = Rubrica 3
paper-heading3-desc = Denominazione della sezione piccola
paper-paragraph = Paragrafo
paper-paragraph-desc = Testo semplice
paper-bullet-list = Elenco puntato
paper-bullet-list-desc = Elenco non ordinato
paper-numbered-list = Elenco numerato
paper-numbered-list-desc = Elenco ordinato
paper-todo-list = Elenco da fare
paper-todo-list-desc = Elenco attività verificabile
paper-quote = Quotazione
paper-quote-desc = Blocco citazione per le citazioni
paper-divider = Separatore
paper-divider-desc = Linea orizzontale
paper-code-block = Blocco di codice
paper-code-block-desc = Codice formattato
paper-table = Tabella
paper-table-desc = Inserisci tabella
paper-image = Immagine
paper-image-desc = Inserire l'immagine dall'URL
paper-callout = Riquadro di richiamo
paper-callout-desc = Casella d'informazione evidenziata
paper-ai-write = Scrivere con IA
paper-ai-write-desc = Genera testo con AI
paper-ai-summarize = Riassumere con IA
paper-ai-summarize-desc = Riepilogo del testo selezionato
paper-ai-expand = Espandere con IA
paper-ai-expand-desc = Espandi il testo selezionato
paper-ai-improve = Migliorare con IA
paper-ai-improve-desc = Migliorare la qualità della scrittura
paper-ai-translate = Tradurre con IA
paper-ai-translate-desc = Tradurre in un'altra lingua
paper-ai-assistant = Assistente IA
paper-ai-quick-actions = Azioni rapide
paper-ai-rewrite = Riscrivi
paper-ai-make-shorter = Rendi più breve
paper-ai-make-longer = Rendi più lungo
paper-ai-fix-grammar = Risolvi la grammatica
paper-ai-tone = Tono
paper-ai-tone-professional = Professionista
paper-ai-tone-casual = Informale
paper-ai-tone-friendly = Amico
paper-ai-tone-formal = Formale
paper-ai-translate-to = Traduci a
paper-ai-custom-prompt = Prompt personalizzato
paper-ai-custom-placeholder = Descriva cosa vuole...
paper-ai-generate = Genera
paper-ai-response = Risposta IA
paper-ai-apply = Applica
paper-ai-regenerate = Rigenerare
paper-ai-copy = Copi
paper-word-count = Parole { $count }
paper-char-count = caratteri { $count }
paper-saved = Salvato
paper-saving = Salvataggio...
paper-last-edited = Ultima modifica: { $time }
paper-last-edited-now = Ultima modifica: Appena ora
paper-export = Esporta documento
paper-export-pdf = PDF
paper-export-docx = Parola (.docx)
paper-export-markdown = Contrassegna
paper-export-html = HTML
paper-export-txt = Testo semplice

# Tasks translations
tasks-active = Intenti attivi
tasks-awaiting = In attesa di decisione
tasks-paused = Pausa
tasks-blocked = Bloccati/Problemi
tasks-time-saved = Tempo attivo risparmiato:
tasks-input-placeholder = Cosa vorresti fare? Ad esempio, 'creare un'app CRM' o 'ricordami di chiamare John domani'

# Calendar additional translations
calendar-my-calendars = I miei calendari

# Email additional translations
email-scheduled = Programmato
email-tracking = Tracciamento

# Email folder translations
email-inbox = Posta in arrivo
email-starred = Stellato
email-sent = Inviato
email-drafts = Bozze
email-spam = Spam
email-trash = Cestino
email-compose = Componi

# -----------------------------------------------------------------------------
# Research
# -----------------------------------------------------------------------------
research-title = Ricerca
research-search-placeholder = Chieda qualsiasi cosa...
research-collections = Raccolte
research-new-collection = Nuova collezione
research-recent = Recenti
research-academic = Accademico
research-code = Codice
research-internal = Interno
research-search-all = Cerca tutto
research-academic-papers = Articoli accademici
research-code-docs = Codice e documentazione
research-internal-kb = Base di conoscenze interne
research-sources = Fonti
research-trending = Tendenza
research-pro-search = Ricerca pro
research-include-images = Includi immagini
research-try-asking = Prova a chiedere di
research-related = Domande correlate
research-view-all-sources = Visualizza tutte le fonti
research-export-citations = Esporta citazioni
research-save-to-collection = Salva alla raccolta

# -----------------------------------------------------------------------------
# Meet (additional keys)
# -----------------------------------------------------------------------------
meet-new-meeting = Nuova riunione
meet-join-meeting = Iscriviti alla riunione
meet-active-rooms = Stanze attive
meet-room-title = Sala riunioni
meet-record = Registrazione
meet-camera = Fotocamera
meet-share = Condividi
meet-info = Informazioni
meet-more = Di più
meet-share-meeting = Riunione condivisa
meet-meeting-title = Titolo riunione
meet-meeting-code = Codice riunione
meet-meeting-link = Collegamento riunione
meet-send-invite = Invia invito

# ── #2307 Tasks goal/monitor detail panel ──────────────────────────
tasks-detail-untitled-goal = Obiettivo senza titolo
tasks-detail-goal-progress = Progressi verso l'obiettivo
tasks-detail-current-value = Valore corrente
tasks-detail-target-value = Valore obiettivo
tasks-detail-percent = Progressi
tasks-detail-no-actions = Nessuna azione ancora
tasks-detail-pending-info = Informazioni mancanti
tasks-detail-scheduler-info = Informazioni sullo schema
tasks-detail-monitor-info = Informazioni di monitoraggio
tasks-detail-unavailable = Non disponibile
tasks-detail-load-failed = Impossibile caricare i dettagli dell'obiettivo


# ── #2307 Database insert/confirm dialogs ──────────────────────────
db-insert-row = Inserisci riga
db-insert-into = Inserisci
db-provide-value = Fornire almeno un valore
db-insert-failed = Inserisci non riuscito
db-confirm-title = Confermare
db-confirm-accept = Confermare
db-confirm-delete = Eliminare questa riga?
db-confirm-drop = Elimina tabella
db-delete-failed = Eliminazione non riuscita
db-drop-failed = Rilascio non riuscito

# Tasks detail view (#2307)
tasks-detail-type-goal = Obiettivo
tasks-detail-status-pending = In attesa
tasks-detail-pending-title = Voci in sospeso
tasks-detail-required-reason = È necessario specificare un motivo.
tasks-detail-type-scheduler = Programmatore
tasks-detail-type-monitor = Monitoraggio
tasks-detail-last-action = Ultima azione
tasks-detail-next-run = Prossima esecuzione
tasks-detail-file = File
tasks-detail-interval = Intervallo
tasks-detail-last-check = Ultimo controllo
tasks-detail-last-value = Ultimo valore
tasks-detail-progress = Progressi
tasks-detail-app = App
tasks-detail-field = Campo
tasks-detail-reason = Motivazione
tasks-detail-cron = CronCity name (optional, probably does not need a translation)

# Database dialog copy (#2307)
db-error = Errore
db-database = Banca dati
db-empty-query = Eseguire una query per visualizzare i risultati
db-insert = Inserisci
db-select-table = Seleziona una tabella
tasks-load-error = Non è stato possibile caricare le attività.
tasks-sign-in-to-load = Accedi per caricare le attività.
calendar-collapse-sidebar = Comprima barra laterale
calendar-prev-month = Mese precedente
calendar-next-month = Il mese prossimo
calendar-prev-period = Periodo precedente
calendar-next-period = Periodo successivo
calendar-add-calendar = Aggiungi calendario
paper-toggle-sidebar = Commuta barra laterale
paper-insert-ai-response = Inserisci la risposta AI
paper-replace-selection = Sostituisci selezione
paper-highlight = Evidenzia
paper-align-left = Allinea a sinistra
paper-align-center = Allinea al centro
paper-align-right = Allinea a destra
paper-insert-link = Inserisci collegamento
paper-insert-image = Inserisci immagine
paper-insert-table = Inserisci tabella
paper-bold = Grassetto
paper-italic = Corsivo
paper-underline = Sottolineato
paper-strikethrough = Strikethrough
products-view-grid = Vista griglia
products-view-list = Vista elenco
sheet-freeze-panes = Riquadri di congelamento

# semantic i18n key (#2542)
db-schema-title = 🗄️ Database Schema

# semantic i18n key (#2542)
db-loading-tables = Caricamento tabelle...

# semantic i18n key (#2542)
db-query-builder = Costruttore di interrogazioni

# semantic i18n key (#2542)
db-run-query = Esegui interrogazione

# semantic i18n key (#2542)
db-select-table-hint = Seleziona una tabella per visualizzare i dati

# semantic i18n key (#2542)
db-prev-page = ◀ Prev

# semantic i18n key (#2542)
db-page-info-default = Pagina 1 di 1

# semantic i18n key (#2542)
db-next-page = Avanti '

# semantic i18n key (#2542)
research-searching-sources = Sorgenti di ricerca...

# semantic i18n key (#2542)
research-suggestion-latest-ai = Ultimi sviluppi dell'IA

# semantic i18n key (#2542)
research-suggestion-quantum = Quantum computing spiegato

# semantic i18n key (#2542)
research-suggestion-software-architecture = Architettura software

# semantic i18n key (#2542)
research-suggestion-ai-productivity = Strumenti di produttività IA

# semantic i18n key (#2542)
research-related-heading = correlati

# semantic i18n key (#2542)
research-loading = Caricamento Ricerca...

# semantic i18n key (#2542)
research-service-unavailable = Servizio di ricerca non disponibile

# semantic i18n key (#2542)
calendar-mini-month-placeholder = Gennaio 2025

# semantic i18n key (#2542)
calendar-upcoming = Prossima

# semantic i18n key (#2542)
calendar-import-export = Importa / Esporta

# semantic i18n key (#2542)
calendar-popup-event-title = Titolo evento

# semantic i18n key (#2542)
calendar-popup-location-placeholder = Sala riunioni A

# semantic i18n key (#2542)
calendar-popup-event-description = Descrizione evento

# semantic i18n key (#2542)
calendar-personal = Personale

# semantic i18n key (#2542)
calendar-import-ical-title = Importa file iCal

# semantic i18n key (#2542)
calendar-select-ics-file = Seleziona file .ics

# semantic i18n key (#2542)
calendar-import-to-calendar-label = Importa nel calendario

# semantic i18n key (#2542)
calendar-default-calendar = Calendario predefinito

# semantic i18n key (#2542)
calendar-loading = Caricamento calendario...

# semantic i18n key (#2542)
calendar-unavailable = Calendario non disponibile

# semantic i18n key (#2542)
tasks-progress-status = STATUS

# semantic i18n key (#2542)
tasks-progress-sample-title = Implementazione del sistema di autenticazione utente

# semantic i18n key (#2542)
tasks-progress-runtime = Runtime:

# semantic i18n key (#2542)
tasks-progress-sample-action = Scelga la strategia di scadenza del token

# semantic i18n key (#2542)
tasks-progress-estimated = Stimato:

# semantic i18n key (#2542)
tasks-progress-sample-decision = Avrà bisogno di approvazione per la configurazione di sicurezza

# semantic i18n key (#2542)
tasks-progress-log = EGREZIO DI PROGRESSO

# semantic i18n key (#2542)
tasks-progress-terminal-live = TERMINALE (ATTIVITÀ AGENTE VIVE)

# semantic i18n key (#2542)
tasks-progress-processed = Trattamento:

# semantic i18n key (#2542)
tasks-progress-data-points = punti dati

# semantic i18n key (#2542)
tasks-progress-sample-speed = ~8 fonti/min

# semantic i18n key (#2542)
tasks-progress-sample-insight = Sintetizzare le intuizioni competitive...

# semantic i18n key (#2542)
tasks-progress-sample-pricing = Incrocio dei dati di prezzo in corso...

# semantic i18n key (#2542)
tasks-progress-view-details = Visualizza dettagli

# semantic i18n key (#2542)
tasks-create-run = Crea ed esegui

# semantic i18n key (#2542)
tasks-loading-tasks = Caricamento delle attività...

# semantic i18n key (#2542)
tasks-select-task = Seleziona un'attività

# semantic i18n key (#2542)
tasks-select-task-desc = Fai clic su un'attività nell'elenco per visualizzare i dettagli.

# semantic i18n key (#2542)
tasks-bot-database-label = Database del Bot:

# semantic i18n key (#2542)
tasks-shared-resources-label = Risorse condivise:

# semantic i18n key (#2542)
tasks-starting = Inizio...

# semantic i18n key (#2542)
tasks-create-new-intent = Crea nuova intenzione

# semantic i18n key (#2542)
tasks-intent-accomplish-label = Cosa vorresti ottenere?

# semantic i18n key (#2542)
tasks-execution-mode = Modalità di esecuzione

# semantic i18n key (#2542)
tasks-mode-automatic = Automatico

# semantic i18n key (#2542)
tasks-mode-supervised = Supervisionato

# semantic i18n key (#2542)
tasks-mode-manual-approval = Omologazione manuale

# semantic i18n key (#2542)
tasks-make-decision = Prendere decisione

# semantic i18n key (#2542)
tasks-decision-required = Decisione richiesta

# semantic i18n key (#2542)
tasks-loading-decision-details = Caricamento dettagli sulle decisioni...

# semantic i18n key (#2542)
tasks-skip-for-now = Salta per ora

# semantic i18n key (#2542)
tasks-confirm-decision = Conferma decisione

# semantic i18n key (#2542)
tasks-due-soon = Scadenza imminente

# semantic i18n key (#2542)
tasks-autotask-items = // AUTOTASK ELEMENTI

# semantic i18n key (#2542)
tasks-edit-task = Modifica attività

# semantic i18n key (#2542)
tasks-assignee-user-id = Assegnatario (ID utente)

# semantic i18n key (#2542)
tasks-parent-task-subtask-of = Attività principale (sottoattività di...)

# semantic i18n key (#2542)
tasks-reopen-action = ↩ Riapri

# semantic i18n key (#2542)
tasks-complete-action = ✓ Complete

# semantic i18n key (#2542)
tasks-delete-action = 🗑 Delete

# semantic i18n key (#2542)
db-schema-browser = Browser schema

# semantic i18n key (#2542)
db-loading-schema = Caricamento schema...

# semantic i18n key (#2542)
db-new-table = + Nuova tabella

# semantic i18n key (#2542)
db-previous = ← Precedente

# semantic i18n key (#2542)
db-next = Avanti →

# semantic i18n key (#2542)
db-sql-console = Console SQL

# semantic i18n key (#2542)
db-execute-query = Esegui (Ctrl+Enter)

# semantic i18n key (#2542)
db-explain = Spiega

# semantic i18n key (#2542)
db-create-new-table = Crea nuova tabella

# semantic i18n key (#2542)
db-table-name = Nome tabella

# semantic i18n key (#2542)
db-columns = Colonne

# semantic i18n key (#2542)
db-not-null = NON NIENTE

# semantic i18n key (#2542)
db-type-varchar-255 = VARCHAR(255)

# semantic i18n key (#2542)
db-type-text = TESTO

# semantic i18n key (#2542)
db-type-integer = INTEGER

# semantic i18n key (#2542)
db-type-bigint = GRANDE

# semantic i18n key (#2542)
db-type-decimal = DECIMALE

# semantic i18n key (#2542)
db-type-boolean = BOOLANO

# semantic i18n key (#2542)
db-type-date = DATA

# semantic i18n key (#2542)
db-type-timestamptz = TIMESTAMPTZ

# semantic i18n key (#2542)
db-type-jsonb = JSONB

# semantic i18n key (#2542)
db-add-column = + Aggiungi colonna

# semantic i18n key (#2542)
db-create-table = Crei una tabella

# semantic i18n key (#2542)
db-edit-row = Modifica riga

# semantic i18n key (#2542)
db-import-csv = Importa CSV

# semantic i18n key (#2542)
db-select-csv-file = Seleziona file CSV

# semantic i18n key (#2542)
analytics-ask-placeholder = Chieda informazioni sui suoi dati di analisi...
tasks-intent-description-placeholder = Descriva cosa vuole creare o automatizzare...
tasks-assignee-placeholder = UUID del delegato (facoltativo)
tasks-parent-placeholder = UUID dell'attività madre (facoltativo)

# semantic i18n key (#2542)
db-placeholder-search-tables = Tabelle di ricerca...
db-aria-close-sql-console = Chiudi console SQL
db-placeholder-sql-query = SELEZIONA * DA Table_name DOVE stato...
db-aria-sql-query = Interrogazione SQL
db-placeholder-table-name-example = my_table
db-placeholder-column-id = id
db-placeholder-column-name-example = colonna_nome

# semantic i18n key (#2542)
calendar-event-attendees-placeholder = email1, email2

# semantic i18n key (#2542)
db-sql-editor-placeholder = SELEZIONA * DAL tavolo...
db-sql-editor-aria-label = Interrogazione SQL

# Chat header / app chrome (#2542)
paper-save-failed = Riprova
paper-save-retry-hint = Fai clic per ripetere il salvataggio.
paper-saved-locally = Salvato su questo dispositivo
paper-status-bar = Stato documento
tasks-detail-region = Dettagli dell'attività
tasks-filters = Filtri attività
tasks-progress-bar = Barra di avanzamento attività
tasks-progress-close = Chiudi progresso
tasks-progress-minimize = Minimizza i progressi
tasks-progress-output = Uscita attività
tasks-progress-region = Progressi dell'attività
tasks-terminal-log = Attività degli agenti viventi

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
tasks-state-empty-title = Non ci sono ancora compiti
tasks-state-empty-desc = Descriva cosa deve essere fatto nella casella sopra e l'assistente lo trasformerà in un'attività.
tasks-state-empty-action = Inizia con un intento
calendar-state-empty-calendars-title = Ancora nessun calendario
calendar-state-empty-calendars-desc = Un calendario contiene i suoi eventi. Ne crei uno per iniziare a pianificare.
calendar-state-empty-upcoming-title = Niente di programmato
calendar-state-empty-upcoming-desc = I tuoi prossimi appuntamenti arriveranno qui, cosi' il giorno non ti sorprendera' mai.
calendar-state-empty-upcoming-action = Organizza un evento
