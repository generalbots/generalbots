# Transactional email text (#2094). Subject and body only; MIME and envelope stay untouched.

email-greeting = Ciao { $name },
email-signoff = Cordiali saluti,
email-team = Squadra generale dei bot
email-welcome-subject = Benvenuti al General Bots Cloud{ $plan }Piano
email-welcome-body = Benvenuti su General Bots Cloud! Il vostro piano { $plan } è ora attivo. È possibile iniziare a creare bot e gestire il proprio spazio di lavoro a { $url }
email-invoice-subject = Fattura #{ $invoice }
email-invoice-body = È stata creata una nuova fattura. Fattura: { $invoice }. Importo: { $amount } { $currency }. Visualizzi i dettagli all'indirizzo { $url }
email-payment-success-subject = Pagamento confermato
email-payment-success-body = Il pagamento di { $amount } { $currency } è stato confermato. Piano: { $plan }. Grazie per il tuo abbonamento!
email-subscription-activated-subject = Abbonamento attivato
email-subscription-activated-body = Il tuo abbonamento { $plan } è ora attivo. Puoi gestirlo a { $url }
email-recurring-charge-subject = Fattura di ricarica ricorrente #{ $invoice }
email-recurring-charge-body = Il tuo abbonamento ricorrente è stato addebitato. Fattura: { $invoice }. Importo: { $amount } { $currency }. Vedi i dettagli a { $url }
email-payment-failed-subject = Pagamento non riuscito Azione richiesta
email-payment-failed-body = Il pagamento di { $amount } { $currency } non è riuscito. Fattura: { $invoice }. Aggiornare il metodo di pagamento a { $url }
email-subscription-cancelled-subject = Abbonamento annullato
email-subscription-cancelled-body = Il tuo abbonamento { $plan } è stato annullato. Se vuoi riattivarlo, visita { $url }
email-grace-warning-subject = Pagamento Sovraccambio
email-grace-warning-body = Il vostro pagamento è in ritardo. Il vostro servizio sarà sospeso in { $days } giorni. Si prega di aggiornare il metodo di pagamento immediatamente a { $url }
email-invitation-subject = Invito a partecipare { $org }
email-invitation-body = Sei stato invitato a partecipare a { $org }. Accetta il tuo invito a { $url }. Questo invito scade a { $expires }.
email-quota-limit-reached = Ha raggiunto il limite di quota { $metric }.
email-quota-threshold = Hai usato { $current }% della tua quota { $metric } (soglia: { $threshold }%).
email-metric-messages = messaggi
email-metric-storage = stoccaggio
email-metric-api-calls = chiamate API
email-metric-bots = bot
email-metric-users = utenti
email-metric-kb-documents = documenti di base sulla conoscenza
email-metric-apps = apps
email-metric-emails-per-day = email al giorno
email-metric-builds-per-day = costruisce al giorno
email-metric-uploads-per-hour = caricamenti all'ora

# Sharing and mention notifications (#2094)
email-share-subject = { $sender } condiviso "{ $file }" con te
email-share-download = Scarica { $file }
email-share-link-expires = Questo link scade su { $date }. Puoi revocarlo in qualsiasi momento.
email-share-link-no-expiry = Questo link non scade. Puoi revocarlo in qualsiasi momento.
email-mention-subject = { $author } ha menzionato in un commento
email-mention-body = { $author } ti ha menzionato in { $resource_type } ({ $resource_id }). "{ $preview }" Apri l'app per visualizzare e rispondere.

# semantic i18n key (#2542)
mail-new-message = Nuovo messaggio

# semantic i18n key (#2542)
mail-cc-bcc = CC/Bcc

# semantic i18n key (#2542)
mail-subject-label = Oggetto:

# semantic i18n key (#2542)
mail-track-opens = Traccia aperta

# semantic i18n key (#2542)
mail-request-read-receipt = Richiesta ricevuta di lettura

# semantic i18n key (#2542)
mail-priority-normal = Priorità normale

# semantic i18n key (#2542)
mail-priority-high = Alta priorità

# semantic i18n key (#2542)
mail-priority-low = Bassa priorità

# semantic i18n key (#2542)
mail-dialog-templates-title = Modelli email

# semantic i18n key (#2542)
mail-dialog-new-template = + Nuovo modello

# semantic i18n key (#2542)
mail-dialog-signatures-title = Firme email

# semantic i18n key (#2542)
mail-dialog-new-signature = + Nuova firma

# semantic i18n key (#2542)
mail-dialog-rules-title = Regole email

# semantic i18n key (#2542)
mail-dialog-new-rule = + Nuova regola

# semantic i18n key (#2542)
mail-dialog-automatic-replies-title = Risposte automatiche

# semantic i18n key (#2542)
mail-dialog-send-automatic-replies = Invia risposte automatiche

# semantic i18n key (#2542)
mail-dialog-start-date = Data di inizio

# semantic i18n key (#2542)
mail-dialog-end-date = Data di fine

# semantic i18n key (#2542)
mail-dialog-reply-only-org = Manda solo a persone della mia organizzazione

# semantic i18n key (#2542)
mail-dialog-add-account-title = Aggiungi account email

# semantic i18n key (#2542)
mail-dialog-account-name = Nome account

# semantic i18n key (#2542)
mail-dialog-imap-server = Server IMAP

# semantic i18n key (#2542)
mail-dialog-imap-port = Porta IMAP

# semantic i18n key (#2542)
mail-dialog-smtp-server = Server SMTP

# semantic i18n key (#2542)
mail-dialog-smtp-port = Porta SMTP

# semantic i18n key (#2542)
mail-dialog-use-ssl-tls = Usa SSL/TLS

# semantic i18n key (#2542)
mail-dialog-add-account-action = Aggiungi account

# semantic i18n key (#2542)
mail-dialog-schedule-send-title = Invia programma

# semantic i18n key (#2542)
mail-dialog-schedule-action = Schema

# semantic i18n key (#2542)
mail-manage-labels-title = Gestisci etichette

# semantic i18n key (#2542)
mail-label-name = Nome etichetta

# semantic i18n key (#2542)
mail-label-color = Colore

# semantic i18n key (#2542)
mail-new-signature-title = Nuova firma di posta elettronica

# semantic i18n key (#2542)
mail-signature-name = Nome della firma

# semantic i18n key (#2542)
mail-html-content = Contenuto HTML

# semantic i18n key (#2542)
mail-new-rule-title = Nuova regola di posta elettronica

# semantic i18n key (#2542)
mail-rule-name = Nome regola

# semantic i18n key (#2542)
mail-condition-sender-contains = Condizione (Mittente contiene)

# semantic i18n key (#2542)
mail-action-star = Stella

# semantic i18n key (#2542)
mail-new-template-title = Nuovo modello di email

# semantic i18n key (#2542)
mail-template-name = Nome modello

# semantic i18n key (#2542)
mail-body-html = Corpo HTML

# semantic i18n key (#2542)
mail-placeholder-label-name = Per esempio, lavoro
mail-placeholder-signature-name = Ad esempio, firma professionale
mail-placeholder-signature-content = Cordiali saluti,<br><b>Il Suo Nome</b>

# semantic i18n key (#2542)
mail-placeholder-rule-name = Ad esempio, Archivio Newsletter
mail-placeholder-rule-condition = Ad esempio, newsletter
mail-placeholder-template-name = Ad esempio, email di benvenuto
mail-placeholder-template-subject = Oggetto standard di benvenuto
mail-placeholder-template-body = Ciao {{name}}, benvenuti sulla nostra piattaforma!
mail-placeholder-autoreply-subject = Oggetto della risposta
mail-placeholder-autoreply-message = Il tuo messaggio di risposta automatico...

# semantic i18n key (#2542)
mail-placeholder-search-emails = Cerca nelle email...
mail-placeholder-recipients = Destinatari
mail-placeholder-carbon-copy = Copia carbone
mail-placeholder-blind-carbon-copy = Copia nascosta
mail-placeholder-subject = Oggetto
mail-placeholder-compose-message = Scrivi il tuo messaggio...
mail-placeholder-account-name = Lavoro, personale, ecc.

# semantic i18n key (#2542)
mail-placeholder-imap-server = imap.example.com
mail-placeholder-smtp-server = smtp.example.com
mail-placeholder-username-or-email = nome utente o email
mail-placeholder-password = ••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••••

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
mail-state-empty-title = Questa cartella è vuota
mail-state-empty-desc = Nuovi messaggi arrivano qui. Scrivetene uno voi stessi se avete bisogno di iniziare la conversazione.
mail-state-empty-action = Scrivi un messaggio
mail-state-loading = Caricamento email...
mail-templates-empty-title = Nessun modello ancora
mail-templates-empty-desc = Un modello salva un oggetto e un corpo che invia spesso, così non deve mai riscriverlo.
mail-templates-empty-action = Crei un modello
mail-signatures-empty-title = Ancora nessuna firma
mail-signatures-empty-desc = Una firma viene aggiunta ai messaggi che invia. Aggiunga una volta il suo nome, ruolo e i recapiti.
mail-signatures-empty-action = Crei una firma
mail-rules-empty-title = Ancora nessuna regola
mail-rules-empty-desc = Una regola archivia, contrassegna con stella o elimina automaticamente la posta in arrivo, ad esempio tutto da un determinato mittente.
mail-rules-empty-action = Crei una regola
