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
