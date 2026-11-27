# =============================================================================
# General Bots — en — Desktop shell, navigation, controls
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# =============================================================================
# General Bots - English UI Translations
# =============================================================================
# -----------------------------------------------------------------------------
# Navigation
# -----------------------------------------------------------------------------
nav-home = Partenza
nav-chat = Chat
nav-drive = Unità
nav-tasks = Compiti
nav-mail = Posta
nav-calendar = Calendario
nav-meet = Incontra
nav-paper = Carta
nav-video = Video
nav-research = Ricerca
nav-analytics = Analisi
nav-settings = Impostazioni
nav-admin = Amministratore
nav-monitoring = Monitoraggio
nav-sources = Fonti
nav-tools = Strumenti
nav-attendant = Addetto
nav-learn = Imparare
nav-crm = CRM
nav-billing = Fatturazione
nav-products = Prodotti
nav-tickets = Biglietti
nav-docs = Documenti
nav-sheet = Fogli
nav-slides = Diapositive
nav-social = Sociale
nav-all-apps = Tutte le app
nav-people = Persone
nav-editor = Editor
nav-dashboards = Dashboard
nav-security = Sicurezza
nav-designer = Progettista
nav-project = Progetto
nav-canvas = Tela
nav-goals = Obiettivi
nav-player = Giocatore
nav-workspace = Spazio di lavoro

# -----------------------------------------------------------------------------
# Dashboard
# -----------------------------------------------------------------------------
dashboard-title = Cruscotto
dashboard-welcome = Bentornato, { $name }!
dashboard-quick-actions = Azioni rapide
dashboard-recent-activity = Attività recente
dashboard-no-activity = Nessuna attività recente ancora. Iniziate ad esplorare!
dashboard-analytics = Analisi

# -----------------------------------------------------------------------------
# Quick Actions
# -----------------------------------------------------------------------------
quick-start-chat = Avvia conversazione
quick-upload-files = Carica file
quick-new-task = Nuova attività
quick-compose-email = Componi email
quick-start-meeting = Inizia riunione
quick-new-event = Nuovo evento

# -----------------------------------------------------------------------------
# Application Cards
# -----------------------------------------------------------------------------
app-chat-name = Chat
app-chat-desc = Conversazioni basate sull'IA. Faccia domande, ottenga assistenza e automatizzi le attività.
app-drive-name = Unità
app-drive-desc = Archiviazione cloud per tutti i Suoi file. Carichi, organizzi e condivida.
app-tasks-name = Compiti
app-tasks-desc = Resta organizzato con le liste di cose da fare, priorità e date di scadenza.
app-mail-name = Posta
app-mail-desc = Client email con scrittura assistita da IA e organizzazione intelligente.
app-calendar-name = Calendario
app-calendar-desc = Organizza incontri, eventi e gestisci il tuo tempo in modo efficace.
app-meet-name = Incontra
app-meet-desc = Videoconferenza con condivisione dello schermo e trascrizione in diretta.
app-paper-name = Carta
app-paper-desc = Scrivere documenti con l'assistenza AI. Note, relazioni, e altro ancora.
app-research-name = Ricerca
app-research-desc = Ricerca e scoperta basate sull'IA in tutte le Sue fonti.
app-analytics-name = Analisi
app-analytics-desc = Dashboard e report per monitorare l'utilizzo e le informazioni dettagliate.

# -----------------------------------------------------------------------------
# Suite Header
# -----------------------------------------------------------------------------
suite-title = Suite generale dei bot
suite-tagline = Il tuo spazio di lavoro di produttività alimentato da IA. Chat, collaborare e creare.
suite-new-intent = Nuovo intento

# -----------------------------------------------------------------------------
# AI Panel
# -----------------------------------------------------------------------------
ai-developer = Sviluppatore IA
ai-developing = Sviluppo: { $project }
ai-quick-actions = Azioni rapide
ai-add-field = Aggiungi campo
ai-change-color = Cambia colore
ai-add-validation = Aggiungi validazione
ai-export-data = Esporta dati
ai-placeholder = Digita le modifiche...
ai-thinking = L'IA sta elaborando…
ai-status-online = Online
ai-status-offline = Fuori rete

# -----------------------------------------------------------------------------
# Search
# -----------------------------------------------------------------------------
search-placeholder = Cerca...
search-no-results = Nessun risultato trovato
search-results = { $count ->
    [one] Risultato { $count }
    *[other] Risultati { $count }
    }
search-in-progress = Ricerca...
search-advanced = Ricerca avanzata
search-filters = Filtri
search-clear-filters = Cancellare filtri

# -----------------------------------------------------------------------------
# Pagination
# -----------------------------------------------------------------------------
# -----------------------------------------------------------------------------
# Tables
# -----------------------------------------------------------------------------
table-no-data = Nessun dato disponibile
table-loading = Caricamento dati...
table-actions = Azioni
table-select-all = Seleziona tutto
table-deselect-all = Deseleziona tutto
table-export = Esporta
table-import = Importa

# -----------------------------------------------------------------------------
# Forms
# -----------------------------------------------------------------------------
form-required = Richiesto
form-optional = Facoltativo
form-submit = Invia
form-reset = Reimposta
form-clear = Cancellare
form-uploading = Caricamento...
form-processing = Elaborazione...

# -----------------------------------------------------------------------------
# Modals / Dialogs
# -----------------------------------------------------------------------------
modal-confirm-title = Conferma azione
modal-confirm-message = È sicuro di voler procedere?
modal-delete-title = Conferma eliminazione
modal-delete-message = Questa azione non puo' essere annullata.

# -----------------------------------------------------------------------------
# Tooltips
# -----------------------------------------------------------------------------
tooltip-copy = Copi negli Appunti
tooltip-copied = Copiato!
tooltip-expand = Espandi
tooltip-collapse = Comprima
tooltip-refresh = Aggiorna
tooltip-download = Scarica
tooltip-upload = Carica
tooltip-print = Stampa
tooltip-fullscreen = Schermo intero
tooltip-exit-fullscreen = Esci da schermo intero

# -----------------------------------------------------------------------------
# Common Labels (additional)
# -----------------------------------------------------------------------------
label-username = Nome utente
label-email = Email
label-display-name = Nome visualizzato
label-password = Password
label-role = Ruolo
label-group-name = Nome gruppo
label-hostname = Nome host
label-record-type = Tipo di registrazione
label-target = Obiettivo
label-your-name = Il tuo nome

# -----------------------------------------------------------------------------
# Actions (additional)
# -----------------------------------------------------------------------------
action-register = Registro

# -----------------------------------------------------------------------------
# Notifications
# -----------------------------------------------------------------------------
notifications-title = Notifiche
notifications-clear = Cancellare tutto
notifications-empty = Nessuna notifica

# #1853 — desktop chrome, launchers and notification settings.
window-ready = Pronto
window-loading = Caricamento app...
window-minimize = Minimizza
window-maximize = Massimizza
window-fullscreen = Schermo intero
window-exit-fullscreen = Esci da schermo intero
window-close = Chiudi
app-loading = Caricamento dell'applicazione…
start-menu-search-placeholder = Digitare per trovare un'app...
start-menu-recent = Recenti
start-menu-no-apps = Nessuna applicazione trovata
start-menu-user = Utente
category-ai = IA
category-business = Azienda
category-office = Ufficio
category-dev = Sviluppatore
category-system = Sistema
category-tools = Strumenti
sidebar-apps = App
sidebar-apps-toggle = Mostra/nascondi app
sidebar-apps-toggle-aria = Commuta pannello delle applicazioni
sidebar-apps-filter = Filtra le applicazioni...
sidebar-apps-filter-aria = Filtro apps
sidebar-preview = Modalità anteprima
sidebar-preview-hint = Mostra le applicazioni che sono ancora in anteprima
sidebar-preview-toggle = Moda di anteprima a due livelli
sidebar-app-preview-title = {app} (anteprima)
sidebar-apps-count = App {count}
sidebar-apps-count-preview = {count} (+ anteprima {preview})
spotlight-placeholder = Cerca app, file... o digita un obiettivo per il Concierge
spotlight-hint = ↑↓ navigare · Inserisci esecuzione · Ctrl+Spazio o Esc chiudere
spotlight-empty = Nessun risultato
spotlight-goal = Obiettivo di esecuzione:{goal}
spotlight-goal-subtitle = Concierge · pianifica ed esegui tra le app
palette-placeholder = Ricerca app e comandi. Oppure digita @ <comando> da inviare all'assistente...
palette-empty = Nessuna applicazione o comando corrispondente
palette-kind-app = App
palette-hint = ↑↓ navigare · Inserisci esecuzione · Esc close · Ctrl+K toggle
palette-open-app = Apri {app}
palette-value-prompt = {name}:{summary}Valore{key}:
palette-execute = Eseguire {method} {path} e riepilogare il risultato.
palette-use-command = Usare il comando {command} a {summary}.
toast-dismiss = Ignora
settings-language-display = Lingua di visualizzazione
settings-language-display-desc = Scegliere la lingua per l'interfaccia dell'applicazione
settings-language-regional = Impostazioni regionali
settings-language-regional-desc = Configurare formati di data, ora e numeri
settings-language-date-format = Formato data
settings-language-time-format = Formato orario
settings-language-timezone = Fuso orario
settings-language-first-day = Primo giorno della settimana
settings-language-time-12h = 12 ore (1:30 PM)
settings-language-time-24h = 24 ore (13:30)
settings-language-sunday = Domenica
settings-language-monday = Lunedì
settings-email-notifications-desc = Inviato dal mittente di notifica dello spazio di lavoro, mai da una casella di posta Gmail/Outlook collegata.
settings-push-enable = Abilita notifiche push
settings-push-enable-desc = Ricevi notifiche push del browser
settings-push-sound = Suono
settings-push-sound-desc = Riproduce il suono per le notifiche
settings-desktop-notifications = Notifiche desktop
settings-desktop-notifications-desc = Mostra le notifiche sul desktop
settings-badge-count = Conteggio badge
settings-badge-count-desc = Mostra il badge non letto
settings-save-preferences = Salva preferenze
settings-notifications-saved = Preferenze di notifica salvate
settings-notification-recipient = Cassetta di consegna
settings-notification-recipient-desc = Lascia vuoto per utilizzare il tuo account di posta principale o email di profilo.
language-search-placeholder = Cerca lingue...
settings-profile-title = Impostazioni profilo
settings-security-title = Impostazioni di sicurezza

# semantic i18n key (#2542)
apps-menu-placeholder-search = Cerca app...
apps-menu-aria-search = Cerca nelle app
apps-menu-aria-close = Chiudi menu app

# --- #2921 missing keys added to every locale catalogue ---
palette-kind-tour = Tour
palette-show-tour = Mostra tour
palette-show-tour-hint = Mostra il tour del prodotto
sidebar-app-hide = Nascondi { $app } da questa barra
sidebar-apps-count-off = { $count } spento
sidebar-apps-empty = Nessuna applicazione da mostrare
sidebar-apps-manage = Gestisci app
sidebar-apps-manage-all = Mostra tutto
sidebar-apps-manage-close = Chiudi
sidebar-apps-manage-empty = Nessuna applicazione disponibile
sidebar-apps-manage-summary = { $on } di { $total } mostrata
sidebar-apps-none = Nessuna applicazione
toast-open = Apri

# Assistant shell — knowledge-base source blocks
trecho = estratto
trechos = estratti

# Assistant/Drive shell keys integrated from the gbgov edition
arquivo = file
arquivos = file
assistant-agent-backend-off = Il backend agente è disattivato
assistant-agent-locked-hint = L'agente è bloccato per questo modello
assistant-agent-locked-tip = Scelga un altro modello per utilizzare l'agente
assistant-regenerate-older = Rigenerare una risposta più vecchia inizia una nuova svolta
caracteres = caratteri
corrigido = fisso
de = di
drive-actions = Azioni
drive-folders = Cartelle
drive-usage-hint = Conservazione utilizzata
drive-used = Usato
item = voce
itens = elementi
linha = riga
linhas = linee
resultado = risultato
resultados = risultati
shell-close = Chiudi
shell-conversations = Conversazioni
shell-machine = Macchina

# Assistant shell — literal keys
assistant-vm-wants = L'agente vuole fare un'azione
assistant-vm-wants-command = L'agente vuole eseguire un comando sulla macchina
drive-label = Unità

# Chat / assistant parity — VM consent and subscription-agent UI keys
chat-vm-consent-assistant = Assistente
chat-vm-consent-subject = Agente
chat-vm-consent-wants = L'agente vuole fare un'azione
chat-vm-consent-wants-command = L'agente vuole eseguire un comando
chat-vm-consent-kicker = Ha bisogno del tuo permesso
chat-vm-consent-command-label = Comando
chat-vm-consent-where = Dove
chat-vm-consent-where-vm = Macchina virtuale
chat-vm-consent-where-drive = il tuo disco
chat-vm-consent-destination = Destinazione
chat-vm-consent-language = Lingua
chat-vm-consent-packages = Installa da internet
chat-vm-consent-from = Fonte
chat-vm-consent-background = Esegui sullo sfondo
chat-vm-consent-repair = Versione corretta del programma
chat-vm-consent-protected = Questa richiesta riguarda le credenziali dell'agente; può essere consentita solo una volta.
chat-vm-consent-all-note = Consentire tutto copre ogni azione di questo agente in questa conversazione, solo all'interno della Macchina Virtuale.
chat-vm-consent-outside-note = Questa azione lascia la Macchina Virtuale: permette a tutti di non coprirla.
chat-vm-consent-allow-once = Consenta una volta
chat-vm-consent-allow-conversation = Consenta in questa conversazione
chat-vm-consent-allow-all = Consenta tutto
chat-vm-consent-deny = Nega
chat-vm-consent-outcome-all = Autorizzazione completa in questa conversazione
chat-vm-consent-outcome-conversation = Consentito in questa conversazione
chat-vm-consent-outcome-once = Consentito una volta
chat-vm-consent-outcome-denied = Negato da Lei: nulla è stato eseguito
chat-vm-consent-outcome-failed = Impossibile registrare la sua decisione
chat-vm-consent-forbidden = Questa decisione non può essere registrata qui. La richiesta rimane aperta.
chat-vm-consent-announce = Richiesta di autorizzazione aperta. Premere Alt+P per rispondere.
chat-vm-consent-still-open = Una richiesta di autorizzazione è ancora aperta. Scelga un'opzione; Nega non esegue nulla.
chat-vm-consent-waiting = Aspettando il tuo permesso
chat-vm-all-chip = Autorizzazione completa
chat-vm-all-scope = solo all'interno della Macchina Virtuale
chat-vm-all-revoke = Revoke
chat-vm-all-revoked = Autorizzazione totale revocata
chat-vm-send-head = Conferma
chat-vm-send-text = Inviare ora annulla la richiesta di autorizzazione aperta. Inviare comunque?
chat-vm-send-back = Torni alla richiesta
chat-vm-send-go = Invia
chat-vm-engine-local-short = Locale
chat-vm-engine-local-group = Modelli locali
chat-vm-engine-local-detail = Su questo computer
chat-vm-engine-free-short = Libero
chat-vm-engine-free-group = Modello libero
chat-vm-engine-free-detail = Modello libero
chat-vm-engine-own-short = Chiave propria
chat-vm-engine-own-group = Chiavi proprie
chat-vm-engine-own-detail = Server o chiave propri
chat-vm-engine-sub-short = Abbonamento
chat-vm-engine-sub-group = Abbonamenti
chat-vm-engine-sub-detail = Agente aggiunto
chat-vm-agent-locked-tip = L'agente selezionato viene eseguito nella macchina virtuale; scegli un altro modello per disattivarlo
chat-vm-agent-locked-note = gira nella macchina virtuale, quindi l'interruttore Agent rimane acceso mentre viene selezionato. Scegliere un altro modello per disattivarlo.
atende = partecipare
