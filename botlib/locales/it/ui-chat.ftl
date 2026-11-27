# =============================================================================
# General Bots — en — Chat
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Chat
# -----------------------------------------------------------------------------
chat-title = Chat
chat-placeholder = Digita il tuo messaggio...
chat-send = Invia
chat-new-conversation = Nuova conversazione
chat-history = Cronologia chat
chat-clear = Cancellare chat
chat-export = Esporta chat
chat-typing = { $name } sta digitando...
chat-online = Online
chat-offline = Fuori rete
chat-last-seen = Ultima vista { $time }
chat-mention-title = Entità di riferimento
chat-mention-placeholder = Messaggio... (tipo @ da menzionare)
chat-mention-search = Enti di ricerca...
chat-mention-no-results = Nessun risultato trovato
chat-mention-type-hint = Tipo: da cercare

# Additional Chat translations
chat-camera = Fotocamera
chat-camera-title = Scatta una foto
chat-camera-capture = Scatta foto
chat-camera-cancel = Annulla
chat-camera-waiting = Aspettando che la telecamera inizi...
chat-camera-waiting-frames = Permesso concesso; in attesa di fotogrammi video
chat-camera-ready = Fotocamera pronta. Scatti la foto o annulli.
chat-camera-started = Fotocamera avviata — tocchi Scatta foto.
chat-camera-sent = Foto inviata.
chat-camera-denied = L'autorizzazione per la fotocamera è stata negata. Consenta l'accesso alla fotocamera per l'app desktop nelle impostazioni di sistema per scattare foto.
chat-camera-missing = Non c'e' stata nessuna telecamera su questo dispositivo.
chat-camera-no-device = nessun dispositivo video
chat-camera-device = dispositivo video
chat-camera-no-frames = L'anteprima della fotocamera non è iniziata. Scegliere invece una foto dal picker di sistema.
chat-camera-timeout = La fotocamera non ha risposto. Scegliere una foto dal picker di sistema invece.
chat-camera-error = Impossibile avviare la fotocamera.
chat-camera-unsupported = Questa app desktop non fornisce l'accesso alla fotocamera.
voice-mode-title = Conversazione vocale
voice-mode-listening = Ascoltare... parlare in modo naturale
voice-mode-thinking = Pensando...
voice-mode-speaking = Parlando...
voice-mode-idle = Conversazione vocale
voice-mode-end = Termina conversazione vocale
chat-message-placeholder = Messaggio...

# Chat message controls and notices
chat-copy-message = Copi messaggio
chat-copied = Copiato
chat-message-copied = Messaggio copiato
chat-copy-failed = Copia non riuscita
chat-listen-tts = Ascolta (TTS)
chat-listen-message = Ascolta questo messaggio
chat-image-unavailable = immagine non disponibile
chat-open = Apri
chat-expand = Espandi
chat-collapse = Comprima
chat-preview = Anteprima
chat-consent-affected-count = Record interessati
chat-close-preview = Chiudi anteprima
chat-preview-load-failed = Non è stato possibile caricare l'anteprima.
chat-show-more = Mostra di più
chat-show-less = Mostra meno
chat-tool = strumento
chat-param = param
chat-params = parami
chat-tool-result = Risultato strumento
chat-retry = Riprova
chat-open-as-document = Apri come documento
chat-show-more-history = Mostra di più...
chat-disconnected-reconnecting = Chat disconnessa. Riconnessione in corso...
chat-connect-retrying = Impossibile raggiungere il server di chat. Nuovo tentativo in corso...
chat-connection-lost = Connessione persa. Tentativo di riconnessione in corso...
chat-attachment-sent = Allegato inviato:
chat-attachments-sent = Allegati inviati:
chat-message-not-delivered = Messaggio non consegnato
chat-attachment-read-failed = Impossibile leggere un allegato. Riallegarlo.
chat-attachment-queued = Connessione ripristinata a breve — allegato in coda
chat-reconnecting-queued = Riconnessione del messaggio verrà inviato quando la connessione è di nuovo.
chat-bot-private-signin = Questo bot è privato
chat-approved-plan = Piano approvato:
chat-retrying-attachment = Riprova dell'allegato
chat-expanded-preview = Anteprima espansa

# #2362 — chat code/artifact control labels
chat-code-show-all = Mostra tutto
chat-code-lines = linee
chat-artifact-save-drive = Salva su disco
chat-artifact-download-svg = Scarica SVG
chat-artifact-download-png = Scarica PNG
chat-artifact-open-canvas = Apri in tela
chat-artifact-folder = Cartella (opzionale)
chat-artifact-kind-markdown = Documento di markdown
chat-artifact-kind-html = Pagina HTML
chat-artifact-kind-code = Codice
chat-artifact-words = parole

# #2362 — attachment action label
chat-open-in-app = Apri in app
# Assistant-look chat: transcript export and answer feedback
chat-export-actions = Azioni della conversazione
chat-export-md = Esporta conversazione come Markdown
chat-export-print = Stampa o salva come PDF
chat-export-done = Conversazione esportata
chat-export-empty = Ancora niente da esportare
chat-export-untitled = Conversazione
chat-export-user = Tu
chat-export-assistant = Assistente
chat-feedback-label = Feedback sulla risposta
chat-feedback-good = Buona risposta
chat-feedback-bad = Risposta non valida
chat-feedback-comment = Cosa potrebbe esserci di meglio?
chat-feedback-send = Invia
chat-staged-files = File allegati
chat-staged-close = Chiudi anteprima

# Chat engineer status strip (#2342)
chat-status-bot = BotCity name (optional, probably does not need a translation)
chat-status-session = Sessione
chat-status-model = Modello
chat-status-state = Stato
chat-status-no-session = —
chat-status-copy-session = Copi ID sessione
chat-status-copied = Copiato
chat-status-idle = Inattivo
chat-status-streaming = Flusso
chat-status-thinking = Pensare
chat-status-error = Errore
chat-status-stopped = Fermato

chat-response-ready = Risposta pronta
chat-response-started = L'assistente sta rispondendo
chat-tool-result-ready = Risultato strumento disponibile

# semantic i18n key (#2542)
chat-no-suggestions-available = Nessun suggerimento disponibile

# semantic i18n key (#2542)
chat-start-conversation = Avvia una conversazione

# semantic i18n key (#2542)
chat-empty-hint = Digitare un messaggio sotto o usare input vocale

# semantic i18n key (#2542)
chat-no-conversations-yet = Ancora nessuna conversazione

# semantic i18n key (#2542)
chat-context-label = Contesto:

# semantic i18n key (#2542)
chat-select-context = Seleziona contesto...

# semantic i18n key (#2542)
chat-no-contexts-configured = Nessun contesto configurato

# semantic i18n key (#2542)
projector-title = Visualizzatore contenuti

# semantic i18n key (#2542)
projector-loading-content = Caricamento contenuto...

# semantic i18n key (#2542)
projector-slide-info = Diapositiva 1 di 1

# semantic i18n key (#2542)
projector-code-info = codice.rs

# Projector viewer control (#2542)
chat-projector-rotate = Ruota

# Chat header / app chrome (#2542)
chat-conversation-clear = Cancellare contesto
chat-conversation-cleared = Contesto della conversazione cancellato
chat-conversation-copy-link = Copi link
chat-conversation-delete = Elimina
chat-conversation-delete-confirm = Eliminare questa conversazione?
chat-conversation-delete-failed = Impossibile eliminare la conversazione
chat-conversation-delete-title = Elimina conversazione
chat-conversation-delete-unavailable = Finestra di dialogo di conferma non disponibile
chat-conversation-deleted = Conversazione eliminata
chat-conversation-export-json = Esporta trascrizione (.json)
chat-conversation-export-md = Esporta trascrizione (.md)
chat-conversation-exported = Conversazione esportata
chat-conversation-link-copied = Link della conversazione copiato
chat-conversation-link-copy-failed = Impossibile copiare il link
chat-conversation-menu = Menu della conversazione
chat-conversation-rename = Rinomina
chat-conversation-rename-failed = Impossibile rinominare la conversazione
chat-conversation-renamed = Conversazione rinominata
chat-conversation-title = Titolo della conversazione
chat-conversation-untitled = Conversazione senza titolo

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
chat-empty-title = Avvia la conversazione
chat-empty-desc = Digitare un messaggio qui sotto, o scegliere un suggerimento quando il vostro bot offre uno.
chat-empty-action = Scrivi il primo messaggio

# -----------------------------------------------------------------------------
# #2342 — chat engineer UX: slash commands, help, inspect/copy-markdown
# -----------------------------------------------------------------------------
chat-cmd-menu-title = Comandi
chat-cmd-help = Mostra le scorciatoie da tastiera
chat-cmd-new = Avvia una nuova conversazione
chat-cmd-clear = Cancella questa conversazione
chat-cmd-stop = Smettila di generare
chat-stop = Ferma
chat-cmd-failed = Comando non riuscito
chat-help = Scorciatoie da tastiera
chat-inspect = Ispezionare
chat-inspect-title = Dettagli messaggio
chat-inspect-sender = Mittente
chat-inspect-id = Id messaggio
chat-inspect-no-id = nessuno
chat-inspect-length = Caratteri
chat-copy-markdown = Copi come Markdown
chat-copied-markdown = Copiato come Markdown

# Model attribution on each bot answer
chat-model-written-by = Risposta scritta da

# Assistant parity in chat
assistant-parity-block-allow = Consenta
assistant-parity-block-allowed = Consentito
assistant-parity-block-api-command = comando API
assistant-parity-block-attachment-read = Allegato letto
assistant-parity-block-chars = caratteri
assistant-parity-block-command = Comando
assistant-parity-block-decision-failed = Impossibile registrare la decisione
assistant-parity-block-denied = Negato
assistant-parity-block-deny = Nega
assistant-parity-block-detail = Dettaglio
assistant-parity-block-drive-list = Elenco di Drive
assistant-parity-block-drive-read = File di Drive letto
assistant-parity-block-drive-search = Ricerca in Drive
assistant-parity-block-elapsed = Trascorso
assistant-parity-block-error = Errore
assistant-parity-block-file = File
assistant-parity-block-kb-search = Ricerca delle conoscenze
assistant-parity-block-notice = Avviso
assistant-parity-block-path = Percorso
assistant-parity-block-permission = Permesso richiesto
assistant-parity-block-query = Interrogazione
assistant-parity-block-read = Leggi
assistant-parity-block-reason = Motivazione
assistant-parity-block-result = Risultato
assistant-parity-block-retry = Riprova
assistant-parity-block-seconds = s
assistant-parity-block-size = Dimensione
assistant-parity-block-status-done = Fatto
assistant-parity-block-status-failed = Non riuscito
assistant-parity-block-status-needs-you = Ha bisogno del tuo input
assistant-parity-block-status-running = Esecuzione
assistant-parity-block-step = Passo
assistant-parity-block-thinking = Pensare
assistant-parity-block-tool = Strumento
assistant-parity-block-tool-run = Esecuzione strumento
assistant-parity-block-total = Totale
assistant-parity-block-truncated = troncato
assistant-parity-block-type = Tipo
assistant-parity-block-yes = Sì
chat-parity-attach-title = Anteprima allegato
chat-parity-attach-unavailable = Contenuto non disponibile - usa Download per aprire questo file.
chat-parity-attach-download-failed = Non è stato possibile scaricare l'allegato.
chat-parity-model-tiers = Automatico e livelli
chat-parity-model-accounts = Conti del fornitore
chat-parity-model-local = Locale & libero
chat-drive-parity-attaching = Allegando...
chat-drive-parity-cancel = Annulla
chat-drive-parity-delete = Elimina
chat-drive-parity-deleted = Eliminato
chat-drive-parity-delete-failed = Non è stato possibile eliminare il file.
chat-drive-parity-delete-question = Eliminare questo file?
chat-drive-parity-delete-title = Elimina file
chat-drive-parity-deleting = Eliminazione in corso...
chat-drive-parity-download = Scarica
chat-drive-parity-download-failed = Non è stato possibile scaricare il file.
chat-drive-parity-downloading = Download in corso...
chat-drive-parity-more = Altre azioni
chat-drive-parity-new-name = Nuovo nome
chat-drive-parity-preview = Anteprima
chat-drive-parity-preview-failed = Impossibile visualizzare in anteprima il file.
chat-drive-parity-previewing = Anteprima di apertura...
chat-drive-parity-rename = Rinomina
chat-drive-parity-renamed = Rinomina
chat-drive-parity-rename-failed = Impossibile rinominare il file.
chat-drive-parity-rename-title = Rinomina file
chat-drive-parity-renaming = Rinominando...
chat-drive-parity-unavailable = Questa azione non è disponibile in questo spiegamento.
chat-drive-parity-use-in-chat = Usare nella conversazione
chat-parity-agent-does = L'agente lo fa.
chat-parity-back = Indietro
chat-parity-checked-now = Verificato poco fa
chat-parity-close = Chiudi
chat-parity-coming-soon = Presto disponibile
chat-parity-cost-contract = Per il tuo contratto
chat-parity-cost-free = Libero
chat-parity-cost-free-limit = Gratis, con limiti di utilizzo
chat-parity-cost-sub = Il tuo abbonamento
chat-parity-cuda-no = Nessuna GPU appropriata rilevata
chat-parity-cuda-yes = Scheda grafica rilevata
chat-parity-depends-agent = Dipende dall'agente
chat-parity-driver = Conducente
chat-parity-engine = Motore in uso
chat-parity-feature-docs = Documenti
chat-parity-feature-ocr = Testo in immagini
chat-parity-feature-photos = Foto e grafici
chat-parity-feature-web = Ricerca web
chat-parity-features = Funzionalità attive
chat-parity-gains = I tuoi guadagni
chat-parity-go-agent = Va dal fornitore dell'agente.
chat-parity-go-ext = Va a un provider esterno
chat-parity-go-net = Rimane nella rete della tua organizzazione
chat-parity-go-server = Va al server impostato
chat-parity-guide = Non sai quale scegliere?
chat-parity-guide-sub = Risponda a due domande.
chat-parity-in-use = In uso
chat-parity-load-failed = Non è stato possibile caricare queste informazioni.
chat-parity-loading = Caricamento in corso...
chat-parity-machine = Questo computer
chat-parity-machine-ok = Questo computer ha: %d, %v GB
chat-parity-machine-slow = Questo computer non ha una scheda grafica adatta: sarebbe lento
chat-parity-need-acp = Installare l'agente e accedere al tuo account
chat-parity-need-free = Niente.
chat-parity-need-local = Una scheda grafica con 6 GB o più e un download da 4 a 6 GB
chat-parity-need-own = L'indirizzo del server e una chiave
chat-parity-no-read = Non legge
chat-parity-no-work = Non funziona
chat-parity-none-found = Nessuno trovato
chat-parity-not-yet = Non ancora
chat-parity-only-net = Solo con un server sulla rete
chat-parity-optional = Facoltativo, lo accendi
chat-parity-pick = Scegli un'opzione
chat-parity-priv-ext = Le conversazioni vengono inviate a un host esterno (%h).
chat-parity-priv-local = Le tue conversazioni rimangono su questo computer.
chat-parity-priv-net = Le tue conversazioni vengono inviate al server della tua organizzazione (%h).
chat-parity-privacy = Privacy
chat-parity-q1 = Quello che scrive può uscire da questo computer?
chat-parity-q1-help = Pensate a informazioni riservate o personali.
chat-parity-q1-no = No, deve restare qui.
chat-parity-q1-no-desc = Solo il trattamento locale è accettabile.
chat-parity-q1-yes = Sì, può andare.
chat-parity-q1-yes-desc = Un servizio esterno potrebbe elaborarlo.
chat-parity-q2-no = No, o non lo so.
chat-parity-q2-no-desc = Nessun server aziendale o chiave disponibile.
chat-parity-q2-open = La sua organizzazione offre un server o una chiave AI?
chat-parity-q2-open-help = Verifica con l'area IT in caso di dubbi.
chat-parity-q2-secret = La sua organizzazione consente l'uso di AI esterna?
chat-parity-q2-secret-help = Alcune politiche in materia di dati vietano i servizi esterni.
chat-parity-q2-secret-no-desc = L'IA esterna non è consentita.
chat-parity-q2-secret-yes-desc = L'IA esterna è consentita.
chat-parity-q2-yes = Sì
chat-parity-q2-yes-desc = È disponibile un server o una chiave aziendale.
chat-parity-reads = Letture
chat-parity-ready = Pronto
chat-parity-rec-free = Iniziate con il modello gratuito: nessuna configurazione necessaria. Per questioni riservate, preferite su questo computer.
chat-parity-rec-local = Utilizzare su questo computer: nulla lascia qui e questa macchina ha una scheda grafica.
chat-parity-rec-local-slow = Questa macchina non ha una scheda grafica adatta per un modello locale. Usa il server della tua organizzazione all'interno della sua rete.
chat-parity-rec-local-tight = Un modello locale funzionerà, ma potrebbe essere lento con questa scheda grafica.
chat-parity-rec-own = Usa il server o la chiave della tua organizzazione: il contratto definisce costi e limiti.
chat-parity-rec-own-net = Utilizzare il server della vostra organizzazione all'interno della sua rete per mantenere i dati privati.
chat-parity-restart = Ricomincia
chat-parity-retry = Riprova
chat-parity-row-cost = Costo
chat-parity-row-docs = Documenti
chat-parity-row-needs = Hai bisogno di
chat-parity-row-ocr = Testo in immagini
chat-parity-row-offline = Senza internet
chat-parity-row-photos = Foto e grafici
chat-parity-row-privacy = Il tuo testo
chat-parity-row-web = Ricerca web
chat-parity-rule = Regola
chat-parity-show-me = Guarda come accenderlo
chat-parity-stay-here = Rimane su questo computer
chat-parity-step = Domanda %n di 2
chat-parity-sub-acp = Il tuo abbonamento agente
chat-parity-sub-free = Pronto all'uso
chat-parity-sub-local = Privato e offline
chat-parity-sub-own = La tua organizzazione è
chat-parity-suggested = Consigliato
chat-parity-system-memory = Memoria di sistema
chat-parity-verdict-bad-gpu = Nessuna scheda grafica adatta
chat-parity-verdict-bad-gpu-long = Non è stata trovata nessuna scheda grafica NVIDIA con CUDA. I modelli locali sarebbero eseguiti sul processore, molto più lentamente: una risposta può richiedere minuti. Per mantenere il testo all'interno della rete, preferire il server della vostra organizzazione.
chat-parity-verdict-capacity = Questa scheda esegue modelli che necessitano fino a %v della memoria video.
chat-parity-verdict-good = Comodo
chat-parity-verdict-low-vram = Carta con memoria insufficiente (%v)
chat-parity-verdict-low-vram-long = La scheda ha %v della memoria video; il modello locale più piccolo ha bisogno di %m.
chat-parity-verdict-mid = Stretto ma utilizzabile
chat-parity-verdict-none = Questo computer non può essere esaminato
chat-parity-video-memory = Memoria video
chat-parity-where = Scelga il motore
chat-parity-where-acp = Agente (ACP)
chat-parity-where-free = Modello libero
chat-parity-where-local = Su questo computer
chat-parity-where-own = Server o chiave propri
chat-parity-where-sub = Confronta i quattro modi per utilizzare l'assistente. Clicchi su uno per visualizzare i dettagli e attivarlo.
chat-parity-works = Lavori
chat-parity-workshop = Laboratorio assistente
chat-parity-workshop-lead = Segui dove e come pensa la tua assistente.
chat-parity-workshop-title = Laboratorio assistente
chat-parity-graphics-card = Scheda grafica

# --- #2921 missing keys added to every locale catalogue ---
chat-confirm-title = Conferma azione
chat-confirm-unavailable = Conferma non disponibile
chat-copy = Copi
chat-delete = Elimina
chat-delete-message-confirm = Eliminare questo messaggio dalla conversazione?
chat-delete-title = Elimina messaggio
chat-edit = Modifica
chat-edit-unavailable = Compositore non disponibile
chat-message-actions = Azioni messaggio
chat-regenerate = Rigenerare
chat-regenerate-no-source = Nessun messaggio utente precedente da rigenerare
chat-regenerate-unavailable = Non è possibile rigenerare questo turno
chat-voice-capture-silent = Nessun audio catturato
chat-voice-duplex-audio = Audio
chat-voice-duplex-closed = Sessione vocale chiusa
chat-voice-duplex-connect = Connettere voce
chat-voice-duplex-connected = Voce connessa
chat-voice-duplex-disabled = Voce disabilitata
chat-voice-duplex-error = Errore di voce
chat-voice-duplex-fallback = Passaggio al testo
chat-voice-duplex-live = Voce dal vivo
chat-voice-duplex-lost = Connessione persa
chat-voice-duplex-timeout = Tempo di sessione vocale
chat-voice-duplex-unavailable = Voce non disponibile
chat-voice-mic-failed = Microfono non disponibile
chat-web-title = Ricerca web
voice-mode-connecting = Connessione in corso...
voice-mode-fallback-idle = Inattivo
voice-mode-fallback-listening = Ascolto
voice-mode-fallback-speaking = Parlante
voice-mode-fallback-thinking = Pensare
voice-mode-live-listening = Ascolto
voice-mode-live-ready = Pronto
voice-mode-live-speaking = Parlante

# gbgov assistant parity: busy-send and actionable engine refusals (#2563)
chat-busy-send-head = La risposta è ancora in corso di stesura
chat-busy-send-text = Aspetta che la risposta finisca, o fermala e invia subito il tuo messaggio.
chat-busy-send-wait = Continua ad aspettare
chat-busy-send-stop = Ferma e invia
chat-engine-error-not-linked = Il tuo account %label non è connesso. Scegli un altro modello o collega l'account in Impostazioni.
chat-engine-error-switch-off = %label richiede che l'interruttore Agente sia attivato in questa conversazione.
chat-engine-error-unavailable = Gli agenti di abbonamento non sono disponibili su questo computer. Scegli un altro modello.
chat-engine-error-model-unavailable = Il modello %model non è disponibile su %label. Scegli un altro modello.
chat-engine-error-workspace-failed = Impossibile avviare la macchina virtuale dell'agente. Controllarne lo stato e riprovare.
chat-engine-error-account-model-off = Il modello %model non è abilitato per %label. Scegli un altro modello.
chat-engine-error-free-unavailable = Il modello gratuito non è disponibile in questo momento. Provare più tardi o scegliere un altro modello.
chat-engine-error-limit-reached = Hai raggiunto il limite di utilizzo del piano su %label. %when Scegli un altro modello.
chat-engine-error-unsafe-version = Questa versione di %label (%version) è stata bloccata perché non ha chiesto il permesso prima di agire. Scegliere un altro modello.
chat-engine-error-choose-model = Scelga un altro modello
chat-engine-error-back-at = Ritorna %quando.
