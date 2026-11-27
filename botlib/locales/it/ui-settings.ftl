# =============================================================================
# General Bots — en — Settings
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Settings
# -----------------------------------------------------------------------------
settings-title = Impostazioni
settings-general = Generale
settings-account = Account
settings-notifications = Notifiche
settings-privacy = Privacy
settings-security = Sicurezza
settings-language = Lingua
settings-theme = Tema
settings-theme-light = Luce
settings-theme-dark = Scuro
settings-theme-system = Sistema
settings-save = Salva modifiche
settings-saved = Impostazioni salvate con successo
settings-create-api-key = Crea chiave API
settings-api-key-name = Nome chiave
settings-api-key-scopes = Campo di applicazione (separato da un comitato)
settings-api-key-scopes-hint = Lascia vuoto per l'ambito predefinito "utente."
settings-api-key-expiry = Scade tra (giorni)
settings-api-key-expiry-hint = Lasciare vuoto senza scadenza.
settings-add-webhook = Aggiungi webhook
settings-webhook-url = URL di Webhook
settings-webhook-events = Eventi (separati da virgola)
settings-webhook-events-hint = Lascia vuoto per ricevere tutti gli eventi.
settings-timezone = Fuso orario
settings-date-format = Formato data
settings-time-format = Formato orario

# -----------------------------------------------------------------------------
# Settings - Language & Localization
# -----------------------------------------------------------------------------
settings-language-desc = Scegliere la lingua preferita
settings-display-language = Lingua di visualizzazione
settings-language-affects = Riguarda tutto il testo nell'applicazione
settings-date-format-desc = Come vengono visualizzate le date
settings-time-format-desc = 12 ore o 24 ore
settings-language-changed = Lingua cambiata con successo
settings-reload-required = Page reload obbligatorio per applicare le modifiche

# Settings - Profile
settings-profile-desc = Gestisci le tue informazioni personali e le tue preferenze
settings-profile-photo = Foto profilo
settings-profile-photo-desc = La tua foto del profilo è visibile ad altri utenti
settings-upload-photo = Carica foto
settings-remove-photo = Rimuovi
settings-basic-info = Informazioni di base
settings-display-name = Nome visualizzato
settings-username = Nome utente
settings-email-address = Indirizzo email
settings-bio = Biografia
settings-bio-placeholder = Parlaci di te...
settings-contact-info = Informazioni sul contatto
settings-phone-number = Numero di telefono
settings-location = Ubicazione
settings-website = Sito web

# Settings - Security
settings-security-desc = Proteggi il tuo account con una maggiore sicurezza
settings-change-password = Modifica password
settings-change-password-desc = Aggiornare la password regolarmente per una migliore sicurezza
settings-current-password = Password corrente
settings-new-password = Nuova password
settings-confirm-password = Conferma nuova password
settings-update-password = Aggiorna password
settings-2fa = Autenticazione a due fattori
settings-2fa-desc = Aggiungi un livello di sicurezza aggiuntivo al tuo account
settings-authenticator-app = App di autenticazione
settings-authenticator-desc = Utilizzare un'app autenticatrice per codici 2FA
settings-enable-2fa = Abilita 2FA
settings-disable-2fa = Disabilita 2FA
settings-active-sessions = Sessioni attive
settings-active-sessions-desc = Gestisci le sessioni di login attive
settings-this-device = Questo dispositivo
settings-terminate-session = Termina
settings-terminate-all = Terminare tutte le altre sessioni

# Settings - Appearance
settings-appearance = Aspetto
settings-appearance-desc = Personalizzi l'aspetto dell'applicazione
settings-theme-selection = Tema
settings-theme-selection-desc = Scegliere il tema di colore preferito
settings-theme-blue = Blu
settings-theme-purple = Viola
settings-theme-green = Verde
settings-theme-orange = Arancione
settings-layout-preferences = Preferenze di disposizione
settings-compact-mode = Modalità compatta
settings-compact-mode-desc = Ridurre la spaziatura per più contenuti
settings-show-sidebar = Mostra barra laterale
settings-show-sidebar-desc = Mostra sempre la barra laterale di navigazione
settings-animations = Animazioni
settings-animations-desc = Abilita animazioni e transizioni dell'interfaccia utente

# Settings - Notifications
settings-notifications-title = Notifiche
settings-notifications-desc = Controlli come riceve le notifiche.
settings-email-notifications = Notifiche email
settings-direct-messages = Messaggi diretti
settings-direct-messages-desc = Ricevi email per nuovi messaggi diretti
settings-mentions = Menzioni
settings-mentions-desc = Ricevi email quando qualcuno ti menziona
settings-weekly-digest = Scavo settimanale
settings-weekly-digest-desc = Ottieni un riepilogo settimanale dell'attività
settings-marketing = Commercializzazione
settings-marketing-desc = Ricevi aggiornamenti di novità e prodotti
settings-push-notifications = Notifiche push
settings-enable-push = Abilita notifiche push
settings-enable-push-desc = Ricevi notifiche push del browser
settings-notification-sound = Suono
settings-notification-sound-desc = Riproduce il suono per le notifiche
settings-in-app-notifications = Notifiche in-app

# Settings - Storage
settings-storage = Conservazione
settings-storage-desc = Gestisci l'utilizzo dello storage
settings-storage-usage = Uso di archiviazione
settings-storage-used = { $used } di { $total } utilizzato
settings-storage-upgrade = Aggiornamento storage

# Settings - Privacy
settings-privacy-title = Privacy
settings-privacy-desc = Controlli le impostazioni sulla privacy.
settings-data-collection = Raccolta dati
settings-analytics = Analisi
settings-analytics-desc = Aiutaci a migliorare inviando dati di utilizzo anonimi
settings-crash-reports = Rapporti di arresto anomalo
settings-crash-reports-desc = Invia automaticamente i report di arresto anomalo
settings-download-data = Scarica i suoi dati
settings-download-data-desc = Ottieni una copia di tutti i tuoi dati
settings-delete-account = Elimina account
settings-delete-account-desc = Elimina definitivamente il tuo account e tutti i dati
settings-delete-account-warning = Questa azione non può essere annullata

# Settings - Billing
settings-billing = Fatturazione
settings-billing-desc = Gestisci il tuo abbonamento e i metodi di pagamento
settings-current-plan = Piano corrente
settings-free-plan = Piano libero
settings-pro-plan = Piano pro
settings-enterprise-plan = Piano Enterprise
settings-upgrade-plan = Piano di aggiornamento
settings-payment-methods = Metodi di pagamento
settings-add-payment = Aggiungi metodo di pagamento
settings-billing-history = Cronologia fatturazione

# -----------------------------------------------------------------------------
# OAuth Account Linking (Settings)
# -----------------------------------------------------------------------------
oauth-connected-accounts = Account connessi
oauth-connect = Connettere
oauth-unlink = Slegare
oauth-not-connected = Non connesso
oauth-linked = Collegato
oauth-no-accounts = Nessun account collegato.
oauth-loading = Caricamento account collegati...

## Payment cards (Stripe SetupIntent)
cards-title = & Carte di pagamento
cards-saved = Carte salvate
cards-hint = Le carte sono conservate in modo sicuro dal nostro provider di pagamento. I numeri delle carte non raggiungono mai i nostri server.
cards-add = Aggiungi carta
cards-add-first = Aggiungi la tua prima scheda
cards-none = Nessuna carta ancora salvata
cards-empty-hint = Aggiungi una carta per abilitare la fatturazione automatica e pagamenti più rapidi. Sarà reindirizzato al nostro provider di pagamento sicuro per inserire i dettagli della carta.
cards-default = Predefinito
cards-set-default = Imposta impostazione predefinita
cards-default-btn = Carta predefinita
cards-remove = Rimuovi
cards-remove-confirm = Rimuovere questa carta?
cards-expires = Scade
cards-load-error = Non è stato possibile caricare le schede salvate.
cards-add-error = Impossibile aggiungere una carta
cards-default-error = Impossibile aggiornare il predefinito
cards-remove-error = Impossibile rimuovere la carta
cards-default-updated = Carta predefinita aggiornata
cards-removed = Carta rimossa

# Settings → Model (#2568)
mdl-title = Modello
mdl-subtitle = Dove pensa il bot. Scegli un modello che gira su questa macchina, il tuo fornitore di endpoint, o il modello libero ospitato.
mdl-current = Modello corrente
mdl-where = Dove il bot pensa
mdl-local = Su questo computer
mdl-own = Il tuo server o la chiave del provider
mdl-free = Modello ospitato gratuito
mdl-active = In uso
mdl-endpoint = URL endpoint
mdl-model = Modello
mdl-key = chiave API
mdl-key-stored = chiave memorizzata
mdl-test = Prova
mdl-save = Salva
mdl-test-ok = Raggiungibile, latenza %ms ms
mdl-test-fail = Non riuscito: %e
mdl-testing = Prova...
mdl-saving = Salvataggio...
mdl-saved = Salvato. Prende effetto dal prossimo messaggio.
mdl-swapped = Il nuovo modello è ora attivo.
mdl-restart = Salvato. Riavviare l'app per utilizzare il nuovo modello.
mdl-admin-only = Solo l'amministratore può cambiare il modello.
mdl-loading = Caricamento in corso...
mdl-local-active = Modello locale attivo: %m
mdl-local-hint = Scegli un modello nella pagina AI On-Device, quindi può essere utilizzato qui.
mdl-local-open = Apri AI in servizio
mdl-free-note = Tutto ciò che Lei scrive e allega viene inviato a %h.
mdl-inconsistent = La configurazione del modello è incoerente.
mdl-restore-free = Ripristina modello libero
mdl-confirm-external = Quello che scrivi e alleghi verrà inviato a %h, fuori da questo computer. Continua?
mdl-scope-machine = Su questo computer
mdl-scope-network = Server della tua organizzazione (%h)
mdl-scope-external = Un host esterno (%h)
mdl-unavailable = Questo server non espone le API del modello.
# Settings → Model — provider picker and per-key models (#2568)
mdl-prov-intro = 选择提供商。地址会自动填入；您只需选择模型并粘贴密钥。
mdl-prov-search = 搜索提供商
mdl-prov-connect = 连接
mdl-prov-none = Nessun fornitore con quel nome.
mdl-prov-org-title = 您的组织服务器
mdl-prov-org-desc = 任何与 OpenAI API 兼容的服务器（vLLM、Ollama、LM Studio、Azure…）
mdl-prov-add = 添加
mdl-prov-back = 所有提供商
mdl-prov-address = •
mdl-prov-auto = 自动
mdl-prov-url-ph = https://server.example/v1/chat/completions
mdl-prov-model-list = 模型列表
mdl-prov-other = Hoppenstedt...
mdl-prov-other-ph = 准确的模型名称
mdl-prov-get-key = 获取密钥
mdl-prov-key-ph = 在此粘贴密钥
mdl-prov-key-opt = 如果服务器不需要密钥，请留空。
mdl-prov-key-again = 已存储密钥。请重新粘贴密钥以保存。
mdl-prov-sent = 您编写和附加的内容将发送到 %h（此计算机之外）。
mdl-prov-said = 提供商返回：
mdl-prov-test = 测试连接
mdl-prov-use = 使用此提供商
mdl-prov-url-needed = 请输入服务器地址。
mdl-prov-model-needed = 请选择模型。
mdl-prov-key-needed = API
mdl-err-key-refused = 密钥被拒绝
mdl-err-provider-refused = 提供商拒绝了请求：请检查密钥和模型
mdl-err-not-found = 找不到模型或地址
mdl-err-limit = 已达到使用限额
mdl-err-no-answer = 服务器未响应
mdl-err-timeout = 服务器响应超时
mdl-prov-desc-openai = GPT
mdl-prov-desc-xai = Grok
mdl-prov-desc-kimi = Kimi
mdl-prov-desc-zhipu = GLM
mdl-prov-desc-hunyuan = 模型 Hunyuan
mdl-prov-desc-minimax = 模型 MiniMax
mdl-prov-desc-anthropic = Claude Opus, Sonnet e Haiku
mdl-prov-desc-gemini = • Google AI Studio • Gemini
mdl-prov-desc-meta = Meta API Llama
mdl-prov-desc-mistral = Mistral Large ... Piccolo (...)
mdl-prov-desc-deepseek = DeepSeek Chat Reasoner
mdl-prov-desc-qwen = • Model Studio • Qwen
mdl-prov-desc-openrouter = 一个密钥即可使用数百个模型
mdl-prov-desc-groq = 响应极快的开放模型
mdl-km-title = 此密钥的模型
mdl-km-fetch = 获取模型
mdl-km-hint = 获取此密钥可用的模型，并开启要在聊天菜单中显示的模型。
mdl-km-unavailable = 此服务器不支持模型发现功能。
mdl-km-no-match = Nessun modello con quel nome.
mdl-km-filter = 筛选模型
mdl-km-count = %n di %t attivati
mdl-km-save = Salva modelli
mdl-km-fetch-fail = 无法获取模型（%e）。
mdl-km-empty = 提供商未返回任何模型。
mdl-km-fetching = Hoppenstedt...
mdl-km-saving = Hoppenstedt...
mdl-km-saved = 模型已保存。已开启的模型将显示在聊天菜单中。
mdl-km-save-fail = 无法保存（%e）。
mdl-km-stored = 密钥字段为空时，搜索将使用已存储的密钥。

# Settings → Instructions, Shortcuts, Text size (#2568)
instr-title = Istruzioni personalizzate
instr-subtitle = Di' all'assistente come vuoi che risponda.
instr-enabled = Abilita istruzioni personalizzate
instr-enabled-desc = Quando è fuori, l'assistente risponde senza queste istruzioni.
instr-about = Cosa dovrebbe sapere l'assistente di te?
instr-about-placeholder = Ruolo, lingua, contesto, preferenze...
instr-response = Come dovrebbe rispondere l'assistente?
instr-response-placeholder = Tono, livello di dettaglio, formato...
instr-save = Salva
instr-saving = Salvataggio...
instr-saved = Salvato.
instr-save-failed = Impossibile salvare: %e
instr-loading = Caricamento in corso...
instr-load-failed = Non è stato possibile caricare: %e
instr-unavailable = Questo server non espone istruzioni personalizzate.
sc-subtitle = Premere Shift + ? in qualsiasi punto della suite per aprire questa lista.
sc-group-general = Generale
sc-group-drive = Unità
sc-note = Le scorciatoie vengono ignorate mentre si digita in un campo, quindi non interrompono mai un messaggio o un modulo.
ts-title = Dimensione testo
ts-desc = Scelga come viene visualizzato il testo di grandi dimensioni in tutta la suite
ts-small = Piccolo
ts-default = Predefinito
ts-large = Grande
ts-xlarge = Extra grande
# Settings → Appearance: chat background pattern
chatbg-title = Sfondo della chat
chatbg-desc = Scegliere il modello dietro le conversazioni
chatbg-none = Nessuno
chatbg-image = Carta
chatbg-dots = Punti
chatbg-microdots = Micropuntini
chatbg-grid = Griglia
chatbg-blueprint = Progetto
chatbg-weave = Tessitura
chatbg-mosaic = Mosaico

# semantic i18n key (#2542)
settings-network-participation-desc = Controlli se questa installazione partecipa alla rete peer.

# semantic i18n key (#2542)
settings-p2p-network = Rete P2P

# semantic i18n key (#2542)
settings-network-mode-label = Modalità rete

# semantic i18n key (#2542)
settings-network-mode-p2p = P2P (raccomandato)
settings-network-p2p-description = La modalità offline mantiene ogni connessione su <code>127.0.0.1</code> e i link condivisi funzionano solo sulla rete locale.<br />P2P consente funzionalità future come il relè libero di collegamento condiviso e la scoperta peer. Nessun dato lascia questa macchina fino a quando non condividi esplicitamente qualcosa.
settings-network-p2p-restart-hint = Questa preferenza viene salvata localmente e ha effetto dopo il riavvio dell'app. L'endpoint del relè è fisso (<code id="network-relay-host">peer.generalbots.org</code>); la risoluzione peer e home-site lo utilizza sempre, quindi non è richiesta alcuna configurazione manuale.

# semantic i18n key (#2542)
settings-role-viewer = Visualizzatore

# semantic i18n key (#2542)
settings-register-dns-hostname = Registrare il nome host DNS

# semantic i18n key (#2542)
settings-dns-aaaa-ipv6 = AAAA (IPv6)

# semantic i18n key (#2542)
settings-dns-cname = CNAME

# semantic i18n key (#2542)
settings-create-token = Crei un token

# semantic i18n key (#2542)
settings-system-preview-desc = Modalità di anteprima e informazioni sulla versione per questa distribuzione

# semantic i18n key (#2542)
settings-system-preview-surface-desc = Controlla quanta superficie viene rivelata, non cosa è installato.

# semantic i18n key (#2542)
settings-system-preview-toggle-desc = Rivela le app in anteprima e l'albero delle impostazioni complete. Off mantiene solo le impostazioni essenziali per questa edizione.

# semantic i18n key (#2542)
settings-component-versions = Versioni del componente

# semantic i18n key (#2542)
settings-component-versions-desc = Versioni di BotServer, BotUI e dei servizi per questa installazione

# semantic i18n key (#2542)
settings-open-versions = Apri versioni

# semantic i18n key (#2542)
settings-on-device-ai = IA in servizio

# semantic i18n key (#2542)
settings-operating-system = Sistema operativo

# semantic i18n key (#2542)
settings-ledger = Ledger

# semantic i18n key (#2542)
settings-updates = Aggiornamenti

# semantic i18n key (#2542)
settings-this-workspace = Questo spazio di lavoro

# semantic i18n key (#2542)
settings-loading-identity = Caricamento identità...

# semantic i18n key (#2542)
settings-plan-placeholder = Piano:

# semantic i18n key (#2542)
settings-preview-placeholder = Anteprima:

# semantic i18n key (#2542)
settings-oda-subtitle = Modelli che funzionano su questa macchina. Scaricare un modello che si adatta all'hardware, attivarlo, e il motore locale lo utilizza al suo prossimo avvio. Non vengono utilizzati i tasti API qui.

# semantic i18n key (#2542)
settings-oda-detecting-hardware = Rilevamento hardware in corso…

# semantic i18n key (#2542)
settings-oda-recommended-for-machine = Consigliato per questa macchina

# semantic i18n key (#2542)
settings-oda-activate = Attiva

# semantic i18n key (#2542)
settings-oda-models = Modelli

# semantic i18n key (#2542)
settings-oda-model = Modello

# semantic i18n key (#2542)
settings-oda-params = Parami

# semantic i18n key (#2542)
settings-oda-quant = Quant

# semantic i18n key (#2542)
settings-oda-min-vram = Min VRAM

# semantic i18n key (#2542)
settings-oda-loading-models = Caricamento modelli...

# semantic i18n key (#2542)
settings-date-format-mm-dd-yyyy-us = MM/GG/AAAA (US)

# semantic i18n key (#2542)
settings-date-format-dd-mm-yyyy-eu-br = DD/MM/AAAA (EU/BR)

# semantic i18n key (#2542)
settings-date-format-yyyy-mm-dd-iso = AAAA-MM-GG (ISO)

# semantic i18n key (#2542)
settings-timezone-america-new-york = New York (GMT-5)

# semantic i18n key (#2542)
settings-timezone-america-los-angeles = Los Angeles (GMT-8)

# semantic i18n key (#2542)
settings-timezone-europe-london = Londra (GMT+0)

# semantic i18n key (#2542)
settings-timezone-europe-paris = Parigi (GMT+1)

# semantic i18n key (#2542)
settings-timezone-asia-tokyo = Tokyo (GMT+9)

# semantic i18n key (#2542)
settings-desktop-wallpaper = Sfondo desktop

# semantic i18n key (#2542)
settings-desktop-wallpaper-desc = Scelga lo sfondo visualizzato sul desktop

# semantic i18n key (#2542)
settings-organizations-bots = Organizzazioni e Bots

# semantic i18n key (#2542)
settings-organizations-bots-desc = Gestisci le tue organizzazioni e la gerarchia dei bot

# semantic i18n key (#2542)
settings-current-organization = Organizzazione corrente

# semantic i18n key (#2542)
settings-current-organization-desc = Seleziona l'organizzazione da gestire

# semantic i18n key (#2542)
settings-loading-organizations = Caricamento delle organizzazioni...

# semantic i18n key (#2542)
settings-bot-tree = Albero del bot

# semantic i18n key (#2542)
settings-bot-tree-desc = Bot appartenenti all'organizzazione selezionata

# semantic i18n key (#2542)
settings-loading-bots = Caricamento bot...

# semantic i18n key (#2542)
settings-storage-sync-desc = Gestisci le impostazioni di archiviazione e sincronizzazione

# semantic i18n key (#2542)
settings-knowledge = Conoscenza

# semantic i18n key (#2542)
settings-drive-files-as-context = Utilizzare tutti i file di unità come contesto di conoscenza

# semantic i18n key (#2542)
settings-drive-files-as-context-desc = Quando abilitata (predefinitamente), i nomi dei file recenti di unità viaggiano con ogni turno di chat in modo che l'assistente possa usarli come contesto. Spegnere per tenere Drive fuori dal prompt.

# semantic i18n key (#2542)
settings-loading-storage-information = Caricamento delle informazioni di memorizzazione...

# semantic i18n key (#2542)
settings-connected-cloud-storage = Archiviazione cloud connessa

# semantic i18n key (#2542)
settings-loading-storage-connections = Caricamento connessioni di archiviazione...

# semantic i18n key (#2542)
settings-os-subtitle = Impostazioni e strumenti di host a basso livello per questa installazione shell

# semantic i18n key (#2542)
settings-os-host-identity-desc = Identità host, kernel e fuso orario

# semantic i18n key (#2542)
settings-os-kernel = Kernel

# semantic i18n key (#2542)
settings-os-architecture = Architettura

# semantic i18n key (#2542)
settings-os-host-readonly-hint = Nome host e fuso orario sono in sola lettura in questa fase. Utilizzare l'app Terminal o gli strumenti host nell'immagine del chiosco per modificarli.

# semantic i18n key (#2542)
settings-os-services-desc = Avviare, fermare e riavviare i servizi host

# semantic i18n key (#2542)
settings-os-load-services-hint = Aprire il pannello per caricare i servizi.

# semantic i18n key (#2542)
settings-os-disk-usage-desc = Utilizzo disco per i filesystem montati

# semantic i18n key (#2542)
settings-os-load-storage-hint = Aprire il pannello per caricare lo stoccaggio.

# semantic i18n key (#2542)
settings-os-log-files = File di registro

# semantic i18n key (#2542)
settings-os-log-files-desc = File di questa installazione scrive, con posizione e dimensione

# semantic i18n key (#2542)
settings-os-load-log-files-hint = Aprire il pannello per caricare i file di registro.

# semantic i18n key (#2542)
settings-os-tail-log-desc = Tail le ultime 200 linee di un registro di servizio

# semantic i18n key (#2542)
settings-os-reload = Ricarica

# semantic i18n key (#2542)
settings-os-restart-service-title = Riavvia servizio

# semantic i18n key (#2542)
settings-os-the-service = il servizio

# semantic i18n key (#2542)
settings-os-service-name = Nome del servizio

# semantic i18n key (#2542)
settings-os-restart-action = Riavvia

# semantic i18n key (#2542)
settings-os-restart-confirm-message = Riavviare <strong id="os-restart-service-name" data-i18n="settings-os-the-service">il servizio</strong>? Digitare il nome del servizio da confermare; il riavvio è scritto nel registro di audit.

# semantic i18n key (#2542)
settings-profile-directory-managed = Gestito dalla directory Zitadel

# semantic i18n key (#2542)
settings-profile-open-directory-profile = Apri profilo directory

# semantic i18n key (#2542)
settings-profile-public-title = Profilo pubblico

# semantic i18n key (#2542)
settings-profile-public-desc = Come appari sulla home page del Generale Bots

# semantic i18n key (#2542)
settings-profile-public-display-name = Nome pubblico della visualizzazione

# semantic i18n key (#2542)
settings-profile-public-url-label = Il tuo profilo URL sarà

# semantic i18n key (#2542)
settings-profile-coming-soon = Presto disponibile

# semantic i18n key (#2542)
settings-profile-public-bio = Biografia pubblica

# semantic i18n key (#2542)
settings-profile-public-url-placeholder = il tuo nome.generalbots.org

# semantic i18n key (#2542)
settings-profile-language-region = Lingua e regione

# semantic i18n key (#2542)
settings-profile-public-url-hint-html = La pagina sarà in diretta a <strong><span id="public-profile-url-hint" data-i18n="settings-profile-public-url-placeholder">your-name.generalbots.org</span></strong> once the General Bots home-site relè ships (#1867).

# semantic i18n key (#2542)
settings-security-directory-note = Identità, sessioni e 2FA dal vivo nella directory Zitadel

# semantic i18n key (#2542)
settings-security-open-directory-security = Apri la sicurezza delle directory

# semantic i18n key (#2542)
settings-security-connected-devices = Dispositivi connessi

# semantic i18n key (#2542)
settings-security-connected-devices-desc = Dispositivi che hanno effettuato l'accesso al Suo account.

# semantic i18n key (#2542)
settings-security-local-data = Dati locali

# semantic i18n key (#2542)
settings-security-local-data-desc = Questa installazione mantiene tutto su questo dispositivo.

# semantic i18n key (#2542)
settings-security-export-local-data = Esporta dati locali

# semantic i18n key (#2542)
settings-security-export-local-data-desc = Scarica i dati memorizzati su questo dispositivo

# semantic i18n key (#2542)
settings-security-clear-local-cache = Svuota cache locale

# semantic i18n key (#2542)
settings-security-clear-local-cache-desc = Rimuove le impostazioni e le bozze memorizzate in questo profilo del browser

# semantic i18n key (#2542)
settings-security-clear-cache-action = Cancellare cache

# semantic i18n key (#2542)
settings-shared-links-subtitle = Scelga come un link a un file pubblico raggiunge la persona a cui lo invia

# semantic i18n key (#2542)
settings-shared-links-loopback-warning-title = Solo questa macchina è raggiungibile in questo momento.

# semantic i18n key (#2542)
settings-shared-links-loopback-warning-body-html = Un link loopback copiato (<code>127.0.0.1</code>) si apre sul computer del destinatario, non sul suo. Usi un indirizzo LAN, imposti un dominio o attenda l'indirizzo gratuito di General Bots prima di condividere un link al di fuori di questo computer.

# semantic i18n key (#2542)
settings-shared-links-lan-title = Questa rete

# semantic i18n key (#2542)
settings-shared-links-lan-desc = Chiunque si trovi sulla rete Wi-Fi o cablata

# semantic i18n key (#2542)
settings-shared-links-on-this-computer = Su questo computer

# semantic i18n key (#2542)
settings-shared-links-on-this-network = Su questa rete

# semantic i18n key (#2542)
settings-shared-links-lan-detail = Lavora per chiunque si trovi sulla stessa rete, niente lascia questa macchina.

# semantic i18n key (#2542)
settings-shared-links-lan-config-title = Configurazione

# semantic i18n key (#2542)
settings-shared-links-lan-config-body-html = Questo livello è attivo per impostazione predefinita: il server locale ascolta su tutte le interfacce. Aprire l'indirizzo di rete da un altro dispositivo sulla stessa rete. Se il traffico da dispositivo a dispositivo è bloccato, consentire la porta TCP in entrata <code id="shared-links-port">8080</code> su questo computer.

# semantic i18n key (#2542)
settings-shared-links-lan-config-local-html = Se il nome <code>.local</code> non risolve, utilizzare invece l'indirizzo IPv4 di questo computer (ad esempio <code>http://192.168.1.20:8080</code>).

# semantic i18n key (#2542)
settings-shared-links-domain-title = Il mio dominio

# semantic i18n key (#2542)
settings-shared-links-domain-desc = Un nome che controlla, raggiungibile da Internet

# semantic i18n key (#2542)
settings-shared-links-public-link = Collegamento pubblico

# semantic i18n key (#2542)
settings-shared-links-domain-not-configured = Non configurato I nuovi collegamenti

# semantic i18n key (#2542)
settings-shared-links-domain-config-title = Come configurare

# semantic i18n key (#2542)
settings-shared-links-domain-config-dns-html = Puntare un record DNS (per esempio <code>files.example.com</code>) all'indirizzo pubblico di questo computer.

# semantic i18n key (#2542)
settings-shared-links-domain-config-url-html = Impostare l'URL di base pubblica per questa installazione: <pre><code>PUCLIC_BASE_URL=https://files.example.com</code></pre> oppure, nel file prodotto: <pre><code>public_base_url=https://files.example.com</code></pre>

# semantic i18n key (#2542)
settings-shared-links-domain-config-restart = Riavviare l'app desktop in modo che i nuovi collegamenti di condivisione siano composti contro il dominio; i collegamenti esistenti mantengono i loro token.

# semantic i18n key (#2542)
settings-shared-links-domain-config-hint = Il token registrar rimane nel vault locale e non è mai scritto sui log. I certificati DNS automatici e TLS fanno parte del lavoro di domain-provider tracciato in questo numero; fino a quando non atterra, mantenere il record aggiornato al tuo registrar.

# semantic i18n key (#2542)
settings-shared-links-free-title = Indirizzo generale gratuito dei bot

# semantic i18n key (#2542)
settings-shared-links-free-desc = Un nome fornito da General Bots, nessun dominio necessario

# semantic i18n key (#2542)
settings-shared-links-free-unavailable-html = Non ancora disponibile. Il desktop funziona offline, quindi questo livello ha bisogno di una zona DNS gratuita e di un relè per macchine che non possono essere raggiunte direttamente. Spedisce disabilitato con il lavoro <code>botpublic</code> tracciato nel # 1864.

# semantic i18n key (#2542)
settings-shared-links-free-what-title = Cosa farà

# semantic i18n key (#2542)
settings-shared-links-free-what-body-html = L'account gratuito contiene un nome ® non i tuoi file. Quando una macchina non può essere raggiunta direttamente, il relè gestito (<code id="shared-links-relay-host">peer.generalbots.org</code>) conserva solo una copia cifrata del file condiviso e quella copia scade. Non è richiesta alcuna configurazione manuale del relè.

# semantic i18n key (#2542)
settings-shared-links-peer-title = Interruttore di peer principale

# semantic i18n key (#2542)
settings-shared-links-peer-desc-html = Interrompere tutto il traffico P2P per questa installazione. La modalità offline mantiene ogni connessione su <code>127.0.0.1</code>.

# semantic i18n key (#2542)
settings-shared-links-offline-default = Fuori rete (predefinito)

# semantic i18n key (#2542)
settings-shared-links-p2p-opt-in = P2P (opt-in)

# semantic i18n key (#2542)
settings-shared-links-relay-hint-html = L'endpoint del relè è fisso (<code>peer.generalbots.org</code>). Cambiare questa impostazione può richiedere il riavvio dell'app per avere pieno effetto.

# semantic i18n key (#2542)
settings-shared-links-check-again = Verifica di nuovo

# semantic i18n key (#2542)
settings-admin-bots-loading = Caricamento bot...

# semantic i18n key (#2542)
settings-admin-domain-configuration = Configurazione dominio

# semantic i18n key (#2542)
settings-admin-audit-loading = Caricamento registro audit...

# semantic i18n key (#2542)
settings-updates-subtitle = Verifica l'ultima release di GitHub e vai al download ufficiale

# semantic i18n key (#2542)
settings-updates-desktop-title = Aggiornamenti desktop

# semantic i18n key (#2542)
settings-updates-desktop-checks-html = L'app controlla <code>github.com/generalbots/generalbots/releases/latest</code>. L'installazione è sempre opt-in.

# semantic i18n key (#2542)
settings-updates-check-startup = Verifica aggiornamenti all'avvio

# semantic i18n key (#2542)
settings-updates-check-startup-desc = Esegui al massimo una volta al giorno e mostra solo una notifica. È ancora possibile utilizzare

# semantic i18n key (#2542)
settings-updates-current-version = Versione corrente

# semantic i18n key (#2542)
settings-updates-never = Mai

# semantic i18n key (#2542)
settings-updates-check-now = Verifica ora

# semantic i18n key (#2542)
settings-updates-view-release-notes = Visualizza note di rilascio

# semantic i18n key (#2542)
settings-updates-not-checked-yet = Non ancora controllato.

# semantic i18n key (#2542)
settings-versions-subtitle = Versioni del componente e disponibilità aggiornamenti

# semantic i18n key (#2542)
settings-versions-loading = Caricamento delle informazioni sulla versione...

# semantic i18n key (#2542)
settings-ledger-description = Ogni azione del terminale e del browser, con la decisione di consenso, l'esito e il percorso dello screenshot.

# semantic i18n key (#2542)
settings-ledger-actions-title = Registro azioni

# semantic i18n key (#2542)
settings-ledger-tab-terminal = Terminale

# semantic i18n key (#2542)
settings-ledger-tab-browser = Navigatore

# semantic i18n key (#2542)
settings-ledger-tab-budgets = Budget

# semantic i18n key (#2542)
settings-ledger-tab-ttft = TTFT

# semantic i18n key (#2542)
settings-integrations-subtitle = Connettere servizi esterni e gestire l'accesso API

# semantic i18n key (#2542)
settings-open-integrations = Apri integrazioni

# semantic i18n key (#2542)
settings-personal-access-tokens = Token di accesso personale

# semantic i18n key (#2542)
settings-personal-tokens-desc = Ti permette di chiamare l'API come te stesso (<code>gb_...</code>).Le chiavi del fornitore di modelli sono presenti nell'app Integrations.

# semantic i18n key (#2542)
settings-no-api-keys = Nessuna chiave API ancora creata

# semantic i18n key (#2542)
settings-webhooks-title = Agganci web

# semantic i18n key (#2542)
settings-no-webhooks = Nessun webhook configurato

# semantic i18n key (#2542)
settings-full-catalog = Catalogo completo

# semantic i18n key (#2542)
settings-loading-mcp-servers = Caricamento server MCP

# semantic i18n key (#2542)
settings-agent-skills = Competenze agente

# semantic i18n key (#2542)
settings-import-skill = Importa abilità

# semantic i18n key (#2542)
settings-loading-skills = Capacità di carico

# semantic i18n key (#2542)
settings-ai-accounts-subtitle = Le sottoscrizioni ai piani Claude Code e ChatGPT sono per utente. Le credenziali rimangono nella VM dell'agente o nel caveau locale; solo lo stato e i metadati di visualizzazione vengono memorizzati qui.

# semantic i18n key (#2542)
settings-subscription-accounts = Conti di abbonamento

# semantic i18n key (#2542)
settings-ai-accounts-desc = Connettere un account, quindi assegnarlo a un tipo di agente Vibe

# semantic i18n key (#2542)
settings-loading-ai-accounts = Caricamento account AI...

# semantic i18n key (#2542)
settings-vibe-subtitle = Assegni un provider LLM a ciascun tipo di agente Vibe. I provider sono definiti in Integrazioni → Provider modello (BYOK) o nello store Cloud.

# semantic i18n key (#2542)
settings-agent-llm-providers = Provider LLM agente

# semantic i18n key (#2542)
settings-agent-llm-providers-desc = Ogni tipo di agente instrada le proprie esecuzioni al fornitore selezionato qui per questo ramo.

# semantic i18n key (#2542)
settings-loading-agent-providers = Caricatore di agenti...

# semantic i18n key (#2542)
settings-privacy-subtitle = Gestisci le impostazioni dei tuoi dati e della tua privacy

# semantic i18n key (#2542)
settings-privacy-preferences = Preferenze sulla privacy

# semantic i18n key (#2542)
settings-profile-visibility = Visibilità del profilo

# semantic i18n key (#2542)
settings-profile-visibility-desc = Chi può vedere il tuo profilo

# semantic i18n key (#2542)
settings-contacts-only = Solo contatti

# semantic i18n key (#2542)
settings-online-status = Stato in linea

# semantic i18n key (#2542)
settings-online-status-desc = Mostra quando sei online

# semantic i18n key (#2542)
settings-read-receipts = Leggi ricevute

# semantic i18n key (#2542)
settings-read-receipts-desc = Fai sapere agli altri quando hai letto i messaggi

# semantic i18n key (#2542)
settings-your-data = I tuoi dati

# semantic i18n key (#2542)
settings-export-data = Esporta dati

# semantic i18n key (#2542)
settings-export-data-desc = Scarica tutti i suoi dati in un formato portatile

# semantic i18n key (#2542)
settings-delete-account-title = Elimina definitivamente il tuo account e tutti i dati associati

# semantic i18n key (#2542)
settings-delete-my-account = Elimina il mio account

# semantic i18n key (#2542)
settings-memory-subtitle = Quello che l'assistente ricorda di te attraverso le conversazioni

# semantic i18n key (#2542)
settings-remembered-facts = Fatti ricordati

# semantic i18n key (#2542)
settings-memory-facts-desc = Questi fatti vengono inviati con ogni messaggio in modo da risposte rimanere coerenti tra le conversazioni. Rimuovere tutto ciò che non si desidera più l'assistente da utilizzare.

# semantic i18n key (#2542)
settings-memory-all-scopes = Tutti gli ambiti

# semantic i18n key (#2542)
settings-memory-global-all-bots = Globale (tutti i bot)

# semantic i18n key (#2542)
settings-memory-bot-specific = Specifico del bot

# semantic i18n key (#2542)
settings-default-scope = predefinito

# semantic i18n key (#2542)
settings-manage-memories = Gestisci le memorie

# semantic i18n key (#2542)
settings-manage-memories-desc = I ricordi sono creati dall' assistente. Modificali o rimuovili qui.

# semantic i18n key (#2542)
settings-no-plan-info = Non è possibile gestire informazioni di piano da questo spiegamento.

# semantic i18n key (#2542)
settings-manage-plan-on-cloud = Gestisci il piano su Cloud

# semantic i18n key (#2542)
settings-add-method = Aggiungi metodo

# semantic i18n key (#2542)
settings-no-payment-method = Nessun metodo di pagamento sul file.

# semantic i18n key (#2542)
settings-add-method-on-cloud = Aggiungi metodo sul cloud

# semantic i18n key (#2542)
settings-no-invoices = Nessuna fattura per questo conto.

# semantic i18n key (#2542)
settings-view-invoices-on-cloud = Visualizza le fatture su Cloud

# semantic i18n key (#2542)
settings-skills-placeholder = 
    Paste a SKILL.md package:
    ---
    name: deploy-check
    description: Verify a deploy before announcing it
    ---
    1. …
settings-memory-search-placeholder = Cerca nei ricordi...
settings-notification-recipient-placeholder = you@example.com

# semantic i18n key (#2542)
settings-profile-placeholder-display-name = John Doe
settings-profile-placeholder-username = johndoe
settings-profile-placeholder-email = john@example.com
settings-profile-placeholder-public-display-name = Il tuo nome
settings-profile-placeholder-public-bio = Una breve descrizione visibile sul suo profilo pubblico...
settings-profile-placeholder-phone = +1 (555) 000-0000
settings-profile-placeholder-location = Città, Paese
settings-profile-placeholder-website = https://example.com

# semantic i18n key (#2542)
settings-placeholder-dns-hostname = mybot.example.com
settings-placeholder-dns-target = 192.168.1.1 o target.domain.com
settings-placeholder-api-key-name = Chiave di produzione
settings-placeholder-api-key-scopes = utente, amministratore
settings-placeholder-api-key-expiry = 90
settings-placeholder-webhook-url = https://example.com/hooks/gb
settings-placeholder-webhook-events = user.created, order. completed
os-update-channel = Canale
os-update-size = Dimensioni del download
os-update-usb = Installa dalla chiavetta USB
os-update-slot = Slot attivo: {slot} · canale: {channel} · rollback: {rollback}
weba-title = Costruttore di siti
weba-subtitle = Pubblicare pagine con moduli al CRM
weba-site-name = Nome del sito
weba-slug = Slug
weba-create = Crei un sito
weba-refresh = Aggiorna
weba-domain = Dominio personalizzato
weba-domain-save = Imposta dominio
weba-campaign = Link campagna
weba-campaign-save = Crei un link
weba-publish = Pubblica
weba-unpublish = Non pubblicare
weba-preview = Anteprima
weba-domain-pending = Verifica dominio in attesa:
weba-campaign-created = Link campagna:
weba-created = Sito creato:
weba-published = Sito pubblicato.
weba-unpublished = Sito inedito.
weba-error = Non è stato possibile caricare i siti:
theme-high-contrast = Alto contrasto

# Shell switching (assistant / windowed suite)
settings-shell-assistant = Modalità assistente
settings-shell-assistant-desc = Torna al chat-first assistente shell. Le tue conversazioni rimangono qui.
settings-shell-assistant-open = Assistente aperto

# Settings → Usage & limits (.ftl keys added for the Settings port)
eng-short-local = Locale
eng-short-free = Libero
eng-short-own-key = Chiave propria
eng-short-acp = Abbonamento
eng-group-local = Modelli locali
eng-group-free = Modello libero
eng-group-own-key = Chiavi proprie
eng-group-acp = Abbonamenti
eng-detail-acp = Agente aggiunto
settings-limits = & Limiti di utilizzo
lim-title = & Limiti di utilizzo
lim-subtitle = Quanto ogni motore può utilizzare.
lim-intro = Ogni tipo di motore ha un limite diverso: quota di abbonamento, spesa della tua chiave, limite del servizio gratuito o capacità di questo computer.
lim-engine-types = Tipi di motore
lim-refresh = Aggiorna
lim-refresh-tip = Leggi di nuovo i limiti ora
lim-updated = Aggiornato al
lim-refreshing = Rinfrescante...
lim-refresh-fail = Impossibile aggiornare ora.
lim-loading = Limiti di caricamento...
lim-unavailable = Questo server non espone i limiti di utilizzo.
lim-load-fail = Non è stato possibile caricare i limiti: %e
lim-no-data = Questo motore non ha ancora segnalato limiti.
lim-none-acp = L'agente non ha segnalato la quota di piano.
lim-none-key = Il provider non segnala la spesa della chiave. Controlla il cruscotto del provider.
lim-none-free = Il servizio gratuito fissa la sua indennità e non la riferisce.
lim-none-local = Il modello locale non ha quote di utilizzo. Il limite è la capacità di questo computer.
lim-w-5h = finestra di 5 ore
lim-w-weekly = Limite settimanale
lim-w-daily = Limite giornaliero
lim-w-requests = Richieste
lim-w-tokens = Token
lim-w-input = Token di ingresso
lim-w-output = Token di uscita
lim-w-plan = Quota piano
lim-within = Entro il limite
lim-close = Vicino al limite
lim-full = Quota di esaurimento
lim-left = %n di %t rimanenti
lim-remaining = %n rimanenti
lim-limit = Limite %n
lim-renew = RENEWS %s
lim-renews-midnight = RENEWS A MIDNIGHT
lim-renews-today = RENEWS AT %s
