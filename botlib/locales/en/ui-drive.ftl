# =============================================================================
# General Bots — en — Drive and files
#
# Split from ui.ftl (#2087). English is the source of truth: author keys in
# botlib/locales/en/ and never define the same key in two files.
# =============================================================================

# -----------------------------------------------------------------------------
# Drive / Files
# -----------------------------------------------------------------------------
drive-title = Drive
drive-upload = Upload
drive-new-folder = New Folder
drive-empty = No files yet. Upload something!
drive-search = Search files...
drive-sort-name = Name
drive-sort-date = Date
drive-sort-size = Size
drive-sort-type = Type
drive-view-grid = Grid View
drive-view-list = List View
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
drive-drop-files = Drop files here to upload

# Drive translations
drive-my-drive = My Drive
drive-shared = Shared with me
drive-recent = Recent
drive-starred = Starred
drive-trash = Trash
drive-loading-storage = Loading storage...
drive-storage-used = { $used } of { $total } used
drive-empty-folder = This folder is empty
