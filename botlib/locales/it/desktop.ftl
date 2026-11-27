# General Bots — desktop shell and setup wizard (#2089)

desktop-document-title = Desktop generale dei bot
sidebar-toggle = Commuta barra laterale
sidebar-collapsed-chat = Chat
sidebar-collapsed-terminal = Terminale
sidebar-collapsed-drive = Unità
sidebar-collapsed-settings = Impostazioni
sidebar-collapsed-account = Account
setup-welcome-title = Benvenuti al generale Bots
setup-welcome-subtitle = Ti organizziamo in pochi passi
setup-welcome-quick-subtitle = Due passi veloci per iniziare
setup-step-llm = Fornitore LLM
setup-step-profile = Profilo utente
setup-step-bot = Configurazione del Bot
setup-step-training = Formazione
setup-step-ai-model = Modello IA
setup-step-your-name = Il tuo nome
setup-choose-provider-title = Scegliere il provider LLM
setup-choose-provider-subtitle = Selezionare il fornitore di modelli AI per il vostro bot
setup-provider-openai-desc = Modelli GPT-4, GPT-3.5
setup-provider-nvidia-desc = Inferenza di NVIDIA NIM
setup-provider-anthropic-desc = Modelli Claude 3, Claude 3.5
setup-provider-local-desc = lama.cpp, Ollama, ecc.
setup-profile-title = Cosa ti descrive meglio?
setup-profile-subtitle = Seleziona il tuo tipo di profilo per default personalizzati
setup-profile-developer = Sviluppatore
setup-profile-developer-desc = Creare e distribuire bot, scrivere script BASIC.
setup-profile-business = Azienda
setup-profile-business-desc = Automatizza i flussi di lavoro, gestisci il CRM
setup-profile-support = Supporto
setup-profile-support-desc = Servizio clienti, gestione ticket
setup-bot-title = Configurare il proprio Bot
setup-bot-subtitle = Dai al tuo bot un nome, uno scopo e un modello opzionale
setup-bot-name = Nome del Bot
setup-bot-name-placeholder = Il mio assistente
setup-bot-purpose = Scopo del Bot
setup-bot-purpose-placeholder = Descriva cosa dovrebbe fare il suo bot...
setup-bot-template = Modello del Bot
setup-optional = (opzionale)
setup-template-scratch = • Iniziare da zero • Iniziare da zero • Iniziare da zero • Iniziare da zero • Iniziare da zero • Iniziare da zero • Iniziare da zero • Iniziare da zero • Iniziare da zero • Iniziare da zero • Iniziare da zero • Iniziare da zero • Iniziare da zero • Iniziare da zero • Iniziare da zero • Iniziare da
setup-name-title = Come dovremmo chiamarti?
setup-name-subtitle = Questo nome appare nelle tue conversazioni
setup-name-label = Il tuo nome
setup-name-placeholder = Maria
setup-training-title = Allena il tuo bot
setup-training-subtitle = Rilasci file .pdf, .xls, .doc per l'addestramento iniziale. Avrà accesso a Drive e potrà modificarlo in seguito.
setup-drop-text = Rilasci i file qui o faccia clic per selezionare
setup-drop-hint = .pdf, .xls, .doc — fino a 10 MB ciascuno
setup-select-files = Seleziona file
setup-file-status-pending = In attesa
setup-file-status-uploading = Caricamento
setup-file-status-done = Fatto
setup-file-status-error = Errore
setup-saving = Salvataggio...
setup-complete-message = Benvenuti al generale Bots.
setup-failed = Impostazione non riuscita: {error}
setup-unknown-error = Errore sconosciuto
setup-back = Indietro
setup-next = Avanti
setup-finish = Finisci
setup-skip = Salta configurazione
agent-all-apps = Tutte le app

# --- Desktop shell panels: versions, accessibility, sidebar, collaboration,
# --- updates, modals and usage (suite/js/*.js) — #2095

versions-component = Componente
versions-version = Versione
versions-status = Stato
versions-action = Azione
versions-up-to-date = Aggiornato
versions-unknown = Sconosciuto
versions-check-update = Controlli aggiornamenti
versions-checking = Verifica in corso...
versions-checked = Verificato
versions-error = Errore
versions-update-available = Aggiornamento: { $version }
versions-could-not-load = Non è stato possibile caricare le informazioni sulla versione.
versions-ensure-backend = Assicurati che il server del backend sia in esecuzione.
versions-retry = Riprova

a11y-chat-conversation = Conversazione della chat
a11y-chat-messages = Messaggi della chat
a11y-send-message = Invia messaggio
a11y-type-message = Digita il tuo messaggio
a11y-keyboard-shortcuts = Scorciatoie da tastiera
a11y-shortcut-focus-chat = Input chat di messa a fuoco
a11y-shortcut-show-help = Mostra questo aiuto
a11y-shortcut-escape = Chiudi modale / cancella selezione
a11y-shortcut-command-palette = Palette di comandi
a11y-shortcut-privacy = Impostazioni della privacy
a11y-shortcut-scroll-history = Scorrere la cronologia delle chat
a11y-shortcut-navigate = Navigare tra gli elementi
a11y-shortcut-activate = Attiva elemento attivo
a11y-shortcut-delete-files = Elimina file selezionati (Drive)
a11y-shortcut-select-all = Seleziona tutti i file (Drive)
a11y-shortcut-clipboard = Copia / Taglia / Incolla file
a11y-shortcut-navigate-up = Naviga in alto in unità
a11y-shortcut-rename = Rinomina il file selezionato

sidebar-locked-conversations = Conversazioni chiuse
sidebar-no-conversations = Ancora nessuna conversazione
sidebar-sign-in-conversations = Accedi per vedere le tue conversazioni
sidebar-history-unavailable = Cronologia non disponibile
sidebar-back-to-conversations = ← Torna alle conversazioni
sidebar-no-locked-conversations = Nessuna conversazione bloccata
sidebar-locked-conversation = Conversazione chiusa
# #2095 — conversation-history time buckets.
sidebar-bucket-today = Oggi
sidebar-bucket-yesterday = Ieri
sidebar-bucket-previous-7-days = Precedente 7 Giorni
sidebar-bucket-older = Più vecchio

sidebar-sign-in = Firma
sidebar-add-pin = Aggiungi PIN

collab-collaborators = Collaboratori ({ $count })
collab-no-one-else = Non c'è nessun altro qui.
collab-unavailable = Collaborazione non disponibile
collab-retry = Riprova

update-available-title = Aggiornamento disponibile{ $version }
update-now = Aggiorna ora
update-later = Più tardi
update-skip-version = Salta questa versione
update-platform = Piattaforma: { $name }
update-asset = Risorsa: { $name }
update-check-failed = Controllo aggiornamento non riuscito: { $error }
update-view-release-notes = Visualizza note di rilascio
update-release-notes-title = Note di rilascio{ $version }
update-no-release-notes = Nessuna nota di rilascio disponibile.

modal-alert-title = Avviso
modal-input-title = Input

external-save-to = Salva a { $target }

usage-title = Uso e costo
usage-last-days = Ultimi giorni { $days }
usage-loading = Caricamento utilizzo...
usage-per-model = Per modello
usage-per-day = Al giorno
usage-model = Modello
usage-tokens = Token
usage-messages = Messaggi
usage-cost = Costo
usage-day = Giorno
usage-days-7 = 7 giorni
usage-days-30 = 30 giorni
usage-days-90 = 90 giorni

pin-close = Chiudi
pin-length-hint = 4–8 cifre
pin-cancel = Annulla
pin-ok = OK

# --- Collaboration surfaces shared by every suite app: share dialog, activity
# --- trail, conflict banner and version history (suite/js/collab-*.js) — #2095

role-owner = proprietario
role-editor = Editor
role-commenter = Commentatore
role-viewer = Visualizzatore

share-request-failed = Richiesta non riuscita ({ $status })
share-warning-title = I link condivisi non sono ancora raggiungibili
share-warning-body = Questa installazione è raggiungibile solo su localhost, quindi un link qui non si apre da nessuna parte. Imposta uno di questi primi: abilita P2P nelle impostazioni → Rete, o punti Collegamenti condivisi nel tuo dominio.
share-open-shared-links = Apri collegamenti condivisi
share-open-network = Apri rete
share-invite-people = Invita le persone
share-anyone-with-link = Chiunque abbia il link
share-create-link = Crei un link
share-transfer-ownership = Trasferimento proprietà
share-transfer = Trasferimento
share-new-owner-placeholder = nuova email del proprietario
share-owner-suffix = · proprietario
share-no-people = Non ci sono ancora state persone.
share-no-links = Non c'e' ancora nessun collegamento pubblico.
share-copied = Copiato
share-revoke = Revoke
share-enter-email = Inserisca un'email
share-enter-new-owner = Inserisca l'email del nuovo proprietario
share-needs-base = Imposta P2P o il tuo dominio prima di creare un link condiviso.

activity-title = Attività
activity-request-failed = Richiesta non riuscita ({ $status })
activity-load-more = Carica di più
activity-loading = Caricamento in corso...
activity-empty = Nessuna attività ancora
activity-action-create = creato
activity-action-edit = modificato
activity-action-comment = commentato
activity-action-delete = soppresso
activity-action-resolve = ha risolto un commento su
activity-action-reopen = ha riaperto un commento su
activity-action-reaction = ha reagito a
activity-action-share = condiviso
activity-action-restore = ripristinato
activity-action-transfer = trasferito

conflict-title = Modifiche in conflitto
conflict-dismiss = Ignora
conflict-edited-concurrently = modifica contemporaneamente ({ $op })
conflict-keep-server = Mantieni la versione del server
conflict-keep-mine = Tieni il resto.
conflict-not-started = Monitoraggio conflitti non avviato
conflict-request-failed = Richiesta non riuscita ({ $status })

versions-title = Cronologia delle versioni
versions-loading = Caricamento in corso...
versions-empty = Nessuna versione ancora Hoppenstedt il primo salvataggio ne crea uno.
versions-version-n = Versione { $n }
versions-current-tag = (attuale)
versions-name = Nome...
versions-restore-failed = Ripristino non riuscito: { $error }
versions-rename-failed = Rinomina non riuscita: { $error }
versions-restore-dialog-title = Ripristina versione
versions-restore-confirm = Ripristinare questa versione? Una nuova versione corrente verrà creata
versions-name-dialog-title = Versione del nome
versions-name-prompt = Indicare questa versione (ad esempio "approvata v2
versions-request-failed = Richiesta non riuscita ({ $status })

# --- Desktop shell: snap picker, workspace tabs, virtual desktops, context
# --- menu, git panel, browser panel, widgets and data export (suite/js/*.js)

snap-title = Snap "{ $app }"
snap-layout-top-left = In alto a sinistra
snap-layout-top = Mezzo superiore
snap-layout-top-right = In alto a destra
snap-layout-left = Mezzo sinistro
snap-layout-full = Massimizza
snap-layout-right = Meta destra
snap-layout-bottom-left = In basso a sinistra
snap-layout-bottom = Metà inferiore
snap-layout-bottom-right = In basso a destra
snap-layout-third-left = Terza a sinistra
snap-layout-two-thirds = Due terzi
snap-layout-third-right = Terzo destro

wstabs-close-tab = Chiuda scheda
wstabs-new-tab = Nuova scheda spazio di lavoro
wstabs-default-name = Spazio di lavoro { $n }
wstabs-default-main = Principale
wstabs-rename-title = Rinomina spazio di lavoro
wstabs-name-prompt = Nome spazio di lavoro:

vdesk-default-name = Scrivania { $n }
vdesk-title = Desktop virtuali
vdesk-new = Nuovo desktop
vdesk-rename-title = Rinomina desktop
vdesk-name-prompt = Nome desktop:
vdesk-restore-title = Ripristina sessione
vdesk-restore-body = Riaprire la finestra { $count }(s) dall'ultima sessione su questo desktop?
vdesk-restore-body-short = Ripristinare l'ultima sessione (finestre { $count }(s))?

ctxmenu-search = Cerca DuckDuckGo
ctxmenu-search-prompt = Cerca DuckDuckGo:
ctxmenu-wikipedia = Definizione di Wikipedia
ctxmenu-wikipedia-prompt = Definizione di Wikipedia:

git-loading-status = Caricamento stato...
git-no-changes = Nessuna modifica
git-new-branch-prompt = Nuovo nome della filiale:
git-loading-diff = Caricamento diff...
git-diff-failed = Impossibile caricare il diff
git-commit-message-required = Inserire un messaggio di commit
git-pushing = Spingere...
git-committed = Commit e push effettuati con successo
git-failed = Operazione Git non riuscita

browser-session-active = Sessione browser { $id } attiva.
browser-init-failed = Impossibile inizializzare il browser
browser-init-first = Si prega di inizializzare il browser prima
browser-capture = Acquisisci
browser-no-session = Nessuna sessione attiva
browser-export-failed = Esportazione non riuscita
browser-no-actions = Nessuna azione registrata

widget-load-failed = Widget non riuscito a caricare
widget-no-source = Nessun sorgente widget configurato
widget-open-windowed = Apri finestra
widget-pin-launcher = Pin al lanciatore

dataexport-title = Esportazione e backup dati
dataexport-desc = Esporta tutti i dati, inclusa la cronologia chat, la memoria, i metadati di Drive e le impostazioni, come file di backup JSON.
dataexport-export-all = Esporta tutti i dati
dataexport-import = Importa backup
dataexport-exported = Backup esportato con successo
dataexport-too-large = Backup rifiutato: il file è più grande di 4 MB
dataexport-invalid = File di backup non valido: { $error }
dataexport-nothing = Questo backup non contiene impostazioni ristabilibili
dataexport-unknown-date = data sconosciuta
dataexport-restore-confirm = Questo ripristina le impostazioni { $count }(s) dal backup di { $date }. Credenziali e token di sessione non vengono mai importati. Continua?
dataexport-restored = Backup ripristinato ({ $count } elementi).
dataexport-rejected = { $count } entry(ies) bocciata dall'allowlist.
# #2095 — backup-import validation details shown inside
# `dataexport-invalid`.
dataexport-error-not-object = il file non è un oggetto di backup
dataexport-error-version = Versione di backup non supportata
dataexport-error-date = il backup non ha una data di esportazione valida
dataexport-error-section = sezione inattesa: { $section }
dataexport-reloading = Ricaricamento...

comments-title = Commenti
comments-request-failed = Richiesta non riuscita ({ $status })
comments-loading = Caricamento commenti...
comments-load-failed = Impossibile caricare i commenti: { $error }
comments-empty-open = Non ci sono commenti aperti. Usa @name per menzionare qualcuno.
comments-empty-resolved = Nessun commento risolto. Usa @name per menzionare qualcuno.
comments-resolved-chip = Risolto
comments-resolve = Risolvi
comments-reopen = Riapri
comments-resolved-announce = Commento risolto
comments-reopened-announce = Commento riaperto
comments-resolve-failed = Risoluzione non riuscita: { $error }
comments-added = Commento aggiunto
comments-post = Posta
comments-post-failed = Commento non riuscito: { $error }
comments-reaction-failed = Reazione non riuscita: { $error }
comments-delete-failed = Eliminazione non riuscita: { $error }
comments-reply-failed = Risposta non riuscita: { $error }
comments-delete-title = Elimina commento
comments-delete-confirm = Eliminare questo commento?
comments-reply-title = Rispondi
comments-reply-prompt = Rispondi al commento:
comments-filter-open = Apri
comments-filter-all = Tutto
comments-filter-resolved = Risolto
comments-input-placeholder = Aggiungi un commento… usa @name per menzionare
comments-someone = qualcuno
comments-mention-announce = Sei stato citato in un commento da { $author }

# semantic i18n key (#2542)
user-menu-help-support = Aiuto e supporto

# semantic i18n key (#2542)
user-menu-sign-out = Esci

# semantic i18n key (#2542)
desktop-session-management = Gestione sessione

# semantic i18n key (#2542)
desktop-sessions-hint = Sessioni desktop remoto attive in tutte le VM

# semantic i18n key (#2542)
desktop-active-sessions = Sessioni attive

# semantic i18n key (#2542)
desktop-session-idle = Inattivo

# semantic i18n key (#2542)
desktop-session-state = Stato

# semantic i18n key (#2542)
desktop-session-duration = Durata

# semantic i18n key (#2542)
desktop-no-active-sessions = Nessuna sessione attiva.

# semantic i18n key (#2542)
setup-provider-nvidia-title = NVIDIA

# semantic i18n key (#2542)
setup-bot-config-desc = Dare al tuo bot un nome e descrivere il suo scopo

# semantic i18n key (#2542)
setup-media-features = Caratteristiche del supporto

# semantic i18n key (#2542)
git-commit-placeholder = Messaggio (Ctrl+Invio per commit)

# --- #2921 missing keys added to every locale catalogue ---
collab-typing-one = { $name } sta digitando...
collab-typing-two = { $first } e { $second } stanno digitando...
collab-typing-many = { $name } e { $count } altri stanno digitando...
