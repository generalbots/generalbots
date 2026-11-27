# General Bots - Error Messages (English)
# This file contains all error message translations

# =============================================================================
# HTTP Errors
# =============================================================================

error-http-400 = Richiesta non valida. Verifichi l'input.
error-http-401 = Autenticazione richiesta. Effettui il login.
error-http-403 = Non hai il permesso di accedere a questa risorsa.
error-http-404 = { $entity } non trovato.
error-http-409 = Conflitto: { $message }
error-http-429 = Troppe richieste. Attendere i secondi { $seconds }.
error-http-500 = Errore interno del server. Riprova più tardi.
error-http-502 = Gateway non valido. Il server ha ricevuto una risposta non valida.
error-http-503 = Servizio temporaneamente non disponibile. Riprova più tardi.
error-http-504 = Richiedi timed out dopo { $milliseconds }ms.

# =============================================================================
# Validation Errors
# =============================================================================

error-validation-required = { $field } è richiesto.
error-validation-email = Inserire un indirizzo email valido.
error-validation-url = Inserire un URL valido.
error-validation-phone = Inserire un numero di telefono valido.
error-validation-min-length = { $field } deve essere almeno { $min } caratteri.
error-validation-max-length = { $field } non deve essere più di { $max } caratteri.
error-validation-min-value = { $field } deve essere almeno { $min }.
error-validation-max-value = { $field } non deve essere più di { $max }.
error-validation-pattern = Formato { $field } non valido.
error-validation-unique = { $field } esiste già.
error-validation-mismatch = { $field } non corrisponde a { $other }.
error-validation-date-format = Inserire una data valida nel formato { $format }.
error-validation-date-past = { $field } deve essere nel passato.
error-validation-date-future = { $field } deve essere in futuro.

# =============================================================================
# Authentication Errors
# =============================================================================

error-auth-invalid-credentials = Email o password non valide.
error-auth-account-locked = Il tuo account è stato bloccato. Contatta il supporto.
error-auth-account-disabled = Il tuo account è stato disabilitato.
error-auth-session-expired = La tua sessione è scaduta. Effettua il login di nuovo.
error-auth-token-invalid = Token non valido o scaduto.
error-auth-token-missing = È richiesto un token di autenticazione.
error-auth-mfa-required = È richiesta l'autenticazione multi-fattore.
error-auth-mfa-invalid = Codice di verifica non valido.
error-auth-password-weak = La password è troppo debole. Usa una password più forte.
error-auth-password-expired = La password è scaduta. Reimpostarla.

# =============================================================================
# Configuration Errors
# =============================================================================

error-config = Errore di configurazione: { $message }
error-config-missing = Configurazione mancante: { $key }
error-config-invalid = Valore di configurazione non valido per { $key }: { $reason }
error-config-file-not-found = File di configurazione non trovato: { $path }
error-config-parse = Impossibile analizzare la configurazione: { $message }

# =============================================================================
# Database Errors
# =============================================================================

error-database = Errore del database: { $message }
error-database-connection = Impossibile connettersi al database.
error-database-timeout = Operazione del database scaduta.
error-database-constraint = Violazione di vincolo del database: { $constraint }
error-database-duplicate = Esiste già un record con questo { $field }.
error-database-migration = Migrazione del database non riuscita: { $message }

# =============================================================================
# File & Storage Errors
# =============================================================================

error-file-not-found = File non trovato: { $filename }
error-file-too-large = Il file è troppo grande. La dimensione massima è { $maxSize }.
error-file-type-not-allowed = Tipo di file non consentito. Tipi ammessi: { $allowedTypes }.
error-file-empty = Il file caricato è vuoto.
error-file-name-invalid = Il nome del file non è valido.
error-file-content-mismatch = Il contenuto del file non corrisponde al suo tipo ({ $expected } atteso, { $detected } trovato).
error-file-archive-unsafe = L'archivio è stato rifiutato: { $reason }.
error-file-extension-blocked = I file con estensione .{ $extension } non sono ammessi.
error-file-extension-unknown = L'estensione .{ $extension } non è riconosciuta.
error-file-upload-failed = Caricamento del file non riuscito: { $message }
error-file-read = Impossibile leggere il file: { $message }
error-file-write = Impossibile scrivere il file: { $message }
error-storage-full = Quota di stoccaggio superata.
error-storage-unavailable = Il servizio di stoccaggio non è disponibile.

# =============================================================================
# Network & External Service Errors
# =============================================================================

error-network = Errore di rete: { $message }
error-network-timeout = Connessione scaduta.
error-network-unreachable = Il server è irraggiungibile.
error-service-unavailable = Servizio non disponibile: { $service }
error-external-api = Errore API esterno: { $message }
error-rate-limit = Tariffa limitata. Riprova dopo { $seconds }s.

# =============================================================================
# Bot & Dialog Errors
# =============================================================================

error-bot-not-found = Bot non trovato: { $botId }
error-bot-disabled = Questo bot è attualmente disabilitato.
error-bot-script-error = Errore di script alla riga { $line }: { $message }
error-bot-timeout = Risposta del bot scaduta.
error-bot-quota-exceeded = Quota di utilizzo del bot superata.
error-dialog-not-found = Finestra di dialogo non trovata: { $dialogId }
error-dialog-invalid = Configurazione della finestra di dialogo non valida: { $message }

# =============================================================================
# LLM & AI Errors
# =============================================================================

error-llm-unavailable = Il servizio IA non è attualmente disponibile.
error-llm-timeout = La richiesta IA è scaduta.
error-llm-rate-limit = Limite di velocità IA superato. Attendere prima di riprovare.
error-llm-content-filter = Il contenuto è stato filtrato dalle linee guida di sicurezza.
error-llm-context-length = L'ingresso è troppo lungo. Vi preghiamo di abbreviare il vostro messaggio.
error-llm-invalid-response = Risposta non valida ricevuta dal servizio AI.
error-llm-empty-response = Scusa, non ho potuto elaborare il tuo messaggio ora. Per favore riprova tra pochi secondi.

# =============================================================================
# Email Errors
# =============================================================================

error-email-send-failed = Impossibile inviare l'email: { $message }
error-email-invalid-recipient = Indirizzo email del destinatario non valido: { $email }
error-email-attachment-failed = Impossibile allegare il file: { $filename }
error-email-template-not-found = Modello email non trovato: { $template }

# =============================================================================
# Calendar & Scheduling Errors
# =============================================================================

error-calendar-conflict = La slot time è in conflitto con l'evento esistente.
error-calendar-past-date = Non è possibile pianificare eventi nel passato.
error-calendar-invalid-recurrence = Modello di ricorrenza non valido.
error-calendar-event-not-found = Evento non trovato: { $eventId }

# =============================================================================
# Task Errors
# =============================================================================

error-task-not-found = Attività non trovata: { $taskId }
error-task-already-completed = L'attività è già stata completata.
error-task-circular-dependency = Dipendenza circolare rilevata nelle attività.
error-task-invalid-status = Transizione dello stato delle attività non valida.

# =============================================================================
# Permission Errors
# =============================================================================

error-permission-denied = Non hai il permesso di eseguire questa azione.
error-permission-resource = Non hai accesso a questo { $resource }.
error-permission-action = Non puoi { $action } questo { $resource }.
error-permission-owner-only = Solo il proprietario può eseguire questa azione.

# =============================================================================
# Generic Errors
# =============================================================================

error-internal = Errore interno: { $message }
error-unexpected = Si è verificato un errore imprevisto. La prego di riprovare.
error-not-implemented = Questa caratteristica non è ancora implementata.
error-maintenance = Il sistema è in manutenzione. Riprova più tardi.
error-unknown = Si è verificato un errore sconosciuto.

# =============================================================================
# Localized API contract (#2094) — stable codes, localized messages
# =============================================================================

error-locale-invalid = La lingua richiesta non è valida.
error-locale-unsupported = Questa lingua non è disponibile.
error-locale-persist-failed = Impossibile salvare la sua preferenza di lingua.
error-bot-id-required = L'ID del bot è richiesto.
error-bot-id-invalid = L'ID del bot non è valido.
error-bot-name-required = Il nome del bot è richiesto.
error-token-invalid-format = Il formato token non è valido.
error-token-signing-failed = Impossibile firmare il token.
error-token-signature-invalid = La firma token non è valida.
error-token-payload-invalid = Il carico token non è valido.
error-token-payload-json-invalid = Il carico token non è valido JSON.
error-cloud-token-required = Fornire un cloud_token.
error-llm-subscription-token-missing = Token per abbonamento mancante.
error-llm-token-store-unavailable = Il negozio di gettoni non è disponibile.
error-llm-subscription-token-expired = Il token di sottoscrizione è scaduto; invia il messaggio di nuovo.
error-llm-prompt-missing = Non c'è nessuna richiesta.
error-llm-account-unlinked = Connettere l'account Claude o ChatGPT in Impostazioni → Account AI prima.

# =============================================================================
# Localized API errors — routes, drive sync, peers, computer use (#2094)
# =============================================================================

error-agent-drive-database-unavailable = Il archivio dei file non è temporaneamente disponibile.
error-agent-drive-lookup-failed = Impossibile leggere la conversazione. Riprovare.
error-agent-drive-conversation-not-found = Conversazione non trovata.
error-agent-drive-bad-owner = Il record del proprietario della conversazione non è valido.
error-agent-drive-forbidden = Questa conversazione appartiene ad un altro utente.
error-agent-drive-bad-bot = Il record di bot per questa conversazione non è valido.
error-agent-drive-workspace-stopped = Lo spazio di lavoro dell'agente non funziona.
error-agent-drive-no-files = Non ci sono file nella cartella di output dello spazio di lavoro.
error-agent-drive-named-files-missing = I file richiesti non sono nella cartella di output dello spazio di lavoro.
error-agent-drive-save-failed = Impossibile salvare alcun file su Drive.
error-agent-drive-file-missing = Il file non è nella cartella di output dello spazio di lavoro.
error-agent-drive-save-one-failed = Impossibile salvare il file su Drive.
error-agent-drive-bad-path = Il percorso di trasmissione restituito dal negozio non è valido.
error-agent-drive-link-failed = Non è stato possibile creare il link di condivisione.
error-subagent-no-llm = Nessun provider LLM è configurato per i sub-agenti.
error-goal-run-not-running = Questa corsa goal non è in esecuzione.
error-goal-run-not-paused = Questa corsa obiettivo non è in pausa.
error-autotask-not-found = Attività automatica non trovata.
error-autotask-intent-required = Descriva cosa dovrebbe fare l'attività.
error-peer-network-disabled = La rete inter pares è disabilitata su questo nodo.
error-peer-network-unauthorized = È richiesto un token bearer peer valido.
error-peer-network-unconfigured = Questo nodo non ha token peer configurato, quindi gli endpoint peer sono chiusi.
error-peer-network-forbidden = È richiesta una sessione amministratore per questo controllo peer.
error-peer-network-invalid = La richiesta peer non è valida: { $message }
error-peer-network-unknown-node = nodo peer sconosciuto: { $message }
error-peer-network-registry-full = Il registro dei pari e' pieno.
error-peer-network-payload-store-full = Il payload store è pieno: { $message }
error-peer-network-recipient-required = Il parametro query del destinatario è richiesto.
error-drive-sync-field-required = { $field } è richiesto.
error-drive-sync-field-absolute = { $field } deve essere un percorso assoluto.
error-drive-sync-field-parent = { $field } non deve contenere '.'.
error-drive-sync-field-not-folder = { $field } non è una cartella su questa macchina: { $path }
error-drive-sync-field-local-folder = La cartella locale
error-drive-sync-field-drive-folder = La cartella Unità
error-drive-sync-job-not-found = Sincronizzazione lavoro { $id } non trovata.
error-drive-sync-job-name-required = È richiesto un nome per il processo di sincronizzazione.
error-drive-sync-job-exists = Esiste già un processo di sincronizzazione denominato { $name }.
error-drive-sync-conflict-path-required = È richiesto un percorso di conflitto.
error-drive-sync-conflict-path-escape = Il percorso di conflitto sfugge alla sua cartella di sincronizzazione.
error-drive-sync-task-failed = L'attività { $task } non è riuscita: { $error }
error-org-auth-service-unavailable = Il servizio directory non è disponibile.
error-org-default-delete = L'organizzazione predefinita non può essere eliminata.
error-org-settings-save-failed = Impossibile salvare le impostazioni dell'organizzazione.
error-org-name-required = Il nome dell'organizzazione è richiesto.
error-org-tenant-missing = Nessun inquilino esiste ancora
error-computer-use-session-required = È richiesto un ID di sessione.
error-computer-use-kind-invalid = Il tipo deve essere "browser" o "terminale."
error-computer-use-action-not-found = Azione non trovata.
error-computer-use-no-consent = Non vi è alcun consenso per questa azione; approvarlo prima.
error-computer-use-browser-only = Questa azione deve essere eseguita nel pannello del browser.
error-computer-use-url-missing = L'azione non ha URL.
error-terminal-approval-required = L'esecuzione del terminale richiede l'approvazione esplicita dell'utente.
error-agent-vm-disabled = Agente/VM disabilitato per questa conversazione. Attivare nei controlli della chat per utilizzare l'area di lavoro.
error-terminal-action-id-required = L'esecuzione del terminale richiede un id di azione approvato.
error-terminal-command-required = È richiesto un comando.
error-terminal-command-too-long = Il comando è limitato a 16000 caratteri.
error-terminal-blocked-pattern = Il comando corrisponde a un modello distruttivo bloccato.
error-terminal-workspace-stopped = Lo spazio di lavoro agente non è in esecuzione. Terminare il provisioning (o riavviare l'app desktop), quindi riprovare.
error-terminal-timeout = Il comando è scaduto.

# =============================================================================
# Collaboration, groups, autotasks and AI accounts (#2094)
# =============================================================================

error-db-unavailable = Il database non è disponibile in questo momento. Provare di nuovo.
error-collab-invalid-resource = Il tipo di risorsa o l'id risorsa non è valido.
error-collab-owner-only = Solo il proprietario può farlo.
error-collab-invalid-grantee-type = Il tipo di beneficiario deve essere "utente" o "dominio."
error-collab-invalid-role = Il ruolo non è valido.
error-collab-invalid-grantee-id = Il codice del beneficiario non è valido.
error-collab-grant-not-found = La sovvenzione non è stata trovata.
error-collab-invalid-new-owner = Il nuovo ID proprietario non è valido.
error-collab-link-not-found = Il link delle azioni non è stato trovato.
error-collab-comment-not-found = Il commento non è stato trovato.
error-collab-comment-resolve-forbidden = Il commento non è stato trovato, o non è tuo risolvere.
error-collab-comment-body-required = È richiesto il corpo del commento.
error-collab-comment-too-long = Il commento è limitato a 8000 caratteri.
error-collab-invalid-emoji = L'emoji non è valido.
error-collab-invalid-cursor = Il cursore "prima" non è valido.
error-collab-invalid-action = L'azione non è valida.
error-collab-invalid-name = Il nome non è valido.
error-collab-content-required = Il contenuto è obbligatorio.
error-collab-content-too-large = Il contenuto è troppo grande.
error-collab-version-not-found = La versione non è stata trovata.
error-collab-invalid-base-version = La versione base non è valida.
error-collab-invalid-parent = L'ID genitore non è valido.
error-collab-invalid-op-type = Il tipo di operazione non è valido.
error-collab-operation-not-found = L'operazione non è stata trovata.
error-collab-not-a-conflict = L'operazione non è un conflitto irrisolto.
error-collab-invalid-resolution = La risoluzione deve essere "accetto-server" o "accettare-cliente."
error-group-budget-not-found = Il bilancio del gruppo non è stato trovato.
error-group-budget-invalid-id = L'ID del bilancio non è valido.
error-group-budget-amount-invalid = L'importo deve essere pari a zero o positivo.
error-group-budget-caps-invalid = I tappi devono essere a zero o positivi (zero significa nessun limite).
error-autotask-fields-required = Titolo, calendario e comando sono richiesti.
error-autotask-kind-invalid = Il tipo deve essere "crono" o "sistemato."
error-autotask-schedule-invalid = Il programma deve essere un'espressione cron valida a cinque campi.
error-conversation-not-found = La conversazione non è stata trovata.
error-conversation-session-invalid = La sessione di conversazione non è valida.
error-subagent-invalid-tasks = Le attività del sub-agente non sono valide.
error-ai-account-unknown-provider = Il provider di account AI è sconosciuto.
error-ai-account-workspace-unavailable = Lo spazio di lavoro dell'agente non è disponibile in questa generazione.
error-ai-account-agent-invalid = L'agente deve essere ragionante, agente o veloce.
error-ai-account-not-connected-vibe = Connettere l'account prima di assegnarlo a Vibe.
error-ai-account-not-connected-chat = Connettere l'account prima di selezionarlo per la chat.
error-computer-use-action-id-mismatch = L' id azione non corrisponde al percorso.
error-http-no-route = Nessun percorso corrisponde a questo percorso.
error-goal-run-advance-refused = La corsa non può avanzare nel suo stato attuale.

# =============================================================================
# People, attendance and products routes (#2094)
# =============================================================================

error-person-not-found = Persona non trovata.
error-team-not-found = Squadra non trovata.
error-time-off-not-found = Richiesta di time-off non trovata.
error-people-invalid-start-date = La data di inizio non è valida. Usa il formato AAAA-MM-GG.
error-people-invalid-end-date = La data di fine non è valida. Usa il formato AAAA-MM-GG.
error-attendance-webhook-not-found = Webhook non trovato.
error-attendance-session-id-invalid = L'ID sessione non è valido.
error-attendance-session-not-found = Sessione non trovata.
error-attendance-attendant-id-required = È richiesto un ID addetto.
attendance-no-attendant-matching-skills = Nessun assistente è stato trovato con le competenze richieste.
attendance-response-sent = Risposta inviata.
error-product-not-found = Prodotto non trovato.
error-service-not-found = Servizio non trovato.
error-service-invalid-value = Il valore del servizio non è valido.
error-service-price-required = Questo servizio non ha prezzo; fornire un valore.

# Service-tax value validation (#2094)
error-invalid-value = Il valore fornito non può essere analizzato.
error-value-required = È richiesto un valore per questo servizio.

# Language model errors shown in the chat
error-model-unreachable = Impossibile raggiungere il modello linguistico. Verificare la connessione e riprovare.
error-model-auth = Il modello di lingua ha rifiutato le credenziali. Controlla la chiave in Impostazioni.
error-model-rate-limited = Il modello di linguaggio sta limitando le richieste in questo momento. Aspetta un momento e riprova.
error-model-error = Il modello di lingua ha restituito un errore. Provare di nuovo.
error-model-timeout = Il modello linguistico ci ha messo troppo a rispondere.
error-model-context = La conversazione è più lunga di quanto il modello possa leggere contemporaneamente. Avviare una nuova conversazione o accorciare il messaggio.

# Agent workspace start before a subscription turn
workspace-step-starting-vm = Avviare la macchina virtuale
workspace-step-starting-vm-detail = Il primo inizio può richiedere alcuni minuti.
error-workspace-start-timeout = La macchina virtuale non è iniziata in tempo. Continua a partire in background; riprovare in pochi minuti.
error-workspace-start-failed = Impossibile avviare la macchina virtuale. Provare di nuovo; se continua a fallire, controllare la macchina virtuale in Impostazioni.
error-workspace-service-restarting = Riavviare il servizio Virtual Machine...
error-workspace-service-failed = Il servizio Virtual Machine non ha risposto. Riavviare il computer o WSL.

# Chat attachments that could not be kept in Drive
error-attachment-storage-full = Il disco di questo computer è pieno: il file non è stato salvato. Liberare un po 'di spazio e collegarlo di nuovo.
error-attachment-not-stored = Il file non può essere salvato in Drive.
error-attachment-too-large = Il file supera il limite di 50 MB.
error-attachment-not-read = Il file non può essere memorizzato o letto.

# Chat answers given without the model
error-memory-not-updated = Non ho potuto aggiornare la tua memoria memorizzata.
memory-forgot-none = Niente nella tua memoria corrispondeva a questo, quindi niente e' stato rimosso.
memory-forgot-one = Fatto: ho rimosso 1 elemento dalla memoria.
memory-forgot-many = Fatto: ho rimosso { $count } elementi dalla memoria.
error-command-failed = Non potevo completare quell'azione. Provare di nuovo, o chiedere in altre parole.
error-tool-invalid-name = Nome strumento non valido: { $name }
error-tool-not-available = Lo strumento{ $name }

# Drive listing in the chat (the model reads its own English copy)
chat-drive-unavailable = Drive non è disponibile per quest'area di lavoro.
chat-drive-list-failed = Non è stato possibile elencare i file nel Suo Drive.
chat-drive-list-empty = Nessun file trovato nel tuo disco.
chat-drive-list-empty-query = Nessun file corrispondente a{ $query }Sono stati trovati nella tua unità.
chat-drive-list-heading = File nella tua unità (prima di tutto, fino a { $limit }):
chat-drive-list-heading-query = File nel tuo Drive corrispondenti a .{ $query } [49] (prima la più recente, fino a { $limit }):
chat-drive-open = Apri nell'unità
