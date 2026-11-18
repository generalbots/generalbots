# =============================================================================
# General Bots — de — Drive and files
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Drive / Files
# -----------------------------------------------------------------------------
drive-title = Fahren
drive-upload = Hochladen
drive-new-folder = Neuer Ordner
drive-empty = Noch keine Dateien. Lade etwas hoch!
drive-search = Dateien durchsuchen...
drive-sort-name = Name
drive-sort-date = Datum
drive-sort-size = Größe
drive-sort-type = Typ
drive-view-grid = Rasteransicht
drive-view-list = Listenansicht
drive-selected = { $count ->
    [one] { $count } item selected
   *[other] { $count } items selected
}
drive-file-size = { $size ->
    [bytes] { $value } B
    [kb] { $value } KB
    [mb] { $value } MB
    [gb] { $value } GB
   *[other] { $value } bytes
}
drive-drop-files = Legen Sie Dateien hier ab, um sie hochzuladen

# Drive translations
drive-my-drive = Mein Laufwerk
drive-shared = Mit mir geteilt
drive-recent = Neu
drive-starred = Mit einem Stern versehen
drive-trash = Müll
drive-loading-storage = Speicher wird geladen...
drive-storage-used = { $used } von { $total } verwendet
drive-empty-folder = Dieser Ordner ist leer

# Drive → Use in chat (#2568)
drive-use-in-chat = In Unterhaltung verwenden
drive-use-in-chat-preparing = %n wird vorbereitet…
drive-use-in-chat-attached = An die Unterhaltung angehängt: %n
drive-use-in-chat-failed = Die Datei konnte nicht an die Unterhaltung angehängt werden.
drive-use-in-chat-timeout = Öffnen Sie die Unterhaltung, um die Datei anzuhängen.
# Freigabe-Eingang (PWA-Freigabeziel)
drive-share-intake-title = Geteilte Dateien in Drive speichern
drive-share-intake-intro = Überprüfen Sie die unten geteilten Elemente und speichern Sie sie in Ihrem Drive. Möglicherweise müssen Sie sich vor dem Speichern anmelden.
drive-share-intake-heading = Geteilte Elemente
drive-share-intake-loading = Geteilte Elemente werden geladen…
drive-share-intake-auth-hint = Wenn das Speichern fehlschlägt, melden Sie sich bei General Bots an und versuchen Sie es erneut. Geteilte Dateien werden in diesem Browser vorübergehend gespeichert, bis sie gespeichert oder gelöscht werden.
drive-share-intake-save = In Drive speichern
drive-share-intake-open-drive = Drive öffnen
drive-share-intake-no-shared = Keine geteilten Dateien gefunden. Sie können Drive trotzdem öffnen und manuell hochladen.
drive-share-intake-no-files = Es wurden keine Dateien geteilt.
drive-share-intake-text-shown = Geteilter Text/Link oben angezeigt.
drive-share-intake-unavailable = nicht verfügbar
drive-share-intake-read-error = Geteilte Elemente konnten nicht gelesen werden.
drive-share-intake-read-failed = Geteilte Dateien konnten nicht gelesen werden. Öffnen Sie Drive, um manuell hochzuladen.
drive-share-intake-sign-in-required = Anmeldung erforderlich, um Dateien in Drive zu speichern.
drive-share-intake-uploading = Wird hochgeladen
drive-share-intake-saved = Dateien in Drive gespeichert.
drive-share-intake-saved-partial = Einige Dateien konnten nicht gespeichert werden. Sie können es erneut versuchen.

# semantic i18n key (#2542)
drive-my-files = Meine Dateien

# semantic i18n key (#2542)
drive-loading-files = Dateien werden geladen...

# semantic i18n key (#2542)
drive-drop-hint = Dateien oder Ordner hier ablegen, um sie hochzuladen

# semantic i18n key (#2542)
drive-search-all-placeholder = Alle Dateien durchsuchen…
drive-search-all-aria-label = Alle Dateien nach Name und Inhalt durchsuchen
