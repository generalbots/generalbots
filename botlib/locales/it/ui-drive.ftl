# =============================================================================
# General Bots — en — Drive and files
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Drive / Files
# -----------------------------------------------------------------------------
drive-title = Unità
drive-upload = Carica
drive-new-folder = Nuova cartella
drive-empty = Ancora nessun file. Carica qualcosa!
drive-search = Cerca nei file...
drive-sort-name = Nome
drive-sort-date = Data
drive-sort-size = Dimensione
drive-sort-type = Tipo
drive-view-grid = Vista griglia
drive-view-list = Vista elenco
drive-selected = { $count ->
    [one] Elemento { $count } selezionato
    *[other] Elementi { $count } selezionati
    }
drive-file-size = { $size ->
    [bytes] { $value } B
    [kb] { $value } KB
    [mb] { $value } MB
    [gb] { $value } GB
    *[other] Byte { $value }
    }
drive-drop-files = Rilasci i file qui per caricare

# Drive translations
drive-my-drive = My Drive
drive-shared = Condiviso con me
drive-recent = Recenti
drive-starred = Stellato
drive-trash = Cestino
drive-loading-storage = Caricamento magazzino...
drive-storage-used = { $used } di { $total } utilizzato
drive-empty-folder = Questa cartella è vuota

# Drive → Use in chat (#2568)
drive-use-in-chat = Usa nella chat
drive-use-in-chat-preparing = Preparazione di %n...
drive-use-in-chat-attached = Allegato alla conversazione: %n
drive-use-in-chat-failed = Non è stato possibile allegare il file alla conversazione.
drive-use-in-chat-timeout = Aprire la chat per allegare il file.
# Share intake (PWA share target)
drive-share-intake-title = Salva i file condivisi su disco
drive-share-intake-intro = Rivedere gli elementi condivisi qui sotto, quindi salvarli sul disco. Potrebbe essere necessario accedere prima di salvare.
drive-share-intake-heading = Elementi condivisi
drive-share-intake-loading = Caricamento degli elementi condivisi...
drive-share-intake-auth-hint = Se il salvataggio fallisce, accedi a General Bots e riprova. I file condivisi vengono memorizzati temporaneamente in questo browser fino a quando non vengono salvati o cancellati.
drive-share-intake-save = Salva su disco
drive-share-intake-open-drive = Apri unità
drive-share-intake-no-shared = Non sono stati trovati file condivisi. Puoi ancora aprire Drive e caricare manualmente.
drive-share-intake-no-files = Non sono stati condivisi file.
drive-share-intake-text-shown = Testo/link condiviso mostrato sopra.
drive-share-intake-unavailable = non disponibile
drive-share-intake-read-error = Impossibile leggere gli elementi condivisi.
drive-share-intake-read-failed = Impossibile leggere i file condivisi. Aprire l'unità per caricare manualmente.
drive-share-intake-sign-in-required = Accedi per salvare i file su Drive.
drive-share-intake-uploading = Caricamento
drive-share-intake-saved = File salvati su Drive.
drive-share-intake-saved-partial = Alcuni file non possono essere salvati. È possibile riprovare.

# semantic i18n key (#2542)
drive-my-files = I miei file

# semantic i18n key (#2542)
drive-loading-files = Caricamento file...

# semantic i18n key (#2542)
drive-drop-hint = Rilasci file o cartelle qui per caricare

# semantic i18n key (#2542)
drive-search-all-placeholder = Cerca tutti i file...
drive-search-all-aria-label = Cerca tutti i file per nome e contenuto

# -----------------------------------------------------------------------------
# #2328 — loading / empty / error state copy
# -----------------------------------------------------------------------------
drive-starred-empty-title = Nessun file stellato
drive-starred-empty-desc = Star qualsiasi file, o utilizzare il suo menu Hoppenstedt, per tenerlo a portata di click.
drive-shared-empty-title = Nessun file condiviso
drive-shared-empty-desc = I file condivisi con te appaiono qui. Condividi uno dei tuoi per iniziare una recensione.
drive-list-error-title = Impossibile caricare questo elenco
drive-list-error-desc = Drive non ha risposto in tempo. Controlli la connessione e riprovi.
